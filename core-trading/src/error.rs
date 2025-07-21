// =====================================================================================
// File: core-trading/src/error.rs
// Description: Error types for trading system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type alias for trading operations
pub type TradingResult<T> = Result<T, TradingError>;

/// Comprehensive error types for trading operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum TradingError {
    /// Order-related errors
    #[error("Order error: {message}")]
    OrderError { message: String },

    /// Order book errors
    #[error("Order book error: {trading_pair} - {message}")]
    OrderBookError { trading_pair: String, message: String },

    /// Matching engine errors
    #[error("Matching engine error: {message}")]
    MatchingError { message: String },

    /// Settlement errors
    #[error("Settlement error: {trade_id} - {message}")]
    SettlementError { trade_id: String, message: String },

    /// Liquidity errors
    #[error("Liquidity error: {pool_id} - {message}")]
    LiquidityError { pool_id: String, message: String },

    /// Market data errors
    #[error("Market data error: {source} - {message}")]
    MarketDataError { source: String, message: String },

    /// Price discovery errors
    #[error("Price discovery error: {message}")]
    PriceDiscoveryError { message: String },

    /// Market making errors
    #[error("Market making error: {strategy} - {message}")]
    MarketMakingError { strategy: String, message: String },

    /// Risk control errors
    #[error("Risk control error: {control_type} - {message}")]
    RiskControlError { control_type: String, message: String },

    /// Invalid order
    #[error("Invalid order: {reason}")]
    InvalidOrder { reason: String },

    /// Order not found
    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: String },

    /// Trading pair not supported
    #[error("Trading pair not supported: {trading_pair}")]
    UnsupportedTradingPair { trading_pair: String },

    /// Insufficient balance
    #[error("Insufficient balance: {asset} - required {required}, available {available}")]
    InsufficientBalance { asset: String, required: String, available: String },

    /// Insufficient liquidity
    #[error("Insufficient liquidity: {trading_pair} - {message}")]
    InsufficientLiquidity { trading_pair: String, message: String },

    /// Price out of range
    #[error("Price out of range: {price} not within [{min_price}, {max_price}]")]
    PriceOutOfRange { price: String, min_price: String, max_price: String },

    /// Quantity below minimum
    #[error("Quantity below minimum: {quantity} < {min_quantity}")]
    QuantityBelowMinimum { quantity: String, min_quantity: String },

    /// Order size too large
    #[error("Order size too large: {size} > {max_size}")]
    OrderSizeTooLarge { size: String, max_size: String },

    /// Market closed
    #[error("Market closed: {trading_pair} - {reason}")]
    MarketClosed { trading_pair: String, reason: String },

    /// Trading halted
    #[error("Trading halted: {trading_pair} - {reason}")]
    TradingHalted { trading_pair: String, reason: String },

    /// Circuit breaker triggered
    #[error("Circuit breaker triggered: {trigger_type} - {message}")]
    CircuitBreakerTriggered { trigger_type: String, message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {limit_type} - {message}")]
    RateLimitExceeded { limit_type: String, message: String },

    /// Order expired
    #[error("Order expired: {order_id} - expired at {expired_at}")]
    OrderExpired { order_id: String, expired_at: String },

    /// Self-trade prevention
    #[error("Self-trade prevented: user {user_id} cannot trade with themselves")]
    SelfTradePrevented { user_id: String },

    /// Slippage too high
    #[error("Slippage too high: expected {expected}%, actual {actual}%")]
    SlippageTooHigh { expected: String, actual: String },

    /// Position limit exceeded
    #[error("Position limit exceeded: {asset} - current {current}, limit {limit}")]
    PositionLimitExceeded { asset: String, current: String, limit: String },

    /// Margin requirement not met
    #[error("Margin requirement not met: required {required}, available {available}")]
    MarginRequirementNotMet { required: String, available: String },

    /// Trade already settled
    #[error("Trade already settled: {trade_id}")]
    TradeAlreadySettled { trade_id: String },

    /// Settlement timeout
    #[error("Settlement timeout: {trade_id} - timeout after {timeout_seconds}s")]
    SettlementTimeout { trade_id: String, timeout_seconds: u64 },

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

    /// Concurrent modification error
    #[error("Concurrent modification detected for {resource_type} {resource_id}")]
    ConcurrentModification { resource_type: String, resource_id: String },

    /// Business rule violation
    #[error("Business rule violation: {rule} - {message}")]
    BusinessRuleViolation { rule: String, message: String },

    /// Internal system errors
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Generic trading error
    #[error("Trading error: {0}")]
    Generic(String),
}

impl TradingError {
    /// Create an order error
    pub fn order_error<S: Into<String>>(message: S) -> Self {
        Self::OrderError { message: message.into() }
    }

    /// Create an order book error
    pub fn order_book_error<S: Into<String>>(trading_pair: S, message: S) -> Self {
        Self::OrderBookError {
            trading_pair: trading_pair.into(),
            message: message.into(),
        }
    }

