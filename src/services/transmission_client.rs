use async_lock::Mutex;
use base64::{engine::general_purpose::STANDARD, Engine};
use request_processors::{Torrent, TorrentClientError};
use std::ops::Deref;
use transmission_rpc::types::{
    BasicAuth, Id, RpcResponse, TorrentAddArgs, TorrentAddedOrDuplicate,
};
use transmission_rpc::TransClient;

pub(crate) struct TransmissionClient {
    client: Mutex<TransClient>,
    download_dir: String,
}

pub(crate) struct TorrentId(pub(crate) i64);

impl Deref for TorrentId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum TransmissionClientError {
    #[error("Torrent already exists")]
    AlreadyExists,
    #[error("Erroneous result: {0}")]
    ErroneousResult(String),
    #[error("Unable to perform RPC request on transmission server: {0}")]
    TransmissionError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub(crate) type TransmissionClientResult<T> = Result<T, TransmissionClientError>;

impl TransmissionClient {
    pub(crate) fn create(
        url: String,
        username: Option<String>,
        password: Option<String>,
        download_dir: String,
    ) -> Self {
        let url = (&url).parse().unwrap();
        let client = match (username, password) {
            (Some(user), Some(password)) => {
                TransClient::with_auth(url, BasicAuth { user, password })
            }
            _ => TransClient::new(url),
        };

        Self {
            client: Mutex::new(client),
            download_dir,
        }
    }

    pub(crate) async fn add(
        &self,
        torrent_file_content: Vec<u8>,
    ) -> TransmissionClientResult<TorrentId> {
        let metainfo = STANDARD.encode(torrent_file_content);

        let RpcResponse { arguments, result } = self
            .client
            .lock()
            .await
            .torrent_add(TorrentAddArgs {
                metainfo: Some(metainfo.clone()),
                download_dir: Some(format!("{}/", self.download_dir.clone(),)),
                ..TorrentAddArgs::default()
            })
            .await?;

        if result != "success" {
            return Err(TransmissionClientError::ErroneousResult(result));
        }

        let torrent = match arguments {
            TorrentAddedOrDuplicate::TorrentAdded(torrent_added) => torrent_added,
            TorrentAddedOrDuplicate::TorrentDuplicate(_) => {
                return Err(TransmissionClientError::AlreadyExists);
            }
        };

        Ok(TorrentId(torrent.id.unwrap()))
    }

    pub(crate) async fn remove(&self, torrent_id: &TorrentId) -> TransmissionClientResult<()> {
        let RpcResponse { result, .. } = self
            .client
            .lock()
            .await
            .torrent_remove(vec![Id::Id(**torrent_id)], false)
            .await?;

        if result != "success" {
            return Err(TransmissionClientError::ErroneousResult(result));
        }

        Ok(())
    }

    pub(crate) async fn remove_with_data(
        &self,
        torrent_id: &TorrentId,
    ) -> TransmissionClientResult<()> {
        let id = Id::Id(**torrent_id);
        let RpcResponse { result, .. } = self
            .client
            .lock()
            .await
            .torrent_remove(vec![id], true)
            .await?;

        if result != "success" {
            return Err(TransmissionClientError::ErroneousResult(result));
        }

        Ok(())
    }

    pub(crate) async fn get(&self, torrent_id: &TorrentId) -> TransmissionClientResult<()> {
        let id = Id::Id(**torrent_id);
        let RpcResponse { result, arguments } = self
            .client
            .lock()
            .await
            .torrent_get(None, Some(vec![id]))
            .await?;

        if result != "success" {
            return Err(TransmissionClientError::ErroneousResult(result));
        }

        Ok(())
    }
}

impl request_processors::TorrentClient for TransmissionClient {
    async fn create(
        &self,
        path_to_download: &str,
        torrent_file_data: Vec<u8>,
    ) -> Result<request_processors::TorrentId, TorrentClientError> {
        todo!()
    }

    async fn get(
        &self,
        torrent_id: &request_processors::TorrentId,
    ) -> Result<Torrent, TorrentClientError> {
        todo!()
    }

    async fn delete(
        &self,
        torrent_id: &request_processors::TorrentId,
    ) -> Result<(), TorrentClientError> {
        todo!()
    }
}
