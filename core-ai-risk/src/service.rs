// =====================================================================================
// File: core-ai-risk/src/service.rs
// Description: Main AI risk assessment service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    error::{AIRiskError, AIRiskResult},
    fraud_detection::{AnomalyDetector, FraudConfig, FraudDetector, FraudModel},
    types::{
        CreditScore, FraudScore, MarketRiskScore, ModelMetrics, OperationalRiskScore, RiskFeatures,
        RiskModel, RiskPrediction, RiskScore,
    },
};

/// AI Risk service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRiskServiceConfig {
    pub fraud_config: FraudConfig,
    pub enable_real_time_scoring: bool,
    pub model_update_interval_hours: u32,
    pub performance_monitoring: bool,
    pub explainability_enabled: bool,
}

impl Default for AIRiskServiceConfig {
    fn default() -> Self {
        Self {
            fraud_config: FraudConfig {
                fraud_threshold: 0.7,
                anomaly_threshold: 0.6,
                behavior_window_hours: 24,
                min_transaction_count: 5,
                velocity_limits: crate::fraud_detection::VelocityLimits {
                    max_transactions_per_hour: 10,
                    max_amount_per_hour: 10000.0,
                    max_transactions_per_day: 100,
                    max_amount_per_day: 100000.0,
                },
                geographic_rules: crate::fraud_detection::GeographicRules {
                    high_risk_countries: vec!["XX".to_string()],
                    impossible_travel_threshold_hours: 2.0,
                    geolocation_required: true,
                },
                device_fingerprinting: true,
                real_time_scoring: true,
            },
            enable_real_time_scoring: true,
            model_update_interval_hours: 24,
            performance_monitoring: true,
            explainability_enabled: true,
        }
    }
}

/// Main AI Risk service trait
#[async_trait]
pub trait AIRiskService: Send + Sync {
    /// Assess fraud risk for a transaction
    async fn assess_fraud_risk(&self, features: &RiskFeatures) -> AIRiskResult<FraudScore>;

