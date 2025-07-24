// =====================================================================================
// File: core-stablecoin/src/error.rs
// Description: Error types and handling for stablecoin operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;

/// Result type for stablecoin operations
pub type StablecoinResult<T> = Result<T, StablecoinError>;

/// Comprehensive error types for stablecoin operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StablecoinError {
    // Configuration errors
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    // Validation errors
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid token symbol: {0}")]
    InvalidSymbol(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    // Business logic errors
    #[error("Insufficient collateral")]
    InsufficientCollateral,

    #[error("Insufficient collateral: required {required}, available {available}")]
    InsufficientCollateralDetailed { required: String, available: String },

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    #[error("Collateral ratio too low: current {current}, minimum {minimum}")]
    CollateralRatioTooLow { current: String, minimum: String },

    #[error("Price deviation too high: current {current}, target {target}, max_deviation {max_deviation}")]
    PriceDeviationTooHigh { current: String, target: String, max_deviation: String },

    #[error("Stability mechanism not supported: {0}")]
    UnsupportedStabilityMechanism(String),

    #[error("Redemption not allowed: {0}")]
    RedemptionNotAllowed(String),

    #[error("Issuance not allowed: {0}")]
    IssuanceNotAllowed(String),

    // State errors
    #[error("Stablecoin not found: {0}")]
    StablecoinNotFound(String),

    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Collateral not found: {0}")]
    CollateralNotFound(String),

    #[error("Transaction not found")]
    TransactionNotFound,

    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Operation not permitted in current state: {0}")]
    OperationNotPermitted(String),

    // Compliance errors
    #[error("KYC verification required for user: {0}")]
    KYCRequired(String),

    #[error("AML check failed: {0}")]
    AMLCheckFailed(String),

    #[error("Regulatory limit exceeded: {0}")]
    RegulatoryLimitExceeded(String),

    #[error("Jurisdiction not supported: {0}")]
    UnsupportedJurisdiction(String),

    #[error("Compliance error: {0}")]
    ComplianceError(String),

    // Risk management errors
    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    #[error("Liquidation threshold reached: {0}")]
    LiquidationThreshold(String),

    #[error("Emergency pause activated: {0}")]
    EmergencyPause(String),

    // Blockchain errors
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Smart contract error: {0}")]
    SmartContractError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),

    // Oracle errors
    #[error("Price oracle unavailable: {0}")]
    OracleUnavailable(String),

    #[error("Stale price data: last_update {last_update}, max_age {max_age}")]
    StalePriceData { last_update: String, max_age: String },

    #[error("Price feed manipulation detected: {0}")]
    PriceFeedManipulation(String),

    // Governance errors
    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),

    #[error("Voting period ended: {0}")]
    VotingPeriodEnded(String),

    #[error("Insufficient voting power: required {required}, available {available}")]
    InsufficientVotingPower { required: String, available: String },

    // System errors
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    // Generic errors
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
}

impl StablecoinError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StablecoinError::NetworkError(_)
                | StablecoinError::ServiceUnavailable(_)
                | StablecoinError::TimeoutError(_)
                | StablecoinError::DatabaseError(_)
                | StablecoinError::OracleUnavailable(_)
        )
    }

    /// Check if error is critical (requires immediate attention)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            StablecoinError::EmergencyPause(_)
                | StablecoinError::LiquidationThreshold(_)
                | StablecoinError::PriceFeedManipulation(_)
                | StablecoinError::SmartContractError(_)
        )
    }

    /// Get error category for monitoring and alerting
    pub fn category(&self) -> &'static str {
        match self {
            StablecoinError::InvalidConfiguration(_) | StablecoinError::MissingParameter(_) => "configuration",
            StablecoinError::InvalidAmount(_) | StablecoinError::InvalidAddress(_) | StablecoinError::InvalidSymbol(_) => "validation",
            StablecoinError::InsufficientCollateral
            | StablecoinError::InsufficientCollateralDetailed { .. }
            | StablecoinError::InsufficientBalance { .. } => "business_logic",
            StablecoinError::KYCRequired(_) | StablecoinError::AMLCheckFailed(_) => "compliance",
            StablecoinError::RiskLimitExceeded(_) | StablecoinError::LiquidationThreshold(_) => "risk_management",
            StablecoinError::TransactionFailed(_) | StablecoinError::SmartContractError(_) => "blockchain",
            StablecoinError::OracleUnavailable(_) | StablecoinError::StalePriceData { .. } => "oracle",
            StablecoinError::ProposalNotFound(_) | StablecoinError::VotingPeriodEnded(_) => "governance",
            _ => "system",
        }
    }
}

// Integration with other error types
impl From<core_utils::UtilError> for StablecoinError {
    fn from(err: core_utils::UtilError) -> Self {
        StablecoinError::InternalError(err.to_string())
    }
}

