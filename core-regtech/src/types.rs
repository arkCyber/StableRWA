// =====================================================================================
// File: core-regtech/src/types.rs
// Description: Core types for regulatory technology operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// RegTech service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegTechConfig {
    pub aml_config: AMLConfig,
    pub kyc_config: KYCConfig,
    pub sanctions_config: SanctionsConfig,
    pub travel_rule_config: TravelRuleConfig,
    pub reporting_config: ReportingConfig,
    pub monitoring_config: MonitoringConfig,
    pub risk_assessment_config: RiskAssessmentConfig,
}

/// AML (Anti-Money Laundering) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMLConfig {
    pub transaction_threshold: Decimal,
    pub suspicious_pattern_detection: bool,
    pub real_time_monitoring: bool,
    pub alert_thresholds: HashMap<String, Decimal>,
    pub reporting_frequency: ReportingFrequency,
    pub data_retention_days: u32,
}

/// KYC (Know Your Customer) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KYCConfig {
    pub verification_levels: Vec<VerificationLevel>,
    pub document_types: Vec<DocumentType>,
    pub biometric_verification: bool,
    pub liveness_detection: bool,
    pub risk_scoring: bool,
    pub periodic_review_days: u32,
}

/// Sanctions screening configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsConfig {
    pub watchlist_sources: Vec<WatchlistSource>,
    pub screening_frequency: ScreeningFrequency,
    pub fuzzy_matching_threshold: f64,
    pub auto_block_matches: bool,
    pub manual_review_threshold: f64,
}

/// Travel Rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelRuleConfig {
    pub threshold_amount: Decimal,
    pub vasp_directory: String,
    pub message_encryption: bool,
    pub compliance_frameworks: Vec<ComplianceFramework>,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    pub report_types: Vec<ReportType>,
    pub jurisdictions: Vec<Jurisdiction>,
    pub automated_filing: bool,
    pub report_retention_years: u32,
    pub notification_settings: NotificationSettings,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub real_time_alerts: bool,
    pub alert_channels: Vec<AlertChannel>,
    pub escalation_rules: Vec<EscalationRule>,
    pub dashboard_refresh_seconds: u32,
}

/// Risk assessment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentConfig {
    pub risk_models: Vec<RiskModel>,
    pub scoring_algorithms: Vec<ScoringAlgorithm>,
    pub risk_thresholds: HashMap<String, f64>,
    pub periodic_reassessment_days: u32,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub check_id: Uuid,
    pub check_type: ComplianceCheckType,
    pub entity_id: String,
    pub status: ComplianceStatus,
    pub risk_score: f64,
    pub findings: Vec<ComplianceFinding>,
    pub recommendations: Vec<String>,
    pub checked_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Regulatory report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryReport {
    pub report_id: Uuid,
    pub report_type: ReportType,
    pub jurisdiction: Jurisdiction,
    pub reporting_period: ReportingPeriod,
    pub data: HashMap<String, serde_json::Value>,
    pub status: ReportStatus,
    pub filed_at: Option<DateTime<Utc>>,
    pub due_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub assessment_id: Uuid,
    pub entity_id: String,
    pub entity_type: EntityType,
    pub risk_score: f64,
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_measures: Vec<String>,
    pub assessed_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

/// Sanctions check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsCheck {
    pub check_id: Uuid,
    pub entity_id: String,
    pub entity_data: EntityData,
    pub matches: Vec<SanctionsMatch>,
    pub overall_status: SanctionsStatus,
    pub confidence_score: f64,
    pub checked_at: DateTime<Utc>,
    pub watchlist_version: String,
}

/// KYC profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KYCProfile {
    pub profile_id: Uuid,
    pub user_id: String,
    pub verification_level: VerificationLevel,
    pub identity_data: IdentityData,
    pub documents: Vec<VerifiedDocument>,
    pub biometric_data: Option<BiometricData>,
    pub risk_assessment: RiskAssessment,
    pub status: KYCStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// AML alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMLAlert {
    pub alert_id: Uuid,
    pub alert_type: AMLAlertType,
    pub entity_id: String,
    pub transaction_ids: Vec<String>,
    pub risk_score: f64,
    pub description: String,
    pub status: AlertStatus,
    pub assigned_to: Option<String>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Compliance status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    Pending,
    UnderReview,
    Expired,
    Exempted,
}

/// Regulatory framework enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegulatoryFramework {
    FATF,
    BSA,
    AMLD5,
    AMLD6,
    MiCA,
    GDPR,
    CCPA,
    SOX,
    BASEL,
    IFRS,
}

/// Compliance check types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceCheckType {
    AML,
    KYC,
    Sanctions,
    TravelRule,
    TaxCompliance,
    DataPrivacy,
    LicenseCompliance,
    OperationalRisk,
}

