// =====================================================================================
// File: core-risk-management/src/error.rs
// Description: Error types for risk management
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type alias for risk management operations
pub type RiskResult<T> = Result<T, RiskError>;

/// Comprehensive error types for risk management operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum RiskError {
    /// Risk assessment errors
    #[error("Risk assessment error: {message}")]
    AssessmentError { message: String },

    /// Insurance-related errors
    #[error("Insurance error: {message}")]
    InsuranceError { message: String },

    /// Hedging strategy errors
    #[error("Hedging error: {message}")]
    HedgingError { message: String },

    /// Risk monitoring errors
    #[error("Monitoring error: {message}")]
    MonitoringError { message: String },

    /// Emergency response errors
    #[error("Emergency response error: {message}")]
    EmergencyError { message: String },

    /// Risk model errors
    #[error("Model error: {model_type} - {message}")]
    ModelError { model_type: String, message: String },

    /// Data quality errors
    #[error("Data quality error: {source} - {message}")]
    DataQualityError { source: String, message: String },

    /// Risk limit breach
    #[error("Risk limit breach: {limit_type} - current: {current_value}, limit: {limit_value}")]
    RiskLimitBreach {
        limit_type: String,
        current_value: String,
        limit_value: String,
    },

    /// Insufficient risk data
    #[error("Insufficient risk data: {data_type} - {message}")]
    InsufficientData { data_type: String, message: String },

    /// Risk calculation errors
    #[error("Calculation error: {calculation_type} - {message}")]
    CalculationError {
        calculation_type: String,
        message: String,
    },

    /// Market data errors
    #[error("Market data error: {provider} - {message}")]
    MarketDataError { provider: String, message: String },

    /// Scenario analysis errors
    #[error("Scenario analysis error: {scenario} - {message}")]
    ScenarioError { scenario: String, message: String },

    /// Stress testing errors
    #[error("Stress test error: {test_type} - {message}")]
    StressTestError { test_type: String, message: String },

    /// Portfolio risk errors
    #[error("Portfolio risk error: {portfolio_id} - {message}")]
    PortfolioRiskError { portfolio_id: String, message: String },

    /// Asset risk errors
    #[error("Asset risk error: {asset_id} - {message}")]
    AssetRiskError { asset_id: String, message: String },

    /// Counterparty risk errors
    #[error("Counterparty risk error: {counterparty} - {message}")]
    CounterpartyRiskError {
        counterparty: String,
        message: String,
    },

    /// Regulatory compliance errors
    #[error("Regulatory compliance error: {regulation} - {message}")]
    RegulatoryError {
        regulation: String,
        message: String,
    },

    /// Risk reporting errors
    #[error("Risk reporting error: {report_type} - {message}")]
    ReportingError {
        report_type: String,
        message: String,
    },

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
    InsufficientPermissions {
        user_id: String,
        required_permission: String,
    },

    /// Concurrent modification error
    #[error("Concurrent modification detected for {resource_type} {resource_id}")]
    ConcurrentModification {
        resource_type: String,
        resource_id: String,
    },

    /// Business rule violation
    #[error("Business rule violation: {rule} - {message}")]
    BusinessRuleViolation { rule: String, message: String },

    /// Risk threshold exceeded
    #[error("Risk threshold exceeded: {threshold_type} - {message}")]
    ThresholdExceeded {
        threshold_type: String,
        message: String,
    },

    /// Model validation failed
    #[error("Model validation failed: {model_name} - {message}")]
    ModelValidationFailed {
        model_name: String,
        message: String,
    },

    /// Backtesting failed
    #[error("Backtesting failed: {model_name} - {message}")]
    BacktestingFailed {
        model_name: String,
        message: String,
    },

    /// Risk aggregation error
    #[error("Risk aggregation error: {aggregation_type} - {message}")]
    AggregationError {
        aggregation_type: String,
        message: String,
    },

    /// Internal system errors
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Generic risk error
    #[error("Risk error: {0}")]
    Generic(String),
}

impl RiskError {
    /// Create an assessment error
    pub fn assessment_error<S: Into<String>>(message: S) -> Self {
        Self::AssessmentError {
            message: message.into(),
        }
    }

    /// Create an insurance error
    pub fn insurance_error<S: Into<String>>(message: S) -> Self {
        Self::InsuranceError {
            message: message.into(),
        }
    }

    /// Create a hedging error
    pub fn hedging_error<S: Into<String>>(message: S) -> Self {
        Self::HedgingError {
            message: message.into(),
        }
    }

    /// Create a monitoring error
    pub fn monitoring_error<S: Into<String>>(message: S) -> Self {
        Self::MonitoringError {
            message: message.into(),
        }
    }

    /// Create a model error
    pub fn model_error<S: Into<String>>(model_type: S, message: S) -> Self {
        Self::ModelError {
            model_type: model_type.into(),
            message: message.into(),
        }
    }

