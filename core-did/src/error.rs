// =====================================================================================
// DID Error Types
// 
// Comprehensive error handling for DID operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// DID operation errors
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum DidError {
    /// Invalid DID format
    #[error("Invalid DID format: {0}")]
    InvalidDidFormat(String),

    /// Invalid DID method
    #[error("Invalid DID method: {0}")]
    InvalidDidMethod(String),

    /// DID not found
    #[error("DID not found: {0}")]
    DidNotFound(String),

    /// DID already exists
    #[error("DID already exists: {0}")]
    DidAlreadyExists(String),

    /// DID deactivated
    #[error("DID is deactivated: {0}")]
    DidDeactivated(String),

    /// Invalid DID document
    #[error("Invalid DID document: {0}")]
    InvalidDidDocument(String),

    /// Invalid verification method
    #[error("Invalid verification method: {0}")]
    InvalidVerificationMethod(String),

    /// Invalid service endpoint
    #[error("Invalid service endpoint: {0}")]
    InvalidServiceEndpoint(String),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Cryptographic error
    #[error("Cryptographic error: {0}")]
    CryptographicError(String),

    /// Invalid credential
    #[error("Invalid credential: {0}")]
    InvalidCredential(String),

    /// Credential expired
    #[error("Credential expired: {0}")]
    CredentialExpired(String),

    /// Credential revoked
    #[error("Credential revoked: {0}")]
    CredentialRevoked(String),

    /// Invalid presentation
    #[error("Invalid presentation: {0}")]
    InvalidPresentation(String),

    /// Verification failed
    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    /// Resolution error
    #[error("Resolution error: {0}")]
    ResolutionError(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout error
    #[error("Operation timeout: {0}")]
    TimeoutError(String),

    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Storage error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl DidError {
    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            DidError::InvalidDidFormat(_) => "INVALID_DID_FORMAT",
            DidError::InvalidDidMethod(_) => "INVALID_DID_METHOD",
            DidError::DidNotFound(_) => "DID_NOT_FOUND",
            DidError::DidAlreadyExists(_) => "DID_ALREADY_EXISTS",
            DidError::DidDeactivated(_) => "DID_DEACTIVATED",
            DidError::InvalidDidDocument(_) => "INVALID_DID_DOCUMENT",
            DidError::InvalidVerificationMethod(_) => "INVALID_VERIFICATION_METHOD",
            DidError::InvalidServiceEndpoint(_) => "INVALID_SERVICE_ENDPOINT",
            DidError::KeyNotFound(_) => "KEY_NOT_FOUND",
            DidError::InvalidKeyFormat(_) => "INVALID_KEY_FORMAT",
            DidError::InvalidSignature(_) => "INVALID_SIGNATURE",
            DidError::CryptographicError(_) => "CRYPTOGRAPHIC_ERROR",
            DidError::InvalidCredential(_) => "INVALID_CREDENTIAL",
            DidError::CredentialExpired(_) => "CREDENTIAL_EXPIRED",
            DidError::CredentialRevoked(_) => "CREDENTIAL_REVOKED",
            DidError::InvalidPresentation(_) => "INVALID_PRESENTATION",
            DidError::VerificationFailed(_) => "VERIFICATION_FAILED",
            DidError::ResolutionError(_) => "RESOLUTION_ERROR",
            DidError::NetworkError(_) => "NETWORK_ERROR",
            DidError::TimeoutError(_) => "TIMEOUT_ERROR",
            DidError::RegistryError(_) => "REGISTRY_ERROR",
            DidError::StorageError(_) => "STORAGE_ERROR",
            DidError::SerializationError(_) => "SERIALIZATION_ERROR",
            DidError::ConfigurationError(_) => "CONFIGURATION_ERROR",
            DidError::PermissionDenied(_) => "PERMISSION_DENIED",
            DidError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            DidError::InternalError(_) => "INTERNAL_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DidError::NetworkError(_) | 
            DidError::TimeoutError(_) | 
            DidError::RegistryError(_) |
            DidError::RateLimitExceeded(_)
        )
    }

    /// Check if error is client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            DidError::InvalidDidFormat(_) |
            DidError::InvalidDidMethod(_) |
            DidError::DidNotFound(_) |
            DidError::DidAlreadyExists(_) |
            DidError::DidDeactivated(_) |
            DidError::InvalidDidDocument(_) |
            DidError::InvalidVerificationMethod(_) |
            DidError::InvalidServiceEndpoint(_) |
            DidError::KeyNotFound(_) |
            DidError::InvalidKeyFormat(_) |
            DidError::InvalidSignature(_) |
            DidError::InvalidCredential(_) |
            DidError::CredentialExpired(_) |
            DidError::CredentialRevoked(_) |
            DidError::InvalidPresentation(_) |
            DidError::VerificationFailed(_) |
            DidError::PermissionDenied(_)
        )
    }

    /// Check if error is server error (5xx)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            DidError::CryptographicError(_) |
            DidError::ResolutionError(_) |
            DidError::NetworkError(_) |
            DidError::TimeoutError(_) |
            DidError::RegistryError(_) |
            DidError::StorageError(_) |
            DidError::SerializationError(_) |
            DidError::ConfigurationError(_) |
            DidError::RateLimitExceeded(_) |
            DidError::InternalError(_)
        )
    }
}

// Convert from common error types
impl From<serde_json::Error> for DidError {
    fn from(err: serde_json::Error) -> Self {
        DidError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for DidError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            DidError::TimeoutError(err.to_string())
        } else if err.is_connect() {
            DidError::NetworkError(err.to_string())
        } else {
            DidError::ResolutionError(err.to_string())
        }
    }
}

impl From<url::ParseError> for DidError {
    fn from(err: url::ParseError) -> Self {
        DidError::InvalidDidFormat(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(DidError::InvalidDidFormat("test".to_string()).error_code(), "INVALID_DID_FORMAT");
        assert_eq!(DidError::DidNotFound("test".to_string()).error_code(), "DID_NOT_FOUND");
        assert_eq!(DidError::NetworkError("test".to_string()).error_code(), "NETWORK_ERROR");
    }

    #[test]
    fn test_error_classification() {
        let client_error = DidError::InvalidDidFormat("test".to_string());
        assert!(client_error.is_client_error());
        assert!(!client_error.is_server_error());
        assert!(!client_error.is_retryable());

        let server_error = DidError::NetworkError("test".to_string());
        assert!(!server_error.is_client_error());
        assert!(server_error.is_server_error());
        assert!(server_error.is_retryable());
    }
}
