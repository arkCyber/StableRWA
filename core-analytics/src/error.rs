// =====================================================================================
// File: core-analytics/src/error.rs
// Description: Error types for analytics system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type alias for analytics operations
pub type AnalyticsResult<T> = Result<T, AnalyticsError>;

/// Comprehensive error types for analytics operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsError {
    /// Metrics collection errors
    #[error("Metrics error: {message}")]
    MetricsError { message: String },

    /// Data aggregation errors
    #[error("Aggregation error: {message}")]
    AggregationError { message: String },

    /// Report generation errors
    #[error("Report generation error: {message}")]
    ReportGenerationError { message: String },

    /// Visualization errors
    #[error("Visualization error: {message}")]
    VisualizationError { message: String },

    /// Forecasting errors
    #[error("Forecasting error: {model} - {message}")]
    ForecastingError { model: String, message: String },

    /// Performance analysis errors
    #[error("Performance analysis error: {message}")]
    PerformanceError { message: String },

    /// Query execution errors
    #[error("Query error: {query_type} - {message}")]
    QueryError { query_type: String, message: String },

    /// Data source errors
    #[error("Data source error: {message}")]
    DataSourceError { message: String },

    /// Data quality errors
    #[error("Data quality error: {issue_type} - {message}")]
    DataQualityError { issue_type: String, message: String },

    /// Invalid query syntax
    #[error("Invalid query: {query} - {message}")]
    InvalidQuery { query: String, message: String },

    /// Metric not found
    #[error("Metric not found: {metric_name}")]
    MetricNotFound { metric_name: String },

    /// Dashboard errors
    #[error("Dashboard error: {dashboard_id} - {message}")]
    DashboardError {
        dashboard_id: String,
        message: String,
    },

    /// Widget errors
    #[error("Widget error: {widget_id} - {message}")]
    WidgetError { widget_id: String, message: String },

    /// Alert configuration errors
    #[error("Alert configuration error: {alert_id} - {message}")]
    AlertConfigError { alert_id: String, message: String },

    /// Data export errors
    #[error("Export error: {format} - {message}")]
    ExportError { format: String, message: String },

    /// Data import errors
    #[error("Import error: {message}")]
    ImportError { message: String },

    /// Cache errors
    #[error("Cache error: {operation} - {message}")]
    CacheError { operation: String, message: String },

    /// Computation timeout
    #[error("Computation timeout: {operation} - exceeded {timeout_seconds}s")]
    ComputationTimeout {
        operation: String,
        timeout_seconds: u64,
    },

    /// Insufficient data
    #[error("Insufficient data: {data_type} - {message}")]
    InsufficientData { data_type: String, message: String },

    /// Data format errors
    #[error("Data format error: {expected_format} - {message}")]
    DataFormatError {
        expected_format: String,
        message: String,
    },

    /// Schema validation errors
    #[error("Schema validation error: {schema} - {message}")]
    SchemaValidationError { schema: String, message: String },

    /// Permission errors
    #[error("Permission denied: {resource} - {required_permission}")]
    PermissionDenied {
        resource: String,
        required_permission: String,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource_type} - {limit}")]
    ResourceLimitExceeded {
        resource_type: String,
        limit: String,
    },

    /// Configuration errors
    #[error("Configuration error: {component} - {message}")]
    ConfigurationError { component: String, message: String },

    /// External service errors
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    /// Database errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// Network/HTTP errors
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Authentication/Authorization errors
    #[error("Authentication error: {message}")]
    AuthenticationError { message: String },

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Concurrent modification error
    #[error("Concurrent modification detected for {resource_type} {resource_id}")]
    ConcurrentModification {
        resource_type: String,
        resource_id: String,
    },

    /// Business rule violation
    #[error("Business rule violation: {rule} - {message}")]
    BusinessRuleViolation { rule: String, message: String },

    /// Internal system errors
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Generic analytics error
    #[error("Analytics error: {0}")]
    Generic(String),
}

impl AnalyticsError {
    /// Create a metrics error
    pub fn metrics_error<S: Into<String>>(message: S) -> Self {
        Self::MetricsError {
            message: message.into(),
        }
    }

    /// Create an aggregation error
    pub fn aggregation_error<S: Into<String>>(message: S) -> Self {
        Self::AggregationError {
            message: message.into(),
        }
    }

    /// Create a report generation error
    pub fn report_generation_error<S: Into<String>>(message: S) -> Self {
        Self::ReportGenerationError {
            message: message.into(),
        }
    }

    /// Create a visualization error
    pub fn visualization_error<S: Into<String>>(message: S) -> Self {
        Self::VisualizationError {
            message: message.into(),
        }
    }

    /// Create a forecasting error
    pub fn forecasting_error<S: Into<String>>(model: S, message: S) -> Self {
        Self::ForecastingError {
            model: model.into(),
            message: message.into(),
        }
    }

    /// Create a query error
    pub fn query_error<S: Into<String>>(query_type: S, message: S) -> Self {
        Self::QueryError {
            query_type: query_type.into(),
            message: message.into(),
        }
    }

