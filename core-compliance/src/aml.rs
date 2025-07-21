// =====================================================================================
// File: core-compliance/src/aml.rs
// Description: AML (Anti-Money Laundering) service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::{
    error::{ComplianceError, ComplianceResult},
    types::{AmlCheck, AmlCheckType, AmlResult, AmlMatch, RiskLevel, KycData},
};

/// AML service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlConfig {
    /// Default AML provider
    pub default_provider: String,
    /// Provider configurations
    pub providers: HashMap<String, AmlProviderConfig>,
    /// Risk thresholds
    pub risk_thresholds: RiskThresholds,
    /// Check intervals
    pub check_intervals: CheckIntervals,
    /// Monitoring settings
    pub monitoring: MonitoringConfig,
}

impl Default for AmlConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "chainalysis".to_string(),
            AmlProviderConfig {
                api_url: "https://api.chainalysis.com".to_string(),
                api_key: "".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
                enabled_checks: vec![
                    AmlCheckType::Sanctions,
                    AmlCheckType::PoliticallyExposed,
                    AmlCheckType::AdverseMedia,
                    AmlCheckType::Watchlist,
                ],
            },
        );

        Self {
            default_provider: "chainalysis".to_string(),
            providers,
            risk_thresholds: RiskThresholds::default(),
            check_intervals: CheckIntervals::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

/// AML provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlProviderConfig {
    pub api_url: String,
    pub api_key: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub enabled_checks: Vec<AmlCheckType>,
}

/// Risk score thresholds for different actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskThresholds {
    /// Threshold for automatic approval (below this score)
    pub auto_approve: f64,
    /// Threshold for manual review (between auto_approve and block)
    pub manual_review: f64,
    /// Threshold for automatic blocking (above this score)
    pub auto_block: f64,
    /// Enhanced due diligence threshold
    pub enhanced_due_diligence: f64,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            auto_approve: 0.3,
            manual_review: 0.7,
            auto_block: 0.9,
            enhanced_due_diligence: 0.5,
        }
    }
}

/// Check intervals configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckIntervals {
    /// Initial check on user registration
    pub initial_check: bool,
    /// Periodic re-screening interval in days
    pub periodic_days: u32,
    /// Transaction-based screening threshold
    pub transaction_threshold: f64,
    /// High-risk user check interval in days
    pub high_risk_days: u32,
}

impl Default for CheckIntervals {
    fn default() -> Self {
        Self {
            initial_check: true,
            periodic_days: 90,
            transaction_threshold: 10000.0,
            high_risk_days: 30,
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable real-time monitoring
    pub real_time_monitoring: bool,
    /// Alert thresholds
    pub alert_thresholds: HashMap<AmlCheckType, f64>,
    /// Notification settings
    pub notifications: NotificationConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        let mut alert_thresholds = HashMap::new();
        alert_thresholds.insert(AmlCheckType::Sanctions, 0.8);
        alert_thresholds.insert(AmlCheckType::PoliticallyExposed, 0.6);
        alert_thresholds.insert(AmlCheckType::AdverseMedia, 0.7);
        alert_thresholds.insert(AmlCheckType::Watchlist, 0.8);

        Self {
            real_time_monitoring: true,
            alert_thresholds,
            notifications: NotificationConfig::default(),
        }
    }
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub email_alerts: bool,
    pub webhook_url: Option<String>,
    pub slack_webhook: Option<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            email_alerts: true,
            webhook_url: None,
            slack_webhook: None,
        }
    }
}

/// AML screening result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlScreeningResult {
    pub id: Uuid,
    pub user_id: String,
    pub overall_result: AmlResult,
    pub overall_risk_score: f64,
    pub risk_level: RiskLevel,
    pub checks: Vec<AmlCheck>,
    pub screening_date: DateTime<Utc>,
    pub next_screening_date: Option<DateTime<Utc>>,
    pub provider: String,
    pub metadata: HashMap<String, String>,
}

/// AML provider trait
#[async_trait]
pub trait AmlProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Perform comprehensive AML screening
    async fn screen_user(&self, kyc_data: &KycData) -> ComplianceResult<AmlScreeningResult>;

    /// Perform specific AML check
    async fn perform_check(
        &self,
        check_type: AmlCheckType,
        kyc_data: &KycData,
    ) -> ComplianceResult<AmlCheck>;

    /// Monitor ongoing transactions
    async fn monitor_transaction(
        &self,
        user_id: &str,
        transaction_data: &TransactionData,
    ) -> ComplianceResult<AmlCheck>;

    /// Get updated screening result
    async fn get_screening_result(&self, screening_id: &str) -> ComplianceResult<AmlScreeningResult>;
}

/// Transaction data for AML monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub transaction_id: String,
    pub user_id: String,
    pub amount: f64,
    pub currency: String,
    pub counterparty: Option<String>,
    pub transaction_type: String,
    pub timestamp: DateTime<Utc>,
    pub blockchain_address: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Main AML service
pub struct AmlService {
    config: AmlConfig,
    providers: HashMap<String, Box<dyn AmlProvider>>,
}

impl AmlService {
    /// Create new AML service
    pub fn new(config: AmlConfig) -> Self {
        Self {
            config,
            providers: HashMap::new(),
        }
    }

