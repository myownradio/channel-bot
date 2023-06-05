use crate::services::playlist_processor::traits::{
    MetadataService, MusicSearchService, PlaylistProvider, RadioManager, TrackDownloader,
};
use crate::services::playlist_processor::types::{
    AudioMetadata, DownloadId, DownloadingStatus, MetadataServiceError, MusicSearchServiceError,
    PlaylistEntry, PlaylistProviderError, RadioManagerError, TopicId, TrackDownloadEntry,
    TrackDownloaderError,
};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, PartialEq, Default)]
pub(crate) struct AudioTrackProcessingData {
    metadata: AudioMetadata,
    tried_topics: Vec<TopicId>,
    current_download_id: Option<DownloadId>,
    path_to_audio_file: Option<String>,
    radioterio_track_id: Option<u64>,
    radioterio_channel_id: Option<u64>,
}

impl AudioTrackProcessingData {
    pub(crate) fn get_step(&self) -> AudioTrackProcessingStep {
        if self.radioterio_channel_id.is_some() {
            return AudioTrackProcessingStep::Finish;
        }

        if self.radioterio_track_id.is_some() {
            return AudioTrackProcessingStep::AddToChannel;
        }

        if self.path_to_audio_file.is_some() {
            return AudioTrackProcessingStep::Upload;
        }

        if self.current_download_id.is_some() {
            AudioTrackProcessingStep::Downloading;
        }

        AudioTrackProcessingStep::SearchAlbum
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum AudioTrackProcessingStep {
    SearchAlbum,
    Downloading,
    Upload,
    AddToChannel,
    Finish,
}

impl AudioTrackProcessingStep {
    pub(crate) fn is_finish(&self) -> bool {
        matches!(self, AudioTrackProcessingStep::Finish)
    }
}

#[derive(Default)]
pub(crate) struct PlaylistProcessingData {
    unfiltered_tracks: Option<Vec<PlaylistEntry>>,
    filtered_tracks: Option<Vec<PlaylistEntry>>,
    audio_tracks_data: Option<Vec<AudioTrackProcessingData>>,
}

impl PlaylistProcessingData {
    pub(crate) fn get_step(&self) -> PlaylistProcessingStep {
        if let Some(audio_tracks) = &self.audio_tracks_data {
            return if audio_tracks
                .iter()
                .map(AudioTrackProcessingData::get_step)
                .all(|step| step.is_finish())
            {
                PlaylistProcessingStep::Finish
            } else {
                PlaylistProcessingStep::DownloadingTracks
            };
        }

        if self.filtered_tracks.is_some() {
            return PlaylistProcessingStep::StartDownloadingTracks;
        }

        if self.unfiltered_tracks.is_some() {
            return PlaylistProcessingStep::FilterNewTracks;
        }

        PlaylistProcessingStep::DownloadPlaylist
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum PlaylistProcessingStep {
    DownloadPlaylist,
    FilterNewTracks,
    StartDownloadingTracks,
    DownloadingTracks,
    Finish,
}

impl PlaylistProcessingStep {
    pub(crate) fn is_final(&self) -> bool {
        matches!(self, PlaylistProcessingStep::Finish)
    }
}

pub(crate) struct PlaylistProcessor {
    track_downloader: Arc<dyn TrackDownloader>,
    playlist_provider: Arc<dyn PlaylistProvider>,
    radio_manager: Arc<dyn RadioManager>,
    metadata_service: Arc<dyn MetadataService>,
    search_service: Arc<dyn MusicSearchService>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PlaylistProcessingError {
    #[error(transparent)]
    PlaylistProviderError(#[from] PlaylistProviderError),
    #[error(transparent)]
    RadioManagerError(#[from] RadioManagerError),
    #[error(transparent)]
    TrackDownloaderError(#[from] TrackDownloaderError),
    #[error(transparent)]
    MetadataServiceError(#[from] MetadataServiceError),
    #[error(transparent)]
    MusicSearchServiceError(#[from] MusicSearchServiceError),
    #[error("Source playlist not found")]
    SourcePlaylistNotFound,
}

impl PlaylistProcessor {
    pub(crate) fn create(
        track_downloader: Arc<dyn TrackDownloader>,
        playlist_provider: Arc<dyn PlaylistProvider>,
        radio_manager: Arc<dyn RadioManager>,
        metadata_service: Arc<dyn MetadataService>,
        search_service: Arc<dyn MusicSearchService>,
    ) -> Self {
        Self {
            track_downloader,
            playlist_provider,
            radio_manager,
            metadata_service,
            search_service,
        }
    }

    pub(crate) async fn process_playlist(
        &self,
        user_id: &u64,
        src_playlist_id: &str,
        dst_playlist_id: &str,
        ctx: &mut PlaylistProcessingData,
    ) -> Result<(), PlaylistProcessingError> {
        let step = ctx.get_step();

        info!(
            user_id,
            src_playlist_id,
            dst_playlist_id,
            ?step,
            "Processing playlist"
        );

        match step {
            PlaylistProcessingStep::DownloadPlaylist => {
                info!("Downloading playlist...");
                match self.playlist_provider.get_playlist(src_playlist_id).await? {
                    Some(unfiltered_tracks) => {
                        ctx.unfiltered_tracks.replace(unfiltered_tracks);
                    }
                    None => {
                        return Err(PlaylistProcessingError::SourcePlaylistNotFound);
                    }
                };
            }
            PlaylistProcessingStep::FilterNewTracks => {
                info!("Filtering playlist tracks...");

                let filtered_tracks = match self.radio_manager.get_playlist(dst_playlist_id).await?
                {
                    Some(dst_tracks) => {
                        let dst_tracks_set = dst_tracks
                            .into_iter()
                            .map(|track| {
                                format!(
                                    "{}-{}-{}",
                                    track.metadata.artist,
                                    track.metadata.album,
                                    track.metadata.title
                                )
                            })
                            .collect::<HashSet<_>>();

                        ctx.unfiltered_tracks
                            .take()
                            .unwrap_or_default()
                            .iter()
                            .filter(move |track| {
                                let key = format!(
                                    "{}-{}-{}",
                                    track.metadata.artist,
                                    track.metadata.album,
                                    track.metadata.title
                                );
                                !dst_tracks_set.contains(&key)
                            })
                            .cloned()
                            .collect()
                    }
                    None => ctx.unfiltered_tracks.take().unwrap_or_default(),
                };

                ctx.filtered_tracks.replace(filtered_tracks);
            }
            PlaylistProcessingStep::StartDownloadingTracks => {
                info!("Initializing tracks download...");

                let tracks_data = ctx
                    .filtered_tracks
                    .take()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|track| AudioTrackProcessingData {
                        metadata: track.metadata,
                        tried_topics: vec![],
                        current_download_id: None,
                        path_to_audio_file: None,
                        radioterio_track_id: None,
                        radioterio_channel_id: None,
                    })
                    .collect();
                ctx.audio_tracks_data.replace(tracks_data);
            }
            PlaylistProcessingStep::DownloadingTracks => {
                info!("Downloading tracks...");

                if let Some(audio_tracks_data) = &mut ctx.audio_tracks_data {
                    for audio_track_data in audio_tracks_data.iter_mut() {
                        self.process_audio_track(audio_track_data).await?;
                    }
                }
            }
            PlaylistProcessingStep::Finish => {
                info!("Finished");
            }
        };

        Ok(())
    }

    async fn process_audio_track(
        &self,
        track_ctx: &mut AudioTrackProcessingData,
    ) -> Result<(), PlaylistProcessingError> {
        let step = track_ctx.get_step();

        match step {
            AudioTrackProcessingStep::SearchAlbum => {
                let album_query = format!(
                    "{} - {}",
                    track_ctx.metadata.artist, track_ctx.metadata.album
                );
                debug!(album_query, "Searching for album...");
                let maybe_result = self
                    .search_service
                    .search(&album_query)
                    .await?
                    .into_iter()
                    .find(|entry| !track_ctx.tried_topics.contains(&entry.topic_id));

                let result = match maybe_result {
                    Some(result) => result,
                    None => {
                        // TODO: Mark as "Not found"
                        return Ok(());
                    }
                };

                debug!("Getting download url...");

                let maybe_download_url = self
                    .search_service
                    .get_download_url(&result.topic_id)
                    .await?;

                let download_url = match maybe_download_url {
                    Some(download_url) => download_url,
                    None => {
                        // TODO: Mark as "Not found"
                        return Ok(());
                    }
                };

                debug!("Starting download...");

                let download_id = self
                    .track_downloader
                    .create_download("/tmp/downloads", download_url)
                    .await?;

                track_ctx.current_download_id.replace(download_id);
            }
            AudioTrackProcessingStep::Downloading => {
                if let Some(download_id) = &track_ctx.current_download_id {
                    let maybe_download = self.track_downloader.get_download(download_id).await?;
                    let download = match maybe_download {
                        Some(download) => download,
                        None => {
                            warn!("Download does not exist!");
                            track_ctx.current_download_id.take();
                            return Ok(());
                        }
                    };

                    if !matches!(download.status, DownloadingStatus::Finished) {
                        return Ok(());
                    }

                    debug!("Searching for the track in finished download...");

                    for file_path in download.files {
                        let maybe_metadata =
                            self.metadata_service.get_audio_metadata(&file_path).await?;

                        if let Some(metadata) = maybe_metadata {
                            if metadata.artist == track_ctx.metadata.artist
                                && metadata.title == track_ctx.metadata.title
                            {
                                track_ctx.path_to_audio_file.replace(file_path);
                                return Ok(());
                            }
                        }
                    }

                    info!("The current download appears to be missing the required audio track");

                    track_ctx.current_download_id.take();
                }
            }
            AudioTrackProcessingStep::Upload => {
                todo!()
            }
            AudioTrackProcessingStep::AddToChannel => {
                todo!()
            }
            AudioTrackProcessingStep::Finish => {
                debug!("Finished")
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::playlist_processor::types::{
        RadioManagerPlaylistEntry, SearchResultsEntry,
    };
    use async_trait::async_trait;

    struct TrackDownloaderMock;

    #[async_trait]
    impl TrackDownloader for TrackDownloaderMock {
        async fn create_download(
            &self,
            path_to_download: &str,
            url: Vec<u8>,
        ) -> Result<DownloadId, TrackDownloaderError> {
            Ok(DownloadId(String::from("DownloadingId")))
        }

        async fn get_download(
            &self,
            download_id: &DownloadId,
        ) -> Result<Option<TrackDownloadEntry>, TrackDownloaderError> {
            Ok(if download_id.0 == String::from("DownloadingId") {
                Some(TrackDownloadEntry {
                    id: download_id.clone(),
                    status: DownloadingStatus::Downloading,
                    files: vec![
                        String::from("path/to/downloading_file1.mp3"),
                        String::from("path/to/downloading_file2.mp3"),
                    ],
                })
            } else if download_id.0 == String::from("DownloadedId") {
                Some(TrackDownloadEntry {
                    id: download_id.clone(),
                    status: DownloadingStatus::Finished,
                    files: vec![
                        String::from("path/to/downloaded_file1.mp3"),
                        String::from("path/to/downloaded_file2.mp3"),
                    ],
                })
            } else {
                None
            })
        }

        async fn delete_download(
            &self,
            download_id: &DownloadId,
        ) -> Result<(), TrackDownloaderError> {
            Ok(())
        }
    }

    struct PlaylistProviderMock;

    #[async_trait]
    impl PlaylistProvider for PlaylistProviderMock {
        async fn get_playlist(
            &self,
            playlist_id: &str,
        ) -> Result<Option<Vec<PlaylistEntry>>, PlaylistProviderError> {
            if playlist_id == "ExistingPlaylistId" {
                Ok(Some(vec![
                    PlaylistEntry {
                        metadata: AudioMetadata {
                            title: String::from("Track Title 1"),
                            artist: String::from("Track Artist 1"),
                            album: String::from("Track Album 1"),
                        },
                    },
                    PlaylistEntry {
                        metadata: AudioMetadata {
                            title: String::from("Track Title 2"),
                            artist: String::from("Track Artist 2"),
                            album: String::from("Track Album 2"),
                        },
                    },
                    PlaylistEntry {
                        metadata: AudioMetadata {
                            title: String::from("Track Title 3"),
                            artist: String::from("Track Artist 3"),
                            album: String::from("Track Album 3"),
                        },
                    },
                ]))
            } else {
                Ok(None)
            }
        }
    }

    struct RadioManagerMock;

    #[async_trait]
    impl RadioManager for RadioManagerMock {
        async fn get_playlist(
            &self,
            playlist_id: &str,
        ) -> Result<Option<Vec<RadioManagerPlaylistEntry>>, RadioManagerError> {
            if playlist_id == "ExistingPlaylistId" {
                Ok(Some(vec![RadioManagerPlaylistEntry {
                    id: String::from("entry1"),
                    metadata: AudioMetadata {
                        title: String::from("Track Title 2"),
                        artist: String::from("Track Artist 2"),
                        album: String::from("Track Album 2"),
                    },
                }]))
            } else {
                Ok(None)
            }
        }

        async fn add_track_to_playlist(
            &self,
            playlist_id: &str,
            file_path: &str,
        ) -> Result<(), RadioManagerError> {
            todo!()
        }
    }

    struct MetadataServiceMock;

    #[async_trait]
    impl MetadataService for MetadataServiceMock {
        async fn get_audio_metadata(
            &self,
            file_path: &str,
        ) -> Result<Option<AudioMetadata>, MetadataServiceError> {
            todo!()
        }
    }

    struct MusicSearchServiceMock;

    #[async_trait]
    impl MusicSearchService for MusicSearchServiceMock {
        async fn search(
            &self,
            query: &str,
        ) -> Result<Vec<SearchResultsEntry>, MusicSearchServiceError> {
            Ok(match query {
                "Track Artist 3 - Track Album 3" => vec![
                    SearchResultsEntry {
                        title: String::from("Track Artist 3 - Track Album 3"),
                        topic_id: TopicId(String::from("Track Artist 3 - Track Album 3 [MP3]")),
                        tracks_hint: vec![],
                    },
                    SearchResultsEntry {
                        title: String::from("Track Artist 3 - Track Album 3"),
                        topic_id: TopicId(String::from("Track Artist 3 - Track Album 3 [123123]")),
                        tracks_hint: vec![],
                    },
                ],
                "Track Artist 1 - Track Album 1" => vec![SearchResultsEntry {
                    title: String::from("Track Artist 1 - Track Album 1"),
                    topic_id: TopicId(String::from("Track Artist 1 - Track Album 1")),
                    tracks_hint: vec![],
                }],
                "Track Artist 2" => vec![
                    SearchResultsEntry {
                        title: String::from("Track Artist 2 Discography [MP3]"),
                        topic_id: TopicId(String::from("Track Artist 2 Discography [MP3]")),
                        tracks_hint: vec![],
                    },
                    SearchResultsEntry {
                        title: String::from("Track Artist 2 Discography [FLAC]"),
                        topic_id: TopicId(String::from("Track Artist 2 Discography [FLAC]")),
                        tracks_hint: vec![],
                    },
                ],
                _ => vec![],
            })
        }

        async fn get_download_url(
            &self,
            topic_id: &TopicId,
        ) -> Result<Option<Vec<u8>>, MusicSearchServiceError> {
            Ok(match topic_id.0.as_str() {
                "Track Artist 3 - Track Album 3 [MP3]" => Some(vec![0, 0, 0, 0]),
                "Track Artist 3 - Track Album 3 [123123]" => Some(vec![0, 0, 0, 1]),
                "Track Artist 1 - Track Album 1" => Some(vec![0, 0, 0, 2]),
                "Track Artist 2 Discography [MP3]" => Some(vec![0, 0, 0, 3]),
                "Track Artist 2 Discography [FLAC]" => Some(vec![0, 0, 0, 4]),
                _ => None,
            })
        }
    }

    #[actix_rt::test]
    async fn test_initializing_playlist_processor() {
        let playlist_processor = PlaylistProcessor::create(
            Arc::new(TrackDownloaderMock),
            Arc::new(PlaylistProviderMock),
            Arc::new(RadioManagerMock),
            Arc::new(MetadataServiceMock),
            Arc::new(MusicSearchServiceMock),
        );

        drop(playlist_processor);
    }

    #[actix_rt::test]
    async fn test_download_source_playlist() {
        let playlist_processor = PlaylistProcessor::create(
            Arc::new(TrackDownloaderMock),
            Arc::new(PlaylistProviderMock),
            Arc::new(RadioManagerMock),
            Arc::new(MetadataServiceMock),
            Arc::new(MusicSearchServiceMock),
        );

        let mut processing_data = PlaylistProcessingData::default();

        assert_eq!(
            processing_data.get_step(),
            PlaylistProcessingStep::DownloadPlaylist
        );

        let result = playlist_processor
            .process_playlist(
                &1,
                "ExistingPlaylistId",
                "ExistingPlaylistId",
                &mut processing_data,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(
            processing_data.get_step(),
            PlaylistProcessingStep::FilterNewTracks
        );
        assert_eq!(
            processing_data.unfiltered_tracks,
            Some(vec![
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 1"),
                        artist: String::from("Track Artist 1"),
                        album: String::from("Track Album 1"),
                    },
                },
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 2"),
                        artist: String::from("Track Artist 2"),
                        album: String::from("Track Album 2"),
                    },
                },
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 3"),
                        artist: String::from("Track Artist 3"),
                        album: String::from("Track Album 3"),
                    },
                },
            ])
        );
    }

    #[actix_rt::test]
    async fn test_filtering_new_tracks() {
        let playlist_processor = PlaylistProcessor::create(
            Arc::new(TrackDownloaderMock),
            Arc::new(PlaylistProviderMock),
            Arc::new(RadioManagerMock),
            Arc::new(MetadataServiceMock),
            Arc::new(MusicSearchServiceMock),
        );

        let mut processing_data = PlaylistProcessingData {
            unfiltered_tracks: Some(vec![
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 1"),
                        artist: String::from("Track Artist 1"),
                        album: String::from("Track Album 1"),
                    },
                },
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 2"),
                        artist: String::from("Track Artist 2"),
                        album: String::from("Track Album 2"),
                    },
                },
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 3"),
                        artist: String::from("Track Artist 3"),
                        album: String::from("Track Album 3"),
                    },
                },
            ]),
            ..PlaylistProcessingData::default()
        };

        assert_eq!(
            processing_data.get_step(),
            PlaylistProcessingStep::FilterNewTracks
        );

        let result = playlist_processor
            .process_playlist(
                &1,
                "ExistingPlaylistId",
                "ExistingPlaylistId",
                &mut processing_data,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(
            processing_data.get_step(),
            PlaylistProcessingStep::StartDownloadingTracks
        );
        assert_eq!(
            processing_data.filtered_tracks,
            Some(vec![
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 1"),
                        artist: String::from("Track Artist 1"),
                        album: String::from("Track Album 1"),
                    },
                },
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 3"),
                        artist: String::from("Track Artist 3"),
                        album: String::from("Track Album 3"),
                    },
                },
            ])
        );
    }

    #[actix_rt::test]
    async fn test_start_downloading_new_tracks() {
        let playlist_processor = PlaylistProcessor::create(
            Arc::new(TrackDownloaderMock),
            Arc::new(PlaylistProviderMock),
            Arc::new(RadioManagerMock),
            Arc::new(MetadataServiceMock),
            Arc::new(MusicSearchServiceMock),
        );

        let mut processing_data = PlaylistProcessingData {
            filtered_tracks: Some(vec![
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 1"),
                        artist: String::from("Track Artist 1"),
                        album: String::from("Track Album 1"),
                    },
                },
                PlaylistEntry {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 3"),
                        artist: String::from("Track Artist 3"),
                        album: String::from("Track Album 3"),
                    },
                },
            ]),
            ..PlaylistProcessingData::default()
        };

        assert_eq!(
            processing_data.get_step(),
            PlaylistProcessingStep::StartDownloadingTracks
        );

        let result = playlist_processor
            .process_playlist(
                &1,
                "ExistingPlaylistId",
                "ExistingPlaylistId",
                &mut processing_data,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(
            processing_data.get_step(),
            PlaylistProcessingStep::DownloadingTracks
        );
        assert_eq!(
            processing_data.audio_tracks_data,
            Some(vec![
                AudioTrackProcessingData {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 1"),
                        artist: String::from("Track Artist 1"),
                        album: String::from("Track Album 1"),
                    },
                    ..AudioTrackProcessingData::default()
                },
                AudioTrackProcessingData {
                    metadata: AudioMetadata {
                        title: String::from("Track Title 3"),
                        artist: String::from("Track Artist 3"),
                        album: String::from("Track Album 3"),
                    },
                    ..AudioTrackProcessingData::default()
                }
            ])
        );
        for track_data in processing_data.audio_tracks_data.unwrap() {
            assert_eq!(track_data.get_step(), AudioTrackProcessingStep::SearchAlbum);
        }
    }
}
