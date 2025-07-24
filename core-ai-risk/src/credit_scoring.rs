// =====================================================================================
// File: core-ai-risk/src/credit_scoring.rs
// Description: Credit scoring and default probability assessment
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    error::{AIRiskError, AIRiskResult},
    types::{CreditFactor, CreditGrade, CreditScore, RiskFactor, RiskLevel, RiskScore},
};

/// Credit scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditConfig {
    pub min_credit_history_months: u32,
    pub max_debt_to_income_ratio: f64,
    pub min_credit_score: u16,
    pub default_probability_threshold: f64,
    pub feature_weights: CreditFeatureWeights,
}

/// Credit feature weights for scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditFeatureWeights {
    pub payment_history: f64,
    pub credit_utilization: f64,
    pub credit_history_length: f64,
    pub credit_mix: f64,
    pub new_credit: f64,
    pub income_stability: f64,
    pub debt_to_income: f64,
}

impl Default for CreditFeatureWeights {
    fn default() -> Self {
        Self {
            payment_history: 0.35,
            credit_utilization: 0.30,
            credit_history_length: 0.15,
            credit_mix: 0.10,
            new_credit: 0.10,
            income_stability: 0.05,
            debt_to_income: 0.05,
        }
    }
}

/// Credit scoring model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditModel {
    pub model_id: uuid::Uuid,
    pub version: String,
    pub algorithm: CreditAlgorithm,
    pub feature_weights: CreditFeatureWeights,
    pub score_ranges: CreditScoreRanges,
}

/// Credit scoring algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreditAlgorithm {
    LogisticRegression,
    RandomForest,
    GradientBoosting,
    NeuralNetwork,
    Ensemble,
}

/// Credit score ranges for different grades
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditScoreRanges {
    pub excellent: (u16, u16), // A grade
    pub good: (u16, u16),      // B grade
    pub fair: (u16, u16),      // C grade
    pub poor: (u16, u16),      // D grade
    pub bad: (u16, u16),       // E grade
    pub very_bad: (u16, u16),  // F grade
}

impl Default for CreditScoreRanges {
    fn default() -> Self {
        Self {
            excellent: (750, 850),
            good: (700, 749),
            fair: (650, 699),
            poor: (600, 649),
            bad: (550, 599),
            very_bad: (300, 549),
        }
    }
}

/// Credit features for scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditFeatures {
    pub payment_history_score: f64,    // 0.0 - 1.0
    pub credit_utilization_ratio: f64, // 0.0 - 1.0+
    pub credit_history_length_months: u32,
    pub number_of_credit_accounts: u32,
    pub recent_credit_inquiries: u32,
    pub monthly_income: f64,
    pub monthly_debt_payments: f64,
    pub employment_length_months: u32,
    pub bankruptcy_history: bool,
    pub delinquency_history: u32, // Number of late payments
}

/// Credit scorer trait
#[async_trait]
pub trait CreditScorer: Send + Sync {
    /// Calculate credit score for a user
    async fn calculate_credit_score(
        &self,
        user_id: &str,
        features: &CreditFeatures,
    ) -> AIRiskResult<CreditScore>;

    /// Calculate probability of default
    async fn calculate_default_probability(&self, features: &CreditFeatures) -> AIRiskResult<f64>;

    /// Assess portfolio risk for multiple users
    async fn assess_portfolio_risk(
        &self,
        user_features: &[(String, CreditFeatures)],
    ) -> AIRiskResult<PortfolioRisk>;

    /// Update credit model with new data
    async fn update_model(&self, training_data: &[(CreditFeatures, bool)]) -> AIRiskResult<()>;
}

/// Portfolio risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRisk {
    pub total_exposure: f64,
    pub expected_loss: f64,
    pub unexpected_loss: f64,
    pub default_correlation: f64,
    pub risk_concentration: HashMap<CreditGrade, f64>,
    pub var_95: f64,
    pub expected_shortfall: f64,
}

/// Default probability calculator
pub struct DefaultProbability {
    config: CreditConfig,
}

impl DefaultProbability {
    pub fn new(config: CreditConfig) -> Self {
        Self { config }
    }

