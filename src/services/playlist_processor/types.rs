//
// Downloader
//
#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct DownloadId(pub(crate) String);

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

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct PlaylistEntry {
    pub(crate) metadata: AudioMetadata,
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
    pub(crate) id: String,
    pub(crate) metadata: AudioMetadata,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RadioManagerError {
    #[error("Unexpected error")]
    Unexpected,
}

// Audio Metadata Service
#[derive(Clone, PartialEq, Debug, Default)]
pub(crate) struct AudioMetadata {
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MetadataServiceError {
    #[error("Unexpected error")]
    Unexpected,
}

// Audio Search Service
#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub(crate) struct TopicId(pub(crate) String);

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
