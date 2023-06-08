use crate::services::track_request_processor::traits::{
    Downloader, DownloaderError, DownloadingStatus, SearchProvider, SearchProviderError,
    SearchResult, StateStorage, StateStorageError,
};
use crate::services::track_request_processor::types::{
    RequestId, TrackFetcherContext, TrackFetcherState, TrackFetcherStep,
};
use crate::types::{AudioMetadata, RadioterioChannelId, UserId};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CreateJobError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProceedNextStepError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
    #[error(transparent)]
    SearchProviderError(#[from] SearchProviderError),
    #[error(transparent)]
    DownloaderError(#[from] DownloaderError),
    #[error("Job has not been found in the storage")]
    JobNotFound,
}

pub(crate) struct TrackRequestProcessor {
    state_storage: Arc<dyn StateStorage>,
    search_provider: Arc<dyn SearchProvider>,
    downloader: Arc<dyn Downloader>,
}

impl TrackRequestProcessor {
    pub(crate) fn new(
        state_storage: Arc<dyn StateStorage>,
        search_provider: Arc<dyn SearchProvider>,
        downloader: Arc<dyn Downloader>,
    ) -> Self {
        Self {
            state_storage,
            search_provider,
            downloader,
        }
    }

    pub(crate) async fn create_track_request(
        &self,
        user_id: &UserId,
        track_metadata: &AudioMetadata,
        target_channel_id: &RadioterioChannelId,
    ) -> Result<RequestId, CreateJobError> {
        let request_id = Uuid::new_v4().into();
        let ctx = TrackFetcherContext::new(
            track_metadata.title.clone(),
            track_metadata.artist.clone(),
            track_metadata.album.clone(),
            target_channel_id.clone(),
        );
        let state = TrackFetcherState::default();

        self.state_storage
            .create_context(user_id, &request_id, ctx)
            .await?;
        self.state_storage
            .create_state(user_id, &request_id, state)
            .await?;

        info!(%user_id, %request_id, "New track request successfully created");

        Ok(request_id)
    }

    pub(crate) async fn process_track_request(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), ProceedNextStepError> {
        info!(%user_id, %request_id, "Processing track request");

        let ctx = self
            .state_storage
            .load_context(user_id, request_id)
            .await?
            .ok_or_else(|| ProceedNextStepError::JobNotFound)?;
        let mut state = self
            .state_storage
            .load_state(user_id, request_id)
            .await?
            .ok_or_else(|| ProceedNextStepError::JobNotFound)?;

        while !matches!(state.get_step(), TrackFetcherStep::Finish) {
            self.run_next_step(user_id, &ctx, &mut state).await?;
            self.state_storage
                .update_state(user_id, request_id, &state)
                .await?;
        }

        info!(%user_id, %request_id, "Track processing finished");

        self.state_storage.delete_state(user_id, request_id).await?;
        self.state_storage
            .delete_context(user_id, request_id)
            .await?;

        debug!("Track processing state and context have been cleaned");

        Ok(())
    }

    async fn run_next_step(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let step = state.get_step();

        debug!(%user_id, ?step, "Running next processing step");

        match step {
            TrackFetcherStep::SearchAudioAlbum => {
                self.search_audio_album(ctx, state).await?;
            }
            TrackFetcherStep::GetAlbumURL => {
                self.get_album_url(ctx, state).await?;
            }
            TrackFetcherStep::DownloadAlbum => {
                self.download_album(ctx, state).await?;
            }
            TrackFetcherStep::CheckDownloadStatus => {
                self.check_download_status(ctx, state).await?;
            }
            TrackFetcherStep::UploadToRadioterio => {
                self.upload_to_radioterio(ctx, state).await?;
            }
            TrackFetcherStep::AddToRadioterioChannel => {
                self.add_to_radioterio_channel(ctx, state).await?;
            }
            TrackFetcherStep::Finish => (),
        }

        Ok(())
    }

    async fn search_audio_album(
        &self,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let query = format!("{} - {}", ctx.track_artist, ctx.track_album);
        let results = self.search_provider.search(&query).await?;

        let tried_topics_set = state.tried_topics.iter().collect::<HashSet<_>>();

        let next_result = results
            .into_iter()
            .filter(|r| !tried_topics_set.contains(&r.topic_id))
            .next();

        match next_result {
            Some(result) => {
                let SearchResult { topic_id, title } = result;
                debug!(%topic_id, title, "Found result possibly containing the requested track");
                state.current_topic_id.replace(topic_id);
                Ok(())
            }
            None => {
                debug!("Requested track has not get been found...");
                todo!();
            }
        }
    }

    async fn get_album_url(
        &self,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let topic_id = state
            .current_topic_id
            .clone()
            .take()
            .expect("Current topic_id should be defined");

        debug!("Getting URL to download the audio album...");

        match self.search_provider.get_url(&topic_id).await? {
            Some(url) => {
                state.current_url.replace(url);
                Ok(())
            }
            None => {
                warn!("The current topic is gone. Going to look for another one.");
                state.tried_topics.push(topic_id);
                Ok(())
            }
        }
    }

    async fn download_album(
        &self,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let url = state
            .current_url
            .clone()
            .take()
            .expect("Current current_url should be defined");

        let download_id = self.downloader.create("tmp/downloads", url).await?;

        debug!(%download_id, "Started download of the audio album...");

        state.current_download_id.replace(download_id);

        Ok(())
    }

    async fn check_download_status(
        &self,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let download_id = state
            .current_download_id
            .clone()
            .take()
            .expect("Current current_download_id should be defined");

        let downloading_entry = self.downloader.get(&download_id).await?;

        match downloading_entry {
            Some(entry) => {
                if matches!(entry.status, DownloadingStatus::Complete) {
                    debug!(%download_id, "Download complete");
                    // TODO: Check downloaded files for the requested audio track...
                    todo!();
                }
            }
            None => {
                warn!(%download_id, "Download has not been found");
                state.current_download_id.take();
            }
        }

        Ok(())
    }

    async fn upload_to_radioterio(
        &self,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        Ok(())
    }

    async fn add_to_radioterio_channel(
        &self,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        Ok(())
    }
}
