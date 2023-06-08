use serde::Serialize;
use std::ops::Deref;

#[derive(Eq, PartialEq, Clone, Debug, Serialize)]
pub(crate) struct DownloadId(String);

impl Into<DownloadId> for &str {
    fn into(self) -> DownloadId {
        DownloadId(self.to_string())
    }
}

impl Deref for DownloadId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for DownloadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct TopicId(String);

impl Into<TopicId> for &str {
    fn into(self) -> TopicId {
        TopicId(self.to_string())
    }
}

impl Deref for TopicId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TopicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioManagerTrackId(u64);

impl Into<RadioManagerTrackId> for u64 {
    fn into(self) -> RadioManagerTrackId {
        RadioManagerTrackId(self)
    }
}

impl std::fmt::Display for RadioManagerTrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioManagerChannelId(u64);

impl Into<RadioManagerChannelId> for u64 {
    fn into(self) -> RadioManagerChannelId {
        RadioManagerChannelId(self)
    }
}

impl std::fmt::Display for RadioManagerChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioManagerLinkId(String);

impl Into<RadioManagerLinkId> for &str {
    fn into(self) -> RadioManagerLinkId {
        RadioManagerLinkId(self.into())
    }
}

impl std::fmt::Display for RadioManagerLinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct UserId(u64);

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

#[derive(Clone, PartialEq, Debug, Default)]
pub(crate) struct AudioMetadata {
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
}
