use crate::services::track_request_processor::traits::{
    Downloader, DownloaderError, DownloadingStatus, MetadataService, MetadataServiceError,
    RadioManager, RadioManagerError, SearchProvider, SearchProviderError, SearchResult,
    StateStorage, StateStorageError,
};
use crate::services::track_request_processor::types::{
    RequestId, TrackFetcherContext, TrackFetcherState, TrackFetcherStep,
};
use crate::types::{AudioMetadata, RadioManagerChannelId, UserId};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub(crate) enum CreateJobError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ProceedNextStepError {
    #[error(transparent)]
    StateStorageError(#[from] StateStorageError),
    #[error(transparent)]
    SearchProviderError(#[from] SearchProviderError),
    #[error(transparent)]
    DownloaderError(#[from] DownloaderError),
    #[error(transparent)]
    MetadataServiceError(#[from] MetadataServiceError),
    #[error(transparent)]
    RadioManagerError(#[from] RadioManagerError),
    #[error("Job has not been found in the storage")]
    JobNotFound,
}

pub(crate) struct TrackRequestProcessor {
    state_storage: Arc<dyn StateStorage>,
    search_provider: Arc<dyn SearchProvider>,
    downloader: Arc<dyn Downloader>,
    metadata_service: Arc<dyn MetadataService>,
    radio_manager: Arc<dyn RadioManager>,
}

impl TrackRequestProcessor {
    pub(crate) fn new(
        state_storage: Arc<dyn StateStorage>,
        search_provider: Arc<dyn SearchProvider>,
        downloader: Arc<dyn Downloader>,
        metadata_service: Arc<dyn MetadataService>,
        radio_manager: Arc<dyn RadioManager>,
    ) -> Self {
        Self {
            state_storage,
            search_provider,
            downloader,
            metadata_service,
            radio_manager,
        }
    }

    #[instrument(skip(self))]
    pub(crate) async fn create_track_request(
        &self,
        user_id: &UserId,
        track_metadata: &AudioMetadata,
        target_channel_id: &RadioManagerChannelId,
    ) -> Result<RequestId, CreateJobError> {
        let request_id = Uuid::new_v4().into();
        let ctx = TrackFetcherContext::new(
            track_metadata.title.clone(),
            track_metadata.artist.clone(),
            track_metadata.album.clone(),
            target_channel_id.clone(),
        );
        let state = TrackFetcherState::default();

        self.state_storage
            .create_context(user_id, &request_id, ctx)
            .await?;
        self.state_storage
            .create_state(user_id, &request_id, state)
            .await?;

        info!(%user_id, %request_id, "New track request successfully created");

        Ok(request_id)
    }

    #[instrument(skip(self))]
    pub(crate) async fn process_track_request(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), ProceedNextStepError> {
        info!(%user_id, %request_id, "Processing track request");

        let ctx = self
            .state_storage
            .load_context(user_id, request_id)
            .await?
            .ok_or_else(|| ProceedNextStepError::JobNotFound)?;
        let mut state = self
            .state_storage
            .load_state(user_id, request_id)
            .await?
            .ok_or_else(|| ProceedNextStepError::JobNotFound)?;

        while !matches!(state.get_step(), TrackFetcherStep::Finish) {
            self.run_next_step(user_id, &ctx, &mut state).await?;
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

    #[instrument(skip(self))]
    async fn run_next_step(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let step = state.get_step();

        debug!(%user_id, ?step, "Running next processing step");

        match step {
            TrackFetcherStep::SearchAudioAlbum => {
                self.search_audio_album(user_id, ctx, state).await?;
            }
            TrackFetcherStep::GetAlbumURL => {
                self.get_album_url(user_id, ctx, state).await?;
            }
            TrackFetcherStep::DownloadAlbum => {
                self.download_album(user_id, ctx, state).await?;
            }
            TrackFetcherStep::CheckDownloadStatus => {
                self.check_download_status(user_id, ctx, state).await?;
            }
            TrackFetcherStep::UploadToRadioManager => {
                self.upload_to_radio_manager(user_id, ctx, state).await?;
            }
            TrackFetcherStep::AddToRadioManagerChannel => {
                self.add_to_radio_manager_channel(user_id, ctx, state)
                    .await?;
            }
            TrackFetcherStep::Finish => (),
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn search_audio_album(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let query = format!("{} - {}", ctx.track_artist, ctx.track_album);
        let results = self.search_provider.search(&query).await?;

        let tried_topics_set = state.tried_topics.iter().collect::<HashSet<_>>();

        let next_result = results
            .into_iter()
            .filter(|r| !tried_topics_set.contains(&r.topic_id))
            .next();

        match next_result {
            Some(result) => {
                let SearchResult { topic_id, title } = result;
                debug!(%topic_id, title, "Found result possibly containing the requested track");
                state.current_topic_id.replace(topic_id);
                Ok(())
            }
            None => {
                debug!("Requested track has not get been found...");
                todo!();
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_album_url(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let topic_id = state
            .current_topic_id
            .clone()
            .take()
            .expect("Current topic_id should be defined");

        debug!("Getting URL to download the audio album...");

        match self.search_provider.get_url(&topic_id).await? {
            Some(url) => {
                state.current_url.replace(url);
                Ok(())
            }
            None => {
                warn!("The current topic is gone. Going to look for another one.");
                state.tried_topics.push(topic_id);
                Ok(())
            }
        }
    }

    #[instrument(skip(self))]
    async fn download_album(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let url = state
            .current_url
            .clone()
            .take()
            .expect("Current current_url should be defined");

        let download_id = self.downloader.create("tmp/downloads", url).await?;

        debug!(%download_id, "Started download of the audio album...");

        state.current_download_id.replace(download_id);

        Ok(())
    }

    #[instrument(skip(self))]
    async fn check_download_status(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let download_id = state
            .current_download_id
            .clone()
            .take()
            .expect("Current current_download_id should be defined");

        let downloading_entry = self.downloader.get(&download_id).await?;

        let entry = match downloading_entry {
            Some(entry) => entry,
            None => {
                warn!(%download_id, "Download has not been found");
                state.current_download_id.take();
                return Ok(());
            }
        };

        if !matches!(entry.status, DownloadingStatus::Complete) {
            // Still downloading
            return Ok(());
        }

        debug!(%download_id, "Download complete");

        for file in entry.files {
            let metadata = match self.metadata_service.get_audio_metadata(&file).await? {
                Some(metadata) => metadata,
                None => {
                    continue;
                }
            };

            if metadata.artist == ctx.track_artist && metadata.title == ctx.track_title {
                state.path_to_downloaded_file.replace(file);
                return Ok(());
            }
        }

        info!("Downloaded audio album does not contain the requested audio track");

        if let Some(topic_id) = state.current_topic_id.take() {
            state.tried_topics.push(topic_id);
        }
        state.current_download_id.take();
        state.current_url.take();

        Ok(())
    }

    #[instrument(skip(self))]
    async fn upload_to_radio_manager(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let path = state
            .path_to_downloaded_file
            .clone()
            .take()
            .expect("Current path_to_downloaded_file should be defined");

        info!("Uploading requested audio track to radio manager");

        let track_id = self
            .radio_manager
            .upload_audio_track(user_id, &path)
            .await?;

        state.radio_manager_track_id.replace(track_id);

        Ok(())
    }

    #[instrument(skip(self))]
    async fn add_to_radio_manager_channel(
        &self,
        user_id: &UserId,
        ctx: &TrackFetcherContext,
        state: &mut TrackFetcherState,
    ) -> Result<(), ProceedNextStepError> {
        let track_id = state
            .radio_manager_track_id
            .clone()
            .take()
            .expect("Current path_to_downloaded_file should be defined");

        info!("Adding uploaded audio track to radio manager channel");

        let link_id = self
            .radio_manager
            .add_track_to_channel_playlist(user_id, &track_id, &ctx.target_channel_id)
            .await?;

        state.radio_manager_link_id.replace(link_id);

        Ok(())
    }
}