    /// Calculate credit score for a user
    async fn calculate_credit_score(
        &self,
        user_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<CreditScore>;

    /// Assess market risk for an asset or portfolio
    async fn assess_market_risk(
        &self,
        asset_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<MarketRiskScore>;

    /// Assess operational risk for a process or system
    async fn assess_operational_risk(
        &self,
        process_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<OperationalRiskScore>;

    /// Get model performance metrics
    async fn get_model_metrics(&self, model_id: &str) -> AIRiskResult<ModelMetrics>;

    /// Update model with new training data
    async fn update_model(
        &self,
        model_id: &str,
        training_data: Vec<RiskFeatures>,
        labels: Vec<f64>,
    ) -> AIRiskResult<()>;

    /// Get service health status
    async fn health_check(&self) -> AIRiskResult<AIRiskHealthStatus>;
}

/// AI Risk service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRiskHealthStatus {
    pub status: String,
    pub active_models: u32,
    pub total_predictions_today: u64,
    pub average_response_time_ms: f64,
    pub model_performance: HashMap<String, f64>,
    pub last_model_update: chrono::DateTime<chrono::Utc>,
}

/// Main AI Risk service implementation
pub struct AIRiskServiceImpl {
    config: AIRiskServiceConfig,
    fraud_detector: Arc<dyn FraudDetector>,
    models: Arc<RwLock<HashMap<String, RiskModel>>>,
    metrics: Arc<RwLock<HashMap<String, ModelMetrics>>>,
}

impl AIRiskServiceImpl {
    pub fn new(config: AIRiskServiceConfig) -> Self {
        // Create fraud detector
        let fraud_model = FraudModel {
            model_id: uuid::Uuid::new_v4(),
            version: "1.0".to_string(),
            algorithm: crate::fraud_detection::FraudAlgorithm::RandomForest,
            features: vec![
                "transaction_amount".to_string(),
                "transaction_frequency".to_string(),
                "geographic_distance".to_string(),
            ],
            thresholds: crate::fraud_detection::FraudThresholds {
                fraud_probability: config.fraud_config.fraud_threshold,
                anomaly_score: config.fraud_config.anomaly_threshold,
                risk_score: 0.8,
            },
            performance_metrics: HashMap::new(),
        };

        let fraud_detector = Arc::new(AnomalyDetector::new(
            config.fraud_config.clone(),
            fraud_model,
        ));

        Self {
            config,
            fraud_detector,
            models: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Convert risk features to fraud features
    fn convert_to_fraud_features(
        &self,
        features: &RiskFeatures,
    ) -> crate::fraud_detection::FraudFeatures {
        crate::fraud_detection::FraudFeatures {
            transaction_amount: features
                .features
                .get("transaction_amount")
                .copied()
                .unwrap_or(0.0),
            transaction_frequency: features
                .features
                .get("transaction_frequency")
                .copied()
                .unwrap_or(0.0),
            time_since_last_transaction: features
                .features
                .get("time_since_last_transaction")
                .copied()
                .unwrap_or(0.0),
            geographic_distance: features
                .features
                .get("geographic_distance")
                .copied()
                .unwrap_or(0.0),
            device_fingerprint_match: features
                .categorical_features
                .get("device_fingerprint_match")
                .map(|v| v == "true")
                .unwrap_or(true),
            merchant_category: features
                .categorical_features
                .get("merchant_category")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            payment_method: features
                .categorical_features
                .get("payment_method")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            user_age_days: features
                .features
                .get("user_age_days")
                .copied()
                .unwrap_or(0.0) as u32,
            historical_patterns: HashMap::new(),
        }
    }
}

#[async_trait]
impl AIRiskService for AIRiskServiceImpl {
    async fn assess_fraud_risk(&self, features: &RiskFeatures) -> AIRiskResult<FraudScore> {
        let fraud_features = self.convert_to_fraud_features(features);
        self.fraud_detector.detect_fraud(&fraud_features).await
    }

    async fn calculate_credit_score(
        &self,
        user_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<CreditScore> {
        // Mock credit scoring implementation
        let credit_score_value = features
            .features
            .get("credit_history_length")
            .map(|v| (v * 100.0 + 600.0).min(850.0).max(300.0) as u16)
            .unwrap_or(650);

        let probability_of_default = 1.0 - (credit_score_value as f64 - 300.0) / 550.0;

        let risk_factors = vec![
            crate::types::RiskFactor {
                name: "Credit History Length".to_string(),
                weight: 0.35,
                value: features
                    .features
                    .get("credit_history_length")
                    .copied()
                    .unwrap_or(0.0),
                description: "Length of credit history in years".to_string(),
            },
            crate::types::RiskFactor {
                name: "Payment History".to_string(),
                weight: 0.35,
                value: features
                    .features
                    .get("payment_history_score")
                    .copied()
                    .unwrap_or(0.8),
                description: "Historical payment performance".to_string(),
            },
        ];

        let risk_score = RiskScore {
            score: probability_of_default,
            confidence: 0.85,
            risk_level: crate::types::RiskLevel::from_score(probability_of_default),
            factors: risk_factors,
            timestamp: chrono::Utc::now(),
        };

        Ok(CreditScore {
            user_id: user_id.to_string(),
            credit_score: credit_score_value,
            probability_of_default,
            credit_grade: crate::types::CreditGrade::from_score(credit_score_value),
            risk_score,
            credit_factors: vec![crate::types::CreditFactor {
                factor_name: "Payment History".to_string(),
                impact: 0.35,
                current_value: 0.9,
                optimal_range: (0.95, 1.0),
            }],
        })
    }

    async fn assess_market_risk(
        &self,
        asset_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<MarketRiskScore> {
        // Mock market risk assessment
        let volatility = features.features.get("volatility").copied().unwrap_or(0.2);
        let beta = features.features.get("beta").copied().unwrap_or(1.0);

        let var_1d = rust_decimal::Decimal::from_f64_retain(volatility * 1.65 * 100000.0)
            .unwrap_or_default();
        let var_10d = rust_decimal::Decimal::from_f64_retain(volatility * 1.65 * 316227.0)
            .unwrap_or_default(); // sqrt(10) scaling
        let expected_shortfall =
            var_1d * rust_decimal::Decimal::from_f64_retain(1.3).unwrap_or_default();

        let risk_factors = vec![
            crate::types::RiskFactor {
                name: "Volatility".to_string(),
                weight: 0.4,
                value: volatility,
                description: "Historical price volatility".to_string(),
            },
            crate::types::RiskFactor {
                name: "Beta".to_string(),
                weight: 0.3,
                value: beta,
                description: "Market correlation coefficient".to_string(),
            },
        ];

        let risk_score = RiskScore {
            score: volatility,
            confidence: 0.9,
            risk_level: crate::types::RiskLevel::from_score(volatility),
            factors: risk_factors,
            timestamp: chrono::Utc::now(),
        };

        Ok(MarketRiskScore {
            asset_id: asset_id.to_string(),
            portfolio_id: None,
            var_1d,
            var_10d,
            expected_shortfall,
            volatility,
            beta,
            risk_score,
        })
    }

    async fn assess_operational_risk(
        &self,
        process_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<OperationalRiskScore> {
        // Mock operational risk assessment
        let failure_probability = features
            .features
            .get("failure_rate")
            .copied()
            .unwrap_or(0.01);
        let operational_risk_score = failure_probability * 10.0; // Scale to 0-1 range

        let risk_factors = vec![crate::types::RiskFactor {
            name: "System Reliability".to_string(),
            weight: 0.4,
            value: 1.0 - failure_probability,
            description: "Historical system uptime".to_string(),
        }];

        let risk_score = RiskScore {
            score: operational_risk_score,
            confidence: 0.8,
            risk_level: crate::types::RiskLevel::from_score(operational_risk_score),
            factors: risk_factors,
            timestamp: chrono::Utc::now(),
        };

        Ok(OperationalRiskScore {
            process_id: process_id.to_string(),
            system_id: "system_1".to_string(),
            operational_risk_score,
            failure_probability,
            impact_severity: crate::types::ImpactSeverity::Moderate,
            risk_score,
            mitigation_strategies: vec![crate::types::MitigationStrategy {
                strategy_id: "redundancy".to_string(),
                name: "System Redundancy".to_string(),
                effectiveness: 0.8,
                cost: rust_decimal::Decimal::from(50000),
                implementation_time: 30,
            }],
        })
    }

    async fn get_model_metrics(&self, model_id: &str) -> AIRiskResult<ModelMetrics> {
        let metrics = self.metrics.read().await;
        metrics
            .get(model_id)
            .cloned()
            .ok_or_else(|| AIRiskError::model_not_found(model_id))
    }

    async fn update_model(
        &self,
        model_id: &str,
        _training_data: Vec<RiskFeatures>,
        _labels: Vec<f64>,
    ) -> AIRiskResult<()> {
        // Mock model update - in reality, this would retrain the model
        let mut models = self.models.write().await;
        if models.contains_key(model_id) {
            // Update model timestamp
            if let Some(model) = models.get_mut(model_id) {
                model.updated_at = chrono::Utc::now();
            }
            Ok(())
        } else {
            Err(AIRiskError::model_not_found(model_id))
        }
    }

    async fn health_check(&self) -> AIRiskResult<AIRiskHealthStatus> {
        let models = self.models.read().await;
        let metrics = self.metrics.read().await;

        let mut model_performance = HashMap::new();
        for (model_id, _) in models.iter() {
            if let Some(metric) = metrics.get(model_id) {
                model_performance.insert(model_id.clone(), metric.accuracy);
            }
        }

        Ok(AIRiskHealthStatus {
            status: "healthy".to_string(),
            active_models: models.len() as u32,
            total_predictions_today: 1000, // Mock value
            average_response_time_ms: 50.0,
            model_performance,
            last_model_update: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_fraud_risk_assessment() {
        let config = AIRiskServiceConfig::default();
        let service = AIRiskServiceImpl::new(config);

        let mut features = HashMap::new();
        features.insert("transaction_amount".to_string(), 5000.0);
        features.insert("transaction_frequency".to_string(), 3.0);
        features.insert("geographic_distance".to_string(), 100.0);

        let risk_features = RiskFeatures {
            user_id: "test_user".to_string(),
            transaction_id: Some("tx_123".to_string()),
            features,
            categorical_features: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        let result = service.assess_fraud_risk(&risk_features).await;
        assert!(result.is_ok());

        let fraud_score = result.unwrap();
        assert!(fraud_score.fraud_probability >= 0.0 && fraud_score.fraud_probability <= 1.0);
    }

    #[tokio::test]
    async fn test_credit_score_calculation() {
        let config = AIRiskServiceConfig::default();
        let service = AIRiskServiceImpl::new(config);

        let mut features = HashMap::new();
        features.insert("credit_history_length".to_string(), 5.0);
        features.insert("payment_history_score".to_string(), 0.95);

        let risk_features = RiskFeatures {
            user_id: "test_user".to_string(),
            transaction_id: None,
            features,
            categorical_features: HashMap::new(),
            timestamp: chrono::Utc::now(),
        };

        let result = service
            .calculate_credit_score("test_user", &risk_features)
            .await;
        assert!(result.is_ok());

        let credit_score = result.unwrap();
        assert!(credit_score.credit_score >= 300 && credit_score.credit_score <= 850);
        assert!(
            credit_score.probability_of_default >= 0.0
                && credit_score.probability_of_default <= 1.0
        );
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = AIRiskServiceConfig::default();
        let service = AIRiskServiceImpl::new(config);

        let result = service.health_check().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert_eq!(health.status, "healthy");
    }
}
