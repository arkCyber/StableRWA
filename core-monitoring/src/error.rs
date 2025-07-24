// =====================================================================================
// File: core-monitoring/src/error.rs
// Description: Error types for monitoring and alerting operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for monitoring operations
pub type MonitoringResult<T> = Result<T, MonitoringError>;

/// Monitoring service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringError {
    /// Metrics collection errors
    #[error("Metrics collection error: {message}")]
    MetricsError { message: String },

    /// Alert processing errors
    #[error("Alert processing error: {message}")]
    AlertError { message: String },

    /// Dashboard errors
    #[error("Dashboard error: {message}")]
    DashboardError { message: String },

    /// Anomaly detection errors
    #[error("Anomaly detection error: {message}")]
    AnomalyDetectionError { message: String },

    /// Performance monitoring errors
    #[error("Performance monitoring error: {message}")]
    PerformanceError { message: String },

    /// Health check errors
    #[error("Health check error: {message}")]
    HealthCheckError { message: String },

    /// Log aggregation errors
    #[error("Log aggregation error: {message}")]
    LogAggregationError { message: String },

    /// Distributed tracing errors
    #[error("Distributed tracing error: {message}")]
    DistributedTracingError { message: String },

    /// Notification errors
    #[error("Notification error: {message}")]
    NotificationError { message: String },

    /// Scheduler errors
    #[error("Scheduler error: {message}")]
    SchedulerError { message: String },

    /// Data validation errors
    #[error("Data validation error: {field}: {message}")]
    DataValidationError { field: String, message: String },

    /// External service errors
    #[error("External service error: {service}: {message}")]
    ExternalServiceError { service: String, message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Database errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// Network errors
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Authentication errors
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },

    /// Authorization errors
    #[error("Authorization error: {message}")]
    AuthorizationError { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {service}")]
    RateLimitExceeded { service: String },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Storage errors
    #[error("Storage error: {message}")]
    StorageError { message: String },

    /// Query errors
    #[error("Query error: {message}")]
    QueryError { message: String },

    /// Aggregation errors
    #[error("Aggregation error: {message}")]
    AggregationError { message: String },

    /// Export errors
    #[error("Export error: {message}")]
    ExportError { message: String },

    /// Internal errors
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl MonitoringError {
    /// Create a metrics error
    pub fn metrics_error<S: Into<String>>(message: S) -> Self {
        Self::MetricsError {
            message: message.into(),
        }
    }

    /// Create an alert error
    pub fn alert_error<S: Into<String>>(message: S) -> Self {
        Self::AlertError {
            message: message.into(),
        }
    }

    /// Create a dashboard error
    pub fn dashboard_error<S: Into<String>>(message: S) -> Self {
        Self::DashboardError {
            message: message.into(),
        }
    }

    /// Create an anomaly detection error
    pub fn anomaly_detection_error<S: Into<String>>(message: S) -> Self {
        Self::AnomalyDetectionError {
            message: message.into(),
        }
    }

    /// Create a performance error
    pub fn performance_error<S: Into<String>>(message: S) -> Self {
        Self::PerformanceError {
            message: message.into(),
        }
    }

    /// Create a health check error
    pub fn health_check_error<S: Into<String>>(message: S) -> Self {
        Self::HealthCheckError {
            message: message.into(),
        }
    }

    /// Create a log aggregation error
    pub fn log_aggregation_error<S: Into<String>>(message: S) -> Self {
        Self::LogAggregationError {
            message: message.into(),
        }
    }

    /// Create a distributed tracing error
    pub fn distributed_tracing_error<S: Into<String>>(message: S) -> Self {
        Self::DistributedTracingError {
            message: message.into(),
        }
    }

    /// Create a notification error
    pub fn notification_error<S: Into<String>>(message: S) -> Self {
        Self::NotificationError {
            message: message.into(),
        }
    }

    /// Create a scheduler error
    pub fn scheduler_error<S: Into<String>>(message: S) -> Self {
        Self::SchedulerError {
            message: message.into(),
        }
    }

    /// Create a data validation error
    pub fn data_validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::DataValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an external service error
    pub fn external_service_error<S: Into<String>>(service: S, message: S) -> Self {
        Self::ExternalServiceError {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a database error
    pub fn database_error<S: Into<String>>(message: S) -> Self {
        Self::DatabaseError {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication_error<S: Into<String>>(message: S) -> Self {
        Self::AuthenticationError {
            message: message.into(),
        }
    }

    /// Create an authorization error
    pub fn authorization_error<S: Into<String>>(message: S) -> Self {
        Self::AuthorizationError {
            message: message.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded<S: Into<String>>(service: S) -> Self {
        Self::RateLimitExceeded {
            service: service.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error<S: Into<String>>(message: S) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create a storage error
    pub fn storage_error<S: Into<String>>(message: S) -> Self {
        Self::StorageError {
            message: message.into(),
        }
    }

    /// Create a query error
    pub fn query_error<S: Into<String>>(message: S) -> Self {
        Self::QueryError {
            message: message.into(),
        }
    }

    /// Create an aggregation error
    pub fn aggregation_error<S: Into<String>>(message: S) -> Self {
        Self::AggregationError {
            message: message.into(),
        }
    }

    /// Create an export error
    pub fn export_error<S: Into<String>>(message: S) -> Self {
        Self::ExportError {
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
            Self::MetricsError { .. } => "METRICS_ERROR",
            Self::AlertError { .. } => "ALERT_ERROR",
            Self::DashboardError { .. } => "DASHBOARD_ERROR",
            Self::AnomalyDetectionError { .. } => "ANOMALY_DETECTION_ERROR",
            Self::PerformanceError { .. } => "PERFORMANCE_ERROR",
            Self::HealthCheckError { .. } => "HEALTH_CHECK_ERROR",
            Self::LogAggregationError { .. } => "LOG_AGGREGATION_ERROR",
            Self::DistributedTracingError { .. } => "DISTRIBUTED_TRACING_ERROR",
            Self::NotificationError { .. } => "NOTIFICATION_ERROR",
            Self::SchedulerError { .. } => "SCHEDULER_ERROR",
            Self::DataValidationError { .. } => "DATA_VALIDATION_ERROR",
            Self::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            Self::DatabaseError { .. } => "DATABASE_ERROR",
            Self::NetworkError { .. } => "NETWORK_ERROR",
            Self::AuthenticationError { .. } => "AUTHENTICATION_ERROR",
            Self::AuthorizationError { .. } => "AUTHORIZATION_ERROR",
            Self::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            Self::Timeout { .. } => "TIMEOUT",
            Self::SerializationError { .. } => "SERIALIZATION_ERROR",
            Self::StorageError { .. } => "STORAGE_ERROR",
            Self::QueryError { .. } => "QUERY_ERROR",
            Self::AggregationError { .. } => "AGGREGATION_ERROR",
            Self::ExportError { .. } => "EXPORT_ERROR",
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
                | Self::ExternalServiceError { .. }
                | Self::DatabaseError { .. }
                | Self::StorageError { .. }
        )
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::HealthCheckError { .. }
                | Self::PerformanceError { .. }
                | Self::InternalError { .. }
                | Self::ConfigurationError { .. }
        )
    }

    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(
            self,
            Self::HealthCheckError { .. }
                | Self::PerformanceError { .. }
                | Self::AnomalyDetectionError { .. }
                | Self::InternalError { .. }
        )
    }

    /// Check if error is monitoring-related
    pub fn is_monitoring_related(&self) -> bool {
        matches!(
            self,
            Self::MetricsError { .. }
                | Self::AlertError { .. }
                | Self::DashboardError { .. }
                | Self::AnomalyDetectionError { .. }
                | Self::PerformanceError { .. }
                | Self::HealthCheckError { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = MonitoringError::metrics_error("Test metrics error");
        assert_eq!(error.error_code(), "METRICS_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(error.is_monitoring_related());
    }

    #[test]
    fn test_alert_error() {
        let error = MonitoringError::alert_error("Alert processing failed");
        assert_eq!(error.error_code(), "ALERT_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(error.is_monitoring_related());
    }

    #[test]
    fn test_health_check_error() {
        let error = MonitoringError::health_check_error("Service unhealthy");
        assert_eq!(error.error_code(), "HEALTH_CHECK_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(error.is_monitoring_related());
    }

    #[test]
    fn test_retryable_error() {
        let error = MonitoringError::network_error("Connection failed");
        assert_eq!(error.error_code(), "NETWORK_ERROR");
        assert!(error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_monitoring_related());
    }

    #[test]
    fn test_performance_error() {
        let error = MonitoringError::performance_error("High latency detected");
        assert_eq!(error.error_code(), "PERFORMANCE_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(error.is_monitoring_related());
    }

    #[test]
    fn test_data_validation_error() {
        let error = MonitoringError::data_validation_error("metric_name", "Invalid format");
        assert_eq!(error.error_code(), "DATA_VALIDATION_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_monitoring_related());
    }
}
