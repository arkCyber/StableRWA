// =====================================================================================
// File: core-compliance/src/aml.rs
// Description: AML (Anti-Money Laundering) service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{ComplianceError, ComplianceResult},
    types::{AmlCheck, AmlCheckType, AmlMatch, AmlResult, KycData, RiskLevel},
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
    async fn get_screening_result(
        &self,
        screening_id: &str,
    ) -> ComplianceResult<AmlScreeningResult>;
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
                    self.config.default_provider.clone(),
                    "AML provider not found".to_string(),
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
                    self.config.default_provider.clone(),
                    "AML provider not found".to_string(),
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

/// Transaction pattern analysis for AML monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPattern {
    pub user_id: String,
    pub pattern_type: PatternType,
    pub frequency: u32,
    pub total_amount: f64,
    pub average_amount: f64,
    pub time_window: chrono::Duration,
    pub risk_score: f64,
    pub detected_at: DateTime<Utc>,
}

/// Types of suspicious transaction patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    /// Rapid succession of transactions
    RapidTransactions,
    /// Structuring (breaking large amounts into smaller ones)
    Structuring,
    /// Round number transactions
    RoundNumbers,
    /// Unusual time patterns
    UnusualTiming,
    /// Geographic anomalies
    GeographicAnomaly,
    /// Velocity anomalies
    VelocityAnomaly,
    /// Layering pattern
    Layering,
}

/// Suspicious activity report data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousActivityReport {
    pub id: Uuid,
    pub user_id: String,
    pub report_type: SarType,
    pub activity_description: String,
    pub transaction_ids: Vec<String>,
    pub total_amount: f64,
    pub currency: String,
    pub detection_date: DateTime<Utc>,
    pub filing_date: Option<DateTime<Utc>>,
    pub status: SarStatus,
    pub risk_level: RiskLevel,
    pub patterns: Vec<TransactionPattern>,
    pub metadata: HashMap<String, String>,
}

/// SAR (Suspicious Activity Report) types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SarType {
    MoneyLaundering,
    TerroristFinancing,
    Structuring,
    IdentityTheft,
    CyberCrime,
    Other,
}

/// SAR status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SarStatus {
    Detected,
    UnderReview,
    Filed,
    Dismissed,
}

/// Enterprise AML provider with advanced monitoring capabilities
pub struct EnterpriseAmlProvider {
    config: AmlProviderConfig,
    client: reqwest::Client,
    pattern_analyzer: TransactionPatternAnalyzer,
}

#[async_trait]
impl AmlProvider for EnterpriseAmlProvider {
    fn name(&self) -> &str {
        "enterprise_aml"
    }

