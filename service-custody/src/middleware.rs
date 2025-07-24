// =====================================================================================
// RWA Tokenization Platform - Custody Service Middleware
// 
// HTTP middleware for custody service including authentication, authorization,
// rate limiting, request logging, and security headers.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower::layer::util::Stack;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

/// Authentication middleware state
#[derive(Debug, Clone)]
pub struct AuthState {
    /// JWT secret key
    pub jwt_secret: String,
    /// Token expiration time
    pub token_expiration: Duration,
}

/// Rate limiting middleware state
#[derive(Debug)]
pub struct RateLimitState {
    /// Rate limit buckets per client
    buckets: Arc<RwLock<HashMap<String, RateLimitBucket>>>,
    /// Requests per minute limit
    requests_per_minute: u32,
    /// Bucket capacity
    bucket_capacity: u32,
}

/// Rate limit bucket for tracking client requests
#[derive(Debug, Clone)]
pub struct RateLimitBucket {
    /// Number of tokens in bucket
    tokens: u32,
    /// Last refill timestamp
    last_refill: Instant,
    /// Bucket capacity
    capacity: u32,
    /// Refill rate (tokens per second)
    refill_rate: f64,
}

/// Request context for middleware chain
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Request ID for tracing
    pub request_id: String,
    /// Client IP address
    pub client_ip: String,
    /// Authenticated user ID (if any)
    pub user_id: Option<String>,
    /// Request start time
    pub start_time: Instant,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Authentication middleware
/// 
/// # Arguments
/// 
/// * `auth_state` - Authentication state
/// * `request` - HTTP request
/// * `next` - Next middleware in chain
/// 
/// # Returns
/// 
/// Returns HTTP response
pub async fn auth_middleware(
    State(auth_state): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            // Validate JWT token (simplified implementation)
            if validate_jwt_token(token, &auth_state.jwt_secret) {
                // Extract user ID from token and add to request context
                if let Some(user_id) = extract_user_id_from_token(token) {
                    request.extensions_mut().insert(user_id);
                }
                return Ok(next.run(request).await);
            }
        }
    }

    // Return unauthorized if no valid token
    Err(StatusCode::UNAUTHORIZED)
}

/// Rate limiting middleware
/// 
/// # Arguments
/// 
/// * `rate_limit_state` - Rate limiting state
/// * `request` - HTTP request
/// * `next` - Next middleware in chain
/// 
/// # Returns
/// 
/// Returns HTTP response or rate limit error
pub async fn rate_limit_middleware(
    State(rate_limit_state): State<Arc<RateLimitState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client identifier (IP address or user ID)
    let client_id = extract_client_id(&request);
    
    // Check rate limit
    let mut buckets = rate_limit_state.buckets.write().await;
    let bucket = buckets
        .entry(client_id.clone())
        .or_insert_with(|| RateLimitBucket::new(rate_limit_state.bucket_capacity));

    if bucket.consume_token() {
        drop(buckets);
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}

/// Request logging middleware
/// 
/// # Arguments
/// 
/// * `request` - HTTP request
/// * `next` - Next middleware in chain
/// 
/// # Returns
/// 
/// Returns HTTP response with logging
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = uuid::Uuid::new_v4().to_string();

    // Add request context
    let mut request = request;
    let client_ip = extract_client_ip(&request);
    let user_id = request.extensions().get::<String>().cloned();

    request.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
        client_ip,
        user_id,
        start_time,
        metadata: HashMap::new(),
    });

    println!("Request started: {} {} {}", request_id, method, uri);

    let response = next.run(request).await;
    let duration = start_time.elapsed();

    println!(
        "Request completed: {} {} {} - {} - {:?}",
        request_id,
        method,
        uri,
        response.status(),
        duration
    );

    response
}

/// Security headers middleware
/// 
/// # Arguments
/// 
/// * `request` - HTTP request
/// * `next` - Next middleware in chain
/// 
/// # Returns
/// 
/// Returns HTTP response with security headers
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Add security headers
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

