// =====================================================================================
// File: core-ai-risk/src/fraud_detection.rs
// Description: Fraud detection and anomaly detection services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{AIRiskError, AIRiskResult},
    types::{
        AnomalyIndicator, BehaviorPattern, FraudScore, ModelExplanation, RiskFactor, RiskFeatures,
        RiskLevel, RiskPrediction, RiskScore,
    },
};

/// Fraud detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudConfig {
    pub fraud_threshold: f64,
    pub anomaly_threshold: f64,
    pub behavior_window_hours: u32,
    pub min_transaction_count: u32,
    pub velocity_limits: VelocityLimits,
    pub geographic_rules: GeographicRules,
    pub device_fingerprinting: bool,
    pub real_time_scoring: bool,
}

/// Transaction velocity limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityLimits {
    pub max_transactions_per_hour: u32,
    pub max_amount_per_hour: f64,
    pub max_transactions_per_day: u32,
    pub max_amount_per_day: f64,
}

/// Geographic fraud rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicRules {
    pub high_risk_countries: Vec<String>,
    pub impossible_travel_threshold_hours: f64,
    pub geolocation_required: bool,
}

/// Fraud detection model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudModel {
    pub model_id: Uuid,
    pub version: String,
    pub algorithm: FraudAlgorithm,
    pub features: Vec<String>,
    pub thresholds: FraudThresholds,
    pub performance_metrics: HashMap<String, f64>,
}

/// Fraud detection algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FraudAlgorithm {
    RandomForest,
    GradientBoosting,
    NeuralNetwork,
    IsolationForest,
    OneClassSVM,
    Ensemble,
}

/// Fraud detection thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudThresholds {
    pub fraud_probability: f64,
    pub anomaly_score: f64,
    pub risk_score: f64,
}

/// Fraud detection features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudFeatures {
    pub transaction_amount: f64,
    pub transaction_frequency: f64,
    pub time_since_last_transaction: f64,
    pub geographic_distance: f64,
    pub device_fingerprint_match: bool,
    pub merchant_category: String,
    pub payment_method: String,
    pub user_age_days: u32,
    pub historical_patterns: HashMap<String, f64>,
}

/// Fraud detector trait
#[async_trait]
pub trait FraudDetector: Send + Sync {
    /// Detect fraud in a transaction
    async fn detect_fraud(&self, features: &FraudFeatures) -> AIRiskResult<FraudScore>;

    /// Detect anomalies in user behavior
    async fn detect_anomalies(
        &self,
        user_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<Vec<AnomalyIndicator>>;

    /// Analyze behavioral patterns
    async fn analyze_behavior(
        &self,
        user_id: &str,
        window_hours: u32,
    ) -> AIRiskResult<Vec<BehaviorPattern>>;

    /// Update fraud model with new data
    async fn update_model(
        &self,
        training_data: &[FraudFeatures],
        labels: &[bool],
    ) -> AIRiskResult<()>;

    /// Get model performance metrics
    async fn get_model_metrics(&self) -> AIRiskResult<HashMap<String, f64>>;
}

/// Anomaly detector implementation
pub struct AnomalyDetector {
    config: FraudConfig,
    model: FraudModel,
}

impl AnomalyDetector {
    pub fn new(config: FraudConfig, model: FraudModel) -> Self {
        Self { config, model }
    }

    /// Calculate anomaly score using isolation forest approach
    pub fn calculate_anomaly_score(&self, features: &FraudFeatures) -> f64 {
        // Simplified anomaly detection algorithm
        let mut score = 0.0;

        // Transaction amount anomaly
        if features.transaction_amount > 10000.0 {
            score += 0.3;
        }

        // Frequency anomaly
        if features.transaction_frequency > 10.0 {
            score += 0.2;
        }

        // Geographic anomaly
        if features.geographic_distance > 1000.0 {
            score += 0.25;
        }

        // Device fingerprint mismatch
        if !features.device_fingerprint_match {
            score += 0.15;
        }

        // Time-based anomaly
        if features.time_since_last_transaction < 60.0 {
            // Less than 1 minute
            score += 0.1;
        }

        score.min(1.0)
    }

