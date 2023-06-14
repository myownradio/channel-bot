use crate::services::track_request_processor::{
    DownloadId, SearchProvider, SearchProviderError, TopicData,
};
use async_trait::async_trait;
use search_providers::RuTrackerClient;

#[async_trait]
impl SearchProvider for RuTrackerClient {
    async fn search_music(&self, query: &str) -> Result<Vec<TopicData>, SearchProviderError> {
        todo!()
    }

    async fn download_torrent(
        &self,
        download_id: &DownloadId,
    ) -> Result<Vec<u8>, SearchProviderError> {
        todo!()
    }
}
