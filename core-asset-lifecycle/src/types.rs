// =====================================================================================
// File: core-asset-lifecycle/src/types.rs
// Description: Core types for asset lifecycle management
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Asset status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetStatus {
    /// Asset is being registered
    Registering,
    /// Asset registration is complete
    Registered,
    /// Asset is under verification
    Verifying,
    /// Asset verification is complete
    Verified,
    /// Asset is being valued
    Valuing,
    /// Asset valuation is complete
    Valued,
    /// Asset is ready for tokenization
    ReadyForTokenization,
    /// Asset is being tokenized
    Tokenizing,
    /// Asset is tokenized and active
    Active,
    /// Asset is under maintenance
    UnderMaintenance,
    /// Asset is suspended from trading
    Suspended,
    /// Asset is being liquidated
    Liquidating,
    /// Asset is retired
    Retired,
    /// Asset has failed verification or other processes
    Failed,
}

/// Asset type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    /// Real estate properties
    RealEstate,
    /// Precious metals and commodities
    Commodities,
    /// Artwork and collectibles
    Art,
    /// Intellectual property
    IntellectualProperty,
    /// Infrastructure assets
    Infrastructure,
    /// Equipment and machinery
    Equipment,
    /// Vehicles
    Vehicles,
    /// Energy assets
    Energy,
    /// Agricultural assets
    Agriculture,
    /// Other asset types
    Other,
}

impl AssetType {
    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            AssetType::RealEstate => "Real Estate",
            AssetType::Commodities => "Commodities",
            AssetType::Art => "Art & Collectibles",
            AssetType::IntellectualProperty => "Intellectual Property",
            AssetType::Infrastructure => "Infrastructure",
            AssetType::Equipment => "Equipment & Machinery",
            AssetType::Vehicles => "Vehicles",
            AssetType::Energy => "Energy Assets",
            AssetType::Agriculture => "Agricultural Assets",
            AssetType::Other => "Other",
        }
    }

    /// Get typical valuation methods for this asset type
    pub fn typical_valuation_methods(&self) -> Vec<crate::valuation::ValuationMethod> {
        match self {
            AssetType::RealEstate => vec![
                crate::valuation::ValuationMethod::ComparablesSales,
                crate::valuation::ValuationMethod::IncomeApproach,
                crate::valuation::ValuationMethod::CostApproach,
            ],
            AssetType::Commodities => vec![
                crate::valuation::ValuationMethod::MarketPrice,
                crate::valuation::ValuationMethod::CostApproach,
            ],
            AssetType::Art => vec![
                crate::valuation::ValuationMethod::ComparablesSales,
                crate::valuation::ValuationMethod::ExpertAppraisal,
            ],
            _ => vec![
                crate::valuation::ValuationMethod::CostApproach,
                crate::valuation::ValuationMethod::ExpertAppraisal,
            ],
        }
    }
}

/// Asset category for more granular classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCategory {
    pub primary: AssetType,
    pub subcategory: String,
    pub classification_code: Option<String>,
}

/// Main asset structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub category: AssetCategory,
    pub status: AssetStatus,
    pub owner_id: String,
    pub custodian_id: Option<String>,
    pub location: AssetLocation,
    pub metadata: AssetMetadata,
    pub documents: Vec<AssetDocument>,
    pub valuations: Vec<AssetValuation>,
    pub maintenance_records: Vec<AssetMaintenance>,
    pub ownership_history: Vec<AssetOwnership>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub lifecycle_stage: crate::LifecycleStage,
}

