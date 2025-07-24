// =====================================================================================
// File: core-compliance/src/jurisdiction.rs
// Description: Jurisdiction-specific compliance rules and checks
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    error::{ComplianceError, ComplianceResult},
    types::{ComplianceLevel, JurisdictionCode},
};

/// Jurisdiction-specific compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionCheck {
    pub jurisdiction: JurisdictionCode,
    pub compliant: bool,
    pub required_level: ComplianceLevel,
    pub current_level: ComplianceLevel,
    pub violations: Vec<ComplianceViolation>,
    pub requirements: Vec<ComplianceRequirement>,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub severity: ViolationSeverity,
    pub remediation: Option<String>,
}

/// Violation type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    InsufficientKyc,
    AmlViolation,
    DocumentationMissing,
    GeographicRestriction,
    LicenseRequired,
    DataProtection,
    TaxCompliance,
    Other,
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub requirement_type: RequirementType,
    pub description: String,
    pub mandatory: bool,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

/// Requirement type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementType {
    KycVerification,
    AmlScreening,
    DocumentSubmission,
    LicenseObtainment,
    DataProtectionCompliance,
    TaxRegistration,
    ReportingObligation,
    Other,
}

/// Jurisdiction-specific rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionRules {
    pub jurisdiction: JurisdictionCode,
    pub enabled: bool,
    pub minimum_compliance_level: ComplianceLevel,
    pub kyc_requirements: KycRequirements,
    pub aml_requirements: AmlRequirements,
    pub geographic_restrictions: Vec<String>,
    pub license_requirements: Vec<LicenseRequirement>,
    pub data_protection_rules: DataProtectionRules,
    pub tax_obligations: TaxObligations,
}

/// KYC requirements for jurisdiction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycRequirements {
    pub identity_verification: bool,
    pub address_verification: bool,
    pub document_verification: bool,
    pub biometric_verification: bool,
    pub enhanced_due_diligence: bool,
    pub ongoing_monitoring: bool,
}

/// AML requirements for jurisdiction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlRequirements {
    pub sanctions_screening: bool,
    pub pep_screening: bool,
    pub adverse_media_screening: bool,
    pub transaction_monitoring: bool,
    pub suspicious_activity_reporting: bool,
    pub record_keeping_years: u32,
}

/// License requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseRequirement {
    pub license_type: String,
    pub required_for: Vec<String>, // Activities that require this license
    pub issuing_authority: String,
    pub validity_period_months: u32,
}

/// Data protection rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProtectionRules {
    pub gdpr_applicable: bool,
    pub ccpa_applicable: bool,
    pub data_localization_required: bool,
    pub consent_required: bool,
    pub right_to_deletion: bool,
    pub data_retention_months: u32,
}

/// Tax obligations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxObligations {
    pub tax_reporting_required: bool,
    pub withholding_tax_applicable: bool,
    pub vat_registration_required: bool,
    pub reporting_thresholds: HashMap<String, f64>,
}

/// Jurisdiction service
pub struct JurisdictionService {
    rules: HashMap<JurisdictionCode, JurisdictionRules>,
}

impl JurisdictionService {
    /// Create new jurisdiction service
    pub fn new(enabled_jurisdictions: Vec<JurisdictionCode>) -> Self {
        let mut rules = HashMap::new();

        for jurisdiction in enabled_jurisdictions {
            rules.insert(jurisdiction, Self::create_default_rules(jurisdiction));
        }

        Self { rules }
    }

    /// Check compliance for a specific jurisdiction
    pub async fn check_compliance(
        &self,
        jurisdiction: JurisdictionCode,
        current_level: ComplianceLevel,
    ) -> ComplianceResult<Vec<JurisdictionCheck>> {
        let rules = self.rules.get(&jurisdiction).ok_or_else(|| {
            ComplianceError::UnsupportedJurisdiction {
                jurisdiction: format!("{:?}", jurisdiction),
            }
        })?;

        if !rules.enabled {
            return Err(ComplianceError::UnsupportedJurisdiction {
                jurisdiction: format!("{:?}", jurisdiction),
            });
        }

        let mut violations = Vec::new();
        let mut requirements = Vec::new();

        // Check minimum compliance level
        if current_level < rules.minimum_compliance_level {
            violations.push(ComplianceViolation {
                violation_type: ViolationType::InsufficientKyc,
                description: format!(
                    "Minimum compliance level {} required, current level is {:?}",
                    rules.minimum_compliance_level as u8, current_level
                ),
                severity: ViolationSeverity::High,
                remediation: Some("Complete additional KYC verification".to_string()),
            });
        }

        // Check KYC requirements
        self.check_kyc_requirements(&rules.kyc_requirements, &mut violations, &mut requirements);

        // Check AML requirements
        self.check_aml_requirements(&rules.aml_requirements, &mut violations, &mut requirements);

        // Check license requirements
        self.check_license_requirements(
            &rules.license_requirements,
            &mut violations,
            &mut requirements,
        );

        let check = JurisdictionCheck {
            jurisdiction,
            compliant: violations.is_empty(),
            required_level: rules.minimum_compliance_level,
            current_level,
            violations,
            requirements,
        };

        Ok(vec![check])
    }

