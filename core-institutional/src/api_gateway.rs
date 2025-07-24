// =====================================================================================
// File: core-institutional/src/api_gateway.rs
// Description: API Gateway service for institutional clients
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{InstitutionalError, InstitutionalResult},
    types::{ApiKey, Permission},
};

/// API Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGatewayConfig {
    /// Default rate limit per minute
    pub default_rate_limit_per_minute: u32,
    /// Maximum rate limit per minute
    pub max_rate_limit_per_minute: u32,
    /// Request timeout in seconds
    pub request_timeout_seconds: u64,
    /// Maximum request size in MB
    pub max_request_size_mb: u32,
    /// Maximum response size in MB
    pub max_response_size_mb: u32,
    /// Enable request/response logging
    pub enable_logging: bool,
    /// Enable request caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable API versioning
    pub enable_versioning: bool,
    /// Supported API versions
    pub supported_versions: Vec<String>,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable compression
    pub enable_compression: bool,
}

impl Default for ApiGatewayConfig {
    fn default() -> Self {
        Self {
            default_rate_limit_per_minute: 1000,
            max_rate_limit_per_minute: 10000,
            request_timeout_seconds: 30,
            max_request_size_mb: 10,
            max_response_size_mb: 50,
            enable_logging: true,
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            enable_versioning: true,
            supported_versions: vec!["v1".to_string(), "v2".to_string()],
            enable_cors: true,
            enable_compression: true,
        }
    }
}

/// API endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub id: Uuid,
    pub path: String,
    pub method: HttpMethod,
    pub description: String,
    pub required_permissions: Vec<Permission>,
    pub rate_limit_config: Option<RateLimitConfig>,
    pub authentication_config: AuthenticationConfig,
    pub caching_config: Option<CachingConfig>,
    pub validation_config: Option<ValidationConfig>,
    pub transformation_config: Option<TransformationConfig>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// HTTP method enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_limit: u32,
    pub rate_limit_by: RateLimitBy,
    pub custom_headers: HashMap<String, String>,
}

/// Rate limit criteria
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitBy {
    ApiKey,
    IpAddress,
    UserId,
    InstitutionId,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    pub auth_type: AuthenticationType,
    pub required: bool,
    pub token_validation: TokenValidation,
    pub session_config: Option<SessionConfig>,
}

/// Authentication type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthenticationType {
    ApiKey,
    Bearer,
    Basic,
    OAuth2,
    JWT,
    Custom,
}

/// Token validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidation {
    pub validate_expiry: bool,
    pub validate_signature: bool,
    pub validate_issuer: bool,
    pub validate_audience: bool,
    pub allowed_issuers: Vec<String>,
    pub allowed_audiences: Vec<String>,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout_minutes: u32,
    pub sliding_expiration: bool,
    pub require_https: bool,
    pub same_site_policy: SameSitePolicy,
}

/// SameSite policy for cookies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}

/// Caching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
    pub cache_key_strategy: CacheKeyStrategy,
    pub cache_conditions: Vec<CacheCondition>,
    pub vary_headers: Vec<String>,
}

/// Cache key strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheKeyStrategy {
    FullUrl,
    PathOnly,
    PathAndQuery,
    Custom,
}

/// Cache condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheCondition {
    pub condition_type: CacheConditionType,
    pub value: String,
    pub operator: ComparisonOperator,
}

/// Cache condition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheConditionType {
    Header,
    QueryParam,
    StatusCode,
    ResponseSize,
}

/// Comparison operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
}

/// Request validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub validate_headers: bool,
    pub validate_query_params: bool,
    pub validate_body: bool,
    pub schema_validation: Option<SchemaValidation>,
    pub custom_validators: Vec<CustomValidator>,
}

/// Schema validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidation {
    pub schema_type: SchemaType,
    pub schema_content: String,
    pub strict_mode: bool,
}

/// Schema type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaType {
    JsonSchema,
    OpenApi,
    GraphQL,
    Custom,
}

/// Custom validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomValidator {
    pub name: String,
    pub validator_type: ValidatorType,
    pub configuration: serde_json::Value,
}

/// Validator type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorType {
    Regex,
    Range,
    Length,
    Format,
    Custom,
}

/// Request/Response transformation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationConfig {
    pub request_transformations: Vec<Transformation>,
    pub response_transformations: Vec<Transformation>,
}

/// Transformation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    pub transformation_type: TransformationType,
    pub source_path: String,
    pub target_path: String,
    pub transformation_rule: String,
}

/// Transformation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformationType {
    Map,
    Filter,
    Aggregate,
    Format,
    Rename,
    Remove,
    Add,
}

