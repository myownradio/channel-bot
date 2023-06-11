use crate::services::search_provider::parser::{
    parse_and_validate_auth_state, parse_search_results, AuthError, ParseError, SearchResult,
};
use reqwest::redirect::Policy;
use reqwest::{Client, StatusCode};
use serde::Serialize;

const RU_TRACKER_HOST: &str = "https://rutracker.net";
const MAGIC_LOGIN_WORD: &str = "вход";

#[derive(Debug, thiserror::Error)]
pub(crate) enum RuTrackerClientError {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    AuthError(#[from] AuthError),
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
            .redirect(Policy::limited(10))
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
            .post(format!("{}/forum/login.php", RU_TRACKER_HOST))
            .form(&form)
            .send()
            .await?;

        let raw_html = response.text().await?;

        parse_and_validate_auth_state(&raw_html)?;

        Ok(Self { client })
    }

    pub(crate) async fn search_music(
        &self,
        query_str: &str,
    ) -> Result<Vec<SearchResult>, RuTrackerClientError> {
        #[derive(Serialize)]
        struct Query {
            nm: String,
        }

        let query = Query {
            nm: query_str.to_string(),
        };

        let response = self
            .client
            .get(format!("{}/forum/tracker.php", RU_TRACKER_HOST))
            .query(&query)
            .send()
            .await?;

        let raw_html = response.text().await?;

        parse_and_validate_auth_state(&raw_html)?;

        Ok(parse_search_results(&raw_html)?)
    }

    pub(crate) async fn download_torrent(
        &self,
        torrent_id: u64,
    ) -> Result<Vec<u8>, RuTrackerClientError> {
        todo!();
    }
}

#[cfg(test)]
mod tests {}
