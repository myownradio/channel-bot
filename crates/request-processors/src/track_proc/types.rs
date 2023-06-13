use crate::{DownloadId, TopicId};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RequestId(Uuid);

impl Deref for RequestId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<RequestId> for Uuid {
    fn into(self) -> RequestId {
        RequestId(self)
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RadioManagerTrackId(pub(crate) u64);

impl std::fmt::Display for RadioManagerTrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RadioManagerChannelId(pub(crate) u64);

impl std::fmt::Display for RadioManagerChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RadioManagerLinkId(pub(crate) String);

impl std::fmt::Display for RadioManagerLinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub(crate) struct AudioMetadata {
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum TorrentStatus {
    Downloading,
    Complete,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Torrent {
    pub(crate) status: TorrentStatus,
    pub(crate) files: Vec<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct TopicData {
    pub(crate) topic_id: TopicId,
    pub(crate) download_id: DownloadId,
    pub(crate) title: String,
}