/// API request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub institution_id: Uuid,
    pub user_id: Option<Uuid>,
    pub api_key_id: Option<Uuid>,
    pub method: HttpMethod,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
}

/// API response information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub request_id: Uuid,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub response_time_ms: u64,
    pub cache_hit: bool,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// API metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetrics {
    pub institution_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub bandwidth_usage_mb: f64,
    pub top_endpoints: Vec<EndpointMetrics>,
    pub error_breakdown: HashMap<String, u64>,
}

/// Endpoint-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub endpoint_id: Uuid,
    pub path: String,
    pub method: HttpMethod,
    pub request_count: u64,
    pub average_response_time_ms: f64,
    pub error_count: u64,
    pub error_rate: f64,
}

/// API Gateway service trait
#[async_trait]
pub trait ApiGatewayService: Send + Sync {
    /// Register a new API endpoint
    async fn register_endpoint(&self, endpoint: ApiEndpoint) -> InstitutionalResult<ApiEndpoint>;

    /// Get endpoint by ID
    async fn get_endpoint(&self, endpoint_id: Uuid) -> InstitutionalResult<Option<ApiEndpoint>>;

    /// Get all endpoints for an institution
    async fn get_institution_endpoints(
        &self,
        institution_id: Uuid,
    ) -> InstitutionalResult<Vec<ApiEndpoint>>;

    /// Update endpoint configuration
    async fn update_endpoint(&self, endpoint: ApiEndpoint) -> InstitutionalResult<ApiEndpoint>;

    /// Delete endpoint
    async fn delete_endpoint(&self, endpoint_id: Uuid) -> InstitutionalResult<()>;

    /// Process API request
    async fn process_request(&self, request: ApiRequest) -> InstitutionalResult<ApiResponse>;

    /// Get API metrics
    async fn get_api_metrics(
        &self,
        institution_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> InstitutionalResult<ApiMetrics>;

    /// Get request logs
    async fn get_request_logs(
        &self,
        institution_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> InstitutionalResult<Vec<ApiRequest>>;

    /// Validate API key
    async fn validate_api_key(&self, api_key: &str) -> InstitutionalResult<Option<ApiKey>>;

    /// Health check
    async fn health_check(&self) -> InstitutionalResult<ApiGatewayHealthStatus>;
}

/// API Gateway health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGatewayHealthStatus {
    pub status: String,
    pub total_endpoints: u64,
    pub active_endpoints: u64,
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub rate_limit_violations_24h: u64,
    pub authentication_failures_24h: u64,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_gateway_config_default() {
        let config = ApiGatewayConfig::default();
        assert_eq!(config.default_rate_limit_per_minute, 1000);
        assert_eq!(config.request_timeout_seconds, 30);
        assert!(config.enable_logging);
        assert!(config.enable_caching);
        assert!(config.enable_versioning);
    }

    #[test]
    fn test_api_endpoint_creation() {
        let endpoint = ApiEndpoint {
            id: Uuid::new_v4(),
            path: "/api/v1/accounts".to_string(),
            method: HttpMethod::GET,
            description: "Get account information".to_string(),
            required_permissions: vec![Permission::ViewBalances],
            rate_limit_config: Some(RateLimitConfig {
                requests_per_minute: 100,
                requests_per_hour: 1000,
                requests_per_day: 10000,
                burst_limit: 10,
                rate_limit_by: RateLimitBy::ApiKey,
                custom_headers: HashMap::new(),
            }),
            authentication_config: AuthenticationConfig {
                auth_type: AuthenticationType::ApiKey,
                required: true,
                token_validation: TokenValidation {
                    validate_expiry: true,
                    validate_signature: true,
                    validate_issuer: false,
                    validate_audience: false,
                    allowed_issuers: vec![],
                    allowed_audiences: vec![],
                },
                session_config: None,
            },
            caching_config: None,
            validation_config: None,
            transformation_config: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(endpoint.method, HttpMethod::GET);
        assert_eq!(endpoint.path, "/api/v1/accounts");
        assert!(endpoint.is_active);
        assert!(endpoint.authentication_config.required);
    }

    #[test]
    fn test_rate_limit_config() {
        let rate_limit = RateLimitConfig {
            requests_per_minute: 100,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_limit: 10,
            rate_limit_by: RateLimitBy::ApiKey,
            custom_headers: HashMap::new(),
        };

        assert_eq!(rate_limit.requests_per_minute, 100);
        assert_eq!(rate_limit.rate_limit_by, RateLimitBy::ApiKey);
        assert_eq!(rate_limit.burst_limit, 10);
    }
}
