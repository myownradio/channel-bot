use crate::config::Config;
use crate::services::{
    MetadataService, OpenAIService, RadioManagerClient, TrackRequestProcessor, TransmissionClient,
};
use crate::storage::on_disk::OnDiskStorage;
use actix_rt::signal::unix;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use futures_lite::FutureExt;
use std::sync::Arc;
use tracing::{error, info};

mod config;
mod http;
mod impls;
mod services;
mod storage;
mod types;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut terminate = unix::signal(unix::SignalKind::terminate())?;
    let mut interrupt = unix::signal(unix::SignalKind::interrupt())?;

    dotenv::dotenv().ok();
    env_logger::init();

    let config = Arc::from(Config::from_env());

    info!("Starting application...");

    let state_storage = OnDiskStorage::create(config.state_storage_directory.clone());
    let rutracker_client = search_providers::RuTrackerClient::create(
        &config.rutracker.username,
        &config.rutracker.password,
    )
    .await
    .expect("Unable to initialize RuTracker client");
    let transmission_client = TransmissionClient::create(
        config.transmission.transmission_rpc_endpoint.clone(),
        config.transmission.username.clone(),
        config.transmission.password.clone(),
        config.transmission.download_directory.clone(),
    );
    let radio_manager_client = Arc::new(
        RadioManagerClient::create(
            &config.radiomanager.endpoint,
            &config.radiomanager.username,
            &config.radiomanager.password,
        )
        .await
        .expect("Unable to initialize RadioManager client"),
    );
    let metadata_service = MetadataService::new();

    let track_request_processor = {
        Arc::new(TrackRequestProcessor::new(
            Arc::from(state_storage),
            Arc::from(rutracker_client),
            Arc::from(transmission_client),
            Arc::from(metadata_service),
            radio_manager_client.clone(),
            config.download_directory.clone(),
        ))
    };
    let openai_service = Arc::new(OpenAIService::create(config.openai_api_key.clone()));

    let shutdown_timeout = config.shutdown_timeout.clone();
    let bind_address = config.bind_address.clone();

    let server = HttpServer::new({
        move || {
            App::new()
                .app_data(Data::new(Arc::clone(&track_request_processor)))
                .app_data(Data::new(Arc::clone(&openai_service)))
                .app_data(Data::new(Arc::clone(&radio_manager_client)))
                .service(web::resource("/").route(web::get().to(http::get_track_request_statuses)))
                .service(web::resource("/create").route(web::post().to(http::make_track_request)))
                .service(
                    web::resource("/suggest").route(web::post().to(http::make_tracks_suggestion)),
                )
        }
    })
    .shutdown_timeout(shutdown_timeout)
    .bind(bind_address)?
    .run();

    let server_handle = server.handle();

    actix_rt::spawn({
        async move {
            if let Err(error) = server.await {
                error!(?error, "Error on http server");
            }
        }
    });

    info!("Application started");

    interrupt.recv().or(terminate.recv()).await;

    info!("Received shutdown signal. Shutting down gracefully...");

    server_handle.stop(true).await;

    Ok(())
}
