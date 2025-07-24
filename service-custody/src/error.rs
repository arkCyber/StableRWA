// =====================================================================================
// RWA Tokenization Platform - Custody Service Error Types
// 
// Comprehensive error handling for custody operations including digital asset custody,
// physical asset custody, proof systems, and insurance integration.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use std::fmt;
use thiserror::Error;

/// Result type alias for custody operations
pub type CustodyResult<T> = Result<T, CustodyError>;

/// Comprehensive error types for custody service operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CustodyError {
    /// Configuration-related errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Database operation errors
    #[error("Database error: {operation} failed - {message}")]
    Database { operation: String, message: String },

    /// Cryptographic operation errors
    #[error("Cryptographic error: {operation} failed - {message}")]
    Cryptographic { operation: String, message: String },

    /// Key management errors
    #[error("Key management error: {operation} failed - {message}")]
    KeyManagement { operation: String, message: String },

    /// Multi-signature wallet errors
    #[error("Multi-signature error: {operation} failed - {message}")]
    MultiSignature { operation: String, message: String },

    /// Hardware Security Module (HSM) errors
    #[error("HSM error: {operation} failed - {message}")]
    Hsm { operation: String, message: String },

    /// Physical custody integration errors
    #[error("Physical custody error: {provider} - {message}")]
    PhysicalCustody { provider: String, message: String },

    /// Proof system errors
    #[error("Proof system error: {proof_type} - {message}")]
    ProofSystem { proof_type: String, message: String },

    /// Insurance integration errors
    #[error("Insurance error: {provider} - {message}")]
    Insurance { provider: String, message: String },

    /// Asset validation errors
    #[error("Asset validation error: {asset_id} - {message}")]
    AssetValidation { asset_id: String, message: String },

    /// Authorization and access control errors
    #[error("Authorization error: {operation} - {message}")]
    Authorization { operation: String, message: String },

    /// Network and communication errors
    #[error("Network error: {endpoint} - {message}")]
    Network { endpoint: String, message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {format} - {message}")]
    Serialization { format: String, message: String },

    /// Validation errors for input data
    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {operation} - {message}")]
    RateLimit { operation: String, message: String },

    /// Service unavailable errors
    #[error("Service unavailable: {service} - {message}")]
    ServiceUnavailable { service: String, message: String },

    /// Internal server errors
    #[error("Internal error: {message}")]
    Internal { message: String },

    /// External service integration errors
    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },
}

