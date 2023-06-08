use serde::Serialize;

#[derive(Eq, PartialEq, Clone, Debug, Serialize)]
pub(crate) struct DownloadId(String);

impl Into<DownloadId> for &str {
    fn into(self) -> DownloadId {
        DownloadId(self.to_string())
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct TopicId(String);

impl Into<TopicId> for &str {
    fn into(self) -> TopicId {
        TopicId(self.to_string())
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioterioTrackId(u64);

impl Into<RadioterioTrackId> for u64 {
    fn into(self) -> RadioterioTrackId {
        RadioterioTrackId(self)
    }
}

impl std::fmt::Display for RadioterioTrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioterioChannelId(u64);

impl Into<RadioterioChannelId> for u64 {
    fn into(self) -> RadioterioChannelId {
        RadioterioChannelId(self)
    }
}

impl std::fmt::Display for RadioterioChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioterioLinkId(String);

impl Into<RadioterioLinkId> for &str {
    fn into(self) -> RadioterioLinkId {
        RadioterioLinkId(self.into())
    }
}

impl std::fmt::Display for RadioterioLinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub(crate) struct UserId(pub(crate) u64);

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
