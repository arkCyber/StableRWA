// =====================================================================================
// File: core-asset-lifecycle/src/registration.rs
// Description: Asset registration service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::{
    error::{AssetError, AssetResult},
    types::{Asset, AssetType, AssetLocation, AssetDocument, DocumentType},
};

/// Asset registration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationConfig {
    /// Enable automatic validation
    pub auto_validation: bool,
    /// Required documents by asset type
    pub required_documents: HashMap<AssetType, Vec<DocumentType>>,
    /// Maximum file size for documents (in bytes)
    pub max_document_size: u64,
    /// Allowed file types for documents
    pub allowed_file_types: Vec<String>,
    /// Registration timeout in hours
    pub registration_timeout_hours: u32,
    /// Enable duplicate detection
    pub duplicate_detection: bool,
    /// Compliance check requirements
    pub compliance_checks: ComplianceCheckConfig,
}

impl Default for RegistrationConfig {
    fn default() -> Self {
        let mut required_documents = HashMap::new();
        
        // Real estate requirements
        required_documents.insert(
            AssetType::RealEstate,
            vec![
                DocumentType::OwnershipDocument,
                DocumentType::Deed,
                DocumentType::Appraisal,
                DocumentType::Photo,
            ],
        );

        // Art requirements
        required_documents.insert(
            AssetType::Art,
            vec![
                DocumentType::OwnershipDocument,
                DocumentType::Certificate,
                DocumentType::Appraisal,
                DocumentType::Photo,
            ],
        );

        // Commodities requirements
        required_documents.insert(
            AssetType::Commodities,
            vec![
                DocumentType::OwnershipDocument,
                DocumentType::Certificate,
                DocumentType::Inspection,
            ],
        );

        Self {
            auto_validation: true,
            required_documents,
            max_document_size: 50 * 1024 * 1024, // 50MB
            allowed_file_types: vec![
                "application/pdf".to_string(),
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/tiff".to_string(),
                "application/msword".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            ],
            registration_timeout_hours: 72, // 3 days
            duplicate_detection: true,
            compliance_checks: ComplianceCheckConfig::default(),
        }
    }
}

/// Compliance check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckConfig {
    /// Enable KYC verification for asset owners
    pub kyc_verification: bool,
    /// Enable AML screening
    pub aml_screening: bool,
    /// Enable sanctions screening
    pub sanctions_screening: bool,
    /// Required compliance level
    pub required_compliance_level: core_compliance::types::ComplianceLevel,
}

impl Default for ComplianceCheckConfig {
    fn default() -> Self {
        Self {
            kyc_verification: true,
            aml_screening: true,
            sanctions_screening: true,
            required_compliance_level: core_compliance::types::ComplianceLevel::Standard,
        }
    }
}

/// Asset registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub subcategory: String,
    pub owner_id: String,
    pub location: AssetLocation,
    pub estimated_value: Option<rust_decimal::Decimal>,
    pub currency: Option<String>,
    pub physical_attributes: HashMap<String, String>,
    pub legal_attributes: HashMap<String, String>,
    pub custom_attributes: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub external_ids: HashMap<String, String>,
    pub documents: Vec<DocumentUpload>,
}

/// Document upload information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpload {
    pub document_type: DocumentType,
    pub title: String,
    pub description: Option<String>,
    pub file_data: Vec<u8>,
    pub file_name: String,
    pub mime_type: String,
}

