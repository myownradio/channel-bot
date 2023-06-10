use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
use serde::Serialize;

const RU_TRACKER_HOST: &str = "https://rutracker.net";
const MAGIC_LOGIN_WORD: &str = "вход";
const CAPTCHA_IS_REQUIRED_TEXT: &str = "введите код подтверждения";
const INCORRECT_PASSWORD_TEXT: &str = "неверный пароль";
const SUCCESSFUL_LOGIN_TEXT: &str = "log-out-icon";

#[derive(Debug, thiserror::Error)]
pub(crate) enum RuTrackerClientError {
    #[error("Captcha verification is required.")]
    CaptchaVerificationIsRequired,
    #[error("Incorrect login or password.")]
    IncorrectLoginOrPassword,
    #[error("Unexpected")]
    UnknownAuthError,
}

pub(crate) struct RuTrackerClient {
    client: Client,
}

impl RuTrackerClient {
    pub(crate) async fn create(
        username: &str,
        password: &str,
    ) -> Result<Self, RuTrackerClientError> {
        let client = Client::builder()
            .redirect(Policy::none())
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP Client");

        #[derive(Serialize)]
        struct LoginForm {
            login_username: String,
            login_password: String,
            login: String,
        }

        let form = LoginForm {
            login_username: username.to_string(),
            login_password: password.to_string(),
            login: MAGIC_LOGIN_WORD.to_string(),
        };

        let response = client
            .post(format!("{}/login.php", RU_TRACKER_HOST))
            .form(&form)
            .send()
            .await?;

        let html = response.text().await?;

        if html.contains(CAPTCHA_IS_REQUIRED_TEXT) {
            return Err(RuTrackerClientError::CaptchaVerificationIsRequired);
        }

        if html.contains(INCORRECT_PASSWORD_TEXT) {
            return Err(RuTrackerClientError::IncorrectLoginOrPassword);
        }

        if !html.contains(SUCCESSFUL_LOGIN_TEXT) {
            return Err(RuTrackerClientError::UnknownAuthError);
        }

        Ok(Self { client })
    }
}