    /// Get jurisdiction rules
    pub fn get_rules(&self, jurisdiction: JurisdictionCode) -> Option<&JurisdictionRules> {
        self.rules.get(&jurisdiction)
    }

    /// Update jurisdiction rules
    pub fn update_rules(&mut self, jurisdiction: JurisdictionCode, rules: JurisdictionRules) {
        self.rules.insert(jurisdiction, rules);
    }

    /// Check if jurisdiction is supported
    pub fn is_supported(&self, jurisdiction: JurisdictionCode) -> bool {
        self.rules.contains_key(&jurisdiction)
    }

    // Private helper methods

    fn create_default_rules(jurisdiction: JurisdictionCode) -> JurisdictionRules {
        match jurisdiction {
            JurisdictionCode::US => Self::create_us_rules(),
            JurisdictionCode::EU => Self::create_eu_rules(),
            JurisdictionCode::UK => Self::create_uk_rules(),
            JurisdictionCode::SG => Self::create_sg_rules(),
            _ => Self::create_generic_rules(jurisdiction),
        }
    }

    fn create_us_rules() -> JurisdictionRules {
        JurisdictionRules {
            jurisdiction: JurisdictionCode::US,
            enabled: true,
            minimum_compliance_level: ComplianceLevel::Standard,
            kyc_requirements: KycRequirements {
                identity_verification: true,
                address_verification: true,
                document_verification: true,
                biometric_verification: false,
                enhanced_due_diligence: true,
                ongoing_monitoring: true,
            },
            aml_requirements: AmlRequirements {
                sanctions_screening: true,
                pep_screening: true,
                adverse_media_screening: true,
                transaction_monitoring: true,
                suspicious_activity_reporting: true,
                record_keeping_years: 5,
            },
            geographic_restrictions: vec!["OFAC sanctioned countries".to_string()],
            license_requirements: vec![LicenseRequirement {
                license_type: "Money Transmitter License".to_string(),
                required_for: vec!["cryptocurrency exchange".to_string()],
                issuing_authority: "State Financial Regulators".to_string(),
                validity_period_months: 12,
            }],
            data_protection_rules: DataProtectionRules {
                gdpr_applicable: false,
                ccpa_applicable: true,
                data_localization_required: false,
                consent_required: true,
                right_to_deletion: true,
                data_retention_months: 84, // 7 years
            },
            tax_obligations: TaxObligations {
                tax_reporting_required: true,
                withholding_tax_applicable: false,
                vat_registration_required: false,
                reporting_thresholds: {
                    let mut thresholds = HashMap::new();
                    thresholds.insert("1099-K".to_string(), 600.0);
                    thresholds
                },
            },
        }
    }

    fn create_eu_rules() -> JurisdictionRules {
        JurisdictionRules {
            jurisdiction: JurisdictionCode::EU,
            enabled: true,
            minimum_compliance_level: ComplianceLevel::Enhanced,
            kyc_requirements: KycRequirements {
                identity_verification: true,
                address_verification: true,
                document_verification: true,
                biometric_verification: true,
                enhanced_due_diligence: true,
                ongoing_monitoring: true,
            },
            aml_requirements: AmlRequirements {
                sanctions_screening: true,
                pep_screening: true,
                adverse_media_screening: true,
                transaction_monitoring: true,
                suspicious_activity_reporting: true,
                record_keeping_years: 5,
            },
            geographic_restrictions: vec!["EU sanctions list".to_string()],
            license_requirements: vec![LicenseRequirement {
                license_type: "MiFID II License".to_string(),
                required_for: vec!["investment services".to_string()],
                issuing_authority: "National Competent Authority".to_string(),
                validity_period_months: 12,
            }],
            data_protection_rules: DataProtectionRules {
                gdpr_applicable: true,
                ccpa_applicable: false,
                data_localization_required: true,
                consent_required: true,
                right_to_deletion: true,
                data_retention_months: 60, // 5 years
            },
            tax_obligations: TaxObligations {
                tax_reporting_required: true,
                withholding_tax_applicable: true,
                vat_registration_required: true,
                reporting_thresholds: HashMap::new(),
            },
        }
    }

    fn create_uk_rules() -> JurisdictionRules {
        // Similar to EU but with UK-specific requirements
        let mut rules = Self::create_eu_rules();
        rules.jurisdiction = JurisdictionCode::UK;
        rules.data_protection_rules.gdpr_applicable = false; // UK GDPR instead
        rules
    }

