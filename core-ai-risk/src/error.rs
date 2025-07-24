// =====================================================================================
// File: core-ai-risk/src/error.rs
// Description: Error types for AI risk assessment services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for AI risk operations
pub type AIRiskResult<T> = Result<T, AIRiskError>;

/// AI risk service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AIRiskError {
    /// Model errors
    #[error("Model error: {message}")]
    ModelError { message: String },

    /// Training errors
    #[error("Training error: {message}")]
    TrainingError { message: String },

    /// Prediction errors
    #[error("Prediction error: {message}")]
    PredictionError { message: String },

    /// Feature engineering errors
    #[error("Feature engineering error: {message}")]
    FeatureError { message: String },

    /// Data validation errors
    #[error("Data validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    /// Model not found
    #[error("Model not found: {model_id}")]
    ModelNotFound { model_id: String },

    /// Model version mismatch
    #[error("Model version mismatch: expected {expected}, found {actual}")]
    VersionMismatch { expected: String, actual: String },

    /// Insufficient data
    #[error("Insufficient data: {message}")]
    InsufficientData { message: String },

    /// Model drift detected
    #[error("Model drift detected: {metric} = {value}, threshold = {threshold}")]
    ModelDrift {
        metric: String,
        value: f64,
        threshold: f64,
    },

    /// Performance degradation
    #[error("Performance degradation: {metric} = {value}, minimum = {minimum}")]
    PerformanceDegradation {
        metric: String,
        value: f64,
        minimum: f64,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Database errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// External service errors
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Resource not available
    #[error("Resource not available: {resource}")]
    ResourceUnavailable { resource: String },

    /// Internal server errors
    #[error("Internal server error: {message}")]
    InternalError { message: String },
}

impl AIRiskError {
    /// Create a model error
    pub fn model_error<S: Into<String>>(message: S) -> Self {
        Self::ModelError {
            message: message.into(),
        }
    }

    /// Create a training error
    pub fn training_error<S: Into<String>>(message: S) -> Self {
        Self::TrainingError {
            message: message.into(),
        }
    }

    /// Create a prediction error
    pub fn prediction_error<S: Into<String>>(message: S) -> Self {
        Self::PredictionError {
            message: message.into(),
        }
    }

    /// Create a feature error
    pub fn feature_error<S: Into<String>>(message: S) -> Self {
        Self::FeatureError {
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

    /// Create a model not found error
    pub fn model_not_found<S: Into<String>>(model_id: S) -> Self {
        Self::ModelNotFound {
            model_id: model_id.into(),
        }
    }

    /// Create a version mismatch error
    pub fn version_mismatch<S: Into<String>>(expected: S, actual: S) -> Self {
        Self::VersionMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create an insufficient data error
    pub fn insufficient_data<S: Into<String>>(message: S) -> Self {
        Self::InsufficientData {
            message: message.into(),
        }
    }

    /// Create a model drift error
    pub fn model_drift<S: Into<String>>(metric: S, value: f64, threshold: f64) -> Self {
        Self::ModelDrift {
            metric: metric.into(),
            value,
            threshold,
        }
    }

    /// Create a performance degradation error
    pub fn performance_degradation<S: Into<String>>(metric: S, value: f64, minimum: f64) -> Self {
        Self::PerformanceDegradation {
            metric: metric.into(),
            value,
            minimum,
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

    /// Create an external service error
    pub fn external_service_error<S: Into<String>>(service: S, message: S) -> Self {
        Self::ExternalServiceError {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a resource unavailable error
    pub fn resource_unavailable<S: Into<String>>(resource: S) -> Self {
        Self::ResourceUnavailable {
            resource: resource.into(),
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
            Self::ModelError { .. } => "MODEL_ERROR",
            Self::TrainingError { .. } => "TRAINING_ERROR",
            Self::PredictionError { .. } => "PREDICTION_ERROR",
            Self::FeatureError { .. } => "FEATURE_ERROR",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::ModelNotFound { .. } => "MODEL_NOT_FOUND",
            Self::VersionMismatch { .. } => "VERSION_MISMATCH",
            Self::InsufficientData { .. } => "INSUFFICIENT_DATA",
            Self::ModelDrift { .. } => "MODEL_DRIFT",
            Self::PerformanceDegradation { .. } => "PERFORMANCE_DEGRADATION",
            Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            Self::DatabaseError { .. } => "DATABASE_ERROR",
            Self::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::Timeout { .. } => "TIMEOUT",
            Self::ResourceUnavailable { .. } => "RESOURCE_UNAVAILABLE",
            Self::InternalError { .. } => "INTERNAL_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::ResourceUnavailable { .. }
                | Self::ExternalServiceError { .. }
                | Self::DatabaseError { .. }
        )
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::ModelDrift { .. }
                | Self::PerformanceDegradation { .. }
                | Self::InternalError { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AIRiskError::model_error("Test model error");
        assert_eq!(error.error_code(), "MODEL_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
    }

    #[test]
    fn test_model_drift_error() {
        let error = AIRiskError::model_drift("accuracy", 0.75, 0.85);
        assert_eq!(error.error_code(), "MODEL_DRIFT");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
    }

    #[test]
    fn test_retryable_error() {
        let error = AIRiskError::timeout("model_prediction");
        assert_eq!(error.error_code(), "TIMEOUT");
        assert!(error.is_retryable());
        assert!(!error.is_critical());
    }
}
