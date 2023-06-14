use async_trait::async_trait;
use request_processors::{DownloadId, SearchProviderError, TopicData};
use std::ops::Deref;

pub(crate) struct RuTrackerClient(pub(crate) search_providers::RuTrackerClient);

impl Deref for RuTrackerClient {
    type Target = search_providers::RuTrackerClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl request_processors::SearchProvider for RuTrackerClient {
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
