// =====================================================================================
// File: core-compliance/src/lib.rs
// Description: Compliance and regulatory framework for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Compliance Module
//!
//! This module provides comprehensive compliance and regulatory functionality for the
//! StableRWA platform, including KYC/AML services, regulatory reporting, jurisdictional
//! compliance, and audit trails.

pub mod aml;
pub mod audit;
pub mod error;
pub mod jurisdiction;
pub mod kyc;
pub mod reporting;
pub mod sanctions;
pub mod service;
pub mod types;

// Re-export main types and traits
pub use aml::{AmlProvider, AmlService, EnterpriseAmlProvider, TransactionPatternAnalyzer};
pub use audit::AuditService;
pub use error::{ComplianceError, ComplianceResult};
pub use jurisdiction::JurisdictionService;
pub use kyc::{EnterpriseKycProvider, KycProvider, KycService};
pub use reporting::ReportingService;
pub use sanctions::{SanctionsList, SanctionsService};
pub use service::ComplianceService;
pub use types::{
    AmlCheck, AmlCheckType, AmlMatch, AmlResult, AuditEvent, ComplianceLevel, ComplianceReport,
    ComplianceStatus, JurisdictionCode, KycData, RiskLevel, VerificationStatus,
};
pub use kyc::KycResult;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Main compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Default compliance level required
    pub default_compliance_level: ComplianceLevel,
    /// Enabled jurisdictions
    pub enabled_jurisdictions: Vec<JurisdictionCode>,
    /// KYC provider configuration
    pub kyc_config: kyc::KycConfig,
    /// AML provider configuration
    pub aml_config: aml::AmlConfig,
    /// Reporting configuration
    pub reporting_config: reporting::ReportingConfig,
    /// Audit configuration
    pub audit_config: audit::AuditConfig,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            default_compliance_level: ComplianceLevel::Standard,
            enabled_jurisdictions: vec![
                JurisdictionCode::US,
                JurisdictionCode::EU,
                JurisdictionCode::UK,
                JurisdictionCode::SG,
            ],
            kyc_config: kyc::KycConfig::default(),
            aml_config: aml::AmlConfig::default(),
            reporting_config: reporting::ReportingConfig::default(),
            audit_config: audit::AuditConfig::default(),
        }
    }
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    pub id: Uuid,
    pub user_id: String,
    pub status: ComplianceStatus,
    pub level: ComplianceLevel,
    pub risk_level: RiskLevel,
    pub kyc_result: Option<crate::kyc::KycResult>,
    pub aml_result: Option<types::AmlResult>,
    pub jurisdiction_checks: Vec<jurisdiction::JurisdictionCheck>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

impl ComplianceCheckResult {
    pub fn new(user_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            status: ComplianceStatus::Pending,
            level: ComplianceLevel::Basic,
            risk_level: RiskLevel::Unknown,
            kyc_result: None,
            aml_result: None,
            jurisdiction_checks: Vec::new(),
            created_at: Utc::now(),
            expires_at: None,
            notes: None,
        }
    }

    /// Check if the compliance result is valid and not expired
    pub fn is_valid(&self) -> bool {
        match self.status {
            ComplianceStatus::Approved => {
                if let Some(expires_at) = self.expires_at {
                    Utc::now() < expires_at
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Check if the compliance level meets the required minimum
    pub fn meets_level(&self, required_level: ComplianceLevel) -> bool {
        self.is_valid() && self.level >= required_level
    }

    /// Check if the risk level is acceptable
    pub fn is_risk_acceptable(&self, max_risk: RiskLevel) -> bool {
        self.risk_level <= max_risk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_config_default() {
        let config = ComplianceConfig::default();
        assert_eq!(config.default_compliance_level, ComplianceLevel::Standard);
        assert!(!config.enabled_jurisdictions.is_empty());
    }

    #[test]
    fn test_compliance_check_result_creation() {
        let result = ComplianceCheckResult::new("user123".to_string());
        assert_eq!(result.user_id, "user123");
        assert_eq!(result.status, ComplianceStatus::Pending);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_compliance_check_result_validity() {
        let mut result = ComplianceCheckResult::new("user123".to_string());

        // Should not be valid when pending
        assert!(!result.is_valid());

        // Should be valid when approved without expiry
        result.status = ComplianceStatus::Approved;
        assert!(result.is_valid());

        // Should not be valid when expired
        result.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(!result.is_valid());

        // Should be valid when not expired
        result.expires_at = Some(Utc::now() + chrono::Duration::hours(1));
        assert!(result.is_valid());
    }

    #[test]
    fn test_compliance_level_comparison() {
        let mut result = ComplianceCheckResult::new("user123".to_string());
        result.status = ComplianceStatus::Approved;
        result.level = ComplianceLevel::Enhanced;

        assert!(result.meets_level(ComplianceLevel::Basic));
        assert!(result.meets_level(ComplianceLevel::Standard));
        assert!(result.meets_level(ComplianceLevel::Enhanced));
        assert!(!result.meets_level(ComplianceLevel::Premium));
    }

    #[test]
    fn test_risk_level_acceptance() {
        let mut result = ComplianceCheckResult::new("user123".to_string());
        result.risk_level = RiskLevel::Medium;

        assert!(!result.is_risk_acceptable(RiskLevel::Low));
        assert!(result.is_risk_acceptable(RiskLevel::Medium));
        assert!(result.is_risk_acceptable(RiskLevel::High));
    }
}
