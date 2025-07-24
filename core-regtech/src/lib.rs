// =====================================================================================
// File: core-regtech/src/lib.rs
// Description: Regulatory technology (RegTech) automation for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core RegTech Module
//!
//! This module provides comprehensive regulatory technology (RegTech) automation
//! for the StableRWA platform, including AML/KYC compliance, regulatory reporting,
//! risk assessment, and automated compliance monitoring.

pub mod aml;
pub mod audit_trail;
pub mod document_analysis;
pub mod error;
pub mod kyc;
pub mod monitoring;
pub mod regulatory_calendar;
pub mod reporting;
pub mod risk_assessment;
pub mod sanctions;
pub mod travel_rule;
pub mod types;

pub mod service;

// Re-export main types and traits
pub use aml::{
    AMLConfig, AMLService, PatternDetection, SuspiciousActivityReport, TransactionData,
    TransactionMonitoring,
};
pub use audit_trail::{AuditConfig, AuditEvent, AuditReport, AuditTrail, AuditTrailImpl};
pub use document_analysis::{
    DocumentAnalysis, DocumentAnalyzer, DocumentAnalyzerImpl, DocumentConfig,
};
pub use error::{RegTechError, RegTechResult};
pub use kyc::{
    BiometricSubmission, DocumentSubmission, IdentityVerification, KYCConfig, KYCService,
    ReviewResult,
};
pub use monitoring::{
    ComplianceMetrics, ComplianceMonitor, ComplianceMonitorImpl, MonitoringConfig,
};
pub use regulatory_calendar::{
    CalendarConfig, ComplianceDeadline, RegulatoryCalendar, RegulatoryCalendarImpl,
    RegulatoryUpdate,
};
pub use reporting::{ReportConfig, ReportingService, ReportingServiceImpl};
pub use risk_assessment::{
    RiskAssessmentService, RiskAssessmentServiceImpl, RiskConfig, RiskMatrix,
};
pub use sanctions::{
    SanctionsAlert, SanctionsConfig, SanctionsScreening, SanctionsService, WatchlistManager,
};
pub use service::{RegTechHealthStatus, RegTechService, RegTechServiceImpl};
pub use travel_rule::{
    TravelRuleConfig, TravelRuleMessage, TravelRuleService, TravelRuleServiceImpl,
};
pub use types::{
    AMLAlert, ComplianceCheck, ComplianceStatus, KYCProfile, RegulatoryFramework, RegulatoryReport,
    RiskAssessment, SanctionsCheck,
};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main RegTech service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegTechServiceConfig {
    /// AML configuration
    pub aml_config: AMLConfig,
    /// KYC configuration
    pub kyc_config: KYCConfig,
    /// Reporting configuration
    pub reporting_config: ReportConfig,
    /// Monitoring configuration
    pub monitoring_config: MonitoringConfig,
    /// Travel Rule configuration
    pub travel_rule_config: TravelRuleConfig,
    /// Sanctions configuration
    pub sanctions_config: SanctionsConfig,
    /// Risk assessment configuration
    pub risk_config: RiskConfig,
    /// Document analysis configuration
    pub document_config: DocumentConfig,
    /// Global RegTech settings
    pub global_settings: GlobalRegTechSettings,
}

impl Default for RegTechServiceConfig {
    fn default() -> Self {
        Self {
            aml_config: AMLConfig::default(),
            kyc_config: KYCConfig::default(),
            reporting_config: ReportConfig::default(),
            monitoring_config: MonitoringConfig::default(),
            travel_rule_config: TravelRuleConfig::default(),
            sanctions_config: SanctionsConfig::default(),
            risk_config: RiskConfig::default(),
            document_config: DocumentConfig::default(),
            global_settings: GlobalRegTechSettings::default(),
        }
    }
}

/// Global RegTech settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalRegTechSettings {
    /// Enable real-time monitoring
    pub enable_real_time_monitoring: bool,
    /// Enable automated reporting
    pub enable_automated_reporting: bool,
    /// Enable AI-powered analysis
    pub enable_ai_analysis: bool,
    /// Default regulatory framework
    pub default_regulatory_framework: RegulatoryFramework,
    /// Compliance threshold
    pub compliance_threshold: Decimal,
    /// Alert escalation timeout in minutes
    pub alert_escalation_timeout_minutes: u32,
    /// Enable multi-jurisdiction support
    pub enable_multi_jurisdiction: bool,
    /// Data retention period in days
    pub data_retention_days: u32,
    /// Enable audit trail
    pub enable_audit_trail: bool,
    /// Maximum risk score
    pub max_risk_score: u32,
}

impl Default for GlobalRegTechSettings {
    fn default() -> Self {
        Self {
            enable_real_time_monitoring: true,
            enable_automated_reporting: true,
            enable_ai_analysis: true,
            default_regulatory_framework: RegulatoryFramework::FATF,
            compliance_threshold: Decimal::new(95, 2), // 95%
            alert_escalation_timeout_minutes: 30,
            enable_multi_jurisdiction: true,
            data_retention_days: 2555, // 7 years
            enable_audit_trail: true,
            max_risk_score: 100,
        }
    }
}

/// RegTech metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegTechMetrics {
    pub total_compliance_checks: u64,
    pub successful_checks: u64,
    pub failed_checks: u64,
    pub aml_alerts_24h: u64,
    pub kyc_verifications_24h: u64,
    pub sanctions_hits_24h: u64,
    pub reports_generated_24h: u64,
    pub average_check_time_ms: f64,
    pub compliance_rate: Decimal,
    pub false_positive_rate: Decimal,
    pub regulatory_violations: u64,
    pub framework_breakdown: HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

/// RegTech health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegTechHealthStatus {
    pub overall_status: String,
    pub aml_status: String,
    pub kyc_status: String,
    pub reporting_status: String,
    pub monitoring_status: String,
    pub sanctions_status: String,
    pub risk_assessment_status: String,
    pub document_analysis_status: String,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regtech_config_default() {
        let config = RegTechServiceConfig::default();
        assert_eq!(
            config.aml_config.transaction_threshold,
            Decimal::new(1000000, 2)
        );
        assert!(config.aml_config.pattern_detection_enabled);
        assert!(config.kyc_config.biometric_verification);
        assert_eq!(config.reporting_config.report_types.len(), 2);
        assert!(config.monitoring_config.real_time_alerts);
    }

    #[test]
    fn test_global_regtech_settings() {
        let settings = GlobalRegTechSettings::default();
        assert!(settings.enable_real_time_monitoring);
        assert!(settings.enable_automated_reporting);
        assert!(settings.enable_ai_analysis);
        assert_eq!(
            settings.default_regulatory_framework,
            RegulatoryFramework::FATF
        );
        assert_eq!(settings.compliance_threshold, Decimal::new(95, 2));
        assert_eq!(settings.data_retention_days, 2555);
    }

    #[test]
    fn test_aml_config() {
        let config = AMLConfig::default();
        assert_eq!(config.transaction_threshold, Decimal::new(1000000, 2));
        assert!(config.pattern_detection_enabled);
        assert!(config.real_time_monitoring);
        assert_eq!(config.suspicious_patterns.len(), 2);
    }
}
