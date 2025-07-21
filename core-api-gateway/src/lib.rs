// =====================================================================================
// File: core-api-gateway/src/lib.rs
// Description: Enterprise-grade API gateway and SDK for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core API Gateway Module
//! 
//! This module provides a comprehensive enterprise-grade API gateway and SDK generation
//! system for the StableRWA platform, supporting REST, GraphQL, WebSocket, and gRPC
//! protocols with advanced features like rate limiting, authentication, and monitoring.

pub mod error;
pub mod types;
pub mod rest;
pub mod graphql;
pub mod websocket;
pub mod grpc;
pub mod auth;
pub mod rate_limiting;
pub mod load_balancing;
pub mod circuit_breaker;
pub mod middleware;
pub mod sdk;
pub mod documentation;
pub mod monitoring;
pub mod service;

// Re-export main types and traits
pub use error::{APIGatewayError, APIGatewayResult};
pub use types::{
    APIRequest, APIResponse, APIKey, ClientCredentials, RateLimitConfig,
    LoadBalancerConfig, CircuitBreakerConfig, MiddlewareConfig
};
pub use rest::{
    RESTService, RESTConfig, RESTHandler, RESTRouter,
    OpenAPISpec, SwaggerUI, RESTMiddleware
};
pub use graphql::{
    GraphQLService, GraphQLConfig, GraphQLSchema, GraphQLResolver,
    GraphQLSubscription, GraphQLPlayground
};
pub use websocket::{
    WebSocketService, WebSocketConfig, WebSocketHandler,
    WebSocketConnection, WebSocketMessage, WebSocketBroadcast
};
pub use grpc::{
    GRPCService, GRPCConfig, GRPCHandler,
    ProtocolBuffer, GRPCReflection, GRPCHealthCheck
};
pub use auth::{
    AuthService, AuthConfig, JWTAuth, OAuth2Auth,
    APIKeyAuth, BasicAuth, BearerAuth
};
pub use rate_limiting::{
    RateLimiter, RateLimitConfig as RLConfig, TokenBucket,
    SlidingWindow, FixedWindow, DistributedRateLimit
};
pub use load_balancing::{
    LoadBalancer, LoadBalancerConfig as LBConfig, RoundRobin,
    WeightedRoundRobin, LeastConnections, HealthBasedRouting
};
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig as CBConfig, CircuitState,
    FailureDetector, RecoveryStrategy, BulkheadPattern
};
pub use middleware::{
    MiddlewareStack, MiddlewareConfig as MConfig, RequestMiddleware,
    ResponseMiddleware, ErrorMiddleware, LoggingMiddleware
};
pub use sdk::{
    SDKGenerator, SDKConfig, TypeScriptSDK, PythonSDK,
    JavaSDK, GoSDK, RustSDK, PHPSDK
};
pub use documentation::{
    DocumentationGenerator, DocConfig, APIDocumentation,
    InteractiveDocumentation, CodeExamples, TutorialGenerator
};
pub use monitoring::{
    APIMonitoring, MonitoringConfig, RequestMetrics,
    ResponseMetrics, ErrorMetrics, PerformanceMetrics
};
pub use service::APIGatewayService;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main API Gateway service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIGatewayServiceConfig {
    /// REST API configuration
    pub rest_config: rest::RESTConfig,
    /// GraphQL configuration
    pub graphql_config: graphql::GraphQLConfig,
    /// WebSocket configuration
    pub websocket_config: websocket::WebSocketConfig,
    /// gRPC configuration
    pub grpc_config: grpc::GRPCConfig,
    /// Authentication configuration
    pub auth_config: auth::AuthConfig,
    /// Rate limiting configuration
    pub rate_limit_config: rate_limiting::RateLimitConfig,
    /// Load balancing configuration
    pub load_balancer_config: load_balancing::LoadBalancerConfig,
    /// Circuit breaker configuration
    pub circuit_breaker_config: circuit_breaker::CircuitBreakerConfig,
    /// SDK generation configuration
    pub sdk_config: sdk::SDKConfig,
    /// Documentation configuration
    pub doc_config: documentation::DocConfig,
    /// Global API gateway settings
    pub global_settings: GlobalAPIGatewaySettings,
}

