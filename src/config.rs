use serde::Deserialize;

fn default_bind_address() -> String {
    "0.0.0.0:8080".to_string()
}

fn default_shutdown_timeout() -> u64 {
    30u64
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RuTrackerCredentials {
    #[serde(rename = "rutracker_username")]
    pub(crate) username: String,
    #[serde(rename = "rutracker_password")]
    pub(crate) password: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default = "default_bind_address")]
    pub(crate) bind_address: String,
    #[serde(default = "default_shutdown_timeout")]
    pub(crate) shutdown_timeout: u64,
    #[serde(flatten)]
    pub(crate) rutracker_credentials: RuTrackerCredentials,
}

impl Config {
    pub(crate) fn from_env() -> Self {
        match envy::from_env::<Self>() {
            Ok(config) => config,
            Err(error) => panic!("Missing environment variable: {:#?}", error),
        }
    }
}
