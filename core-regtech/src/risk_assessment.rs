// =====================================================================================
// File: core-regtech/src/risk_assessment.rs
// Description: Risk assessment and scoring module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{RegTechError, RegTechResult},
    types::{EntityType, RiskAssessment, RiskLevel, RiskModel, ScoringAlgorithm},
};

/// Risk assessment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub risk_models: Vec<RiskModel>,
    pub scoring_algorithms: Vec<ScoringAlgorithm>,
    pub risk_thresholds: HashMap<String, f64>,
    pub periodic_reassessment_days: u32,
}

/// Risk assessment service trait
#[async_trait]
pub trait RiskAssessmentService: Send + Sync {
    /// Assess entity risk
    async fn assess_risk(
        &self,
        entity_id: &str,
        entity_type: EntityType,
    ) -> RegTechResult<RiskAssessment>;

    /// Calculate risk score
    async fn calculate_risk_score(
        &self,
        entity_id: &str,
        factors: &[RiskFactor],
    ) -> RegTechResult<f64>;

    /// Get risk matrix
    async fn get_risk_matrix(&self) -> RegTechResult<RiskMatrix>;
}

/// Risk score calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub overall_score: f64,
    pub component_scores: HashMap<String, f64>,
    pub confidence_level: f64,
}

/// Risk factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactors {
    pub geographic_risk: f64,
    pub transactional_risk: f64,
    pub behavioral_risk: f64,
    pub industry_risk: f64,
    pub political_exposure_risk: f64,
}

/// Risk matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMatrix {
    pub matrix_id: Uuid,
    pub risk_categories: Vec<RiskCategory>,
    pub scoring_weights: HashMap<String, f64>,
    pub thresholds: HashMap<RiskLevel, f64>,
}

/// Risk category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskCategory {
    pub category_id: String,
    pub category_name: String,
    pub weight: f64,
    pub factors: Vec<RiskFactor>,
}

/// Risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_id: String,
    pub factor_name: String,
    pub factor_type: RiskFactorType,
    pub weight: f64,
    pub score: f64,
    pub description: String,
}

/// Risk factor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskFactorType {
    Geographic,
    Transactional,
    Behavioral,
    Industry,
    PoliticalExposure,
    AdverseMedia,
    SanctionsRisk,
}

/// Risk assessment service implementation
pub struct RiskAssessmentServiceImpl {
    config: RiskConfig,
    risk_matrix: RiskMatrix,
}

impl RiskAssessmentServiceImpl {
    pub fn new(config: RiskConfig) -> Self {
        let risk_matrix = Self::create_default_risk_matrix();

        Self {
            config,
            risk_matrix,
        }
    }

    fn create_default_risk_matrix() -> RiskMatrix {
        let mut thresholds = HashMap::new();
        thresholds.insert(RiskLevel::Low, 0.25);
        thresholds.insert(RiskLevel::Medium, 0.5);
        thresholds.insert(RiskLevel::High, 0.75);
        thresholds.insert(RiskLevel::Critical, 1.0);

        let mut scoring_weights = HashMap::new();
        scoring_weights.insert("geographic".to_string(), 0.2);
        scoring_weights.insert("transactional".to_string(), 0.3);
        scoring_weights.insert("behavioral".to_string(), 0.2);
        scoring_weights.insert("industry".to_string(), 0.15);
        scoring_weights.insert("political_exposure".to_string(), 0.15);

        RiskMatrix {
            matrix_id: Uuid::new_v4(),
            risk_categories: vec![
                RiskCategory {
                    category_id: "geographic".to_string(),
                    category_name: "Geographic Risk".to_string(),
                    weight: 0.2,
                    factors: vec![RiskFactor {
                        factor_id: "country_risk".to_string(),
                        factor_name: "Country Risk".to_string(),
                        factor_type: RiskFactorType::Geographic,
                        weight: 1.0,
                        score: 0.0,
                        description: "Risk associated with country of operation".to_string(),
                    }],
                },
                RiskCategory {
                    category_id: "transactional".to_string(),
                    category_name: "Transactional Risk".to_string(),
                    weight: 0.3,
                    factors: vec![
                        RiskFactor {
                            factor_id: "transaction_volume".to_string(),
                            factor_name: "Transaction Volume".to_string(),
                            factor_type: RiskFactorType::Transactional,
                            weight: 0.5,
                            score: 0.0,
                            description: "Risk based on transaction volume".to_string(),
                        },
                        RiskFactor {
                            factor_id: "transaction_frequency".to_string(),
                            factor_name: "Transaction Frequency".to_string(),
                            factor_type: RiskFactorType::Transactional,
                            weight: 0.5,
                            score: 0.0,
                            description: "Risk based on transaction frequency".to_string(),
                        },
                    ],
                },
            ],
            scoring_weights,
            thresholds,
        }
    }

    fn calculate_weighted_score(&self, factors: &[RiskFactor]) -> f64 {
        let total_weight: f64 = factors.iter().map(|f| f.weight).sum();

        if total_weight == 0.0 {
            return 0.0;
        }

        let weighted_sum: f64 = factors.iter().map(|f| f.score * f.weight).sum();

        weighted_sum / total_weight
    }