impl From<core_security::SecurityError> for StablecoinError {
    fn from(err: core_security::SecurityError) -> Self {
        match err {
            core_security::SecurityError::AuthenticationFailed(_) => {
                StablecoinError::AuthenticationError(err.to_string())
            }
            core_security::SecurityError::AuthorizationFailed(_) => {
                StablecoinError::AuthorizationError(err.to_string())
            }
            _ => StablecoinError::InternalError(err.to_string()),
        }
    }
}

impl From<core_compliance::ComplianceError> for StablecoinError {
    fn from(err: core_compliance::ComplianceError) -> Self {
        StablecoinError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for StablecoinError {
    fn from(err: serde_json::Error) -> Self {
        StablecoinError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        // Configuration errors
        assert_eq!(StablecoinError::InvalidConfiguration("test".to_string()).category(), "configuration");
        assert_eq!(StablecoinError::MissingParameter("param".to_string()).category(), "configuration");

        // Validation errors
        assert_eq!(StablecoinError::InvalidAmount("test".to_string()).category(), "validation");
        assert_eq!(StablecoinError::InvalidAddress("addr".to_string()).category(), "validation");
        assert_eq!(StablecoinError::InvalidSymbol("SYM".to_string()).category(), "validation");

        // Business logic errors
        assert_eq!(StablecoinError::InsufficientCollateral.category(), "business_logic");
        assert_eq!(StablecoinError::InsufficientCollateralDetailed {
            required: "100".to_string(),
            available: "50".to_string()
        }.category(), "business_logic");
        assert_eq!(StablecoinError::InsufficientBalance {
            required: "100".to_string(),
            available: "50".to_string()
        }.category(), "business_logic");

        // Compliance errors
        assert_eq!(StablecoinError::KYCRequired("user123".to_string()).category(), "compliance");
        assert_eq!(StablecoinError::AMLCheckFailed("suspicious".to_string()).category(), "compliance");

        // Risk management errors
        assert_eq!(StablecoinError::RiskLimitExceeded("limit".to_string()).category(), "risk_management");
        assert_eq!(StablecoinError::LiquidationThreshold("threshold".to_string()).category(), "risk_management");

        // Blockchain errors
        assert_eq!(StablecoinError::TransactionFailed("tx".to_string()).category(), "blockchain");
        assert_eq!(StablecoinError::SmartContractError("contract".to_string()).category(), "blockchain");

        // Oracle errors
        assert_eq!(StablecoinError::OracleUnavailable("oracle".to_string()).category(), "oracle");
        assert_eq!(StablecoinError::StalePriceData {
            last_update: "2023-01-01".to_string(),
            max_age: "300".to_string()
        }.category(), "oracle");

        // Governance errors
        assert_eq!(StablecoinError::ProposalNotFound("prop123".to_string()).category(), "governance");
        assert_eq!(StablecoinError::VotingPeriodEnded("vote123".to_string()).category(), "governance");

        // System errors (default)
        assert_eq!(StablecoinError::NetworkError("timeout".to_string()).category(), "system");
        assert_eq!(StablecoinError::InternalError("internal".to_string()).category(), "system");
    }

    #[test]
    fn test_retryable_errors() {
        // Retryable errors
        assert!(StablecoinError::NetworkError("timeout".to_string()).is_retryable());
        assert!(StablecoinError::ServiceUnavailable("service down".to_string()).is_retryable());
        assert!(StablecoinError::TimeoutError("timeout".to_string()).is_retryable());
        assert!(StablecoinError::DatabaseError("connection lost".to_string()).is_retryable());
        assert!(StablecoinError::OracleUnavailable("oracle down".to_string()).is_retryable());

        // Non-retryable errors
        assert!(!StablecoinError::InvalidAmount("negative".to_string()).is_retryable());
        assert!(!StablecoinError::InvalidAddress("invalid".to_string()).is_retryable());
        assert!(!StablecoinError::KYCRequired("user123".to_string()).is_retryable());
        assert!(!StablecoinError::EmergencyPause("system halt".to_string()).is_retryable());
    }

    #[test]
    fn test_critical_errors() {
        // Critical errors
        assert!(StablecoinError::EmergencyPause("system halt".to_string()).is_critical());
        assert!(StablecoinError::LiquidationThreshold("threshold reached".to_string()).is_critical());
        assert!(StablecoinError::PriceFeedManipulation("manipulation detected".to_string()).is_critical());
        assert!(StablecoinError::SmartContractError("contract error".to_string()).is_critical());

        // Non-critical errors
        assert!(!StablecoinError::InvalidAmount("negative".to_string()).is_critical());
        assert!(!StablecoinError::NetworkError("timeout".to_string()).is_critical());
        assert!(!StablecoinError::KYCRequired("user123".to_string()).is_critical());
    }

    #[test]
    fn test_error_conversion_from_util_error() {
        let util_error = core_utils::UtilError::ProcessingError("test".to_string());
        let stablecoin_error: StablecoinError = util_error.into();

        match stablecoin_error {
            StablecoinError::InternalError(msg) => assert!(msg.contains("test")),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_error_conversion_from_security_error() {
        // Test authentication error conversion
        let auth_error = core_security::SecurityError::AuthenticationFailed("invalid token".to_string());
        let stablecoin_error: StablecoinError = auth_error.into();

        match stablecoin_error {
            StablecoinError::AuthenticationError(msg) => assert!(msg.contains("invalid token")),
            _ => panic!("Expected AuthenticationError"),
        }

        // Test authorization error conversion
        let authz_error = core_security::SecurityError::AuthorizationFailed("insufficient permissions".to_string());
        let stablecoin_error: StablecoinError = authz_error.into();

        match stablecoin_error {
            StablecoinError::AuthorizationError(msg) => assert!(msg.contains("insufficient permissions")),
            _ => panic!("Expected AuthorizationError"),
        }

        // Test other security error conversion
        let other_error = core_security::SecurityError::EncryptionError("encryption failed".to_string());
        let stablecoin_error: StablecoinError = other_error.into();

        match stablecoin_error {
            StablecoinError::InternalError(msg) => assert!(msg.contains("encryption failed")),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_error_conversion_from_compliance_error() {
        let compliance_error = core_compliance::ComplianceError::kyc_error("user123");
        let stablecoin_error: StablecoinError = compliance_error.into();

        match stablecoin_error {
            StablecoinError::InternalError(msg) => assert!(msg.contains("user123")),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_error_conversion_from_serde_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json")
            .unwrap_err();
        let stablecoin_error: StablecoinError = json_error.into();

        match stablecoin_error {
            StablecoinError::SerializationError(_) => {},
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = StablecoinError::InvalidAmount("negative value".to_string());
        assert_eq!(error.to_string(), "Invalid amount: negative value");

        let error = StablecoinError::InsufficientCollateralDetailed {
            required: "1000".to_string(),
            available: "500".to_string(),
        };
        assert_eq!(error.to_string(), "Insufficient collateral: required 1000, available 500");
    }

    #[test]
    fn test_error_debug() {
        let error = StablecoinError::InvalidAmount("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidAmount"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_error_clone() {
        let error = StablecoinError::InvalidAmount("test".to_string());
        let cloned_error = error.clone();
        assert_eq!(error, cloned_error);
    }

    #[test]
    fn test_error_partial_eq() {
        let error1 = StablecoinError::InvalidAmount("test".to_string());
        let error2 = StablecoinError::InvalidAmount("test".to_string());
        let error3 = StablecoinError::InvalidAmount("different".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_all_error_variants_have_categories() {
        // Test that all error variants have proper categories
        let errors = vec![
            StablecoinError::InvalidConfiguration("test".to_string()),
            StablecoinError::MissingParameter("test".to_string()),
            StablecoinError::InvalidAmount("test".to_string()),
            StablecoinError::InvalidAddress("test".to_string()),
            StablecoinError::InvalidSymbol("test".to_string()),
            StablecoinError::InsufficientCollateralDetailed { required: "100".to_string(), available: "50".to_string() },
            StablecoinError::InsufficientBalance { required: "100".to_string(), available: "50".to_string() },
            StablecoinError::KYCRequired("test".to_string()),
            StablecoinError::AMLCheckFailed("test".to_string()),
            StablecoinError::RiskLimitExceeded("test".to_string()),
            StablecoinError::LiquidationThreshold("test".to_string()),
            StablecoinError::EmergencyPause("test".to_string()),
            StablecoinError::TransactionFailed("test".to_string()),
            StablecoinError::SmartContractError("test".to_string()),
            StablecoinError::NetworkError("test".to_string()),
            StablecoinError::GasEstimationFailed("test".to_string()),
            StablecoinError::OracleUnavailable("test".to_string()),
            StablecoinError::StalePriceData { last_update: "test".to_string(), max_age: "test".to_string() },
            StablecoinError::PriceFeedManipulation("test".to_string()),
            StablecoinError::ProposalNotFound("test".to_string()),
            StablecoinError::VotingPeriodEnded("test".to_string()),
            StablecoinError::InternalError("test".to_string()),
            StablecoinError::ServiceUnavailable("test".to_string()),
            StablecoinError::TimeoutError("test".to_string()),
            StablecoinError::DatabaseError("test".to_string()),
            StablecoinError::SerializationError("test".to_string()),
            StablecoinError::AuthenticationError("test".to_string()),
            StablecoinError::AuthorizationError("test".to_string()),
            StablecoinError::PositionNotFound("test".to_string()),
            StablecoinError::OperationNotPermitted("test".to_string()),
            StablecoinError::ValidationError { field: "test".to_string(), message: "test".to_string() },
        ];

        for error in errors {
            let category = error.category();
            assert!(!category.is_empty(), "Error {:?} should have a category", error);
            assert!(matches!(
                category,
                "configuration" | "validation" | "business_logic" | "compliance" |
                "risk_management" | "blockchain" | "oracle" | "governance" | "system"
            ), "Invalid category '{}' for error {:?}", category, error);
        }
    }
}
