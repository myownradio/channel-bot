pub(crate) struct RadioManagerClient {
    endpoint: String,
}

impl RadioManagerClient {
    pub(crate) fn create(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}
