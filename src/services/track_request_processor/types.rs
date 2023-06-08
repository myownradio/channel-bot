use crate::types::{
    DownloadId, RadioManagerChannelId, RadioManagerLinkId, RadioManagerTrackId, TopicId, UserId,
};
use serde::Serialize;
use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug)]
pub(crate) struct RequestId(Uuid);

impl Deref for RequestId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<RequestId> for Uuid {
    fn into(self) -> RequestId {
        RequestId(self)
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct TrackFetcherContext {
    pub(crate) track_title: String,
    pub(crate) track_artist: String,
    pub(crate) track_album: String,
    pub(crate) target_channel_id: RadioManagerChannelId,
}

impl TrackFetcherContext {
    pub(crate) fn new(
        track_title: String,
        track_artist: String,
        track_album: String,
        target_channel_id: RadioManagerChannelId,
    ) -> Self {
        Self {
            track_title,
            track_artist,
            track_album,
            target_channel_id,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct TrackFetcherState {
    pub(crate) tried_topics: Vec<TopicId>,
    pub(crate) current_topic_id: Option<TopicId>,
    pub(crate) current_url: Option<Vec<u8>>,
    pub(crate) current_download_id: Option<DownloadId>,
    pub(crate) path_to_downloaded_file: Option<String>,
    pub(crate) radio_manager_track_id: Option<RadioManagerTrackId>,
    pub(crate) radio_manager_link_id: Option<RadioManagerLinkId>,
}

impl TrackFetcherState {
    pub(crate) fn get_step(&self) -> TrackFetcherStep {
        if self.current_topic_id.is_none() {
            TrackFetcherStep::SearchAudioAlbum
        } else if self.current_url.is_none() {
            TrackFetcherStep::GetAlbumURL
        } else if self.current_download_id.is_none() {
            TrackFetcherStep::DownloadAlbum
        } else if self.path_to_downloaded_file.is_none() {
            TrackFetcherStep::CheckDownloadStatus
        } else if self.radio_manager_track_id.is_none() {
            TrackFetcherStep::UploadToRadioManager
        } else if self.radio_manager_link_id.is_none() {
            TrackFetcherStep::AddToRadioManagerChannel
        } else {
            TrackFetcherStep::Finish
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum TrackFetcherStep {
    SearchAudioAlbum,
    GetAlbumURL,
    DownloadAlbum,
    CheckDownloadStatus,
    UploadToRadioManager,
    AddToRadioManagerChannel,
    Finish,
}

#[cfg(test)]
mod track_fetcher_step_tests {
    use crate::services::track_request_processor::types::{TrackFetcherState, TrackFetcherStep};

    #[test]
    fn should_return_search_audio_album_by_default() {
        let state = TrackFetcherState::default();

        assert_eq!(state.get_step(), TrackFetcherStep::SearchAudioAlbum)
    }

    #[test]
    fn should_return_get_album_url_if_current_topic_id_is_set() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::GetAlbumURL)
    }

    #[test]
    fn should_return_download_album_if_current_url_is_set() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_url: Some(vec![]),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::DownloadAlbum)
    }

    #[test]
    fn should_return_check_download_status_if_current_download_id_is_set() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_url: Some(vec![]),
            current_download_id: Some("download".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::CheckDownloadStatus)
    }

    #[test]
    fn should_return_upload_to_radioterio_if_path_to_downloaded_file_is_set() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_url: Some(vec![]),
            current_download_id: Some("download".into()),
            path_to_downloaded_file: Some("path/to/file".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::UploadToRadioManager)
    }

    #[test]
    fn should_return_add_track_to_radioterio_channel_if_radioterio_track_id_is_set() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_url: Some(vec![]),
            current_download_id: Some("download".into()),
            path_to_downloaded_file: Some("path/to/file".into()),
            radio_manager_track_id: Some(1.into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::AddToRadioManagerChannel)
    }

    #[test]
    fn should_return_finish_if_radioterio_link_id_is_set() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_url: Some(vec![]),
            current_download_id: Some("download".into()),
            path_to_downloaded_file: Some("path/to/file".into()),
            radio_manager_track_id: Some(1.into()),
            radio_manager_link_id: Some("foo".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::Finish)
    }
}
