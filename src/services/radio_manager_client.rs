use reqwest::redirect::Policy;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

pub(crate) struct RadioManagerClient {
    endpoint: String,
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RadioManagerClientError {
    #[error(transparent)]
    ReqwestError(#[from] Error),
    #[error("Incorrect username or password")]
    Unauthorized,
}

impl RadioManagerClient {
    pub(crate) async fn create(
        endpoint: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, RadioManagerClientError> {
        let client = Client::builder()
            .redirect(Policy::limited(10))
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP Client");

        #[derive(Serialize)]
        struct LoginForm {
            login: String,
            password: String,
            save: bool,
        }

        let form = LoginForm {
            login: username.to_string(),
            password: password.to_string(),
            save: false,
        };

        #[derive(Deserialize)]
        struct LoginResult {
            message: String,
        }

        let response = client
            .post(format!("{}api/v2/user/login", endpoint))
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<LoginResult>()
            .await?;

        if &response.message != "OK" {
            return Err(RadioManagerClientError::Unauthorized);
        }

        Ok(Self {
            endpoint: endpoint.into(),
            client,
        })
    }
}