impl Asset {
    /// Create a new asset
    pub fn new(
        name: String,
        description: String,
        asset_type: AssetType,
        owner_id: String,
        location: AssetLocation,
    ) -> Self {
        let id = Uuid::new_v4();
        let now = Utc::now();

        Self {
            id,
            name,
            description,
            asset_type,
            category: AssetCategory {
                primary: asset_type,
                subcategory: "General".to_string(),
                classification_code: None,
            },
            status: AssetStatus::Registering,
            owner_id: owner_id.clone(),
            custodian_id: None,
            location,
            metadata: AssetMetadata::default(),
            documents: Vec::new(),
            valuations: Vec::new(),
            maintenance_records: Vec::new(),
            ownership_history: vec![AssetOwnership {
                id: Uuid::new_v4(),
                asset_id: id,
                owner_id,
                ownership_percentage: Decimal::new(100, 0), // 100%
                acquired_at: now,
                acquisition_price: None,
                ownership_type: OwnershipType::Full,
                legal_documents: Vec::new(),
            }],
            created_at: now,
            updated_at: now,
            lifecycle_stage: crate::LifecycleStage::Registration,
        }
    }

    /// Get current valuation
    pub fn current_valuation(&self) -> Option<&AssetValuation> {
        self.valuations
            .iter()
            .max_by_key(|v| v.valuation_date)
    }

    /// Get latest maintenance record
    pub fn latest_maintenance(&self) -> Option<&AssetMaintenance> {
        self.maintenance_records
            .iter()
            .max_by_key(|m| m.scheduled_date)
    }

    /// Check if asset needs maintenance
    pub fn needs_maintenance(&self) -> bool {
        if let Some(latest) = self.latest_maintenance() {
            if let Some(next_due) = latest.next_maintenance_due {
                return Utc::now() >= next_due;
            }
        }
        false
    }

    /// Get current owner
    pub fn current_owner(&self) -> Option<&AssetOwnership> {
        self.ownership_history
            .iter()
            .max_by_key(|o| o.acquired_at)
    }
}

/// Asset location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLocation {
    pub address: String,
    pub city: String,
    pub state_province: Option<String>,
    pub country: String,
    pub postal_code: Option<String>,
    pub coordinates: Option<Coordinates>,
    pub timezone: Option<String>,
}

/// Geographic coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub physical_attributes: HashMap<String, String>,
    pub legal_attributes: HashMap<String, String>,
    pub financial_attributes: HashMap<String, String>,
    pub custom_attributes: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub external_ids: HashMap<String, String>,
}

impl Default for AssetMetadata {
    fn default() -> Self {
        Self {
            physical_attributes: HashMap::new(),
            legal_attributes: HashMap::new(),
            financial_attributes: HashMap::new(),
            custom_attributes: HashMap::new(),
            tags: Vec::new(),
            external_ids: HashMap::new(),
        }
    }
}

/// Asset document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDocument {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub document_type: DocumentType,
    pub title: String,
    pub description: Option<String>,
    pub file_path: String,
    pub file_hash: String,
    pub file_size: u64,
    pub mime_type: String,
    pub uploaded_by: String,
    pub uploaded_at: DateTime<Utc>,
    pub verified: bool,
    pub verification_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Document type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// Legal ownership documents
    OwnershipDocument,
    /// Property deeds
    Deed,
    /// Insurance policies
    Insurance,
    /// Appraisal reports
    Appraisal,
    /// Inspection reports
    Inspection,
    /// Maintenance records
    MaintenanceRecord,
    /// Photos and images
    Photo,
    /// Certificates
    Certificate,
    /// Contracts
    Contract,
    /// Financial statements
    FinancialStatement,
    /// Compliance documents
    Compliance,
    /// Other documents
    Other,
}

/// Asset valuation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValuation {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub valuation_method: crate::valuation::ValuationMethod,
    pub valuation_amount: Decimal,
    pub currency: String,
    pub valuation_date: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub appraiser_id: String,
    pub appraiser_credentials: String,
    pub confidence_level: f64,
    pub methodology_notes: String,
    pub supporting_documents: Vec<Uuid>,
    pub market_conditions: Option<String>,
    pub assumptions: Vec<String>,
    pub limitations: Vec<String>,
}

/// Asset maintenance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMaintenance {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub maintenance_type: MaintenanceType,
    pub description: String,
    pub scheduled_date: DateTime<Utc>,
    pub completed_date: Option<DateTime<Utc>>,
    pub performed_by: Option<String>,
    pub cost: Option<Decimal>,
    pub currency: Option<String>,
    pub status: MaintenanceStatus,
    pub notes: Option<String>,
    pub next_maintenance_due: Option<DateTime<Utc>>,
    pub supporting_documents: Vec<Uuid>,
}