/// Create middleware stack for custody service
/// 
/// # Arguments
/// 
/// * `auth_state` - Authentication state
/// * `rate_limit_state` - Rate limiting state
/// 
/// # Returns
/// 
/// Returns configured middleware stack
pub fn create_middleware_stack(
    auth_state: AuthState,
    rate_limit_state: Arc<RateLimitState>,
) -> ServiceBuilder<
    Stack<
        axum::middleware::FromFnLayer<
            impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
        >,
        Stack<
            axum::middleware::FromFnLayer<
                impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>,
            >,
            Stack<
                axum::middleware::FromFnLayer<
                    impl Fn(
                        State<Arc<RateLimitState>>,
                        Request,
                        Next,
                    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>>,
                >,
                Stack<
                    axum::middleware::FromFnLayer<
                        impl Fn(
                            State<AuthState>,
                            Request,
                            Next,
                        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>>,
                    >,
                    Stack<CorsLayer, TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>>,
                >,
            >,
        >,
    >,
> {
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(axum::middleware::from_fn_with_state(auth_state, auth_middleware))
        .layer(axum::middleware::from_fn_with_state(rate_limit_state, rate_limit_middleware))
        .layer(axum::middleware::from_fn(logging_middleware))
        .layer(axum::middleware::from_fn(security_headers_middleware))
}

impl AuthState {
    /// Create new authentication state
    /// 
    /// # Arguments
    /// 
    /// * `jwt_secret` - JWT secret key
    /// * `token_expiration` - Token expiration duration
    /// 
    /// # Returns
    /// 
    /// Returns new authentication state
    pub fn new(jwt_secret: String, token_expiration: Duration) -> Self {
        Self {
            jwt_secret,
            token_expiration,
        }
    }
}

impl RateLimitState {
    /// Create new rate limiting state
    /// 
    /// # Arguments
    /// 
    /// * `requests_per_minute` - Maximum requests per minute
    /// * `bucket_capacity` - Token bucket capacity
    /// 
    /// # Returns
    /// 
    /// Returns new rate limiting state
    pub fn new(requests_per_minute: u32, bucket_capacity: u32) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            requests_per_minute,
            bucket_capacity,
        }
    }
}

impl RateLimitBucket {
    /// Create new rate limit bucket
    /// 
    /// # Arguments
    /// 
    /// * `capacity` - Bucket capacity
    /// 
    /// # Returns
    /// 
    /// Returns new rate limit bucket
    pub fn new(capacity: u32) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
            refill_rate: capacity as f64 / 60.0, // tokens per second
        }
    }

    /// Consume a token from the bucket
    /// 
    /// # Returns
    /// 
    /// Returns true if token was consumed, false if bucket is empty
    pub fn consume_token(&mut self) -> bool {
        self.refill_tokens();
        
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = (elapsed * self.refill_rate) as u32;
        
        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill = now;
        }
    }
}

/// Validate JWT token (simplified implementation)
/// 
/// # Arguments
/// 
/// * `token` - JWT token to validate
/// * `secret` - Secret key for validation
/// 
/// # Returns
/// 
/// Returns true if token is valid
fn validate_jwt_token(token: &str, secret: &str) -> bool {
    // This is a simplified implementation
    // In a real implementation, this would use a proper JWT library
    !token.is_empty() && !secret.is_empty()
}

/// Extract user ID from JWT token
/// 
/// # Arguments
/// 
/// * `token` - JWT token
/// 
/// # Returns
/// 
/// Returns user ID if found
fn extract_user_id_from_token(token: &str) -> Option<String> {
    // This is a simplified implementation
    // In a real implementation, this would decode the JWT payload
    if !token.is_empty() {
        Some("user123".to_string())
    } else {
        None
    }
}

/// Extract client identifier from request
/// 
/// # Arguments
/// 
/// * `request` - HTTP request
/// 
/// # Returns
/// 
/// Returns client identifier
fn extract_client_id(request: &Request) -> String {
    // Try to get user ID from extensions first
    if let Some(user_id) = request.extensions().get::<String>() {
        return user_id.clone();
    }
    
    // Fall back to IP address
    extract_client_ip(request)
}

