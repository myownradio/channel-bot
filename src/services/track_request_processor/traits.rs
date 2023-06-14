use super::types::{
    AudioMetadata, DownloadId, RadioManagerChannelId, RadioManagerLinkId, RadioManagerTrackId,
    RequestId, TopicData, Torrent, TorrentId, UserId,
};
use super::{TrackRequestProcessingContext, TrackRequestProcessingState};
use async_trait::async_trait;
use std::fmt::Formatter;

#[derive(Debug, thiserror::Error)]
pub struct StateStorageError(pub Box<dyn std::error::Error>);

impl std::fmt::Display for StateStorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait StateStorageTrait: Sync {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingState,
    ) -> Result<(), StateStorageError>;
    async fn create_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingContext,
    ) -> Result<(), StateStorageError>;
    async fn update_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackRequestProcessingState,
    ) -> Result<(), StateStorageError>;
    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingState, StateStorageError>;
    async fn load_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingContext, StateStorageError>;
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

#[derive(Debug, thiserror::Error)]
pub struct SearchProviderError(pub Box<dyn std::error::Error>);

impl std::fmt::Display for SearchProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait SearchProviderTrait: Sync {
    async fn search_music(&self, query: &str) -> Result<Vec<TopicData>, SearchProviderError>;
    async fn download_torrent(
        &self,
        download_id: &DownloadId,
    ) -> Result<Vec<u8>, SearchProviderError>;
}
#[derive(Debug, thiserror::Error)]

pub struct TorrentClientError(pub Box<dyn std::error::Error>);

impl std::fmt::Display for TorrentClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait TorrentClientTrait: Sync {
    async fn create(
        &self,
        path_to_download: &str,
        torrent_file_data: Vec<u8>,
    ) -> Result<TorrentId, TorrentClientError>;
    async fn get(&self, torrent_id: &TorrentId) -> Result<Torrent, TorrentClientError>;
    async fn delete(&self, torrent_id: &TorrentId) -> Result<(), TorrentClientError>;
}

#[derive(Debug, thiserror::Error)]
pub struct MetadataServiceError(pub Box<dyn std::error::Error>);

impl std::fmt::Display for MetadataServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait MetadataServiceTrait: Sync {
    async fn get_audio_metadata(
        &self,
        file_path: &str,
    ) -> Result<Option<AudioMetadata>, MetadataServiceError>;
}

#[derive(Debug, thiserror::Error)]
pub struct RadioManagerClientError(pub Box<dyn std::error::Error>);

impl std::fmt::Display for RadioManagerClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait RadioManagerClientTrait: Sync {
    async fn upload_audio_track(
        &self,
        user_id: &UserId,
        path_to_audio_file: &str,
    ) -> Result<RadioManagerTrackId, RadioManagerClientError>;
    async fn add_track_to_channel_playlist(
        &self,
        user_id: &UserId,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError>;
}
