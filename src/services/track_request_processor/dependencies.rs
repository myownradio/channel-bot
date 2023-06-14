use crate::services::track_request_processor::{
    Torrent, TorrentClientError, TorrentClientTrait, TorrentId,
};
use crate::services::TransmissionClient;
use async_trait::async_trait;
use std::sync::Arc;

pub(crate) struct TorrentClient(pub(crate) Arc<TransmissionClient>);

#[async_trait]
impl TorrentClientTrait for TorrentClient {
    async fn create(
        &self,
        path_to_download: &str,
        torrent_file_data: Vec<u8>,
    ) -> Result<TorrentId, TorrentClientError> {
        todo!()
    }

    async fn get(&self, torrent_id: &TorrentId) -> Result<Torrent, TorrentClientError> {
        todo!()
    }

    async fn delete(&self, torrent_id: &TorrentId) -> Result<(), TorrentClientError> {
        todo!()
    }
}