impl Default for APIGatewayServiceConfig {
    fn default() -> Self {
        Self {
            rest_config: rest::RESTConfig::default(),
            graphql_config: graphql::GraphQLConfig::default(),
            websocket_config: websocket::WebSocketConfig::default(),
            grpc_config: grpc::GRPCConfig::default(),
            auth_config: auth::AuthConfig::default(),
            rate_limit_config: rate_limiting::RateLimitConfig::default(),
            load_balancer_config: load_balancing::LoadBalancerConfig::default(),
            circuit_breaker_config: circuit_breaker::CircuitBreakerConfig::default(),
            sdk_config: sdk::SDKConfig::default(),
            doc_config: documentation::DocConfig::default(),
            global_settings: GlobalAPIGatewaySettings::default(),
        }
    }
}

/// Global API gateway settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAPIGatewaySettings {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable HTTPS
    pub enable_https: bool,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable compression
    pub enable_compression: bool,
    /// Enable request logging
    pub enable_request_logging: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable distributed tracing
    pub enable_distributed_tracing: bool,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Maximum request body size in MB
    pub max_request_body_size_mb: u32,
    /// Enable API versioning
    pub enable_api_versioning: bool,
    /// Default API version
    pub default_api_version: String,
}

impl Default for GlobalAPIGatewaySettings {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            enable_https: true,
            enable_cors: true,
            enable_compression: true,
            enable_request_logging: true,
            enable_metrics: true,
            enable_distributed_tracing: true,
            request_timeout_seconds: 30,
            max_request_body_size_mb: 10,
            enable_api_versioning: true,
            default_api_version: "v1".to_string(),
        }
    }
}

/// API Gateway metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIGatewayMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub requests_per_second: f64,
    pub active_connections: u64,
    pub rate_limited_requests: u64,
    pub circuit_breaker_trips: u64,
    pub cache_hit_rate: Decimal,
    pub error_rate: Decimal,
    pub endpoint_metrics: HashMap<String, EndpointMetrics>,
    pub client_metrics: HashMap<String, ClientMetrics>,
    pub last_updated: DateTime<Utc>,
}

/// Endpoint-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub endpoint: String,
    pub method: String,
    pub requests_24h: u64,
    pub average_response_time_ms: f64,
    pub error_rate: Decimal,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
}

/// Client-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMetrics {
    pub client_id: String,
    pub requests_24h: u64,
    pub rate_limit_hits: u64,
    pub error_rate: Decimal,
    pub last_request: DateTime<Utc>,
}

/// API Gateway health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIGatewayHealthStatus {
    pub overall_status: String,
    pub rest_api_status: String,
    pub graphql_status: String,
    pub websocket_status: String,
    pub grpc_status: String,
    pub auth_service_status: String,
    pub rate_limiter_status: String,
    pub load_balancer_status: String,
    pub circuit_breaker_status: String,
    pub upstream_services: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
}

// Stub modules for compilation
pub mod rest {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RESTConfig {
        pub enable_openapi: bool,
        pub enable_swagger_ui: bool,
        pub api_prefix: String,
        pub enable_versioning: bool,
    }
    
    impl Default for RESTConfig {
        fn default() -> Self {
            Self {
                enable_openapi: true,
                enable_swagger_ui: true,
                api_prefix: "/api".to_string(),
                enable_versioning: true,
            }
        }
    }
    
    pub struct RESTService;
    pub struct RESTHandler;
    pub struct RESTRouter;
    pub struct OpenAPISpec;
    pub struct SwaggerUI;
    pub struct RESTMiddleware;
}

