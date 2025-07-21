// =====================================================================================
// File: core-defi/src/error.rs
// Description: Error types for DeFi services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type for DeFi operations
pub type DeFiResult<T> = Result<T, DeFiError>;

/// DeFi service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum DeFiError {
    /// Validation errors
    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    /// Insufficient balance errors
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    /// Slippage tolerance exceeded
    #[error("Slippage tolerance exceeded: expected {expected}, actual {actual}")]
    SlippageExceeded { expected: String, actual: String },

    /// Liquidity errors
    #[error("Insufficient liquidity: {message}")]
    InsufficientLiquidity { message: String },

    /// Price impact too high
    #[error("Price impact too high: {impact}% exceeds maximum {max_impact}%")]
    PriceImpactTooHigh { impact: String, max_impact: String },

    /// Protocol errors
    #[error("Protocol error in {protocol}: {message}")]
    ProtocolError { protocol: String, message: String },

    /// Smart contract errors
    #[error("Smart contract error: {message}")]
    SmartContractError { message: String },

    /// Transaction errors
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },

    /// Gas estimation errors
    #[error("Gas estimation failed: {message}")]
    GasEstimationFailed { message: String },

    /// Oracle errors
    #[error("Oracle error: {message}")]
    OracleError { message: String },

    /// Price feed errors
    #[error("Price feed error for {asset}: {message}")]
    PriceFeedError { asset: String, message: String },

    /// Liquidation errors
    #[error("Liquidation error: {message}")]
    LiquidationError { message: String },

    /// Collateral errors
    #[error("Collateral error: {message}")]
    CollateralError { message: String },

    /// Flash loan errors
    #[error("Flash loan error: {message}")]
    FlashLoanError { message: String },

    /// Yield farming errors
    #[error("Yield farming error: {message}")]
    YieldFarmingError { message: String },

    /// Staking errors
    #[error("Staking error: {message}")]
    StakingError { message: String },

    /// Governance errors
    #[error("Governance error: {message}")]
    GovernanceError { message: String },

    /// MEV protection errors
    #[error("MEV protection error: {message}")]
    MEVProtectionError { message: String },

    /// Front-running protection errors
    #[error("Front-running detected: {message}")]
    FrontRunningDetected { message: String },

    /// Network errors
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Database errors
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimitError { message: String },

    /// Resource not found
    #[error("Resource not found: {resource_type} with id '{id}'")]
    NotFound { resource_type: String, id: String },

    /// Resource already exists
    #[error("Resource already exists: {resource_type} with id '{id}'")]
    AlreadyExists { resource_type: String, id: String },

    /// Timeout errors
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Internal server errors
    #[error("Internal server error: {message}")]
    InternalError { message: String },

    /// External service errors
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
}

impl DeFiError {
    /// Create a validation error
    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
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

