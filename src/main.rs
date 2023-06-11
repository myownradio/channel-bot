use crate::config::Config;
use crate::services::search_provider::RuTrackerClient;
use crate::services::storage::MemoryBasedStorage;
use crate::services::{SearchProvider, StateStorage, TrackRequestProcessor};
use actix_rt::signal::unix;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use futures_lite::FutureExt;
use std::sync::Arc;
use tracing::{error, info};

mod config;
mod http;
mod services;
pub(crate) mod types;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut terminate = unix::signal(unix::SignalKind::terminate())?;
    let mut interrupt = unix::signal(unix::SignalKind::interrupt())?;

    let config = Arc::new(Config::from_env());

    info!("Starting application...");

    let state_storage = Arc::new(MemoryBasedStorage::new());
    let rutracker_client = Arc::new(
        RuTrackerClient::create(
            &config.rutracker_credentials.username,
            &config.rutracker_credentials.password,
        )
        .await
        .expect("Unable to initialize RuTracker client"),
    );
    let downloader = Arc::new(todo!());
    let metadata_service = Arc::new(todo!());
    let radio_manager_client = Arc::new(todo!());

    let track_request_processor = TrackRequestProcessor::new(
        Arc::clone(&state_storage) as Arc<dyn StateStorage>,
        Arc::clone(&rutracker_client) as Arc<dyn SearchProvider>,
        Arc::clone(&downloader),
        Arc::clone(&metadata_service),
        Arc::clone(&radio_manager_client),
    );

    let shutdown_timeout = config.shutdown_timeout.clone();
    let bind_address = config.bind_address.clone();

    let server =
        HttpServer::new({ move || App::new().app_data(Data::new(track_request_processor)) })
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