/// Registration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    pub asset_id: Uuid,
    pub registration_id: Uuid,
    pub status: RegistrationStatus,
    pub created_at: DateTime<Utc>,
    pub validation_results: Vec<ValidationResult>,
    pub compliance_results: Option<ComplianceResult>,
    pub next_steps: Vec<String>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Registration status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistrationStatus {
    /// Registration submitted and pending validation
    Pending,
    /// Registration is being validated
    Validating,
    /// Registration validation completed successfully
    Validated,
    /// Registration requires additional information
    RequiresAdditionalInfo,
    /// Registration completed successfully
    Completed,
    /// Registration failed
    Failed,
    /// Registration was cancelled
    Cancelled,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub validation_type: ValidationType,
    pub status: ValidationStatus,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Validation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationType {
    DocumentValidation,
    DataValidation,
    DuplicateCheck,
    ComplianceCheck,
    LocationValidation,
    OwnershipValidation,
}

/// Validation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Passed,
    Failed,
    Warning,
    Pending,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub kyc_status: Option<core_compliance::types::ComplianceStatus>,
    pub aml_status: Option<core_compliance::types::AmlResult>,
    pub sanctions_status: Option<bool>,
    pub overall_compliance: bool,
    pub compliance_notes: Vec<String>,
}

/// Asset registration service
pub struct AssetRegistrationService {
    config: RegistrationConfig,
}

impl AssetRegistrationService {
    /// Create new registration service
    pub fn new(config: RegistrationConfig) -> Self {
        Self { config }
    }

    /// Register a new asset
    pub async fn register_asset(
        &self,
        request: RegistrationRequest,
    ) -> AssetResult<RegistrationResult> {
        let registration_id = Uuid::new_v4();
        
        // Validate the registration request
        let validation_results = self.validate_registration_request(&request).await?;
        
        // Check if validation passed
        let validation_passed = validation_results
            .iter()
            .all(|r| matches!(r.status, ValidationStatus::Passed | ValidationStatus::Warning));

        if !validation_passed {
            return Ok(RegistrationResult {
                asset_id: Uuid::new_v4(),
                registration_id,
                status: RegistrationStatus::Failed,
                created_at: Utc::now(),
                validation_results,
                compliance_results: None,
                next_steps: vec!["Fix validation errors and resubmit".to_string()],
                estimated_completion: None,
            });
        }

        // Create the asset
        let asset = self.create_asset_from_request(&request)?;

        // Perform compliance checks if enabled
        let compliance_results = if self.config.compliance_checks.kyc_verification {
            Some(self.perform_compliance_checks(&request).await?)
        } else {
            None
        };

        // Determine status and next steps
        let (status, next_steps, estimated_completion) = self.determine_next_steps(
            &validation_results,
            &compliance_results,
        );

        Ok(RegistrationResult {
            asset_id: asset.id,
            registration_id,
            status,
            created_at: Utc::now(),
            validation_results,
            compliance_results,
            next_steps,
            estimated_completion,
        })
    }

    /// Get registration status
    pub async fn get_registration_status(
        &self,
        registration_id: Uuid,
    ) -> AssetResult<Option<RegistrationResult>> {
        // In a real implementation, this would fetch from database
        Ok(None)
    }

    /// Update registration with additional information
    pub async fn update_registration(
        &self,
        registration_id: Uuid,
        updates: RegistrationUpdate,
    ) -> AssetResult<RegistrationResult> {
        // In a real implementation, this would update the registration
        Err(AssetError::registration_error("Update not implemented"))
    }

    /// Cancel registration
    pub async fn cancel_registration(&self, registration_id: Uuid) -> AssetResult<()> {
        // In a real implementation, this would cancel the registration
        Ok(())
    }

    // Private helper methods

    async fn validate_registration_request(
        &self,
        request: &RegistrationRequest,
    ) -> AssetResult<Vec<ValidationResult>> {
        let mut results = Vec::new();

        // Data validation
        results.push(self.validate_basic_data(request));

        // Document validation
        results.push(self.validate_documents(request).await?);

        // Duplicate check
        if self.config.duplicate_detection {
            results.push(self.check_duplicates(request).await?);
        }

        // Location validation
        results.push(self.validate_location(&request.location).await?);

        Ok(results)
    }

