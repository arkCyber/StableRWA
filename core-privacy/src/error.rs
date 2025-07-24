// =====================================================================================
// File: core-privacy/src/error.rs
// Description: Error types for privacy and cryptographic operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for privacy operations
pub type PrivacyResult<T> = Result<T, PrivacyError>;

/// Privacy service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyError {
    /// Cryptographic errors
    #[error("Cryptographic error: {message}")]
    CryptographicError { message: String },

    /// Zero-knowledge proof errors
    #[error("ZK proof error: {message}")]
    ZKProofError { message: String },

    /// Homomorphic encryption errors
    #[error("Homomorphic encryption error: {message}")]
    HomomorphicError { message: String },

    /// Secure computation errors
    #[error("Secure computation error: {message}")]
    SecureComputationError { message: String },

    /// Differential privacy errors
    #[error("Differential privacy error: {message}")]
    DifferentialPrivacyError { message: String },

    /// Key management errors
    #[error("Key management error: {message}")]
    KeyManagementError { message: String },

    /// Proof verification errors
    #[error("Proof verification failed: {reason}")]
    ProofVerificationError { reason: String },

    /// Invalid parameters
    #[error("Invalid parameter '{parameter}': {message}")]
    InvalidParameter { parameter: String, message: String },

    /// Insufficient privacy budget
    #[error("Insufficient privacy budget: required {required}, available {available}")]
    InsufficientPrivacyBudget { required: f64, available: f64 },

    /// Circuit compilation errors
    #[error("Circuit compilation error: {message}")]
    CircuitCompilationError { message: String },

    /// Witness generation errors
    #[error("Witness generation error: {message}")]
    WitnessGenerationError { message: String },

    /// Commitment scheme errors
    #[error("Commitment scheme error: {message}")]
    CommitmentError { message: String },

    /// Merkle tree errors
    #[error("Merkle tree error: {message}")]
    MerkleTreeError { message: String },

    /// Range proof errors
    #[error("Range proof error: {message}")]
    RangeProofError { message: String },

    /// Signature scheme errors
    #[error("Signature scheme error: {message}")]
    SignatureError { message: String },

    /// Anonymization errors
    #[error("Anonymization error: {message}")]
    AnonymizationError { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Hardware security module errors
    #[error("HSM error: {message}")]
    HSMError { message: String },

    /// Trusted execution environment errors
    #[error("TEE error: {message}")]
    TEEError { message: String },

    /// Network protocol errors
    #[error("Network protocol error: {message}")]
    NetworkProtocolError { message: String },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Resource exhaustion
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    /// Internal errors
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl PrivacyError {
    /// Create a cryptographic error
    pub fn cryptographic_error<S: Into<String>>(message: S) -> Self {
        Self::CryptographicError {
            message: message.into(),
        }
    }

    /// Create a ZK proof error
    pub fn zkproof_error<S: Into<String>>(message: S) -> Self {
        Self::ZKProofError {
            message: message.into(),
        }
    }

    /// Create a homomorphic encryption error
    pub fn homomorphic_error<S: Into<String>>(message: S) -> Self {
        Self::HomomorphicError {
            message: message.into(),
        }
    }

    /// Create a secure computation error
    pub fn secure_computation_error<S: Into<String>>(message: S) -> Self {
        Self::SecureComputationError {
            message: message.into(),
        }
    }

    /// Create a differential privacy error
    pub fn differential_privacy_error<S: Into<String>>(message: S) -> Self {
        Self::DifferentialPrivacyError {
            message: message.into(),
        }
    }

    /// Create a key management error
    pub fn key_management_error<S: Into<String>>(message: S) -> Self {
        Self::KeyManagementError {
            message: message.into(),
        }
    }

    /// Create a proof verification error
    pub fn proof_verification_error<S: Into<String>>(reason: S) -> Self {
        Self::ProofVerificationError {
            reason: reason.into(),
        }
    }

    /// Create an invalid parameter error
    pub fn invalid_parameter<S: Into<String>>(parameter: S, message: S) -> Self {
        Self::InvalidParameter {
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    /// Create an insufficient privacy budget error
    pub fn insufficient_privacy_budget(required: f64, available: f64) -> Self {
        Self::InsufficientPrivacyBudget {
            required,
            available,
        }
    }

    /// Create a circuit compilation error
    pub fn circuit_compilation_error<S: Into<String>>(message: S) -> Self {
        Self::CircuitCompilationError {
            message: message.into(),
        }
    }

    /// Create a witness generation error
    pub fn witness_generation_error<S: Into<String>>(message: S) -> Self {
        Self::WitnessGenerationError {
            message: message.into(),
        }
    }

    /// Create a commitment error
    pub fn commitment_error<S: Into<String>>(message: S) -> Self {
        Self::CommitmentError {
            message: message.into(),
        }
    }

    /// Create a Merkle tree error
    pub fn merkle_tree_error<S: Into<String>>(message: S) -> Self {
        Self::MerkleTreeError {
            message: message.into(),
        }
    }

    /// Create a range proof error
    pub fn range_proof_error<S: Into<String>>(message: S) -> Self {
        Self::RangeProofError {
            message: message.into(),
        }
    }

    /// Create a signature error
    pub fn signature_error<S: Into<String>>(message: S) -> Self {
        Self::SignatureError {
            message: message.into(),
        }
    }

    /// Create an anonymization error
    pub fn anonymization_error<S: Into<String>>(message: S) -> Self {
        Self::AnonymizationError {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error<S: Into<String>>(message: S) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    /// Create an HSM error
    pub fn hsm_error<S: Into<String>>(message: S) -> Self {
        Self::HSMError {
            message: message.into(),
        }
    }

    /// Create a TEE error
    pub fn tee_error<S: Into<String>>(message: S) -> Self {
        Self::TEEError {
            message: message.into(),
        }
    }

    /// Create a network protocol error
    pub fn network_protocol_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkProtocolError {
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted<S: Into<String>>(resource: S) -> Self {
        Self::ResourceExhausted {
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
            Self::CryptographicError { .. } => "CRYPTOGRAPHIC_ERROR",
            Self::ZKProofError { .. } => "ZK_PROOF_ERROR",
            Self::HomomorphicError { .. } => "HOMOMORPHIC_ERROR",
            Self::SecureComputationError { .. } => "SECURE_COMPUTATION_ERROR",
            Self::DifferentialPrivacyError { .. } => "DIFFERENTIAL_PRIVACY_ERROR",
            Self::KeyManagementError { .. } => "KEY_MANAGEMENT_ERROR",
            Self::ProofVerificationError { .. } => "PROOF_VERIFICATION_ERROR",
            Self::InvalidParameter { .. } => "INVALID_PARAMETER",
            Self::InsufficientPrivacyBudget { .. } => "INSUFFICIENT_PRIVACY_BUDGET",
            Self::CircuitCompilationError { .. } => "CIRCUIT_COMPILATION_ERROR",
            Self::WitnessGenerationError { .. } => "WITNESS_GENERATION_ERROR",
            Self::CommitmentError { .. } => "COMMITMENT_ERROR",
            Self::MerkleTreeError { .. } => "MERKLE_TREE_ERROR",
            Self::RangeProofError { .. } => "RANGE_PROOF_ERROR",
            Self::SignatureError { .. } => "SIGNATURE_ERROR",
            Self::AnonymizationError { .. } => "ANONYMIZATION_ERROR",
            Self::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            Self::SerializationError { .. } => "SERIALIZATION_ERROR",
            Self::HSMError { .. } => "HSM_ERROR",
            Self::TEEError { .. } => "TEE_ERROR",
            Self::NetworkProtocolError { .. } => "NETWORK_PROTOCOL_ERROR",
            Self::Timeout { .. } => "TIMEOUT",
            Self::ResourceExhausted { .. } => "RESOURCE_EXHAUSTED",
            Self::InternalError { .. } => "INTERNAL_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::ResourceExhausted { .. }
                | Self::NetworkProtocolError { .. }
                | Self::HSMError { .. }
                | Self::TEEError { .. }
        )
    }

    /// Check if error is critical
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::KeyManagementError { .. }
                | Self::ProofVerificationError { .. }
                | Self::InternalError { .. }
        )
    }

    /// Check if error is security-related
    pub fn is_security_related(&self) -> bool {
        matches!(
            self,
            Self::CryptographicError { .. }
                | Self::ZKProofError { .. }
                | Self::KeyManagementError { .. }
                | Self::ProofVerificationError { .. }
                | Self::HSMError { .. }
                | Self::TEEError { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = PrivacyError::cryptographic_error("Test crypto error");
        assert_eq!(error.error_code(), "CRYPTOGRAPHIC_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_security_related());
    }

    #[test]
    fn test_zkproof_error() {
        let error = PrivacyError::zkproof_error("Invalid proof");
        assert_eq!(error.error_code(), "ZK_PROOF_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_security_related());
    }

    #[test]
    fn test_insufficient_privacy_budget() {
        let error = PrivacyError::insufficient_privacy_budget(1.0, 0.5);
        assert_eq!(error.error_code(), "INSUFFICIENT_PRIVACY_BUDGET");
        assert!(!error.is_retryable());
        assert!(!error.is_security_related());
    }

    #[test]
    fn test_retryable_error() {
        let error = PrivacyError::timeout("zkproof_generation");
        assert_eq!(error.error_code(), "TIMEOUT");
        assert!(error.is_retryable());
        assert!(!error.is_security_related());
    }

    #[test]
    fn test_critical_error() {
        let error = PrivacyError::key_management_error("Key rotation failed");
        assert_eq!(error.error_code(), "KEY_MANAGEMENT_ERROR");
        assert!(!error.is_retryable());
        assert!(error.is_critical());
        assert!(error.is_security_related());
    }
}
