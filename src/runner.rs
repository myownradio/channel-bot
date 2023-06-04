use crate::types::AudioTrack;
use std::sync::Arc;

pub(crate) enum RunnerError {}

pub(crate) struct DownloadId(String);

pub(crate) enum DownloadingState {
    DOWNLOADING,
    FINISHED,
}

pub(crate) struct DownloadEntry {
    status: DownloadingState,
    files: Vec<String>,
}

pub(crate) enum DownloadingServiceError {}

#[async_trait::async_trait]
pub(crate) trait DownloadingService {
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

pub(crate) struct StaticState {
    downloading_service: Arc<dyn DownloadingService>,
}

pub(crate) struct TrackProcessingState {
    track: AudioTrack,
    downloads: Vec<DownloadId>,
}

pub(crate) enum ProcessingStep {
    GetPlaylist,
    FilterNewTracks(Vec<AudioTrack>),
    ProcessTracks(Vec<TrackProcessingState>),
}

pub(crate) struct DynamicState {
    processing_step: ProcessingStep,
}

pub(crate) struct RunnerContext {
    static_state: StaticState,
    dynamic_state: DynamicState,
}

/**
    Steps:
    1. Download playlist from Spotify
    2. Download/create playlist on Radioterio
    3. Filter only new tracks
    4. Initialize tracks processing.
    4. Until all tracks are in finished state:
    4.1. Update status of current downloads
    4.2. For each track do:
    4.2.1. If associated download contains the track:
    4.2.1.1. Upload track to Radioterio.
    4.2.1.2. Add uploaded track to the end of playlist.
    4.2.1.3. Mark the track processing as finished (||).
    4.2.2. If associated with the track downloads have been finished:
    4.2.2.1. Find another new candidate on a torrent tracker.
    4.2.2.2. Start the new download.
    4.2.2.3. Add the download to the track's candidates.
    5. Drop tracks processing state.
*/

pub(crate) async fn run(mut runner_context: RunnerContext) {
    loop {
        match &runner_context.dynamic_state.processing_step {
            ProcessingStep::GetPlaylist => {
                // TODO: Download playlist
                let tracks = vec![];

                runner_context.dynamic_state.processing_step =
                    ProcessingStep::FilterNewTracks(tracks);
            }
            ProcessingStep::FilterNewTracks(tracks) => {
                // TODO: Filter new tracks
            }
            ProcessingStep::ProcessTracks(tracks) => {}
        }
    }
}
