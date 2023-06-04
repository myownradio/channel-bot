use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::warn;

//
// Downloader
//
#[derive(Eq, PartialEq, Clone)]
pub(crate) struct DownloadId(String);

pub(crate) enum DownloadingStatus {
    Downloading,
    Finished,
}

pub(crate) struct DownloadEntry {
    status: DownloadingStatus,
    files: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DownloadingServiceError {
    #[error("Unexpected error")]
    Unexpected,
}

#[async_trait]
trait Downloader {
    async fn create_download(&self, path: &str) -> Result<DownloadId, DownloadingServiceError>;
    async fn get_download(
        &self,
        download_id: &DownloadId,
    ) -> Result<Option<DownloadEntry>, DownloadingServiceError>;
    async fn delete_download(
        &self,
        download_id: &DownloadId,
    ) -> Result<(), DownloadingServiceError>;
}

//
// Playlist Provider
//

#[derive(Clone)]
pub(crate) struct PlaylistProviderPlaylistEntry {
    title: String,
    artist: String,
    album: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PlaylistProvidingError {
    #[error("Unexpected error")]
    Unexpected,
}

#[async_trait]
trait PlaylistProvider {
    async fn get_playlist(
        &self,
        playlist_id: &str,
    ) -> Result<Option<Vec<PlaylistProviderPlaylistEntry>>, PlaylistProvidingError>;
}

//
// Radio Manager
//

pub(crate) struct RadioManagerPlaylistEntry {
    id: String,
    title: String,
    artist: String,
    album: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RadioManagerError {
    #[error("Unexpected error")]
    Unexpected,
}

#[async_trait]
trait RadioManager {
    async fn get_playlist(
        &self,
        playlist_id: &str,
    ) -> Result<Option<Vec<RadioManagerPlaylistEntry>>, RadioManagerError>;
    async fn add_track_to_playlist(
        &self,
        playlist_id: &str,
        path_to_track: &str,
    ) -> Result<(), RadioManagerError>;
}

// Audio Metadata Service
pub(crate) struct AudioMetadata {
    title: String,
    artist: String,
    album: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AudioMetadataServiceError {
    #[error("Unexpected error")]
    Unexpected,
}

#[async_trait]
trait AudioMetadataService {
    async fn get_metadata(
        &self,
        path_to_new_track: &str,
    ) -> Result<Option<AudioMetadata>, AudioMetadataServiceError>;
}

// Audio Search Service
#[derive(Eq, PartialEq, Clone, Hash)]
pub(crate) struct CandidateId(String);

pub(crate) struct DownloadCandidate {
    candidate_id: CandidateId,
    download_id: DownloadId,
    tracks_hint: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AudioSearchingServiceError {
    #[error("Unexpected error")]
    Unexpected,
}

#[async_trait]
trait AudioSearchingService {
    async fn search(
        &self,
        query: &str,
    ) -> Result<Vec<DownloadCandidate>, AudioSearchingServiceError>;
    async fn get_download(
        &self,
        candidate_id: &CandidateId,
    ) -> Result<Option<()>, AudioSearchingServiceError>;
}

// Processing Context
#[derive(Clone)]
pub(crate) enum TrackProcessingStep {
    Initial,
    GatherDownloadCandidate(Vec<CandidateId>),
    Download(Vec<DownloadId>),
    AddToPlaylist(String),
    Finish,
}

pub(crate) struct TrackProcessingContext {
    track: PlaylistProviderPlaylistEntry,
    step: TrackProcessingStep,
}

pub(crate) enum ProcessingStep {
    GetSourcePlaylist,
    FilterNewTracks(Vec<PlaylistProviderPlaylistEntry>),
    ProcessPlaylistTracks(Vec<TrackProcessingContext>),
    Finish,
}

pub(crate) struct ProcessingContext {
    step: ProcessingStep,
}

pub(crate) struct PlaylistProcessor {
    downloader: Arc<dyn Downloader>,
    playlist_provider: Arc<dyn PlaylistProvider>,
    radio_manager: Arc<dyn RadioManager>,
    audio_metadata_service: Arc<dyn AudioMetadataService>,
    audio_searching_service: Arc<dyn AudioSearchingService>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PlaylistProcessingError {
    #[error(transparent)]
    PlaylistProvidingError(#[from] PlaylistProvidingError),
    #[error(transparent)]
    RadioManagerError(#[from] RadioManagerError),
    #[error(transparent)]
    DownloadingServiceError(#[from] DownloadingServiceError),
    #[error(transparent)]
    AudioMetadataServiceError(#[from] AudioMetadataServiceError),
    #[error(transparent)]
    AudioSearchingServiceError(#[from] AudioSearchingServiceError),
    #[error("Source playlist not found")]
    SourcePlaylistNotFound,
}

impl PlaylistProcessor {
    pub(crate) fn create(
        downloader: Arc<dyn Downloader>,
        playlist_provider: Arc<dyn PlaylistProvider>,
        radio_manager: Arc<dyn RadioManager>,
        audio_metadata_service: Arc<dyn AudioMetadataService>,
        audio_searching_service: Arc<dyn AudioSearchingService>,
    ) -> Self {
        Self {
            downloader,
            playlist_provider,
            radio_manager,
            audio_metadata_service,
            audio_searching_service,
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
                    Some(src_tracks) => {
                        ctx.step = ProcessingStep::FilterNewTracks(src_tracks);
                    }
                    None => {
                        return Err(PlaylistProcessingError::SourcePlaylistNotFound);
                    }
                };
            }
            ProcessingStep::FilterNewTracks(tracks) => {
                let filtered_tracks = match self.radio_manager.get_playlist(dst_playlist_id).await?
                {
                    Some(dst_tracks) => {
                        let dst_tracks_set = dst_tracks
                            .into_iter()
                            .map(|track| {
                                format!("{}-{}-{}", track.artist, track.album, track.title)
                            })
                            .collect::<HashSet<_>>();

                        tracks
                            .iter()
                            .filter(move |track| {
                                let key =
                                    format!("{}-{}-{}", track.artist, track.album, track.title);
                                !dst_tracks_set.contains(&key)
                            })
                            .cloned()
                            .collect()
                    }
                    None => tracks.clone(),
                };

                ctx.step = ProcessingStep::ProcessPlaylistTracks(
                    filtered_tracks
                        .into_iter()
                        .map(|track| TrackProcessingContext {
                            track,
                            step: TrackProcessingStep::Initial,
                        })
                        .collect(),
                );
            }
            ProcessingStep::ProcessPlaylistTracks(track_context_list)
                if track_context_list
                    .iter()
                    .all(|track_ctx| matches!(track_ctx.step, TrackProcessingStep::Finish)) =>
            {
                ctx.step = ProcessingStep::Finish;
            }
            ProcessingStep::ProcessPlaylistTracks(track_context_list) => {
                for track_context in track_context_list.iter_mut() {
                    self.process_single_track(track_context).await?;
                }
            }
            ProcessingStep::Finish => (),
        };

        Ok(())
    }

    async fn process_single_track(
        &self,
        ctx: &mut TrackProcessingContext,
    ) -> Result<(), PlaylistProcessingError> {
        match ctx.step.clone() {
            TrackProcessingStep::Initial => {
                ctx.step = TrackProcessingStep::GatherDownloadCandidate(vec![]);
            }
            TrackProcessingStep::GatherDownloadCandidate(other_candidates) => {
                let other_candidates_set = other_candidates.into_iter().collect::<HashSet<_>>();
                // TODO: Search for the download candidate
                let query = format!("{} - {}", ctx.track.artist, ctx.track.album);
                let results = self
                    .audio_searching_service
                    .search(&query)
                    .await?
                    .into_iter()
                    .filter(|c| other_candidates_set.contains(&c.candidate_id))
                    .fi();
            }
            TrackProcessingStep::Download => {
                for download_id in track.download_ids.clone().into_iter() {
                    match self.downloader.get_download(&download_id).await? {
                        Some(download) => {
                            for file in download.files {
                                if let Some(metadata) =
                                    self.audio_metadata_service.get_metadata(&file).await?
                                {
                                    if metadata.title == track.track.title
                                        && metadata.artist == track.track.artist
                                    {
                                        track.step = TrackProcessingStep::AddToPlaylist(file);
                                        continue 'track;
                                    }
                                }
                            }
                        }
                        None => {
                            warn!("Track has reference to download which does not exist");
                            track.download_ids.retain(|c| c != &download_id);
                        }
                    }
                }
            }
            TrackProcessingStep::Finish => (),
        }

        Ok(())
    }
}