/// Extract client IP address from request
/// 
/// # Arguments
/// 
/// * `request` - HTTP request
/// 
/// # Returns
/// 
/// Returns client IP address
fn extract_client_ip(request: &Request) -> String {
    // Check X-Forwarded-For header first
    if let Some(forwarded) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip) = forwarded_str.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }
    
    // Check X-Real-IP header
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Default to unknown
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Uri};
    use std::time::Duration;

    #[test]
    fn test_auth_state_creation() {
        let auth_state = AuthState::new(
            "test_secret".to_string(),
            Duration::from_secs(3600),
        );
        
        assert_eq!(auth_state.jwt_secret, "test_secret");
        assert_eq!(auth_state.token_expiration, Duration::from_secs(3600));
    }

    #[test]
    fn test_rate_limit_state_creation() {
        let rate_limit_state = RateLimitState::new(100, 10);
        
        assert_eq!(rate_limit_state.requests_per_minute, 100);
        assert_eq!(rate_limit_state.bucket_capacity, 10);
    }

    #[test]
    fn test_rate_limit_bucket_creation() {
        let bucket = RateLimitBucket::new(10);
        
        assert_eq!(bucket.tokens, 10);
        assert_eq!(bucket.capacity, 10);
        assert_eq!(bucket.refill_rate, 10.0 / 60.0);
    }

    #[test]
    fn test_rate_limit_bucket_token_consumption() {
        let mut bucket = RateLimitBucket::new(5);
        
        // Should be able to consume tokens
        assert!(bucket.consume_token());
        assert_eq!(bucket.tokens, 4);
        
        assert!(bucket.consume_token());
        assert_eq!(bucket.tokens, 3);
    }

    #[test]
    fn test_rate_limit_bucket_exhaustion() {
        let mut bucket = RateLimitBucket::new(2);
        
        // Consume all tokens
        assert!(bucket.consume_token());
        assert!(bucket.consume_token());
        
        // Should not be able to consume more
        assert!(!bucket.consume_token());
        assert_eq!(bucket.tokens, 0);
    }

    #[test]
    fn test_jwt_token_validation() {
        assert!(validate_jwt_token("valid_token", "secret"));
        assert!(!validate_jwt_token("", "secret"));
        assert!(!validate_jwt_token("token", ""));
    }

    #[test]
    fn test_user_id_extraction() {
        let user_id = extract_user_id_from_token("valid_token");
        assert_eq!(user_id, Some("user123".to_string()));
        
        let empty_user_id = extract_user_id_from_token("");
        assert_eq!(empty_user_id, None);
    }

    #[test]
    fn test_request_context_creation() {
        let context = RequestContext {
            request_id: "req_123".to_string(),
            client_ip: "192.168.1.1".to_string(),
            user_id: Some("user123".to_string()),
            start_time: Instant::now(),
            metadata: HashMap::new(),
        };
        
        assert_eq!(context.request_id, "req_123");
        assert_eq!(context.client_ip, "192.168.1.1");
        assert_eq!(context.user_id, Some("user123".to_string()));
    }

    #[tokio::test]
    async fn test_rate_limit_bucket_refill() {
        let mut bucket = RateLimitBucket::new(5);
        
        // Consume all tokens
        for _ in 0..5 {
            assert!(bucket.consume_token());
        }
        assert!(!bucket.consume_token());
        
        // Wait a bit and manually trigger refill
        tokio::time::sleep(Duration::from_millis(100)).await;
        bucket.last_refill = Instant::now() - Duration::from_secs(1);
        
        // Should be able to consume again after refill
        assert!(bucket.consume_token());
    }

    #[test]
    fn test_client_ip_extraction_fallback() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        
        let ip = extract_client_ip(&request);
        assert_eq!(ip, "unknown");
    }

    #[test]
    fn test_client_id_extraction_with_user() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        
        request.extensions_mut().insert("user456".to_string());
        
        let client_id = extract_client_id(&request);
        assert_eq!(client_id, "user456");
    }

    #[test]
    fn test_client_id_extraction_fallback() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(axum::body::Body::empty())
            .unwrap();
        
        let client_id = extract_client_id(&request);
        assert_eq!(client_id, "unknown");
    }
}