    /// Calculate probability of default using logistic regression approach
    pub fn calculate(&self, features: &CreditFeatures) -> f64 {
        let weights = &self.config.feature_weights;

        // Calculate weighted score
        let mut score = 0.0;

        // Payment history (higher is better)
        score += weights.payment_history * features.payment_history_score;

        // Credit utilization (lower is better)
        score += weights.credit_utilization * (1.0 - features.credit_utilization_ratio.min(1.0));

        // Credit history length (longer is better)
        let history_score = (features.credit_history_length_months as f64 / 120.0).min(1.0);
        score += weights.credit_history_length * history_score;

        // Credit mix (more accounts can be better, but not too many)
        let mix_score = if features.number_of_credit_accounts < 3 {
            features.number_of_credit_accounts as f64 / 3.0
        } else if features.number_of_credit_accounts <= 10 {
            1.0
        } else {
            1.0 - ((features.number_of_credit_accounts - 10) as f64 * 0.1)
        };
        score += weights.credit_mix * mix_score.max(0.0);

        // New credit (fewer recent inquiries is better)
        let new_credit_score = if features.recent_credit_inquiries == 0 {
            1.0
        } else {
            (1.0 - (features.recent_credit_inquiries as f64 * 0.1)).max(0.0)
        };
        score += weights.new_credit * new_credit_score;

        // Income stability
        let income_stability_score = (features.employment_length_months as f64 / 60.0).min(1.0);
        score += weights.income_stability * income_stability_score;

        // Debt to income ratio (lower is better)
        let debt_to_income = features.monthly_debt_payments / features.monthly_income.max(1.0);
        let dti_score = (1.0 - debt_to_income.min(1.0)).max(0.0);
        score += weights.debt_to_income * dti_score;

        // Apply penalties for negative factors
        if features.bankruptcy_history {
            score *= 0.5; // Significant penalty for bankruptcy
        }

        if features.delinquency_history > 0 {
            let delinquency_penalty = 1.0 - (features.delinquency_history as f64 * 0.05);
            score *= delinquency_penalty.max(0.3);
        }

        // Convert score to probability using logistic function
        let logit = (score - 0.5) * 10.0; // Scale and center
        1.0 / (1.0 + (-logit).exp())
    }
}

/// Credit rating calculator
pub struct CreditRating {
    config: CreditConfig,
    default_prob_calculator: DefaultProbability,
}

impl CreditRating {
    pub fn new(config: CreditConfig) -> Self {
        let default_prob_calculator = DefaultProbability::new(config.clone());
        Self {
            config,
            default_prob_calculator,
        }
    }

    /// Convert probability of default to credit score
    pub fn probability_to_score(&self, probability: f64) -> u16 {
        // Inverse relationship: lower probability = higher score
        let normalized_prob = probability.max(0.001).min(0.999);
        let score = 850.0 - (normalized_prob.ln() / (-0.01)).min(550.0);
        score.max(300.0) as u16
    }

    /// Create credit factors explanation
    pub fn create_credit_factors(&self, features: &CreditFeatures) -> Vec<CreditFactor> {
        let mut factors = Vec::new();

        factors.push(CreditFactor {
            factor_name: "Payment History".to_string(),
            impact: self.config.feature_weights.payment_history,
            current_value: features.payment_history_score,
            optimal_range: (0.95, 1.0),
        });

        factors.push(CreditFactor {
            factor_name: "Credit Utilization".to_string(),
            impact: self.config.feature_weights.credit_utilization,
            current_value: features.credit_utilization_ratio,
            optimal_range: (0.0, 0.3),
        });

        factors.push(CreditFactor {
            factor_name: "Credit History Length".to_string(),
            impact: self.config.feature_weights.credit_history_length,
            current_value: features.credit_history_length_months as f64,
            optimal_range: (120.0, f64::INFINITY),
        });

        factors.push(CreditFactor {
            factor_name: "Debt-to-Income Ratio".to_string(),
            impact: self.config.feature_weights.debt_to_income,
            current_value: features.monthly_debt_payments / features.monthly_income.max(1.0),
            optimal_range: (0.0, 0.36),
        });

        factors
    }
}

/// Main credit scorer implementation
pub struct CreditScorerImpl {
    config: CreditConfig,
    model: CreditModel,
    rating_calculator: CreditRating,
}

impl CreditScorerImpl {
    pub fn new(config: CreditConfig, model: CreditModel) -> Self {
        let rating_calculator = CreditRating::new(config.clone());
        Self {
            config,
            model,
            rating_calculator,
        }
    }
}

#[async_trait]
impl CreditScorer for CreditScorerImpl {
    async fn calculate_credit_score(
        &self,
        user_id: &str,
        features: &CreditFeatures,
    ) -> AIRiskResult<CreditScore> {
        // Calculate probability of default
        let probability_of_default = self
            .rating_calculator
            .default_prob_calculator
            .calculate(features);

        // Convert to credit score
        let credit_score_value = self
            .rating_calculator
            .probability_to_score(probability_of_default);

        // Determine credit grade
        let credit_grade = CreditGrade::from_score(credit_score_value);

        // Create risk factors
        let risk_factors = vec![
            RiskFactor {
                name: "Payment History".to_string(),
                weight: self.config.feature_weights.payment_history,
                value: features.payment_history_score,
                description: "Historical payment performance".to_string(),
            },
            RiskFactor {
                name: "Credit Utilization".to_string(),
                weight: self.config.feature_weights.credit_utilization,
                value: features.credit_utilization_ratio,
                description: "Current credit usage ratio".to_string(),
            },
        ];

        let risk_score = RiskScore {
            score: probability_of_default,
            confidence: 0.85,
            risk_level: RiskLevel::from_score(probability_of_default),
            factors: risk_factors,
            timestamp: chrono::Utc::now(),
        };

        let credit_factors = self.rating_calculator.create_credit_factors(features);

        Ok(CreditScore {
            user_id: user_id.to_string(),
            credit_score: credit_score_value,
            probability_of_default,
            credit_grade,
            risk_score,
            credit_factors,
        })
    }

