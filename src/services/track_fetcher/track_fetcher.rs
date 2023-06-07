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
        let state = TrackFetcherState::default();
        let ctx = TrackFetcherContext {
            track_title: track_metadata.title.clone(),
            track_artist: track_metadata.artist.clone(),
            track_album: track_metadata.album.clone(),
            target_channel_id: target_channel_id.clone(),
        };

        let key_str = key.to_string();
        self.state_storage
            .save_context(user_id, &key_str, ctx)
            .await?;
        self.state_storage
            .save_state(user_id, &key_str, state)
            .await?;

        Ok(JobId(key))
    }
}