/// Maintenance type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenanceType {
    /// Routine preventive maintenance
    Preventive,
    /// Corrective maintenance to fix issues
    Corrective,
    /// Emergency repairs
    Emergency,
    /// Upgrades and improvements
    Upgrade,
    /// Inspections
    Inspection,
    /// Cleaning and upkeep
    Cleaning,
    /// Other maintenance types
    Other,
}

/// Maintenance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenanceStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
    Overdue,
}

/// Maintenance record structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRecord {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub maintenance_type: MaintenanceType,
    pub status: MaintenanceStatus,
    pub scheduled_date: DateTime<Utc>,
    pub completed_date: Option<DateTime<Utc>>,
    pub description: String,
    pub performed_by: Option<String>,
    pub cost: Option<rust_decimal::Decimal>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tokenization status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenizationStatus {
    /// Not yet tokenized
    NotTokenized,
    /// Tokenization in progress
    InProgress,
    /// Successfully deployed
    Deployed,
    /// Tokenization failed
    Failed,
    /// Tokens burned/retired
    Burned,
}

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub name: String,
    pub description: String,
    pub image: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Vec<TokenAttribute>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Token attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAttribute {
    pub trait_type: String,
    pub value: serde_json::Value,
    pub display_type: Option<String>,
}

/// Custody status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyStatus {
    /// Asset is in active custody
    Active,
    /// Custody is being transferred
    InTransfer,
    /// Custody is suspended
    Suspended,
    /// Custody is terminated
    Terminated,
    /// Custody is under review
    UnderReview,
}

/// Custody provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyProvider {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub location: String,
    pub insurance_coverage: Option<f64>,
    pub certifications: Vec<String>,
    pub contact_info: HashMap<String, String>,
}

/// Custody type enumeration (to avoid circular dependency)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyType {
    SelfCustody,
    ThirdPartyCustody,
    MultiSigCustody,
    InstitutionalCustody,
    EscrowCustody,
}

/// Custody record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyRecord {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub provider: CustodyProvider,
    pub custody_type: CustodyType,
    pub status: CustodyStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub location: String,
    pub insurance_coverage: Option<f64>,
    pub access_controls: Vec<String>,
    pub last_verified: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset lifecycle stage enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleStage {
    /// Asset registration stage
    Registration,
    /// Asset verification stage
    Verification,
    /// Asset valuation stage
    Valuation,
    /// Asset is active and operational
    Active,
    /// Asset maintenance stage
    Maintenance,
    /// Asset disposal stage
    Disposal,
    /// Asset is retired
    Retired,
}

/// Asset lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLifecycleEvent {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub event_type: String,
    pub lifecycle_stage: LifecycleStage,
    pub description: String,
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub triggered_by: String,
}

/// Asset ownership record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetOwnership {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub owner_id: String,
    pub ownership_percentage: Decimal,
    pub acquired_at: DateTime<Utc>,
    pub acquisition_price: Option<Decimal>,
    pub ownership_type: OwnershipType,
    pub legal_documents: Vec<Uuid>,
}