pub mod graphql {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GraphQLConfig {
        pub enable_playground: bool,
        pub enable_introspection: bool,
        pub enable_subscriptions: bool,
        pub max_query_depth: u32,
    }
    
    impl Default for GraphQLConfig {
        fn default() -> Self {
            Self {
                enable_playground: true,
                enable_introspection: true,
                enable_subscriptions: true,
                max_query_depth: 15,
            }
        }
    }
    
    pub struct GraphQLService;
    pub struct GraphQLSchema;
    pub struct GraphQLResolver;
    pub struct GraphQLSubscription;
    pub struct GraphQLPlayground;
}

pub mod websocket {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WebSocketConfig {
        pub max_connections: u32,
        pub heartbeat_interval_seconds: u64,
        pub enable_compression: bool,
        pub max_message_size_kb: u32,
    }
    
    impl Default for WebSocketConfig {
        fn default() -> Self {
            Self {
                max_connections: 10000,
                heartbeat_interval_seconds: 30,
                enable_compression: true,
                max_message_size_kb: 1024,
            }
        }
    }
    
    pub struct WebSocketService;
    pub struct WebSocketHandler;
    pub struct WebSocketConnection;
    pub struct WebSocketMessage;
    pub struct WebSocketBroadcast;
}

pub mod grpc {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GRPCConfig {
        pub enable_reflection: bool,
        pub enable_health_check: bool,
        pub max_message_size_mb: u32,
        pub keepalive_time_seconds: u64,
    }
    
    impl Default for GRPCConfig {
        fn default() -> Self {
            Self {
                enable_reflection: true,
                enable_health_check: true,
                max_message_size_mb: 4,
                keepalive_time_seconds: 60,
            }
        }
    }
    
    pub struct GRPCService;
    pub struct GRPCHandler;
    pub struct ProtocolBuffer;
    pub struct GRPCReflection;
    pub struct GRPCHealthCheck;
}

pub mod auth {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AuthConfig {
        pub enable_jwt: bool,
        pub enable_oauth2: bool,
        pub enable_api_key: bool,
        pub jwt_secret: String,
        pub token_expiry_hours: u32,
    }
    
    impl Default for AuthConfig {
        fn default() -> Self {
            Self {
                enable_jwt: true,
                enable_oauth2: true,
                enable_api_key: true,
                jwt_secret: "your-secret-key".to_string(),
                token_expiry_hours: 24,
            }
        }
    }
    
    pub struct AuthService;
    pub struct JWTAuth;
    pub struct OAuth2Auth;
    pub struct APIKeyAuth;
    pub struct BasicAuth;
    pub struct BearerAuth;
}

// Additional stub modules
pub mod rate_limiting {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RateLimitConfig {
        pub requests_per_minute: u32,
        pub burst_size: u32,
        pub enable_distributed: bool,
        pub window_size_seconds: u64,
    }
    
    impl Default for RateLimitConfig {
        fn default() -> Self {
            Self {
                requests_per_minute: 1000,
                burst_size: 100,
                enable_distributed: true,
                window_size_seconds: 60,
            }
        }
    }
    
    pub struct RateLimiter;
    pub struct TokenBucket;
    pub struct SlidingWindow;
    pub struct FixedWindow;
    pub struct DistributedRateLimit;
}

pub mod load_balancing {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoadBalancerConfig {
        pub algorithm: String,
        pub health_check_interval_seconds: u64,
        pub enable_sticky_sessions: bool,
        pub max_retries: u32,
    }
    
    impl Default for LoadBalancerConfig {
        fn default() -> Self {
            Self {
                algorithm: "round_robin".to_string(),
                health_check_interval_seconds: 30,
                enable_sticky_sessions: false,
                max_retries: 3,
            }
        }
    }
    
    pub struct LoadBalancer;
    pub struct RoundRobin;
    pub struct WeightedRoundRobin;
    pub struct LeastConnections;
    pub struct HealthBasedRouting;
}

