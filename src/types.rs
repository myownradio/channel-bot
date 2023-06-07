use serde::Serialize;

#[derive(Eq, PartialEq, Clone, Debug, Serialize)]
pub(crate) struct DownloadId(pub(crate) String);

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct TopicId(pub(crate) String);

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioterioTrackId(pub(crate) u64);

impl std::fmt::Display for RadioterioTrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioterioChannelId(pub(crate) u64);

impl std::fmt::Display for RadioterioChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize)]
pub(crate) struct RadioterioLinkId;

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
