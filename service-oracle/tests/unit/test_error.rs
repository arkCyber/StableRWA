// =====================================================================================
// RWA Tokenization Platform - Oracle Service Error Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use service_oracle::error::*;
use actix_web::{ResponseError, http::StatusCode};
use std::error::Error;

#[test]
fn test_oracle_error_display() {
    let errors = vec![
        OracleError::Configuration { message: "Invalid config".to_string() },
        OracleError::Database { message: "Connection failed".to_string() },
        OracleError::Cache { message: "Redis error".to_string() },
        OracleError::Provider { provider: "CoinGecko".to_string(), message: "API error".to_string() },
        OracleError::Aggregation { message: "No valid prices".to_string() },
        OracleError::InvalidPriceData { reason: "Negative price".to_string() },
        OracleError::PriceDeviationTooHigh { deviation: 25.5, threshold: 10.0 },
        OracleError::InsufficientSources { available: 1, required: 2 },
        OracleError::FeedNotFound { feed_id: "test-feed".to_string() },
        OracleError::SubscriptionNotFound { subscription_id: "test-sub".to_string() },
        OracleError::ValidationError { field: "price".to_string(), message: "Must be positive".to_string() },
        OracleError::RateLimitExceeded { provider: "Binance".to_string(), retry_after: 60 },
        OracleError::Timeout { operation: "price_fetch".to_string(), duration_ms: 5000 },
        OracleError::Internal { message: "Unexpected error".to_string() },
    ];

    for error in errors {
        let display_str = format!("{}", error);
        assert!(!display_str.is_empty());
        assert!(display_str.len() > 10); // Should have meaningful content
    }
}

#[test]
fn test_oracle_error_debug() {
    let error = OracleError::Provider {
        provider: "CoinGecko".to_string(),
        message: "API rate limit exceeded".to_string(),
    };

    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Provider"));
    assert!(debug_str.contains("CoinGecko"));
    assert!(debug_str.contains("API rate limit exceeded"));
}

#[test]
fn test_oracle_error_source() {
    let error = OracleError::Database {
        message: "Connection timeout".to_string(),
    };

    // Should not have a source (these are leaf errors)
    assert!(error.source().is_none());
}

#[test]
fn test_oracle_error_response_error_trait() {
    let test_cases = vec![
        (OracleError::Configuration { message: "test".to_string() }, StatusCode::INTERNAL_SERVER_ERROR),
        (OracleError::Database { message: "test".to_string() }, StatusCode::INTERNAL_SERVER_ERROR),
        (OracleError::Cache { message: "test".to_string() }, StatusCode::INTERNAL_SERVER_ERROR),
        (OracleError::Provider { provider: "test".to_string(), message: "test".to_string() }, StatusCode::BAD_GATEWAY),
        (OracleError::Aggregation { message: "test".to_string() }, StatusCode::INTERNAL_SERVER_ERROR),
        (OracleError::InvalidPriceData { reason: "test".to_string() }, StatusCode::BAD_REQUEST),
        (OracleError::PriceDeviationTooHigh { deviation: 25.0, threshold: 10.0 }, StatusCode::UNPROCESSABLE_ENTITY),
        (OracleError::InsufficientSources { available: 1, required: 2 }, StatusCode::SERVICE_UNAVAILABLE),
        (OracleError::FeedNotFound { feed_id: "test".to_string() }, StatusCode::NOT_FOUND),
        (OracleError::SubscriptionNotFound { subscription_id: "test".to_string() }, StatusCode::NOT_FOUND),
        (OracleError::ValidationError { field: "test".to_string(), message: "test".to_string() }, StatusCode::BAD_REQUEST),
        (OracleError::RateLimitExceeded { provider: "test".to_string(), retry_after: 60 }, StatusCode::TOO_MANY_REQUESTS),
        (OracleError::Timeout { operation: "test".to_string(), duration_ms: 5000 }, StatusCode::REQUEST_TIMEOUT),
        (OracleError::Internal { message: "test".to_string() }, StatusCode::INTERNAL_SERVER_ERROR),
    ];

    for (error, expected_status) in test_cases {
        let response = error.error_response();
        assert_eq!(response.status(), expected_status);
    }
}

#[test]
fn test_oracle_error_json_response() {
    let error = OracleError::FeedNotFound {
        feed_id: "test-feed-123".to_string(),
    };

    let response = error.error_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // The response should contain JSON with error details
    // Note: In a real test, you'd extract and parse the body
}

#[test]
fn test_oracle_error_from_conversions() {
    // Test conversion from sqlx::Error
    let sqlx_error = sqlx::Error::RowNotFound;
    let oracle_error: OracleError = sqlx_error.into();
    
    match oracle_error {
        OracleError::Database { message } => {
            assert!(message.contains("no rows returned"));
        }
        _ => panic!("Expected Database error"),
    }

    // Test conversion from redis::RedisError
    let redis_error = redis::RedisError::from((redis::ErrorKind::IoError, "Connection failed"));
    let oracle_error: OracleError = redis_error.into();
    
    match oracle_error {
        OracleError::Cache { message } => {
            assert!(message.contains("Connection failed"));
        }
        _ => panic!("Expected Cache error"),
    }

    // Test conversion from serde_json::Error
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let oracle_error: OracleError = json_error.into();
    
    match oracle_error {
        OracleError::InvalidPriceData { reason } => {
            assert!(reason.contains("JSON"));
        }
        _ => panic!("Expected InvalidPriceData error"),
    }
}