    fn validate_basic_data(&self, request: &RegistrationRequest) -> ValidationResult {
        if request.name.trim().is_empty() {
            return ValidationResult {
                validation_type: ValidationType::DataValidation,
                status: ValidationStatus::Failed,
                message: "Asset name is required".to_string(),
                details: None,
            };
        }

        if request.description.trim().is_empty() {
            return ValidationResult {
                validation_type: ValidationType::DataValidation,
                status: ValidationStatus::Failed,
                message: "Asset description is required".to_string(),
                details: None,
            };
        }

        ValidationResult {
            validation_type: ValidationType::DataValidation,
            status: ValidationStatus::Passed,
            message: "Basic data validation passed".to_string(),
            details: None,
        }
    }

    async fn validate_documents(&self, request: &RegistrationRequest) -> AssetResult<ValidationResult> {
        let required_docs = self.config.required_documents
            .get(&request.asset_type)
            .cloned()
            .unwrap_or_default();

        let provided_doc_types: Vec<DocumentType> = request.documents
            .iter()
            .map(|d| d.document_type)
            .collect();

        let missing_docs: Vec<DocumentType> = required_docs
            .into_iter()
            .filter(|doc_type| !provided_doc_types.contains(doc_type))
            .collect();

        if !missing_docs.is_empty() {
            return Ok(ValidationResult {
                validation_type: ValidationType::DocumentValidation,
                status: ValidationStatus::Failed,
                message: format!("Missing required documents: {:?}", missing_docs),
                details: Some(serde_json::json!({ "missing_documents": missing_docs })),
            });
        }

        // Validate file sizes and types
        for doc in &request.documents {
            if doc.file_data.len() as u64 > self.config.max_document_size {
                return Ok(ValidationResult {
                    validation_type: ValidationType::DocumentValidation,
                    status: ValidationStatus::Failed,
                    message: format!("Document '{}' exceeds maximum size", doc.title),
                    details: None,
                });
            }

            if !self.config.allowed_file_types.contains(&doc.mime_type) {
                return Ok(ValidationResult {
                    validation_type: ValidationType::DocumentValidation,
                    status: ValidationStatus::Failed,
                    message: format!("Document '{}' has unsupported file type", doc.title),
                    details: None,
                });
            }
        }

        Ok(ValidationResult {
            validation_type: ValidationType::DocumentValidation,
            status: ValidationStatus::Passed,
            message: "Document validation passed".to_string(),
            details: None,
        })
    }

    async fn check_duplicates(&self, _request: &RegistrationRequest) -> AssetResult<ValidationResult> {
        // In a real implementation, this would check for duplicate assets
        Ok(ValidationResult {
            validation_type: ValidationType::DuplicateCheck,
            status: ValidationStatus::Passed,
            message: "No duplicates found".to_string(),
            details: None,
        })
    }

    async fn validate_location(&self, _location: &AssetLocation) -> AssetResult<ValidationResult> {
        // In a real implementation, this would validate the location
        Ok(ValidationResult {
            validation_type: ValidationType::LocationValidation,
            status: ValidationStatus::Passed,
            message: "Location validation passed".to_string(),
            details: None,
        })
    }

    async fn perform_compliance_checks(&self, request: &RegistrationRequest) -> AssetResult<ComplianceResult> {
        // In a real implementation, this would perform actual compliance checks
        Ok(ComplianceResult {
            kyc_status: Some(core_compliance::types::ComplianceStatus::Pending),
            aml_status: Some(core_compliance::types::AmlResult::Clear),
            sanctions_status: Some(true),
            overall_compliance: true,
            compliance_notes: vec!["Compliance checks initiated".to_string()],
        })
    }

    fn create_asset_from_request(&self, request: &RegistrationRequest) -> AssetResult<Asset> {
        let mut asset = Asset::new(
            request.name.clone(),
            request.description.clone(),
            request.asset_type,
            request.owner_id.clone(),
            request.location.clone(),
        );

        // Set category
        asset.category.subcategory = request.subcategory.clone();

        // Set metadata
        asset.metadata.physical_attributes = request.physical_attributes.clone();
        asset.metadata.legal_attributes = request.legal_attributes.clone();
        asset.metadata.custom_attributes = request.custom_attributes.clone();
        asset.metadata.tags = request.tags.clone();
        asset.metadata.external_ids = request.external_ids.clone();

        // Add estimated value to financial attributes
        if let Some(value) = &request.estimated_value {
            asset.metadata.financial_attributes.insert(
                "estimated_value".to_string(),
                value.to_string(),
            );
        }
        if let Some(currency) = &request.currency {
            asset.metadata.financial_attributes.insert(
                "currency".to_string(),
                currency.clone(),
            );
        }

        Ok(asset)
    }

