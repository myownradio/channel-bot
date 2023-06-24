use crate::services::track_request_processor::{
    AudioMetadata, CreateRequestOptions, RadioManagerChannelId, TrackRequestController,
};
use crate::services::{OpenAIService, RadioManagerClient, TrackRequestProcessor};
use crate::types::UserId;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MakeTrackRequestData {
    #[serde(flatten)]
    metadata: AudioMetadata,
    target_channel_id: RadioManagerChannelId,
}

pub(crate) async fn make_track_request(
    track_request_controller: web::Data<Arc<TrackRequestController>>,
    params: web::Json<MakeTrackRequestData>,
) -> impl Responder {
    let query = params.into_inner();
    let user_id = UserId(1); // Not used yet

    let request_id = match track_request_controller
        .create_request(&user_id, &query.metadata, &query.target_channel_id)
        .await
    {
        Err(error) => {
            error!(?error, "Unable to create track request");
            return HttpResponse::InternalServerError().finish();
        }
        Ok(request_id) => request_id,
    };

    HttpResponse::Accepted().json(serde_json::json!({
        "requestId": request_id,
    }))
}

#[derive(Deserialize)]
pub(crate) struct MakeTracksSuggestionData {
    target_channel_id: RadioManagerChannelId,
}

pub(crate) async fn make_tracks_suggestion(
    track_request_processor: web::Data<Arc<TrackRequestProcessor>>,
    openai_service: web::Data<Arc<OpenAIService>>,
    radio_manager_client: web::Data<Arc<RadioManagerClient>>,
    params: web::Json<MakeTracksSuggestionData>,
) -> impl Responder {
    let query = params.into_inner();
    let user_id = UserId(1); // Not used yet

    let tracks = radio_manager_client
        .get_channel_tracks(&query.target_channel_id)
        .await
        .expect("Unable to get channel tracks")
        .into_iter()
        .map(|t| AudioMetadata {
            title: t.title,
            artist: t.artist,
            album: t.album,
        })
        .collect();

    let suggested_tracks = openai_service
        .get_audio_tracks_suggestion(&tracks)
        .await
        .expect("Unable to get suggestions");

    for track in suggested_tracks {
        let request_id = match track_request_processor
            .create_request(
                &user_id,
                &track,
                &CreateRequestOptions {
                    validate_metadata: false,
                },
                &query.target_channel_id,
            )
            .await
        {
            Ok(request_id) => request_id,
            Err(error) => {
                error!(?error, "Unable to create track request");
                return HttpResponse::InternalServerError().finish();
            }
        };

        match track_request_processor
            .process_request(&user_id, &request_id)
            .await
        {
            Ok(()) => (),
            Err(error) => {
                error!(?error, "Unable to process track request");
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    HttpResponse::Ok().finish()
}

pub(crate) async fn get_track_request_statuses(
    track_request_processor: web::Data<Arc<TrackRequestProcessor>>,
) -> impl Responder {
    let user_id = UserId(1); // Not used yet

    let statuses = match track_request_processor
        .get_processing_requests(&user_id)
        .await
    {
        Ok(statuses) => statuses,
        Err(error) => {
            error!(?error, "Unable to get track processing statuses");
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(statuses)
}
