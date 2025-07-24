// =====================================================================================
// File: service-asset/src/middleware.rs
// Description: Enterprise-grade middleware for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::metrics::AssetMetrics;

/// Authentication middleware
pub struct AuthenticationMiddleware {
    jwt_secret: String,
    excluded_paths: Vec<String>,
}

impl AuthenticationMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/ready".to_string(),
                "/live".to_string(),
            ],
        }
    }
    
    pub fn exclude_path(mut self, path: String) -> Self {
        self.excluded_paths.push(path);
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthenticationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddlewareService {
            service,
            jwt_secret: self.jwt_secret.clone(),
            excluded_paths: self.excluded_paths.clone(),
        }))
    }
}

pub struct AuthenticationMiddlewareService<S> {
    service: S,
    jwt_secret: String,
    excluded_paths: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddlewareService<S>
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
        let path = req.path().to_string();
        let excluded = self.excluded_paths.iter().any(|p| path.starts_with(p));
        
        if excluded {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        // Extract and validate JWT token
        let auth_header = req.headers().get("Authorization");
        let token = match auth_header {
            Some(header) => {
                let header_str = header.to_str().unwrap_or("");
                if header_str.starts_with("Bearer ") {
                    Some(&header_str[7..])
                } else {
                    None
                }
            }
            None => None,
        };

        if let Some(token) = token {
            // In a real implementation, validate JWT token here
            // For now, we'll extract a mock user ID
            if let Ok(user_id) = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000") {
                req.extensions_mut().insert(user_id);
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await });
            }
        }

        // Return unauthorized response
        Box::pin(async move {
            let response = HttpResponse::Unauthorized()
                .json(serde_json::json!({
                    "error": "unauthorized",
                    "message": "Valid authentication token required"
                }));
            Ok(req.into_response(response))
        })
    }
}

/// Rate limiting middleware
pub struct RateLimitingMiddleware {
    requests_per_minute: u32,
    burst_size: u32,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
    burst_count: u32,
    last_request: Instant,
}

impl RateLimitingMiddleware {
    pub fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        Self {
            requests_per_minute,
            burst_size,
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitingMiddlewareService {
            service,
            requests_per_minute: self.requests_per_minute,
            burst_size: self.burst_size,
            store: self.store.clone(),
        }))
    }
}

pub struct RateLimitingMiddlewareService<S> {
    service: S,
    requests_per_minute: u32,
    burst_size: u32,
    store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimitingMiddlewareService<S>
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
        let client_ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        
        let store = self.store.clone();
        let requests_per_minute = self.requests_per_minute;
        let burst_size = self.burst_size;
        
        Box::pin(async move {
            let now = Instant::now();
            let mut store = store.write().await;
            
            let entry = store.entry(client_ip.clone()).or_insert(RateLimitEntry {
                count: 0,
                window_start: now,
                burst_count: 0,
                last_request: now,
            });
            
            // Reset window if a minute has passed
            if now.duration_since(entry.window_start).as_secs() >= 60 {
                entry.count = 0;
                entry.window_start = now;
            }
            
            // Reset burst if 10 seconds have passed
            if now.duration_since(entry.last_request).as_secs() >= 10 {
                entry.burst_count = 0;
            }
            
            entry.count += 1;
            entry.burst_count += 1;
            entry.last_request = now;
            
            // Check rate limits
            if entry.count > requests_per_minute || entry.burst_count > burst_size {
                warn!("Rate limit exceeded for client: {}", client_ip);
                let response = HttpResponse::TooManyRequests()
                    .json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": "Too many requests",
                        "retry_after": 60
                    }));
                return Ok(req.into_response(response));
            }
            
            drop(store); // Release the lock before calling the service
            
            self.service.call(req).await
        })
    }
}

/// Metrics middleware
pub struct MetricsMiddleware {
    metrics: AssetMetrics,
}

impl MetricsMiddleware {
    pub fn new(metrics: AssetMetrics) -> Self {
        Self { metrics }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService {
            service,
            metrics: self.metrics.clone(),
        }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
    metrics: AssetMetrics,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
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
        let metrics = self.metrics.clone();
        
        // Record request start
        metrics.record_http_request_start(&method, &path);
        
        let fut = self.service.call(req);
        Box::pin(async move {
            let result = fut.await;

            let duration = start_time.elapsed();

            // Simplified metrics recording
            info!(
                method = %method,
                path = %path,
                duration_ms = %duration.as_millis(),
                "Request processed"
            );

            result
        })
    }
}

/// Request ID middleware
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService { service }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
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
        // Generate or extract request ID
        let request_id = req
            .headers()
            .get("X-Request-ID")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Store request ID in extensions
        req.extensions_mut().insert(request_id.clone());
        
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut result = fut.await?;

            // Add request ID to response headers
            result.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                actix_web::http::header::HeaderValue::from_str(&request_id).unwrap(),
            );

            Ok(result)
        })
    }
}

/// CORS middleware configuration
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u32,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Request-ID".to_string(),
                "X-API-Key".to_string(),
            ],
            max_age: 3600,
        }
    }
}

/// Security headers middleware
pub struct SecurityHeadersMiddleware {
    csp_policy: String,
    hsts_max_age: u32,
}

impl SecurityHeadersMiddleware {
    pub fn new() -> Self {
        Self {
            csp_policy: "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".to_string(),
            hsts_max_age: 31536000, // 1 year
        }
    }
    
    pub fn with_csp_policy(mut self, policy: String) -> Self {
        self.csp_policy = policy;
        self
    }
    
    pub fn with_hsts_max_age(mut self, max_age: u32) -> Self {
        self.hsts_max_age = max_age;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddlewareService {
            service,
            csp_policy: self.csp_policy.clone(),
            hsts_max_age: self.hsts_max_age,
        }))
    }
}

pub struct SecurityHeadersMiddlewareService<S> {
    service: S,
    csp_policy: String,
    hsts_max_age: u32,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddlewareService<S>
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
        let csp_policy = self.csp_policy.clone();
        let hsts_max_age = self.hsts_max_age;
        
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut result = fut.await?;
            
            let headers = result.headers_mut();
            
            // Content Security Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::header::HeaderValue::from_str(&csp_policy).unwrap(),
            );
            
            // Strict Transport Security
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                actix_web::http::header::HeaderValue::from_str(&format!("max-age={}", hsts_max_age)).unwrap(),
            );
            
            // X-Content-Type-Options
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );
            
            // X-Frame-Options
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::header::HeaderValue::from_static("DENY"),
            );
            
            // X-XSS-Protection
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );
            
            // Referrer Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            
            Ok(result)
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
    async fn test_request_id_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIdMiddleware)
                .route("/test", web::get().to(test_handler))
        ).await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.headers().contains_key("x-request-id"));
    }

    #[actix_web::test]
    async fn test_security_headers_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(SecurityHeadersMiddleware::new())
                .route("/test", web::get().to(test_handler))
        ).await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.headers().contains_key("content-security-policy"));
        assert!(resp.headers().contains_key("strict-transport-security"));
        assert!(resp.headers().contains_key("x-content-type-options"));
    }
}
