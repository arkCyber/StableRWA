// =====================================================================================
// File: service-payment/src/main.rs
// Description: Main entry point for the Payment Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{middleware, web, App, HttpServer, Result as ActixResult};
use core_config::{AppConfig, ConfigError};
use core_database::DatabaseManager;
use core_observability::{init_tracing, BusinessMetrics};
use service_payment::{
    handlers,
    models::PaymentRepository,
    service::PaymentService,
    AppState,
};
use std::sync::Arc;
use tracing::{error, info, warn};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize configuration
    let config = match load_config().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize tracing
    init_tracing(&config.observability).expect("Failed to initialize tracing");
    info!("Starting Payment Service");

    // Initialize database
    let database_manager = DatabaseManager::new(&config.database)
        .await
        .expect("Failed to initialize database");

    // Run migrations
    database_manager
        .run_migrations()
        .await
        .expect("Failed to run database migrations");

    // Initialize metrics
    let metrics = Arc::new(BusinessMetrics::new());

    // Initialize repository and service
    let payment_repository = Arc::new(PaymentRepository::new(database_manager.get_pool()));
    let payment_service = Arc::new(PaymentService::new(
        payment_repository.clone(),
        metrics.clone(),
        &config,
    ).await.expect("Failed to initialize payment service"));

    // Create application state
    let app_state = web::Data::new(AppState {
        config: config.clone(),
        payment_service,
        metrics,
        database: database_manager,
    });

    // Get server configuration
    let host = config.server.host.clone();
    let port = config.server.port;
    let workers = config.server.workers.unwrap_or_else(num_cpus::get);

    info!("Payment Service configuration:");
    info!("  Host: {}", host);
    info!("  Port: {}", port);
    info!("  Workers: {}", workers);
    info!("  Environment: {}", config.environment);

    // Start HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new()
                .add(("X-Version", env!("CARGO_PKG_VERSION")))
                .add(("X-Service", "payment-service"))
            )
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/payments")
                            .route("", web::get().to(handlers::list_payments))
                            .route("", web::post().to(handlers::create_payment))
                            .route("/{id}", web::get().to(handlers::get_payment))
                            .route("/{id}/status", web::get().to(handlers::get_payment_status))
                            .route("/{id}/cancel", web::post().to(handlers::cancel_payment))
                            .route("/{id}/refund", web::post().to(handlers::refund_payment))
                    )
                    .service(
                        web::scope("/payment-methods")
                            .route("", web::get().to(handlers::list_payment_methods))
                            .route("", web::post().to(handlers::create_payment_method))
                            .route("/{id}", web::get().to(handlers::get_payment_method))
                            .route("/{id}", web::put().to(handlers::update_payment_method))
                            .route("/{id}", web::delete().to(handlers::delete_payment_method))
                    )
                    .service(
                        web::scope("/webhooks")
                            .route("/stripe", web::post().to(handlers::stripe_webhook))
                            .route("/paypal", web::post().to(handlers::paypal_webhook))
                    )
            )
            .service(
                web::scope("/health")
                    .route("", web::get().to(handlers::health_check))
                    .route("/ready", web::get().to(handlers::readiness_check))
                    .route("/live", web::get().to(handlers::liveness_check))
            )
            .route("/metrics", web::get().to(handlers::metrics_endpoint))
    })
    .workers(workers)
    .bind(format!("{}:{}", host, port))?;

    info!("🚀 Payment Service started successfully");
    info!("📡 Listening on http://{}:{}", host, port);
    info!("📊 Metrics available at http://{}:{}/metrics", host, port);
    info!("🏥 Health check at http://{}:{}/health", host, port);

    // Start the server
    server.run().await
}

/// Load application configuration
async fn load_config() -> Result<AppConfig, ConfigError> {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config/payment-service.toml".to_string());

    match AppConfig::from_file(&config_path).await {
        Ok(config) => {
            info!("Configuration loaded from: {}", config_path);
            Ok(config)
        }
        Err(ConfigError::FileNotFound(_)) => {
            warn!("Configuration file not found, using environment variables");
            AppConfig::from_env().await
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(handlers::health_check))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
