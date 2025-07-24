// =====================================================================================
// File: core-bridge/src/security.rs
// Description: Bridge security monitoring and protection service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{BridgeError, BridgeResult},
    types::{ChainId, SecurityAlert, ThreatLevel},
};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Maximum transaction amount without additional verification
    pub max_unverified_amount: Decimal,
    /// Rate limiting: max transactions per hour per user
    pub max_transactions_per_hour: u32,
    /// Rate limiting: max amount per hour per user
    pub max_amount_per_hour: Decimal,
    /// Suspicious activity detection enabled
    pub suspicious_activity_detection: bool,
    /// Automatic transaction blocking enabled
    pub auto_block_suspicious: bool,
    /// Multi-signature requirement threshold
    pub multisig_threshold: u32,
    /// Required confirmations for high-value transactions
    pub high_value_confirmations: u32,
    /// High-value transaction threshold
    pub high_value_threshold: Decimal,
    /// Enable real-time monitoring
    pub real_time_monitoring: bool,
    /// Alert notification enabled
    pub alert_notifications: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_unverified_amount: Decimal::new(1000000, 2), // $10,000
            max_transactions_per_hour: 50,
            max_amount_per_hour: Decimal::new(10000000, 2), // $100,000
            suspicious_activity_detection: true,
            auto_block_suspicious: true,
            multisig_threshold: 3,
            high_value_confirmations: 12,
            high_value_threshold: Decimal::new(10000000, 2), // $100,000
            real_time_monitoring: true,
            alert_notifications: true,
        }
    }
}

/// Security check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheckRequest {
    pub transaction_id: Uuid,
    pub user_id: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub amount: Decimal,
    pub asset_symbol: String,
    pub source_address: String,
    pub destination_address: String,
    pub timestamp: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub session_id: Option<String>,
}

/// Security check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheckResult {
    pub transaction_id: Uuid,
    pub approved: bool,
    pub risk_score: f64,
    pub threat_level: ThreatLevel,
    pub required_confirmations: u32,
    pub requires_manual_review: bool,
    pub blocked_reasons: Vec<String>,
    pub warnings: Vec<String>,
    pub additional_verification_required: bool,
    pub estimated_processing_time: Duration,
    pub checked_at: DateTime<Utc>,
}

/// Suspicious activity pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousPattern {
    pub pattern_id: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub risk_weight: f64,
    pub enabled: bool,
    pub detection_rules: Vec<DetectionRule>,
}

/// Pattern type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    VelocityAnomaly,
    AmountAnomaly,
    FrequencyAnomaly,
    GeographicAnomaly,
    BehavioralAnomaly,
    AddressReuse,
    ChainHopping,
    SandwichAttack,
    FlashLoan,
    Arbitrage,
}

/// Detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    pub rule_id: String,
    pub condition: String,
    pub threshold: Decimal,
    pub time_window: Duration,
    pub action: SecurityAction,
}

/// Security action enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityAction {
    Allow,
    Warn,
    RequireReview,
    Block,
    Quarantine,
}

/// Security monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub total_transactions_checked: u64,
    pub blocked_transactions: u64,
    pub flagged_transactions: u64,
    pub false_positives: u64,
    pub true_positives: u64,
    pub average_risk_score: f64,
    pub high_risk_transactions: u64,
    pub manual_reviews_pending: u64,
    pub last_updated: DateTime<Utc>,
}

/// Security service trait
#[async_trait]
pub trait SecurityService: Send + Sync {
    /// Perform security check on a transaction
    async fn check_transaction(
        &self,
        request: SecurityCheckRequest,
    ) -> BridgeResult<SecurityCheckResult>;

    /// Report suspicious activity
    async fn report_suspicious_activity(
        &self,
        transaction_id: Uuid,
        pattern_type: PatternType,
        details: String,
    ) -> BridgeResult<()>;

    /// Get security alerts
    async fn get_alerts(
        &self,
        severity: Option<ThreatLevel>,
        limit: Option<usize>,
    ) -> BridgeResult<Vec<SecurityAlert>>;

    /// Update security configuration
    async fn update_config(&self, config: SecurityConfig) -> BridgeResult<()>;

    /// Get security metrics
    async fn get_metrics(&self, time_range: Duration) -> BridgeResult<SecurityMetrics>;

    /// Whitelist an address
    async fn whitelist_address(&self, address: String, chain_id: ChainId) -> BridgeResult<()>;

    /// Blacklist an address
    async fn blacklist_address(
        &self,
        address: String,
        chain_id: ChainId,
        reason: String,
    ) -> BridgeResult<()>;

    /// Check if address is whitelisted
    async fn is_whitelisted(&self, address: &str, chain_id: ChainId) -> BridgeResult<bool>;

    /// Check if address is blacklisted
    async fn is_blacklisted(&self, address: &str, chain_id: ChainId) -> BridgeResult<bool>;

    /// Get user risk profile
    async fn get_user_risk_profile(&self, user_id: &str) -> BridgeResult<UserRiskProfile>;

