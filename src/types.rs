use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
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
