// =====================================================================================
// File: core-oracle/src/error.rs
// Description: Error types for oracle and price data operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for oracle operations
pub type OracleResult<T> = Result<T, OracleError>;

/// Oracle service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum OracleError {
    /// Price feed errors
    #[error("Price feed error: {message}")]
    PriceFeedError { message: String },

    /// Oracle provider errors
    #[error("Oracle provider error: {provider}: {message}")]
    ProviderError { provider: String, message: String },

    /// Data validation errors
    #[error("Data validation error: {field}: {message}")]
    ValidationError { field: String, message: String },

    /// Aggregation errors
    #[error("Aggregation error: {message}")]
    AggregationError { message: String },

    /// Network errors
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Authentication errors
    #[error("Authentication error: {provider}: {message}")]
    AuthenticationError { provider: String, message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {provider}")]
    RateLimitExceeded { provider: String },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Stale data errors
    #[error(
        "Stale data detected: {feed}: age {age_seconds}s exceeds threshold {threshold_seconds}s"
    )]
    StaleData {
        feed: String,
        age_seconds: u64,
        threshold_seconds: u64,
    },

    /// Price deviation errors
    #[error(
        "Price deviation detected: {feed}: deviation {deviation}% exceeds threshold {threshold}%"
    )]
    PriceDeviation {
        feed: String,
        deviation: f64,
        threshold: f64,
    },

    /// Circuit breaker errors
    #[error("Circuit breaker triggered: {feed}: {reason}")]
    CircuitBreakerTriggered { feed: String, reason: String },

    /// Insufficient data errors
    #[error("Insufficient data: {feed}: required {required}, available {available}")]
    InsufficientData {
        feed: String,
        required: u32,
        available: u32,
    },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Database errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// External API errors
    #[error("External API error: {api}: {status}: {message}")]
    ExternalApiError {
        api: String,
        status: u16,
        message: String,
    },

    /// Subscription errors
    #[error("Subscription error: {feed}: {message}")]
    SubscriptionError { feed: String, message: String },

    /// Feed not found errors
    #[error("Feed not found: {feed_id}")]
    FeedNotFound { feed_id: String },

    /// Invalid price errors
    #[error("Invalid price: {feed}: {price}: {reason}")]
    InvalidPrice {
        feed: String,
        price: String,
        reason: String,
    },

    /// Consensus errors
    #[error("Consensus error: {message}")]
    ConsensusError { message: String },

    /// Internal errors
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl OracleError {
    /// Create a price feed error
    pub fn price_feed_error<S: Into<String>>(message: S) -> Self {
        Self::PriceFeedError {
            message: message.into(),
        }
    }

    /// Create a provider error
    pub fn provider_error<S: Into<String>>(provider: S, message: S) -> Self {
        Self::ProviderError {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an aggregation error
    pub fn aggregation_error<S: Into<String>>(message: S) -> Self {
        Self::AggregationError {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication_error<S: Into<String>>(provider: S, message: S) -> Self {
        Self::AuthenticationError {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded<S: Into<String>>(provider: S) -> Self {
        Self::RateLimitExceeded {
            provider: provider.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a stale data error
    pub fn stale_data<S: Into<String>>(feed: S, age_seconds: u64, threshold_seconds: u64) -> Self {
        Self::StaleData {
            feed: feed.into(),
            age_seconds,
            threshold_seconds,
        }
    }

    /// Create a price deviation error
    pub fn price_deviation<S: Into<String>>(feed: S, deviation: f64, threshold: f64) -> Self {
        Self::PriceDeviation {
            feed: feed.into(),
            deviation,
            threshold,
        }
    }

    /// Create a circuit breaker triggered error
    pub fn circuit_breaker_triggered<S: Into<String>>(feed: S, reason: S) -> Self {
        Self::CircuitBreakerTriggered {
            feed: feed.into(),
            reason: reason.into(),
        }
    }

    /// Create an insufficient data error
    pub fn insufficient_data<S: Into<String>>(feed: S, required: u32, available: u32) -> Self {
        Self::InsufficientData {
            feed: feed.into(),
            required,
            available,
        }
    }

    /// Create a serialization error
    pub fn serialization_error<S: Into<String>>(message: S) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create a database error
    pub fn database_error<S: Into<String>>(message: S) -> Self {
        Self::DatabaseError {
            message: message.into(),
        }
    }

    /// Create an external API error
    pub fn external_api_error<S: Into<String>>(api: S, status: u16, message: S) -> Self {
        Self::ExternalApiError {
            api: api.into(),
            status,
            message: message.into(),
        }
    }

    /// Create a subscription error
    pub fn subscription_error<S: Into<String>>(feed: S, message: S) -> Self {
        Self::SubscriptionError {
            feed: feed.into(),
            message: message.into(),
        }
    }

    /// Create a feed not found error
    pub fn feed_not_found<S: Into<String>>(feed_id: S) -> Self {
        Self::FeedNotFound {
            feed_id: feed_id.into(),
        }
    }

    /// Create an invalid price error
    pub fn invalid_price<S: Into<String>>(feed: S, price: S, reason: S) -> Self {
        Self::InvalidPrice {
            feed: feed.into(),
            price: price.into(),
            reason: reason.into(),
        }
    }

    /// Create a consensus error
    pub fn consensus_error<S: Into<String>>(message: S) -> Self {
        Self::ConsensusError {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Get error code for categorization
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::PriceFeedError { .. } => "PRICE_FEED_ERROR",
            Self::ProviderError { .. } => "PROVIDER_ERROR",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::AggregationError { .. } => "AGGREGATION_ERROR",
            Self::NetworkError { .. } => "NETWORK_ERROR",
            Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            Self::AuthenticationError { .. } => "AUTHENTICATION_ERROR",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            Self::Timeout { .. } => "TIMEOUT",
            Self::StaleData { .. } => "STALE_DATA",
            Self::PriceDeviation { .. } => "PRICE_DEVIATION",
            Self::CircuitBreakerTriggered { .. } => "CIRCUIT_BREAKER_TRIGGERED",
            Self::InsufficientData { .. } => "INSUFFICIENT_DATA",
            Self::SerializationError { .. } => "SERIALIZATION_ERROR",
            Self::DatabaseError { .. } => "DATABASE_ERROR",
            Self::ExternalApiError { .. } => "EXTERNAL_API_ERROR",
            Self::SubscriptionError { .. } => "SUBSCRIPTION_ERROR",
            Self::FeedNotFound { .. } => "FEED_NOT_FOUND",
            Self::InvalidPrice { .. } => "INVALID_PRICE",
            Self::ConsensusError { .. } => "CONSENSUS_ERROR",
            Self::InternalError { .. } => "INTERNAL_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError { .. }
                | Self::Timeout { .. }
                | Self::RateLimitExceeded { .. }
                | Self::ExternalApiError { .. }
                | Self::DatabaseError { .. }
        )
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::CircuitBreakerTriggered { .. }
                | Self::PriceDeviation { .. }
                | Self::StaleData { .. }
                | Self::InternalError { .. }
        )
    }

    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(
            self,
            Self::CircuitBreakerTriggered { .. }
                | Self::PriceDeviation { .. }
                | Self::StaleData { .. }
                | Self::ConsensusError { .. }
        )
    }

    /// Check if error is data quality related
    pub fn is_data_quality_issue(&self) -> bool {
        matches!(
            self,
            Self::StaleData { .. }
                | Self::PriceDeviation { .. }
                | Self::InvalidPrice { .. }
                | Self::InsufficientData { .. }
                | Self::ValidationError { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = OracleError::price_feed_error("Test price feed error");
        assert_eq!(error.error_code(), "PRICE_FEED_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_data_quality_issue());
    }

    #[test]
    fn test_stale_data_error() {
        let error = OracleError::stale_data("ETH/USD", 300, 120);
        assert_eq!(error.error_code(), "STALE_DATA");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(error.is_data_quality_issue());
    }

    #[test]
    fn test_price_deviation_error() {
        let error = OracleError::price_deviation("BTC/USD", 15.5, 10.0);
        assert_eq!(error.error_code(), "PRICE_DEVIATION");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(error.is_data_quality_issue());
    }

    #[test]
    fn test_network_error() {
        let error = OracleError::network_error("Connection failed");
        assert_eq!(error.error_code(), "NETWORK_ERROR");
        assert!(error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_data_quality_issue());
    }

    #[test]
    fn test_circuit_breaker_error() {
        let error = OracleError::circuit_breaker_triggered("ETH/USD", "Price volatility too high");
        assert_eq!(error.error_code(), "CIRCUIT_BREAKER_TRIGGERED");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(!error.is_data_quality_issue());
    }

    #[test]
    fn test_validation_error() {
        let error = OracleError::validation_error("price", "Price cannot be negative");
        assert_eq!(error.error_code(), "VALIDATION_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(error.is_data_quality_issue());
    }
}
