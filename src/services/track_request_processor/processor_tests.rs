use super::track_request_processor::{
    AudioMetadata, DownloadId, MetadataServiceError, MetadataServiceTrait, RadioManagerChannelId,
    RadioManagerClientError, RadioManagerClientTrait, RadioManagerLinkId, RadioManagerTrackId,
    RequestId, SearchProviderError, SearchProviderTrait, StateStorageError, StateStorageTrait,
    TopicData, TopicId, Torrent, TorrentClientError, TorrentClientTrait, TorrentId, TorrentStatus,
    TrackRequestProcessingContext, TrackRequestProcessingState, TrackRequestProcessingStep,
    TrackRequestProcessor,
};
use crate::services::track_request_processor::CreateRequestOptions;
use crate::types::UserId;
use async_trait::async_trait;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};

struct StateStorageMock {
    context_storage: Mutex<HashMap<UserId, HashMap<RequestId, TrackRequestProcessingContext>>>,
    state_storage: Mutex<HashMap<UserId, HashMap<RequestId, TrackRequestProcessingState>>>,
}

impl StateStorageMock {
    fn new() -> Self {
        Self {
            context_storage: Mutex::new(HashMap::new()),
            state_storage: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StateStorageTrait for StateStorageMock {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingState,
    ) -> Result<(), StateStorageError> {
        let mut lock = self.state_storage.lock().unwrap();

        let user_map = lock.entry(user_id.clone()).or_default();

        match user_map.entry(request_id.clone()) {
            Entry::Occupied(_) => todo!(),
            Entry::Vacant(entry) => {
                entry.insert(state);
                Ok(())
            }
        }
    }

    async fn create_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingContext,
    ) -> Result<(), StateStorageError> {
        let mut lock = self.context_storage.lock().unwrap();

        let user_map = lock.entry(user_id.clone()).or_default();

        match user_map.entry(request_id.clone()) {
            Entry::Occupied(_) => todo!(),
            Entry::Vacant(entry) => {
                entry.insert(state);
                Ok(())
            }
        }
    }

    async fn update_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackRequestProcessingState,
    ) -> Result<(), StateStorageError> {
        let mut lock = self.state_storage.lock().unwrap();

        let user_map = match lock.get_mut(user_id) {
            Some(user_map) => user_map,
            None => todo!(),
        };

        let stored_state = match user_map.get_mut(request_id) {
            Some(state) => state,
            None => todo!(),
        };

        *stored_state = state.clone();

        Ok(())
    }

    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingState, StateStorageError> {
        let mut lock = self.state_storage.lock().unwrap();

        let state = lock
            .get(user_id)
            .ok_or_else(|| todo!())?
            .get(request_id)
            .ok_or_else(|| todo!())
            .map(Clone::clone)?;

        Ok(state)
    }

    async fn load_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingContext, StateStorageError> {
        let mut lock = self.context_storage.lock().unwrap();

        let ctx = lock
            .get(user_id)
            .ok_or_else(|| todo!())?
            .get(request_id)
            .ok_or_else(|| todo!())
            .map(Clone::clone)?;

        Ok(ctx)
    }

    async fn delete_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let mut lock = self.state_storage.lock().unwrap();

        let _ = lock.get_mut(user_id).and_then(|map| map.remove(request_id));

        Ok(())
    }

    async fn delete_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let mut lock = self.context_storage.lock().unwrap();

        let _ = lock.get_mut(user_id).and_then(|map| map.remove(request_id));

        Ok(())
    }
}

struct SearchProviderMock;

#[async_trait]
impl SearchProviderTrait for SearchProviderMock {
    async fn search_music(&self, query: &str) -> Result<Vec<TopicData>, SearchProviderError> {
        match query {
            "Robert Miles - Children" => Ok(vec![
                TopicData {
                    title: "Robert Miles - Children [MP3]".into(),
                    topic_id: TopicId(1),
                    download_id: DownloadId(1),
                },
                TopicData {
                    title: "Robert Miles - Children [FLAC]".into(),
                    topic_id: TopicId(2),
                    download_id: DownloadId(2),
                },
            ]),
            _ => Ok(vec![]),
        }
    }

    async fn download_torrent(
        &self,
        download_id: &DownloadId,
    ) -> Result<Vec<u8>, SearchProviderError> {
        match **download_id {
            1 => Ok(vec![1]),
            _ => Err(SearchProviderError(Box::new(Error::from(
                ErrorKind::NotFound,
            )))),
        }
    }
}

struct TorrentClientMock;

