use crate::types::{DownloadId, RadioterioChannelId, RadioterioLinkId, RadioterioTrackId, TopicId};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct TrackFetcherContext {
    pub(crate) track_title: String,
    pub(crate) track_artist: String,
    pub(crate) track_album: String,
    pub(crate) target_channel_id: RadioterioChannelId,
}

impl TrackFetcherContext {
    pub(crate) fn new(
        track_title: String,
        track_artist: String,
        track_album: String,
        target_channel_id: RadioterioChannelId,
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
    pub(crate) current_download_id: Option<DownloadId>,
    pub(crate) path_to_downloaded_file: Option<String>,
    pub(crate) radioterio_track_id: Option<RadioterioTrackId>,
    pub(crate) radioterio_link_id: Option<RadioterioLinkId>,
}

impl TrackFetcherState {
    pub(crate) fn get_step(&self) -> TrackFetcherStep {
        if self.current_topic_id.is_none() {
            TrackFetcherStep::FindTrackAlbum
        } else if self.current_download_id.is_none() {
            TrackFetcherStep::DownloadTorrent
        } else if self.path_to_downloaded_file.is_none() {
            TrackFetcherStep::DownloadTrackAlbum
        } else if self.radioterio_track_id.is_none() {
            TrackFetcherStep::UploadTrackToRadioterio
        } else if self.radioterio_link_id.is_none() {
            TrackFetcherStep::AddTrackToRadioterioChannel
        } else {
            TrackFetcherStep::Finish
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum TrackFetcherStep {
    FindTrackAlbum,
    DownloadTorrent,
    DownloadTrackAlbum,
    UploadTrackToRadioterio,
    AddTrackToRadioterioChannel,
    Finish,
}

#[cfg(test)]
mod track_fetcher_step_tests {
    use crate::services::track_fetcher::types::{TrackFetcherState, TrackFetcherStep};

    #[test]
    fn should_return_find_track_album_by_default() {
        let state = TrackFetcherState::default();

        assert_eq!(state.get_step(), TrackFetcherStep::FindTrackAlbum)
    }

    #[test]
    fn should_return_download_torrent_if_current_topic() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::DownloadTorrent)
    }

    #[test]
    fn should_return_download_track_album_if_current_download() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_download_id: Some("download".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::DownloadTrackAlbum)
    }

    #[test]
    fn should_return_upload_to_radioterio_if_path_to_downloaded_file() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_download_id: Some("download".into()),
            path_to_downloaded_file: Some("path/to/file".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::UploadTrackToRadioterio)
    }

    #[test]
    fn should_return_add_track_to_radioterio_channel_if_track_id() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_download_id: Some("download".into()),
            path_to_downloaded_file: Some("path/to/file".into()),
            radioterio_track_id: Some(1.into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(
            state.get_step(),
            TrackFetcherStep::AddTrackToRadioterioChannel
        )
    }

    #[test]
    fn should_return_finish_if_radioterio_link_id() {
        let state = TrackFetcherState {
            current_topic_id: Some("topic".into()),
            current_download_id: Some("download".into()),
            path_to_downloaded_file: Some("path/to/file".into()),
            radioterio_track_id: Some(1.into()),
            radioterio_link_id: Some("foo".into()),
            ..TrackFetcherState::default()
        };

        assert_eq!(state.get_step(), TrackFetcherStep::Finish)
    }
}
