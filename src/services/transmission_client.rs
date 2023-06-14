use crate::services::track_request_processor::{
    Torrent, TorrentClientError, TorrentClientTrait, TorrentId,
};
use async_lock::Mutex;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::ops::Deref;
use transmission_rpc::types::{
    BasicAuth, Id, RpcResponse, TorrentAddArgs, TorrentAddedOrDuplicate,
};
use transmission_rpc::TransClient;

pub(crate) struct TransmissionClient {
    client: Mutex<TransClient>,
    download_dir: String,
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
