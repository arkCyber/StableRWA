// =====================================================================================
// File: core-regtech/src/aml.rs
// Description: Anti-Money Laundering (AML) compliance module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{RegTechError, RegTechResult},
    types::{AMLAlert, AMLAlertType, AlertStatus, RiskLevel},
};

/// AML service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMLConfig {
    pub transaction_threshold: Decimal,
    pub velocity_threshold: u32,
    pub pattern_detection_enabled: bool,
    pub real_time_monitoring: bool,
    pub alert_auto_escalation: bool,
    pub suspicious_patterns: Vec<SuspiciousPattern>,
}

/// AML service trait
#[async_trait]
pub trait AMLService: Send + Sync {
    /// Monitor transaction for suspicious activity
    async fn monitor_transaction(
        &self,
        transaction: &TransactionData,
    ) -> RegTechResult<Option<AMLAlert>>;

    /// Analyze transaction patterns
    async fn analyze_patterns(
        &self,
        entity_id: &str,
        transactions: &[TransactionData],
    ) -> RegTechResult<Vec<PatternMatch>>;

    /// Generate suspicious activity report
    async fn generate_sar(&self, alert_id: &Uuid) -> RegTechResult<SuspiciousActivityReport>;

    /// Update transaction risk score
    async fn update_risk_score(&self, transaction_id: &str, risk_score: f64) -> RegTechResult<()>;

    /// Get entity risk profile
    async fn get_risk_profile(&self, entity_id: &str) -> RegTechResult<EntityRiskProfile>;
}

/// Transaction data for AML monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub transaction_id: String,
    pub from_entity: String,
    pub to_entity: String,
    pub amount: Decimal,
    pub currency: String,
    pub transaction_type: TransactionType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

/// Transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Deposit,
    Withdrawal,
    Exchange,
    Payment,
    Refund,
}

/// Suspicious patterns configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub threshold: f64,
    pub time_window_hours: u32,
    pub risk_weight: f64,
}

/// Pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    Structuring,
    RapidMovement,
    RoundAmount,
    UnusualVelocity,
    GeographicAnomaly,
    TimeAnomaly,
    AmountAnomaly,
}

/// Pattern match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub confidence_score: f64,
    pub matched_transactions: Vec<String>,
    pub description: String,
}

/// Suspicious Activity Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousActivityReport {
    pub sar_id: Uuid,
    pub alert_id: Uuid,
    pub entity_id: String,
    pub suspicious_activity: String,
    pub transaction_ids: Vec<String>,
    pub total_amount: Decimal,
    pub currency: String,
    pub time_period: TimePeriod,
    pub narrative: String,
    pub supporting_documents: Vec<String>,
    pub filed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: SARStatus,
}

/// SAR status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SARStatus {
    Draft,
    UnderReview,
    Filed,
    Rejected,
}

/// Time period for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

/// Entity risk profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRiskProfile {
    pub entity_id: String,
    pub risk_level: RiskLevel,
    pub risk_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub transaction_patterns: Vec<TransactionPattern>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: RiskFactorType,
    pub description: String,
    pub weight: f64,
    pub score: f64,
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
}

/// Transaction pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPattern {
    pub pattern_type: PatternType,
    pub frequency: u32,
    pub average_amount: Decimal,
    pub time_distribution: Vec<u32>, // Hours of day
    pub counterparties: Vec<String>,
}

/// Transaction monitoring implementation
pub struct TransactionMonitoring {
    config: AMLConfig,
    risk_profiles: HashMap<String, EntityRiskProfile>,
    pattern_detectors: Vec<Box<dyn PatternDetection>>,
}

/// Pattern detection trait
pub trait PatternDetection: Send + Sync {
    fn detect_pattern(&self, transactions: &[TransactionData]) -> Vec<PatternMatch>;
    fn pattern_type(&self) -> PatternType;
}

/// Structuring pattern detector
pub struct StructuringDetector {
    threshold: Decimal,
    time_window_hours: u32,
}

impl PatternDetection for StructuringDetector {
    fn detect_pattern(&self, transactions: &[TransactionData]) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        // Group transactions by time windows
        let mut time_windows: HashMap<i64, Vec<&TransactionData>> = HashMap::new();

        for tx in transactions {
            let window = tx.timestamp.timestamp() / (self.time_window_hours as i64 * 3600);
            time_windows.entry(window).or_default().push(tx);
        }

        // Check for structuring patterns
        for (_, window_txs) in time_windows {
            if window_txs.len() >= 3 {
                let total_amount: Decimal = window_txs.iter().map(|tx| tx.amount).sum();

                if total_amount > self.threshold {
                    let tx_ids: Vec<String> = window_txs
                        .iter()
                        .map(|tx| tx.transaction_id.clone())
                        .collect();

                    matches.push(PatternMatch {
                        pattern_id: "structuring_001".to_string(),
                        pattern_type: PatternType::Structuring,
                        confidence_score: 0.8,
                        matched_transactions: tx_ids,
                        description: format!(
                            "Multiple transactions totaling {} in {} hours",
                            total_amount, self.time_window_hours
                        ),
                    });
                }
            }
        }

