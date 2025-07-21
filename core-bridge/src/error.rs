// =====================================================================================
// File: core-bridge/src/error.rs
// Description: Error types for cross-chain bridge
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type alias for bridge operations
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Comprehensive error types for bridge operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BridgeError {
    /// Transfer-related errors
    #[error("Transfer error: {message}")]
    TransferError { message: String },

    /// Liquidity-related errors
    #[error("Liquidity error: {message}")]
    LiquidityError { message: String },

    /// Atomic swap errors
    #[error("Atomic swap error: {message}")]
    AtomicSwapError { message: String },

    /// Security-related errors
    #[error("Security error: {message}")]
    SecurityError { message: String },

    /// Validator errors
    #[error("Validator error: {validator_id} - {message}")]
    ValidatorError { validator_id: String, message: String },

    /// Relayer errors
    #[error("Relayer error: {relayer_id} - {message}")]
    RelayerError { relayer_id: String, message: String },

    /// Chain connection errors
    #[error("Chain connection error: {chain_id} - {message}")]
    ChainConnectionError { chain_id: String, message: String },

    /// Smart contract errors
    #[error("Contract error: {contract_address} - {message}")]
    ContractError { contract_address: String, message: String },

    /// Transaction errors
    #[error("Transaction error: {tx_hash} - {message}")]
    TransactionError { tx_hash: String, message: String },

    /// Insufficient balance
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    /// Insufficient liquidity
    #[error("Insufficient liquidity: pool {pool_id} - {message}")]
    InsufficientLiquidity { pool_id: String, message: String },

    /// Invalid transaction
    #[error("Invalid transaction: {reason}")]
    InvalidTransaction { reason: String },

    /// Transaction timeout
    #[error("Transaction timeout: {transaction_id} - expired after {timeout_seconds}s")]
    TransactionTimeout { transaction_id: String, timeout_seconds: u64 },

    /// Bridge paused
    #[error("Bridge paused: {reason}")]
    BridgePaused { reason: String },

    /// Unsupported chain
    #[error("Unsupported chain: {chain_id}")]
    UnsupportedChain { chain_id: String },

    /// Unsupported token
    #[error("Unsupported token: {token_symbol} on chain {chain_id}")]
    UnsupportedToken { token_symbol: String, chain_id: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {limit_type} - {message}")]
    RateLimitExceeded { limit_type: String, message: String },

    /// Amount limits exceeded
    #[error("Amount limit exceeded: {limit_type} - amount {amount}, limit {limit}")]
    AmountLimitExceeded { limit_type: String, amount: String, limit: String },

    /// Slippage too high
    #[error("Slippage too high: expected {expected}%, actual {actual}%")]
    SlippageTooHigh { expected: String, actual: String },

    /// Proof verification failed
    #[error("Proof verification failed: {proof_type} - {message}")]
    ProofVerificationFailed { proof_type: String, message: String },

    /// Signature verification failed
    #[error("Signature verification failed: {signer} - {message}")]
    SignatureVerificationFailed { signer: String, message: String },

    /// Insufficient confirmations
    #[error("Insufficient confirmations: {current}/{required} for transaction {tx_hash}")]
    InsufficientConfirmations { current: u64, required: u64, tx_hash: String },

    /// Nonce error
    #[error("Nonce error: {message}")]
    NonceError { message: String },

    /// Gas estimation failed
    #[error("Gas estimation failed: {message}")]
    GasEstimationFailed { message: String },

    /// Gas price too high
    #[error("Gas price too high: {current} > {maximum}")]
    GasPriceTooHigh { current: String, maximum: String },

    /// Configuration errors
    #[error("Configuration error: {component} - {message}")]
    ConfigurationError { component: String, message: String },

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

    /// Validation errors
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },

    /// Insufficient permissions
    #[error("Insufficient permissions: {required_permission} for user {user_id}")]
    InsufficientPermissions { user_id: String, required_permission: String },

    /// Concurrent modification error
    #[error("Concurrent modification detected for {resource_type} {resource_id}")]
    ConcurrentModification { resource_type: String, resource_id: String },

    /// Business rule violation
    #[error("Business rule violation: {rule} - {message}")]
    BusinessRuleViolation { rule: String, message: String },

    /// Emergency shutdown
    #[error("Emergency shutdown activated: {reason}")]
    EmergencyShutdown { reason: String },

    /// Maintenance mode
    #[error("Bridge in maintenance mode: {message}")]
    MaintenanceMode { message: String },

    /// Internal system errors
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Generic bridge error
    #[error("Bridge error: {0}")]
    Generic(String),
}