#[async_trait]
impl TorrentClientTrait for TorrentClientMock {
    async fn add_torrent(&self, url: Vec<u8>) -> Result<TorrentId, TorrentClientError> {
        match url[..] {
            [1] => Ok(TorrentId(1)),
            _ => todo!(),
        }
    }

    async fn get_torrent(&self, torrent_id: &TorrentId) -> Result<Torrent, TorrentClientError> {
        match **torrent_id {
            1 => Ok(Torrent {
                status: TorrentStatus::Complete,
                files: vec!["path/to/track01.mp3".into(), "path/to/track02.mp3".into()],
            }),
            _ => todo!(),
        }
    }

    async fn delete_torrent(&self, torrent_id: &TorrentId) -> Result<(), TorrentClientError> {
        todo!()
    }
}

struct MetadataServiceMock;

#[async_trait]
impl MetadataServiceTrait for MetadataServiceMock {
    async fn get_audio_metadata(
        &self,
        file_path: &str,
    ) -> Result<Option<AudioMetadata>, MetadataServiceError> {
        match file_path {
            "path/to/track01.mp3" => Ok(Some(AudioMetadata {
                title: "Fable".into(),
                artist: "Robert Miles".into(),
                album: "Dreamland".into(),
            })),
            "path/to/track02.mp3" => Ok(Some(AudioMetadata {
                title: "Children".into(),
                artist: "Robert Miles".into(),
                album: "Children".into(),
            })),
            _ => Ok(None),
        }
    }
}

struct RadioManagerMock;

#[async_trait]
impl RadioManagerClientTrait for RadioManagerMock {
    async fn upload_audio_track(
        &self,
        _user_id: &UserId,
        path_to_audio_file: &str,
    ) -> Result<RadioManagerTrackId, RadioManagerClientError> {
        match path_to_audio_file {
            "downloads/path/to/track02.mp3" => Ok(RadioManagerTrackId(1)),
            _ => Err(RadioManagerClientError(Box::new(Error::from(
                ErrorKind::NotFound,
            )))),
        }
    }

    async fn add_track_to_channel_playlist(
        &self,
        user_id: &UserId,
        track_id: &RadioManagerTrackId,
        channel_id: &RadioManagerChannelId,
    ) -> Result<RadioManagerLinkId, RadioManagerClientError> {
        Ok(RadioManagerLinkId("link".into()))
    }
}

#[actix_rt::test]
async fn test_create_track_request() {
    let state_storage = Arc::new(StateStorageMock::new());

    let processor = TrackRequestProcessor::new(
        state_storage.clone(),
        Arc::new(SearchProviderMock),
        Arc::new(TorrentClientMock),
        Arc::new(MetadataServiceMock),
        Arc::new(RadioManagerMock),
        "downloads".to_string(),
    );
    let user_id = 1.into();
    let metadata = AudioMetadata {
        title: "Children".into(),
        artist: "Robert Miles".into(),
        album: "Children".into(),
    };
    let channel_id = RadioManagerChannelId(1);
    let request_id = processor
        .create_request(
            &user_id,
            &metadata,
            &CreateRequestOptions {
                validate_metadata: true,
            },
            &channel_id,
        )
        .await
        .unwrap();

    let stored_context = state_storage
        .load_context(&user_id, &request_id)
        .await
        .unwrap();
    assert_eq!(stored_context.metadata.title, "Children");
    assert_eq!(stored_context.metadata.artist, "Robert Miles");
    assert_eq!(stored_context.metadata.album, "Children");
    assert_eq!(stored_context.target_channel_id, channel_id);

    let stored_state = state_storage
        .load_state(&user_id, &request_id)
        .await
        .unwrap();
    assert_eq!(
        stored_state.get_step(),
        TrackRequestProcessingStep::SearchAudioAlbum
    );
}

#[actix_rt::test]
async fn test_processing_track_request() {
    let processor = TrackRequestProcessor::new(
        Arc::from(StateStorageMock::new()),
        Arc::from(SearchProviderMock),
        Arc::from(TorrentClientMock),
        Arc::from(MetadataServiceMock),
        Arc::from(RadioManagerMock),
        "downloads".into(),
    );
    let user_id = UserId(1);
    let metadata = AudioMetadata {
        title: "Children".into(),
        artist: "Robert Miles".into(),
        album: "Children".into(),
    };
    let channel_id = RadioManagerChannelId(1);
    let request_id = processor
        .create_request(
            &user_id,
            &metadata,
            &CreateRequestOptions {
                validate_metadata: true,
            },
            &channel_id,
        )
        .await
        .unwrap();

    processor
        .process_request(&user_id, &request_id)
        .await
        .unwrap();
}