    /// Create a risk limit breach error
    pub fn risk_limit_breach<S: Into<String>>(
        limit_type: S,
        current_value: S,
        limit_value: S,
    ) -> Self {
        Self::RiskLimitBreach {
            limit_type: limit_type.into(),
            current_value: current_value.into(),
            limit_value: limit_value.into(),
        }
    }

    /// Create a calculation error
    pub fn calculation_error<S: Into<String>>(calculation_type: S, message: S) -> Self {
        Self::CalculationError {
            calculation_type: calculation_type.into(),
            message: message.into(),
        }
    }

    /// Create a market data error
    pub fn market_data_error<S: Into<String>>(provider: S, message: S) -> Self {
        Self::MarketDataError {
            provider: provider.into(),
            message: message.into(),
        }
    }

    /// Create a portfolio risk error
    pub fn portfolio_risk_error<S: Into<String>>(portfolio_id: S, message: S) -> Self {
        Self::PortfolioRiskError {
            portfolio_id: portfolio_id.into(),
            message: message.into(),
        }
    }

    /// Create an asset risk error
    pub fn asset_risk_error<S: Into<String>>(asset_id: S, message: S) -> Self {
        Self::AssetRiskError {
            asset_id: asset_id.into(),
            message: message.into(),
        }
    }

    /// Create a threshold exceeded error
    pub fn threshold_exceeded<S: Into<String>>(threshold_type: S, message: S) -> Self {
        Self::ThresholdExceeded {
            threshold_type: threshold_type.into(),
            message: message.into(),
        }
    }

    /// Create a model validation failed error
    pub fn model_validation_failed<S: Into<String>>(model_name: S, message: S) -> Self {
        Self::ModelValidationFailed {
            model_name: model_name.into(),
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

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            RiskError::NetworkError { .. }
                | RiskError::DatabaseError { .. }
                | RiskError::ExternalServiceError { .. }
                | RiskError::MarketDataError { .. }
                | RiskError::InternalError { .. }
        )
    }

    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            RiskError::AssessmentError { .. } => "assessment",
            RiskError::InsuranceError { .. } => "insurance",
            RiskError::HedgingError { .. } => "hedging",
            RiskError::MonitoringError { .. } => "monitoring",
            RiskError::EmergencyError { .. } => "emergency",
            RiskError::ModelError { .. } => "model",
            RiskError::DataQualityError { .. } => "data_quality",
            RiskError::RiskLimitBreach { .. } => "limit_breach",
            RiskError::InsufficientData { .. } => "insufficient_data",
            RiskError::CalculationError { .. } => "calculation",
            RiskError::MarketDataError { .. } => "market_data",
            RiskError::ScenarioError { .. } => "scenario",
            RiskError::StressTestError { .. } => "stress_test",
            RiskError::PortfolioRiskError { .. } => "portfolio_risk",
            RiskError::AssetRiskError { .. } => "asset_risk",
            RiskError::CounterpartyRiskError { .. } => "counterparty_risk",
            RiskError::RegulatoryError { .. } => "regulatory",
            RiskError::ReportingError { .. } => "reporting",
            RiskError::ConfigurationError { .. } => "configuration",
            RiskError::ExternalServiceError { .. } => "external_service",
            RiskError::DatabaseError { .. } => "database",
            RiskError::NetworkError { .. } => "network",
            RiskError::SerializationError { .. } => "serialization",
            RiskError::AuthenticationError { .. } => "authentication",
            RiskError::ValidationError { .. } => "validation",
            RiskError::InsufficientPermissions { .. } => "permissions",
            RiskError::ConcurrentModification { .. } => "concurrency",
            RiskError::BusinessRuleViolation { .. } => "business_rule",
            RiskError::ThresholdExceeded { .. } => "threshold",
            RiskError::ModelValidationFailed { .. } => "model_validation",
            RiskError::BacktestingFailed { .. } => "backtesting",
            RiskError::AggregationError { .. } => "aggregation",
            RiskError::InternalError { .. } => "internal",
            RiskError::Generic(_) => "generic",
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            RiskError::InternalError { .. } => ErrorSeverity::Critical,
            RiskError::RiskLimitBreach { .. } => ErrorSeverity::Critical,
            RiskError::ThresholdExceeded { .. } => ErrorSeverity::High,
            RiskError::EmergencyError { .. } => ErrorSeverity::Critical,
            RiskError::ModelValidationFailed { .. } => ErrorSeverity::High,
            RiskError::BacktestingFailed { .. } => ErrorSeverity::High,
            RiskError::DatabaseError { .. } => ErrorSeverity::High,
            RiskError::AuthenticationError { .. } => ErrorSeverity::High,
            RiskError::BusinessRuleViolation { .. } => ErrorSeverity::High,
            RiskError::RegulatoryError { .. } => ErrorSeverity::High,
            RiskError::DataQualityError { .. } => ErrorSeverity::Medium,
            RiskError::CalculationError { .. } => ErrorSeverity::Medium,
            RiskError::MarketDataError { .. } => ErrorSeverity::Medium,
            RiskError::ValidationError { .. } => ErrorSeverity::Medium,
            RiskError::NetworkError { .. } => ErrorSeverity::Low,
            RiskError::SerializationError { .. } => ErrorSeverity::Low,
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

