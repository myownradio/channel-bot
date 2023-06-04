use crate::services::playlist_processor::traits::{
    MetadataService, MusicSearchService, PlaylistProvider, RadioManager, TrackDownloader,
};
use crate::services::playlist_processor::types::{
    DownloadId, DownloadingStatus, MetadataServiceError, MusicSearchServiceError, PlaylistEntry,
    PlaylistProviderError, RadioManagerError, TopicId, TrackDownloadEntry, TrackDownloaderError,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::warn;

pub(crate) struct AudioTrackProcessingData {
    tried_topics: Vec<TopicId>,
    current_download: Option<TrackDownloadEntry>,
    path_to_audio_file: Option<String>,
    radioterio_track_id: Option<u64>,
    radioterio_channel_id: Option<u64>,
}

impl AudioTrackProcessingData {
    pub(crate) fn get_step(&self) -> AudioTrackProcessingStep {
        if self.radioterio_channel_id.is_some() {
            return AudioTrackProcessingStep::End;
        }

        if self.radioterio_track_id.is_some() {
            return AudioTrackProcessingStep::AddToChannel;
        }

        if self.path_to_audio_file.is_some() {
            return AudioTrackProcessingStep::Upload;
        }

        if let Some(current_download) = &self.current_download {
            return match current_download.status {
                DownloadingStatus::Finished => AudioTrackProcessingStep::CheckDownload,
                DownloadingStatus::Downloading => AudioTrackProcessingStep::Downloading,
            };
        }

        AudioTrackProcessingStep::SearchAlbum
    }
}

pub(crate) enum AudioTrackProcessingStep {
    Initial,
    SearchAlbum,
    Downloading,
    CheckDownload,
    Upload,
    AddToChannel,
    End,
}

impl AudioTrackProcessingStep {
    pub(crate) fn is_final(&self) -> bool {
        matches!(self, AudioTrackProcessingStep::End)
    }
}

pub(crate) struct PlaylistProcessingData {
    unfiltered_tracks: Option<Vec<PlaylistEntry>>,
    filtered_tracks: Option<Vec<PlaylistEntry>>,
    audio_tracks: Option<Vec<AudioTrackProcessingData>>,
}

impl PlaylistProcessingData {
    pub(crate) fn get_step(&self) -> PlaylistProcessingStep {
        if let Some(audio_tracks) = &self.audio_tracks {
            return if audio_tracks.iter().all(|track| track.get_step().is_final()) {
                PlaylistProcessingStep::Final
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

pub(crate) enum PlaylistProcessingStep {
    DownloadPlaylist,
    FilterNewTracks,
    StartDownloadingTracks,
    DownloadingTracks,
    Final,
}

impl PlaylistProcessingStep {
    pub(crate) fn is_final(&self) -> bool {
        matches!(self, PlaylistProcessingStep::Final)
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
        ctx: &mut ProcessingContext,
    ) -> Result<(), PlaylistProcessingError> {
        match &mut ctx.step {
            ProcessingStep::GetSourcePlaylist => {
                match self.playlist_provider.get_playlist(src_playlist_id).await? {
                    Some(unfiltered_tracks) => {
                        ctx.step = ProcessingStep::FilterNewTracks { unfiltered_tracks };
                    }
                    None => {
                        return Err(PlaylistProcessingError::SourcePlaylistNotFound);
                    }
                };
            }
            ProcessingStep::FilterNewTracks { unfiltered_tracks } => {
                let playlist_entries =
                    match self.radio_manager.get_playlist(dst_playlist_id).await? {
                        Some(dst_tracks) => {
                            let dst_tracks_set = dst_tracks
                                .into_iter()
                                .map(|track| {
                                    format!("{}-{}-{}", track.artist, track.album, track.title)
                                })
                                .collect::<HashSet<_>>();

                            unfiltered_tracks
                                .iter()
                                .filter(move |track| {
                                    let key =
                                        format!("{}-{}-{}", track.artist, track.album, track.title);
                                    !dst_tracks_set.contains(&key)
                                })
                                .cloned()
                                .collect()
                        }
                        None => unfiltered_tracks.clone(),
                    };
                ctx.step = ProcessingStep::ProcessPlaylistTracks {
                    tracks_ctx: playlist_entries
                        .into_iter()
                        .map(|track| TrackProcessingContext {
                            track,
                            step: TrackProcessingStep::Initial,
                        })
                        .collect(),
                };
            }
            ProcessingStep::ProcessPlaylistTracks { tracks_ctx } => {
                if tracks_ctx
                    .iter()
                    .all(|track_ctx| matches!(track_ctx.step, TrackProcessingStep::Finish))
                {
                    ctx.step = ProcessingStep::Finish;
                } else {
                    for track_context in tracks_ctx.iter_mut() {
                        self.process_single_track(track_context, &mut ctx.topics_map)
                            .await?;
                    }
                }
            }
            ProcessingStep::Finish => (),
        };

        Ok(())
    }

    async fn process_single_track(
        &self,
        ctx: &mut TrackProcessingContext,
        topics_map: &mut HashMap<TopicId, DownloadId>,
    ) -> Result<(), PlaylistProcessingError> {
        match ctx.step.clone() {
            TrackProcessingStep::Initial => {
                ctx.step = TrackProcessingStep::Search {
                    other_topics: vec![],
                };
            }
            TrackProcessingStep::Search { other_topics } => {
                let other_topics_set = other_topics.iter().collect::<HashSet<_>>();

                let album_query = format!("{} - {}", ctx.track.artist, ctx.track.album);
                let maybe_entry = self
                    .search_service
                    .search(&album_query)
                    .await?
                    .into_iter()
                    .filter(|entry| other_topics_set.contains(&entry.topic_id))
                    .next();

                if let Some(entry) = maybe_entry {
                    let maybe_download_url = self
                        .search_service
                        .get_download_url(&entry.topic_id)
                        .await?;

                    if let Some(download_url) = maybe_download_url {
                        let download_id = self
                            .track_downloader
                            .create_download("/tmp/downloads", download_url)
                            .await?;

                        let mut topics = other_topics.clone();
                        topics_map.insert(entry.topic_id.clone(), download_id);
                        topics.push(entry.topic_id);
                        ctx.step = TrackProcessingStep::Download { topics };

                        return Ok(());
                    }
                }

                ctx.step = TrackProcessingStep::NotFound;
            }
            TrackProcessingStep::Download { topics } => {
                let download_ids = topics
                    .into_iter()
                    .filter_map(|topic_id| topics_map.get(&topic_id))
                    .cloned()
                    .collect::<Vec<_>>();

                for download_id in download_ids.into_iter() {
                    let download_entry =
                        match self.track_downloader.get_download(&download_id).await? {
                            None => {
                                warn!("Track has reference to download which does not exist");
                                continue;
                            }
                            Some(download_entry) => download_entry,
                        };

                    if !matches!(download_entry.status, DownloadingStatus::Finished) {
                        continue;
                    }
                }
            }
            TrackProcessingStep::AddToPlaylist { path_to_file } => (),
            TrackProcessingStep::Finish | TrackProcessingStep::NotFound => (),
        }

        Ok(())
    }
}
