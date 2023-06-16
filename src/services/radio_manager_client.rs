use crate::services::track_request_processor::RadioManagerTrackId;
use reqwest::redirect::Policy;
use reqwest::{multipart, Body, Client, Error};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio_util::codec::{BytesCodec, FramedRead};

pub(crate) struct RadioManagerClient {
    endpoint: String,
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RadioManagerClientError {
    #[error(transparent)]
    ReqwestError(#[from] Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Incorrect username or password")]
    UnauthorizedError,
}

#[derive(Debug, Deserialize)]
struct UploadedTrack {}

#[derive(Debug, Deserialize)]
struct TrackUploadResponseData {
    tracks: Vec<UploadedTrack>,
}

#[derive(Debug, Deserialize)]
struct TrackUploadResponse {
    message: String,
    data: TrackUploadResponseData,
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
            return Err(RadioManagerClientError::UnauthorizedError);
        }

        Ok(Self {
            endpoint: endpoint.into(),
            client,
        })
    }

    pub(crate) async fn upload_track(
        &self,
        path_to_track_file: &str,
    ) -> Result<RadioManagerTrackId, RadioManagerClientError> {
        let path = Path::new(path_to_track_file);
        let file = tokio::fs::File::open(path).await?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);
        let file_part = multipart::Part::stream(file_body).file_name(
            path.file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_string(),
        );

        let form = multipart::Form::new().part("file", file_part);

        let response = self
            .client
            .post(format!("{}api/v2/track/upload", self.endpoint))
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .json::<TrackUploadResponse>()
            .await?;

        eprintln!("res: {:?}", response);

        Ok(RadioManagerTrackId(0))
    }
}
