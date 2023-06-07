use crate::services::track_fetcher::traits::{StateStorage, StateStorageError};
use crate::services::track_fetcher::types::{TrackFetcherContext, TrackFetcherState};
use crate::types::{AudioMetadata, RadioterioChannelId, UserId};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) struct JobId(pub(crate) Uuid);

#[derive(Debug, thiserror::Error)]
pub(crate) enum CreateJobError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProceedNextStepError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
}

pub(crate) struct TrackFetcher {
    state_storage: Arc<dyn StateStorage>,
}

impl TrackFetcher {
    pub(crate) fn new(state_storage: Arc<dyn StateStorage>) -> Self {
        Self { state_storage }
    }

    pub(crate) async fn create_job(
        &self,
        user_id: &UserId,
        track_metadata: &AudioMetadata,
        target_channel_id: &RadioterioChannelId,
    ) -> Result<JobId, CreateJobError> {
        let key = Uuid::new_v4();
        let ctx = TrackFetcherContext::new(
            track_metadata.title.clone(),
            track_metadata.artist.clone(),
            track_metadata.album.clone(),
            target_channel_id.clone(),
        );
        let state = TrackFetcherState::default();

        let key_str = key.to_string();
        self.state_storage
            .save_context(user_id, &key_str, ctx)
            .await?;
        self.state_storage
            .save_state(user_id, &key_str, state)
            .await?;

        Ok(JobId(key))
    }

    pub(crate) async fn continue_job(
        &self,
        user_id: &UserId,
        job_id: &JobId,
    ) -> Result<(), ProceedNextStepError> {
        Ok(())
    }

    async fn proceed_next_step(&self, ctx: &TrackFetcherContext, state: &mut TrackFetcherState) {}
}