    /// Register an AML provider
    pub fn register_provider(&mut self, provider: Box<dyn AmlProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    /// Perform comprehensive AML screening
    pub async fn screen_user(&self, kyc_data: &KycData) -> ComplianceResult<AmlScreeningResult> {
        let provider = self
            .providers
            .get(&self.config.default_provider)
            .ok_or_else(|| {
                ComplianceError::provider_error(
                    &self.config.default_provider,
                    "AML provider not found",
                )
            })?;

        let mut result = provider.screen_user(kyc_data).await?;

        // Apply risk assessment
        result.risk_level = self.assess_risk_level(result.overall_risk_score);
        result.overall_result = self.determine_result(result.overall_risk_score);

        // Set next screening date
        result.next_screening_date = Some(self.calculate_next_screening_date(&result));

        Ok(result)
    }

    /// Monitor transaction for AML compliance
    pub async fn monitor_transaction(
        &self,
        transaction_data: &TransactionData,
    ) -> ComplianceResult<AmlCheck> {
        let provider = self
            .providers
            .get(&self.config.default_provider)
            .ok_or_else(|| {
                ComplianceError::provider_error(
                    &self.config.default_provider,
                    "AML provider not found",
                )
            })?;

        provider
            .monitor_transaction(&transaction_data.user_id, transaction_data)
            .await
    }

    /// Check if user needs re-screening
    pub fn needs_rescreening(&self, last_screening: &AmlScreeningResult) -> bool {
        if let Some(next_screening_date) = last_screening.next_screening_date {
            Utc::now() >= next_screening_date
        } else {
            true
        }
    }

    /// Assess risk level based on score
    fn assess_risk_level(&self, risk_score: f64) -> RiskLevel {
        if risk_score >= self.config.risk_thresholds.auto_block {
            RiskLevel::Critical
        } else if risk_score >= self.config.risk_thresholds.manual_review {
            RiskLevel::High
        } else if risk_score >= self.config.risk_thresholds.enhanced_due_diligence {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Determine AML result based on risk score
    fn determine_result(&self, risk_score: f64) -> AmlResult {
        if risk_score >= self.config.risk_thresholds.auto_block {
            AmlResult::Block
        } else if risk_score >= self.config.risk_thresholds.manual_review {
            AmlResult::Review
        } else if risk_score >= self.config.risk_thresholds.auto_approve {
            AmlResult::Alert
        } else {
            AmlResult::Clear
        }
    }

    /// Calculate next screening date based on risk level
    fn calculate_next_screening_date(&self, result: &AmlScreeningResult) -> DateTime<Utc> {
        let days = match result.risk_level {
            RiskLevel::Critical | RiskLevel::High => self.config.check_intervals.high_risk_days,
            _ => self.config.check_intervals.periodic_days,
        };

        Utc::now() + chrono::Duration::days(days as i64)
    }

    /// Get risk thresholds
    pub fn get_risk_thresholds(&self) -> &RiskThresholds {
        &self.config.risk_thresholds
    }

    /// Check if transaction amount requires enhanced screening
    pub fn requires_enhanced_screening(&self, amount: f64) -> bool {
        amount >= self.config.check_intervals.transaction_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, IdentityDocument, DocumentType};

    fn create_test_kyc_data() -> KycData {
        KycData {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            date_of_birth: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            nationality: "US".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "New York".to_string(),
                state_province: Some("NY".to_string()),
                postal_code: "10001".to_string(),
                country: "US".to_string(),
            },
            identity_document: IdentityDocument {
                document_type: DocumentType::Passport,
                document_number: "123456789".to_string(),
                issuing_country: "US".to_string(),
                issue_date: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                expiry_date: Some(chrono::NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()),
            },
            verification_status: crate::types::VerificationStatus::NotStarted,
            verification_date: None,
            expiry_date: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_aml_config_default() {
        let config = AmlConfig::default();
        assert_eq!(config.default_provider, "chainalysis");
        assert!(!config.providers.is_empty());
    }

    #[test]
    fn test_risk_thresholds_default() {
        let thresholds = RiskThresholds::default();
        assert!(thresholds.auto_approve < thresholds.manual_review);
        assert!(thresholds.manual_review < thresholds.auto_block);
    }

    #[test]
    fn test_aml_service_creation() {
        let config = AmlConfig::default();
        let service = AmlService::new(config);
        assert!(service.providers.is_empty());
    }

    #[test]
    fn test_risk_level_assessment() {
        let config = AmlConfig::default();
        let service = AmlService::new(config);

        assert_eq!(service.assess_risk_level(0.1), RiskLevel::Low);
        assert_eq!(service.assess_risk_level(0.6), RiskLevel::Medium);
        assert_eq!(service.assess_risk_level(0.8), RiskLevel::High);
        assert_eq!(service.assess_risk_level(0.95), RiskLevel::Critical);
    }

    #[test]
    fn test_aml_result_determination() {
        let config = AmlConfig::default();
        let service = AmlService::new(config);

        assert_eq!(service.determine_result(0.1), AmlResult::Clear);
        assert_eq!(service.determine_result(0.5), AmlResult::Alert);
        assert_eq!(service.determine_result(0.8), AmlResult::Review);
        assert_eq!(service.determine_result(0.95), AmlResult::Block);
    }

    #[test]
    fn test_enhanced_screening_threshold() {
        let config = AmlConfig::default();
        let service = AmlService::new(config);

        assert!(!service.requires_enhanced_screening(5000.0));
        assert!(service.requires_enhanced_screening(15000.0));
    }
}
