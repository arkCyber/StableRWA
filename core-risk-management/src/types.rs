// =====================================================================================
// File: core-risk-management/src/types.rs
// Description: Core types for risk management
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Risk level enumeration (ordered by severity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Very low risk
    VeryLow = 1,
    /// Low risk
    Low = 2,
    /// Medium risk
    Medium = 3,
    /// High risk
    High = 4,
    /// Very high risk
    VeryHigh = 5,
    /// Extreme risk
    Extreme = 6,
}

impl RiskLevel {
    /// Get numeric score for calculations
    pub fn score(&self) -> f64 {
        match self {
            RiskLevel::VeryLow => 1.0,
            RiskLevel::Low => 2.0,
            RiskLevel::Medium => 3.0,
            RiskLevel::High => 4.0,
            RiskLevel::VeryHigh => 5.0,
            RiskLevel::Extreme => 6.0,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::VeryLow => "Very Low Risk",
            RiskLevel::Low => "Low Risk",
            RiskLevel::Medium => "Medium Risk",
            RiskLevel::High => "High Risk",
            RiskLevel::VeryHigh => "Very High Risk",
            RiskLevel::Extreme => "Extreme Risk",
        }
    }

    /// Get color code for UI display
    pub fn color_code(&self) -> &'static str {
        match self {
            RiskLevel::VeryLow => "#00FF00", // Green
            RiskLevel::Low => "#90EE90",     // Light Green
            RiskLevel::Medium => "#FFFF00",  // Yellow
            RiskLevel::High => "#FFA500",    // Orange
            RiskLevel::VeryHigh => "#FF0000", // Red
            RiskLevel::Extreme => "#8B0000", // Dark Red
        }
    }
}

/// Risk type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskType {
    /// Market risk (price movements, volatility)
    Market,
    /// Credit risk (counterparty default)
    Credit,
    /// Liquidity risk (inability to sell assets)
    Liquidity,
    /// Operational risk (system failures, human error)
    Operational,
    /// Regulatory risk (compliance, legal changes)
    Regulatory,
    /// Technology risk (cyber attacks, system failures)
    Technology,
    /// Concentration risk (over-exposure to single asset/sector)
    Concentration,
    /// Model risk (incorrect model assumptions)
    Model,
    /// Reputational risk (brand damage)
    Reputational,
    /// Environmental risk (climate, natural disasters)
    Environmental,
    /// Political risk (government actions, instability)
    Political,
    /// Currency risk (foreign exchange fluctuations)
    Currency,
    /// Interest rate risk
    InterestRate,
    /// Inflation risk
    Inflation,
}

impl RiskType {
    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            RiskType::Market => "Market Risk",
            RiskType::Credit => "Credit Risk",
            RiskType::Liquidity => "Liquidity Risk",
            RiskType::Operational => "Operational Risk",
            RiskType::Regulatory => "Regulatory Risk",
            RiskType::Technology => "Technology Risk",
            RiskType::Concentration => "Concentration Risk",
            RiskType::Model => "Model Risk",
            RiskType::Reputational => "Reputational Risk",
            RiskType::Environmental => "Environmental Risk",
            RiskType::Political => "Political Risk",
            RiskType::Currency => "Currency Risk",
            RiskType::InterestRate => "Interest Rate Risk",
            RiskType::Inflation => "Inflation Risk",
        }
    }

    /// Get typical measurement methods for this risk type
    pub fn measurement_methods(&self) -> Vec<&'static str> {
        match self {
            RiskType::Market => vec!["VaR", "Expected Shortfall", "Beta", "Volatility"],
            RiskType::Credit => vec!["PD", "LGD", "EAD", "Credit Spread"],
            RiskType::Liquidity => vec!["Bid-Ask Spread", "Market Impact", "Time to Liquidate"],
            RiskType::Operational => vec!["Loss Frequency", "Loss Severity", "Key Risk Indicators"],
            RiskType::Regulatory => vec!["Compliance Score", "Regulatory Capital"],
            RiskType::Technology => vec!["System Uptime", "Security Score", "Incident Frequency"],
            _ => vec!["Qualitative Assessment", "Expert Judgment"],
        }
    }
}

