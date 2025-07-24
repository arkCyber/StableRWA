// =====================================================================================
// RWA Tokenization Platform - Oracle Service Library
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod config;
pub mod error;
pub mod models;
pub mod service;
pub mod handlers;
pub mod providers;
pub mod aggregator;
pub mod cache;
pub mod health;
pub mod metrics;

use actix_web::{web, HttpResponse};
use serde_json::json;
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub oracle_service: Arc<dyn service::OracleServiceTrait + Send + Sync>,
    pub health_service: Arc<health::HealthService>,
    pub metrics: Arc<metrics::OracleMetrics>,
}

/// Configure application routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Health endpoints
        .route("/health", web::get().to(handlers::health_check))
        .route("/health/ready", web::get().to(handlers::readiness_check))
        .route("/health/live", web::get().to(handlers::liveness_probe))
        .route("/metrics", web::get().to(handlers::metrics_endpoint))

        // API v1 routes
        .service(
            web::scope("/api/v1")
                // Price endpoints
                .route("/prices/{asset_id}", web::get().to(handlers::get_asset_price))
                .route("/prices/{asset_id}/history", web::get().to(handlers::get_price_history))
                .route("/prices/batch", web::post().to(handlers::get_batch_prices))

                // Feed management
                .route("/feeds", web::get().to(handlers::list_price_feeds))
                .route("/feeds", web::post().to(handlers::create_price_feed))
                .route("/feeds/{feed_id}", web::get().to(handlers::get_price_feed))
                .route("/feeds/{feed_id}", web::put().to(handlers::update_price_feed))
                .route("/feeds/{feed_id}", web::delete().to(handlers::delete_price_feed))

                // Subscriptions
                .route("/feeds/{feed_id}/subscribe", web::post().to(handlers::subscribe_to_feed))
                .route("/subscriptions/{subscription_id}", web::delete().to(handlers::unsubscribe_from_feed))

                // Providers
                .route("/providers", web::get().to(handlers::list_providers))
                .route("/providers/{provider_id}", web::get().to(handlers::get_provider_status))
        )

        // Root endpoint
        .route("/", web::get().to(|| async {
            HttpResponse::Ok().json(json!({
                "service": "RWA Oracle Service",
                "version": env!("CARGO_PKG_VERSION"),
                "status": "running",
                "timestamp": chrono::Utc::now()
            }))
        }));
}

/// Oracle service error type
pub use error::OracleError;

/// Oracle service result type
pub type OracleResult<T> = Result<T, OracleError>;