    fn create_sg_rules() -> JurisdictionRules {
        JurisdictionRules {
            jurisdiction: JurisdictionCode::SG,
            enabled: true,
            minimum_compliance_level: ComplianceLevel::Standard,
            kyc_requirements: KycRequirements {
                identity_verification: true,
                address_verification: true,
                document_verification: true,
                biometric_verification: false,
                enhanced_due_diligence: true,
                ongoing_monitoring: true,
            },
            aml_requirements: AmlRequirements {
                sanctions_screening: true,
                pep_screening: true,
                adverse_media_screening: true,
                transaction_monitoring: true,
                suspicious_activity_reporting: true,
                record_keeping_years: 5,
            },
            geographic_restrictions: vec!["MAS restricted countries".to_string()],
            license_requirements: vec![LicenseRequirement {
                license_type: "Payment Services License".to_string(),
                required_for: vec!["digital payment token services".to_string()],
                issuing_authority: "Monetary Authority of Singapore".to_string(),
                validity_period_months: 36,
            }],
            data_protection_rules: DataProtectionRules {
                gdpr_applicable: false,
                ccpa_applicable: false,
                data_localization_required: false,
                consent_required: true,
                right_to_deletion: false,
                data_retention_months: 60,
            },
            tax_obligations: TaxObligations {
                tax_reporting_required: true,
                withholding_tax_applicable: false,
                vat_registration_required: true,
                reporting_thresholds: HashMap::new(),
            },
        }
    }

    fn create_generic_rules(jurisdiction: JurisdictionCode) -> JurisdictionRules {
        JurisdictionRules {
            jurisdiction,
            enabled: false,
            minimum_compliance_level: ComplianceLevel::Basic,
            kyc_requirements: KycRequirements {
                identity_verification: true,
                address_verification: false,
                document_verification: true,
                biometric_verification: false,
                enhanced_due_diligence: false,
                ongoing_monitoring: false,
            },
            aml_requirements: AmlRequirements {
                sanctions_screening: true,
                pep_screening: false,
                adverse_media_screening: false,
                transaction_monitoring: false,
                suspicious_activity_reporting: false,
                record_keeping_years: 3,
            },
            geographic_restrictions: Vec::new(),
            license_requirements: Vec::new(),
            data_protection_rules: DataProtectionRules {
                gdpr_applicable: false,
                ccpa_applicable: false,
                data_localization_required: false,
                consent_required: false,
                right_to_deletion: false,
                data_retention_months: 36,
            },
            tax_obligations: TaxObligations {
                tax_reporting_required: false,
                withholding_tax_applicable: false,
                vat_registration_required: false,
                reporting_thresholds: HashMap::new(),
            },
        }
    }

    fn check_kyc_requirements(
        &self,
        _requirements: &KycRequirements,
        _violations: &mut Vec<ComplianceViolation>,
        _requirements_list: &mut Vec<ComplianceRequirement>,
    ) {
        // Implementation would check actual KYC status against requirements
    }

    fn check_aml_requirements(
        &self,
        _requirements: &AmlRequirements,
        _violations: &mut Vec<ComplianceViolation>,
        _requirements_list: &mut Vec<ComplianceRequirement>,
    ) {
        // Implementation would check actual AML status against requirements
    }

    fn check_license_requirements(
        &self,
        _requirements: &[LicenseRequirement],
        _violations: &mut Vec<ComplianceViolation>,
        _requirements_list: &mut Vec<ComplianceRequirement>,
    ) {
        // Implementation would check license status against requirements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jurisdiction_service_creation() {
        let jurisdictions = vec![JurisdictionCode::US, JurisdictionCode::EU];
        let service = JurisdictionService::new(jurisdictions);

        assert!(service.is_supported(JurisdictionCode::US));
        assert!(service.is_supported(JurisdictionCode::EU));
        assert!(!service.is_supported(JurisdictionCode::JP));
    }

    #[tokio::test]
    async fn test_compliance_check() {
        let jurisdictions = vec![JurisdictionCode::US];
        let service = JurisdictionService::new(jurisdictions);

        let result = service
            .check_compliance(JurisdictionCode::US, ComplianceLevel::Basic)
            .await;

        assert!(result.is_ok());
        let checks = result.unwrap();
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].jurisdiction, JurisdictionCode::US);
    }

    #[tokio::test]
    async fn test_unsupported_jurisdiction() {
        let jurisdictions = vec![JurisdictionCode::US];
        let service = JurisdictionService::new(jurisdictions);

        let result = service
            .check_compliance(JurisdictionCode::JP, ComplianceLevel::Basic)
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_us_rules_creation() {
        let rules = JurisdictionService::create_us_rules();
        assert_eq!(rules.jurisdiction, JurisdictionCode::US);
        assert!(rules.enabled);
        assert_eq!(rules.minimum_compliance_level, ComplianceLevel::Standard);
        assert!(rules.kyc_requirements.identity_verification);
        assert!(rules.aml_requirements.sanctions_screening);
    }
}
