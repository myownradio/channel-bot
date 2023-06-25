use crate::services::torrent_parser::{get_files, TorrentParserError};
use crate::types::UserId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
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

impl std::fmt::Display for AudioMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} ({})", self.artist, self.title, self.album)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
pub(crate) struct RadioManagerChannelId(pub(crate) u64);

impl Deref for RadioManagerChannelId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum TrackRequestProcessingStatus {
    Processing,
    NotFound,
    Failed,
    Finished,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RadioManagerChannelTrack {
    pub(crate) album: String,
    pub(crate) artist: String,
    pub(crate) title: String,
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
    async fn update_status(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        status: &TrackRequestProcessingStatus,
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
    async fn delete_status(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError>;
    async fn get_all_statuses(
        &self,
        user_id: &UserId,
    ) -> Result<HashMap<RequestId, TrackRequestProcessingStatus>, StateStorageError>;
    async fn get_all_tasks(&self) -> Result<Vec<(UserId, RequestId)>, StateStorageError>;
}

#[derive(Debug, thiserror::Error)]
pub(crate) struct StateStorageError(pub(crate) Box<dyn std::error::Error>);

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
        selected_files_indexes: Vec<i32>,
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
    async fn get_channel_tracks(
        &self,
        channel_id: &RadioManagerChannelId,
    ) -> Result<Vec<RadioManagerChannelTrack>, RadioManagerClientError>;
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
    RadioManagerError(#[from] RadioManagerClientError),
    #[error(transparent)]
    TorrentParserError(#[from] TorrentParserError),
    #[error("Request track has not been found")]
    TrackNotFound,
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
        radio_manager_client: Arc<dyn RadioManagerClientTrait + Send + Sync + 'static>,
        download_directory: String,
    ) -> Self {
        Self {
            state_storage,
            search_provider,
            torrent_client,
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
        debug!(
            ?target_channel_id,
            "Creating the new track request - {}", track_metadata
        );

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

        info!(
            ?target_channel_id,
            "Created new track request {} for {}", request_id, track_metadata
        );

        Ok(request_id)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn process_request(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), ProcessRequestError> {
        debug!("Starting processing the track request {}", request_id);

        let ctx = self.state_storage.load_context(user_id, request_id).await?;
        let mut state = self.state_storage.load_state(user_id, request_id).await?;

        self.state_storage
            .update_status(
                user_id,
                request_id,
                &TrackRequestProcessingStatus::Processing,
            )
            .await?;

        // TODO Check if the file already exists in library.

        while !matches!(state.get_step(), TrackRequestProcessingStep::Finish) {
            if let Err(error) = self
                .handle_next_step(user_id, request_id, &ctx, &mut state)
                .await
            {
                match error {
                    ProcessRequestError::TrackNotFound => {
                        self.state_storage
                            .update_status(
                                user_id,
                                request_id,
                                &TrackRequestProcessingStatus::NotFound,
                            )
                            .await?;
                    }
                    _ => {
                        self.state_storage
                            .update_status(
                                user_id,
                                request_id,
                                &TrackRequestProcessingStatus::Failed,
                            )
                            .await?;
                    }
                }

                return Err(error);
            };
            self.state_storage
                .update_state(user_id, request_id, &state)
                .await?;
            actix_rt::time::sleep(Duration::from_secs(1)).await;
        }

        info!("Track request {} processing finished", request_id);

        self.state_storage
            .update_status(user_id, request_id, &TrackRequestProcessingStatus::Finished)
            .await?;
        self.state_storage.delete_state(user_id, request_id).await?;
        self.state_storage
            .delete_context(user_id, request_id)
            .await?;

        Ok(())
    }

    pub(crate) async fn get_processing_requests(
        &self,
        user_id: &UserId,
    ) -> Result<HashMap<RequestId, TrackRequestProcessingStatus>, ProcessRequestError> {
        let statuses = self.state_storage.get_all_statuses(user_id).await?;

        Ok(statuses)
    }

    async fn handle_next_step(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let step = state.get_step();

        debug!("Running processing step: {:?}", step);

        match step {
            TrackRequestProcessingStep::SearchAudioAlbum => {
                self.search_audio_album(user_id, request_id, ctx, state)
                    .await?;
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
        _request_id: &RequestId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let queries_to_try = vec![
            format!("{} - {}", ctx.metadata.artist, ctx.metadata.album),
            format!("{} дискография", ctx.metadata.artist),
            format!("{} discography", ctx.metadata.artist),
            format!("{} дискографія", ctx.metadata.artist),
        ];

        let tried_topics_set = state.tried_topics.iter().collect::<HashSet<_>>();
        for query in queries_to_try {
            info!("Searching the Internet for \"{}\"...", query);

            let new_results: Vec<_> = self
                .search_provider
                .search_music(&query)
                .await?
                .into_iter()
                .filter(|r| !tried_topics_set.contains(&r.topic_id))
                .collect();
            info!("Found {} new search results...", new_results.len());

            let topic = match new_results.into_iter().next() {
                Some(topic) => topic,
                None => {
                    continue;
                }
            };

            info!("This time we'll try with {}...", topic.title);

            state.current_download_id.replace(topic.download_id);
            state.tried_topics.push(topic.topic_id);

            return Ok(());
        }

        error!(
            "No more search results... The requested track {} has not been found.",
            ctx.metadata
        );

        Err(ProcessRequestError::TrackNotFound)
    }

    async fn download_torrent_file(
        &self,
        _user_id: &UserId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let download_id = state
            .current_download_id
            .clone()
            .take()
            .expect("current_download_id should be defined");

        info!("Downloading torrent file...");

        let torrent_data = self.search_provider.download_torrent(&download_id).await?;
        let files_in_torrent = get_files(&torrent_data)?;

        if files_in_torrent.into_iter().any(|f| {
            f.to_lowercase()
                .contains(&ctx.metadata.title.to_lowercase())
        }) {
            info!("Downloaded torrent file seems to have the requested track...");
            state.current_torrent_data.replace(torrent_data);
        } else {
            state.current_download_id.take();
        }

        Ok(())
    }

    async fn download_album(
        &self,
        _user_id: &UserId,
        ctx: &TrackRequestProcessingContext,
        state: &mut TrackRequestProcessingState,
    ) -> Result<(), ProcessRequestError> {
        let torrent_data = state
            .current_torrent_data
            .clone()
            .take()
            .expect("current_torrent_data should be defined");

        let files_in_torrent = get_files(&torrent_data)?;
        let track_title_lc = ctx.metadata.title.to_lowercase();
        let selected_files: Vec<_> = files_in_torrent
            .into_iter()
            .enumerate()
            .filter(|(index, file_path)| file_path.to_lowercase().contains(&track_title_lc))
            .map(|(index, _)| index as i32)
            .collect();

        debug!("Adding torrent to the torrent client...");
        let torrent_id = self
            .torrent_client
            .add_torrent(torrent_data, selected_files)
            .await?;

        info!(%torrent_id, "Started downloading the torrent contents...");

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

        debug!("Checking the download status of the torrent file...");

        let torrent = self.torrent_client.get_torrent(&torrent_id).await?;

        if !matches!(torrent.status, TorrentStatus::Complete) {
            // Still downloading? Check again in 5 secs...
            actix_rt::time::sleep(Duration::from_secs(5)).await;

            return Ok(());
        }

        debug!(%torrent_id, "Download complete. Checking files metadata...");

        let title_lc = ctx.metadata.title.to_lowercase();
        let artist_lc = ctx.metadata.artist.to_lowercase();

        for file in torrent.files {
            if file.to_lowercase().contains(&title_lc) {
                info!("Found matching audio file: {}", file);
                state.path_to_downloaded_file.replace(file);
                return Ok(());
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

        info!(
            "Adding uploaded audio track to the radio manager channel {}...",
            ctx.target_channel_id
        );

        let link_id = self
            .radio_manager_client
            .add_track_to_channel_playlist(user_id, &track_id, &ctx.target_channel_id)
            .await?;

        state.radio_manager_link_id.replace(link_id);

        Ok(())
    }
}
