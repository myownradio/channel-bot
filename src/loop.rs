use crate::types::{AudioTrack, DownloadCandidate};



enum ProcessingContext {
    
}

enum State {
    GetTracksFromSpotify,
    FilterTracks {
        tracks: Vec<AudioTrack>,
    },
    DownloadTracks {
        tracks: Vec<AudioTrack>,
        candidates: Vec<DownloadCandidate>,
    },
}

async fn run(state: State) -> State {
    match state {
        State::GetTracksFromSpotify => {
            // TODO: Download liked tracks from Spotify
            let tracks = vec![];
            State::FilterTracks { tracks }
        }
        State::FilterTracks { tracks } => {
            // TODO: Filter new liked tracks
            State::DownloadTracks(tracks.map)
        }
    }
}
