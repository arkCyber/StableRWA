// =====================================================================================
// File: core-smart-contract/src/error.rs
// Description: Error types for smart contract management operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for smart contract operations
pub type SmartContractResult<T> = Result<T, SmartContractError>;

/// Smart contract service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SmartContractError {
    /// Compilation errors
    #[error("Compilation error: {message}")]
    CompilationError { message: String },

    /// Deployment errors
    #[error("Deployment error: {contract}: {message}")]
    DeploymentError { contract: String, message: String },

    /// Upgrade errors
    #[error("Upgrade error: {contract}: {message}")]
    UpgradeError { contract: String, message: String },

    /// Proxy errors
    #[error("Proxy error: {proxy_type}: {message}")]
    ProxyError { proxy_type: String, message: String },

    /// Verification errors
    #[error("Verification error: {contract}: {message}")]
    VerificationError { contract: String, message: String },

    /// Audit errors
    #[error("Audit error: {message}")]
    AuditError { message: String },

    /// Monitoring errors
    #[error("Monitoring error: {contract}: {message}")]
    MonitoringError { contract: String, message: String },

    /// Gas optimization errors
    #[error("Gas optimization error: {message}")]
    GasOptimizationError { message: String },

    /// Registry errors
    #[error("Registry error: {message}")]
    RegistryError { message: String },

    /// Template errors
    #[error("Template error: {template}: {message}")]
    TemplateError { template: String, message: String },

    /// Network errors
    #[error("Network error: {network}: {message}")]
    NetworkError { network: String, message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Validation errors
    #[error("Validation error: {field}: {message}")]
    ValidationError { field: String, message: String },

    /// Permission errors
    #[error("Permission error: {operation}: {message}")]
    PermissionError { operation: String, message: String },

    /// State errors
    #[error("State error: {contract}: expected {expected}, found {actual}")]
    StateError {
        contract: String,
        expected: String,
        actual: String,
    },

    /// Version errors
    #[error("Version error: {contract}: {message}")]
    VersionError { contract: String, message: String },

    /// Dependency errors
    #[error("Dependency error: {dependency}: {message}")]
    DependencyError { dependency: String, message: String },

    /// Security errors
    #[error("Security error: {vulnerability}: {message}")]
    SecurityError {
        vulnerability: String,
        message: String,
    },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Insufficient funds errors
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },

    /// Contract not found errors
    #[error("Contract not found: {address}")]
    ContractNotFound { address: String },

    /// ABI errors
    #[error("ABI error: {message}")]
    AbiError { message: String },

    /// Bytecode errors
    #[error("Bytecode error: {message}")]
    BytecodeError { message: String },

    /// Storage errors
    #[error("Storage error: {message}")]
    StorageError { message: String },

    /// Database errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// External service errors
    #[error("External service error: {service}: {message}")]
    ExternalServiceError { service: String, message: String },

    /// Internal errors
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl SmartContractError {
    /// Create a compilation error
    pub fn compilation_error<S: Into<String>>(message: S) -> Self {
        Self::CompilationError {
            message: message.into(),
        }
    }

    /// Create a deployment error
    pub fn deployment_error<S: Into<String>>(contract: S, message: S) -> Self {
        Self::DeploymentError {
            contract: contract.into(),
            message: message.into(),
        }
    }

    /// Create an upgrade error
    pub fn upgrade_error<S: Into<String>>(contract: S, message: S) -> Self {
        Self::UpgradeError {
            contract: contract.into(),
            message: message.into(),
        }
    }

    /// Create a proxy error
    pub fn proxy_error<S: Into<String>>(proxy_type: S, message: S) -> Self {
        Self::ProxyError {
            proxy_type: proxy_type.into(),
            message: message.into(),
        }
    }

    /// Create a verification error
    pub fn verification_error<S: Into<String>>(contract: S, message: S) -> Self {
        Self::VerificationError {
            contract: contract.into(),
            message: message.into(),
        }
    }

    /// Create an audit error
    pub fn audit_error<S: Into<String>>(message: S) -> Self {
        Self::AuditError {
            message: message.into(),
        }
    }

    /// Create a monitoring error
    pub fn monitoring_error<S: Into<String>>(contract: S, message: S) -> Self {
        Self::MonitoringError {
            contract: contract.into(),
            message: message.into(),
        }
    }

    /// Create a gas optimization error
    pub fn gas_optimization_error<S: Into<String>>(message: S) -> Self {
        Self::GasOptimizationError {
            message: message.into(),
        }
    }

    /// Create a registry error
    pub fn registry_error<S: Into<String>>(message: S) -> Self {
        Self::RegistryError {
            message: message.into(),
        }
    }

    /// Create a template error
    pub fn template_error<S: Into<String>>(template: S, message: S) -> Self {
        Self::TemplateError {
            template: template.into(),
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error<S: Into<String>>(network: S, message: S) -> Self {
        Self::NetworkError {
            network: network.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigurationError {
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

    /// Create a permission error
    pub fn permission_error<S: Into<String>>(operation: S, message: S) -> Self {
        Self::PermissionError {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a state error
    pub fn state_error<S: Into<String>>(contract: S, expected: S, actual: S) -> Self {
        Self::StateError {
            contract: contract.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a version error
    pub fn version_error<S: Into<String>>(contract: S, message: S) -> Self {
        Self::VersionError {
            contract: contract.into(),
            message: message.into(),
        }
    }

    /// Create a dependency error
    pub fn dependency_error<S: Into<String>>(dependency: S, message: S) -> Self {
        Self::DependencyError {
            dependency: dependency.into(),
            message: message.into(),
        }
    }

    /// Create a security error
    pub fn security_error<S: Into<String>>(vulnerability: S, message: S) -> Self {
        Self::SecurityError {
            vulnerability: vulnerability.into(),
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create an insufficient funds error
    pub fn insufficient_funds<S: Into<String>>(required: S, available: S) -> Self {
        Self::InsufficientFunds {
            required: required.into(),
            available: available.into(),
        }
    }

    /// Create a contract not found error
    pub fn contract_not_found<S: Into<String>>(address: S) -> Self {
        Self::ContractNotFound {
            address: address.into(),
        }
    }

    /// Create an ABI error
    pub fn abi_error<S: Into<String>>(message: S) -> Self {
        Self::AbiError {
            message: message.into(),
        }
    }

    /// Create a bytecode error
    pub fn bytecode_error<S: Into<String>>(message: S) -> Self {
        Self::BytecodeError {
            message: message.into(),
        }
    }

    /// Create a storage error
    pub fn storage_error<S: Into<String>>(message: S) -> Self {
        Self::StorageError {
            message: message.into(),
        }
    }

    /// Create a database error
    pub fn database_error<S: Into<String>>(message: S) -> Self {
        Self::DatabaseError {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error<S: Into<String>>(message: S) -> Self {
        Self::SerializationError {
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

    /// Create an internal error
    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Get error code for categorization
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::CompilationError { .. } => "COMPILATION_ERROR",
            Self::DeploymentError { .. } => "DEPLOYMENT_ERROR",
            Self::UpgradeError { .. } => "UPGRADE_ERROR",
            Self::ProxyError { .. } => "PROXY_ERROR",
            Self::VerificationError { .. } => "VERIFICATION_ERROR",
            Self::AuditError { .. } => "AUDIT_ERROR",
            Self::MonitoringError { .. } => "MONITORING_ERROR",
            Self::GasOptimizationError { .. } => "GAS_OPTIMIZATION_ERROR",
            Self::RegistryError { .. } => "REGISTRY_ERROR",
            Self::TemplateError { .. } => "TEMPLATE_ERROR",
            Self::NetworkError { .. } => "NETWORK_ERROR",
            Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            Self::ValidationError { .. } => "VALIDATION_ERROR",
            Self::PermissionError { .. } => "PERMISSION_ERROR",
            Self::StateError { .. } => "STATE_ERROR",
            Self::VersionError { .. } => "VERSION_ERROR",
            Self::DependencyError { .. } => "DEPENDENCY_ERROR",
            Self::SecurityError { .. } => "SECURITY_ERROR",
            Self::Timeout { .. } => "TIMEOUT",
            Self::InsufficientFunds { .. } => "INSUFFICIENT_FUNDS",
            Self::ContractNotFound { .. } => "CONTRACT_NOT_FOUND",
            Self::AbiError { .. } => "ABI_ERROR",
            Self::BytecodeError { .. } => "BYTECODE_ERROR",
            Self::StorageError { .. } => "STORAGE_ERROR",
            Self::DatabaseError { .. } => "DATABASE_ERROR",
            Self::SerializationError { .. } => "SERIALIZATION_ERROR",
            Self::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            Self::InternalError { .. } => "INTERNAL_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError { .. }
                | Self::Timeout { .. }
                | Self::ExternalServiceError { .. }
                | Self::DatabaseError { .. }
                | Self::StorageError { .. }
        )
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::SecurityError { .. }
                | Self::UpgradeError { .. }
                | Self::AuditError { .. }
                | Self::InternalError { .. }
        )
    }

    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(
            self,
            Self::SecurityError { .. }
                | Self::AuditError { .. }
                | Self::MonitoringError { .. }
                | Self::InternalError { .. }
        )
    }

    /// Check if error is deployment-related
    pub fn is_deployment_related(&self) -> bool {
        matches!(
            self,
            Self::CompilationError { .. }
                | Self::DeploymentError { .. }
                | Self::VerificationError { .. }
                | Self::GasOptimizationError { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = SmartContractError::compilation_error("Test compilation error");
        assert_eq!(error.error_code(), "COMPILATION_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(error.is_deployment_related());
    }

    #[test]
    fn test_security_error() {
        let error =
            SmartContractError::security_error("reentrancy", "Potential reentrancy vulnerability");
        assert_eq!(error.error_code(), "SECURITY_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.requires_immediate_attention());
        assert!(!error.is_deployment_related());
    }

    #[test]
    fn test_network_error() {
        let error = SmartContractError::network_error("ethereum", "Connection failed");
        assert_eq!(error.error_code(), "NETWORK_ERROR");
        assert!(error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_deployment_related());
    }

    #[test]
    fn test_state_error() {
        let error = SmartContractError::state_error("MyContract", "Deployed", "Undeployed");
        assert_eq!(error.error_code(), "STATE_ERROR");
        assert!(!error.is_retryable());
        assert!(!error.is_critical());
        assert!(!error.requires_immediate_attention());
        assert!(!error.is_deployment_related());
    }
}