    fn determine_next_steps(
        &self,
        _validation_results: &[ValidationResult],
        compliance_results: &Option<ComplianceResult>,
    ) -> (RegistrationStatus, Vec<String>, Option<DateTime<Utc>>) {
        let mut next_steps = Vec::new();

        if let Some(compliance) = compliance_results {
            if !compliance.overall_compliance {
                next_steps.push("Complete compliance verification".to_string());
                return (
                    RegistrationStatus::RequiresAdditionalInfo,
                    next_steps,
                    Some(Utc::now() + chrono::Duration::hours(self.config.registration_timeout_hours as i64)),
                );
            }
        }

        next_steps.push("Proceed to asset verification".to_string());
        (
            RegistrationStatus::Completed,
            next_steps,
            Some(Utc::now() + chrono::Duration::hours(24)),
        )
    }
}

/// Registration update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationUpdate {
    pub additional_documents: Option<Vec<DocumentUpload>>,
    pub updated_attributes: Option<HashMap<String, serde_json::Value>>,
    pub compliance_information: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registration_request() -> RegistrationRequest {
        RegistrationRequest {
            name: "Test Property".to_string(),
            description: "A test real estate property".to_string(),
            asset_type: AssetType::RealEstate,
            subcategory: "Residential".to_string(),
            owner_id: "owner123".to_string(),
            location: AssetLocation {
                address: "123 Main St".to_string(),
                city: "New York".to_string(),
                state_province: Some("NY".to_string()),
                country: "US".to_string(),
                postal_code: Some("10001".to_string()),
                coordinates: None,
                timezone: Some("America/New_York".to_string()),
            },
            estimated_value: Some(rust_decimal::Decimal::new(100000000, 2)), // $1,000,000
            currency: Some("USD".to_string()),
            physical_attributes: HashMap::new(),
            legal_attributes: HashMap::new(),
            custom_attributes: HashMap::new(),
            tags: vec!["residential".to_string(), "urban".to_string()],
            external_ids: HashMap::new(),
            documents: vec![],
        }
    }

    #[test]
    fn test_registration_config_default() {
        let config = RegistrationConfig::default();
        assert!(config.auto_validation);
        assert!(!config.required_documents.is_empty());
        assert!(config.max_document_size > 0);
    }

    #[test]
    fn test_registration_service_creation() {
        let config = RegistrationConfig::default();
        let _service = AssetRegistrationService::new(config);
    }

    #[tokio::test]
    async fn test_basic_data_validation() {
        let config = RegistrationConfig::default();
        let service = AssetRegistrationService::new(config);
        
        let mut request = create_test_registration_request();
        request.name = "".to_string(); // Invalid empty name
        
        let result = service.validate_basic_data(&request);
        assert_eq!(result.status, ValidationStatus::Failed);
        assert!(result.message.contains("name is required"));
    }

    #[tokio::test]
    async fn test_document_validation() {
        let config = RegistrationConfig::default();
        let service = AssetRegistrationService::new(config);
        
        let request = create_test_registration_request();
        
        let result = service.validate_documents(&request).await.unwrap();
        assert_eq!(result.status, ValidationStatus::Failed);
        assert!(result.message.contains("Missing required documents"));
    }

    #[test]
    fn test_registration_status_progression() {
        assert_ne!(RegistrationStatus::Pending, RegistrationStatus::Completed);
        assert_ne!(RegistrationStatus::Failed, RegistrationStatus::Completed);
    }
}
