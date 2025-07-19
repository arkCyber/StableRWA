// =====================================================================================
// File: service-user/src/main.rs
// Description: Main entry point for the User Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{middleware, web, App, HttpServer, Result as ActixResult};
use core_config::{AppConfig, ConfigError};
use core_database::DatabaseManager;
use core_observability::{init_tracing, BusinessMetrics};
use core_security::jwt::{JwtConfig, JwtManager};
use service_user::{
    handlers,
    models::UserRepository,
    service::UserService,
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
    info!("Starting User Service");

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

    // Initialize JWT manager
    let jwt_config = JwtConfig {
        secret: config.security.jwt_secret.clone(),
        expiration: config.security.jwt_expiration,
        refresh_expiration: config.security.jwt_refresh_expiration,
        issuer: "rwa-platform".to_string(),
        audience: "rwa-api".to_string(),
    };
    let jwt_manager = Arc::new(JwtManager::new(jwt_config));

    // Initialize repository and service
    let user_repository = Arc::new(UserRepository::new(database_manager.get_pool()));
    let user_service = Arc::new(UserService::new(
        user_repository.clone(),
        jwt_manager.clone(),
        metrics.clone(),
    ));

    // Create application state
    let app_state = web::Data::new(AppState {
        config: config.clone(),
        user_service,
        metrics,
        database: database_manager,
    });

    // Get server configuration
    let host = config.server.host.clone();
    let port = config.server.port;
    let workers = config.server.workers.unwrap_or_else(num_cpus::get);

    info!("User Service configuration:");
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
                .add(("X-Service", "user-service"))
            )
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::register_user))
                            .route("/login", web::post().to(handlers::login_user))
                            .route("/logout", web::post().to(handlers::logout_user))
                            .route("/refresh", web::post().to(handlers::refresh_token))
                            .route("/verify-email", web::post().to(handlers::verify_email))
                            .route("/forgot-password", web::post().to(handlers::forgot_password))
                            .route("/reset-password", web::post().to(handlers::reset_password))
                    )
                    .service(
                        web::scope("/users")
                            .route("", web::get().to(handlers::list_users))
                            .route("", web::post().to(handlers::create_user))
                            .route("/profile", web::get().to(handlers::get_user_profile))
                            .route("/profile", web::put().to(handlers::update_user_profile))
                            .route("/{id}", web::get().to(handlers::get_user))
                            .route("/{id}", web::put().to(handlers::update_user))
                            .route("/{id}", web::delete().to(handlers::delete_user))
                            .route("/{id}/activate", web::post().to(handlers::activate_user))
                            .route("/{id}/deactivate", web::post().to(handlers::deactivate_user))
                    )
                    .service(
                        web::scope("/sessions")
                            .route("", web::get().to(handlers::list_user_sessions))
                            .route("/{id}", web::delete().to(handlers::revoke_session))
                            .route("/revoke-all", web::post().to(handlers::revoke_all_sessions))
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

    info!("🚀 User Service started successfully");
    info!("📡 Listening on http://{}:{}", host, port);
    info!("📊 Metrics available at http://{}:{}/metrics", host, port);
    info!("🏥 Health check at http://{}:{}/health", host, port);

    // Start the server
    server.run().await
}

/// Load application configuration
async fn load_config() -> Result<AppConfig, ConfigError> {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config/user-service.toml".to_string());

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
