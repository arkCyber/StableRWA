// =====================================================================================
// File: core-compliance/src/error.rs
// Description: Error types for compliance module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    JurisdictionError {
        jurisdiction: String,
        message: String,
    },

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
    DocumentVerificationError {
        document_type: String,
        message: String,
    },

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

    /// Create a document verification error
    pub fn document_verification_error<S: Into<String>>(document_type: S, message: S) -> Self {
        Self::DocumentVerificationError {
            document_type: document_type.into(),
            message: message.into(),
        }
    }

    /// Create a reporting error
    pub fn reporting_error<S: Into<String>>(message: S) -> Self {
        Self::ReportingError {
            message: message.into(),
        }
    }

    /// Create an audit error
    pub fn audit_error<S: Into<String>>(message: S) -> Self {
        Self::AuditError {
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

    /// Create a serialization error
    pub fn serialization_error<S: Into<String>>(message: S) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication_error<S: Into<String>>(message: S) -> Self {
        Self::AuthenticationError {
            message: message.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limit_error<S: Into<String>>(message: S) -> Self {
        Self::RateLimitError {
            message: message.into(),
        }
    }

    /// Create a status error
    pub fn status_error<S: Into<String>>(status: S, message: S) -> Self {
        Self::StatusError {
            status: status.into(),
            message: message.into(),
        }
    }

    /// Create a compliance expired error
    pub fn compliance_expired<S: Into<String>>(expired_at: S) -> Self {
        Self::ComplianceExpired {
            expired_at: expired_at.into(),
        }
    }

    /// Create an unsupported jurisdiction error
    pub fn unsupported_jurisdiction<S: Into<String>>(jurisdiction: S) -> Self {
        Self::UnsupportedJurisdiction {
            jurisdiction: jurisdiction.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        Self::InternalError {
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

    #[test]
    fn test_error_helper_methods() {
        let doc_error = ComplianceError::document_verification_error("passport", "Invalid document");
        assert!(matches!(doc_error, ComplianceError::DocumentVerificationError { .. }));
        assert_eq!(doc_error.category(), "document_verification");

        let reporting_error = ComplianceError::reporting_error("Failed to generate report");
        assert!(matches!(reporting_error, ComplianceError::ReportingError { .. }));
        assert_eq!(reporting_error.category(), "reporting");

        let audit_error = ComplianceError::audit_error("Audit trail corrupted");
        assert!(matches!(audit_error, ComplianceError::AuditError { .. }));
        assert_eq!(audit_error.category(), "audit");
    }

    #[test]
    fn test_error_severity_levels() {
        let critical_error = ComplianceError::internal_error("System failure");
        assert_eq!(critical_error.severity(), ErrorSeverity::Critical);

        let high_error = ComplianceError::database_error("Connection failed");
        assert_eq!(high_error.severity(), ErrorSeverity::High);

        let medium_error = ComplianceError::validation_error("field", "Invalid value");
        assert_eq!(medium_error.severity(), ErrorSeverity::Medium);

        let low_error = ComplianceError::network_error("Timeout");
        assert_eq!(low_error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_serialization() {
        let error = ComplianceError::kyc_error("Test error");
        let json = serde_json::to_string(&error).expect("Failed to serialize");
        let deserialized: ComplianceError = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(error.category(), deserialized.category());
    }

    #[test]
    fn test_compliance_level_error() {
        let error = ComplianceError::insufficient_compliance_level("Premium", "Basic");
        assert!(matches!(error, ComplianceError::InsufficientComplianceLevel { .. }));
        assert_eq!(error.category(), "compliance_level");
        assert_eq!(error.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_risk_level_error() {
        let error = ComplianceError::risk_level_too_high("Critical", "User poses high risk");
        assert!(matches!(error, ComplianceError::RiskLevelTooHigh { .. }));
        assert_eq!(error.category(), "risk_level");
        assert_eq!(error.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_jurisdiction_error() {
        let error = ComplianceError::jurisdiction_error("US", "Restricted jurisdiction");
        assert!(matches!(error, ComplianceError::JurisdictionError { .. }));
        assert_eq!(error.category(), "jurisdiction");
    }

    #[test]
    fn test_provider_error() {
        let error = ComplianceError::provider_error("jumio", "API rate limit exceeded");
        assert!(matches!(error, ComplianceError::ProviderError { .. }));
        assert_eq!(error.category(), "provider");
    }

    #[test]
    fn test_error_from_conversions() {
        // Test serde_json::Error conversion
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());
        let compliance_error: ComplianceError = json_error.unwrap_err().into();
        assert!(matches!(compliance_error, ComplianceError::SerializationError { .. }));

        // Test reqwest::Error conversion would require actual HTTP client
        // So we'll test the pattern instead
        let network_error = ComplianceError::network_error("Connection failed");
        assert!(matches!(network_error, ComplianceError::NetworkError { .. }));
    }

    #[test]
    fn test_error_display() {
        let kyc_error = ComplianceError::kyc_error("Document verification failed");
        let error_string = format!("{}", kyc_error);
        assert!(error_string.contains("KYC error"));
        assert!(error_string.contains("Document verification failed"));

        let validation_error = ComplianceError::validation_error("email", "Invalid format");
        let validation_string = format!("{}", validation_error);
        assert!(validation_string.contains("Validation error"));
        assert!(validation_string.contains("email"));
        assert!(validation_string.contains("Invalid format"));
    }

    #[test]
    fn test_error_debug() {
        let error = ComplianceError::aml_error("Sanctions list match");
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("AmlError"));
        assert!(debug_string.contains("Sanctions list match"));
    }

    #[test]
    fn test_configuration_error() {
        let error = ComplianceError::configuration_error("Missing API key");
        assert!(matches!(error, ComplianceError::ConfigurationError { .. }));
        assert_eq!(error.category(), "configuration");
    }

    #[test]
    fn test_authentication_error() {
        let error = ComplianceError::authentication_error("Invalid credentials");
        assert!(matches!(error, ComplianceError::AuthenticationError { .. }));
        assert_eq!(error.category(), "authentication");
        assert_eq!(error.severity(), ErrorSeverity::High);
    }

    #[test]
    fn test_rate_limit_error() {
        let error = ComplianceError::rate_limit_error("Too many requests");
        assert!(matches!(error, ComplianceError::RateLimitError { .. }));
        assert_eq!(error.category(), "rate_limit");
        assert_eq!(error.severity(), ErrorSeverity::Low);
        assert!(error.is_retryable());
    }

    #[test]
    fn test_compliance_expired_error() {
        let error = ComplianceError::compliance_expired("2024-01-01T00:00:00Z");
        assert!(matches!(error, ComplianceError::ComplianceExpired { .. }));
        assert_eq!(error.category(), "compliance_expired");
    }

    #[test]
    fn test_unsupported_jurisdiction_error() {
        let error = ComplianceError::unsupported_jurisdiction("XX");
        assert!(matches!(error, ComplianceError::UnsupportedJurisdiction { .. }));
        assert_eq!(error.category(), "unsupported_jurisdiction");
    }

    #[test]
    fn test_status_error() {
        let error = ComplianceError::status_error("REJECTED", "Compliance check failed");
        assert!(matches!(error, ComplianceError::StatusError { .. }));
        assert_eq!(error.category(), "status");
    }
}