    /// Create a matching error
    pub fn matching_error<S: Into<String>>(message: S) -> Self {
        Self::MatchingError { message: message.into() }
    }

    /// Create a settlement error
    pub fn settlement_error<S: Into<String>>(trade_id: S, message: S) -> Self {
        Self::SettlementError {
            trade_id: trade_id.into(),
            message: message.into(),
        }
    }

    /// Create an invalid order error
    pub fn invalid_order<S: Into<String>>(reason: S) -> Self {
        Self::InvalidOrder { reason: reason.into() }
    }

    /// Create an order not found error
    pub fn order_not_found<S: Into<String>>(order_id: S) -> Self {
        Self::OrderNotFound { order_id: order_id.into() }
    }

    /// Create an unsupported trading pair error
    pub fn unsupported_trading_pair<S: Into<String>>(trading_pair: S) -> Self {
        Self::UnsupportedTradingPair { trading_pair: trading_pair.into() }
    }

    /// Create an insufficient balance error
    pub fn insufficient_balance<S: Into<String>>(asset: S, required: S, available: S) -> Self {
        Self::InsufficientBalance {
            asset: asset.into(),
            required: required.into(),
            available: available.into(),
        }
    }

    /// Create an insufficient liquidity error
    pub fn insufficient_liquidity<S: Into<String>>(trading_pair: S, message: S) -> Self {
        Self::InsufficientLiquidity {
            trading_pair: trading_pair.into(),
            message: message.into(),
        }
    }

    /// Create a market closed error
    pub fn market_closed<S: Into<String>>(trading_pair: S, reason: S) -> Self {
        Self::MarketClosed {
            trading_pair: trading_pair.into(),
            reason: reason.into(),
        }
    }

    /// Create a circuit breaker triggered error
    pub fn circuit_breaker_triggered<S: Into<String>>(trigger_type: S, message: S) -> Self {
        Self::CircuitBreakerTriggered {
            trigger_type: trigger_type.into(),
            message: message.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded<S: Into<String>>(limit_type: S, message: S) -> Self {
        Self::RateLimitExceeded {
            limit_type: limit_type.into(),
            message: message.into(),
        }
    }

    /// Create a self-trade prevented error
    pub fn self_trade_prevented<S: Into<String>>(user_id: S) -> Self {
        Self::SelfTradePrevented { user_id: user_id.into() }
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
            TradingError::NetworkError { .. }
                | TradingError::DatabaseError { .. }
                | TradingError::ExternalServiceError { .. }
                | TradingError::SettlementTimeout { .. }
                | TradingError::InternalError { .. }
        )
    }

    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            TradingError::OrderError { .. } => "order",
            TradingError::OrderBookError { .. } => "order_book",
            TradingError::MatchingError { .. } => "matching",
            TradingError::SettlementError { .. } => "settlement",
            TradingError::LiquidityError { .. } => "liquidity",
            TradingError::MarketDataError { .. } => "market_data",
            TradingError::PriceDiscoveryError { .. } => "price_discovery",
            TradingError::MarketMakingError { .. } => "market_making",
            TradingError::RiskControlError { .. } => "risk_control",
            TradingError::InvalidOrder { .. } => "invalid_order",
            TradingError::OrderNotFound { .. } => "order_not_found",
            TradingError::UnsupportedTradingPair { .. } => "unsupported_pair",
            TradingError::InsufficientBalance { .. } => "insufficient_balance",
            TradingError::InsufficientLiquidity { .. } => "insufficient_liquidity",
            TradingError::PriceOutOfRange { .. } => "price_out_of_range",
            TradingError::QuantityBelowMinimum { .. } => "quantity_below_minimum",
            TradingError::OrderSizeTooLarge { .. } => "order_size_too_large",
            TradingError::MarketClosed { .. } => "market_closed",
            TradingError::TradingHalted { .. } => "trading_halted",
            TradingError::CircuitBreakerTriggered { .. } => "circuit_breaker",
            TradingError::RateLimitExceeded { .. } => "rate_limit",
            TradingError::OrderExpired { .. } => "order_expired",
            TradingError::SelfTradePrevented { .. } => "self_trade",
            TradingError::SlippageTooHigh { .. } => "slippage",
            TradingError::PositionLimitExceeded { .. } => "position_limit",
            TradingError::MarginRequirementNotMet { .. } => "margin_requirement",
            TradingError::TradeAlreadySettled { .. } => "trade_settled",
            TradingError::SettlementTimeout { .. } => "settlement_timeout",
            TradingError::ConfigurationError { .. } => "configuration",
            TradingError::ExternalServiceError { .. } => "external_service",
            TradingError::DatabaseError { .. } => "database",
            TradingError::NetworkError { .. } => "network",
            TradingError::SerializationError { .. } => "serialization",
            TradingError::AuthenticationError { .. } => "authentication",
            TradingError::ValidationError { .. } => "validation",
            TradingError::ConcurrentModification { .. } => "concurrency",
            TradingError::BusinessRuleViolation { .. } => "business_rule",
            TradingError::InternalError { .. } => "internal",
            TradingError::Generic(_) => "generic",
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            TradingError::InternalError { .. } => ErrorSeverity::Critical,
            TradingError::CircuitBreakerTriggered { .. } => ErrorSeverity::Critical,
            TradingError::TradingHalted { .. } => ErrorSeverity::High,
            TradingError::SettlementError { .. } => ErrorSeverity::High,
            TradingError::DatabaseError { .. } => ErrorSeverity::High,
            TradingError::AuthenticationError { .. } => ErrorSeverity::High,
            TradingError::BusinessRuleViolation { .. } => ErrorSeverity::High,
            TradingError::MarketClosed { .. } => ErrorSeverity::Medium,
            TradingError::RateLimitExceeded { .. } => ErrorSeverity::Medium,
            TradingError::InsufficientBalance { .. } => ErrorSeverity::Medium,
            TradingError::InsufficientLiquidity { .. } => ErrorSeverity::Medium,
            TradingError::ValidationError { .. } => ErrorSeverity::Medium,
            TradingError::OrderNotFound { .. } => ErrorSeverity::Low,
            TradingError::UnsupportedTradingPair { .. } => ErrorSeverity::Low,
            TradingError::NetworkError { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }

    /// Check if error requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Critical | ErrorSeverity::High)
    }

