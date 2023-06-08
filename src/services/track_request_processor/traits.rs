use crate::services::track_request_processor::types::{
    RequestId, TrackFetcherContext, TrackFetcherState,
};
use crate::types::UserId;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub(crate) enum StateStorageError {
    #[error("Unexpected error")]
    Unexpected,
}

#[async_trait]
pub(crate) trait StateStorage {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackFetcherState,
    ) -> Result<(), StateStorageError>;
    async fn create_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackFetcherContext,
    ) -> Result<(), StateStorageError>;
    async fn update_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackFetcherState,
    ) -> Result<(), StateStorageError>;
    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<Option<TrackFetcherState>, StateStorageError>;
    async fn load_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<Option<TrackFetcherContext>, StateStorageError>;
    async fn delete_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError>;
    async fn delete_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError>;
}
