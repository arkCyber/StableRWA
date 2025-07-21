// =====================================================================================
// File: core-asset-lifecycle/src/error.rs
// Description: Error types for asset lifecycle management
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type alias for asset operations
pub type AssetResult<T> = Result<T, AssetError>;

/// Comprehensive error types for asset lifecycle operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AssetError {
    /// Asset registration errors
    #[error("Registration error: {message}")]
    RegistrationError { message: String },

    /// Asset verification errors
    #[error("Verification error: {message}")]
    VerificationError { message: String },

    /// Asset valuation errors
    #[error("Valuation error: {message}")]
    ValuationError { message: String },

    /// Asset maintenance errors
    #[error("Maintenance error: {message}")]
    MaintenanceError { message: String },

    /// Tokenization errors
    #[error("Tokenization error: {message}")]
    TokenizationError { message: String },

    /// Custody errors
    #[error("Custody error: {message}")]
    CustodyError { message: String },

    /// Asset not found
    #[error("Asset not found: {asset_id}")]
    AssetNotFound { asset_id: String },

    /// Invalid asset status for operation
    #[error("Invalid asset status: {current_status} for operation {operation}")]
    InvalidAssetStatus {
        current_status: String,
        operation: String,
    },

    /// Invalid lifecycle stage for operation
    #[error("Invalid lifecycle stage: {current_stage} for operation {operation}")]
    InvalidLifecycleStage {
        current_stage: String,
        operation: String,
    },

    /// Ownership errors
    #[error("Ownership error: {message}")]
    OwnershipError { message: String },

    /// Document errors
    #[error("Document error: {document_type} - {message}")]
    DocumentError {
        document_type: String,
        message: String,
    },

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

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

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// File system errors
    #[error("File system error: {message}")]
    FileSystemError { message: String },

    /// Insufficient permissions
    #[error("Insufficient permissions: {required_permission} for user {user_id}")]
    InsufficientPermissions {
        user_id: String,
        required_permission: String,
    },

    /// Concurrent modification error
    #[error("Concurrent modification detected for asset {asset_id}")]
    ConcurrentModification { asset_id: String },

    /// Business rule violation
    #[error("Business rule violation: {rule} - {message}")]
    BusinessRuleViolation { rule: String, message: String },

    /// Compliance error
    #[error("Compliance error: {message}")]
    ComplianceError { message: String },

    /// Internal system errors
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Generic asset error
    #[error("Asset error: {0}")]
    Generic(String),
}

impl AssetError {
    /// Create a registration error
    pub fn registration_error<S: Into<String>>(message: S) -> Self {
        Self::RegistrationError {
            message: message.into(),
        }
    }

    /// Create a verification error
    pub fn verification_error<S: Into<String>>(message: S) -> Self {
        Self::VerificationError {
            message: message.into(),
        }
    }

    /// Create a valuation error
    pub fn valuation_error<S: Into<String>>(message: S) -> Self {
        Self::ValuationError {
            message: message.into(),
        }
    }

    /// Create a maintenance error
    pub fn maintenance_error<S: Into<String>>(message: S) -> Self {
        Self::MaintenanceError {
            message: message.into(),
        }
    }

    /// Create a tokenization error
    pub fn tokenization_error<S: Into<String>>(message: S) -> Self {
        Self::TokenizationError {
            message: message.into(),
        }
    }

    /// Create an asset not found error
    pub fn asset_not_found<S: Into<String>>(asset_id: S) -> Self {
        Self::AssetNotFound {
            asset_id: asset_id.into(),
        }
    }

    /// Create an invalid asset status error
    pub fn invalid_asset_status<S: Into<String>>(current_status: S, operation: S) -> Self {
        Self::InvalidAssetStatus {
            current_status: current_status.into(),
            operation: operation.into(),
        }
    }

    /// Create an invalid lifecycle stage error
    pub fn invalid_lifecycle_stage<S: Into<String>>(current_stage: S, operation: S) -> Self {
        Self::InvalidLifecycleStage {
            current_stage: current_stage.into(),
            operation: operation.into(),
        }
    }

    /// Create an ownership error
    pub fn ownership_error<S: Into<String>>(message: S) -> Self {
        Self::OwnershipError {
            message: message.into(),
        }
    }

