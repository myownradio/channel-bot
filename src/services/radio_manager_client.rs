use crate::services::track_request_processor::{
    RadioManagerChannelId, RadioManagerLinkId, RadioManagerTrackId,
};
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
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

#[derive(Debug, Deserialize)]
pub(crate) struct RadioManagerResponse<Data> {
    code: i64,
    message: String,
    data: Data,
}

impl<Data> RadioManagerResponse<Data> {
    fn error_for_code(self) -> Result<Data, RadioManagerClientError> {
        match self.code {
            1 => Ok(self.data),
            _ => Err(RadioManagerClientError::Unexpected(self.message)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RadioManagerUploadedTrack {
    pub(crate) tid: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RadioManagerChannelTrack {
    pub(crate) album: String,
    pub(crate) artist: String,
    pub(crate) title: String,
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

        let data = self
            .client
            .post(format!("{}api/v2/track/upload", self.endpoint))
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .json::<RadioManagerResponse<Vec<RadioManagerUploadedTrack>>>()
            .await?
            .error_for_code()?;

        let first_track_id = match data.first().map(|t| t.tid) {
            Some(track_id) => track_id,
            None => {
                return Err(RadioManagerClientError::Unexpected(String::from(
                    "No tracks were uploaded",
                )))
            }
        };

        Ok(RadioManagerTrackId(first_track_id))
    }

    pub(crate) async fn add_track_to_channel(
        &self,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError> {
        #[derive(Serialize)]
        struct AddToChannelForm {
            stream_id: u64,
            tracks: String,
        }

        let form = AddToChannelForm {
            stream_id: **channel_id,
            tracks: format!("{}", track_id),
        };

        #[derive(Deserialize)]
        struct AddToChannelResult {
            message: String,
        }

        let response = self
            .client
            .post(format!("{}api/v2/stream/addTracks", self.endpoint))
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json::<AddToChannelResult>()
            .await?;

        if response.message != "OK" {
            return Err(RadioManagerClientError::Unexpected(response.message));
        }

        Ok(RadioManagerLinkId("123".into()))
    }

    pub(crate) async fn get_channel_tracks(
        &self,
        channel_id: &RadioManagerChannelId,
    ) -> Result<Vec<RadioManagerChannelTrack>, RadioManagerClientError> {
        let response = self
            .client
            .get(format!(
                "{}radio-manager/api/v0/streams/{}/tracks",
                self.endpoint, channel_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<RadioManagerResponse<Vec<RadioManagerChannelTrack>>>()
            .await?
            .error_for_code()?;

        Ok(response)
    }
}