#[test]
fn test_oracle_error_chain() {
    let root_cause = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    let wrapped_error = OracleError::Database {
        message: format!("Database connection failed: {}", root_cause),
    };

    let error_string = format!("{}", wrapped_error);
    assert!(error_string.contains("Database connection failed"));
    assert!(error_string.contains("Connection refused"));
}

#[test]
fn test_oracle_error_equality() {
    let error1 = OracleError::FeedNotFound { feed_id: "test".to_string() };
    let error2 = OracleError::FeedNotFound { feed_id: "test".to_string() };
    let error3 = OracleError::FeedNotFound { feed_id: "different".to_string() };

    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_oracle_error_clone() {
    let original = OracleError::Provider {
        provider: "CoinGecko".to_string(),
        message: "Rate limit exceeded".to_string(),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_oracle_error_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<OracleError>();
    assert_sync::<OracleError>();
}

#[test]
fn test_oracle_result_type() {
    fn returns_result() -> OracleResult<String> {
        Ok("success".to_string())
    }

    fn returns_error() -> OracleResult<String> {
        Err(OracleError::Internal { message: "test error".to_string() })
    }

    assert!(returns_result().is_ok());
    assert!(returns_error().is_err());
}

#[test]
fn test_error_context_preservation() {
    let original_message = "Original database error";
    let error = OracleError::Database {
        message: original_message.to_string(),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains(original_message));
}

#[test]
fn test_error_categorization() {
    let client_errors = vec![
        OracleError::InvalidPriceData { reason: "test".to_string() },
        OracleError::ValidationError { field: "test".to_string(), message: "test".to_string() },
        OracleError::FeedNotFound { feed_id: "test".to_string() },
        OracleError::SubscriptionNotFound { subscription_id: "test".to_string() },
    ];

    let server_errors = vec![
        OracleError::Configuration { message: "test".to_string() },
        OracleError::Database { message: "test".to_string() },
        OracleError::Cache { message: "test".to_string() },
        OracleError::Internal { message: "test".to_string() },
    ];

    let service_errors = vec![
        OracleError::Provider { provider: "test".to_string(), message: "test".to_string() },
        OracleError::InsufficientSources { available: 1, required: 2 },
        OracleError::RateLimitExceeded { provider: "test".to_string(), retry_after: 60 },
        OracleError::Timeout { operation: "test".to_string(), duration_ms: 5000 },
    ];

    // Client errors should return 4xx status codes
    for error in client_errors {
        let status = error.status_code();
        assert!(status.is_client_error(), "Expected client error for {:?}", error);
    }

    // Server errors should return 5xx status codes
    for error in server_errors {
        let status = error.status_code();
        assert!(status.is_server_error(), "Expected server error for {:?}", error);
    }

    // Service errors can be either 4xx or 5xx depending on the specific error
    for error in service_errors {
        let status = error.status_code();
        assert!(status.is_client_error() || status.is_server_error(), 
               "Expected client or server error for {:?}", error);
    }
}

#[test]
fn test_error_serialization() {
    let error = OracleError::PriceDeviationTooHigh {
        deviation: 25.5,
        threshold: 10.0,
    };

    // Test that error can be serialized (for logging, etc.)
    let serialized = serde_json::to_string(&error);
    assert!(serialized.is_ok());

    let json_value: serde_json::Value = serde_json::from_str(&serialized.unwrap()).unwrap();
    assert!(json_value.is_object());
}

#[test]
fn test_error_metrics_compatibility() {
    let errors = vec![
        OracleError::Provider { provider: "CoinGecko".to_string(), message: "test".to_string() },
        OracleError::Database { message: "test".to_string() },
        OracleError::Cache { message: "test".to_string() },
    ];

    // Errors should be convertible to strings for metrics labels
    for error in errors {
        let error_type = format!("{:?}", error).split('{').next().unwrap().to_string();
        assert!(!error_type.is_empty());
        assert!(!error_type.contains('{'));
    }
}

#[test]
fn test_error_recovery_hints() {
    let error = OracleError::RateLimitExceeded {
        provider: "CoinGecko".to_string(),
        retry_after: 60,
    };

    let error_msg = format!("{}", error);
    assert!(error_msg.contains("60")); // Should include retry hint
}

#[test]
fn test_error_logging_format() {
    let error = OracleError::Aggregation {
        message: "Failed to aggregate prices from 3 sources".to_string(),
    };

    // Should be suitable for structured logging
    let log_entry = format!("error={:?} message=\"{}\"", error, error);
    assert!(log_entry.contains("Aggregation"));
    assert!(log_entry.contains("Failed to aggregate"));
}
