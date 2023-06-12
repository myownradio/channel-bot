mod rutracker;

use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub use rutracker::*;

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub title: String,
    pub topic_id: TopicId,
    pub seeds_number: u64,
}

pub type SearchResults = Vec<SearchResult>;

#[derive(Debug, PartialEq)]
pub struct Topic {
    pub topic_id: TopicId,
    pub download_id: DownloadId,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub struct TopicId(pub(crate) u64);

impl Into<TopicId> for u64 {
    fn into(self) -> TopicId {
        TopicId(self)
    }
}

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

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub struct DownloadId(pub(crate) u64);

impl Deref for DownloadId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
