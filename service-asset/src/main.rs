// =====================================================================================
// File: service-asset/src/main.rs
// Description: Asset management service - Production-grade microservice for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, App, HttpServer, middleware::Logger};
use std::env;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;
mod service;

use handlers::*;
use service::AssetService;

/// Main function to start the Asset Service
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", host, port);

    info!("Starting Asset Service on {}", bind_address);

    // Initialize services
    let asset_service = web::Data::new(AssetService::new().await.map_err(|e| {
        error!("Failed to initialize asset service: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(asset_service.clone())
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .service(get_assets)
                    .service(get_asset)
                    .service(create_asset)
                    .service(update_asset)
                    .service(delete_asset)
                    .service(tokenize_asset)
                    .service(get_asset_valuation)
                    .service(update_asset_valuation)
                    .service(get_asset_documents)
                    .service(upload_asset_document)
            )
            .service(health_check)
            .service(readiness_check)
            .service(metrics)
    })
    .bind(&bind_address)?
    .run()
    .await
}

