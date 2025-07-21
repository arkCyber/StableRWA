// =====================================================================================
// File: core-institutional/src/error.rs
// Description: Error types for institutional services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type for institutional operations
pub type InstitutionalResult<T> = Result<T, InstitutionalError>;

/// Institutional service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum InstitutionalError {
    /// Validation errors
    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    /// Authentication errors
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },

    /// Authorization errors
    #[error("Authorization failed: {message}")]
    AuthorizationError { message: String },

    /// Custody service errors
    #[error("Custody service error: {message}")]
    CustodyError { message: String },

    /// Bulk trading errors
    #[error("Bulk trading error: {message}")]
    BulkTradingError { message: String },

    /// White label service errors
    #[error("White label service error: {message}")]
    WhiteLabelError { message: String },

    /// API gateway errors
    #[error("API gateway error: {message}")]
    ApiGatewayError { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Database errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// Network errors
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimitError { message: String },

    /// Resource not found
    #[error("Resource not found: {resource_type} with id '{id}'")]
    NotFound { resource_type: String, id: String },

    /// Resource already exists
    #[error("Resource already exists: {resource_type} with id '{id}'")]
    AlreadyExists { resource_type: String, id: String },

    /// Insufficient permissions
    #[error("Insufficient permissions: {required_permission}")]
    InsufficientPermissions { required_permission: String },

    /// Service unavailable
    #[error("Service unavailable: {service_name}")]
    ServiceUnavailable { service_name: String },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Capacity errors
    #[error("Capacity exceeded: {resource}")]
    CapacityExceeded { resource: String },

    /// Compliance errors
    #[error("Compliance violation: {violation}")]
    ComplianceViolation { violation: String },

    /// Security errors
    #[error("Security violation: {violation}")]
    SecurityViolation { violation: String },

    /// Internal server errors
    #[error("Internal server error: {message}")]
    InternalError { message: String },

    /// External service errors
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
}

impl InstitutionalError {
    /// Create a validation error
    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
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

    /// Create a custody error
    pub fn custody_error<S: Into<String>>(message: S) -> Self {
        Self::CustodyError {
            message: message.into(),
        }
    }

    /// Create a bulk trading error
    pub fn bulk_trading_error<S: Into<String>>(message: S) -> Self {
        Self::BulkTradingError {
            message: message.into(),
        }
    }

    /// Create a white label error
    pub fn white_label_error<S: Into<String>>(message: S) -> Self {
        Self::WhiteLabelError {
            message: message.into(),
        }
    }

