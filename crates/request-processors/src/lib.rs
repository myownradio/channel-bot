mod track_proc;
pub use track_proc::*;

use serde::{Deserialize, Serialize};
use std::ops::Deref;

// UserId
#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub(crate) struct UserId(pub(crate) u64);

impl Deref for UserId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<UserId> for u64 {
    fn into(self) -> UserId {
        UserId(self)
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TopicId
#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub struct TopicId(pub(crate) u64);

impl Deref for TopicId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TopicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// DownloadId
#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub struct DownloadId(pub(crate) u64);

impl Deref for DownloadId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TorrentId
#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct TorrentId(pub(crate) i64);

impl Deref for TorrentId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<TorrentId> for i64 {
    fn into(self) -> TorrentId {
        TorrentId(self)
    }
}

impl std::fmt::Display for TorrentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
