use crate::services::track_request_processor::{
    RadioManagerChannelId, RadioManagerClient as RadioManagerClientTrait, RadioManagerClientError,
    RadioManagerLinkId, RadioManagerTrackId, UserId,
};
use async_trait::async_trait;

pub(crate) struct RadioManagerClient {
    endpoint: String,
}

impl RadioManagerClient {
    pub(crate) fn create(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[async_trait]
impl RadioManagerClientTrait for RadioManagerClient {
    async fn upload_audio_track(
        &self,
        user_id: &UserId,
        path_to_audio_file: &str,
    ) -> Result<RadioManagerTrackId, RadioManagerClientError> {
        todo!()
    }

    async fn add_track_to_channel_playlist(
        &self,
        user_id: &UserId,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError> {
        todo!()
    }
}
