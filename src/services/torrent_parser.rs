use serde::Deserialize;
use serde_bytes::ByteBuf;

#[derive(Debug, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Info {
    name: String,
    pieces: ByteBuf,
    #[serde(rename = "piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    files: Option<Vec<File>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    root_hash: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Torrent {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    creation_date: Option<i64>,
    #[serde(rename = "comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    created_by: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum TorrentParserError {
    #[error(transparent)]
    SerdeError(#[from] serde_bencode::Error),
}

pub(crate) fn get_files_count(torrent_file_content: &[u8]) -> Result<usize, TorrentParserError> {
    let torrent = serde_bencode::from_bytes::<Torrent>(torrent_file_content)?;

    Ok(torrent.info.files.unwrap_or_default().len())
}

pub(crate) fn get_files(torrent_file_content: &[u8]) -> Result<Vec<String>, TorrentParserError> {
    let torrent = serde_bencode::from_bytes::<Torrent>(torrent_file_content)?;

    Ok(torrent
        .info
        .files
        .unwrap_or_default()
        .into_iter()
        .map(|f| f.path.join(std::path::MAIN_SEPARATOR_STR))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getting_files_count() {
        let contents = include_bytes!("../../tests/fixtures/example.torrent");
        let files_count = get_files_count(contents).unwrap();

        assert_eq!(18, files_count);
    }

    #[test]
    fn test_getting_files_list() {
        let contents = include_bytes!("../../tests/fixtures/example.torrent");
        let files = get_files(contents).unwrap();

        assert_eq!(
            vec![
                "00. Ted Irens - Life @ Mirror.m3u",
                "00. Ted Irens - Life @ Mirror.nfo",
                "01. Ted Irens - Sunday Breakfast.flac",
                "02. Ted Irens - Rain In The Forest.flac",
                "03. Ted Irens - Another Moon Night.flac",
                "04. Ted Irens - Rising Star.flac",
                "05. Ted Irens - Dreamland Trip.flac",
                "06. Ted Irens - Northern Lights.flac",
                "07. Ted Irens - Winter's Sunset.flac",
                "08. Ted Irens - Two Mountains.flac",
                "09. Ted Irens - Living In Clouds.flac",
                "10. Ted Irens - Summer Evening.flac",
                "11. Ted Irens - Crystal Driver.flac",
                "12. Ted Irens - Rider.flac",
                "13. Ted Irens - Dancing On The Moon.flac",
                "14. Ted Irens - The City.flac",
                "audiochecker.log",
                "Folder.jpg"
            ],
            files
        );
    }
}
