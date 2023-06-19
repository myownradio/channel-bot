use crate::services::track_request_processor::AudioMetadata;
use reqwest::Client;

const OPENAI_ENDPOINT: &str = "https://api.openai.com";

pub(crate) struct OpenAIService {
    openai_api_key: String,
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum OpenAIServiceError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

impl OpenAIService {
    pub(crate) fn create(openai_api_key: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP Client");

        Self {
            openai_api_key,
            client,
        }
    }

    pub(crate) async fn get_audio_tracks_suggestion(
        &self,
        tracks_list: &Vec<AudioMetadata>,
    ) -> Result<Vec<AudioMetadata>, OpenAIServiceError> {
        let tracks_list_str = tracks_list
            .iter()
            .map(|m| format!("{} - {}", m.artist, m.title))
            .collect::<Vec<_>>();

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", OPENAI_ENDPOINT))
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .json(&serde_json::json!({
                "model": "gpt-3.5-turbo",
                "messages": [
                    {"role": "system", "content": "The user will provide you with a list of audio tracks. One track per each line.\n\nSuggest 2 new audio tracks to that list that will ideally fit existing ones in terms of vibe and mood.\n\nProvide a response as an array of objects with fields: \"title\", \"artist\" and \"album\". Without any additional comments and descriptions."},
                    {"role": "user", "content": tracks_list_str}
                ]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        let response_content = response
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .and_then(|str| serde_json::from_str::<Vec<AudioMetadata>>(str).ok())
            .unwrap_or_default();

        Ok(response_content)
    }
}
