#[derive(Debug, thiserror::Error)]
pub(crate) enum RuTrackerClientError {
    #[error("Unexpected error")]
    Unexpected,
}

pub(crate) struct RuTrackerClient {}

impl RuTrackerClient {
    pub(crate) async fn create(login: &str, password: &str) -> Result<Self, RuTrackerClientError> {
        todo!()
    }
}