/// Ownership type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipType {
    /// Full ownership
    Full,
    /// Partial ownership
    Partial,
    /// Beneficial ownership
    Beneficial,
    /// Legal ownership (trustee)
    Legal,
    /// Leasehold
    Leasehold,
    /// Other ownership types
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_creation() {
        let location = AssetLocation {
            address: "123 Main St".to_string(),
            city: "New York".to_string(),
            state_province: Some("NY".to_string()),
            country: "US".to_string(),
            postal_code: Some("10001".to_string()),
            coordinates: None,
            timezone: Some("America/New_York".to_string()),
        };

        let asset = Asset::new(
            "Test Property".to_string(),
            "A test real estate property".to_string(),
            AssetType::RealEstate,
            "owner123".to_string(),
            location,
        );

        assert_eq!(asset.name, "Test Property");
        assert_eq!(asset.asset_type, AssetType::RealEstate);
        assert_eq!(asset.status, AssetStatus::Registering);
        assert_eq!(asset.lifecycle_stage, crate::LifecycleStage::Registration);
        assert_eq!(asset.ownership_history.len(), 1);
    }

    #[test]
    fn test_asset_type_display_names() {
        assert_eq!(AssetType::RealEstate.display_name(), "Real Estate");
        assert_eq!(AssetType::Art.display_name(), "Art & Collectibles");
        assert_eq!(AssetType::IntellectualProperty.display_name(), "Intellectual Property");
    }

    #[test]
    fn test_asset_type_valuation_methods() {
        let real_estate_methods = AssetType::RealEstate.typical_valuation_methods();
        assert!(!real_estate_methods.is_empty());

        let commodities_methods = AssetType::Commodities.typical_valuation_methods();
        assert!(!commodities_methods.is_empty());
    }

    #[test]
    fn test_asset_metadata_default() {
        let metadata = AssetMetadata::default();
        assert!(metadata.physical_attributes.is_empty());
        assert!(metadata.tags.is_empty());
        assert!(metadata.external_ids.is_empty());
    }

    #[test]
    fn test_asset_document_creation() {
        let document = AssetDocument {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            document_type: DocumentType::OwnershipDocument,
            title: "Property Deed".to_string(),
            description: Some("Legal ownership document".to_string()),
            file_path: "/documents/deed.pdf".to_string(),
            file_hash: "abc123".to_string(),
            file_size: 1024,
            mime_type: "application/pdf".to_string(),
            uploaded_by: "user123".to_string(),
            uploaded_at: Utc::now(),
            verified: false,
            verification_date: None,
            expiry_date: None,
            metadata: HashMap::new(),
        };

        assert_eq!(document.document_type, DocumentType::OwnershipDocument);
        assert_eq!(document.title, "Property Deed");
        assert!(!document.verified);
    }

    #[test]
    fn test_asset_valuation_creation() {
        let valuation = AssetValuation {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            valuation_method: crate::valuation::ValuationMethod::ExpertAppraisal,
            valuation_amount: Decimal::new(100000000, 2), // $1,000,000.00
            currency: "USD".to_string(),
            valuation_date: Utc::now(),
            valid_until: Some(Utc::now() + chrono::Duration::days(365)),
            appraiser_id: "appraiser123".to_string(),
            appraiser_credentials: "Certified Real Estate Appraiser".to_string(),
            confidence_level: 0.95,
            methodology_notes: "Comparable sales analysis".to_string(),
            supporting_documents: vec![],
            market_conditions: Some("Stable market".to_string()),
            assumptions: vec!["Property in good condition".to_string()],
            limitations: vec!["Subject to market changes".to_string()],
        };

        assert_eq!(valuation.valuation_amount, Decimal::new(100000000, 2));
        assert_eq!(valuation.currency, "USD");
        assert_eq!(valuation.confidence_level, 0.95);
    }

    #[test]
    fn test_maintenance_record_creation() {
        let maintenance = AssetMaintenance {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            maintenance_type: MaintenanceType::Preventive,
            description: "Annual HVAC maintenance".to_string(),
            scheduled_date: Utc::now() + chrono::Duration::days(30),
            completed_date: None,
            performed_by: None,
            cost: Some(Decimal::new(50000, 2)), // $500.00
            currency: Some("USD".to_string()),
            status: MaintenanceStatus::Scheduled,
            notes: None,
            next_maintenance_due: Some(Utc::now() + chrono::Duration::days(395)),
            supporting_documents: vec![],
        };

        assert_eq!(maintenance.maintenance_type, MaintenanceType::Preventive);
        assert_eq!(maintenance.status, MaintenanceStatus::Scheduled);
        assert_eq!(maintenance.cost, Some(Decimal::new(50000, 2)));
    }
}