/// Report types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportType {
    SAR,       // Suspicious Activity Report
    CTR,       // Currency Transaction Report
    FBAR,      // Foreign Bank Account Report
    Form8300,  // IRS Form 8300
    CMIR,      // Cross-border Monetary Instrument Report
    FinCEN114, // FinCEN Form 114
    Custom,
}

/// Reporting frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportingFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    OnDemand,
}

/// Verification levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VerificationLevel {
    Basic,
    Enhanced,
    Premium,
    Institutional,
}

/// Document types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentType {
    Passport,
    DriverLicense,
    NationalID,
    UtilityBill,
    BankStatement,
    TaxReturn,
    BusinessLicense,
    ArticlesOfIncorporation,
}

/// Watchlist sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchlistSource {
    OFAC,
    UN,
    EU,
    HMT,
    AUSTRAC,
    PEP,
    AdverseMedia,
}

/// Screening frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScreeningFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// Compliance frameworks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceFramework {
    FATF,
    TRISA,
    OpenVASP,
    IVMS101,
}

/// Jurisdictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Jurisdiction {
    US,
    EU,
    UK,
    CA,
    AU,
    SG,
    JP,
    HK,
    CH,
    Global,
}

/// Reporting periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub period_type: PeriodType,
}

/// Period types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeriodType {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    Custom,
}

/// Report status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportStatus {
    Draft,
    Pending,
    Filed,
    Accepted,
    Rejected,
    UnderReview,
}

/// Entity types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Individual,
    Business,
    Trust,
    Foundation,
    Government,
    Other,
}

/// Risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk factors
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
    SanctionsRisk,
}

/// Sanctions match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsMatch {
    pub match_id: Uuid,
    pub watchlist_source: WatchlistSource,
    pub matched_name: String,
    pub confidence_score: f64,
    pub match_type: MatchType,
    pub additional_info: HashMap<String, String>,
}

/// Match types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Phonetic,
    Alias,
}

/// Sanctions status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SanctionsStatus {
    Clear,
    PotentialMatch,
    Confirmed,
    FalsePositive,
    UnderReview,
}

/// Entity data for sanctions screening
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub name: String,
    pub aliases: Vec<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub place_of_birth: Option<String>,
    pub nationality: Option<String>,
    pub addresses: Vec<Address>,
    pub identification_numbers: Vec<IdentificationNumber>,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
}

/// Identification number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentificationNumber {
    pub id_type: IdentificationType,
    pub id_number: String,
    pub issuing_country: String,
}

/// Identification types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentificationType {
    Passport,
    NationalID,
    DriverLicense,
    SSN,
    TaxID,
    BusinessRegistration,
}

/// Identity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityData {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub date_of_birth: chrono::NaiveDate,
    pub place_of_birth: String,
    pub nationality: String,
    pub gender: Option<Gender>,
    pub addresses: Vec<Address>,
    pub phone_numbers: Vec<String>,
    pub email_addresses: Vec<String>,
}

/// Gender enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
    PreferNotToSay,
}

/// Verified document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedDocument {
    pub document_id: Uuid,
    pub document_type: DocumentType,
    pub document_number: String,
    pub issuing_authority: String,
    pub issue_date: chrono::NaiveDate,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub verification_status: DocumentVerificationStatus,
    pub verification_date: DateTime<Utc>,
}

/// Document verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocumentVerificationStatus {
    Verified,
    Failed,
    Pending,
    Expired,
    Fraudulent,
}

/// Biometric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricData {
    pub biometric_id: Uuid,
    pub biometric_type: BiometricType,
    pub template_hash: String,
    pub liveness_score: f64,
    pub quality_score: f64,
    pub captured_at: DateTime<Utc>,
}

/// Biometric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiometricType {
    Fingerprint,
    FaceRecognition,
    VoiceRecognition,
    IrisRecognition,
}

/// KYC status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KYCStatus {
    Pending,
    Approved,
    Rejected,
    UnderReview,
    Expired,
    Suspended,
}

/// AML alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AMLAlertType {
    StructuringPattern,
    UnusualTransactionAmount,
    HighRiskCountry,
    RapidMovement,
    SanctionsMatch,
    PEPMatch,
    AdverseMediaMatch,
    UnusualBehavior,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertStatus {
    Open,
    InProgress,
    Resolved,
    FalsePositive,
    Escalated,
    Closed,
}

/// Compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub finding_id: Uuid,
    pub finding_type: FindingType,
    pub severity: Severity,
    pub description: String,
    pub evidence: Vec<String>,
    pub remediation_required: bool,
}