impl CustodyError {
    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a database error
    pub fn database<S: Into<String>>(operation: S, message: S) -> Self {
        Self::Database {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a cryptographic error
    pub fn cryptographic<S: Into<String>>(operation: S, message: S) -> Self {
        Self::Cryptographic {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a key management error
    pub fn key_management<S: Into<String>>(operation: S, message: S) -> Self {
        Self::KeyManagement {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a multi-signature error
    pub fn multi_signature<S: Into<String>>(operation: S, message: S) -> Self {
        Self::MultiSignature {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create an HSM error
    pub fn hsm<S: Into<String>>(operation: S, message: S) -> Self {
        Self::Hsm {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a physical custody error
    pub fn physical_custody<S: Into<String>>(provider: S, message: S) -> Self {
        Self::PhysicalCustody {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a proof system error
    pub fn proof_system<S: Into<String>>(proof_type: S, message: S) -> Self {
        Self::ProofSystem {
            proof_type: proof_type.into(),
            message: message.into(),
        }
    }

    /// Create an insurance error
    pub fn insurance<S: Into<String>>(provider: S, message: S) -> Self {
        Self::Insurance {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create an asset validation error
    pub fn asset_validation<S: Into<String>>(asset_id: S, message: S) -> Self {
        Self::AssetValidation {
            asset_id: asset_id.into(),
            message: message.into(),
        }
    }

    /// Create an authorization error
    pub fn authorization<S: Into<String>>(operation: S, message: S) -> Self {
        Self::Authorization {
            operation: operation.into(),
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network<S: Into<String>>(endpoint: S, message: S) -> Self {
        Self::Network {
            endpoint: endpoint.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(field: S, message: S) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Get the error category for metrics and logging
    pub fn category(&self) -> &'static str {
        match self {
            Self::Configuration { .. } => "configuration",
            Self::Database { .. } => "database",
            Self::Cryptographic { .. } => "cryptographic",
            Self::KeyManagement { .. } => "key_management",
            Self::MultiSignature { .. } => "multi_signature",
            Self::Hsm { .. } => "hsm",
            Self::PhysicalCustody { .. } => "physical_custody",
            Self::ProofSystem { .. } => "proof_system",
            Self::Insurance { .. } => "insurance",
            Self::AssetValidation { .. } => "asset_validation",
            Self::Authorization { .. } => "authorization",
            Self::Network { .. } => "network",
            Self::Serialization { .. } => "serialization",
            Self::Validation { .. } => "validation",
            Self::RateLimit { .. } => "rate_limit",
            Self::ServiceUnavailable { .. } => "service_unavailable",
            Self::Internal { .. } => "internal",
            Self::ExternalService { .. } => "external_service",
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network { .. }
                | Self::ServiceUnavailable { .. }
                | Self::RateLimit { .. }
                | Self::ExternalService { .. }
        )
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Configuration { .. } => 500,
            Self::Database { .. } => 500,
            Self::Cryptographic { .. } => 500,
            Self::KeyManagement { .. } => 500,
            Self::MultiSignature { .. } => 400,
            Self::Hsm { .. } => 500,
            Self::PhysicalCustody { .. } => 502,
            Self::ProofSystem { .. } => 400,
            Self::Insurance { .. } => 502,
            Self::AssetValidation { .. } => 400,
            Self::Authorization { .. } => 403,
            Self::Network { .. } => 502,
            Self::Serialization { .. } => 400,
            Self::Validation { .. } => 400,
            Self::RateLimit { .. } => 429,
            Self::ServiceUnavailable { .. } => 503,
            Self::Internal { .. } => 500,
            Self::ExternalService { .. } => 502,
        }
    }
}

// Implement conversion from common error types
impl From<sqlx::Error> for CustodyError {
    fn from(err: sqlx::Error) -> Self {
        Self::database("sqlx_operation", &err.to_string())
    }
}

impl From<serde_json::Error> for CustodyError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization {
            format: "json".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for CustodyError {
    fn from(err: reqwest::Error) -> Self {
        let endpoint = err.url().map(|u| u.to_string()).unwrap_or_default();
        Self::network(endpoint, err.to_string())
    }
}

impl From<config::ConfigError> for CustodyError {
    fn from(err: config::ConfigError) -> Self {
        Self::configuration(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CustodyError::configuration("Invalid config");
        assert_eq!(err.category(), "configuration");
        assert_eq!(err.status_code(), 500);
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_retryable_errors() {
        let network_err = CustodyError::network("api.example.com", "Connection timeout");
        assert!(network_err.is_retryable());
        
        let config_err = CustodyError::configuration("Missing field");
        assert!(!config_err.is_retryable());
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            CustodyError::database("insert", "Constraint violation"),
            CustodyError::cryptographic("encrypt", "Invalid key"),
            CustodyError::key_management("generate", "HSM unavailable"),
            CustodyError::authorization("access", "Insufficient permissions"),
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(categories, vec!["database", "cryptographic", "key_management", "authorization"]);
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(CustodyError::validation("field", "Invalid").status_code(), 400);
        assert_eq!(CustodyError::authorization("op", "Denied").status_code(), 403);
        assert_eq!(CustodyError::rate_limit("api", "Exceeded").status_code(), 429);
        assert_eq!(CustodyError::internal("Error").status_code(), 500);
        assert_eq!(CustodyError::network("url", "Timeout").status_code(), 502);
        assert_eq!(CustodyError::service_unavailable("db", "Down").status_code(), 503);
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_err.is_err());
        
        let custody_err: CustodyError = json_err.unwrap_err().into();
        assert_eq!(custody_err.category(), "serialization");
    }
}
