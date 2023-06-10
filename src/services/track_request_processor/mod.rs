mod impls;
mod track_request_processor;
mod traits;
mod types;

pub(crate) use track_request_processor::*;

#[cfg(test)]
mod tests {
    use super::track_request_processor::TrackRequestProcessor;
    use super::traits::StateStorage;
    use crate::services::track_request_processor::traits::{
        Downloader, DownloaderError, DownloadingEntry, DownloadingStatus, MetadataService,
        MetadataServiceError, RadioManager, RadioManagerError, SearchProvider, SearchProviderError,
        SearchResult, StateStorageError,
    };
    use crate::services::track_request_processor::types::{
        RequestId, TrackFetcherContext, TrackFetcherState, TrackFetcherStep,
    };
    use crate::types::{
        AudioMetadata, DownloadId, RadioManagerChannelId, RadioManagerLinkId, RadioManagerTrackId,
        TopicId, UserId,
    };
    use async_trait::async_trait;
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    struct StateStorageMock {
        context_storage: Mutex<HashMap<UserId, HashMap<RequestId, TrackFetcherContext>>>,
        state_storage: Mutex<HashMap<UserId, HashMap<RequestId, TrackFetcherState>>>,
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
    impl StateStorage for StateStorageMock {
        async fn create_state(
            &self,
            user_id: &UserId,
            request_id: &RequestId,
            state: TrackFetcherState,
        ) -> Result<(), StateStorageError> {
            let mut lock = self.state_storage.lock().unwrap();

            let user_map = lock.entry(user_id.clone()).or_default();

            match user_map.entry(request_id.clone()) {
                Entry::Occupied(_) => Err(StateStorageError::ObjectExists),
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
            state: TrackFetcherContext,
        ) -> Result<(), StateStorageError> {
            let mut lock = self.context_storage.lock().unwrap();

            let user_map = lock.entry(user_id.clone()).or_default();

            match user_map.entry(request_id.clone()) {
                Entry::Occupied(_) => Err(StateStorageError::ObjectExists),
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
            state: &TrackFetcherState,
        ) -> Result<(), StateStorageError> {
            let mut lock = self.state_storage.lock().unwrap();

            let user_map = match lock.get_mut(user_id) {
                Some(user_map) => user_map,
                None => {
                    return Err(StateStorageError::ObjectNotFound);
                }
            };

            let stored_state = match user_map.get_mut(request_id) {
                Some(state) => state,
                None => {
                    return Err(StateStorageError::ObjectNotFound);
                }
            };

            *stored_state = state.clone();

            Ok(())
        }

        async fn load_state(
            &self,
            user_id: &UserId,
            request_id: &RequestId,
        ) -> Result<TrackFetcherState, StateStorageError> {
            let mut lock = self.state_storage.lock().unwrap();

            let state = lock
                .get(user_id)
                .ok_or_else(|| StateStorageError::ObjectNotFound)?
                .get(request_id)
                .ok_or_else(|| StateStorageError::ObjectNotFound)
                .map(Clone::clone)?;

            Ok(state)
        }

        async fn load_context(
            &self,
            user_id: &UserId,
            request_id: &RequestId,
        ) -> Result<TrackFetcherContext, StateStorageError> {
            let mut lock = self.context_storage.lock().unwrap();

            let ctx = lock
                .get(user_id)
                .ok_or_else(|| StateStorageError::ObjectNotFound)?
                .get(request_id)
                .ok_or_else(|| StateStorageError::ObjectNotFound)
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
    impl SearchProvider for SearchProviderMock {
        async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchProviderError> {
            match query {
                "Robert Miles - Children" => Ok(vec![
                    SearchResult {
                        title: "Robert Miles - Children [MP3]".into(),
                        topic_id: "t1".into(),
                    },
                    SearchResult {
                        title: "Robert Miles - Children [FLAC]".into(),
                        topic_id: "t2".into(),
                    },
                ]),
                _ => Ok(vec![]),
            }
        }

        async fn get_url(
            &self,
            topic_id: &TopicId,
        ) -> Result<Option<Vec<u8>>, SearchProviderError> {
            match (*topic_id).as_str() {
                "t1" => Ok(Some(vec![1])),
                _ => Ok(None),
            }
        }
    }

    struct DownloaderMock;

    #[async_trait]
    impl Downloader for DownloaderMock {
        async fn create(
            &self,
            _path_to_download: &str,
            url: Vec<u8>,
        ) -> Result<DownloadId, DownloaderError> {
            match url[..] {
                [1] => Ok("download1".into()),
                _ => Err(DownloaderError::Unexpected),
            }
        }

        async fn get(
            &self,
            download_id: &DownloadId,
        ) -> Result<Option<DownloadingEntry>, DownloaderError> {
            match (*download_id).as_str() {
                "download1" => Ok(Some(DownloadingEntry {
                    status: DownloadingStatus::Complete,
                    files: vec!["path/to/track01.mp3".into(), "path/to/track02.mp3".into()],
                })),
                _ => Ok(None),
            }
        }

        async fn delete(&self, download_id: &DownloadId) -> Result<(), DownloaderError> {
            todo!()
        }
    }

    struct MetadataServiceMock;

    #[async_trait]
    impl MetadataService for MetadataServiceMock {
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
    impl RadioManager for RadioManagerMock {
        async fn upload_audio_track(
            &self,
            user_id: &UserId,
            path_to_audio_file: &str,
        ) -> Result<RadioManagerTrackId, RadioManagerError> {
            match path_to_audio_file {
                "path/to/track02.mp3" => Ok(1.into()),
                _ => Err(RadioManagerError::Unexpected),
            }
        }

        async fn add_track_to_channel_playlist(
            &self,
            user_id: &UserId,
            track_id: &RadioManagerTrackId,
            channel_id: &RadioManagerChannelId,
        ) -> Result<RadioManagerLinkId, RadioManagerError> {
            Ok("link".into())
        }
    }

    #[actix_rt::test]
    async fn test_create_track_request() {
        let state_storage = Arc::new(StateStorageMock::new());

        let processor = TrackRequestProcessor::new(
            Arc::clone(&state_storage) as Arc<dyn StateStorage>,
            Arc::new(SearchProviderMock),
            Arc::new(DownloaderMock),
            Arc::new(MetadataServiceMock),
            Arc::new(RadioManagerMock),
        );
        let user_id = 1.into();
        let metadata = AudioMetadata {
            title: "Children".into(),
            artist: "Robert Miles".into(),
            album: "Children".into(),
        };
        let channel_id = 1.into();
        let request_id = processor
            .create_track_request(&user_id, &metadata, &channel_id)
            .await
            .unwrap();

        let stored_context = state_storage
            .load_context(&user_id, &request_id)
            .await
            .unwrap();
        assert_eq!(stored_context.track_title, "Children");
        assert_eq!(stored_context.track_artist, "Robert Miles");
        assert_eq!(stored_context.track_album, "Children");
        assert_eq!(stored_context.target_channel_id, channel_id);

        let stored_state = state_storage
            .load_state(&user_id, &request_id)
            .await
            .unwrap();
        assert_eq!(stored_state.get_step(), TrackFetcherStep::SearchAudioAlbum);
    }

    #[actix_rt::test]
    async fn test_processing_track_request() {
        let state_storage = Arc::new(StateStorageMock::new());

        let processor = TrackRequestProcessor::new(
            Arc::clone(&state_storage) as Arc<dyn StateStorage>,
            Arc::new(SearchProviderMock),
            Arc::new(DownloaderMock),
            Arc::new(MetadataServiceMock),
            Arc::new(RadioManagerMock),
        );
        let user_id = 1.into();
        let metadata = AudioMetadata {
            title: "Children".into(),
            artist: "Robert Miles".into(),
            album: "Children".into(),
        };
        let channel_id = 1.into();
        let request_id = processor
            .create_track_request(&user_id, &metadata, &channel_id)
            .await
            .unwrap();

        processor
            .process_track_request(&user_id, &request_id)
            .await
            .unwrap();
    }
}