    /// Create a document error
    pub fn document_error<S: Into<String>>(document_type: S, message: S) -> Self {
        Self::DocumentError {
            document_type: document_type.into(),
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

    /// Create an external service error
    pub fn external_service_error<S: Into<String>>(service: S, message: S) -> Self {
        Self::ExternalServiceError {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create an insufficient permissions error
    pub fn insufficient_permissions<S: Into<String>>(user_id: S, required_permission: S) -> Self {
        Self::InsufficientPermissions {
            user_id: user_id.into(),
            required_permission: required_permission.into(),
        }
    }

    /// Create a business rule violation error
    pub fn business_rule_violation<S: Into<String>>(rule: S, message: S) -> Self {
        Self::BusinessRuleViolation {
            rule: rule.into(),
            message: message.into(),
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AssetError::NetworkError { .. }
                | AssetError::DatabaseError { .. }
                | AssetError::ExternalServiceError { .. }
                | AssetError::InternalError { .. }
        )
    }

    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            AssetError::RegistrationError { .. } => "registration",
            AssetError::VerificationError { .. } => "verification",
            AssetError::ValuationError { .. } => "valuation",
            AssetError::MaintenanceError { .. } => "maintenance",
            AssetError::TokenizationError { .. } => "tokenization",
            AssetError::CustodyError { .. } => "custody",
            AssetError::AssetNotFound { .. } => "not_found",
            AssetError::InvalidAssetStatus { .. } => "invalid_status",
            AssetError::InvalidLifecycleStage { .. } => "invalid_stage",
            AssetError::OwnershipError { .. } => "ownership",
            AssetError::DocumentError { .. } => "document",
            AssetError::ValidationError { .. } => "validation",
            AssetError::ExternalServiceError { .. } => "external_service",
            AssetError::DatabaseError { .. } => "database",
            AssetError::NetworkError { .. } => "network",
            AssetError::SerializationError { .. } => "serialization",
            AssetError::AuthenticationError { .. } => "authentication",
            AssetError::ConfigurationError { .. } => "configuration",
            AssetError::FileSystemError { .. } => "filesystem",
            AssetError::InsufficientPermissions { .. } => "permissions",
            AssetError::ConcurrentModification { .. } => "concurrency",
            AssetError::BusinessRuleViolation { .. } => "business_rule",
            AssetError::ComplianceError { .. } => "compliance",
            AssetError::InternalError { .. } => "internal",
            AssetError::Generic(_) => "generic",
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AssetError::InternalError { .. } => ErrorSeverity::Critical,
            AssetError::DatabaseError { .. } => ErrorSeverity::High,
            AssetError::AuthenticationError { .. } => ErrorSeverity::High,
            AssetError::ComplianceError { .. } => ErrorSeverity::High,
            AssetError::BusinessRuleViolation { .. } => ErrorSeverity::High,
            AssetError::ConcurrentModification { .. } => ErrorSeverity::Medium,
            AssetError::InvalidAssetStatus { .. } => ErrorSeverity::Medium,
            AssetError::InvalidLifecycleStage { .. } => ErrorSeverity::Medium,
            AssetError::ValidationError { .. } => ErrorSeverity::Medium,
            AssetError::AssetNotFound { .. } => ErrorSeverity::Low,
            AssetError::NetworkError { .. } => ErrorSeverity::Low,
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
impl From<serde_json::Error> for AssetError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for AssetError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "database")]
impl From<sqlx::Error> for AssetError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for AssetError {
    fn from(err: std::io::Error) -> Self {
        Self::FileSystemError {
            message: err.to_string(),
        }
    }
}

impl From<validator::ValidationErrors> for AssetError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError {
            field: "multiple".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<core_compliance::error::ComplianceError> for AssetError {
    fn from(err: core_compliance::error::ComplianceError) -> Self {
        Self::ComplianceError {
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let registration_error = AssetError::registration_error("Invalid asset data");
        assert!(matches!(registration_error, AssetError::RegistrationError { .. }));
        assert_eq!(registration_error.category(), "registration");
    }

    #[test]
    fn test_error_retryability() {
        let network_error = AssetError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(network_error.is_retryable());

        let validation_error = AssetError::ValidationError {
            field: "name".to_string(),
            message: "Name is required".to_string(),
        };
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let internal_error = AssetError::InternalError {
            message: "System failure".to_string(),
        };
        assert_eq!(internal_error.severity(), ErrorSeverity::Critical);
        assert!(internal_error.requires_immediate_attention());

        let not_found_error = AssetError::AssetNotFound {
            asset_id: "asset123".to_string(),
        };
        assert_eq!(not_found_error.severity(), ErrorSeverity::Low);
        assert!(!not_found_error.requires_immediate_attention());
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            AssetError::registration_error("test"),
            AssetError::verification_error("test"),
            AssetError::valuation_error("test"),
            AssetError::asset_not_found("test"),
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(categories, vec!["registration", "verification", "valuation", "not_found"]);
    }

    #[test]
    fn test_specific_error_constructors() {
        let invalid_status = AssetError::invalid_asset_status("Registering", "tokenize");
        assert!(matches!(invalid_status, AssetError::InvalidAssetStatus { .. }));

        let insufficient_perms = AssetError::insufficient_permissions("user123", "asset:write");
        assert!(matches!(insufficient_perms, AssetError::InsufficientPermissions { .. }));

        let business_rule = AssetError::business_rule_violation("max_valuation", "Exceeds limit");
        assert!(matches!(business_rule, AssetError::BusinessRuleViolation { .. }));
    }
}
