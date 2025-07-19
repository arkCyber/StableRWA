// =====================================================================================
// File: service-gateway/src/lib.rs
// Description: Production-grade API Gateway for RWA platform. Handles routing,
//              authentication, rate limiting, load balancing, and observability.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod auth;
pub mod middleware;
pub mod proxy;
pub mod rate_limit;
pub mod routing;
pub mod health;

use actix_web::{web, App, HttpServer, Result as ActixResult};
use core_config::AppConfig;
use core_observability::BusinessMetrics;
use core_security::SecurityError;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, error};

/// Gateway-specific errors
#[derive(Error, Debug)]
pub enum GatewayError {
    #[error("Route not found: {0}")]
    RouteNotFound(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Proxy error: {0}")]
    ProxyError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl From<SecurityError> for GatewayError {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::AuthenticationFailed(msg) => GatewayError::AuthenticationFailed(msg),
            SecurityError::RateLimitExceeded => GatewayError::RateLimitExceeded,
            _ => GatewayError::AuthenticationFailed(err.to_string()),
        }
    }
}

/// Gateway application state
pub struct GatewayState {
    pub config: AppConfig,
    pub metrics: Arc<BusinessMetrics>,
    pub service_registry: Arc<routing::ServiceRegistry>,
    pub rate_limiter: Arc<rate_limit::RateLimiter>,
}

/// Create and configure the gateway application
pub fn create_app(state: web::Data<GatewayState>) -> App<impl actix_web::dev::ServiceFactory<
    actix_web::dev::ServiceRequest,
    Config = (),
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
    InitError = (),
>> {
    App::new()
        .app_data(state)
        .wrap(middleware::RequestLogging)
        .wrap(middleware::Metrics)
        .wrap(middleware::CORS)
        .service(
            web::scope("/api/v1")
                .wrap(middleware::Authentication)
                .wrap(middleware::RateLimit)
                .service(
                    web::scope("/assets")
                        .route("", web::get().to(proxy::proxy_to_asset_service))
                        .route("", web::post().to(proxy::proxy_to_asset_service))
                        .route("/{id}", web::get().to(proxy::proxy_to_asset_service))
                        .route("/{id}", web::put().to(proxy::proxy_to_asset_service))
                        .route("/{id}", web::delete().to(proxy::proxy_to_asset_service))
                )
                .service(
                    web::scope("/users")
                        .route("", web::get().to(proxy::proxy_to_user_service))
                        .route("", web::post().to(proxy::proxy_to_user_service))
                        .route("/{id}", web::get().to(proxy::proxy_to_user_service))
                        .route("/{id}", web::put().to(proxy::proxy_to_user_service))
                        .route("/{id}", web::delete().to(proxy::proxy_to_user_service))
                )
                .service(
                    web::scope("/payments")
                        .route("", web::get().to(proxy::proxy_to_payment_service))
                        .route("", web::post().to(proxy::proxy_to_payment_service))
                        .route("/{id}", web::get().to(proxy::proxy_to_payment_service))
                        .route("/{id}/status", web::get().to(proxy::proxy_to_payment_service))
                )
                .service(
                    web::scope("/blockchain")
                        .route("/ethereum/balance/{address}", web::get().to(proxy::proxy_to_asset_service))
                        .route("/solana/balance/{address}", web::get().to(proxy::proxy_to_asset_service))
                        .route("/polkadot/balance/{address}", web::get().to(proxy::proxy_to_asset_service))
                )
        )
        .service(
            web::scope("/auth")
                .route("/login", web::post().to(proxy::proxy_to_auth_service))
                .route("/logout", web::post().to(proxy::proxy_to_auth_service))
                .route("/refresh", web::post().to(proxy::proxy_to_auth_service))
        )
        .service(
            web::scope("/health")
                .route("", web::get().to(health::health_check))
                .route("/ready", web::get().to(health::readiness_check))
                .route("/live", web::get().to(health::liveness_check))
        )
        .route("/metrics", web::get().to(health::metrics_endpoint))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_route_request_success() {
        let result = route_request("/home");
        assert!(result.is_ok());
    }
    #[test]
    fn test_route_request_unauthorized() {
        let result = route_request("/forbidden");
        assert!(matches!(result, Err(GatewayError::Unauthorized(_))));
    }
}
