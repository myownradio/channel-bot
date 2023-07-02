use crate::services::{RadioManagerClient, TransmissionClient};
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use search_providers::RuTrackerClient;
use std::sync::Arc;
use tracing::error;

pub(crate) async fn readiness_check(
    transmission_client: Data<Arc<TransmissionClient>>,
    radio_manager_client: Data<Arc<RadioManagerClient>>,
    rutracker_client: Data<Arc<RuTrackerClient>>,
) -> impl Responder {
    if let Err(error) = transmission_client.check_connection().await {
        error!(?error, "Readiness check failed");
    }

    if let Err(error) = radio_manager_client.check_connection().await {
        error!(?error, "Readiness check failed");
    }

    if let Err(error) = rutracker_client.check_connection().await {
        error!(?error, "Readiness check failed");
    }

    HttpResponse::Ok().finish()
}
