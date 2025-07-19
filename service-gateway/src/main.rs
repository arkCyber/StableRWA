// =====================================================================================
// File: service-gateway/src/main.rs
// Description: Main entry point for the StableRWA Framework API Gateway
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

use actix_web::{middleware, web, App, HttpServer, Result as ActixResult};
use core_config::{AppConfig, ConfigError};
use core_observability::{init_tracing, BusinessMetrics};
use core_security::jwt::{JwtConfig, JwtManager};
use service_gateway::{
    auth::AuthService,
    rate_limit::{RateLimiter, RateLimitConfig},
    routing::ServiceRegistry,
    GatewayState,
};
use std::sync::Arc;
use tracing::{error, info, warn};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize framework configuration
    let config = match load_config().await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load framework configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize observability stack
    init_tracing(&config.observability).expect("Failed to initialize tracing");
    info!("Starting StableRWA Framework API Gateway");

    // Initialize business metrics collector
    let metrics = Arc::new(BusinessMetrics::new());

    // Initialize JWT authentication manager
    let jwt_config = JwtConfig {
        secret: config.security.jwt_secret.clone(),
        expiration: config.security.jwt_expiration,
        refresh_expiration: config.security.jwt_refresh_expiration,
        issuer: "stablerwa-framework".to_string(),
        audience: "stablerwa-api".to_string(),
    };
    let jwt_manager = JwtManager::new(jwt_config);

    // Initialize authentication service
    let auth_service = Arc::new(AuthService::new(jwt_manager));

    // Initialize rate limiting service
    let rate_limit_config = RateLimitConfig {
        requests_per_minute: config.security.rate_limit_requests_per_minute,
        burst_size: config.security.rate_limit_burst_size,
        ..Default::default()
    };
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config));

    // Initialize microservice registry
    let service_registry = Arc::new(ServiceRegistry::new(&config).await?);

    // Create gateway application state
    let gateway_state = web::Data::new(GatewayState {
        config: config.clone(),
        metrics: metrics.clone(),
        service_registry,
        rate_limiter,
    });

    // Extract server configuration
    let host = config.server.host.clone();
    let port = config.server.port;
    let workers = config.server.workers.unwrap_or_else(num_cpus::get);

    info!("Gateway configuration:");
    info!("  Host: {}", host);
    info!("  Port: {}", port);
    info!("  Workers: {}", workers);
    info!("  Environment: {}", config.environment);

    // Start HTTP server
    let server = HttpServer::new(move || {
        service_gateway::create_app(gateway_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new()
                .add(("X-Version", env!("CARGO_PKG_VERSION")))
                .add(("X-Service", "rwa-gateway"))
            )
    })
    .workers(workers)
    .bind(format!("{}:{}", host, port))?;

    info!("🚀 RWA Platform API Gateway started successfully");
    info!("📡 Listening on http://{}:{}", host, port);
    info!("📊 Metrics available at http://{}:{}/metrics", host, port);
    info!("🏥 Health check at http://{}:{}/health", host, port);

    // Start the server
    server.run().await
}

/// Load application configuration
async fn load_config() -> Result<AppConfig, ConfigError> {
    let config_path = std::env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config/gateway.toml".to_string());

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

/// Graceful shutdown handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }

    info!("Shutting down gracefully...");
}

/// Health check for the gateway service
pub async fn gateway_health_check() -> ActixResult<web::Json<serde_json::Value>> {
    Ok(web::Json(serde_json::json!({
        "status": "healthy",
        "service": "rwa-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })))
}

/// Readiness check for the gateway service
pub async fn gateway_readiness_check(
    state: web::Data<GatewayState>,
) -> ActixResult<web::Json<serde_json::Value>> {
    // Check if all downstream services are available
    let service_health = state.service_registry.check_all_services().await;
    
    let ready = service_health.iter().all(|(_, healthy)| *healthy);
    
    let status = if ready { "ready" } else { "not_ready" };
    
    Ok(web::Json(serde_json::json!({
        "status": status,
        "service": "rwa-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": service_health
    })))
}

/// Liveness check for the gateway service
pub async fn gateway_liveness_check() -> ActixResult<web::Json<serde_json::Value>> {
    // Simple liveness check - if we can respond, we're alive
    Ok(web::Json(serde_json::json!({
        "status": "alive",
        "service": "rwa-gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Metrics endpoint for Prometheus scraping
pub async fn metrics_endpoint(
    state: web::Data<GatewayState>,
) -> ActixResult<String> {
    let metrics = state.metrics.export_prometheus_metrics().await;
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(gateway_health_check))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_liveness_check() {
        let app = test::init_service(
            App::new().route("/health/live", web::get().to(gateway_liveness_check))
        ).await;

        let req = test::TestRequest::get().uri("/health/live").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}

/// Configuration validation
fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }

    if config.security.jwt_secret.len() < 32 {
        return Err("JWT secret must be at least 32 characters long".to_string());
    }

    if config.security.rate_limit_requests_per_minute == 0 {
        return Err("Rate limit requests per minute cannot be 0".to_string());
    }

    Ok(())
}

/// Initialize panic handler
fn init_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        error!("Gateway panic occurred: {:?}", panic_info);
        
        // In production, you might want to send this to an error tracking service
        if let Some(location) = panic_info.location() {
            error!("Panic location: {}:{}", location.file(), location.line());
        }
        
        if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
            error!("Panic payload: {}", payload);
        }
    }));
}

/// Print startup banner
fn print_banner() {
    println!(r#"
    ██████╗ ██╗    ██╗ █████╗     ██████╗  █████╗ ████████╗███████╗██╗    ██╗ █████╗ ██╗   ██╗
    ██╔══██╗██║    ██║██╔══██╗    ██╔════╝ ██╔══██╗╚══██╔══╝██╔════╝██║    ██║██╔══██╗╚██╗ ██╔╝
    ██████╔╝██║ █╗ ██║███████║    ██║  ███╗███████║   ██║   █████╗  ██║ █╗ ██║███████║ ╚████╔╝ 
    ██╔══██╗██║███╗██║██╔══██║    ██║   ██║██╔══██║   ██║   ██╔══╝  ██║███╗██║██╔══██║  ╚██╔╝  
    ██║  ██║╚███╔███╔╝██║  ██║    ╚██████╔╝██║  ██║   ██║   ███████╗╚███╔███╔╝██║  ██║   ██║   
    ╚═╝  ╚═╝ ╚══╝╚══╝ ╚═╝  ╚═╝     ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚══════╝ ╚══╝╚══╝ ╚═╝  ╚═╝   ╚═╝   
    
    Real World Asset Platform - API Gateway
    Version: {}
    Environment: Production Ready
    "#, env!("CARGO_PKG_VERSION"));
}