    /// Extract behavioral patterns from historical data
    pub fn extract_behavior_patterns(&self, user_id: &str) -> Vec<BehaviorPattern> {
        // Mock implementation - in reality, this would analyze historical data
        vec![
            BehaviorPattern {
                pattern_type: "transaction_timing".to_string(),
                frequency: 0.8,
                deviation_score: 0.2,
                historical_baseline: 0.6,
            },
            BehaviorPattern {
                pattern_type: "amount_distribution".to_string(),
                frequency: 0.9,
                deviation_score: 0.1,
                historical_baseline: 0.8,
            },
        ]
    }
}

#[async_trait]
impl FraudDetector for AnomalyDetector {
    async fn detect_fraud(&self, features: &FraudFeatures) -> AIRiskResult<FraudScore> {
        // Calculate fraud probability using ensemble approach
        let anomaly_score = self.calculate_anomaly_score(features);
        let fraud_probability = anomaly_score * 0.8 + 0.1; // Adjust for base rate

        // Create risk factors
        let risk_factors = vec![
            RiskFactor {
                name: "Transaction Amount".to_string(),
                weight: 0.3,
                value: features.transaction_amount / 10000.0,
                description: "Unusually high transaction amount".to_string(),
            },
            RiskFactor {
                name: "Geographic Distance".to_string(),
                weight: 0.25,
                value: features.geographic_distance / 1000.0,
                description: "Transaction from unusual location".to_string(),
            },
            RiskFactor {
                name: "Transaction Frequency".to_string(),
                weight: 0.2,
                value: features.transaction_frequency / 10.0,
                description: "High transaction frequency".to_string(),
            },
        ];

        let risk_score = RiskScore {
            score: fraud_probability,
            confidence: 0.85,
            risk_level: RiskLevel::from_score(fraud_probability),
            factors: risk_factors,
            timestamp: Utc::now(),
        };

        // Create anomaly indicators
        let mut anomaly_indicators = Vec::new();
        if features.transaction_amount > 10000.0 {
            anomaly_indicators.push(AnomalyIndicator {
                indicator_type: "high_amount".to_string(),
                severity: 0.8,
                description: "Transaction amount exceeds normal range".to_string(),
                detected_at: Utc::now(),
            });
        }

        if features.geographic_distance > 1000.0 {
            anomaly_indicators.push(AnomalyIndicator {
                indicator_type: "geographic_anomaly".to_string(),
                severity: 0.7,
                description: "Transaction from unusual geographic location".to_string(),
                detected_at: Utc::now(),
            });
        }

        let behavioral_patterns = self.extract_behavior_patterns("user_id");

        Ok(FraudScore {
            transaction_id: Uuid::new_v4().to_string(),
            user_id: "user_id".to_string(),
            fraud_probability,
            risk_score,
            anomaly_indicators,
            behavioral_patterns,
        })
    }

    async fn detect_anomalies(
        &self,
        user_id: &str,
        features: &RiskFeatures,
    ) -> AIRiskResult<Vec<AnomalyIndicator>> {
        let mut indicators = Vec::new();

        // Check for velocity anomalies
        if let Some(tx_count) = features.features.get("transaction_count_1h") {
            if *tx_count > self.config.velocity_limits.max_transactions_per_hour as f64 {
                indicators.push(AnomalyIndicator {
                    indicator_type: "velocity_anomaly".to_string(),
                    severity: 0.9,
                    description: "Transaction velocity exceeds limits".to_string(),
                    detected_at: Utc::now(),
                });
            }
        }

        // Check for amount anomalies
        if let Some(amount) = features.features.get("transaction_amount") {
            if *amount > 50000.0 {
                indicators.push(AnomalyIndicator {
                    indicator_type: "amount_anomaly".to_string(),
                    severity: 0.8,
                    description: "Transaction amount is unusually high".to_string(),
                    detected_at: Utc::now(),
                });
            }
        }

        Ok(indicators)
    }

