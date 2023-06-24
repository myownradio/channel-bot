use crate::services::track_request_processor::{
    RequestId, StateStorageError, StateStorageTrait, TrackRequestProcessingContext,
    TrackRequestProcessingState,
};
use crate::services::TrackRequestProcessor;
use crate::types::UserId;
use std::sync::Arc;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub(crate) enum TrackRequestControllerError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
}

pub(crate) struct TrackRequestController {
    track_request_processor: Arc<TrackRequestProcessor>,
}

impl TrackRequestController {
    pub(crate) async fn create(
        state_storage: Arc<dyn StateStorageTrait + Send + Sync + 'static>,
        track_request_processor: Arc<TrackRequestProcessor>,
    ) -> Result<Self, TrackRequestControllerError> {
        let controller = Self {
            track_request_processor,
        };

        let tasks = state_storage.get_all_tasks().await?;

        for (user_id, request_id) in tasks {
            controller.spawn_task(user_id, request_id);
        }

        Ok(controller)
    }

    fn spawn_task(&self, user_id: UserId, request_id: RequestId) {
        actix_rt::spawn({
            let user_id = user_id.clone();
            let request_id = request_id.clone();
            let track_request_processor = self.track_request_processor.clone();

            async move {
                if let Err(error) = track_request_processor
                    .process_request(&user_id, &request_id)
                    .await
                {
                    error!(?error, "Track request processing failed");
                }
            }
        });
    }
}
