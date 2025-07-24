// =====================================================================================
// File: core-asset-lifecycle/src/custody.rs
// Description: Asset custody and safekeeping services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    error::{AssetError, AssetResult},
    types::{CustodyRecord, CustodyStatus, CustodyProvider},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug, warn};
use uuid::Uuid;

// CustodyType is now defined in types.rs to avoid circular dependency
pub use crate::types::CustodyType;

/// Custody configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyConfig {
    pub asset_id: Uuid,
    pub custody_type: CustodyType,
    pub provider_id: Option<String>,
    pub location: Option<String>,
    pub insurance_coverage: Option<f64>,
    pub access_controls: Vec<String>,
    pub backup_locations: Vec<String>,
    pub emergency_contacts: Vec<String>,
}

impl Default for CustodyConfig {
    fn default() -> Self {
        Self {
            asset_id: uuid::Uuid::new_v4(),
            custody_type: CustodyType::SelfCustody,
            provider_id: None,
            location: None,
            insurance_coverage: None,
            access_controls: vec![],
            backup_locations: vec![],
            emergency_contacts: vec![],
        }
    }
}

/// Custody transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransferRequest {
    pub asset_id: Uuid,
    pub from_provider: String,
    pub to_provider: String,
    pub transfer_reason: String,
    pub requested_by: String,
    pub requested_date: DateTime<Utc>,
    pub target_date: Option<DateTime<Utc>>,
    pub special_instructions: Option<String>,
}

/// Custody audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyAudit {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub audit_date: DateTime<Utc>,
    pub auditor: String,
    pub audit_type: AuditType,
    pub findings: Vec<AuditFinding>,
    pub overall_status: AuditStatus,
    pub recommendations: Vec<String>,
    pub next_audit_date: DateTime<Utc>,
}

/// Audit types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditType {
    Routine,
    Compliance,
    Security,
    Insurance,
    Emergency,
}

/// Audit findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub category: String,
    pub severity: FindingSeverity,
    pub description: String,
    pub recommendation: String,
    pub deadline: Option<DateTime<Utc>>,
}

/// Finding severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditStatus {
    Passed,
    PassedWithRecommendations,
    Failed,
    RequiresFollowUp,
}

/// Asset custody service trait
#[async_trait]
pub trait CustodyService: Send + Sync {
    /// Establish custody arrangement for an asset
    async fn establish_custody(
        &self,
        config: CustodyConfig,
    ) -> AssetResult<CustodyRecord>;

    /// Transfer custody between providers
    async fn transfer_custody(
        &self,
        request: CustodyTransferRequest,
    ) -> AssetResult<CustodyRecord>;

