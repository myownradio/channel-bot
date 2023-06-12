use crate::services::storage::MemoryBasedStorage;
use crate::services::track_request_processor::traits::{
    SearchProvider, SearchProviderError, SearchResult, StateStorage, StateStorageError,
};
use crate::services::track_request_processor::types::{
    RequestId, TrackFetcherContext, TrackFetcherState,
};
use crate::services::transmission::TransmissionClient;
use crate::services::{DownloaderError, DownloadingEntry, TorrentClient};
use crate::types::{DownloadId, TopicId, UserId};
use async_trait::async_trait;
use search_providers::RuTrackerClient;

#[async_trait]
impl StateStorage for MemoryBasedStorage {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackFetcherState,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("state-{}", user_id);
        let key = format!("{}", request_id);
        let state_str = &serde_json::to_string(&state).unwrap();

        if self.get(&prefix, &key).is_some() {
            return Err(StateStorageError::ObjectExists);
        }

        self.save(&prefix, &key, state_str);

        Ok(())
    }

    async fn create_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        ctx: TrackFetcherContext,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("ctx-{}", user_id);
        let key = format!("{}", request_id);
        let ctx_str = &serde_json::to_string(&ctx).unwrap();

        if self.get(&prefix, &key).is_some() {
            return Err(StateStorageError::ObjectExists);
        }

        self.save(&prefix, &key, ctx_str);

        Ok(())
    }

    async fn update_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackFetcherState,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("state-{}", user_id);
        let key = format!("{}", request_id);
        let state_str = &serde_json::to_string(&state).unwrap();

        if self.get(&prefix, &key).is_none() {
            return Err(StateStorageError::ObjectNotFound);
        }

        self.save(&prefix, &key, state_str);

        Ok(())
    }

    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackFetcherState, StateStorageError> {
        let prefix = format!("state-{}", user_id);
        let key = format!("{}", request_id);

        match self.get(&prefix, &key) {
            Some(value) => Ok(serde_json::from_str(&value)?),
            None => Err(StateStorageError::ObjectNotFound),
        }
    }

    async fn load_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackFetcherContext, StateStorageError> {
        let prefix = format!("ctx-{}", user_id);
        let key = format!("{}", request_id);

        match self.get(&prefix, &key) {
            Some(value) => Ok(serde_json::from_str(&value)?),
            None => Err(StateStorageError::ObjectNotFound),
        }
    }

    async fn delete_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("state-{}", user_id);
        let key = format!("{}", request_id);

        self.delete(&prefix, &key);

        Ok(())
    }

    async fn delete_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("ctx-{}", user_id);
        let key = format!("{}", request_id);

        self.delete(&prefix, &key);

        Ok(())
    }
}

#[async_trait]
impl SearchProvider for RuTrackerClient {
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchProviderError> {
        self.search_music(query)
            .await
            .map(|results| {
                results
                    .into_iter()
                    .map(|r| SearchResult {
                        title: r.title,
                        topic_id: r.topic_id,
                        download_id: r.download_id,
                    })
                    .collect()
            })
            .map_err(|err| SearchProviderError::Unexpected)
    }

    async fn get_url(&self, topic_id: &TopicId) -> Result<Option<Vec<u8>>, SearchProviderError> {
        let topic = self
            .get_topic(topic_id)
            .await
            .map_err(|_| SearchProviderError::Unexpected)?
            .ok_or_else(|| SearchProviderError::Unexpected)?;

        let torrent_data = self
            .download_torrent(topic.torrent_id)
            .await
            .map_err(|_| SearchProviderError::Unexpected)?;

        Ok(Some(torrent_data))
    }
}

impl TorrentClient for TransmissionClient {
    async fn create(
        &self,
        path_to_download: &str,
        url: Vec<u8>,
    ) -> Result<DownloadId, DownloaderError> {
        todo!()
    }

    async fn get(
        &self,
        download_id: &DownloadId,
    ) -> Result<Option<DownloadingEntry>, DownloaderError> {
        todo!()
    }

    async fn delete(&self, download_id: &DownloadId) -> Result<(), DownloaderError> {
        todo!()
    }
}
