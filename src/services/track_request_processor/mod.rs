mod traits;
pub(crate) use traits::*;

mod types;
pub(crate) use types::*;

#[cfg(test)]
mod processor_tests;

#[cfg(test)]
mod types_tests;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackRequestProcessingContext {
    pub(crate) metadata: AudioMetadata,
    pub(crate) target_channel_id: RadioManagerChannelId,
}

impl TrackRequestProcessingContext {
    pub(crate) fn new(
        track_metadata: AudioMetadata,
        target_channel_id: RadioManagerChannelId,
    ) -> Self {
        Self {
            metadata: track_metadata,
            target_channel_id,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct TrackRequestProcessingState {
    pub(crate) tried_topics: Vec<TopicId>,
    pub(crate) current_download_id: Option<DownloadId>,
    pub(crate) current_torrent_data: Option<Vec<u8>>,
    pub(crate) current_torrent_id: Option<TorrentId>,
    pub(crate) path_to_downloaded_file: Option<String>,
    pub(crate) radio_manager_track_id: Option<RadioManagerTrackId>,
    pub(crate) radio_manager_link_id: Option<RadioManagerLinkId>,
}

impl TrackRequestProcessingState {
    pub(crate) fn get_step(&self) -> TrackRequestProcessingStep {
        if self.current_download_id.is_none() {
            TrackRequestProcessingStep::SearchAudioAlbum
        } else if self.current_torrent_data.is_none() {
            TrackRequestProcessingStep::GetAlbumURL
        } else if self.current_torrent_id.is_none() {
            TrackRequestProcessingStep::DownloadAlbum
        } else if self.path_to_downloaded_file.is_none() {
            TrackRequestProcessingStep::CheckDownloadStatus
        } else if self.radio_manager_track_id.is_none() {
            TrackRequestProcessingStep::UploadToRadioManager
        } else if self.radio_manager_link_id.is_none() {
            TrackRequestProcessingStep::AddToRadioManagerChannel
        } else {
            TrackRequestProcessingStep::Finish
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TrackRequestProcessingStep {
    SearchAudioAlbum,
    GetAlbumURL,
    DownloadAlbum,
    CheckDownloadStatus,
    UploadToRadioManager,
    AddToRadioManagerChannel,
    Finish,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateRequestError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessRequestError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
    #[error(transparent)]
    SearchProviderError(#[from] SearchProviderError),
    #[error(transparent)]
    DownloaderError(#[from] TorrentClientError),
    #[error(transparent)]
    MetadataServiceError(#[from] MetadataServiceError),
    #[error(transparent)]
    RadioManagerError(#[from] RadioManagerClientError),
}

pub struct TrackRequestProcessor {
    state_storage: Arc<dyn StateStorage + Send>,
    search_provider: Arc<dyn SearchProvider + Send>,
    torrent_client: Arc<dyn TorrentClient + Send>,
    metadata_service: Arc<dyn MetadataService + Send>,
    radio_manager_client: Arc<dyn RadioManagerClient + Send>,
}

impl TrackRequestProcessor {
    pub fn new(
        state_storage: Arc<dyn StateStorage + Send>,
        search_provider: Arc<dyn SearchProvider + Send>,
        torrent_client: Arc<dyn TorrentClient + Send>,
        metadata_service: Arc<dyn MetadataService + Send>,
        radio_manager_client: Arc<dyn RadioManagerClient + Send>,
    ) -> Self {
        Self {
            state_storage,
            search_provider,
            torrent_client,
            metadata_service,
            radio_manager_client,
        }
    }

    pub async fn create_request(
        &self,
        user_id: &UserId,
        track_metadata: &AudioMetadata,
        target_channel_id: &RadioManagerChannelId,
    ) -> Result<RequestId, CreateRequestError> {
        let request_id = RequestId(Uuid::new_v4());
        let ctx =
            TrackRequestProcessingContext::new(track_metadata.clone(), target_channel_id.clone());
        let state = TrackRequestProcessingState::default();

        self.state_storage
            .create_context(user_id, &request_id, ctx)
            .await?;
        self.state_storage
            .create_state(user_id, &request_id, state)
            .await?;

        info!(%user_id, %request_id, ?track_metadata, "New track request successfully created");

        Ok(request_id)
    }

    pub async fn process_request(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), ProcessRequestError> {
        info!(%user_id, %request_id, "Track processing request");

        let ctx = self.state_storage.load_context(user_id, request_id).await?;
        let mut state = self.state_storage.load_state(user_id, request_id).await?;

        while !matches!(state.get_step(), TrackRequestProcessingStep::Finish) {
            self.handle_next_step(user_id, &ctx, &mut state).await?;
            self.state_storage
                .update_state(user_id, request_id, &state)
                .await?;
        }

        info!(%user_id, %request_id, "Track processing finished");

        self.state_storage.delete_state(user_id, request_id).await?;
        self.state_storage
            .delete_context(user_id, request_id)
            .await?;

        debug!("Track processing state and context have been cleaned");

        Ok(())
    }

    async fn handle_next_step(
        &self,
        user_id: &UserId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let step = state.get_step();

        debug!(%user_id, ?step, "Running next processing step");

        match step {
            TrackRequestProcessingStep::SearchAudioAlbum => {
                self.search_audio_album(user_id, ctx, state).await?;
            }
            TrackRequestProcessingStep::GetAlbumURL => {
                self.get_album_url(user_id, ctx, state).await?;
            }
            TrackRequestProcessingStep::DownloadAlbum => {
                self.download_album(user_id, ctx, state).await?;
            }
            TrackRequestProcessingStep::CheckDownloadStatus => {
                self.check_download_status(user_id, ctx, state).await?;
            }
            TrackRequestProcessingStep::UploadToRadioManager => {
                self.upload_to_radio_manager(user_id, ctx, state).await?;
            }
            TrackRequestProcessingStep::AddToRadioManagerChannel => {
                self.add_to_radio_manager_channel(user_id, ctx, state)
                    .await?;
            }
            TrackRequestProcessingStep::Finish => (),
        }

        Ok(())
    }

    async fn search_audio_album(
        &self,
        _user_id: &UserId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let query = format!("{} - {}", ctx.metadata.artist, ctx.metadata.album);
        let results = self.search_provider.search_music(&query).await?;

        let tried_topics_set = state.tried_topics.iter().collect::<HashSet<_>>();
        let topic = match results
            .into_iter()
            .filter(|r| !tried_topics_set.contains(&r.topic_id))
            .next()
        {
            Some(topic) => topic,
            None => {
                error!("No more search results containing requested track...");
                todo!();
            }
        };

        debug!(
            ?topic,
            "Found topic possibly containing the requested track"
        );

        state.current_download_id.replace(topic.download_id);
        state.tried_topics.push(topic.topic_id);

        Ok(())
    }

    async fn get_album_url(
        &self,
        _user_id: &UserId,
        _ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let download_id = state
            .current_download_id
            .clone()
            .take()
            .expect("current_download_id should be defined");

        debug!("Downloading torrent possibly containing the audio track...");

        let torrent_data = self.search_provider.download_torrent(&download_id).await?;

        state.current_torrent_data.replace(torrent_data);

        Ok(())
    }

    async fn download_album(
        &self,
        _user_id: &UserId,
        _ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let torrent_data = state
            .current_torrent_data
            .clone()
            .take()
            .expect("current_torrent_data should be defined");

        let torrent_id = self
            .torrent_client
            .create("tmp/downloads", torrent_data)
            .await?;

        debug!(%torrent_id, "Started downloading of the torrent...");

        state.current_torrent_id.replace(torrent_id);

        Ok(())
    }

    async fn check_download_status(
        &self,
        _user_id: &UserId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let torrent_id = state
            .current_torrent_id
            .clone()
            .take()
            .expect("current_torrent_id should be defined");

        let torrent = self.torrent_client.get(&torrent_id).await?;

        if !matches!(torrent.status, TorrentStatus::Complete) {
            // Still downloading
            // TODO: Wait some time and try again
            return Ok(());
        }

        debug!(%torrent_id, "Download complete");

        for file in torrent.files {
            let metadata = match self.metadata_service.get_audio_metadata(&file).await? {
                Some(metadata) => metadata,
                None => continue,
            };

            if metadata.artist == ctx.metadata.artist && metadata.title == ctx.metadata.title {
                state.path_to_downloaded_file.replace(file);
                return Ok(());
            }
        }

        info!("Downloaded audio album does not contain requested audio track");
        state.current_download_id.take();
        state.current_torrent_id.take();
        state.current_torrent_data.take();

        Ok(())
    }

    async fn upload_to_radio_manager(
        &self,
        user_id: &UserId,
        _ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let path = state
            .path_to_downloaded_file
            .clone()
            .take()
            .expect("path_to_downloaded_file should be defined");

        info!("Uploading requested audio track to radio manager...");

        let track_id = self
            .radio_manager_client
            .upload_audio_track(user_id, &path)
            .await?;

        state.radio_manager_track_id.replace(track_id);

        Ok(())
    }

    async fn add_to_radio_manager_channel(
        &self,
        user_id: &UserId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let track_id = state
            .radio_manager_track_id
            .clone()
            .take()
            .expect("radio_manager_track_id should be defined");

        info!("Adding uploaded audio track to radio manager channel");

        let link_id = self
            .radio_manager_client
            .add_track_to_channel_playlist(user_id, &track_id, &ctx.target_channel_id)
            .await?;

        state.radio_manager_link_id.replace(link_id);

        Ok(())
    }
}