/// Risk factor structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub id: Uuid,
    pub name: String,
    pub risk_type: RiskType,
    pub description: String,
    pub impact_level: RiskLevel,
    pub probability: f64, // 0.0 to 1.0
    pub time_horizon: TimeHorizon,
    pub quantitative_impact: Option<Decimal>,
    pub qualitative_impact: String,
    pub mitigation_strategies: Vec<String>,
    pub data_sources: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// Time horizon for risk assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeHorizon {
    Immediate,    // < 1 day
    ShortTerm,    // 1 day - 1 week
    MediumTerm,   // 1 week - 1 month
    LongTerm,     // 1 month - 1 year
    VeryLongTerm, // > 1 year
}

impl TimeHorizon {
    /// Get days for this time horizon
    pub fn days(&self) -> u32 {
        match self {
            TimeHorizon::Immediate => 1,
            TimeHorizon::ShortTerm => 7,
            TimeHorizon::MediumTerm => 30,
            TimeHorizon::LongTerm => 365,
            TimeHorizon::VeryLongTerm => 1825, // 5 years
        }
    }
}

/// Comprehensive risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub portfolio_id: Option<Uuid>,
    pub assessment_type: AssessmentType,
    pub overall_risk_level: RiskLevel,
    pub risk_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub risk_metrics: RiskMetrics,
    pub scenario_analysis: Vec<ScenarioResult>,
    pub recommendations: Vec<RiskRecommendation>,
    pub assessor_id: String,
    pub assessment_date: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub confidence_level: f64,
    pub methodology: String,
    pub assumptions: Vec<String>,
    pub limitations: Vec<String>,
}

/// Assessment type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssessmentType {
    Initial,
    Periodic,
    EventDriven,
    Regulatory,
    PreTransaction,
    PostTransaction,
    StressTest,
}

/// Risk metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub value_at_risk: HashMap<String, Decimal>, // VaR by time horizon
    pub expected_shortfall: HashMap<String, Decimal>, // ES by time horizon
    pub maximum_drawdown: Decimal,
    pub volatility: Decimal,
    pub beta: Option<f64>,
    pub correlation_matrix: Option<Vec<Vec<f64>>>,
    pub sharpe_ratio: Option<f64>,
    pub sortino_ratio: Option<f64>,
    pub information_ratio: Option<f64>,
    pub tracking_error: Option<Decimal>,
}

/// Scenario analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub scenario_type: ScenarioType,
    pub probability: f64,
    pub impact: Decimal,
    pub description: String,
    pub key_assumptions: Vec<String>,
    pub time_horizon: TimeHorizon,
}

/// Scenario type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioType {
    BaseCase,
    BestCase,
    WorstCase,
    StressTest,
    HistoricalSimulation,
    MonteCarloSimulation,
}

/// Risk recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskRecommendation {
    pub recommendation_type: RecommendationType,
    pub priority: Priority,
    pub description: String,
    pub rationale: String,
    pub expected_impact: String,
    pub implementation_cost: Option<Decimal>,
    pub timeline: String,
    pub responsible_party: String,
    pub success_metrics: Vec<String>,
}

/// Recommendation type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationType {
    RiskReduction,
    RiskTransfer,
    RiskAcceptance,
    RiskAvoidance,
    Diversification,
    Hedging,
    Insurance,
    Monitoring,
    ProcessImprovement,
    SystemUpgrade,
}

/// Priority enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Risk profile for assets or portfolios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    pub id: Uuid,
    pub entity_id: Uuid, // Asset or Portfolio ID
    pub entity_type: EntityType,
    pub risk_appetite: RiskAppetite,
    pub risk_capacity: RiskCapacity,
    pub risk_tolerance: RiskTolerance,
    pub risk_constraints: Vec<RiskConstraint>,
    pub risk_objectives: Vec<RiskObjective>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

/// Entity type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Asset,
    Portfolio,
    Fund,
    Client,
    Counterparty,
}