    async fn screen_user(&self, kyc_data: &KycData) -> ComplianceResult<AmlScreeningResult> {
        let mut checks = Vec::new();

        // Perform sanctions screening
        let sanctions_result = self.screen_sanctions(kyc_data).await?;
        checks.push(sanctions_result);

        // Perform PEP screening
        let pep_result = self.screen_pep(kyc_data).await?;
        checks.push(pep_result);

        // Perform adverse media screening
        let media_result = self.screen_adverse_media(kyc_data).await?;
        checks.push(media_result);

        // Calculate overall risk level
        let overall_risk = checks.iter()
            .map(|check| check.risk_level)
            .max()
            .unwrap_or(RiskLevel::Low);

        Ok(AmlScreeningResult {
            id: uuid::Uuid::new_v4(),
            user_id: kyc_data.user_id.clone(),
            overall_result: if overall_risk == RiskLevel::Critical {
                AmlResult::Review
            } else if overall_risk == RiskLevel::High {
                AmlResult::Review
            } else {
                AmlResult::Clear
            },
            overall_risk_score: overall_risk as u8 as f64 / 4.0, // Convert enum to score
            risk_level: overall_risk,
            checks,
            screening_date: Utc::now(),
            next_screening_date: Some(Utc::now() + chrono::Duration::days(90)),
            provider: self.name().to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn monitor_transaction(&self, user_id: &str, transaction: &TransactionData) -> ComplianceResult<AmlCheck> {
        // Monitor transaction for suspicious patterns
        let risk_score = if transaction.amount > 10000.0 {
            0.8
        } else if transaction.amount > 5000.0 {
            0.6
        } else {
            0.3
        };

        let risk_level = if risk_score > 0.7 {
            RiskLevel::High
        } else if risk_score > 0.5 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        let result = if risk_level == RiskLevel::High {
            AmlResult::Review
        } else {
            AmlResult::Clear
        };

        Ok(AmlCheck {
            id: uuid::Uuid::new_v4(),
            user_id: user_id.to_string(),
            check_type: AmlCheckType::Enhanced,
            result: result.clone(),
            overall_result: result,
            risk_level,
            risk_score,
            matches: Vec::new(),
            checked_at: Utc::now(),
            provider: self.name().to_string(),
        })
    }

    async fn perform_check(
        &self,
        check_type: AmlCheckType,
        kyc_data: &KycData,
    ) -> ComplianceResult<AmlCheck> {
        match check_type {
            AmlCheckType::Sanctions => self.screen_sanctions(kyc_data).await,
            AmlCheckType::PoliticallyExposed => self.screen_pep(kyc_data).await,
            AmlCheckType::AdverseMedia => self.screen_adverse_media(kyc_data).await,
            AmlCheckType::Watchlist => {
                // Perform watchlist screening (similar to sanctions)
                self.screen_sanctions(kyc_data).await
            }
            AmlCheckType::Enhanced => {
                // Perform comprehensive check
                let sanctions = self.screen_sanctions(kyc_data).await?;
                let pep = self.screen_pep(kyc_data).await?;
                let media = self.screen_adverse_media(kyc_data).await?;

                // Return the highest risk result
                let checks = vec![&sanctions, &pep, &media];
                let highest_risk = checks
                    .iter()
                    .max_by_key(|check| check.risk_level as u8)
                    .unwrap();

                Ok((*highest_risk).clone())
            }
        }
    }

    async fn get_screening_result(
        &self,
        screening_id: &str,
    ) -> ComplianceResult<AmlScreeningResult> {
        // In a real implementation, this would fetch from database
        // For now, return a mock result
        Ok(AmlScreeningResult {
            id: uuid::Uuid::parse_str(screening_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            user_id: "mock_user".to_string(),
            overall_result: AmlResult::Clear,
            overall_risk_score: 0.3,
            risk_level: RiskLevel::Low,
            checks: Vec::new(),
            screening_date: Utc::now(),
            next_screening_date: Some(Utc::now() + chrono::Duration::days(90)),
            provider: self.name().to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }
}

impl EnterpriseAmlProvider {
    pub fn new(config: AmlProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            pattern_analyzer: TransactionPatternAnalyzer::new(),
        }
    }

    /// Perform comprehensive sanctions screening
    async fn screen_sanctions(&self, kyc_data: &KycData) -> ComplianceResult<AmlCheck> {
        let mut matches = Vec::new();
        let mut risk_score = 0.0;

        // Check name against sanctions lists
        let name_match_score = self.check_name_match(&kyc_data.first_name, &kyc_data.last_name).await?;
        if name_match_score > 0.7 {
            matches.push(AmlMatch {
                match_type: AmlCheckType::Sanctions,
                confidence: name_match_score,
                description: "Name match on sanctions list".to_string(),
                source: "OFAC".to_string(),
            });
            risk_score += name_match_score * 0.8;
        }

        // Check nationality/country
        let country_risk = self.assess_country_risk(&kyc_data.nationality).await?;
        if country_risk > 0.5 {
            matches.push(AmlMatch {
                match_type: AmlCheckType::Sanctions,
                confidence: country_risk,
                description: "High-risk jurisdiction".to_string(),
                source: "FATF".to_string(),
            });
            risk_score += country_risk * 0.3;
        }

        let result = if risk_score > 0.8 {
            AmlResult::Block
        } else if risk_score > 0.5 {
            AmlResult::Review
        } else if risk_score > 0.2 {
            AmlResult::Alert
        } else {
            AmlResult::Clear
        };

        Ok(AmlCheck {
            id: Uuid::new_v4(),
            user_id: kyc_data.user_id.clone(),
            check_type: AmlCheckType::Sanctions,
            result,
            overall_result: result,
            risk_level: self.score_to_risk_level(risk_score),
            risk_score,
            matches,
            checked_at: Utc::now(),
            provider: "EnterpriseAmlProvider".to_string(),
        })
    }

    /// Check for Politically Exposed Persons (PEP)
    async fn screen_pep(&self, kyc_data: &KycData) -> ComplianceResult<AmlCheck> {
        // Simulate PEP screening
        let pep_score = self.calculate_pep_score(kyc_data).await?;
        let mut matches = Vec::new();

        if pep_score > 0.6 {
            matches.push(AmlMatch {
                match_type: AmlCheckType::PoliticallyExposed,
                confidence: pep_score,
                description: "Potential PEP match".to_string(),
                source: "PEP Database".to_string(),
            });
        }

        let result = if pep_score > 0.8 {
            AmlResult::Review
        } else if pep_score > 0.5 {
            AmlResult::Alert
        } else {
            AmlResult::Clear
        };

        Ok(AmlCheck {
            id: Uuid::new_v4(),
            user_id: kyc_data.user_id.clone(),
            check_type: AmlCheckType::PoliticallyExposed,
            result,
            overall_result: result,
            risk_level: self.score_to_risk_level(pep_score),
            risk_score: pep_score,
            matches,
            checked_at: Utc::now(),
            provider: "EnterpriseAmlProvider".to_string(),
        })
    }

    /// Screen against adverse media
    async fn screen_adverse_media(&self, kyc_data: &KycData) -> ComplianceResult<AmlCheck> {
        // Simulate adverse media screening
        let media_score = self.search_adverse_media(kyc_data).await?;
        let mut matches = Vec::new();

        if media_score > 0.5 {
            matches.push(AmlMatch {
                match_type: AmlCheckType::AdverseMedia,
                confidence: media_score,
                description: "Adverse media mentions found".to_string(),
                source: "Media Screening".to_string(),
            });
        }

        let result = if media_score > 0.7 {
            AmlResult::Review
        } else if media_score > 0.4 {
            AmlResult::Alert
        } else {
            AmlResult::Clear
        };

        Ok(AmlCheck {
            id: Uuid::new_v4(),
            user_id: kyc_data.user_id.clone(),
            check_type: AmlCheckType::AdverseMedia,
            result,
            overall_result: result,
            risk_level: self.score_to_risk_level(media_score),
            risk_score: media_score,
            matches,
            checked_at: Utc::now(),
            provider: "EnterpriseAmlProvider".to_string(),
        })
    }

    /// Analyze transaction patterns for suspicious activity
    pub async fn analyze_transaction_patterns(
        &self,
        user_id: &str,
        transactions: &[TransactionData],
    ) -> ComplianceResult<Vec<TransactionPattern>> {
        self.pattern_analyzer.analyze_patterns(user_id, transactions).await
    }

    /// Generate suspicious activity report
    pub async fn generate_sar(
        &self,
        user_id: &str,
        patterns: Vec<TransactionPattern>,
        transactions: &[TransactionData],
    ) -> ComplianceResult<SuspiciousActivityReport> {
        let total_amount: f64 = transactions.iter().map(|t| t.amount).sum();
        let currency = transactions.first()
            .map(|t| t.currency.clone())
            .unwrap_or_else(|| "USD".to_string());

        let risk_level = if patterns.iter().any(|p| p.risk_score > 0.8) {
            RiskLevel::Critical
        } else if patterns.iter().any(|p| p.risk_score > 0.6) {
            RiskLevel::High
        } else {
            RiskLevel::Medium
        };

        let sar_type = self.determine_sar_type(&patterns);
        let activity_description = self.generate_activity_description(&patterns, transactions);

        Ok(SuspiciousActivityReport {
            id: Uuid::new_v4(),
            user_id: user_id.to_string(),
            report_type: sar_type,
            activity_description,
            transaction_ids: transactions.iter().map(|t| t.transaction_id.clone()).collect(),
            total_amount,
            currency,
            detection_date: Utc::now(),
            filing_date: None,
            status: SarStatus::Detected,
            risk_level,
            patterns,
            metadata: HashMap::new(),
        })
    }

    // Helper methods
    async fn check_name_match(&self, first_name: &str, last_name: &str) -> ComplianceResult<f64> {
        // Simulate name matching against sanctions lists
        let full_name = format!("{} {}", first_name, last_name).to_lowercase();

        // Simple fuzzy matching simulation
        let high_risk_names = vec!["john doe", "jane smith"]; // Mock sanctions list

        for risk_name in &high_risk_names {
            if full_name.contains(risk_name) {
                return Ok(0.9);
            }
        }

        Ok(0.1) // Low risk by default
    }

    async fn assess_country_risk(&self, country: &str) -> ComplianceResult<f64> {
        // Simulate country risk assessment
        let high_risk_countries = vec!["XX", "YY", "ZZ"]; // Mock high-risk countries

        if high_risk_countries.contains(&country) {
            Ok(0.8)
        } else {
            Ok(0.2)
        }
    }

    async fn calculate_pep_score(&self, kyc_data: &KycData) -> ComplianceResult<f64> {
        // Simulate PEP scoring
        let full_name = format!("{} {}", kyc_data.first_name, kyc_data.last_name);

        // Mock PEP detection logic
        if full_name.to_lowercase().contains("minister") ||
           full_name.to_lowercase().contains("president") {
            Ok(0.9)
        } else {
            Ok(0.1)
        }
    }

    async fn search_adverse_media(&self, kyc_data: &KycData) -> ComplianceResult<f64> {
        // Simulate adverse media search
        let search_terms = vec!["fraud", "corruption", "money laundering"];
        let full_name = format!("{} {}", kyc_data.first_name, kyc_data.last_name);

        // Mock media search
        for term in &search_terms {
            if full_name.to_lowercase().contains(term) {
                return Ok(0.8);
            }
        }

        Ok(0.1)
    }

    fn score_to_risk_level(&self, score: f64) -> RiskLevel {
        if score >= 0.8 {
            RiskLevel::Critical
        } else if score >= 0.6 {
            RiskLevel::High
        } else if score >= 0.4 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    fn determine_sar_type(&self, patterns: &[TransactionPattern]) -> SarType {
        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::Structuring => return SarType::Structuring,
                PatternType::Layering => return SarType::MoneyLaundering,
                _ => continue,
            }
        }
        SarType::Other
    }

    fn generate_activity_description(&self, patterns: &[TransactionPattern], transactions: &[TransactionData]) -> String {
        let mut description = String::new();

        description.push_str(&format!("Suspicious activity detected involving {} transactions ", transactions.len()));
        description.push_str(&format!("with total amount of ${:.2}. ",
            transactions.iter().map(|t| t.amount).sum::<f64>()));

        for pattern in patterns {
            description.push_str(&format!("Pattern detected: {:?} with risk score {:.2}. ",
                pattern.pattern_type, pattern.risk_score));
        }

        description
    }
}

/// Transaction pattern analyzer for detecting suspicious activities
pub struct TransactionPatternAnalyzer {
    structuring_threshold: f64,
    rapid_transaction_window: chrono::Duration,
    velocity_threshold: f64,
}

impl TransactionPatternAnalyzer {
    pub fn new() -> Self {
        Self {
            structuring_threshold: 10000.0, // $10,000 threshold for structuring detection
            rapid_transaction_window: chrono::Duration::minutes(30),
            velocity_threshold: 50000.0, // $50,000 in short time period
        }
    }

    /// Analyze transaction patterns for suspicious activity
    pub async fn analyze_patterns(
        &self,
        user_id: &str,
        transactions: &[TransactionData],
    ) -> ComplianceResult<Vec<TransactionPattern>> {
        let mut patterns = Vec::new();

        // Check for structuring patterns
        if let Some(structuring) = self.detect_structuring(user_id, transactions).await? {
            patterns.push(structuring);
        }

        // Check for rapid transactions
        if let Some(rapid) = self.detect_rapid_transactions(user_id, transactions).await? {
            patterns.push(rapid);
        }

        // Check for round number patterns
        if let Some(round_numbers) = self.detect_round_numbers(user_id, transactions).await? {
            patterns.push(round_numbers);
        }

        // Check for unusual timing patterns
        if let Some(timing) = self.detect_unusual_timing(user_id, transactions).await? {
            patterns.push(timing);
        }

        // Check for velocity anomalies
        if let Some(velocity) = self.detect_velocity_anomaly(user_id, transactions).await? {
            patterns.push(velocity);
        }

        Ok(patterns)
    }

    /// Detect structuring patterns (breaking large amounts into smaller ones)
    async fn detect_structuring(
        &self,
        user_id: &str,
        transactions: &[TransactionData],
    ) -> ComplianceResult<Option<TransactionPattern>> {
        let mut small_transactions = Vec::new();
        let threshold = self.structuring_threshold * 0.9; // Just below reporting threshold

        for transaction in transactions {
            if transaction.amount > threshold * 0.5 && transaction.amount < threshold {
                small_transactions.push(transaction);
            }
        }

        if small_transactions.len() >= 3 {
            let total_amount: f64 = small_transactions.iter().map(|t| t.amount).sum();
            let average_amount = total_amount / small_transactions.len() as f64;

            // Calculate risk score based on frequency and amounts
            let risk_score = if small_transactions.len() > 5 && total_amount > self.structuring_threshold {
                0.9
            } else if small_transactions.len() > 3 && total_amount > threshold {
                0.7
            } else {
                0.5
            };

            return Ok(Some(TransactionPattern {
                user_id: user_id.to_string(),
                pattern_type: PatternType::Structuring,
                frequency: small_transactions.len() as u32,
                total_amount,
                average_amount,
                time_window: chrono::Duration::days(1), // Assume within a day
                risk_score,
                detected_at: Utc::now(),
            }));
        }

        Ok(None)
    }

    /// Detect rapid succession of transactions
    async fn detect_rapid_transactions(
        &self,
        user_id: &str,
        transactions: &[TransactionData],
    ) -> ComplianceResult<Option<TransactionPattern>> {
        if transactions.len() < 5 {
            return Ok(None);
        }

        // Sort transactions by timestamp
        let mut sorted_transactions = transactions.to_vec();
        sorted_transactions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let mut rapid_groups = Vec::new();
        let mut current_group = vec![&sorted_transactions[0]];

        for i in 1..sorted_transactions.len() {
            let time_diff = sorted_transactions[i].timestamp - sorted_transactions[i-1].timestamp;

            if time_diff <= self.rapid_transaction_window {
                current_group.push(&sorted_transactions[i]);
            } else {
                if current_group.len() >= 5 {
                    rapid_groups.push(current_group.clone());
                }
                current_group = vec![&sorted_transactions[i]];
            }
        }

        if current_group.len() >= 5 {
            rapid_groups.push(current_group);
        }

        if !rapid_groups.is_empty() {
            let largest_group = rapid_groups.iter().max_by_key(|g| g.len()).unwrap();
            let total_amount: f64 = largest_group.iter().map(|t| t.amount).sum();
            let average_amount = total_amount / largest_group.len() as f64;

            let risk_score = if largest_group.len() > 10 {
                0.8
            } else if largest_group.len() > 7 {
                0.6
            } else {
                0.4
            };

            return Ok(Some(TransactionPattern {
                user_id: user_id.to_string(),
                pattern_type: PatternType::RapidTransactions,
                frequency: largest_group.len() as u32,
                total_amount,
                average_amount,
                time_window: self.rapid_transaction_window,
                risk_score,
                detected_at: Utc::now(),
            }));
        }

        Ok(None)
    }

    /// Detect round number transaction patterns
    async fn detect_round_numbers(
        &self,
        user_id: &str,
        transactions: &[TransactionData],
    ) -> ComplianceResult<Option<TransactionPattern>> {
        let round_transactions: Vec<_> = transactions
            .iter()
            .filter(|t| self.is_round_number(t.amount))
            .collect();

        if round_transactions.len() >= 3 &&
           (round_transactions.len() as f64 / transactions.len() as f64) > 0.7 {
            let total_amount: f64 = round_transactions.iter().map(|t| t.amount).sum();
            let average_amount = total_amount / round_transactions.len() as f64;

            let risk_score = if round_transactions.len() > 5 {
                0.6
            } else {
                0.4
            };

            return Ok(Some(TransactionPattern {
                user_id: user_id.to_string(),
                pattern_type: PatternType::RoundNumbers,
                frequency: round_transactions.len() as u32,
                total_amount,
                average_amount,
                time_window: chrono::Duration::days(7),
                risk_score,
                detected_at: Utc::now(),
            }));
        }

        Ok(None)
    }

    /// Check if amount is a round number
    pub fn is_round_number(&self, amount: f64) -> bool {
        let rounded = amount.round();
        (amount - rounded).abs() < 0.01 && // Is essentially a whole number
        (rounded % 100.0 == 0.0 || // Multiple of 100
         rounded % 1000.0 == 0.0 || // Multiple of 1000
         rounded % 10000.0 == 0.0) // Multiple of 10000
    }

    /// Detect unusual timing patterns
    async fn detect_unusual_timing(
        &self,
        user_id: &str,
        transactions: &[TransactionData],
    ) -> ComplianceResult<Option<TransactionPattern>> {
        // Check for transactions at unusual hours (e.g., 2-5 AM)
        let unusual_hour_transactions: Vec<_> = transactions
            .iter()
            .filter(|t| {
                let hour = t.timestamp.hour();
                hour >= 2 && hour <= 5
            })
            .collect();

        if unusual_hour_transactions.len() >= 3 {
            let total_amount: f64 = unusual_hour_transactions.iter().map(|t| t.amount).sum();
            let average_amount = total_amount / unusual_hour_transactions.len() as f64;

            let risk_score = if unusual_hour_transactions.len() > 5 {
                0.7
            } else {
                0.5
            };

            return Ok(Some(TransactionPattern {
                user_id: user_id.to_string(),
                pattern_type: PatternType::UnusualTiming,
                frequency: unusual_hour_transactions.len() as u32,
                total_amount,
                average_amount,
                time_window: chrono::Duration::hours(24),
                risk_score,
                detected_at: Utc::now(),
            }));
        }

        Ok(None)
    }

    /// Detect velocity anomalies
    async fn detect_velocity_anomaly(
        &self,
        user_id: &str,
        transactions: &[TransactionData],
    ) -> ComplianceResult<Option<TransactionPattern>> {
        if transactions.len() < 2 {
            return Ok(None);
        }

        // Sort transactions by timestamp
        let mut sorted_transactions = transactions.to_vec();
        sorted_transactions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Check for high velocity in short time windows
        let mut max_velocity = 0.0;
        let mut velocity_window_start = 0;
        let mut velocity_window_end = 0;

        for i in 0..sorted_transactions.len() {
            let mut window_amount = sorted_transactions[i].amount;
            let window_start_time = sorted_transactions[i].timestamp;

            for j in (i + 1)..sorted_transactions.len() {
                let time_diff = sorted_transactions[j].timestamp - window_start_time;
                if time_diff <= self.rapid_transaction_window {
                    window_amount += sorted_transactions[j].amount;

                    if window_amount > max_velocity {
                        max_velocity = window_amount;
                        velocity_window_start = i;
                        velocity_window_end = j;
                    }
                } else {
                    break;
                }
            }
        }

        if max_velocity > self.velocity_threshold {
            let window_transactions = &sorted_transactions[velocity_window_start..=velocity_window_end];
            let average_amount = max_velocity / window_transactions.len() as f64;

            let risk_score = if max_velocity > self.velocity_threshold * 2.0 {
                0.9
            } else {
                0.7
            };

            return Ok(Some(TransactionPattern {
                user_id: user_id.to_string(),
                pattern_type: PatternType::VelocityAnomaly,
                frequency: window_transactions.len() as u32,
                total_amount: max_velocity,
                average_amount,
                time_window: self.rapid_transaction_window,
                risk_score,
                detected_at: Utc::now(),
            }));
        }

        Ok(None)
    }
}

impl Default for TransactionPatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, DocumentType, IdentityDocument};

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

    #[tokio::test]
    async fn test_enterprise_aml_provider() {
        let config = AmlProviderConfig {
            api_url: "https://test.example.com".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            enabled_checks: vec![AmlCheckType::Sanctions, AmlCheckType::PoliticallyExposed],
        };

        let provider = EnterpriseAmlProvider::new(config);
        assert_eq!(provider.name(), "enterprise_aml");

        let kyc_data = create_test_kyc_data();
        let result = provider.screen_user(&kyc_data).await;
        assert!(result.is_ok());

        let screening_result = result.unwrap();
        assert_eq!(screening_result.user_id, "user123");
        assert_eq!(screening_result.provider, "enterprise_aml");
        assert!(!screening_result.checks.is_empty());
    }

    #[tokio::test]
    async fn test_sanctions_screening() {
        let config = AmlProviderConfig {
            api_url: "https://test.example.com".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            enabled_checks: vec![AmlCheckType::Sanctions],
        };

        let provider = EnterpriseAmlProvider::new(config);
        let kyc_data = create_test_kyc_data();

        let sanctions_check = provider.screen_sanctions(&kyc_data).await;
        assert!(sanctions_check.is_ok());

        let check = sanctions_check.unwrap();
        assert_eq!(check.check_type, AmlCheckType::Sanctions);
        assert_eq!(check.user_id, "user123");
    }

    #[tokio::test]
    async fn test_pep_screening() {
        let config = AmlProviderConfig {
            api_url: "https://test.example.com".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            enabled_checks: vec![AmlCheckType::PoliticallyExposed],
        };

        let provider = EnterpriseAmlProvider::new(config);
        let kyc_data = create_test_kyc_data();

        let pep_check = provider.screen_pep(&kyc_data).await;
        assert!(pep_check.is_ok());

        let check = pep_check.unwrap();
        assert_eq!(check.check_type, AmlCheckType::PoliticallyExposed);
        assert_eq!(check.user_id, "user123");
    }

    #[tokio::test]
    async fn test_adverse_media_screening() {
        let config = AmlProviderConfig {
            api_url: "https://test.example.com".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            enabled_checks: vec![AmlCheckType::AdverseMedia],
        };

        let provider = EnterpriseAmlProvider::new(config);
        let kyc_data = create_test_kyc_data();

        let media_check = provider.screen_adverse_media(&kyc_data).await;
        assert!(media_check.is_ok());

        let check = media_check.unwrap();
        assert_eq!(check.check_type, AmlCheckType::AdverseMedia);
        assert_eq!(check.user_id, "user123");
    }

    #[tokio::test]
    async fn test_transaction_monitoring() {
        let config = AmlProviderConfig {
            api_url: "https://test.example.com".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            enabled_checks: vec![AmlCheckType::Enhanced],
        };

        let provider = EnterpriseAmlProvider::new(config);

        let transaction = TransactionData {
            transaction_id: "tx123".to_string(),
            user_id: "user123".to_string(),
            amount: 15000.0, // Large amount
            currency: "USD".to_string(),
            counterparty: Some("counterparty123".to_string()),
            transaction_type: "transfer".to_string(),
            timestamp: Utc::now(),
            blockchain_address: Some("0x123...".to_string()),
            metadata: HashMap::new(),
        };

        let monitor_result = provider.monitor_transaction("user123", &transaction).await;
        assert!(monitor_result.is_ok());

        let check = monitor_result.unwrap();
        assert_eq!(check.user_id, "user123");
        assert!(check.risk_score > 0.0);
    }

    #[tokio::test]
    async fn test_transaction_pattern_analysis() {
        let analyzer = TransactionPatternAnalyzer::new();

        // Create test transactions for structuring pattern
        let transactions = vec![
            TransactionData {
                transaction_id: "tx1".to_string(),
                user_id: "user123".to_string(),
                amount: 9500.0, // Just below $10k threshold
                currency: "USD".to_string(),
                counterparty: None,
                transaction_type: "transfer".to_string(),
                timestamp: Utc::now(),
                blockchain_address: None,
                metadata: HashMap::new(),
            },
            TransactionData {
                transaction_id: "tx2".to_string(),
                user_id: "user123".to_string(),
                amount: 9800.0,
                currency: "USD".to_string(),
                counterparty: None,
                transaction_type: "transfer".to_string(),
                timestamp: Utc::now() + chrono::Duration::minutes(10),
                blockchain_address: None,
                metadata: HashMap::new(),
            },
            TransactionData {
                transaction_id: "tx3".to_string(),
                user_id: "user123".to_string(),
                amount: 9700.0,
                currency: "USD".to_string(),
                counterparty: None,
                transaction_type: "transfer".to_string(),
                timestamp: Utc::now() + chrono::Duration::minutes(20),
                blockchain_address: None,
                metadata: HashMap::new(),
            },
        ];

        let patterns = analyzer.analyze_patterns("user123", &transactions).await;
        assert!(patterns.is_ok());

        let pattern_list = patterns.unwrap();
        // Should detect structuring pattern
        assert!(!pattern_list.is_empty());
    }

    #[tokio::test]
    async fn test_rapid_transaction_detection() {
        let analyzer = TransactionPatternAnalyzer::new();

        // Create rapid transactions
        let mut transactions = Vec::new();
        let base_time = Utc::now();

        for i in 0..6 {
            transactions.push(TransactionData {
                transaction_id: format!("tx{}", i),
                user_id: "user123".to_string(),
                amount: 1000.0,
                currency: "USD".to_string(),
                counterparty: None,
                transaction_type: "transfer".to_string(),
                timestamp: base_time + chrono::Duration::minutes(i * 2), // 2 minutes apart
                blockchain_address: None,
                metadata: HashMap::new(),
            });
        }

        let rapid_pattern = analyzer.detect_rapid_transactions("user123", &transactions).await;
        assert!(rapid_pattern.is_ok());

        let pattern = rapid_pattern.unwrap();
        if let Some(p) = pattern {
            assert_eq!(p.pattern_type, PatternType::RapidTransactions);
            assert_eq!(p.frequency, 6);
        }
    }

    #[test]
    fn test_round_number_detection() {
        let analyzer = TransactionPatternAnalyzer::new();

        assert!(analyzer.is_round_number(1000.0));
        assert!(analyzer.is_round_number(5000.0));
        assert!(analyzer.is_round_number(10000.0));
        assert!(!analyzer.is_round_number(1234.56));
        assert!(!analyzer.is_round_number(999.0));
    }

    #[tokio::test]
    async fn test_sar_generation() {
        let config = AmlProviderConfig {
            api_url: "https://test.example.com".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            enabled_checks: vec![AmlCheckType::Enhanced],
        };

        let provider = EnterpriseAmlProvider::new(config);

        let patterns = vec![
            TransactionPattern {
                user_id: "user123".to_string(),
                pattern_type: PatternType::Structuring,
                frequency: 5,
                total_amount: 45000.0,
                average_amount: 9000.0,
                time_window: chrono::Duration::hours(2),
                risk_score: 0.8,
                detected_at: Utc::now(),
            }
        ];

        let transactions = vec![
            TransactionData {
                transaction_id: "tx1".to_string(),
                user_id: "user123".to_string(),
                amount: 9000.0,
                currency: "USD".to_string(),
                counterparty: None,
                transaction_type: "transfer".to_string(),
                timestamp: Utc::now(),
                blockchain_address: None,
                metadata: HashMap::new(),
            }
        ];

        let sar = provider.generate_sar("user123", patterns, &transactions).await;
        assert!(sar.is_ok());

        let report = sar.unwrap();
        assert_eq!(report.user_id, "user123");
        assert_eq!(report.status, SarStatus::Detected);
        assert!(!report.patterns.is_empty());
    }
}