    /// Create an API gateway error
    pub fn api_gateway_error<S: Into<String>>(message: S) -> Self {
        Self::ApiGatewayError {
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

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource_type: S, id: S) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Create an already exists error
    pub fn already_exists<S: Into<String>>(resource_type: S, id: S) -> Self {
        Self::AlreadyExists {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Create an insufficient permissions error
    pub fn insufficient_permissions<S: Into<String>>(required_permission: S) -> Self {
        Self::InsufficientPermissions {
            required_permission: required_permission.into(),
        }
    }

    /// Create a service unavailable error
    pub fn service_unavailable<S: Into<String>>(service_name: S) -> Self {
        Self::ServiceUnavailable {
            service_name: service_name.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a capacity exceeded error
    pub fn capacity_exceeded<S: Into<String>>(resource: S) -> Self {
        Self::CapacityExceeded {
            resource: resource.into(),
        }
    }

    /// Create a compliance violation error
    pub fn compliance_violation<S: Into<String>>(violation: S) -> Self {
        Self::ComplianceViolation {
            violation: violation.into(),
        }
    }

    /// Create a security violation error
    pub fn security_violation<S: Into<String>>(violation: S) -> Self {
        Self::SecurityViolation {
            violation: violation.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        Self::InternalError {
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

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            InstitutionalError::ValidationError { .. } => "VALIDATION_ERROR",
            InstitutionalError::AuthenticationError { .. } => "AUTHENTICATION_ERROR",
            InstitutionalError::AuthorizationError { .. } => "AUTHORIZATION_ERROR",
            InstitutionalError::CustodyError { .. } => "CUSTODY_ERROR",
            InstitutionalError::BulkTradingError { .. } => "BULK_TRADING_ERROR",
            InstitutionalError::WhiteLabelError { .. } => "WHITE_LABEL_ERROR",
            InstitutionalError::ApiGatewayError { .. } => "API_GATEWAY_ERROR",
            InstitutionalError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            InstitutionalError::DatabaseError { .. } => "DATABASE_ERROR",
            InstitutionalError::NetworkError { .. } => "NETWORK_ERROR",
            InstitutionalError::SerializationError { .. } => "SERIALIZATION_ERROR",
            InstitutionalError::RateLimitError { .. } => "RATE_LIMIT_ERROR",
            InstitutionalError::NotFound { .. } => "NOT_FOUND",
            InstitutionalError::AlreadyExists { .. } => "ALREADY_EXISTS",
            InstitutionalError::InsufficientPermissions { .. } => "INSUFFICIENT_PERMISSIONS",
            InstitutionalError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            InstitutionalError::Timeout { .. } => "TIMEOUT",
            InstitutionalError::CapacityExceeded { .. } => "CAPACITY_EXCEEDED",
            InstitutionalError::ComplianceViolation { .. } => "COMPLIANCE_VIOLATION",
            InstitutionalError::SecurityViolation { .. } => "SECURITY_VIOLATION",
            InstitutionalError::InternalError { .. } => "INTERNAL_ERROR",
            InstitutionalError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            InstitutionalError::NetworkError { .. }
                | InstitutionalError::ServiceUnavailable { .. }
                | InstitutionalError::Timeout { .. }
                | InstitutionalError::ExternalServiceError { .. }
        )
    }

    /// Check if error is client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            InstitutionalError::ValidationError { .. }
                | InstitutionalError::AuthenticationError { .. }
                | InstitutionalError::AuthorizationError { .. }
                | InstitutionalError::NotFound { .. }
                | InstitutionalError::AlreadyExists { .. }
                | InstitutionalError::InsufficientPermissions { .. }
                | InstitutionalError::RateLimitError { .. }
                | InstitutionalError::ComplianceViolation { .. }
                | InstitutionalError::SecurityViolation { .. }
        )
    }

    /// Check if error is server error (5xx)
    pub fn is_server_error(&self) -> bool {
        !self.is_client_error()
    }
}

// Implement From traits for common error types
impl From<sqlx::Error> for InstitutionalError {
    fn from(err: sqlx::Error) -> Self {
        InstitutionalError::database_error(err.to_string())
    }
}

impl From<redis::RedisError> for InstitutionalError {
    fn from(err: redis::RedisError) -> Self {
        InstitutionalError::database_error(format!("Redis error: {}", err))
    }
}

impl From<reqwest::Error> for InstitutionalError {
    fn from(err: reqwest::Error) -> Self {
        InstitutionalError::network_error(err.to_string())
    }
}

impl From<serde_json::Error> for InstitutionalError {
    fn from(err: serde_json::Error) -> Self {
        InstitutionalError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<tokio::time::error::Elapsed> for InstitutionalError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        InstitutionalError::timeout(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = InstitutionalError::validation_error("amount", "Amount must be positive");
        assert_eq!(error.error_code(), "VALIDATION_ERROR");
        assert!(error.is_client_error());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_classification() {
        let client_error = InstitutionalError::authentication_error("Invalid token");
        assert!(client_error.is_client_error());
        assert!(!client_error.is_server_error());

        let server_error = InstitutionalError::internal_error("Database connection failed");
        assert!(server_error.is_server_error());
        assert!(!server_error.is_client_error());

        let retryable_error = InstitutionalError::network_error("Connection timeout");
        assert!(retryable_error.is_retryable());
    }
}