    /// Create a slippage exceeded error
    pub fn slippage_exceeded<S: Into<String>>(expected: S, actual: S) -> Self {
        Self::SlippageExceeded {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create an insufficient liquidity error
    pub fn insufficient_liquidity<S: Into<String>>(message: S) -> Self {
        Self::InsufficientLiquidity {
            message: message.into(),
        }
    }

    /// Create a price impact too high error
    pub fn price_impact_too_high<S: Into<String>>(impact: S, max_impact: S) -> Self {
        Self::PriceImpactTooHigh {
            impact: impact.into(),
            max_impact: max_impact.into(),
        }
    }

    /// Create a protocol error
    pub fn protocol_error<S: Into<String>>(protocol: S, message: S) -> Self {
        Self::ProtocolError {
            protocol: protocol.into(),
            message: message.into(),
        }
    }

    /// Create a smart contract error
    pub fn smart_contract_error<S: Into<String>>(message: S) -> Self {
        Self::SmartContractError {
            message: message.into(),
        }
    }

    /// Create a transaction failed error
    pub fn transaction_failed<S: Into<String>>(reason: S) -> Self {
        Self::TransactionFailed {
            reason: reason.into(),
        }
    }

    /// Create an oracle error
    pub fn oracle_error<S: Into<String>>(message: S) -> Self {
        Self::OracleError {
            message: message.into(),
        }
    }

    /// Create a price feed error
    pub fn price_feed_error<S: Into<String>>(asset: S, message: S) -> Self {
        Self::PriceFeedError {
            asset: asset.into(),
            message: message.into(),
        }
    }

    /// Create a flash loan error
    pub fn flash_loan_error<S: Into<String>>(message: S) -> Self {
        Self::FlashLoanError {
            message: message.into(),
        }
    }

    /// Create a yield farming error
    pub fn yield_farming_error<S: Into<String>>(message: S) -> Self {
        Self::YieldFarmingError {
            message: message.into(),
        }
    }

    /// Create a staking error
    pub fn staking_error<S: Into<String>>(message: S) -> Self {
        Self::StakingError {
            message: message.into(),
        }
    }

    /// Create a governance error
    pub fn governance_error<S: Into<String>>(message: S) -> Self {
        Self::GovernanceError {
            message: message.into(),
        }
    }

    /// Create an MEV protection error
    pub fn mev_protection_error<S: Into<String>>(message: S) -> Self {
        Self::MEVProtectionError {
            message: message.into(),
        }
    }

    /// Create a front-running detected error
    pub fn front_running_detected<S: Into<String>>(message: S) -> Self {
        Self::FrontRunningDetected {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource_type: S, id: S) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error<S: Into<String>>(message: S) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Get error code for API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            DeFiError::ValidationError { .. } => "VALIDATION_ERROR",
            DeFiError::InsufficientBalance { .. } => "INSUFFICIENT_BALANCE",
            DeFiError::SlippageExceeded { .. } => "SLIPPAGE_EXCEEDED",
            DeFiError::InsufficientLiquidity { .. } => "INSUFFICIENT_LIQUIDITY",
            DeFiError::PriceImpactTooHigh { .. } => "PRICE_IMPACT_TOO_HIGH",
            DeFiError::ProtocolError { .. } => "PROTOCOL_ERROR",
            DeFiError::SmartContractError { .. } => "SMART_CONTRACT_ERROR",
            DeFiError::TransactionFailed { .. } => "TRANSACTION_FAILED",
            DeFiError::GasEstimationFailed { .. } => "GAS_ESTIMATION_FAILED",
            DeFiError::OracleError { .. } => "ORACLE_ERROR",
            DeFiError::PriceFeedError { .. } => "PRICE_FEED_ERROR",
            DeFiError::LiquidationError { .. } => "LIQUIDATION_ERROR",
            DeFiError::CollateralError { .. } => "COLLATERAL_ERROR",
            DeFiError::FlashLoanError { .. } => "FLASH_LOAN_ERROR",
            DeFiError::YieldFarmingError { .. } => "YIELD_FARMING_ERROR",
            DeFiError::StakingError { .. } => "STAKING_ERROR",
            DeFiError::GovernanceError { .. } => "GOVERNANCE_ERROR",
            DeFiError::MEVProtectionError { .. } => "MEV_PROTECTION_ERROR",
            DeFiError::FrontRunningDetected { .. } => "FRONT_RUNNING_DETECTED",
            DeFiError::NetworkError { .. } => "NETWORK_ERROR",
            DeFiError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
            DeFiError::DatabaseError { .. } => "DATABASE_ERROR",
            DeFiError::SerializationError { .. } => "SERIALIZATION_ERROR",
            DeFiError::RateLimitError { .. } => "RATE_LIMIT_ERROR",
            DeFiError::NotFound { .. } => "NOT_FOUND",
            DeFiError::AlreadyExists { .. } => "ALREADY_EXISTS",
            DeFiError::Timeout { .. } => "TIMEOUT",
            DeFiError::InternalError { .. } => "INTERNAL_ERROR",
            DeFiError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DeFiError::NetworkError { .. }
                | DeFiError::Timeout { .. }
                | DeFiError::ExternalServiceError { .. }
                | DeFiError::GasEstimationFailed { .. }
                | DeFiError::OracleError { .. }
        )
    }

    /// Check if error is user error (4xx)
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            DeFiError::ValidationError { .. }
                | DeFiError::InsufficientBalance { .. }
                | DeFiError::SlippageExceeded { .. }
                | DeFiError::PriceImpactTooHigh { .. }
                | DeFiError::NotFound { .. }
                | DeFiError::AlreadyExists { .. }
                | DeFiError::RateLimitError { .. }
        )
    }

    /// Check if error is system error (5xx)
    pub fn is_system_error(&self) -> bool {
        !self.is_user_error()
    }
}

// Implement From traits for common error types
impl From<sqlx::Error> for DeFiError {
    fn from(err: sqlx::Error) -> Self {
        DeFiError::database_error(err.to_string())
    }
}

impl From<redis::RedisError> for DeFiError {
    fn from(err: redis::RedisError) -> Self {
        DeFiError::database_error(format!("Redis error: {}", err))
    }
}

impl From<reqwest::Error> for DeFiError {
    fn from(err: reqwest::Error) -> Self {
        DeFiError::network_error(err.to_string())
    }
}

impl From<serde_json::Error> for DeFiError {
    fn from(err: serde_json::Error) -> Self {
        DeFiError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<tokio::time::error::Elapsed> for DeFiError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        DeFiError::Timeout {
            operation: err.to_string(),
        }
    }
}

/// Helper function to create database error
impl DeFiError {
    pub fn database_error<S: Into<String>>(message: S) -> Self {
        Self::DatabaseError {
            message: message.into(),
        }
    }

    pub fn network_error<S: Into<String>>(message: S) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = DeFiError::validation_error("amount", "Amount must be positive");
        assert_eq!(error.error_code(), "VALIDATION_ERROR");
        assert!(error.is_user_error());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_classification() {
        let user_error = DeFiError::insufficient_balance("100", "50");
        assert!(user_error.is_user_error());
        assert!(!user_error.is_system_error());

        let system_error = DeFiError::internal_error("Database connection failed");
        assert!(system_error.is_system_error());
        assert!(!system_error.is_user_error());

        let retryable_error = DeFiError::network_error("Connection timeout");
        assert!(retryable_error.is_retryable());
    }

    #[test]
    fn test_defi_specific_errors() {
        let slippage_error = DeFiError::slippage_exceeded("1%", "5%");
        assert_eq!(slippage_error.error_code(), "SLIPPAGE_EXCEEDED");

        let liquidity_error = DeFiError::insufficient_liquidity("Pool has insufficient liquidity");
        assert_eq!(liquidity_error.error_code(), "INSUFFICIENT_LIQUIDITY");

        let flash_loan_error = DeFiError::flash_loan_error("Flash loan callback failed");
        assert_eq!(flash_loan_error.error_code(), "FLASH_LOAN_ERROR");
    }
}
