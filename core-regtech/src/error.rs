// =====================================================================================
// File: core-regtech/src/error.rs
// Description: Error types for regulatory technology operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for RegTech operations
pub type RegTechResult<T> = Result<T, RegTechError>;

/// RegTech service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum RegTechError {
    /// AML (Anti-Money Laundering) errors
    #[error("AML error: {message}")]
    AMLError { message: String },

    /// KYC (Know Your Customer) errors
    #[error("KYC error: {message}")]
    KYCError { message: String },

    /// Sanctions screening errors
    #[error("Sanctions screening error: {message}")]
    SanctionsError { message: String },

    /// Travel Rule compliance errors
    #[error("Travel Rule error: {message}")]
    TravelRuleError { message: String },

    /// Regulatory reporting errors
    #[error("Regulatory reporting error: {message}")]
    ReportingError { message: String },

    /// Document verification errors
    #[error("Document verification error: {message}")]
    DocumentVerificationError { message: String },

    /// Identity verification errors
    #[error("Identity verification error: {message}")]
    IdentityVerificationError { message: String },

    /// Risk assessment errors
    #[error("Risk assessment error: {message}")]
    RiskAssessmentError { message: String },

    /// Compliance monitoring errors
    #[error("Compliance monitoring error: {message}")]
    ComplianceMonitoringError { message: String },

    /// Audit trail errors
    #[error("Audit trail error: {message}")]
    AuditTrailError { message: String },

    /// Regulatory framework errors
    #[error("Regulatory framework error: {message}")]
    RegulatoryFrameworkError { message: String },

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

    /// Encryption errors
    #[error("Encryption error: {message}")]
    EncryptionError { message: String },

    /// Compliance violation errors
    #[error("Compliance violation: {violation_type}: {message}")]
    ComplianceViolation {
        violation_type: String,
        message: String,
    },

    /// Regulatory deadline missed
    #[error("Regulatory deadline missed: {deadline_type}")]
    DeadlineMissed { deadline_type: String },

    /// Insufficient data errors
    #[error("Insufficient data for {operation}: {message}")]
    InsufficientData { operation: String, message: String },

    /// Internal errors
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl RegTechError {
    /// Create an AML error
    pub fn aml_error<S: Into<String>>(message: S) -> Self {
        Self::AMLError {
            message: message.into(),
        }
    }

    /// Create a KYC error
    pub fn kyc_error<S: Into<String>>(message: S) -> Self {
        Self::KYCError {
            message: message.into(),
        }
    }

    /// Create a sanctions error
    pub fn sanctions_error<S: Into<String>>(message: S) -> Self {
        Self::SanctionsError {
            message: message.into(),
        }
    }

    /// Create a travel rule error
    pub fn travel_rule_error<S: Into<String>>(message: S) -> Self {
        Self::TravelRuleError {
            message: message.into(),
        }
    }

    /// Create a reporting error
    pub fn reporting_error<S: Into<String>>(message: S) -> Self {
        Self::ReportingError {
            message: message.into(),
        }
    }

    /// Create a document verification error
    pub fn document_verification_error<S: Into<String>>(message: S) -> Self {
        Self::DocumentVerificationError {
            message: message.into(),
        }
    }

    /// Create an identity verification error
    pub fn identity_verification_error<S: Into<String>>(message: S) -> Self {
        Self::IdentityVerificationError {
            message: message.into(),
        }
    }

    /// Create a risk assessment error
    pub fn risk_assessment_error<S: Into<String>>(message: S) -> Self {
        Self::RiskAssessmentError {
            message: message.into(),
        }
    }

    /// Create a compliance monitoring error
    pub fn compliance_monitoring_error<S: Into<String>>(message: S) -> Self {
        Self::ComplianceMonitoringError {
            message: message.into(),
        }
    }

    /// Create an audit trail error
    pub fn audit_trail_error<S: Into<String>>(message: S) -> Self {
        Self::AuditTrailError {
            message: message.into(),
        }
    }

    /// Create a regulatory framework error
    pub fn regulatory_framework_error<S: Into<String>>(message: S) -> Self {
        Self::RegulatoryFrameworkError {
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

    /// Create an encryption error
    pub fn encryption_error<S: Into<String>>(message: S) -> Self {
        Self::EncryptionError {
            message: message.into(),
        }
    }

    /// Create a compliance violation error
    pub fn compliance_violation<S: Into<String>>(violation_type: S, message: S) -> Self {
        Self::ComplianceViolation {
            violation_type: violation_type.into(),
            message: message.into(),
        }
    }

    /// Create a deadline missed error
    pub fn deadline_missed<S: Into<String>>(deadline_type: S) -> Self {
        Self::DeadlineMissed {
            deadline_type: deadline_type.into(),
        }
    }

    /// Create an insufficient data error
    pub fn insufficient_data<S: Into<String>>(operation: S, message: S) -> Self {
        Self::InsufficientData {
            operation: operation.into(),
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
            Self::AMLError { .. } => "AML_ERROR",
            Self::KYCError { .. } => "KYC_ERROR",
            Self::SanctionsError { .. } => "SANCTIONS_ERROR",
            Self::TravelRuleError { .. } => "TRAVEL_RULE_ERROR",
            Self::ReportingError { .. } => "REPORTING_ERROR",
            Self::DocumentVerificationError { .. } => "DOCUMENT_VERIFICATION_ERROR",
            Self::IdentityVerificationError { .. } => "IDENTITY_VERIFICATION_ERROR",
            Self::RiskAssessmentError { .. } => "RISK_ASSESSMENT_ERROR",
            Self::ComplianceMonitoringError { .. } => "COMPLIANCE_MONITORING_ERROR",
            Self::AuditTrailError { .. } => "AUDIT_TRAIL_ERROR",
            Self::RegulatoryFrameworkError { .. } => "REGULATORY_FRAMEWORK_ERROR",
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
            Self::EncryptionError { .. } => "ENCRYPTION_ERROR",
            Self::ComplianceViolation { .. } => "COMPLIANCE_VIOLATION",
            Self::DeadlineMissed { .. } => "DEADLINE_MISSED",
            Self::InsufficientData { .. } => "INSUFFICIENT_DATA",
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
        )
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::ComplianceViolation { .. }
                | Self::DeadlineMissed { .. }
                | Self::AMLError { .. }
                | Self::SanctionsError { .. }
                | Self::InternalError { .. }
        )
    }

    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(
            self,
            Self::ComplianceViolation { .. }
                | Self::SanctionsError { .. }
                | Self::AMLError { .. }
                | Self::DeadlineMissed { .. }
        )
    }

    /// Check if error is compliance-related
    pub fn is_compliance_related(&self) -> bool {
        matches!(
            self,
            Self::AMLError { .. }
                | Self::KYCError { .. }
                | Self::SanctionsError { .. }
                | Self::TravelRuleError { .. }
                | Self::ComplianceViolation { .. }
                | Self::ComplianceMonitoringError { .. }
                | Self::DeadlineMissed { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = RegTechError::aml_error("Test AML error");
        assert_eq!(error.error_code(), "AML_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(error.is_compliance_related());
    }

    #[test]
    fn test_kyc_error() {
        let error = RegTechError::kyc_error("Invalid document");
        assert_eq!(error.error_code(), "KYC_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(error.is_compliance_related());
    }

    #[test]
    fn test_sanctions_error() {
        let error = RegTechError::sanctions_error("Watchlist match found");
        assert_eq!(error.error_code(), "SANCTIONS_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(error.is_compliance_related());
    }

    #[test]
    fn test_retryable_error() {
        let error = RegTechError::network_error("Connection failed");
        assert_eq!(error.error_code(), "NETWORK_ERROR");
        assert!(error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_compliance_related());
    }

    #[test]
    fn test_compliance_violation() {
        let error = RegTechError::compliance_violation("AML", "Suspicious transaction pattern");
        assert_eq!(error.error_code(), "COMPLIANCE_VIOLATION");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(error.is_compliance_related());
    }

    #[test]
    fn test_data_validation_error() {
        let error = RegTechError::data_validation_error("email", "Invalid format");
        assert_eq!(error.error_code(), "DATA_VALIDATION_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_compliance_related());
    }
}
