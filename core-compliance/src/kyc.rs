// =====================================================================================
// File: core-compliance/src/kyc.rs
// Description: KYC (Know Your Customer) service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::{
    error::{ComplianceError, ComplianceResult},
    types::{KycData, VerificationStatus, ComplianceLevel, RiskLevel},
};

/// KYC service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycConfig {
    /// Default KYC provider
    pub default_provider: String,
    /// Provider configurations
    pub providers: HashMap<String, ProviderConfig>,
    /// Verification requirements by compliance level
    pub level_requirements: HashMap<ComplianceLevel, KycRequirements>,
    /// Document retention period in days
    pub retention_period_days: u32,
    /// Auto-renewal settings
    pub auto_renewal: bool,
    /// Renewal period in days before expiry
    pub renewal_period_days: u32,
}

impl Default for KycConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();
        providers.insert(
            "jumio".to_string(),
            ProviderConfig {
                api_url: "https://api.jumio.com".to_string(),
                api_key: "".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
            },
        );

        let mut level_requirements = HashMap::new();
        level_requirements.insert(
            ComplianceLevel::Basic,
            KycRequirements {
                identity_verification: true,
                address_verification: false,
                document_verification: true,
                biometric_verification: false,
                enhanced_due_diligence: false,
            },
        );
        level_requirements.insert(
            ComplianceLevel::Standard,
            KycRequirements {
                identity_verification: true,
                address_verification: true,
                document_verification: true,
                biometric_verification: false,
                enhanced_due_diligence: false,
            },
        );
        level_requirements.insert(
            ComplianceLevel::Enhanced,
            KycRequirements {
                identity_verification: true,
                address_verification: true,
                document_verification: true,
                biometric_verification: true,
                enhanced_due_diligence: false,
            },
        );
        level_requirements.insert(
            ComplianceLevel::Premium,
            KycRequirements {
                identity_verification: true,
                address_verification: true,
                document_verification: true,
                biometric_verification: true,
                enhanced_due_diligence: true,
            },
        );

        Self {
            default_provider: "jumio".to_string(),
            providers,
            level_requirements,
            retention_period_days: 2555, // 7 years
            auto_renewal: true,
            renewal_period_days: 30,
        }
    }
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_url: String,
    pub api_key: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

/// KYC requirements for different compliance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycRequirements {
    pub identity_verification: bool,
    pub address_verification: bool,
    pub document_verification: bool,
    pub biometric_verification: bool,
    pub enhanced_due_diligence: bool,
}

/// KYC verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycResult {
    pub id: Uuid,
    pub user_id: String,
    pub provider: String,
    pub status: VerificationStatus,
    pub compliance_level: ComplianceLevel,
    pub risk_level: RiskLevel,
    pub verification_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub verification_details: KycVerificationDetails,
    pub documents: Vec<DocumentVerification>,
    pub notes: Option<String>,
}

/// Detailed KYC verification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycVerificationDetails {
    pub identity_verified: bool,
    pub address_verified: bool,
    pub document_verified: bool,
    pub biometric_verified: bool,
    pub enhanced_due_diligence_completed: bool,
    pub verification_score: f64,
    pub risk_indicators: Vec<String>,
}

/// Document verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVerification {
    pub document_type: String,
    pub verification_status: VerificationStatus,
    pub confidence_score: f64,
    pub extracted_data: HashMap<String, String>,
    pub verification_date: DateTime<Utc>,
}

/// KYC provider trait
#[async_trait]
pub trait KycProvider: Send + Sync {
    /// Get provider name
    fn name(&self) -> &str;

    /// Verify user identity
    async fn verify_identity(&self, kyc_data: &KycData) -> ComplianceResult<KycResult>;

    /// Check verification status
    async fn check_status(&self, verification_id: &str) -> ComplianceResult<VerificationStatus>;

    /// Get verification result
    async fn get_result(&self, verification_id: &str) -> ComplianceResult<KycResult>;

    /// Update verification data
    async fn update_verification(
        &self,
        verification_id: &str,
        kyc_data: &KycData,
    ) -> ComplianceResult<KycResult>;
}

/// Main KYC service
pub struct KycService {
    config: KycConfig,
    providers: HashMap<String, Box<dyn KycProvider>>,
}

impl KycService {
    /// Create new KYC service
    pub fn new(config: KycConfig) -> Self {
        Self {
            config,
            providers: HashMap::new(),
        }
    }

