use audiotags::Tag;
use request_processors::{AudioMetadata, MetadataServiceError};
use tracing::error;

pub(crate) struct MetadataService;

impl request_processors::MetadataService for MetadataService {
    async fn get_audio_metadata(
        &self,
        file_path: &str,
    ) -> Result<Option<AudioMetadata>, MetadataServiceError> {
        match Tag::new().read_from_path(file_path) {
            Ok(tags) => Ok(Some(AudioMetadata {
                title: tags.title().unwrap_or_default().to_string(),
                artist: tags.artist().unwrap_or_default().to_string(),
                album: tags.album().unwrap_or_default().to_string(),
            })),
            Err(error) => {
                error!(?error, file_path, "Unable to read audio file metadata");
                Err(MetadataServiceError(Box::new(error)))
            }
        }
    }
}
