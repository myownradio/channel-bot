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
        if self.radioterio_link_id.is_some() {
            TrackFetcherStep::Finish
        } else if self.radioterio_track_id.is_some() {
            TrackFetcherStep::AddTrackToRadioterioChannel
        } else if self.path_to_downloaded_file.is_some() {
            TrackFetcherStep::UploadTrackToRadioterio
        } else if self.current_download_id.is_some() {
            TrackFetcherStep::DownloadTrackAlbum
        } else if self.current_topic_id.is_some() {
            TrackFetcherStep::DownloadTorrent
        } else {
            TrackFetcherStep::FindTrackAlbum
        }
    }
}

pub(crate) enum TrackFetcherStep {
    FindTrackAlbum,
    DownloadTorrent,
    DownloadTrackAlbum,
    UploadTrackToRadioterio,
    AddTrackToRadioterioChannel,
    Finish,
}