    /// Get custody information for an asset
    async fn get_custody_info(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Option<CustodyRecord>>;

    /// Update custody status
    async fn update_custody_status(
        &self,
        asset_id: Uuid,
        status: CustodyStatus,
        notes: Option<String>,
    ) -> AssetResult<CustodyRecord>;

    /// Conduct custody audit
    async fn conduct_audit(
        &self,
        asset_id: Uuid,
        audit_type: AuditType,
        auditor: String,
    ) -> AssetResult<CustodyAudit>;

    /// Get audit history
    async fn get_audit_history(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<CustodyAudit>>;

    /// Verify asset presence
    async fn verify_asset_presence(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<bool>;

    /// Get custody costs
    async fn get_custody_costs(
        &self,
        asset_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> AssetResult<f64>;
}

/// Default custody service implementation
pub struct DefaultCustodyService {
    custody_records: HashMap<Uuid, CustodyRecord>,
    audit_history: HashMap<Uuid, Vec<CustodyAudit>>,
}

impl DefaultCustodyService {
    pub fn new() -> Self {
        Self {
            custody_records: HashMap::new(),
            audit_history: HashMap::new(),
        }
    }

    /// Validate custody configuration
    fn validate_config(&self, config: &CustodyConfig) -> AssetResult<()> {
        match config.custody_type {
            CustodyType::ThirdPartyCustody | CustodyType::InstitutionalCustody => {
                if config.provider_id.is_none() {
                    return Err(AssetError::ValidationError {
                        field: "provider_id".to_string(),
                        message: "Provider ID is required for third-party custody".to_string(),
                    });
                }
            }
            _ => {}
        }

        if let Some(coverage) = config.insurance_coverage {
            if coverage < 0.0 {
                return Err(AssetError::ValidationError {
                    field: "insurance_coverage".to_string(),
                    message: "Insurance coverage cannot be negative".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Generate custody provider based on type
    fn get_custody_provider(&self, custody_type: &CustodyType, provider_id: Option<&String>) -> CustodyProvider {
        match custody_type {
            CustodyType::SelfCustody => CustodyProvider {
                id: "self".to_string(),
                name: "Self Custody".to_string(),
                provider_type: "self".to_string(),
                location: "Owner Controlled".to_string(),
                insurance_coverage: None,
                certifications: vec!["Self-Managed".to_string()],
                contact_info: HashMap::new(),
            },
            CustodyType::ThirdPartyCustody => CustodyProvider {
                id: provider_id.unwrap_or(&"default_custodian".to_string()).clone(),
                name: "Professional Custodian".to_string(),
                provider_type: "third_party".to_string(),
                location: "Secure Facility".to_string(),
                insurance_coverage: Some(10_000_000.0),
                certifications: vec!["ISO 27001".to_string(), "SOC 2".to_string()],
                contact_info: HashMap::new(),
            },
            _ => CustodyProvider {
                id: provider_id.unwrap_or(&"institutional".to_string()).clone(),
                name: "Institutional Custodian".to_string(),
                provider_type: "institutional".to_string(),
                location: "Bank Vault".to_string(),
                insurance_coverage: Some(50_000_000.0),
                certifications: vec!["Banking License".to_string(), "FDIC Insured".to_string()],
                contact_info: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl CustodyService for DefaultCustodyService {
    async fn establish_custody(
        &self,
        config: CustodyConfig,
    ) -> AssetResult<CustodyRecord> {
        info!("Establishing custody for asset {} with type {:?}", config.asset_id, config.custody_type);

        self.validate_config(&config)?;

        let provider = self.get_custody_provider(&config.custody_type, config.provider_id.as_ref());

        let record = CustodyRecord {
            id: Uuid::new_v4(),
            asset_id: config.asset_id,
            provider,
            custody_type: config.custody_type,
            status: CustodyStatus::Active,
            start_date: Utc::now(),
            end_date: None,
            location: config.location.unwrap_or_else(|| "Default Location".to_string()),
            insurance_coverage: config.insurance_coverage,
            access_controls: config.access_controls,
            last_verified: Some(Utc::now()),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        debug!("Created custody record: {:?}", record);
        Ok(record)
    }

    async fn transfer_custody(
        &self,
        request: CustodyTransferRequest,
    ) -> AssetResult<CustodyRecord> {
        info!("Transferring custody for asset {} from {} to {}", 
               request.asset_id, request.from_provider, request.to_provider);

        // Mock transfer process
        let provider = CustodyProvider {
            id: request.to_provider.clone(),
            name: "New Custodian".to_string(),
            provider_type: "third_party".to_string(),
            location: "New Secure Facility".to_string(),
            insurance_coverage: Some(25_000_000.0),
            certifications: vec!["ISO 27001".to_string()],
            contact_info: HashMap::new(),
        };

        let record = CustodyRecord {
            id: Uuid::new_v4(),
            asset_id: request.asset_id,
            provider,
            custody_type: CustodyType::ThirdPartyCustody,
            status: CustodyStatus::InTransfer,
            start_date: request.target_date.unwrap_or_else(|| Utc::now() + chrono::Duration::days(7)),
            end_date: None,
            location: "Transfer in Progress".to_string(),
            insurance_coverage: Some(25_000_000.0),
            access_controls: vec!["Transfer Protocol".to_string()],
            last_verified: None,
            notes: Some(format!("Transfer reason: {}", request.transfer_reason)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        debug!("Created custody transfer record: {:?}", record);
        Ok(record)
    }

    async fn get_custody_info(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Option<CustodyRecord>> {
        debug!("Getting custody info for asset {}", asset_id);
        
        // Mock custody record
        let provider = CustodyProvider {
            id: "secure_vault_inc".to_string(),
            name: "Secure Vault Inc.".to_string(),
            provider_type: "institutional".to_string(),
            location: "New York, NY".to_string(),
            insurance_coverage: Some(100_000_000.0),
            certifications: vec!["ISO 27001".to_string(), "SOC 2 Type II".to_string()],
            contact_info: HashMap::new(),
        };

        Ok(Some(CustodyRecord {
            id: Uuid::new_v4(),
            asset_id,
            provider,
            custody_type: CustodyType::InstitutionalCustody,
            status: CustodyStatus::Active,
            start_date: Utc::now() - chrono::Duration::days(365),
            end_date: None,
            location: "Vault A-123".to_string(),
            insurance_coverage: Some(100_000_000.0),
            access_controls: vec!["Biometric".to_string(), "Multi-Signature".to_string()],
            last_verified: Some(Utc::now() - chrono::Duration::days(30)),
            notes: Some("Annual custody arrangement".to_string()),
            created_at: Utc::now() - chrono::Duration::days(365),
            updated_at: Utc::now() - chrono::Duration::days(30),
        }))
    }

    async fn update_custody_status(
        &self,
        asset_id: Uuid,
        status: CustodyStatus,
        notes: Option<String>,
    ) -> AssetResult<CustodyRecord> {
        info!("Updating custody status for asset {} to {:?}", asset_id, status);

        // Mock updated record
        let provider = CustodyProvider {
            id: "secure_vault_inc".to_string(),
            name: "Secure Vault Inc.".to_string(),
            provider_type: "institutional".to_string(),
            location: "New York, NY".to_string(),
            insurance_coverage: Some(100_000_000.0),
            certifications: vec!["ISO 27001".to_string()],
            contact_info: HashMap::new(),
        };

        Ok(CustodyRecord {
            id: Uuid::new_v4(),
            asset_id,
            provider,
            custody_type: CustodyType::InstitutionalCustody,
            status,
            start_date: Utc::now() - chrono::Duration::days(365),
            end_date: if matches!(status, CustodyStatus::Terminated) {
                Some(Utc::now())
            } else {
                None
            },
            location: "Vault A-123".to_string(),
            insurance_coverage: Some(100_000_000.0),
            access_controls: vec!["Biometric".to_string(), "Multi-Signature".to_string()],
            last_verified: Some(Utc::now()),
            notes,
            created_at: Utc::now() - chrono::Duration::days(365),
            updated_at: Utc::now(),
        })
    }

    async fn conduct_audit(
        &self,
        asset_id: Uuid,
        audit_type: AuditType,
        auditor: String,
    ) -> AssetResult<CustodyAudit> {
        info!("Conducting {:?} audit for asset {} by {}", audit_type, asset_id, auditor);

        let audit = CustodyAudit {
            id: Uuid::new_v4(),
            asset_id,
            audit_date: Utc::now(),
            auditor,
            audit_type,
            findings: vec![
                AuditFinding {
                    category: "Security".to_string(),
                    severity: FindingSeverity::Low,
                    description: "Access logs should be reviewed more frequently".to_string(),
                    recommendation: "Implement weekly access log reviews".to_string(),
                    deadline: Some(Utc::now() + chrono::Duration::days(30)),
                }
            ],
            overall_status: AuditStatus::PassedWithRecommendations,
            recommendations: vec![
                "Enhance monitoring systems".to_string(),
                "Update emergency procedures".to_string(),
            ],
            next_audit_date: Utc::now() + chrono::Duration::days(365),
        };

        debug!("Completed audit: {:?}", audit);
        Ok(audit)
    }

    async fn get_audit_history(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<CustodyAudit>> {
        debug!("Getting audit history for asset {}", asset_id);
        
        // Mock audit history
        Ok(vec![])
    }

    async fn verify_asset_presence(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<bool> {
        info!("Verifying presence of asset {}", asset_id);
        
        // Mock verification - always returns true for demo
        debug!("Asset presence verified successfully");
        Ok(true)
    }

    async fn get_custody_costs(
        &self,
        asset_id: Uuid,
        _period_start: DateTime<Utc>,
        _period_end: DateTime<Utc>,
    ) -> AssetResult<f64> {
        debug!("Calculating custody costs for asset {}", asset_id);
        
        // Mock cost calculation
        Ok(12000.0) // Annual custody fee
    }
}

impl Default for DefaultCustodyService {
    fn default() -> Self {
        Self::new()
    }
}