    fn determine_risk_level(&self, risk_score: f64) -> RiskLevel {
        for (level, threshold) in &self.risk_matrix.thresholds {
            if risk_score <= *threshold {
                return *level;
            }
        }
        RiskLevel::Critical
    }

    fn generate_risk_factors(
        &self,
        entity_id: &str,
        entity_type: EntityType,
    ) -> Vec<crate::types::RiskFactor> {
        // Mock risk factor generation based on entity type
        let base_score = match entity_type {
            EntityType::Individual => 0.3,
            EntityType::Business => 0.4,
            EntityType::Trust => 0.6,
            EntityType::Foundation => 0.7,
            EntityType::Government => 0.2,
            EntityType::Other => 0.5,
        };

        vec![
            crate::types::RiskFactor {
                factor_type: crate::types::RiskFactorType::Geographic,
                description: "Geographic risk assessment".to_string(),
                weight: 0.2,
                score: base_score * 0.8,
            },
            crate::types::RiskFactor {
                factor_type: crate::types::RiskFactorType::Transactional,
                description: "Transaction pattern analysis".to_string(),
                weight: 0.3,
                score: base_score * 1.2,
            },
            crate::types::RiskFactor {
                factor_type: crate::types::RiskFactorType::Behavioral,
                description: "Behavioral risk indicators".to_string(),
                weight: 0.2,
                score: base_score * 0.9,
            },
        ]
    }
}

#[async_trait]
impl RiskAssessmentService for RiskAssessmentServiceImpl {
    async fn assess_risk(
        &self,
        entity_id: &str,
        entity_type: EntityType,
    ) -> RegTechResult<RiskAssessment> {
        let risk_factors = self.generate_risk_factors(entity_id, entity_type);

        // Calculate overall risk score
        let total_weight: f64 = risk_factors.iter().map(|f| f.weight).sum();
        let weighted_score: f64 = risk_factors.iter().map(|f| f.score * f.weight).sum();

        let risk_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        };

        let risk_level = self.determine_risk_level(risk_score);

        let assessment = RiskAssessment {
            assessment_id: Uuid::new_v4(),
            entity_id: entity_id.to_string(),
            entity_type,
            risk_score,
            risk_level,
            risk_factors,
            mitigation_measures: vec![
                "Enhanced monitoring".to_string(),
                "Regular review".to_string(),
            ],
            assessed_at: chrono::Utc::now(),
            valid_until: chrono::Utc::now()
                + chrono::Duration::days(self.config.periodic_reassessment_days as i64),
        };

        Ok(assessment)
    }

    async fn calculate_risk_score(
        &self,
        entity_id: &str,
        factors: &[RiskFactor],
    ) -> RegTechResult<f64> {
        let score = self.calculate_weighted_score(factors);
        Ok(score)
    }

    async fn get_risk_matrix(&self) -> RegTechResult<RiskMatrix> {
        Ok(self.risk_matrix.clone())
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        let mut risk_thresholds = HashMap::new();
        risk_thresholds.insert("low".to_string(), 0.25);
        risk_thresholds.insert("medium".to_string(), 0.5);
        risk_thresholds.insert("high".to_string(), 0.75);
        risk_thresholds.insert("critical".to_string(), 1.0);

        Self {
            risk_models: vec![RiskModel::RulesBased, RiskModel::MachineLearning],
            scoring_algorithms: vec![ScoringAlgorithm::WeightedSum],
            risk_thresholds,
            periodic_reassessment_days: 90,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_assessment() {
        let config = RiskConfig::default();
        let service = RiskAssessmentServiceImpl::new(config);

        let result = service
            .assess_risk("test_entity", EntityType::Individual)
            .await;
        assert!(result.is_ok());

        let assessment = result.unwrap();
        assert_eq!(assessment.entity_id, "test_entity");
        assert_eq!(assessment.entity_type, EntityType::Individual);
        assert!(!assessment.risk_factors.is_empty());
        assert!(!assessment.mitigation_measures.is_empty());
    }

    #[tokio::test]
    async fn test_risk_score_calculation() {
        let config = RiskConfig::default();
        let service = RiskAssessmentServiceImpl::new(config);

        let factors = vec![
            RiskFactor {
                factor_id: "test1".to_string(),
                factor_name: "Test Factor 1".to_string(),
                factor_type: RiskFactorType::Geographic,
                weight: 0.5,
                score: 0.8,
                description: "Test factor".to_string(),
            },
            RiskFactor {
                factor_id: "test2".to_string(),
                factor_name: "Test Factor 2".to_string(),
                factor_type: RiskFactorType::Transactional,
                weight: 0.5,
                score: 0.6,
                description: "Test factor".to_string(),
            },
        ];

        let result = service.calculate_risk_score("test_entity", &factors).await;
        assert!(result.is_ok());

        let score = result.unwrap();
        assert_eq!(score, 0.7); // (0.8 * 0.5 + 0.6 * 0.5) / (0.5 + 0.5)
    }

    #[tokio::test]
    async fn test_risk_matrix() {
        let config = RiskConfig::default();
        let service = RiskAssessmentServiceImpl::new(config);

        let result = service.get_risk_matrix().await;
        assert!(result.is_ok());

        let matrix = result.unwrap();
        assert!(!matrix.risk_categories.is_empty());
        assert!(!matrix.scoring_weights.is_empty());
        assert!(!matrix.thresholds.is_empty());
    }
}