/// Risk appetite structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAppetite {
    pub overall_appetite: RiskLevel,
    pub appetite_by_type: HashMap<RiskType, RiskLevel>,
    pub appetite_statement: String,
    pub quantitative_limits: HashMap<String, Decimal>,
}

/// Risk capacity structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskCapacity {
    pub financial_capacity: Decimal,
    pub operational_capacity: String,
    pub regulatory_capacity: String,
    pub time_capacity: String,
    pub capacity_utilization: f64, // 0.0 to 1.0
}

/// Risk tolerance structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTolerance {
    pub loss_tolerance: Decimal,
    pub volatility_tolerance: Decimal,
    pub drawdown_tolerance: Decimal,
    pub liquidity_tolerance: String,
    pub time_horizon_tolerance: TimeHorizon,
}

/// Risk constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConstraint {
    pub constraint_type: ConstraintType,
    pub description: String,
    pub limit_value: Decimal,
    pub current_value: Option<Decimal>,
    pub breach_threshold: Decimal,
    pub enforcement_level: EnforcementLevel,
}

/// Constraint type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    VarLimit,
    ConcentrationLimit,
    LeverageLimit,
    LiquidityLimit,
    DrawdownLimit,
    VolatilityLimit,
    BetaLimit,
    SectorLimit,
    GeographicLimit,
    CurrencyLimit,
}

/// Enforcement level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Advisory,
    Warning,
    HardLimit,
    Regulatory,
}

/// Risk objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskObjective {
    pub objective_type: ObjectiveType,
    pub description: String,
    pub target_value: Option<Decimal>,
    pub measurement_method: String,
    pub achievement_timeline: String,
    pub success_criteria: Vec<String>,
}

/// Objective type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveType {
    ReturnOptimization,
    RiskMinimization,
    VolatilityControl,
    DrawdownControl,
    LiquidityMaintenance,
    DiversificationImprovement,
    ComplianceMaintenance,
    CostMinimization,
}

/// Insurance policy structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsurancePolicy {
    pub id: Uuid,
    pub policy_number: String,
    pub policy_type: PolicyType,
    pub insurer: String,
    pub insured_entity: String,
    pub coverage_amount: Decimal,
    pub deductible: Decimal,
    pub premium: Decimal,
    pub policy_start: DateTime<Utc>,
    pub policy_end: DateTime<Utc>,
    pub covered_risks: Vec<RiskType>,
    pub exclusions: Vec<String>,
    pub claims_history: Vec<Uuid>, // Claim IDs
    pub status: PolicyStatus,
}

/// Policy type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    PropertyInsurance,
    LiabilityInsurance,
    CyberInsurance,
    DirectorsAndOfficers,
    ProfessionalIndemnity,
    BusinessInterruption,
    KeyPersonInsurance,
    CreditInsurance,
    PoliticalRiskInsurance,
}

/// Policy status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyStatus {
    Active,
    Expired,
    Cancelled,
    Suspended,
    UnderReview,
}

/// Insurance claim structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceClaim {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub claim_number: String,
    pub incident_date: DateTime<Utc>,
    pub reported_date: DateTime<Utc>,
    pub claim_amount: Decimal,
    pub claim_type: ClaimType,
    pub description: String,
    pub status: ClaimStatus,
    pub adjuster: Option<String>,
    pub settlement_amount: Option<Decimal>,
    pub settlement_date: Option<DateTime<Utc>>,
    pub documents: Vec<String>,
}

/// Claim type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimType {
    PropertyDamage,
    Theft,
    CyberAttack,
    BusinessInterruption,
    Liability,
    ProfessionalNegligence,
    KeyPersonLoss,
    CreditDefault,
    PoliticalEvent,
}

/// Claim status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimStatus {
    Reported,
    UnderInvestigation,
    Approved,
    Denied,
    Settled,
    Closed,
}

/// Hedging strategy structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingStrategy {
    pub id: Uuid,
    pub strategy_name: String,
    pub strategy_type: HedgingType,
    pub target_risk: RiskType,
    pub hedge_ratio: Decimal,
    pub instruments: Vec<HedgingInstrument>,
    pub effectiveness: Option<f64>,
    pub cost: Decimal,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: HedgingStatus,
}

