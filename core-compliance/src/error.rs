// =====================================================================================
// File: core-compliance/src/error.rs
// Description: Error types for compliance module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type alias for compliance operations
pub type ComplianceResult<T> = Result<T, ComplianceError>;

/// Comprehensive error types for compliance operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceError {
    /// KYC related errors
    #[error("KYC error: {message}")]
    KycError { message: String },

    /// AML related errors
    #[error("AML error: {message}")]
    AmlError { message: String },

    /// Jurisdiction compliance errors
    #[error("Jurisdiction compliance error: {jurisdiction} - {message}")]
    JurisdictionError { jurisdiction: String, message: String },

    /// Regulatory reporting errors
    #[error("Reporting error: {message}")]
    ReportingError { message: String },

    /// Audit trail errors
    #[error("Audit error: {message}")]
    AuditError { message: String },

    /// External provider errors
    #[error("Provider error: {provider} - {message}")]
    ProviderError { provider: String, message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

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

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimitError { message: String },

    /// Compliance status errors
    #[error("Compliance status error: {status} - {message}")]
    StatusError { status: String, message: String },

    /// Insufficient compliance level
    #[error("Insufficient compliance level: required {required}, current {current}")]
    InsufficientComplianceLevel { required: String, current: String },

    /// Risk level too high
    #[error("Risk level too high: {risk_level} - {message}")]
    RiskLevelTooHigh { risk_level: String, message: String },

    /// Document verification errors
    #[error("Document verification error: {document_type} - {message}")]
    DocumentVerificationError { document_type: String, message: String },

    /// Expired compliance
    #[error("Compliance expired: {expired_at}")]
    ComplianceExpired { expired_at: String },

    /// Unsupported jurisdiction
    #[error("Unsupported jurisdiction: {jurisdiction}")]
    UnsupportedJurisdiction { jurisdiction: String },

    /// Internal system errors
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Generic compliance error
    #[error("Compliance error: {0}")]
    Generic(String),
}

impl ComplianceError {
    /// Create a KYC error
    pub fn kyc_error<S: Into<String>>(message: S) -> Self {
        Self::KycError {
            message: message.into(),
        }
    }

    /// Create an AML error
    pub fn aml_error<S: Into<String>>(message: S) -> Self {
        Self::AmlError {
            message: message.into(),
        }
    }

    /// Create a jurisdiction error
    pub fn jurisdiction_error<S: Into<String>>(jurisdiction: S, message: S) -> Self {
        Self::JurisdictionError {
            jurisdiction: jurisdiction.into(),
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

    /// Create an insufficient compliance level error
    pub fn insufficient_compliance_level<S: Into<String>>(required: S, current: S) -> Self {
        Self::InsufficientComplianceLevel {
            required: required.into(),
            current: current.into(),
        }
    }

    /// Create a risk level too high error
    pub fn risk_level_too_high<S: Into<String>>(risk_level: S, message: S) -> Self {
        Self::RiskLevelTooHigh {
            risk_level: risk_level.into(),
            message: message.into(),
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ComplianceError::NetworkError { .. }
                | ComplianceError::RateLimitError { .. }
                | ComplianceError::InternalError { .. }
        )
    }

    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            ComplianceError::KycError { .. } => "kyc",
            ComplianceError::AmlError { .. } => "aml",
            ComplianceError::JurisdictionError { .. } => "jurisdiction",
            ComplianceError::ReportingError { .. } => "reporting",
            ComplianceError::AuditError { .. } => "audit",
            ComplianceError::ProviderError { .. } => "provider",
            ComplianceError::ConfigurationError { .. } => "configuration",
            ComplianceError::ValidationError { .. } => "validation",
            ComplianceError::DatabaseError { .. } => "database",
            ComplianceError::NetworkError { .. } => "network",
            ComplianceError::SerializationError { .. } => "serialization",
            ComplianceError::AuthenticationError { .. } => "authentication",
            ComplianceError::RateLimitError { .. } => "rate_limit",
            ComplianceError::StatusError { .. } => "status",
            ComplianceError::InsufficientComplianceLevel { .. } => "compliance_level",
            ComplianceError::RiskLevelTooHigh { .. } => "risk_level",
            ComplianceError::DocumentVerificationError { .. } => "document_verification",
            ComplianceError::ComplianceExpired { .. } => "compliance_expired",
            ComplianceError::UnsupportedJurisdiction { .. } => "unsupported_jurisdiction",
            ComplianceError::InternalError { .. } => "internal",
            ComplianceError::Generic(_) => "generic",
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ComplianceError::InternalError { .. } => ErrorSeverity::Critical,
            ComplianceError::DatabaseError { .. } => ErrorSeverity::High,
            ComplianceError::AuthenticationError { .. } => ErrorSeverity::High,
            ComplianceError::RiskLevelTooHigh { .. } => ErrorSeverity::High,
            ComplianceError::ComplianceExpired { .. } => ErrorSeverity::Medium,
            ComplianceError::InsufficientComplianceLevel { .. } => ErrorSeverity::Medium,
            ComplianceError::ValidationError { .. } => ErrorSeverity::Medium,
            ComplianceError::NetworkError { .. } => ErrorSeverity::Low,
            ComplianceError::RateLimitError { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
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
impl From<serde_json::Error> for ComplianceError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for ComplianceError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "database")]
impl From<sqlx::Error> for ComplianceError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<validator::ValidationErrors> for ComplianceError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError {
            field: "multiple".to_string(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let kyc_error = ComplianceError::kyc_error("Invalid document");
        assert!(matches!(kyc_error, ComplianceError::KycError { .. }));
        assert_eq!(kyc_error.category(), "kyc");
    }

    #[test]
    fn test_error_retryability() {
        let network_error = ComplianceError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(network_error.is_retryable());

        let validation_error = ComplianceError::ValidationError {
            field: "email".to_string(),
            message: "Invalid format".to_string(),
        };
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let internal_error = ComplianceError::InternalError {
            message: "System failure".to_string(),
        };
        assert_eq!(internal_error.severity(), ErrorSeverity::Critical);

        let network_error = ComplianceError::NetworkError {
            message: "Timeout".to_string(),
        };
        assert_eq!(network_error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            ComplianceError::kyc_error("test"),
            ComplianceError::aml_error("test"),
            ComplianceError::validation_error("field", "message"),
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(categories, vec!["kyc", "aml", "validation"]);
    }
}