    /// Update user risk profile
    async fn update_user_risk_profile(
        &self,
        user_id: &str,
        profile: UserRiskProfile,
    ) -> BridgeResult<()>;

    /// Health check
    async fn health_check(&self) -> BridgeResult<SecurityHealthStatus>;
}

/// User risk profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRiskProfile {
    pub user_id: String,
    pub risk_score: f64,
    pub trust_level: TrustLevel,
    pub transaction_count: u64,
    pub total_volume: Decimal,
    pub first_transaction: DateTime<Utc>,
    pub last_transaction: DateTime<Utc>,
    pub suspicious_activity_count: u32,
    pub manual_reviews_count: u32,
    pub kyc_verified: bool,
    pub enhanced_verification: bool,
    pub notes: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

/// Trust level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    Unknown,
    Low,
    Medium,
    High,
    Verified,
    Trusted,
}

/// Security health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHealthStatus {
    pub status: String,
    pub monitoring_active: bool,
    pub detection_rules_active: u32,
    pub alerts_pending: u64,
    pub blocked_transactions_24h: u64,
    pub false_positive_rate: f64,
    pub average_response_time_ms: u64,
    pub last_check: DateTime<Utc>,
}

/// Security monitor implementation
pub struct SecurityMonitor {
    config: SecurityConfig,
    patterns: Vec<SuspiciousPattern>,
    user_profiles: HashMap<String, UserRiskProfile>,
    whitelist: HashMap<ChainId, Vec<String>>,
    blacklist: HashMap<ChainId, Vec<String>>,
}

impl SecurityMonitor {
    /// Create a new security monitor
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            patterns: Self::default_patterns(),
            user_profiles: HashMap::new(),
            whitelist: HashMap::new(),
            blacklist: HashMap::new(),
        }
    }

    /// Get default suspicious patterns
    fn default_patterns() -> Vec<SuspiciousPattern> {
        vec![
            SuspiciousPattern {
                pattern_id: "velocity_anomaly".to_string(),
                pattern_type: PatternType::VelocityAnomaly,
                description: "Unusual transaction velocity detected".to_string(),
                risk_weight: 0.7,
                enabled: true,
                detection_rules: vec![DetectionRule {
                    rule_id: "high_frequency".to_string(),
                    condition: "transactions_per_hour > 20".to_string(),
                    threshold: Decimal::new(20, 0),
                    time_window: Duration::hours(1),
                    action: SecurityAction::RequireReview,
                }],
            },
            SuspiciousPattern {
                pattern_id: "amount_anomaly".to_string(),
                pattern_type: PatternType::AmountAnomaly,
                description: "Unusual transaction amount detected".to_string(),
                risk_weight: 0.8,
                enabled: true,
                detection_rules: vec![DetectionRule {
                    rule_id: "large_amount".to_string(),
                    condition: "amount > 1000000".to_string(),
                    threshold: Decimal::new(100000000, 2), // $1,000,000
                    time_window: Duration::hours(24),
                    action: SecurityAction::RequireReview,
                }],
            },
        ]
    }

    /// Calculate risk score for a transaction
    pub fn calculate_risk_score(&self, request: &SecurityCheckRequest) -> f64 {
        let mut risk_score = 0.0;

        // Amount-based risk
        if request.amount > self.config.high_value_threshold {
            risk_score += 0.3;
        }

        // User history risk
        if let Some(profile) = self.user_profiles.get(&request.user_id) {
            match profile.trust_level {
                TrustLevel::Unknown => risk_score += 0.5,
                TrustLevel::Low => risk_score += 0.3,
                TrustLevel::Medium => risk_score += 0.1,
                TrustLevel::High => risk_score -= 0.1,
                TrustLevel::Verified => risk_score -= 0.2,
                TrustLevel::Trusted => risk_score -= 0.3,
            }
        } else {
            risk_score += 0.4; // New user
        }

        // Address blacklist check
        if self
            .blacklist
            .get(&request.source_chain)
            .map_or(false, |list| list.contains(&request.source_address))
        {
            risk_score += 1.0;
        }

        risk_score.max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert_eq!(config.max_unverified_amount, Decimal::new(1000000, 2));
        assert_eq!(config.max_transactions_per_hour, 50);
        assert!(config.suspicious_activity_detection);
        assert!(config.auto_block_suspicious);
    }

    #[test]
    fn test_risk_score_calculation() {
        let config = SecurityConfig::default();
        let monitor = SecurityMonitor::new(config);

        let request = SecurityCheckRequest {
            transaction_id: Uuid::new_v4(),
            user_id: "test_user".to_string(),
            source_chain: ChainId::Ethereum,
            destination_chain: ChainId::Polygon,
            amount: Decimal::new(50000000, 2), // $500,000
            asset_symbol: "USDC".to_string(),
            source_address: "0x1234567890123456789012345678901234567890".to_string(),
            destination_address: "0x0987654321098765432109876543210987654321".to_string(),
            timestamp: Utc::now(),
            user_agent: None,
            ip_address: None,
            session_id: None,
        };

        let risk_score = monitor.calculate_risk_score(&request);
        assert!(risk_score >= 0.0 && risk_score <= 1.0);
    }
}