    async fn analyze_behavior(
        &self,
        user_id: &str,
        window_hours: u32,
    ) -> AIRiskResult<Vec<BehaviorPattern>> {
        // Mock implementation - analyze user behavior over time window
        Ok(self.extract_behavior_patterns(user_id))
    }

    async fn update_model(
        &self,
        _training_data: &[FraudFeatures],
        _labels: &[bool],
    ) -> AIRiskResult<()> {
        // Mock implementation - in reality, this would retrain the model
        Ok(())
    }

    async fn get_model_metrics(&self) -> AIRiskResult<HashMap<String, f64>> {
        let mut metrics = HashMap::new();
        metrics.insert("accuracy".to_string(), 0.92);
        metrics.insert("precision".to_string(), 0.89);
        metrics.insert("recall".to_string(), 0.87);
        metrics.insert("f1_score".to_string(), 0.88);
        metrics.insert("auc_roc".to_string(), 0.94);
        Ok(metrics)
    }
}

/// Pattern matcher for known fraud patterns
pub struct PatternMatcher {
    patterns: Vec<FraudPattern>,
}

/// Known fraud pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudPattern {
    pub pattern_id: String,
    pub name: String,
    pub description: String,
    pub conditions: Vec<PatternCondition>,
    pub severity: f64,
}

/// Pattern matching condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCondition {
    pub field: String,
    pub operator: String,
    pub value: f64,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> Vec<FraudPattern> {
        vec![FraudPattern {
            pattern_id: "rapid_succession".to_string(),
            name: "Rapid Succession Transactions".to_string(),
            description: "Multiple transactions in quick succession".to_string(),
            conditions: vec![
                PatternCondition {
                    field: "transaction_frequency".to_string(),
                    operator: ">".to_string(),
                    value: 5.0,
                },
                PatternCondition {
                    field: "time_since_last_transaction".to_string(),
                    operator: "<".to_string(),
                    value: 300.0, // 5 minutes
                },
            ],
            severity: 0.8,
        }]
    }

    pub fn match_patterns(&self, features: &FraudFeatures) -> Vec<String> {
        let mut matched_patterns = Vec::new();

        for pattern in &self.patterns {
            if self.matches_pattern(pattern, features) {
                matched_patterns.push(pattern.pattern_id.clone());
            }
        }

        matched_patterns
    }

    fn matches_pattern(&self, pattern: &FraudPattern, features: &FraudFeatures) -> bool {
        // Simplified pattern matching logic
        pattern
            .conditions
            .iter()
            .all(|condition| match condition.field.as_str() {
                "transaction_frequency" => self.evaluate_condition(
                    features.transaction_frequency,
                    &condition.operator,
                    condition.value,
                ),
                "time_since_last_transaction" => self.evaluate_condition(
                    features.time_since_last_transaction,
                    &condition.operator,
                    condition.value,
                ),
                _ => false,
            })
    }

    fn evaluate_condition(&self, value: f64, operator: &str, threshold: f64) -> bool {
        match operator {
            ">" => value > threshold,
            "<" => value < threshold,
            ">=" => value >= threshold,
            "<=" => value <= threshold,
            "==" => (value - threshold).abs() < f64::EPSILON,
            _ => false,
        }
    }
}

/// Behavior analyzer for user patterns
pub struct BehaviorAnalyzer {
    config: FraudConfig,
}

impl BehaviorAnalyzer {
    pub fn new(config: FraudConfig) -> Self {
        Self { config }
    }

