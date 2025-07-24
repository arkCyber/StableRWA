// =====================================================================================
// RWA Tokenization Platform - Oracle Service Error Types
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{HttpResponse, ResponseError};
// Removed unused import: std::fmt
use thiserror::Error;

/// Oracle service error types
#[derive(Error, Debug)]
pub enum OracleError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Provider error: {provider} - {message}")]
    Provider { provider: String, message: String },

    #[error("Price not found for asset: {asset_id}")]
    PriceNotFound { asset_id: String },

    #[error("Invalid price data: {reason}")]
    InvalidPriceData { reason: String },

    #[error("Aggregation failed: {reason}")]
    AggregationFailed { reason: String },

    #[error("Feed not found: {feed_id}")]
    FeedNotFound { feed_id: String },

    #[error("Subscription not found: {subscription_id}")]
    SubscriptionNotFound { subscription_id: String },

    #[error("Rate limit exceeded for provider: {provider}")]
    RateLimitExceeded { provider: String },

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Timeout error: {operation}")]
    Timeout { operation: String },

    #[error("Insufficient data sources: required {required}, available {available}")]
    InsufficientDataSources { required: usize, available: usize },

    #[error("Price deviation too high: {deviation}% exceeds threshold {threshold}%")]
    PriceDeviationTooHigh { deviation: f64, threshold: f64 },

    #[error("Asset not supported: {asset_id} by provider {provider}")]
    AssetNotSupported { asset_id: String, provider: String },

    #[error("Asset not found: {asset_id} in provider {provider}")]
    AssetNotFound { asset_id: String, provider: String },

    #[error("Currency not supported: {currency} by provider {provider}")]
    CurrencyNotSupported { currency: String, provider: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Parsing error: {message}")]
    Parsing { message: String },


}

impl ResponseError for OracleError {
    fn error_response(&self) -> HttpResponse {
        match self {
            OracleError::PriceNotFound { .. } => {
                HttpResponse::NotFound().json(ErrorResponse::new("price_not_found", &self.to_string()))
            }
            OracleError::FeedNotFound { .. } => {
                HttpResponse::NotFound().json(ErrorResponse::new("feed_not_found", &self.to_string()))
            }
            OracleError::SubscriptionNotFound { .. } => {
                HttpResponse::NotFound().json(ErrorResponse::new("subscription_not_found", &self.to_string()))
            }
            OracleError::Validation { .. } => {
                HttpResponse::BadRequest().json(ErrorResponse::new("validation_error", &self.to_string()))
            }
            OracleError::Authentication(_) => {
                HttpResponse::Unauthorized().json(ErrorResponse::new("authentication_error", &self.to_string()))
            }
            OracleError::Authorization(_) => {
                HttpResponse::Forbidden().json(ErrorResponse::new("authorization_error", &self.to_string()))
            }
            OracleError::RateLimitExceeded { .. } => {
                HttpResponse::TooManyRequests().json(ErrorResponse::new("rate_limit_exceeded", &self.to_string()))
            }
            OracleError::ServiceUnavailable(_) => {
                HttpResponse::ServiceUnavailable().json(ErrorResponse::new("service_unavailable", &self.to_string()))
            }
            OracleError::AssetNotSupported { .. } => {
                HttpResponse::BadRequest().json(ErrorResponse::new("asset_not_supported", &self.to_string()))
            }
            OracleError::AssetNotFound { .. } => {
                HttpResponse::NotFound().json(ErrorResponse::new("asset_not_found", &self.to_string()))
            }
            OracleError::CurrencyNotSupported { .. } => {
                HttpResponse::BadRequest().json(ErrorResponse::new("currency_not_supported", &self.to_string()))
            }
            OracleError::Network { .. } => {
                HttpResponse::BadGateway().json(ErrorResponse::new("network_error", &self.to_string()))
            }
            OracleError::Parsing { .. } => {
                HttpResponse::UnprocessableEntity().json(ErrorResponse::new("parsing_error", &self.to_string()))
            }
            OracleError::Configuration { .. } => {
                HttpResponse::InternalServerError().json(ErrorResponse::new("configuration_error", &self.to_string()))
            }
            OracleError::Timeout { .. } => {
                HttpResponse::RequestTimeout().json(ErrorResponse::new("timeout", &self.to_string()))
            }
            _ => {
                HttpResponse::InternalServerError().json(ErrorResponse::new("internal_error", "An internal error occurred"))
            }
        }
    }
}

/// Standardized error response format
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorResponse {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Oracle service result type
pub type OracleResult<T> = Result<T, OracleError>;

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[test]
    fn test_oracle_error_display() {
        let error = OracleError::PriceNotFound {
            asset_id: "BTC".to_string(),
        };
        assert_eq!(error.to_string(), "Price not found for asset: BTC");
    }

    #[test]
    fn test_oracle_error_from_sqlx() {
        let sqlx_error = sqlx::Error::RowNotFound;
        let oracle_error = OracleError::from(sqlx_error);
        assert!(matches!(oracle_error, OracleError::Database(_)));
    }

    #[test]
    fn test_oracle_error_from_redis() {
        let redis_error = redis::RedisError::from((redis::ErrorKind::TypeError, "test error"));
        let oracle_error = OracleError::from(redis_error);
        assert!(matches!(oracle_error, OracleError::Redis(_)));
    }

    #[actix_web::test]
    async fn test_error_response_format() {
        let error = OracleError::PriceNotFound {
            asset_id: "ETH".to_string(),
        };

        let response = error.error_response();
        assert_eq!(response.status(), 404);

        let body = test::read_body(response).await;
        let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response.error, "price_not_found");
        assert!(error_response.message.contains("ETH"));
    }

    #[test]
    fn test_validation_error() {
        let error = OracleError::Validation {
            field: "asset_id".to_string(),
            message: "cannot be empty".to_string(),
        };

        assert_eq!(error.to_string(), "Validation error: asset_id - cannot be empty");
    }

    #[test]
    fn test_provider_error() {
        let error = OracleError::Provider {
            provider: "CoinGecko".to_string(),
            message: "API key invalid".to_string(),
        };

        assert_eq!(error.to_string(), "Provider error: CoinGecko - API key invalid");
    }

    #[test]
    fn test_price_deviation_error() {
        let error = OracleError::PriceDeviationTooHigh {
            deviation: 15.5,
            threshold: 10.0,
        };

        assert_eq!(error.to_string(), "Price deviation too high: 15.5% exceeds threshold 10%");
    }

    #[test]
    fn test_insufficient_data_sources_error() {
        let error = OracleError::InsufficientDataSources {
            required: 3,
            available: 1,
        };

        assert_eq!(error.to_string(), "Insufficient data sources: required 3, available 1");
    }

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse::new("test_error", "Test message");

        assert_eq!(response.error, "test_error");
        assert_eq!(response.message, "Test message");
        assert!(response.timestamp <= chrono::Utc::now());
    }

    #[actix_web::test]
    async fn test_authentication_error_response() {
        let error = OracleError::Authentication("Invalid token".to_string());
        let response = error.error_response();

        assert_eq!(response.status(), 401);

        let body = test::read_body(response).await;
        let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response.error, "authentication_error");
    }

    #[actix_web::test]
    async fn test_rate_limit_error_response() {
        let error = OracleError::RateLimitExceeded {
            provider: "Binance".to_string(),
        };
        let response = error.error_response();

        assert_eq!(response.status(), 429);

        let body = test::read_body(response).await;
        let error_response: ErrorResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(error_response.error, "rate_limit_exceeded");
    }
}