        matches
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::Structuring
    }
}

/// Rapid movement detector
pub struct RapidMovementDetector {
    velocity_threshold: u32,
    time_window_minutes: u32,
}

impl PatternDetection for RapidMovementDetector {
    fn detect_pattern(&self, transactions: &[TransactionData]) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        // Sort transactions by timestamp
        let mut sorted_txs = transactions.to_vec();
        sorted_txs.sort_by_key(|tx| tx.timestamp);

        // Check for rapid movement patterns
        for window in sorted_txs.windows(self.velocity_threshold as usize) {
            let time_diff = window.last().unwrap().timestamp - window.first().unwrap().timestamp;

            if time_diff.num_minutes() <= self.time_window_minutes as i64 {
                let tx_ids: Vec<String> =
                    window.iter().map(|tx| tx.transaction_id.clone()).collect();

                matches.push(PatternMatch {
                    pattern_id: "rapid_movement_001".to_string(),
                    pattern_type: PatternType::RapidMovement,
                    confidence_score: 0.7,
                    matched_transactions: tx_ids,
                    description: format!(
                        "{} transactions in {} minutes",
                        self.velocity_threshold, self.time_window_minutes
                    ),
                });
            }
        }

        matches
    }

    fn pattern_type(&self) -> PatternType {
        PatternType::RapidMovement
    }
}

impl TransactionMonitoring {
    pub fn new(config: AMLConfig) -> Self {
        let mut pattern_detectors: Vec<Box<dyn PatternDetection>> = Vec::new();

        // Add pattern detectors based on configuration
        pattern_detectors.push(Box::new(StructuringDetector {
            threshold: Decimal::new(1000000, 2), // $10,000
            time_window_hours: 24,
        }));

        pattern_detectors.push(Box::new(RapidMovementDetector {
            velocity_threshold: 5,
            time_window_minutes: 60,
        }));

        Self {
            config,
            risk_profiles: HashMap::new(),
            pattern_detectors,
        }
    }

    pub fn calculate_transaction_risk(&self, transaction: &TransactionData) -> f64 {
        let mut risk_score = 0.0;

        // Amount-based risk
        if transaction.amount > self.config.transaction_threshold {
            risk_score += 0.3;
        }

        // Round amount risk
        if transaction.amount.fract() == Decimal::ZERO {
            risk_score += 0.1;
        }

        // Time-based risk (transactions outside business hours)
        let hour = transaction.timestamp.hour();
        if hour < 6 || hour > 22 {
            risk_score += 0.2;
        }

        // Currency risk (non-standard currencies)
        if transaction.currency != "USD" && transaction.currency != "EUR" {
            risk_score += 0.1;
        }

        risk_score.min(1.0)
    }

    pub fn generate_alert(
        &self,
        transaction: &TransactionData,
        risk_score: f64,
        alert_type: AMLAlertType,
    ) -> AMLAlert {
        AMLAlert {
            alert_id: Uuid::new_v4(),
            alert_type,
            entity_id: transaction.from_entity.clone(),
            transaction_ids: vec![transaction.transaction_id.clone()],
            risk_score,
            description: format!("Suspicious activity detected: {:?}", alert_type),
            status: AlertStatus::Open,
            assigned_to: None,
            created_at: Utc::now(),
            resolved_at: None,
        }
    }
}