    /// Create a data source error
    pub fn data_source_error<S: Into<String>>(message: S) -> Self {
        Self::DataSourceError {
            message: message.into(),
        }
    }

    /// Create an import error
    pub fn import_error<S: Into<String>>(message: S) -> Self {
        Self::ImportError {
            message: message.into(),
        }
    }

    /// Create a data quality error
    pub fn data_quality_error<S: Into<String>>(issue_type: S, message: S) -> Self {
        Self::DataQualityError {
            issue_type: issue_type.into(),
            message: message.into(),
        }
    }

    /// Create an invalid query error
    pub fn invalid_query<S: Into<String>>(query: S, message: S) -> Self {
        Self::InvalidQuery {
            query: query.into(),
            message: message.into(),
        }
    }

    /// Create a metric not found error
    pub fn metric_not_found<S: Into<String>>(metric_name: S) -> Self {
        Self::MetricNotFound {
            metric_name: metric_name.into(),
        }
    }

    /// Create a dashboard error
    pub fn dashboard_error<S: Into<String>>(dashboard_id: S, message: S) -> Self {
        Self::DashboardError {
            dashboard_id: dashboard_id.into(),
            message: message.into(),
        }
    }

    /// Create a computation timeout error
    pub fn computation_timeout<S: Into<String>>(operation: S, timeout_seconds: u64) -> Self {
        Self::ComputationTimeout {
            operation: operation.into(),
            timeout_seconds,
        }
    }