    /// Check if error should halt trading
    pub fn should_halt_trading(&self) -> bool {
        matches!(
            self,
            TradingError::CircuitBreakerTriggered { .. }
                | TradingError::InternalError { .. }
                | TradingError::DatabaseError { .. }
        )
    }

    /// Check if error is user-facing (vs system error)
    pub fn is_user_facing(&self) -> bool {
        matches!(
            self,
            TradingError::InvalidOrder { .. }
                | TradingError::OrderNotFound { .. }
                | TradingError::UnsupportedTradingPair { .. }
                | TradingError::InsufficientBalance { .. }
                | TradingError::InsufficientLiquidity { .. }
                | TradingError::PriceOutOfRange { .. }
                | TradingError::QuantityBelowMinimum { .. }
                | TradingError::OrderSizeTooLarge { .. }
                | TradingError::MarketClosed { .. }
                | TradingError::RateLimitExceeded { .. }
                | TradingError::OrderExpired { .. }
                | TradingError::SelfTradePrevented { .. }
                | TradingError::SlippageTooHigh { .. }
                | TradingError::ValidationError { .. }
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
impl From<serde_json::Error> for TradingError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError { message: err.to_string() }
    }
}

impl From<reqwest::Error> for TradingError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError { message: err.to_string() }
    }
}

#[cfg(feature = "database")]
impl From<sqlx::Error> for TradingError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError { message: err.to_string() }
    }
}

impl From<validator::ValidationErrors> for TradingError {
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
        let order_error = TradingError::order_error("Invalid order parameters");
        assert!(matches!(order_error, TradingError::OrderError { .. }));
        assert_eq!(order_error.category(), "order");
    }

    #[test]
    fn test_error_retryability() {
        let network_error = TradingError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(network_error.is_retryable());

        let validation_error = TradingError::ValidationError {
            field: "price".to_string(),
            message: "Invalid price".to_string(),
        };
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let circuit_breaker = TradingError::CircuitBreakerTriggered {
            trigger_type: "price_change".to_string(),
            message: "Price moved too fast".to_string(),
        };
        assert_eq!(circuit_breaker.severity(), ErrorSeverity::Critical);
        assert!(circuit_breaker.requires_immediate_attention());
        assert!(circuit_breaker.should_halt_trading());

        let order_not_found = TradingError::OrderNotFound {
            order_id: "order123".to_string(),
        };
        assert_eq!(order_not_found.severity(), ErrorSeverity::Low);
        assert!(!order_not_found.requires_immediate_attention());
        assert!(!order_not_found.should_halt_trading());
    }

    #[test]
    fn test_user_facing_errors() {
        let insufficient_balance = TradingError::InsufficientBalance {
            asset: "BTC".to_string(),
            required: "1.0".to_string(),
            available: "0.5".to_string(),
        };
        assert!(insufficient_balance.is_user_facing());

        let internal_error = TradingError::InternalError {
            message: "System failure".to_string(),
        };
        assert!(!internal_error.is_user_facing());
    }

    #[test]
    fn test_specific_error_constructors() {
        let market_closed = TradingError::market_closed("BTC/USD", "Weekend");
        assert!(matches!(market_closed, TradingError::MarketClosed { .. }));

        let self_trade = TradingError::self_trade_prevented("user123");
        assert!(matches!(self_trade, TradingError::SelfTradePrevented { .. }));

        let insufficient_balance = TradingError::insufficient_balance("BTC", "1.0", "0.5");
        assert!(matches!(insufficient_balance, TradingError::InsufficientBalance { .. }));
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            TradingError::order_error("test"),
            TradingError::matching_error("test"),
            TradingError::settlement_error("trade123", "test"),
            TradingError::invalid_order("test"),
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(categories, vec!["order", "matching", "settlement", "invalid_order"]);
    }
}