impl BridgeError {
    /// Create a transfer error
    pub fn transfer_error<S: Into<String>>(message: S) -> Self {
        Self::TransferError { message: message.into() }
    }

    /// Create a liquidity error
    pub fn liquidity_error<S: Into<String>>(message: S) -> Self {
        Self::LiquidityError { message: message.into() }
    }

    /// Create an atomic swap error
    pub fn atomic_swap_error<S: Into<String>>(message: S) -> Self {
        Self::AtomicSwapError { message: message.into() }
    }

    /// Create a security error
    pub fn security_error<S: Into<String>>(message: S) -> Self {
        Self::SecurityError { message: message.into() }
    }

    /// Create a validator error
    pub fn validator_error<S: Into<String>>(validator_id: S, message: S) -> Self {
        Self::ValidatorError {
            validator_id: validator_id.into(),
            message: message.into(),
        }
    }

    /// Create a chain connection error
    pub fn chain_connection_error<S: Into<String>>(chain_id: S, message: S) -> Self {
        Self::ChainConnectionError {
            chain_id: chain_id.into(),
            message: message.into(),
        }
    }

    /// Create an insufficient balance error
    pub fn insufficient_balance<S: Into<String>>(required: S, available: S) -> Self {
        Self::InsufficientBalance {
            required: required.into(),
            available: available.into(),
        }
    }

    /// Create an invalid transaction error
    pub fn invalid_transaction<S: Into<String>>(reason: S) -> Self {
        Self::InvalidTransaction { reason: reason.into() }
    }

    /// Create a transaction timeout error
    pub fn transaction_timeout<S: Into<String>>(transaction_id: S, timeout_seconds: u64) -> Self {
        Self::TransactionTimeout {
            transaction_id: transaction_id.into(),
            timeout_seconds,
        }
    }

    /// Create a bridge paused error
    pub fn bridge_paused<S: Into<String>>(reason: S) -> Self {
        Self::BridgePaused { reason: reason.into() }
    }

    /// Create an unsupported chain error
    pub fn unsupported_chain<S: Into<String>>(chain_id: S) -> Self {
        Self::UnsupportedChain { chain_id: chain_id.into() }
    }