    /// Create an insufficient data error
    pub fn insufficient_data<S: Into<String>>(data_type: S, message: S) -> Self {
        Self::InsufficientData {
            data_type: data_type.into(),
            message: message.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied<S: Into<String>>(resource: S, required_permission: S) -> Self {
        Self::PermissionDenied {
            resource: resource.into(),
            required_permission: required_permission.into(),
        }
    }

    /// Create a resource limit exceeded error
    pub fn resource_limit_exceeded<S: Into<String>>(resource_type: S, limit: S) -> Self {
        Self::ResourceLimitExceeded {
            resource_type: resource_type.into(),
            limit: limit.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a template not found error
    pub fn template_not_found<S: Into<String>>(template_id: S) -> Self {
        Self::MetricNotFound {
            metric_name: format!("template_{}", template_id.into()),
        }
    }

    /// Create a request not found error
    pub fn request_not_found<S: Into<String>>(request_id: S) -> Self {
        Self::MetricNotFound {
            metric_name: format!("request_{}", request_id.into()),
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AnalyticsError::NetworkError { .. }
                | AnalyticsError::DatabaseError { .. }
                | AnalyticsError::ExternalServiceError { .. }
                | AnalyticsError::ComputationTimeout { .. }
                | AnalyticsError::CacheError { .. }
                | AnalyticsError::InternalError { .. }
        )
    }

    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            AnalyticsError::MetricsError { .. } => "metrics",
            AnalyticsError::AggregationError { .. } => "aggregation",
            AnalyticsError::ReportGenerationError { .. } => "report_generation",
            AnalyticsError::VisualizationError { .. } => "visualization",
            AnalyticsError::ForecastingError { .. } => "forecasting",
            AnalyticsError::PerformanceError { .. } => "performance",
            AnalyticsError::QueryError { .. } => "query",
            AnalyticsError::DataSourceError { .. } => "data_source",
            AnalyticsError::DataQualityError { .. } => "data_quality",
            AnalyticsError::InvalidQuery { .. } => "invalid_query",
            AnalyticsError::MetricNotFound { .. } => "metric_not_found",
            AnalyticsError::DashboardError { .. } => "dashboard",
            AnalyticsError::WidgetError { .. } => "widget",
            AnalyticsError::AlertConfigError { .. } => "alert_config",
            AnalyticsError::ExportError { .. } => "export",
            AnalyticsError::ImportError { .. } => "import",
            AnalyticsError::CacheError { .. } => "cache",
            AnalyticsError::ComputationTimeout { .. } => "computation_timeout",
            AnalyticsError::InsufficientData { .. } => "insufficient_data",
            AnalyticsError::DataFormatError { .. } => "data_format",
            AnalyticsError::SchemaValidationError { .. } => "schema_validation",
            AnalyticsError::PermissionDenied { .. } => "permission_denied",
            AnalyticsError::ResourceLimitExceeded { .. } => "resource_limit",
            AnalyticsError::ConfigurationError { .. } => "configuration",
            AnalyticsError::ExternalServiceError { .. } => "external_service",
            AnalyticsError::DatabaseError { .. } => "database",
            AnalyticsError::NetworkError { .. } => "network",
            AnalyticsError::SerializationError { .. } => "serialization",
            AnalyticsError::AuthenticationError { .. } => "authentication",
            AnalyticsError::ValidationError { .. } => "validation",
            AnalyticsError::ConcurrentModification { .. } => "concurrency",
            AnalyticsError::BusinessRuleViolation { .. } => "business_rule",
            AnalyticsError::InternalError { .. } => "internal",
            AnalyticsError::Generic(_) => "generic",
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AnalyticsError::InternalError { .. } => ErrorSeverity::Critical,
            AnalyticsError::DatabaseError { .. } => ErrorSeverity::High,
            AnalyticsError::AuthenticationError { .. } => ErrorSeverity::High,
            AnalyticsError::BusinessRuleViolation { .. } => ErrorSeverity::High,
            AnalyticsError::DataQualityError { .. } => ErrorSeverity::High,
            AnalyticsError::ComputationTimeout { .. } => ErrorSeverity::Medium,
            AnalyticsError::ResourceLimitExceeded { .. } => ErrorSeverity::Medium,
            AnalyticsError::PermissionDenied { .. } => ErrorSeverity::Medium,
            AnalyticsError::ValidationError { .. } => ErrorSeverity::Medium,
            AnalyticsError::QueryError { .. } => ErrorSeverity::Medium,
            AnalyticsError::NetworkError { .. } => ErrorSeverity::Low,
            AnalyticsError::MetricNotFound { .. } => ErrorSeverity::Low,
            AnalyticsError::CacheError { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }

    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(
            self.severity(),
            ErrorSeverity::Critical | ErrorSeverity::High
        )
    }

    /// Check if error affects data accuracy
    pub fn affects_data_accuracy(&self) -> bool {
        matches!(
            self,
            AnalyticsError::DataQualityError { .. }
                | AnalyticsError::DataSourceError { .. }
                | AnalyticsError::AggregationError { .. }
                | AnalyticsError::ForecastingError { .. }
                | AnalyticsError::SchemaValidationError { .. }
        )
    }

    /// Check if error is user-facing (vs system error)
    pub fn is_user_facing(&self) -> bool {
        matches!(
            self,
            AnalyticsError::InvalidQuery { .. }
                | AnalyticsError::MetricNotFound { .. }
                | AnalyticsError::PermissionDenied { .. }
                | AnalyticsError::ValidationError { .. }
                | AnalyticsError::InsufficientData { .. }
                | AnalyticsError::ExportError { .. }
                | AnalyticsError::ImportError { .. }
        )
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Implement conversions from common error types
impl From<serde_json::Error> for AnalyticsError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for AnalyticsError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "database")]
impl From<sqlx::Error> for AnalyticsError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<validator::ValidationErrors> for AnalyticsError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError {
            field: "multiple".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<polars::error::PolarsError> for AnalyticsError {
    fn from(err: polars::error::PolarsError) -> Self {
        Self::DataSourceError {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let metrics_error = AnalyticsError::metrics_error("Failed to collect metrics");
        assert!(matches!(metrics_error, AnalyticsError::MetricsError { .. }));
        assert_eq!(metrics_error.category(), "metrics");
    }

    #[test]
    fn test_error_retryability() {
        let network_error = AnalyticsError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(network_error.is_retryable());

        let validation_error = AnalyticsError::ValidationError {
            field: "metric_name".to_string(),
            message: "Invalid metric name".to_string(),
        };
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let internal_error = AnalyticsError::InternalError {
            message: "System failure".to_string(),
        };
        assert_eq!(internal_error.severity(), ErrorSeverity::Critical);
        assert!(internal_error.requires_immediate_attention());

        let cache_error = AnalyticsError::CacheError {
            operation: "get".to_string(),
            message: "Cache miss".to_string(),
        };
        assert_eq!(cache_error.severity(), ErrorSeverity::Low);
        assert!(!cache_error.requires_immediate_attention());
    }

    #[test]
    fn test_data_accuracy_errors() {
        let data_quality_error = AnalyticsError::DataQualityError {
            issue_type: "missing_values".to_string(),
            message: "Dataset contains missing values".to_string(),
        };
        assert!(data_quality_error.affects_data_accuracy());

        let network_error = AnalyticsError::NetworkError {
            message: "Connection failed".to_string(),
        };
        assert!(!network_error.affects_data_accuracy());
    }

    #[test]
    fn test_user_facing_errors() {
        let invalid_query = AnalyticsError::InvalidQuery {
            query: "SELECT * FROM invalid_table".to_string(),
            message: "Table does not exist".to_string(),
        };
        assert!(invalid_query.is_user_facing());

        let internal_error = AnalyticsError::InternalError {
            message: "System failure".to_string(),
        };
        assert!(!internal_error.is_user_facing());
    }

    #[test]
    fn test_specific_error_constructors() {
        let timeout_error = AnalyticsError::computation_timeout("aggregation", 300);
        assert!(matches!(
            timeout_error,
            AnalyticsError::ComputationTimeout { .. }
        ));

        let permission_error = AnalyticsError::permission_denied("dashboard", "read");
        assert!(matches!(
            permission_error,
            AnalyticsError::PermissionDenied { .. }
        ));

        let forecasting_error =
            AnalyticsError::forecasting_error("ARIMA", "Model convergence failed");
        assert!(matches!(
            forecasting_error,
            AnalyticsError::ForecastingError { .. }
        ));
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            AnalyticsError::metrics_error("test"),
            AnalyticsError::aggregation_error("test"),
            AnalyticsError::report_generation_error("test"),
            AnalyticsError::visualization_error("test"),
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(
            categories,
            vec![
                "metrics",
                "aggregation",
                "report_generation",
                "visualization"
            ]
        );
    }
}
