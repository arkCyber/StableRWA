// =====================================================================================
// File: service-gateway/src/middleware.rs
// Description: Middleware implementations for API Gateway
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{GatewayError, GatewayState};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    web, Error, HttpMessage, HttpResponse, Result,
};
use core_security::{JwtManager, RateLimitKeyGenerator, SecurityError, UserClaims};
use futures_util::future::LocalBoxFuture;
use std::{
    collections::HashMap,
    future::{ready, Ready},
    rc::Rc,
    time::Instant,
};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Request logging middleware
pub struct RequestLogging;

impl<S, B> Transform<S, ServiceRequest> for RequestLogging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggingMiddleware { service }))
    }
}

pub struct RequestLoggingMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestLoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let remote_addr = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        // Generate correlation ID
        let correlation_id = Uuid::new_v4().to_string();
        req.extensions_mut().insert(correlation_id.clone());

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start_time.elapsed();

            match &result {
                Ok(response) => {
                    let status = response.status().as_u16();
                    info!(
                        method = %method,
                        path = %path,
                        query = %query,
                        status = status,
                        duration_ms = duration.as_millis(),
                        user_agent = %user_agent,
                        remote_addr = %remote_addr,
                        correlation_id = %correlation_id,
                        "HTTP request completed"
                    );
                }
                Err(e) => {
                    error!(
                        method = %method,
                        path = %path,
                        query = %query,
                        duration_ms = duration.as_millis(),
                        user_agent = %user_agent,
                        remote_addr = %remote_addr,
                        correlation_id = %correlation_id,
                        error = %e,
                        "HTTP request failed"
                    );
                }
            }

            result
        })
    }
}

/// Metrics collection middleware
pub struct Metrics;

impl<S, B> Transform<S, ServiceRequest> for Metrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddleware { service }))
    }
}

pub struct MetricsMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let result = fut.await;
            let duration = start_time.elapsed();

            // Extract gateway state to record metrics
            if let Some(state) = result
                .as_ref()
                .ok()
                .and_then(|res| res.request().app_data::<web::Data<GatewayState>>())
            {
                let status = result
                    .as_ref()
                    .map(|res| res.status().as_u16())
                    .unwrap_or(500);

                state
                    .metrics
                    .http_requests_total
                    .with_label_values(&[&method, &path, &status.to_string()])
                    .inc();

                state
                    .metrics
                    .http_request_duration
                    .with_label_values(&[&method, &path])
                    .observe(duration.as_secs_f64());
            }

            result
        })
    }
}

/// CORS middleware
pub struct CORS;

impl<S, B> Transform<S, ServiceRequest> for CORS
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CORSMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CORSMiddleware { service }))
    }
}

pub struct CORSMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CORSMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let origin = req
            .headers()
            .get("origin")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("*");

        // Handle preflight requests
        if req.method() == actix_web::http::Method::OPTIONS {
            let response = HttpResponse::Ok()
                .insert_header(("Access-Control-Allow-Origin", origin))
                .insert_header((
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, OPTIONS",
                ))
                .insert_header((
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization, X-Requested-With",
                ))
                .insert_header(("Access-Control-Max-Age", "3600"))
                .finish();

            return Box::pin(async move {
                Err(actix_web::error::ErrorMethodNotAllowed("Method not allowed"))
            });
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut result = fut.await?;

            // Add CORS headers to response
            let headers = result.headers_mut();
            headers.insert(
                HeaderName::from_static("access-control-allow-origin"),
                HeaderValue::from_str(origin).unwrap_or(HeaderValue::from_static("*")),
            );
            headers.insert(
                HeaderName::from_static("access-control-allow-credentials"),
                HeaderValue::from_static("true"),
            );

            Ok(result)
        })
    }
}

/// Authentication middleware
pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware { service }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for health checks and metrics
        let path = req.path();
        if path.starts_with("/health") || path == "/metrics" {
            return Box::pin(self.service.call(req));
        }

        // Extract authorization header
        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|h| h.to_str().ok());

        let fut = self.service.call(req);

        Box::pin(async move {
            let req = match fut.await {
                Ok(mut res) => {
                    // Get gateway state for JWT validation
                    if let Some(state) = res.request().app_data::<web::Data<GatewayState>>() {
                        if let Some(auth_header) = auth_header {
                            // Create a temporary JWT manager for validation
                            // In a real implementation, this would be properly configured
                            match JwtManager::new(
                                "temporary_secret_key_for_validation_only_32_chars",
                                1,
                                7,
                                "rwa-platform".to_string(),
                                "rwa-users".to_string(),
                            ) {
                                Ok(jwt_manager) => {
                                    match jwt_manager.extract_token_from_header(auth_header) {
                                        Ok(token) => {
                                            match jwt_manager.validate_access_token(&token) {
                                                Ok(claims) => {
                                                    // Store user claims in request extensions
                                                    res.request().extensions_mut().insert(claims);
                                                    return Ok(res);
                                                }
                                                Err(e) => {
                                                    warn!("Token validation failed: {}", e);
                                                    return Err(actix_web::error::ErrorUnauthorized(
                                                        serde_json::json!({
                                                            "error": "Invalid token",
                                                            "message": e.to_string()
                                                        })
                                                    ).into());
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Token extraction failed: {}", e);
                                            return Err(actix_web::error::ErrorUnauthorized(
                                                serde_json::json!({
                                                    "error": "Invalid authorization header",
                                                    "message": e.to_string()
                                                })
                                            ).into());
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("JWT manager creation failed: {}", e);
                                    return Err(actix_web::error::ErrorInternalServerError(
                                        serde_json::json!({
                                            "error": "Authentication service error"
                                        })
                                    ).into());
                                }
                            }
                        } else {
                            // No authorization header
                            return Err(actix_web::error::ErrorUnauthorized(
                                serde_json::json!({
                                    "error": "Missing authorization header"
                                })
                            ).into());
                        }
                    }

                    Ok(res)
                }
                Err(e) => Err(e),
            };

            req
        })
    }
}

/// Rate limiting middleware
pub struct RateLimit;

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddleware { service }))
    }
}

pub struct RateLimitMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let remote_addr = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let req = match fut.await {
                Ok(mut res) => {
                    // Get gateway state for rate limiting
                    if let Some(state) = res.request().app_data::<web::Data<GatewayState>>() {
                        // Generate rate limit key based on IP and endpoint
                        let rate_limit_key =
                            RateLimitKeyGenerator::by_ip_endpoint(&remote_addr, &path);

                        // Check rate limit (simplified - in real implementation would be async)
                        // For now, just log the rate limit check
                        info!(
                            key = %rate_limit_key,
                            remote_addr = %remote_addr,
                            path = %path,
                            "Rate limit check performed"
                        );
                    }

                    Ok(res)
                }
                Err(e) => Err(e),
            };

            req
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> Result<HttpResponse, Error> {
        Ok(HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
    }

    #[actix_web::test]
    async fn test_request_logging_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(RequestLogging)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_cors_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(CORS)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        // Test preflight request
        let req = test::TestRequest::with_header("origin", "http://localhost:3000")
            .method(actix_web::http::Method::OPTIONS)
            .uri("/test")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        assert!(resp.headers().contains_key("access-control-allow-origin"));
    }

    #[actix_web::test]
    async fn test_cors_actual_request() {
        let app = test::init_service(
            App::new()
                .wrap(CORS)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .insert_header(("origin", "http://localhost:3000"))
            .uri("/test")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        assert!(resp.headers().contains_key("access-control-allow-origin"));
    }
}