    /// Register a KYC provider
    pub fn register_provider(&mut self, provider: Box<dyn KycProvider>) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    /// Perform KYC verification
    pub async fn verify_user(
        &self,
        kyc_data: &KycData,
        required_level: ComplianceLevel,
    ) -> ComplianceResult<KycResult> {
        // Get provider
        let provider = self
            .providers
            .get(&self.config.default_provider)
            .ok_or_else(|| {
                ComplianceError::provider_error(
                    &self.config.default_provider,
                    "Provider not found",
                )
            })?;

        // Check requirements
        let requirements = self
            .config
            .level_requirements
            .get(&required_level)
            .ok_or_else(|| {
                ComplianceError::validation_error(
                    "compliance_level",
                    "Unsupported compliance level",
                )
            })?;

        // Validate KYC data meets requirements
        self.validate_kyc_data(kyc_data, requirements)?;

        // Perform verification
        let mut result = provider.verify_identity(kyc_data).await?;

        // Set compliance level based on verification
        result.compliance_level = self.determine_compliance_level(&result, required_level)?;

        // Set expiry date
        if self.config.auto_renewal {
            result.expiry_date = Some(
                Utc::now() + chrono::Duration::days(self.config.retention_period_days as i64),
            );
        }

        Ok(result)
    }

    /// Check if KYC is valid and meets requirements
    pub fn is_kyc_valid(
        &self,
        result: &KycResult,
        required_level: ComplianceLevel,
    ) -> ComplianceResult<bool> {
        // Check status
        if result.status != VerificationStatus::Verified {
            return Ok(false);
        }

        // Check compliance level
        if result.compliance_level < required_level {
            return Ok(false);
        }

        // Check expiry
        if let Some(expiry_date) = result.expiry_date {
            if Utc::now() > expiry_date {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get KYC requirements for compliance level
    pub fn get_requirements(&self, level: ComplianceLevel) -> Option<&KycRequirements> {
        self.config.level_requirements.get(&level)
    }

    /// Validate KYC data against requirements
    fn validate_kyc_data(
        &self,
        kyc_data: &KycData,
        requirements: &KycRequirements,
    ) -> ComplianceResult<()> {
        if requirements.identity_verification && kyc_data.first_name.is_empty() {
            return Err(ComplianceError::validation_error(
                "first_name",
                "First name is required",
            ));
        }

        if requirements.address_verification && kyc_data.address.street.is_empty() {
            return Err(ComplianceError::validation_error(
                "address",
                "Address is required",
            ));
        }

        if requirements.document_verification && kyc_data.identity_document.document_number.is_empty() {
            return Err(ComplianceError::validation_error(
                "identity_document",
                "Identity document is required",
            ));
        }

        Ok(())
    }

    /// Determine compliance level based on verification result
    fn determine_compliance_level(
        &self,
        result: &KycResult,
        requested_level: ComplianceLevel,
    ) -> ComplianceResult<ComplianceLevel> {
        let details = &result.verification_details;

        let achieved_level = if details.enhanced_due_diligence_completed
            && details.biometric_verified
            && details.document_verified
            && details.address_verified
            && details.identity_verified
        {
            ComplianceLevel::Premium
        } else if details.biometric_verified
            && details.document_verified
            && details.address_verified
            && details.identity_verified
        {
            ComplianceLevel::Enhanced
        } else if details.document_verified && details.address_verified && details.identity_verified
        {
            ComplianceLevel::Standard
        } else if details.document_verified && details.identity_verified {
            ComplianceLevel::Basic
        } else {
            return Err(ComplianceError::kyc_error(
                "Insufficient verification for any compliance level",
            ));
        };

        if achieved_level < requested_level {
            return Err(ComplianceError::insufficient_compliance_level(
                format!("{:?}", requested_level),
                format!("{:?}", achieved_level),
            ));
        }

        Ok(achieved_level)
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
            verification_status: VerificationStatus::NotStarted,
            verification_date: None,
            expiry_date: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_kyc_config_default() {
        let config = KycConfig::default();
        assert_eq!(config.default_provider, "jumio");
        assert!(!config.providers.is_empty());
        assert!(!config.level_requirements.is_empty());
    }

    #[test]
    fn test_kyc_service_creation() {
        let config = KycConfig::default();
        let service = KycService::new(config);
        assert!(service.providers.is_empty());
    }

    #[test]
    fn test_kyc_data_validation() {
        let config = KycConfig::default();
        let service = KycService::new(config);
        let kyc_data = create_test_kyc_data();
        let requirements = service.get_requirements(ComplianceLevel::Basic).unwrap();

        let result = service.validate_kyc_data(&kyc_data, requirements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kyc_requirements_by_level() {
        let config = KycConfig::default();
        let service = KycService::new(config);

        let basic_req = service.get_requirements(ComplianceLevel::Basic).unwrap();
        assert!(basic_req.identity_verification);
        assert!(!basic_req.address_verification);

        let premium_req = service.get_requirements(ComplianceLevel::Premium).unwrap();
        assert!(premium_req.enhanced_due_diligence);
        assert!(premium_req.biometric_verification);
    }
}
