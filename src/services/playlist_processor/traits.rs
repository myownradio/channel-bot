use crate::services::playlist_processor::types::{
    AudioMetadata, DownloadId, MetadataServiceError, MusicSearchServiceError, PlaylistEntry,
    PlaylistProviderError, RadioManagerError, RadioManagerPlaylistEntry, SearchResultsEntry,
    TopicId, TrackDownloadEntry, TrackDownloaderError,
};
use async_trait::async_trait;

#[async_trait]
pub(crate) trait PlaylistProvider {
    async fn get_playlist(
        &self,
        playlist_id: &str,
    ) -> Result<Option<Vec<PlaylistEntry>>, PlaylistProviderError>;
}

#[async_trait]
pub(crate) trait TrackDownloader {
    async fn create_download(
        &self,
        path_to_download: &str,
        url: Vec<u8>,
    ) -> Result<DownloadId, TrackDownloaderError>;
    async fn get_download(
        &self,
        download_id: &DownloadId,
    ) -> Result<Option<TrackDownloadEntry>, TrackDownloaderError>;
    async fn delete_download(&self, download_id: &DownloadId) -> Result<(), TrackDownloaderError>;
}

#[async_trait]
pub(crate) trait RadioManager {
    async fn get_playlist(
        &self,
        playlist_id: &str,
    ) -> Result<Option<Vec<RadioManagerPlaylistEntry>>, RadioManagerError>;
    async fn add_track_to_playlist(
        &self,
        playlist_id: &str,
        file_path: &str,
    ) -> Result<(), RadioManagerError>;
}

#[async_trait]
pub(crate) trait MetadataService {
    async fn get_audio_metadata(
        &self,
        file_path: &str,
    ) -> Result<Option<AudioMetadata>, MetadataServiceError>;
}

#[async_trait]
pub(crate) trait MusicSearchService {
    async fn search(&self, query: &str)
        -> Result<Vec<SearchResultsEntry>, MusicSearchServiceError>;

    async fn get_download_url(
        &self,
        topic_id: &TopicId,
    ) -> Result<Option<Vec<u8>>, MusicSearchServiceError>;
}
