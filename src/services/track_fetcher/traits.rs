use crate::services::track_fetcher::types::{TrackFetcherContext, TrackFetcherState};
use crate::types::UserId;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub(crate) enum StateStorageError {
    #[error("Unexpected error")]
    Unexpected,
}

#[async_trait]
pub(crate) trait StateStorage {
    async fn save_state(
        &self,
        user_id: &UserId,
        key: &str,
        state: TrackFetcherState,
    ) -> Result<(), StateStorageError>;
    async fn save_context(
        &self,
        user_id: &UserId,
        key: &str,
        state: TrackFetcherContext,
    ) -> Result<(), StateStorageError>;
    async fn load_state(
        &self,
        user_id: &UserId,
        key: &str,
    ) -> Result<Option<TrackFetcherState>, StateStorageError>;
    async fn load_context(
        &self,
        user_id: &UserId,
        key: &str,
    ) -> Result<Option<TrackFetcherContext>, StateStorageError>;
}
