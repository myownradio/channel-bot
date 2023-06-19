use crate::services::track_request_processor::{
    AudioMetadata, CreateRequestOptions, RadioManagerChannelId,
};
use crate::services::TrackRequestProcessor;
use crate::types::UserId;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tracing::error;

#[derive(Deserialize)]
pub(crate) struct MakeTrackRequestData {
    #[serde(flatten)]
    metadata: AudioMetadata,
    #[serde(default)]
    validate_metadata: bool,
    target_channel_id: RadioManagerChannelId,
}

pub(crate) async fn make_track_request(
    track_request_processor: web::Data<Arc<TrackRequestProcessor>>,
    params: web::Json<MakeTrackRequestData>,
) -> impl Responder {
    let query = params.into_inner();
    let user_id = UserId(1); // Not used yet

    let request_id = match track_request_processor
        .create_request(
            &user_id,
            &query.metadata,
            &CreateRequestOptions {
                validate_metadata: query.validate_metadata,
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
        Ok(()) => HttpResponse::Ok().finish(),
        Err(error) => {
            error!(?error, "Unable to process track request");
            return HttpResponse::InternalServerError().finish();
        }
    }
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
