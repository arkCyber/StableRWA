// =====================================================================================
// File: core-risk-management/src/lib.rs
// Description: Risk management and insurance framework for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Risk Management Module
//! 
//! This module provides comprehensive risk management and insurance functionality for the
//! StableRWA platform, including risk assessment engines, insurance integration,
//! hedging strategies, and emergency response systems.

pub mod assessment;
pub mod insurance;
pub mod hedging;
pub mod monitoring;
pub mod emergency;
pub mod models;
pub mod error;
pub mod types;
pub mod service;

// Re-export main types and traits
pub use error::{RiskError, RiskResult};
pub use types::{
    RiskLevel, RiskType, RiskFactor, RiskAssessment, RiskProfile,
    InsurancePolicy, InsuranceClaim, HedgingStrategy, RiskMetrics
};
pub use service::RiskManagementService;
pub use assessment::{RiskAssessmentEngine, AssessmentRequest};
pub use insurance::{InsuranceService, PolicyRequest};
pub use hedging::{HedgingService, HedgingRequest};
pub use monitoring::{RiskMonitoringService, MonitoringAlert};
pub use emergency::{EmergencyResponseService, EmergencyPlan};
pub use models::{RiskModel, ModelType};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;

/// Main risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    /// Risk assessment configuration
    pub assessment_config: assessment::AssessmentConfig,
    /// Insurance service configuration
    pub insurance_config: insurance::InsuranceConfig,
    /// Hedging service configuration
    pub hedging_config: hedging::HedgingConfig,
    /// Risk monitoring configuration
    pub monitoring_config: monitoring::MonitoringConfig,
    /// Emergency response configuration
    pub emergency_config: emergency::EmergencyConfig,
    /// Global risk tolerance levels
    pub risk_tolerance: RiskToleranceConfig,
    /// Enable real-time risk monitoring
    pub real_time_monitoring: bool,
    /// Risk reporting frequency
    pub reporting_frequency: ReportingFrequency,
}

impl Default for RiskManagementConfig {
    fn default() -> Self {
        Self {
            assessment_config: assessment::AssessmentConfig::default(),
            insurance_config: insurance::InsuranceConfig::default(),
            hedging_config: hedging::HedgingConfig::default(),
            monitoring_config: monitoring::MonitoringConfig::default(),
            emergency_config: emergency::EmergencyConfig::default(),
            risk_tolerance: RiskToleranceConfig::default(),
            real_time_monitoring: true,
            reporting_frequency: ReportingFrequency::Daily,
        }
    }
}

/// Risk tolerance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskToleranceConfig {
    /// Maximum acceptable risk level for individual assets
    pub max_asset_risk: RiskLevel,
    /// Maximum acceptable portfolio risk
    pub max_portfolio_risk: RiskLevel,
    /// Risk concentration limits
    pub concentration_limits: ConcentrationLimits,
    /// Value at Risk (VaR) limits
    pub var_limits: VarLimits,
    /// Stress test thresholds
    pub stress_test_thresholds: StressTestThresholds,
}

impl Default for RiskToleranceConfig {
    fn default() -> Self {
        Self {
            max_asset_risk: RiskLevel::High,
            max_portfolio_risk: RiskLevel::Medium,
            concentration_limits: ConcentrationLimits::default(),
            var_limits: VarLimits::default(),
            stress_test_thresholds: StressTestThresholds::default(),
        }
    }
}

/// Concentration limits for risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationLimits {
    /// Maximum percentage of portfolio in single asset
    pub single_asset_limit: Decimal,
    /// Maximum percentage in single asset type
    pub asset_type_limit: Decimal,
    /// Maximum percentage in single geographic region
    pub geographic_limit: Decimal,
    /// Maximum percentage with single counterparty
    pub counterparty_limit: Decimal,
}

impl Default for ConcentrationLimits {
    fn default() -> Self {
        Self {
            single_asset_limit: Decimal::new(10, 0), // 10%
            asset_type_limit: Decimal::new(30, 0),   // 30%
            geographic_limit: Decimal::new(40, 0),   // 40%
            counterparty_limit: Decimal::new(20, 0), // 20%
        }
    }
}

/// Value at Risk (VaR) limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarLimits {
    /// Daily VaR limit as percentage of portfolio value
    pub daily_var_limit: Decimal,
    /// Weekly VaR limit
    pub weekly_var_limit: Decimal,
    /// Monthly VaR limit
    pub monthly_var_limit: Decimal,
    /// Confidence level for VaR calculations
    pub confidence_level: f64,
}

