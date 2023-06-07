use crate::types::{DownloadId, RadioterioChannelId, RadioterioLinkId, RadioterioTrackId, TopicId};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct TrackFetcherContext {
    pub(crate) track_title: String,
    pub(crate) track_artist: String,
    pub(crate) track_album: String,
    pub(crate) target_channel_id: RadioterioChannelId,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct TrackFetcherState {
    pub(crate) tried_topics: Vec<TopicId>,
    pub(crate) download_id: Option<DownloadId>,
    pub(crate) path_to_downloaded_file: Option<String>,
    pub(crate) radioterio_track_id: Option<RadioterioTrackId>,
    pub(crate) radioterio_link_id: Option<RadioterioLinkId>,
}
