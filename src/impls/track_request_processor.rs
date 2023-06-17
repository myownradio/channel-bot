use crate::services::track_request_processor::{
    AudioMetadata, DownloadId, MetadataServiceError, MetadataServiceTrait, RadioManagerChannelId,
    RadioManagerChannelTrack, RadioManagerClientError, RadioManagerClientTrait, RadioManagerLinkId,
    RadioManagerTrackId, RequestId, SearchProviderError, SearchProviderTrait, StateStorageError,
    StateStorageTrait, TopicData, TopicId, Torrent, TorrentClientError, TorrentClientTrait,
    TorrentId, TorrentStatus, TrackRequestProcessingContext, TrackRequestProcessingState,
    TrackRequestProcessingStatus,
};
use crate::services::{
    radio_manager_client, track_request_processor, MetadataService, RadioManagerClient,
    TransmissionClient,
};
use crate::storage::on_disk::OnDiskStorage;
use crate::types::UserId;
use async_trait::async_trait;
use audiotags::Tag;
use search_providers::RuTrackerClient;
use std::collections::HashMap;
use tracing::error;
use uuid::Uuid;

#[async_trait]
impl StateStorageTrait for OnDiskStorage {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingState,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = format!("{}", request_id);
        let state_str = serde_json::to_string(&state).expect("Unable to serialize state");

        self.save(&prefix, &key, &state_str)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

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
        let state_str = serde_json::to_string(&ctx).expect("Unable to serialize context");

        self.save(&prefix, &key, &state_str)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

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

        self.save(&prefix, &key, &state_str)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

        Ok(())
    }

    async fn update_status(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackRequestProcessingStatus,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-status", user_id);
        let key = format!("{}", request_id);
        let state_str = serde_json::to_string(&state).expect("Unable to serialize status");

        self.save(&prefix, &key, &state_str)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

        Ok(())
    }

    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingState, StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = format!("{}", request_id);
        let value = match self
            .get(&prefix, &key)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?
        {
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
        let value = match self
            .get(&prefix, &key)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?
        {
            Some(value) => serde_json::from_str(&value).expect("Unable to deserialize context"),
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

        self.delete(&prefix, &key)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

        Ok(())
    }

    async fn delete_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = format!("{}", request_id);

        self.delete(&prefix, &key)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

        Ok(())
    }

    async fn delete_status(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-status", user_id);
        let key = format!("{}", request_id);

        self.delete(&prefix, &key)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

        Ok(())
    }

    async fn get_all_statuses(
        &self,
        user_id: &UserId,
    ) -> Result<HashMap<RequestId, TrackRequestProcessingStatus>, StateStorageError> {
        let prefix = format!("{}-status", user_id);
        let values = self
            .get_all(&prefix)
            .await
            .map_err(|error| StateStorageError(Box::new(error)))?;

        let mut results = HashMap::new();

        for (key, value) in values {
            let request_id = RequestId(
                key.parse::<Uuid>()
                    .map_err(|error| StateStorageError(Box::new(error)))?,
            );
            let status =
                serde_json::from_str(&value).map_err(|error| StateStorageError(Box::new(error)))?;

            results.insert(request_id, status);
        }

        Ok(results)
    }
}

#[async_trait]
impl TorrentClientTrait for TransmissionClient {
    async fn add_torrent(
        &self,
        torrent_file_data: Vec<u8>,
    ) -> Result<TorrentId, TorrentClientError> {
        let torrent_id = self
            .add(torrent_file_data)
            .await
            .map_err(|err| TorrentClientError(Box::from(err)))?;

        Ok(TorrentId(torrent_id))
    }

    async fn get_torrent(&self, torrent_id: &TorrentId) -> Result<Torrent, TorrentClientError> {
        let torrent = self
            .get(torrent_id)
            .await
            .map_err(|err| TorrentClientError(Box::from(err)))?;

        Ok(Torrent {
            status: match torrent.status {
                Some(transmission_rpc::types::TorrentStatus::Seeding) => TorrentStatus::Complete,
                _ => TorrentStatus::Downloading,
            },
            files: torrent
                .files
                .unwrap_or_default()
                .into_iter()
                .map(|f| f.name)
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

impl Into<RadioManagerChannelTrack> for radio_manager_client::RadioManagerChannelTrack {
    fn into(self) -> RadioManagerChannelTrack {
        RadioManagerChannelTrack {
            title: self.title,
            album: self.album,
            artist: self.artist,
        }
    }
}

#[async_trait]
impl RadioManagerClientTrait for RadioManagerClient {
    async fn upload_audio_track(
        &self,
        _user_id: &UserId,
        path_to_audio_file: &str,
    ) -> Result<RadioManagerTrackId, RadioManagerClientError> {
        let track_id = self
            .upload_track(path_to_audio_file)
            .await
            .map_err(|error| RadioManagerClientError(Box::new(error)))?;

        Ok(track_id)
    }

    async fn add_track_to_channel_playlist(
        &self,
        _user_id: &UserId,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError> {
        let link_id = self
            .add_track_to_channel(track_id, channel_id)
            .await
            .map_err(|error| RadioManagerClientError(Box::new(error)))?;

        Ok(link_id)
    }

    async fn get_channel_tracks(
        &self,
        channel_id: &RadioManagerChannelId,
    ) -> Result<Vec<RadioManagerChannelTrack>, RadioManagerClientError> {
        let tracks = RadioManagerClient::get_channel_tracks(self, channel_id)
            .await
            .map_err(|error| RadioManagerClientError(Box::new(error)))?;

        Ok(tracks.into_iter().map(Into::into).collect())
    }
}