/// Finding types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FindingType {
    PolicyViolation,
    RegulatoryBreach,
    RiskExceedance,
    DataInconsistency,
    ProcessFailure,
    SystemError,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskModel {
    RulesBased,
    MachineLearning,
    Hybrid,
    Statistical,
}

/// Scoring algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScoringAlgorithm {
    WeightedSum,
    LogisticRegression,
    RandomForest,
    NeuralNetwork,
    Ensemble,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub sms_notifications: bool,
    pub webhook_notifications: bool,
    pub dashboard_notifications: bool,
    pub notification_recipients: Vec<String>,
}

/// Alert channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertChannel {
    Email,
    SMS,
    Webhook,
    Dashboard,
    Slack,
    Teams,
}

/// Escalation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub rule_id: Uuid,
    pub trigger_condition: String,
    pub escalation_level: u32,
    pub escalation_delay_minutes: u32,
    pub escalation_recipients: Vec<String>,
}

impl Default for RegTechConfig {
    fn default() -> Self {
        Self {
            aml_config: AMLConfig {
                transaction_threshold: Decimal::new(10000, 2), // $100.00
                suspicious_pattern_detection: true,
                real_time_monitoring: true,
                alert_thresholds: HashMap::new(),
                reporting_frequency: ReportingFrequency::Daily,
                data_retention_days: 2555, // 7 years
            },
            kyc_config: KYCConfig {
                verification_levels: vec![VerificationLevel::Basic, VerificationLevel::Enhanced],
                document_types: vec![DocumentType::Passport, DocumentType::DriverLicense],
                biometric_verification: true,
                liveness_detection: true,
                risk_scoring: true,
                periodic_review_days: 365,
            },
            sanctions_config: SanctionsConfig {
                watchlist_sources: vec![WatchlistSource::OFAC, WatchlistSource::UN],
                screening_frequency: ScreeningFrequency::RealTime,
                fuzzy_matching_threshold: 0.8,
                auto_block_matches: false,
                manual_review_threshold: 0.7,
            },
            travel_rule_config: TravelRuleConfig {
                threshold_amount: Decimal::new(300000, 2), // $3000.00
                vasp_directory: "https://vaspdirectory.org".to_string(),
                message_encryption: true,
                compliance_frameworks: vec![ComplianceFramework::FATF],
            },
            reporting_config: ReportingConfig {
                report_types: vec![ReportType::SAR, ReportType::CTR],
                jurisdictions: vec![Jurisdiction::US],
                automated_filing: false,
                report_retention_years: 7,
                notification_settings: NotificationSettings {
                    email_notifications: true,
                    sms_notifications: false,
                    webhook_notifications: true,
                    dashboard_notifications: true,
                    notification_recipients: Vec::new(),
                },
            },
            monitoring_config: MonitoringConfig {
                real_time_alerts: true,
                alert_channels: vec![AlertChannel::Email, AlertChannel::Dashboard],
                escalation_rules: Vec::new(),
                dashboard_refresh_seconds: 30,
            },
            risk_assessment_config: RiskAssessmentConfig {
                risk_models: vec![RiskModel::RulesBased, RiskModel::MachineLearning],
                scoring_algorithms: vec![ScoringAlgorithm::WeightedSum],
                risk_thresholds: HashMap::new(),
                periodic_reassessment_days: 90,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regtech_config_default() {
        let config = RegTechConfig::default();
        assert_eq!(
            config.aml_config.transaction_threshold,
            Decimal::new(10000, 2)
        );
        assert!(config.aml_config.suspicious_pattern_detection);
        assert_eq!(config.kyc_config.verification_levels.len(), 2);
        assert_eq!(config.sanctions_config.fuzzy_matching_threshold, 0.8);
    }

    #[test]
    fn test_compliance_status() {
        let statuses = vec![
            ComplianceStatus::Compliant,
            ComplianceStatus::NonCompliant,
            ComplianceStatus::Pending,
            ComplianceStatus::UnderReview,
            ComplianceStatus::Expired,
            ComplianceStatus::Exempted,
        ];

        for status in statuses {
            assert!(matches!(
                status,
                ComplianceStatus::Compliant
                    | ComplianceStatus::NonCompliant
                    | ComplianceStatus::Pending
                    | ComplianceStatus::UnderReview
                    | ComplianceStatus::Expired
                    | ComplianceStatus::Exempted
            ));
        }
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_verification_level_ordering() {
        assert!(VerificationLevel::Basic < VerificationLevel::Enhanced);
        assert!(VerificationLevel::Enhanced < VerificationLevel::Premium);
        assert!(VerificationLevel::Premium < VerificationLevel::Institutional);
    }
}