    /// Check if error affects risk calculations
    pub fn affects_risk_calculations(&self) -> bool {
        matches!(
            self,
            RiskError::ModelError { .. }
                | RiskError::DataQualityError { .. }
                | RiskError::CalculationError { .. }
                | RiskError::MarketDataError { .. }
                | RiskError::ModelValidationFailed { .. }
                | RiskError::BacktestingFailed { .. }
        )
    }

    /// Check if error is related to compliance
    pub fn is_compliance_related(&self) -> bool {
        matches!(
            self,
            RiskError::RegulatoryError { .. }
                | RiskError::RiskLimitBreach { .. }
                | RiskError::ThresholdExceeded { .. }
                | RiskError::BusinessRuleViolation { .. }
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
impl From<serde_json::Error> for RiskError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for RiskError {
    fn from(err: reqwest::Error) -> Self {
        Self::NetworkError {
            message: err.to_string(),
        }
    }
}

#[cfg(feature = "database")]
impl From<sqlx::Error> for RiskError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<validator::ValidationErrors> for RiskError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::ValidationError {
            field: "multiple".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<core_compliance::error::ComplianceError> for RiskError {
    fn from(err: core_compliance::error::ComplianceError) -> Self {
        Self::RegulatoryError {
            regulation: "compliance".to_string(),
            message: err.to_string(),
        }
    }
}

impl From<core_asset_lifecycle::error::AssetError> for RiskError {
    fn from(err: core_asset_lifecycle::error::AssetError) -> Self {
        Self::AssetRiskError {
            asset_id: "unknown".to_string(),
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let assessment_error = RiskError::assessment_error("Invalid risk parameters");
        assert!(matches!(assessment_error, RiskError::AssessmentError { .. }));
        assert_eq!(assessment_error.category(), "assessment");
    }

    #[test]
    fn test_error_retryability() {
        let network_error = RiskError::NetworkError {
            message: "Connection timeout".to_string(),
        };
        assert!(network_error.is_retryable());

        let validation_error = RiskError::ValidationError {
            field: "risk_level".to_string(),
            message: "Invalid risk level".to_string(),
        };
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let limit_breach = RiskError::RiskLimitBreach {
            limit_type: "VaR".to_string(),
            current_value: "5%".to_string(),
            limit_value: "3%".to_string(),
        };
        assert_eq!(limit_breach.severity(), ErrorSeverity::Critical);
        assert!(limit_breach.requires_immediate_attention());

        let network_error = RiskError::NetworkError {
            message: "Timeout".to_string(),
        };
        assert_eq!(network_error.severity(), ErrorSeverity::Low);
        assert!(!network_error.requires_immediate_attention());
    }

    #[test]
    fn test_error_categories() {
        let errors = vec![
            RiskError::assessment_error("test"),
            RiskError::insurance_error("test"),
            RiskError::hedging_error("test"),
            RiskError::monitoring_error("test"),
        ];

        let categories: Vec<&str> = errors.iter().map(|e| e.category()).collect();
        assert_eq!(categories, vec!["assessment", "insurance", "hedging", "monitoring"]);
    }

    #[test]
    fn test_risk_calculation_errors() {
        let model_error = RiskError::ModelError {
            model_type: "VaR".to_string(),
            message: "Model failed".to_string(),
        };
        assert!(model_error.affects_risk_calculations());

        let network_error = RiskError::NetworkError {
            message: "Connection failed".to_string(),
        };
        assert!(!network_error.affects_risk_calculations());
    }

    #[test]
    fn test_compliance_related_errors() {
        let regulatory_error = RiskError::RegulatoryError {
            regulation: "Basel III".to_string(),
            message: "Capital requirement not met".to_string(),
        };
        assert!(regulatory_error.is_compliance_related());

        let calculation_error = RiskError::CalculationError {
            calculation_type: "VaR".to_string(),
            message: "Calculation failed".to_string(),
        };
        assert!(!calculation_error.is_compliance_related());
    }

    #[test]
    fn test_specific_error_constructors() {
        let limit_breach = RiskError::risk_limit_breach("VaR", "5%", "3%");
        assert!(matches!(limit_breach, RiskError::RiskLimitBreach { .. }));

        let model_error = RiskError::model_error("Monte Carlo", "Convergence failed");
        assert!(matches!(model_error, RiskError::ModelError { .. }));

        let threshold_error = RiskError::threshold_exceeded("Concentration", "Limit exceeded");
        assert!(matches!(threshold_error, RiskError::ThresholdExceeded { .. }));
    }
}
