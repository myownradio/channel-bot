use crate::services::TrackRequestProcessor;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};

pub(crate) async fn make_track_request(
    track_request_processor: Data<TrackRequestProcessor>,
) -> impl Responder {
    HttpResponse::Ok().finish()
}
