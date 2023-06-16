use crate::types::UserId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) struct RequestId(pub(crate) Uuid);

impl Deref for RequestId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub(crate) struct AudioMetadata {
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) album: String,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RadioManagerChannelId(pub(crate) u64);

impl std::fmt::Display for RadioManagerChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RadioManagerTrackId(pub(crate) u64);

impl std::fmt::Display for RadioManagerTrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RadioManagerLinkId(pub(crate) String);

impl std::fmt::Display for RadioManagerLinkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct TopicId(pub(crate) u64);

impl Deref for TopicId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TopicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct DownloadId(pub(crate) u64);

impl Deref for DownloadId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for DownloadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct TorrentId(pub(crate) i64);

impl Deref for TorrentId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for TorrentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct TopicData {
    pub(crate) topic_id: TopicId,
    pub(crate) download_id: DownloadId,
    pub(crate) title: String,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum TorrentStatus {
    Downloading,
    Complete,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Torrent {
    pub(crate) status: TorrentStatus,
    pub(crate) files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TrackRequestProcessingContext {
    pub(crate) metadata: AudioMetadata,
    pub(crate) options: CreateRequestOptions,
    pub(crate) target_channel_id: RadioManagerChannelId,
}

impl TrackRequestProcessingContext {
    pub(crate) fn new(
        metadata: AudioMetadata,
        options: CreateRequestOptions,
        target_channel_id: RadioManagerChannelId,
    ) -> Self {
        Self {
            metadata,
            options,
            target_channel_id,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub(crate) struct TrackRequestProcessingState {
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
            TrackRequestProcessingStep::DownloadTorrentFile
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
pub(crate) enum TrackRequestProcessingStep {
    SearchAudioAlbum,
    DownloadTorrentFile,
    DownloadAlbum,
    CheckDownloadStatus,
    UploadToRadioManager,
    AddToRadioManagerChannel,
    Finish,
}

#[async_trait]
pub(crate) trait StateStorageTrait {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingState,
    ) -> Result<(), StateStorageError>;
    async fn create_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        ctx: TrackRequestProcessingContext,
    ) -> Result<(), StateStorageError>;
    async fn update_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackRequestProcessingState,
    ) -> Result<(), StateStorageError>;
    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingState, StateStorageError>;
    async fn load_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingContext, StateStorageError>;
    async fn delete_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError>;
    async fn delete_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) struct StateStorageError(Box<dyn std::error::Error>);

impl StateStorageError {
    pub(crate) fn not_found() -> Self {
        StateStorageError(Box::new(std::io::Error::from(ErrorKind::NotFound)))
    }
}

impl std::fmt::Display for StateStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub(crate) trait SearchProviderTrait {
    async fn search_music(&self, query: &str) -> Result<Vec<TopicData>, SearchProviderError>;
    async fn download_torrent(
        &self,
        download_id: &DownloadId,
    ) -> Result<Vec<u8>, SearchProviderError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) struct SearchProviderError(pub(crate) Box<dyn std::error::Error>);

impl std::fmt::Display for SearchProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub(crate) trait TorrentClientTrait {
    async fn add_torrent(
        &self,
        torrent_file_data: Vec<u8>,
    ) -> Result<TorrentId, TorrentClientError>;
    async fn get_torrent(&self, torrent_id: &TorrentId) -> Result<Torrent, TorrentClientError>;
    async fn delete_torrent(&self, torrent_id: &TorrentId) -> Result<(), TorrentClientError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) struct TorrentClientError(pub(crate) Box<dyn std::error::Error>);

impl std::fmt::Display for TorrentClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub(crate) trait MetadataServiceTrait {
    async fn get_audio_metadata(
        &self,
        file_path: &str,
    ) -> Result<Option<AudioMetadata>, MetadataServiceError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) struct MetadataServiceError(pub(crate) Box<dyn std::error::Error>);

impl std::fmt::Display for MetadataServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub(crate) trait RadioManagerClientTrait {
    async fn upload_audio_track(
        &self,
        user_id: &UserId,
        path_to_audio_file: &str,
    ) -> Result<RadioManagerTrackId, RadioManagerClientError>;
    async fn add_track_to_channel_playlist(
        &self,
        user_id: &UserId,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) struct RadioManagerClientError(pub(crate) Box<dyn std::error::Error>);

impl std::fmt::Display for RadioManagerClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) struct TrackRequestProcessor {
    state_storage: Arc<dyn StateStorageTrait + Send + Sync + 'static>,
    search_provider: Arc<dyn SearchProviderTrait + Send + Sync + 'static>,
    torrent_client: Arc<dyn TorrentClientTrait + Send + Sync + 'static>,
    metadata_service: Arc<dyn MetadataServiceTrait + Send + Sync + 'static>,
    radio_manager_client: Arc<dyn RadioManagerClientTrait + Send + Sync + 'static>,
    download_directory: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CreateRequestError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProcessRequestError {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CreateRequestOptions {
    pub(crate) validate_metadata: bool,
}

impl TrackRequestProcessor {
    pub(crate) fn new(
        state_storage: Arc<dyn StateStorageTrait + Send + Sync + 'static>,
        search_provider: Arc<dyn SearchProviderTrait + Send + Sync + 'static>,
        torrent_client: Arc<dyn TorrentClientTrait + Send + Sync + 'static>,
        metadata_service: Arc<dyn MetadataServiceTrait + Send + Sync + 'static>,
        radio_manager_client: Arc<dyn RadioManagerClientTrait + Send + Sync + 'static>,
        download_directory: String,
    ) -> Self {
        Self {
            state_storage,
            search_provider,
            torrent_client,
            metadata_service,
            radio_manager_client,
            download_directory,
        }
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn create_request(
        &self,
        user_id: &UserId,
        track_metadata: &AudioMetadata,
        options: &CreateRequestOptions,
        target_channel_id: &RadioManagerChannelId,
    ) -> Result<RequestId, CreateRequestError> {
        let request_id = RequestId(Uuid::new_v4());
        let ctx = TrackRequestProcessingContext::new(
            track_metadata.clone(),
            options.clone(),
            target_channel_id.clone(),
        );
        let state = TrackRequestProcessingState::default();

        self.state_storage
            .create_context(user_id, &request_id, ctx)
            .await?;
        self.state_storage
            .create_state(user_id, &request_id, state)
            .await?;

        info!(%request_id, "Created new track request");

        Ok(request_id)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn process_request(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), ProcessRequestError> {
        info!("Start processing track request");

        let ctx = self.state_storage.load_context(user_id, request_id).await?;
        let mut state = self.state_storage.load_state(user_id, request_id).await?;

        while !matches!(state.get_step(), TrackRequestProcessingStep::Finish) {
            self.handle_next_step(user_id, &ctx, &mut state).await?;
            self.state_storage
                .update_state(user_id, request_id, &state)
                .await?;
            actix_rt::time::sleep(Duration::from_secs(1)).await;
        }

        info!("Track processing finished");

        self.state_storage.delete_state(user_id, request_id).await?;
        self.state_storage
            .delete_context(user_id, request_id)
            .await?;

        Ok(())
    }

    async fn handle_next_step(
        &self,
        user_id: &UserId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let step = state.get_step();

        debug!("Running processing step: {:?}", step);

        match step {
            TrackRequestProcessingStep::SearchAudioAlbum => {
                self.search_audio_album(user_id, ctx, state).await?;
            }
            TrackRequestProcessingStep::DownloadTorrentFile => {
                self.download_torrent_file(user_id, ctx, state).await?;
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
        debug!("Querying search engine: {}", query);

        let results = self.search_provider.search_music(&query).await?;
        debug!("Found {} results", results.len());

        let tried_topics_set = state.tried_topics.iter().collect::<HashSet<_>>();
        let topic = match results
            .into_iter()
            .filter(|r| !tried_topics_set.contains(&r.topic_id))
            .next()
        {
            Some(topic) => topic,
            None => {
                error!("No more search results containing requested track... Mission impossible.");

                todo!();
            }
        };

        debug!(?topic, "Found topic that may contain the requested track");

        state.current_download_id.replace(topic.download_id);
        state.tried_topics.push(topic.topic_id);

        Ok(())
    }

    async fn download_torrent_file(
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

        debug!("Downloading torrent data...");

        let torrent_data = self.search_provider.download_torrent(&download_id).await?;

        debug!("Torrent data size: {} bytes", torrent_data.len());

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

        debug!("Adding torrent to the torrent client...");
        let torrent_id = self.torrent_client.add_torrent(torrent_data).await?;

        debug!(%torrent_id, "Started downloading the torrent");

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

        debug!("Checking downloading status of the torrent file...");

        let torrent = self.torrent_client.get_torrent(&torrent_id).await?;

        if !matches!(torrent.status, TorrentStatus::Complete) {
            // Still downloading
            actix_rt::time::sleep(Duration::from_secs(5)).await;

            return Ok(());
        }

        debug!(%torrent_id, "Download complete. Checking files metadata...");

        for file in torrent.files {
            if ctx.options.validate_metadata {
                debug!("Checking metadata of {} file...", file);

                let metadata = match self.metadata_service.get_audio_metadata(&file).await {
                    Ok(Some(metadata)) => metadata,
                    _ => continue,
                };

                if metadata.artist.starts_with(&ctx.metadata.artist)
                    && metadata.title.starts_with(&ctx.metadata.title)
                {
                    info!("Found audio file that matches the requested audio track!");
                    state.path_to_downloaded_file.replace(file);
                    return Ok(());
                }
            } else {
                debug!(file, "Checking file...");

                if file.contains(&ctx.metadata.title) {
                    info!("Found audio file that matches the requested audio track!");
                    state.path_to_downloaded_file.replace(file);
                    return Ok(());
                }
            }
        }

        info!("Downloaded torrent does not have the requested audio track");
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

        let full_path_to_file = format!("{}/{}", self.download_directory, path);

        info!(
            full_path_to_file,
            "Uploading audio track to radio manager..."
        );

        let track_id = self
            .radio_manager_client
            .upload_audio_track(user_id, &full_path_to_file)
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