    /// Create an unsupported token error
    pub fn unsupported_token<S: Into<String>>(token_symbol: S, chain_id: S) -> Self {
        Self::UnsupportedToken {
            token_symbol: token_symbol.into(),
            chain_id: chain_id.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded<S: Into<String>>(limit_type: S, message: S) -> Self {
        Self::RateLimitExceeded {
            limit_type: limit_type.into(),
            message: message.into(),
        }
    }

    /// Create an amount limit exceeded error
    pub fn amount_limit_exceeded<S: Into<String>>(limit_type: S, amount: S, limit: S) -> Self {
        Self::AmountLimitExceeded {
            limit_type: limit_type.into(),
            amount: amount.into(),
            limit: limit.into(),
        }
    }

    /// Create a slippage too high error
    pub fn slippage_too_high<S: Into<String>>(expected: S, actual: S) -> Self {
        Self::SlippageTooHigh {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a proof verification failed error
    pub fn proof_verification_failed<S: Into<String>>(proof_type: S, message: S) -> Self {
        Self::ProofVerificationFailed {
            proof_type: proof_type.into(),
            message: message.into(),
        }
    }

    /// Create an insufficient confirmations error
    pub fn insufficient_confirmations<S: Into<String>>(current: u64, required: u64, tx_hash: S) -> Self {
        Self::InsufficientConfirmations {
            current,
            required,
            tx_hash: tx_hash.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            BridgeError::NetworkError { .. }
                | BridgeError::DatabaseError { .. }
                | BridgeError::ExternalServiceError { .. }
                | BridgeError::ChainConnectionError { .. }
                | BridgeError::GasEstimationFailed { .. }
                | BridgeError::NonceError { .. }
                | BridgeError::InternalError { .. }
        )
    }

    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            BridgeError::TransferError { .. } => "transfer",
            BridgeError::LiquidityError { .. } => "liquidity",
            BridgeError::AtomicSwapError { .. } => "atomic_swap",
            BridgeError::SecurityError { .. } => "security",
            BridgeError::ValidatorError { .. } => "validator",
            BridgeError::RelayerError { .. } => "relayer",
            BridgeError::ChainConnectionError { .. } => "chain_connection",
            BridgeError::ContractError { .. } => "contract",
            BridgeError::TransactionError { .. } => "transaction",
            BridgeError::InsufficientBalance { .. } => "insufficient_balance",
            BridgeError::InsufficientLiquidity { .. } => "insufficient_liquidity",
            BridgeError::InvalidTransaction { .. } => "invalid_transaction",
            BridgeError::TransactionTimeout { .. } => "transaction_timeout",
            BridgeError::BridgePaused { .. } => "bridge_paused",
            BridgeError::UnsupportedChain { .. } => "unsupported_chain",
            BridgeError::UnsupportedToken { .. } => "unsupported_token",
            BridgeError::RateLimitExceeded { .. } => "rate_limit",
            BridgeError::AmountLimitExceeded { .. } => "amount_limit",
            BridgeError::SlippageTooHigh { .. } => "slippage",
            BridgeError::ProofVerificationFailed { .. } => "proof_verification",
            BridgeError::SignatureVerificationFailed { .. } => "signature_verification",
            BridgeError::InsufficientConfirmations { .. } => "insufficient_confirmations",
            BridgeError::NonceError { .. } => "nonce",
            BridgeError::GasEstimationFailed { .. } => "gas_estimation",
            BridgeError::GasPriceTooHigh { .. } => "gas_price",
            BridgeError::ConfigurationError { .. } => "configuration",
            BridgeError::ExternalServiceError { .. } => "external_service",
            BridgeError::DatabaseError { .. } => "database",
            BridgeError::NetworkError { .. } => "network",
            BridgeError::SerializationError { .. } => "serialization",
            BridgeError::AuthenticationError { .. } => "authentication",
            BridgeError::ValidationError { .. } => "validation",
            BridgeError::InsufficientPermissions { .. } => "permissions",
            BridgeError::ConcurrentModification { .. } => "concurrency",
            BridgeError::BusinessRuleViolation { .. } => "business_rule",
            BridgeError::EmergencyShutdown { .. } => "emergency_shutdown",
            BridgeError::MaintenanceMode { .. } => "maintenance",
            BridgeError::InternalError { .. } => "internal",
            BridgeError::Generic(_) => "generic",
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            BridgeError::EmergencyShutdown { .. } => ErrorSeverity::Critical,
            BridgeError::SecurityError { .. } => ErrorSeverity::Critical,
            BridgeError::InternalError { .. } => ErrorSeverity::Critical,
            BridgeError::ProofVerificationFailed { .. } => ErrorSeverity::High,
            BridgeError::SignatureVerificationFailed { .. } => ErrorSeverity::High,
            BridgeError::ValidatorError { .. } => ErrorSeverity::High,
            BridgeError::ContractError { .. } => ErrorSeverity::High,
            BridgeError::DatabaseError { .. } => ErrorSeverity::High,
            BridgeError::AuthenticationError { .. } => ErrorSeverity::High,
            BridgeError::BusinessRuleViolation { .. } => ErrorSeverity::High,
            BridgeError::BridgePaused { .. } => ErrorSeverity::Medium,
            BridgeError::MaintenanceMode { .. } => ErrorSeverity::Medium,
            BridgeError::TransactionTimeout { .. } => ErrorSeverity::Medium,
            BridgeError::InsufficientBalance { .. } => ErrorSeverity::Medium,
            BridgeError::RateLimitExceeded { .. } => ErrorSeverity::Medium,
            BridgeError::AmountLimitExceeded { .. } => ErrorSeverity::Medium,
            BridgeError::ValidationError { .. } => ErrorSeverity::Medium,
            BridgeError::NetworkError { .. } => ErrorSeverity::Low,
            BridgeError::UnsupportedChain { .. } => ErrorSeverity::Low,
            BridgeError::UnsupportedToken { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }

    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Critical | ErrorSeverity::High)
    }

    /// Check if error should trigger emergency procedures
    pub fn should_trigger_emergency(&self) -> bool {
        matches!(
            self,
            BridgeError::EmergencyShutdown { .. }
                | BridgeError::SecurityError { .. }
                | BridgeError::ProofVerificationFailed { .. }
                | BridgeError::SignatureVerificationFailed { .. }
        )
    }

    /// Check if error is user-facing (vs system error)
    pub fn is_user_facing(&self) -> bool {
        matches!(
            self,
            BridgeError::InsufficientBalance { .. }
                | BridgeError::InvalidTransaction { .. }
                | BridgeError::UnsupportedChain { .. }
                | BridgeError::UnsupportedToken { .. }
                | BridgeError::RateLimitExceeded { .. }
                | BridgeError::AmountLimitExceeded { .. }
                | BridgeError::SlippageTooHigh { .. }
                | BridgeError::ValidationError { .. }
                | BridgeError::BridgePaused { .. }
                | BridgeError::MaintenanceMode { .. }
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
impl From<serde_json::Error> for BridgeError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError { message: err.to_string() }
    }
}

impl From<reqwest::Error> for BridgeError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError { message: err.to_string() }
    }
}

#[cfg(feature = "database")]
impl From<sqlx::Error> for BridgeError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError { message: err.to_string() }
    }
}