    async fn calculate_default_probability(&self, features: &CreditFeatures) -> AIRiskResult<f64> {
        let probability = self
            .rating_calculator
            .default_prob_calculator
            .calculate(features);
        Ok(probability)
    }

    async fn assess_portfolio_risk(
        &self,
        user_features: &[(String, CreditFeatures)],
    ) -> AIRiskResult<PortfolioRisk> {
        let mut total_exposure = 0.0;
        let mut expected_loss = 0.0;
        let mut grade_distribution = HashMap::new();

        for (_, features) in user_features {
            let prob_default = self
                .rating_calculator
                .default_prob_calculator
                .calculate(features);
            let exposure = features.monthly_income * 12.0; // Annual income as proxy for exposure

            total_exposure += exposure;
            expected_loss += prob_default * exposure * 0.5; // Assume 50% loss given default

            let credit_score = self.rating_calculator.probability_to_score(prob_default);
            let grade = CreditGrade::from_score(credit_score);
            *grade_distribution.entry(grade).or_insert(0.0) += exposure;
        }

        // Normalize grade distribution
        for (_, value) in grade_distribution.iter_mut() {
            *value /= total_exposure;
        }

        // Calculate unexpected loss (simplified)
        let unexpected_loss = expected_loss * 2.0; // Simplified calculation

        Ok(PortfolioRisk {
            total_exposure,
            expected_loss,
            unexpected_loss,
            default_correlation: 0.15, // Assumed correlation
            risk_concentration: grade_distribution,
            var_95: expected_loss + unexpected_loss * 1.65, // 95% VaR
            expected_shortfall: expected_loss + unexpected_loss * 2.0,
        })
    }

    async fn update_model(&self, _training_data: &[(CreditFeatures, bool)]) -> AIRiskResult<()> {
        // Mock implementation - in reality, this would retrain the model
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_probability_calculation() {
        let config = CreditConfig {
            min_credit_history_months: 12,
            max_debt_to_income_ratio: 0.43,
            min_credit_score: 300,
            default_probability_threshold: 0.1,
            feature_weights: CreditFeatureWeights::default(),
        };

        let calculator = DefaultProbability::new(config);

        let good_features = CreditFeatures {
            payment_history_score: 0.95,
            credit_utilization_ratio: 0.2,
            credit_history_length_months: 60,
            number_of_credit_accounts: 5,
            recent_credit_inquiries: 1,
            monthly_income: 5000.0,
            monthly_debt_payments: 1000.0,
            employment_length_months: 36,
            bankruptcy_history: false,
            delinquency_history: 0,
        };

        let probability = calculator.calculate(&good_features);
        assert!(probability < 0.2); // Should be low risk

        let bad_features = CreditFeatures {
            payment_history_score: 0.6,
            credit_utilization_ratio: 0.9,
            credit_history_length_months: 6,
            number_of_credit_accounts: 1,
            recent_credit_inquiries: 5,
            monthly_income: 2000.0,
            monthly_debt_payments: 1800.0,
            employment_length_months: 3,
            bankruptcy_history: true,
            delinquency_history: 5,
        };

        let bad_probability = calculator.calculate(&bad_features);
        assert!(bad_probability > probability); // Should be higher risk
    }

    #[tokio::test]
    async fn test_credit_score_calculation() {
        let config = CreditConfig {
            min_credit_history_months: 12,
            max_debt_to_income_ratio: 0.43,
            min_credit_score: 300,
            default_probability_threshold: 0.1,
            feature_weights: CreditFeatureWeights::default(),
        };

        let model = CreditModel {
            model_id: uuid::Uuid::new_v4(),
            version: "1.0".to_string(),
            algorithm: CreditAlgorithm::LogisticRegression,
            feature_weights: CreditFeatureWeights::default(),
            score_ranges: CreditScoreRanges::default(),
        };

        let scorer = CreditScorerImpl::new(config, model);

        let features = CreditFeatures {
            payment_history_score: 0.9,
            credit_utilization_ratio: 0.3,
            credit_history_length_months: 48,
            number_of_credit_accounts: 4,
            recent_credit_inquiries: 2,
            monthly_income: 4000.0,
            monthly_debt_payments: 800.0,
            employment_length_months: 24,
            bankruptcy_history: false,
            delinquency_history: 1,
        };

        let result = scorer.calculate_credit_score("test_user", &features).await;
        assert!(result.is_ok());

        let credit_score = result.unwrap();
        assert!(credit_score.credit_score >= 300 && credit_score.credit_score <= 850);
        assert!(
            credit_score.probability_of_default >= 0.0
                && credit_score.probability_of_default <= 1.0
        );
        assert!(!credit_score.credit_factors.is_empty());
    }
}