impl Default for VarLimits {
    fn default() -> Self {
        Self {
            daily_var_limit: Decimal::new(200, 2),   // 2%
            weekly_var_limit: Decimal::new(500, 2),  // 5%
            monthly_var_limit: Decimal::new(1000, 2), // 10%
            confidence_level: 0.95, // 95% confidence
        }
    }
}

/// Stress test thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestThresholds {
    /// Maximum acceptable loss in severe stress scenario
    pub severe_stress_threshold: Decimal,
    /// Maximum acceptable loss in extreme stress scenario
    pub extreme_stress_threshold: Decimal,
    /// Minimum liquidity buffer required
    pub liquidity_buffer_threshold: Decimal,
}

impl Default for StressTestThresholds {
    fn default() -> Self {
        Self {
            severe_stress_threshold: Decimal::new(1500, 2), // 15%
            extreme_stress_threshold: Decimal::new(2500, 2), // 25%
            liquidity_buffer_threshold: Decimal::new(1000, 2), // 10%
        }
    }
}

/// Reporting frequency enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportingFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
}

/// Risk management event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEvent {
    pub id: Uuid,
    pub event_type: RiskEventType,
    pub severity: RiskSeverity,
    pub asset_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    pub description: String,
    pub risk_factors: Vec<String>,
    pub impact_assessment: ImpactAssessment,
    pub mitigation_actions: Vec<MitigationAction>,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
    pub resolution_timestamp: Option<DateTime<Utc>>,
}

/// Risk event type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskEventType {
    /// Market risk event
    MarketRisk,
    /// Credit risk event
    CreditRisk,
    /// Operational risk event
    OperationalRisk,
    /// Liquidity risk event
    LiquidityRisk,
    /// Regulatory risk event
    RegulatoryRisk,
    /// Technology risk event
    TechnologyRisk,
    /// Concentration risk event
    ConcentrationRisk,
    /// Model risk event
    ModelRisk,
    /// Reputational risk event
    ReputationalRisk,
}

/// Risk severity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
    Catastrophic = 5,
}

/// Impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub financial_impact: Decimal,
    pub operational_impact: String,
    pub reputational_impact: String,
    pub regulatory_impact: String,
    pub probability: f64,
    pub time_horizon: String,
}

/// Mitigation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationAction {
    pub action_type: MitigationActionType,
    pub description: String,
    pub responsible_party: String,
    pub target_completion: DateTime<Utc>,
    pub status: ActionStatus,
    pub effectiveness_score: Option<f64>,
}

/// Mitigation action type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MitigationActionType {
    /// Reduce risk exposure
    RiskReduction,
    /// Transfer risk to third party
    RiskTransfer,
    /// Accept the risk
    RiskAcceptance,
    /// Avoid the risk entirely
    RiskAvoidance,
    /// Hedge the risk
    RiskHedging,
    /// Diversify to reduce risk
    RiskDiversification,
    /// Monitor risk closely
    RiskMonitoring,
}

/// Action status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
    Overdue,
}

/// Portfolio risk summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskSummary {
    pub portfolio_id: Uuid,
    pub overall_risk_level: RiskLevel,
    pub total_value_at_risk: Decimal,
    pub risk_by_type: std::collections::HashMap<RiskType, Decimal>,
    pub concentration_metrics: ConcentrationMetrics,
    pub stress_test_results: StressTestResults,
    pub risk_adjusted_return: Decimal,
    pub sharpe_ratio: f64,
    pub maximum_drawdown: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Concentration metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationMetrics {
    pub herfindahl_index: f64,
    pub top_5_concentration: Decimal,
    pub top_10_concentration: Decimal,
    pub geographic_concentration: std::collections::HashMap<String, Decimal>,
    pub sector_concentration: std::collections::HashMap<String, Decimal>,
}

