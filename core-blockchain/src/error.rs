// =====================================================================================
// File: core-blockchain/src/error.rs
// Description: Blockchain error types and handling
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;

/// Result type alias for blockchain operations
pub type BlockchainResult<T> = Result<T, BlockchainError>;

/// Comprehensive blockchain error types
#[derive(Error, Debug, Clone)]
pub enum BlockchainError {
    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("Invalid transaction: {message}")]
    InvalidTransaction { message: String },

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    #[error("Transaction failed: hash {hash}, reason: {reason}")]
    TransactionFailed { hash: String, reason: String },

    #[error("Transaction not found: {hash}")]
    TransactionNotFound { hash: String },

    #[error("Block not found: {block_id}")]
    BlockNotFound { block_id: String },

    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },

    #[error("Invalid private key")]
    InvalidPrivateKey,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("RPC error: {code} - {message}")]
    RpcError { code: i32, message: String },

    #[error("Contract error: {message}")]
    ContractError { message: String },

    #[error("ABI error: {message}")]
    AbiError { message: String },

    #[error("Gas estimation failed: {message}")]
    GasEstimationFailed { message: String },

    #[error("Nonce error: {message}")]
    NonceError { message: String },

    #[error("Wallet error: {message}")]
    WalletError { message: String },

    #[error("Encryption error: {message}")]
    EncryptionError { message: String },

    #[error("Decryption error: {message}")]
    DecryptionError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Timeout error: operation timed out after {seconds} seconds")]
    TimeoutError { seconds: u64 },

    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },

    #[error("Chain not supported: {chain_id}")]
    UnsupportedChain { chain_id: u64 },

    #[error("Invalid input: field {field} - {message}")]
    InvalidInput { field: String, message: String },

    #[error("Resource not found: {resource} with id {id}")]
    NotFound { resource: String, id: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl BlockchainError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            BlockchainError::NetworkError { .. }
                | BlockchainError::RpcError { .. }
                | BlockchainError::TimeoutError { .. }
                | BlockchainError::ConnectionError { .. }
        )
    }

    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            BlockchainError::ConnectionError { .. } => "connection",
            BlockchainError::InvalidTransaction { .. } => "transaction",
            BlockchainError::InsufficientBalance { .. } => "balance",
            BlockchainError::TransactionFailed { .. } => "transaction",
            BlockchainError::TransactionNotFound { .. } => "transaction",
            BlockchainError::BlockNotFound { .. } => "block",
            BlockchainError::InvalidAddress { .. } => "address",
            BlockchainError::InvalidPrivateKey => "wallet",
            BlockchainError::InvalidSignature => "signature",
            BlockchainError::NetworkError { .. } => "network",
            BlockchainError::RpcError { .. } => "rpc",
            BlockchainError::ContractError { .. } => "contract",
            BlockchainError::AbiError { .. } => "abi",
            BlockchainError::GasEstimationFailed { .. } => "gas",
            BlockchainError::NonceError { .. } => "nonce",
            BlockchainError::WalletError { .. } => "wallet",
            BlockchainError::EncryptionError { .. } => "encryption",
            BlockchainError::DecryptionError { .. } => "encryption",
            BlockchainError::SerializationError { .. } => "serialization",
            BlockchainError::DeserializationError { .. } => "serialization",
            BlockchainError::ConfigurationError { .. } => "configuration",
            BlockchainError::TimeoutError { .. } => "timeout",
            BlockchainError::RateLimitExceeded { .. } => "rate_limit",
            BlockchainError::UnsupportedChain { .. } => "chain",
            BlockchainError::InvalidInput { .. } => "input",
            BlockchainError::NotFound { .. } => "not_found",
            BlockchainError::PermissionDenied { .. } => "permission",
            BlockchainError::InternalError { .. } => "internal",
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            BlockchainError::InternalError { .. } => ErrorSeverity::Critical,
            BlockchainError::PermissionDenied { .. } => ErrorSeverity::High,
            BlockchainError::InvalidPrivateKey => ErrorSeverity::High,
            BlockchainError::EncryptionError { .. } => ErrorSeverity::High,
            BlockchainError::DecryptionError { .. } => ErrorSeverity::High,
            BlockchainError::TransactionFailed { .. } => ErrorSeverity::Medium,
            BlockchainError::InsufficientBalance { .. } => ErrorSeverity::Medium,
            BlockchainError::ContractError { .. } => ErrorSeverity::Medium,
            BlockchainError::NetworkError { .. } => ErrorSeverity::Low,
            BlockchainError::TimeoutError { .. } => ErrorSeverity::Low,
            BlockchainError::RateLimitExceeded { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

// Implement conversions from common error types
impl From<ethers::providers::ProviderError> for BlockchainError {
    fn from(err: ethers::providers::ProviderError) -> Self {
        BlockchainError::RpcError {
            code: -1,
            message: err.to_string(),
        }
    }
}

impl From<ethers::contract::ContractError<ethers::providers::Provider<ethers::providers::Ws>>> for BlockchainError {
    fn from(err: ethers::contract::ContractError<ethers::providers::Provider<ethers::providers::Ws>>) -> Self {
        BlockchainError::ContractError {
            message: err.to_string(),
        }
    }
}

impl From<ethers::abi::Error> for BlockchainError {
    fn from(err: ethers::abi::Error) -> Self {
        BlockchainError::AbiError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for BlockchainError {
    fn from(err: serde_json::Error) -> Self {
        BlockchainError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for BlockchainError {
    fn from(err: reqwest::Error) -> Self {
        BlockchainError::NetworkError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for BlockchainError {
    fn from(err: std::io::Error) -> Self {
        BlockchainError::InternalError {
            message: err.to_string(),
        }
    }
}

impl From<hex::FromHexError> for BlockchainError {
    fn from(err: hex::FromHexError) -> Self {
        BlockchainError::InvalidInput {
            field: "hex_string".to_string(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = BlockchainError::InvalidTransaction {
            message: "Invalid nonce".to_string(),
        };
        assert_eq!(error.category(), "transaction");
        assert_eq!(error.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_error_retryability() {
        let network_error = BlockchainError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(network_error.is_retryable());

        let invalid_address = BlockchainError::InvalidAddress {
            address: "0xinvalid".to_string(),
        };
        assert!(!invalid_address.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let critical_error = BlockchainError::InternalError {
            message: "System failure".to_string(),
        };
        assert_eq!(critical_error.severity(), ErrorSeverity::Critical);

        let low_error = BlockchainError::NetworkError {
            message: "Temporary network issue".to_string(),
        };
        assert_eq!(low_error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            BlockchainError::ConnectionError { message: "test".to_string() },
            BlockchainError::TransactionFailed { hash: "0x123".to_string(), reason: "test".to_string() },
            BlockchainError::ContractError { message: "test".to_string() },
            BlockchainError::WalletError { message: "test".to_string() },
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(categories, vec!["connection", "transaction", "contract", "wallet"]);
    }
}
