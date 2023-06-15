use crate::services::track_request_processor::{
    AudioMetadata, DownloadId, MetadataServiceError, MetadataServiceTrait, RadioManagerChannelId,
    RadioManagerClientError, RadioManagerClientTrait, RadioManagerLinkId, RadioManagerTrackId,
    RequestId, SearchProviderError, SearchProviderTrait, StateStorageError, StateStorageTrait,
    TopicData, TopicId, Torrent, TorrentClientError, TorrentClientTrait, TorrentId, TorrentStatus,
    TrackRequestProcessingContext, TrackRequestProcessingState,
};
use crate::services::{MetadataService, RadioManagerClient, TransmissionClient};
use crate::storage::InMemoryStorage;
use crate::types::UserId;
use async_trait::async_trait;
use audiotags::Tag;
use search_providers::RuTrackerClient;
use tracing::error;

#[async_trait]
impl StateStorageTrait for InMemoryStorage {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingState,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = format!("{}", request_id);
        let state_str = serde_json::to_string(&state).expect("Unable to serialize state");

        self.save(&prefix, &key, &state_str);

        Ok(())
    }

    async fn create_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        ctx: TrackRequestProcessingContext,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = format!("{}", request_id);
        let state_str = serde_json::to_string(&ctx).expect("Unable to serialize state");

        self.save(&prefix, &key, &state_str);

        Ok(())
    }

    async fn update_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackRequestProcessingState,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = format!("{}", request_id);
        let state_str = serde_json::to_string(&state).expect("Unable to serialize state");

        self.save(&prefix, &key, &state_str);

        Ok(())
    }

    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingState, StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = format!("{}", request_id);
        let value = match self.get(&prefix, &key) {
            Some(value) => serde_json::from_str(&value).expect("Unable to deserialize state"),
            None => return Err(StateStorageError::not_found()),
        };

        Ok(value)
    }

    async fn load_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingContext, StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = format!("{}", request_id);
        let value = match self.get(&prefix, &key) {
            Some(value) => serde_json::from_str(&value).expect("Unable to deserialize state"),
            None => return Err(StateStorageError::not_found()),
        };

        Ok(value)
    }

    async fn delete_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = format!("{}", request_id);

        self.delete(&prefix, &key);

        Ok(())
    }

    async fn delete_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = format!("{}", request_id);

        self.delete(&prefix, &key);

        Ok(())
    }
}

#[async_trait]
impl TorrentClientTrait for TransmissionClient {
    async fn add_torrent(
        &self,
        path_to_download: &str,
        torrent_file_data: Vec<u8>,
    ) -> Result<TorrentId, TorrentClientError> {
        let torrent_id = self
            .add(path_to_download, torrent_file_data)
            .await
            .map_err(|err| TorrentClientError(Box::from(err)))?;

        Ok(TorrentId(torrent_id))
    }

    async fn get_torrent(&self, torrent_id: &TorrentId) -> Result<Torrent, TorrentClientError> {
        let torrent = self
            .get(torrent_id)
            .await
            .map_err(|err| TorrentClientError(Box::from(err)))?;
        let download_dir = torrent.download_dir.clone().unwrap_or_default();

        Ok(Torrent {
            status: match torrent.status {
                Some(transmission_rpc::types::TorrentStatus::Seeding) => TorrentStatus::Complete,
                _ => TorrentStatus::Downloading,
            },
            files: torrent
                .files
                .unwrap_or_default()
                .into_iter()
                .map(|f| format!("{}/{}", download_dir, f.name))
                .collect(),
        })
    }

    async fn delete_torrent(&self, torrent_id: &TorrentId) -> Result<(), TorrentClientError> {
        self.remove_with_data(torrent_id)
            .await
            .map_err(|err| TorrentClientError(Box::from(err)))?;

        Ok(())
    }
}

impl Into<TopicData> for search_providers::TopicData {
    fn into(self) -> TopicData {
        TopicData {
            title: self.title,
            download_id: DownloadId(*self.download_id),
            topic_id: TopicId(*self.topic_id),
        }
    }
}

#[async_trait]
impl SearchProviderTrait for RuTrackerClient {
    async fn search_music(&self, query: &str) -> Result<Vec<TopicData>, SearchProviderError> {
        self.search_music(query)
            .await
            .map(|results| results.into_iter().map(Into::into).collect())
            .map_err(|error| SearchProviderError(Box::new(error)))
    }

    async fn download_torrent(
        &self,
        download_id: &DownloadId,
    ) -> Result<Vec<u8>, SearchProviderError> {
        RuTrackerClient::download_torrent(&self, **download_id)
            .await
            .map_err(|error| SearchProviderError(Box::new(error)))
    }
}

#[async_trait]
impl MetadataServiceTrait for MetadataService {
    #[tracing::instrument(skip(self))]
    async fn get_audio_metadata(
        &self,
        file_path: &str,
    ) -> Result<Option<AudioMetadata>, MetadataServiceError> {
        match Tag::new().read_from_path(file_path) {
            Ok(tags) => Ok(Some(AudioMetadata {
                title: tags.title().unwrap_or_default().to_string(),
                artist: tags.artist().unwrap_or_default().to_string(),
                album: tags.album_title().unwrap_or_default().to_string(),
            })),
            Err(error) => {
                error!(?error, "Unable to read audio file metadata");
                Err(MetadataServiceError(Box::new(error)))
            }
        }
    }
}

#[async_trait]
impl RadioManagerClientTrait for RadioManagerClient {
    async fn upload_audio_track(
        &self,
        user_id: &UserId,
        path_to_audio_file: &str,
    ) -> Result<RadioManagerTrackId, RadioManagerClientError> {
        todo!()
    }

    async fn add_track_to_channel_playlist(
        &self,
        user_id: &UserId,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError> {
        todo!()
    }
}