    pub async fn analyze_user_behavior(&self, user_id: &str) -> AIRiskResult<Vec<BehaviorPattern>> {
        // Mock implementation - analyze user's historical behavior
        Ok(vec![
            BehaviorPattern {
                pattern_type: "spending_pattern".to_string(),
                frequency: 0.85,
                deviation_score: 0.15,
                historical_baseline: 0.7,
            },
            BehaviorPattern {
                pattern_type: "login_pattern".to_string(),
                frequency: 0.92,
                deviation_score: 0.08,
                historical_baseline: 0.84,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_score_calculation() {
        let config = FraudConfig {
            fraud_threshold: 0.7,
            anomaly_threshold: 0.6,
            behavior_window_hours: 24,
            min_transaction_count: 5,
            velocity_limits: VelocityLimits {
                max_transactions_per_hour: 10,
                max_amount_per_hour: 10000.0,
                max_transactions_per_day: 100,
                max_amount_per_day: 100000.0,
            },
            geographic_rules: GeographicRules {
                high_risk_countries: vec!["XX".to_string()],
                impossible_travel_threshold_hours: 2.0,
                geolocation_required: true,
            },
            device_fingerprinting: true,
            real_time_scoring: true,
        };

        let model = FraudModel {
            model_id: Uuid::new_v4(),
            version: "1.0".to_string(),
            algorithm: FraudAlgorithm::RandomForest,
            features: vec!["amount".to_string(), "frequency".to_string()],
            thresholds: FraudThresholds {
                fraud_probability: 0.7,
                anomaly_score: 0.6,
                risk_score: 0.8,
            },
            performance_metrics: HashMap::new(),
        };

        let detector = AnomalyDetector::new(config, model);

        let features = FraudFeatures {
            transaction_amount: 15000.0,
            transaction_frequency: 12.0,
            time_since_last_transaction: 30.0,
            geographic_distance: 1500.0,
            device_fingerprint_match: false,
            merchant_category: "online".to_string(),
            payment_method: "credit_card".to_string(),
            user_age_days: 365,
            historical_patterns: HashMap::new(),
        };

        let score = detector.calculate_anomaly_score(&features);
        assert!(score > 0.8); // Should be high risk
    }

    #[tokio::test]
    async fn test_fraud_detection() {
        let config = FraudConfig {
            fraud_threshold: 0.7,
            anomaly_threshold: 0.6,
            behavior_window_hours: 24,
            min_transaction_count: 5,
            velocity_limits: VelocityLimits {
                max_transactions_per_hour: 10,
                max_amount_per_hour: 10000.0,
                max_transactions_per_day: 100,
                max_amount_per_day: 100000.0,
            },
            geographic_rules: GeographicRules {
                high_risk_countries: vec!["XX".to_string()],
                impossible_travel_threshold_hours: 2.0,
                geolocation_required: true,
            },
            device_fingerprinting: true,
            real_time_scoring: true,
        };

        let model = FraudModel {
            model_id: Uuid::new_v4(),
            version: "1.0".to_string(),
            algorithm: FraudAlgorithm::RandomForest,
            features: vec!["amount".to_string(), "frequency".to_string()],
            thresholds: FraudThresholds {
                fraud_probability: 0.7,
                anomaly_score: 0.6,
                risk_score: 0.8,
            },
            performance_metrics: HashMap::new(),
        };

        let detector = AnomalyDetector::new(config, model);

        let features = FraudFeatures {
            transaction_amount: 5000.0,
            transaction_frequency: 3.0,
            time_since_last_transaction: 3600.0,
            geographic_distance: 100.0,
            device_fingerprint_match: true,
            merchant_category: "retail".to_string(),
            payment_method: "debit_card".to_string(),
            user_age_days: 730,
            historical_patterns: HashMap::new(),
        };

        let result = detector.detect_fraud(&features).await;
        assert!(result.is_ok());

        let fraud_score = result.unwrap();
        assert!(fraud_score.fraud_probability >= 0.0 && fraud_score.fraud_probability <= 1.0);
        assert!(!fraud_score.risk_score.factors.is_empty());
    }
}
