use std::ops::Deref;

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
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

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub struct DownloadId(pub(crate) u64);

impl Into<DownloadId> for u64 {
    fn into(self) -> DownloadId {
        DownloadId(self)
    }
}

impl Deref for DownloadId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for DownloadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