impl From<validator::ValidationErrors> for BridgeError {
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
        let transfer_error = BridgeError::transfer_error("Invalid amount");
        assert!(matches!(transfer_error, BridgeError::TransferError { .. }));
        assert_eq!(transfer_error.category(), "transfer");
    }

    #[test]
    fn test_error_retryability() {
        let network_error = BridgeError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(network_error.is_retryable());

        let validation_error = BridgeError::ValidationError {
            field: "amount".to_string(),
            message: "Invalid amount".to_string(),
        };
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let security_error = BridgeError::SecurityError {
            message: "Potential attack detected".to_string(),
        };
        assert_eq!(security_error.severity(), ErrorSeverity::Critical);
        assert!(security_error.requires_immediate_attention());
        assert!(security_error.should_trigger_emergency());

        let network_error = BridgeError::NetworkError {
            message: "Timeout".to_string(),
        };
        assert_eq!(network_error.severity(), ErrorSeverity::Low);
        assert!(!network_error.requires_immediate_attention());
        assert!(!network_error.should_trigger_emergency());
    }

    #[test]
    fn test_user_facing_errors() {
        let insufficient_balance = BridgeError::InsufficientBalance {
            required: "100".to_string(),
            available: "50".to_string(),
        };
        assert!(insufficient_balance.is_user_facing());

        let internal_error = BridgeError::InternalError {
            message: "System failure".to_string(),
        };
        assert!(!internal_error.is_user_facing());
    }

    #[test]
    fn test_specific_error_constructors() {
        let timeout_error = BridgeError::transaction_timeout("tx123", 3600);
        assert!(matches!(timeout_error, BridgeError::TransactionTimeout { .. }));

        let unsupported_token = BridgeError::unsupported_token("XYZ", "Ethereum");
        assert!(matches!(unsupported_token, BridgeError::UnsupportedToken { .. }));

        let slippage_error = BridgeError::slippage_too_high("1%", "5%");
        assert!(matches!(slippage_error, BridgeError::SlippageTooHigh { .. }));
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            BridgeError::transfer_error("test"),
            BridgeError::liquidity_error("test"),
            BridgeError::atomic_swap_error("test"),
            BridgeError::security_error("test"),
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(categories, vec!["transfer", "liquidity", "atomic_swap", "security"]);
    }
}