pub mod circuit_breaker {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CircuitBreakerConfig {
        pub failure_threshold: u32,
        pub recovery_timeout_seconds: u64,
        pub half_open_max_calls: u32,
        pub enable_bulkhead: bool,
    }
    
    impl Default for CircuitBreakerConfig {
        fn default() -> Self {
            Self {
                failure_threshold: 5,
                recovery_timeout_seconds: 60,
                half_open_max_calls: 3,
                enable_bulkhead: true,
            }
        }
    }
    
    pub struct CircuitBreaker;
    pub struct CircuitState;
    pub struct FailureDetector;
    pub struct RecoveryStrategy;
    pub struct BulkheadPattern;
}

pub mod middleware {
    use super::*;
    
    pub struct MiddlewareStack;
    pub struct MiddlewareConfig;
    pub struct RequestMiddleware;
    pub struct ResponseMiddleware;
    pub struct ErrorMiddleware;
    pub struct LoggingMiddleware;
}

pub mod sdk {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SDKConfig {
        pub enable_typescript: bool,
        pub enable_python: bool,
        pub enable_java: bool,
        pub enable_go: bool,
        pub enable_rust: bool,
        pub enable_php: bool,
    }
    
    impl Default for SDKConfig {
        fn default() -> Self {
            Self {
                enable_typescript: true,
                enable_python: true,
                enable_java: true,
                enable_go: true,
                enable_rust: true,
                enable_php: true,
            }
        }
    }
    
    pub struct SDKGenerator;
    pub struct TypeScriptSDK;
    pub struct PythonSDK;
    pub struct JavaSDK;
    pub struct GoSDK;
    pub struct RustSDK;
    pub struct PHPSDK;
}

pub mod documentation {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DocConfig {
        pub enable_interactive_docs: bool,
        pub enable_code_examples: bool,
        pub enable_tutorials: bool,
        pub auto_generate: bool,
    }
    
    impl Default for DocConfig {
        fn default() -> Self {
            Self {
                enable_interactive_docs: true,
                enable_code_examples: true,
                enable_tutorials: true,
                auto_generate: true,
            }
        }
    }
    
    pub struct DocumentationGenerator;
    pub struct APIDocumentation;
    pub struct InteractiveDocumentation;
    pub struct CodeExamples;
    pub struct TutorialGenerator;
}

pub mod monitoring {
    use super::*;
    
    pub struct APIMonitoring;
    pub struct MonitoringConfig;
    pub struct RequestMetrics;
    pub struct ResponseMetrics;
    pub struct ErrorMetrics;
    pub struct PerformanceMetrics;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_gateway_config_default() {
        let config = APIGatewayServiceConfig::default();
        assert!(config.rest_config.enable_openapi);
        assert!(config.rest_config.enable_swagger_ui);
        assert!(config.graphql_config.enable_playground);
        assert!(config.websocket_config.enable_compression);
        assert!(config.auth_config.enable_jwt);
        assert_eq!(config.rate_limit_config.requests_per_minute, 1000);
    }

    #[test]
    fn test_global_api_gateway_settings() {
        let settings = GlobalAPIGatewaySettings::default();
        assert_eq!(settings.host, "0.0.0.0");
        assert_eq!(settings.port, 8080);
        assert!(settings.enable_https);
        assert!(settings.enable_cors);
        assert!(settings.enable_compression);
        assert_eq!(settings.request_timeout_seconds, 30);
        assert_eq!(settings.default_api_version, "v1");
    }

    #[test]
    fn test_rest_config() {
        let config = rest::RESTConfig::default();
        assert!(config.enable_openapi);
        assert!(config.enable_swagger_ui);
        assert_eq!(config.api_prefix, "/api");
        assert!(config.enable_versioning);
    }

    #[test]
    fn test_rate_limit_config() {
        let config = rate_limiting::RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 1000);
        assert_eq!(config.burst_size, 100);
        assert!(config.enable_distributed);
        assert_eq!(config.window_size_seconds, 60);
    }
}