/// Stress test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResults {
    pub base_case_value: Decimal,
    pub mild_stress_value: Decimal,
    pub severe_stress_value: Decimal,
    pub extreme_stress_value: Decimal,
    pub worst_case_scenario: String,
    pub recovery_time_estimate: u32, // days
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_management_config_default() {
        let config = RiskManagementConfig::default();
        assert!(config.real_time_monitoring);
        assert_eq!(config.reporting_frequency, ReportingFrequency::Daily);
        assert_eq!(config.risk_tolerance.max_asset_risk, RiskLevel::High);
    }

    #[test]
    fn test_risk_tolerance_config_default() {
        let tolerance = RiskToleranceConfig::default();
        assert_eq!(tolerance.max_asset_risk, RiskLevel::High);
        assert_eq!(tolerance.max_portfolio_risk, RiskLevel::Medium);
        assert_eq!(tolerance.concentration_limits.single_asset_limit, Decimal::new(10, 0));
    }

    #[test]
    fn test_concentration_limits_default() {
        let limits = ConcentrationLimits::default();
        assert_eq!(limits.single_asset_limit, Decimal::new(10, 0));
        assert_eq!(limits.asset_type_limit, Decimal::new(30, 0));
        assert_eq!(limits.geographic_limit, Decimal::new(40, 0));
        assert_eq!(limits.counterparty_limit, Decimal::new(20, 0));
    }

    #[test]
    fn test_var_limits_default() {
        let var_limits = VarLimits::default();
        assert_eq!(var_limits.daily_var_limit, Decimal::new(200, 2));
        assert_eq!(var_limits.weekly_var_limit, Decimal::new(500, 2));
        assert_eq!(var_limits.monthly_var_limit, Decimal::new(1000, 2));
        assert_eq!(var_limits.confidence_level, 0.95);
    }

    #[test]
    fn test_risk_severity_ordering() {
        assert!(RiskSeverity::Low < RiskSeverity::Medium);
        assert!(RiskSeverity::Medium < RiskSeverity::High);
        assert!(RiskSeverity::High < RiskSeverity::Critical);
        assert!(RiskSeverity::Critical < RiskSeverity::Catastrophic);
    }

    #[test]
    fn test_risk_event_creation() {
        let event = RiskEvent {
            id: Uuid::new_v4(),
            event_type: RiskEventType::MarketRisk,
            severity: RiskSeverity::Medium,
            asset_id: Some(Uuid::new_v4()),
            portfolio_id: None,
            description: "Market volatility increased".to_string(),
            risk_factors: vec!["High volatility".to_string()],
            impact_assessment: ImpactAssessment {
                financial_impact: Decimal::new(10000000, 2), // $100,000
                operational_impact: "Minimal".to_string(),
                reputational_impact: "Low".to_string(),
                regulatory_impact: "None".to_string(),
                probability: 0.3,
                time_horizon: "1 month".to_string(),
            },
            mitigation_actions: vec![],
            timestamp: Utc::now(),
            resolved: false,
            resolution_timestamp: None,
        };

        assert_eq!(event.event_type, RiskEventType::MarketRisk);
        assert_eq!(event.severity, RiskSeverity::Medium);
        assert!(!event.resolved);
    }

    #[test]
    fn test_mitigation_action_creation() {
        let action = MitigationAction {
            action_type: MitigationActionType::RiskHedging,
            description: "Implement currency hedging strategy".to_string(),
            responsible_party: "Risk Manager".to_string(),
            target_completion: Utc::now() + chrono::Duration::days(30),
            status: ActionStatus::Planned,
            effectiveness_score: None,
        };

        assert_eq!(action.action_type, MitigationActionType::RiskHedging);
        assert_eq!(action.status, ActionStatus::Planned);
        assert!(action.effectiveness_score.is_none());
    }

    #[test]
    fn test_portfolio_risk_summary_creation() {
        let summary = PortfolioRiskSummary {
            portfolio_id: Uuid::new_v4(),
            overall_risk_level: RiskLevel::Medium,
            total_value_at_risk: Decimal::new(50000000, 2), // $500,000
            risk_by_type: std::collections::HashMap::new(),
            concentration_metrics: ConcentrationMetrics {
                herfindahl_index: 0.15,
                top_5_concentration: Decimal::new(4500, 2), // 45%
                top_10_concentration: Decimal::new(7000, 2), // 70%
                geographic_concentration: std::collections::HashMap::new(),
                sector_concentration: std::collections::HashMap::new(),
            },
            stress_test_results: StressTestResults {
                base_case_value: Decimal::new(1000000000, 2), // $10,000,000
                mild_stress_value: Decimal::new(950000000, 2), // $9,500,000
                severe_stress_value: Decimal::new(850000000, 2), // $8,500,000
                extreme_stress_value: Decimal::new(750000000, 2), // $7,500,000
                worst_case_scenario: "Market crash scenario".to_string(),
                recovery_time_estimate: 180, // 6 months
            },
            risk_adjusted_return: Decimal::new(1200, 2), // 12%
            sharpe_ratio: 1.5,
            maximum_drawdown: Decimal::new(1500, 2), // 15%
            last_updated: Utc::now(),
        };

        assert_eq!(summary.overall_risk_level, RiskLevel::Medium);
        assert_eq!(summary.total_value_at_risk, Decimal::new(50000000, 2));
        assert_eq!(summary.sharpe_ratio, 1.5);
    }
}
