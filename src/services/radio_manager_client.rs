use crate::services::track_request_processor::{
    RadioManagerChannelId, RadioManagerLinkId, RadioManagerTrackId,
};
use reqwest::redirect::Policy;
use reqwest::{multipart, Body, Client, Error};
use serde::Deserialize;
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
enum RadioManagerUploadedTrackData {
    Null,
    Tracks {
        tracks: Vec<RadioManagerUploadedTrack>,
    },
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

#[derive(Debug, Deserialize)]
pub(crate) struct RadioManagerTrack {
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

        client
            .post(format!("{}api/v2/user/login", endpoint))
            .form(&serde_json::json!({
                "login": username.to_string(),
                "password": password.to_string(),
                "save": false,
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<RadioManagerResponse<()>>()
            .await?
            .error_for_code()?;

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
            .json::<RadioManagerResponse<RadioManagerUploadedTrackData>>()
            .await?
            .error_for_code()?;

        match data {
            RadioManagerUploadedTrackData::Tracks { tracks } if tracks.len() > 0 => Ok(
                RadioManagerTrackId(tracks.first().map(|t| t.tid).unwrap_or_default()),
            ),
            _ => Err(RadioManagerClientError::Unexpected(String::from(
                "No tracks were uploaded",
            ))),
        }
    }

    pub(crate) async fn add_track_to_channel(
        &self,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError> {
        self.client
            .post(format!("{}api/v2/stream/addTracks", self.endpoint))
            .form(&serde_json::json!({
                "stream_id": **channel_id,
                "tracks": format!("{}", track_id),
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<RadioManagerResponse<()>>()
            .await?
            .error_for_code()?;

        Ok(RadioManagerLinkId("123".into()))
    }

    pub(crate) async fn get_channel_tracks(
        &self,
        channel_id: &RadioManagerChannelId,
    ) -> Result<Vec<RadioManagerChannelTrack>, RadioManagerClientError> {
        let mut tracks = vec![];

        let mut offset = 0;
        loop {
            let mut data = self
                .client
                .get(format!(
                    "{}radio-manager/api/v0/streams/{}/tracks",
                    self.endpoint, channel_id
                ))
                .query(&serde_json::json!({
                    "offset": offset,
                }))
                .send()
                .await?
                .error_for_status()?
                .json::<RadioManagerResponse<Vec<RadioManagerChannelTrack>>>()
                .await?
                .error_for_code()?;

            if data.is_empty() {
                break;
            }

            offset += data.len();

            tracks.append(&mut data);
        }

        Ok(tracks)
    }

    pub(crate) async fn get_tracks(
        &self,
    ) -> Result<Vec<RadioManagerTrack>, RadioManagerClientError> {
        let mut tracks = vec![];

        let mut offset = 0;
        loop {
            let mut data = self
                .client
                .get(format!("{}radio-manager/api/v0/tracks/", self.endpoint))
                .query(&serde_json::json!({
                    "offset": offset,
                }))
                .send()
                .await?
                .error_for_status()?
                .json::<RadioManagerResponse<Vec<RadioManagerTrack>>>()
                .await?
                .error_for_code()?;

            if data.is_empty() {
                break;
            }

            offset += data.len();

            tracks.append(&mut data);
        }

        Ok(tracks)
    }
}
