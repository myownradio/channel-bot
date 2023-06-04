//
// Downloader
//
#[derive(Eq, PartialEq, Clone)]
pub(crate) struct DownloadId(String);

pub(crate) enum DownloadingStatus {
    Downloading,
    Finished,
}

pub(crate) struct TrackDownloadEntry {
    pub(crate) id: DownloadId,
    pub(crate) status: DownloadingStatus,
    pub(crate) files: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TrackDownloaderError {
    #[error("Unexpected error")]
    Unexpected,
}

//
// Playlist Provider
//

#[derive(Clone)]
pub(crate) struct PlaylistEntry {
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PlaylistProviderError {
    #[error("Unexpected error")]
    Unexpected,
}

//
// Radio Manager
//

pub(crate) struct RadioManagerPlaylistEntry {
    id: String,
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RadioManagerError {
    #[error("Unexpected error")]
    Unexpected,
}

// Audio Metadata Service
pub(crate) struct AudioMetadata {
    title: String,
    artist: String,
    album: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MetadataServiceError {
    #[error("Unexpected error")]
    Unexpected,
}

// Audio Search Service
#[derive(Eq, PartialEq, Clone, Hash)]
pub(crate) struct TopicId(String);

pub(crate) struct SearchResultsEntry {
    pub(crate) title: String,
    pub(crate) topic_id: TopicId,
    pub(crate) tracks_hint: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MusicSearchServiceError {
    #[error("Unexpected error")]
    Unexpected,
}
