// =====================================================================================
// File: core-utils/src/lib.rs
// Description: Utility functions and testing tools for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod testing;
pub mod fixtures;
pub mod helpers;
pub mod validation;

pub use testing::*;
pub use fixtures::*;
pub use helpers::*;
pub use validation::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, error};

/// Utility error types
#[derive(Error, Debug)]
pub enum UtilError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Processing error: {0}")]
    ProcessingError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

/// Common result type for utilities
pub type UtilResult<T> = Result<T, UtilError>;

/// Format timestamp in ISO 8601 format
pub fn format_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Format timestamp with custom format
pub fn format_timestamp_custom(format: &str) -> String {
    Utc::now().format(format).to_string()
}

/// Parse timestamp from string
pub fn parse_timestamp(timestamp: &str) -> UtilResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| UtilError::ProcessingError(format!("Failed to parse timestamp: {}", e)))
}

/// Generate a unique identifier
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a short unique identifier (8 characters)
pub fn generate_short_id() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a function with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == config.max_attempts {
                    return Err(e);
                }

                tracing::warn!(
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    delay_ms = delay.as_millis(),
                    error = %e,
                    "Operation failed, retrying"
                );

                tokio::time::sleep(delay).await;
                delay = std::cmp::min(
                    std::time::Duration::from_millis(
                        (delay.as_millis() as f64 * config.backoff_multiplier) as u64
                    ),
                    config.max_delay,
                );
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_format_timestamp() {
        let timestamp = format_timestamp();
        assert!(timestamp.contains('T'));
        // RFC3339 format may end with 'Z' or '+00:00' for UTC
        assert!(timestamp.ends_with('Z') || timestamp.ends_with("+00:00"));
    }

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID v4 length
    }

    #[test]
    fn test_generate_short_id() {
        let id = generate_short_id();
        assert_eq!(id.len(), 8);
    }

    #[test]
    fn test_parse_timestamp() {
        let timestamp_str = "2023-01-01T12:00:00Z";
        let parsed = parse_timestamp(timestamp_str).unwrap();
        assert_eq!(parsed.year(), 2023);
        assert_eq!(parsed.month(), 1);
        assert_eq!(parsed.day(), 1);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, std::time::Duration::from_millis(100));
    }
}