/// Hedging type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HedgingType {
    CurrencyHedge,
    InterestRateHedge,
    CommodityHedge,
    EquityHedge,
    CreditHedge,
    VolatilityHedge,
    InflationHedge,
}

/// Hedging instrument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingInstrument {
    pub instrument_type: InstrumentType,
    pub notional_amount: Decimal,
    pub strike_price: Option<Decimal>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub counterparty: String,
    pub cost: Decimal,
}

/// Instrument type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstrumentType {
    Forward,
    Future,
    Option,
    Swap,
    Collar,
    Swaption,
    CreditDefaultSwap,
}

/// Hedging status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HedgingStatus {
    Active,
    Expired,
    Terminated,
    Suspended,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::VeryLow < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::VeryHigh);
        assert!(RiskLevel::VeryHigh < RiskLevel::Extreme);
    }

    #[test]
    fn test_risk_level_scores() {
        assert_eq!(RiskLevel::VeryLow.score(), 1.0);
        assert_eq!(RiskLevel::Medium.score(), 3.0);
        assert_eq!(RiskLevel::Extreme.score(), 6.0);
    }

    #[test]
    fn test_risk_type_display_names() {
        assert_eq!(RiskType::Market.display_name(), "Market Risk");
        assert_eq!(RiskType::Credit.display_name(), "Credit Risk");
        assert_eq!(RiskType::Operational.display_name(), "Operational Risk");
    }

    #[test]
    fn test_time_horizon_days() {
        assert_eq!(TimeHorizon::Immediate.days(), 1);
        assert_eq!(TimeHorizon::ShortTerm.days(), 7);
        assert_eq!(TimeHorizon::MediumTerm.days(), 30);
        assert_eq!(TimeHorizon::LongTerm.days(), 365);
        assert_eq!(TimeHorizon::VeryLongTerm.days(), 1825);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Medium);
        assert!(Priority::Medium < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }

    #[test]
    fn test_risk_factor_creation() {
        let risk_factor = RiskFactor {
            id: Uuid::new_v4(),
            name: "Market Volatility".to_string(),
            risk_type: RiskType::Market,
            description: "High market volatility affecting asset prices".to_string(),
            impact_level: RiskLevel::High,
            probability: 0.3,
            time_horizon: TimeHorizon::ShortTerm,
            quantitative_impact: Some(Decimal::new(10000000, 2)), // $100,000
            qualitative_impact: "Significant price fluctuations expected".to_string(),
            mitigation_strategies: vec!["Diversification".to_string(), "Hedging".to_string()],
            data_sources: vec!["Market data feed".to_string()],
            last_updated: Utc::now(),
        };

        assert_eq!(risk_factor.risk_type, RiskType::Market);
        assert_eq!(risk_factor.impact_level, RiskLevel::High);
        assert_eq!(risk_factor.probability, 0.3);
    }

    #[test]
    fn test_insurance_policy_creation() {
        let policy = InsurancePolicy {
            id: Uuid::new_v4(),
            policy_number: "POL-2024-001".to_string(),
            policy_type: PolicyType::PropertyInsurance,
            insurer: "ABC Insurance Co.".to_string(),
            insured_entity: "StableRWA Platform".to_string(),
            coverage_amount: Decimal::new(100000000, 2), // $1,000,000
            deductible: Decimal::new(1000000, 2), // $10,000
            premium: Decimal::new(500000, 2), // $5,000
            policy_start: Utc::now(),
            policy_end: Utc::now() + chrono::Duration::days(365),
            covered_risks: vec![RiskType::Operational, RiskType::Technology],
            exclusions: vec!["War and terrorism".to_string()],
            claims_history: vec![],
            status: PolicyStatus::Active,
        };

        assert_eq!(policy.policy_type, PolicyType::PropertyInsurance);
        assert_eq!(policy.status, PolicyStatus::Active);
        assert_eq!(policy.covered_risks.len(), 2);
    }
}
