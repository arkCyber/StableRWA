// =====================================================================================
// File: core-ai-risk/src/types.rs
// Description: Core types for AI risk assessment
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Risk score with confidence interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub score: f64,
    pub confidence: f64,
    pub risk_level: RiskLevel,
    pub factors: Vec<RiskFactor>,
    pub timestamp: DateTime<Utc>,
}

/// Risk level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk factor contributing to the score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub weight: f64,
    pub value: f64,
    pub description: String,
}

/// Fraud detection score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudScore {
    pub transaction_id: String,
    pub user_id: String,
    pub fraud_probability: f64,
    pub risk_score: RiskScore,
    pub anomaly_indicators: Vec<AnomalyIndicator>,
    pub behavioral_patterns: Vec<BehaviorPattern>,
}

/// Credit scoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditScore {
    pub user_id: String,
    pub credit_score: u16, // 300-850 range
    pub probability_of_default: f64,
    pub credit_grade: CreditGrade,
    pub risk_score: RiskScore,
    pub credit_factors: Vec<CreditFactor>,
}

/// Market risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRiskScore {
    pub asset_id: String,
    pub portfolio_id: Option<String>,
    pub var_1d: Decimal,  // Value at Risk 1 day
    pub var_10d: Decimal, // Value at Risk 10 days
    pub expected_shortfall: Decimal,
    pub volatility: f64,
    pub beta: f64,
    pub risk_score: RiskScore,
}

/// Operational risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalRiskScore {
    pub process_id: String,
    pub system_id: String,
    pub operational_risk_score: f64,
    pub failure_probability: f64,
    pub impact_severity: ImpactSeverity,
    pub risk_score: RiskScore,
    pub mitigation_strategies: Vec<MitigationStrategy>,
}

/// Anomaly indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyIndicator {
    pub indicator_type: String,
    pub severity: f64,
    pub description: String,
    pub detected_at: DateTime<Utc>,
}

/// Behavioral pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_type: String,
    pub frequency: f64,
    pub deviation_score: f64,
    pub historical_baseline: f64,
}

/// Credit grade enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CreditGrade {
    A, // Excellent (750-850)
    B, // Good (700-749)
    C, // Fair (650-699)
    D, // Poor (600-649)
    E, // Bad (550-599)
    F, // Very Bad (300-549)
}

/// Credit factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditFactor {
    pub factor_name: String,
    pub impact: f64,
    pub current_value: f64,
    pub optimal_range: (f64, f64),
}

/// Impact severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactSeverity {
    Negligible,
    Minor,
    Moderate,
    Major,
    Catastrophic,
}

/// Mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub strategy_id: String,
    pub name: String,
    pub effectiveness: f64,
    pub cost: Decimal,
    pub implementation_time: u32, // days
}

/// Risk model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskModel {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub model_type: ModelType,
    pub algorithm: String,
    pub features: Vec<String>,
    pub performance_metrics: ModelMetrics,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Model type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    FraudDetection,
    CreditScoring,
    MarketRisk,
    OperationalRisk,
    AnomalyDetection,
    BehaviorAnalysis,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub auc_roc: f64,
    pub auc_pr: f64,
    pub confusion_matrix: Vec<Vec<u32>>,
    pub feature_importance: HashMap<String, f64>,
}

/// Risk features for model input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFeatures {
    pub user_id: String,
    pub transaction_id: Option<String>,
    pub features: HashMap<String, f64>,
    pub categorical_features: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Risk prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPrediction {
    pub prediction_id: Uuid,
    pub model_id: Uuid,
    pub input_features: RiskFeatures,
    pub prediction: f64,
    pub probability_distribution: Vec<f64>,
    pub confidence_interval: (f64, f64),
    pub explanation: ModelExplanation,
    pub created_at: DateTime<Utc>,
}

/// Model explanation for interpretability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelExplanation {
    pub feature_contributions: HashMap<String, f64>,
    pub top_factors: Vec<String>,
    pub decision_path: Vec<DecisionNode>,
    pub counterfactual_examples: Vec<CounterfactualExample>,
}

/// Decision tree node for explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionNode {
    pub feature: String,
    pub threshold: f64,
    pub direction: String, // "left" or "right"
    pub contribution: f64,
}

/// Counterfactual example for explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualExample {
    pub feature_changes: HashMap<String, f64>,
    pub predicted_outcome: f64,
    pub distance: f64,
}

impl RiskLevel {
    /// Convert risk score to risk level
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.3 => RiskLevel::Low,
            s if s < 0.6 => RiskLevel::Medium,
            s if s < 0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    /// Get numeric value for risk level
    pub fn to_numeric(&self) -> u8 {
        match self {
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }
}

impl CreditGrade {
    /// Convert credit score to grade
    pub fn from_score(score: u16) -> Self {
        match score {
            750..=850 => CreditGrade::A,
            700..=749 => CreditGrade::B,
            650..=699 => CreditGrade::C,
            600..=649 => CreditGrade::D,
            550..=599 => CreditGrade::E,
            _ => CreditGrade::F,
        }
    }

    /// Get numeric value for credit grade
    pub fn to_numeric(&self) -> u8 {
        match self {
            CreditGrade::A => 6,
            CreditGrade::B => 5,
            CreditGrade::C => 4,
            CreditGrade::D => 3,
            CreditGrade::E => 2,
            CreditGrade::F => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level_from_score() {
        assert_eq!(RiskLevel::from_score(0.1), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.5), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.7), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(0.9), RiskLevel::Critical);
    }

    #[test]
    fn test_credit_grade_from_score() {
        assert_eq!(CreditGrade::from_score(800), CreditGrade::A);
        assert_eq!(CreditGrade::from_score(720), CreditGrade::B);
        assert_eq!(CreditGrade::from_score(670), CreditGrade::C);
        assert_eq!(CreditGrade::from_score(620), CreditGrade::D);
        assert_eq!(CreditGrade::from_score(570), CreditGrade::E);
        assert_eq!(CreditGrade::from_score(400), CreditGrade::F);
    }
}