#[async_trait]
impl AMLService for TransactionMonitoring {
    async fn monitor_transaction(
        &self,
        transaction: &TransactionData,
    ) -> RegTechResult<Option<AMLAlert>> {
        let risk_score = self.calculate_transaction_risk(transaction);

        if risk_score > 0.7 {
            let alert_type = if transaction.amount > self.config.transaction_threshold {
                AMLAlertType::UnusualTransactionAmount
            } else {
                AMLAlertType::UnusualBehavior
            };

            let alert = self.generate_alert(transaction, risk_score, alert_type);
            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    async fn analyze_patterns(
        &self,
        entity_id: &str,
        transactions: &[TransactionData],
    ) -> RegTechResult<Vec<PatternMatch>> {
        let mut all_matches = Vec::new();

        for detector in &self.pattern_detectors {
            let matches = detector.detect_pattern(transactions);
            all_matches.extend(matches);
        }

        Ok(all_matches)
    }

    async fn generate_sar(&self, alert_id: &Uuid) -> RegTechResult<SuspiciousActivityReport> {
        // Mock SAR generation
        let sar = SuspiciousActivityReport {
            sar_id: Uuid::new_v4(),
            alert_id: *alert_id,
            entity_id: "entity_123".to_string(),
            suspicious_activity: "Potential structuring activity".to_string(),
            transaction_ids: vec!["tx_001".to_string(), "tx_002".to_string()],
            total_amount: Decimal::new(1500000, 2), // $15,000
            currency: "USD".to_string(),
            time_period: TimePeriod {
                start: Utc::now() - chrono::Duration::days(7),
                end: Utc::now(),
            },
            narrative: "Multiple transactions just below reporting threshold detected".to_string(),
            supporting_documents: vec!["transaction_log.pdf".to_string()],
            filed_at: None,
            status: SARStatus::Draft,
        };

        Ok(sar)
    }

    async fn update_risk_score(&self, transaction_id: &str, risk_score: f64) -> RegTechResult<()> {
        // Mock implementation - in reality, this would update a database
        Ok(())
    }

    async fn get_risk_profile(&self, entity_id: &str) -> RegTechResult<EntityRiskProfile> {
        // Mock risk profile generation
        let profile = EntityRiskProfile {
            entity_id: entity_id.to_string(),
            risk_level: RiskLevel::Medium,
            risk_score: 0.6,
            risk_factors: vec![RiskFactor {
                factor_type: RiskFactorType::Transactional,
                description: "High transaction volume".to_string(),
                weight: 0.4,
                score: 0.7,
            }],
            transaction_patterns: vec![TransactionPattern {
                pattern_type: PatternType::UnusualVelocity,
                frequency: 10,
                average_amount: Decimal::new(500000, 2), // $5,000
                time_distribution: vec![
                    0, 0, 0, 0, 0, 0, 2, 5, 8, 10, 12, 8, 5, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                counterparties: vec!["entity_456".to_string(), "entity_789".to_string()],
            }],
            last_updated: Utc::now(),
        };

        Ok(profile)
    }
}

impl Default for AMLConfig {
    fn default() -> Self {
        Self {
            transaction_threshold: Decimal::new(1000000, 2), // $10,000
            velocity_threshold: 5,
            pattern_detection_enabled: true,
            real_time_monitoring: true,
            alert_auto_escalation: false,
            suspicious_patterns: vec![
                SuspiciousPattern {
                    pattern_id: "structuring".to_string(),
                    pattern_type: PatternType::Structuring,
                    threshold: 0.8,
                    time_window_hours: 24,
                    risk_weight: 0.9,
                },
                SuspiciousPattern {
                    pattern_id: "rapid_movement".to_string(),
                    pattern_type: PatternType::RapidMovement,
                    threshold: 0.7,
                    time_window_hours: 1,
                    risk_weight: 0.8,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aml_config_default() {
        let config = AMLConfig::default();
        assert_eq!(config.transaction_threshold, Decimal::new(1000000, 2));
        assert!(config.pattern_detection_enabled);
        assert_eq!(config.suspicious_patterns.len(), 2);
    }

    #[tokio::test]
    async fn test_transaction_monitoring() {
        let config = AMLConfig::default();
        let monitor = TransactionMonitoring::new(config);

        let transaction = TransactionData {
            transaction_id: "tx_001".to_string(),
            from_entity: "user_123".to_string(),
            to_entity: "user_456".to_string(),
            amount: Decimal::new(1500000, 2), // $15,000
            currency: "USD".to_string(),
            transaction_type: TransactionType::Transfer,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let result = monitor.monitor_transaction(&transaction).await;
        assert!(result.is_ok());

        let alert = result.unwrap();
        assert!(alert.is_some());

        let alert = alert.unwrap();
        assert_eq!(alert.alert_type, AMLAlertType::UnusualTransactionAmount);
        assert_eq!(alert.entity_id, "user_123");
    }

    #[test]
    fn test_structuring_detector() {
        let detector = StructuringDetector {
            threshold: Decimal::new(1000000, 2), // $10,000
            time_window_hours: 24,
        };

        let transactions = vec![
            TransactionData {
                transaction_id: "tx_001".to_string(),
                from_entity: "user_123".to_string(),
                to_entity: "user_456".to_string(),
                amount: Decimal::new(400000, 2), // $4,000
                currency: "USD".to_string(),
                transaction_type: TransactionType::Transfer,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            },
            TransactionData {
                transaction_id: "tx_002".to_string(),
                from_entity: "user_123".to_string(),
                to_entity: "user_789".to_string(),
                amount: Decimal::new(400000, 2), // $4,000
                currency: "USD".to_string(),
                transaction_type: TransactionType::Transfer,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            },
            TransactionData {
                transaction_id: "tx_003".to_string(),
                from_entity: "user_123".to_string(),
                to_entity: "user_101".to_string(),
                amount: Decimal::new(400000, 2), // $4,000
                currency: "USD".to_string(),
                transaction_type: TransactionType::Transfer,
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            },
        ];

        let matches = detector.detect_pattern(&transactions);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_type, PatternType::Structuring);
        assert_eq!(matches[0].matched_transactions.len(), 3);
    }

    #[tokio::test]
    async fn test_sar_generation() {
        let config = AMLConfig::default();
        let monitor = TransactionMonitoring::new(config);

        let alert_id = Uuid::new_v4();
        let result = monitor.generate_sar(&alert_id).await;

        assert!(result.is_ok());
        let sar = result.unwrap();
        assert_eq!(sar.alert_id, alert_id);
        assert_eq!(sar.status, SARStatus::Draft);
    }
}
