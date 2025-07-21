// =====================================================================================
// File: core-compliance/src/types.rs
// Description: Core types and enums for compliance module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

/// Compliance status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    /// Initial state, compliance check not started
    Pending,
    /// Compliance check in progress
    InProgress,
    /// Compliance check completed and approved
    Approved,
    /// Compliance check rejected
    Rejected,
    /// Compliance status expired and needs renewal
    Expired,
    /// Compliance suspended due to issues
    Suspended,
}

/// Compliance level enumeration (ordered by strictness)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// Basic compliance for low-value transactions
    Basic = 1,
    /// Standard compliance for regular users
    Standard = 2,
    /// Enhanced compliance for high-value transactions
    Enhanced = 3,
    /// Premium compliance for institutional users
    Premium = 4,
}

/// Risk level enumeration (ordered by risk)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Risk level not determined
    Unknown = 0,
    /// Low risk user/transaction
    Low = 1,
    /// Medium risk user/transaction
    Medium = 2,
    /// High risk user/transaction
    High = 3,
    /// Critical risk - requires immediate attention
    Critical = 4,
}

/// Jurisdiction codes for regulatory compliance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JurisdictionCode {
    /// United States
    US,
    /// European Union
    EU,
    /// United Kingdom
    UK,
    /// Singapore
    SG,
    /// Hong Kong
    HK,
    /// Japan
    JP,
    /// Canada
    CA,
    /// Australia
    AU,
    /// Switzerland
    CH,
    /// Other jurisdiction
    Other(u16),
}

impl JurisdictionCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            JurisdictionCode::US => "US",
            JurisdictionCode::EU => "EU",
            JurisdictionCode::UK => "UK",
            JurisdictionCode::SG => "SG",
            JurisdictionCode::HK => "HK",
            JurisdictionCode::JP => "JP",
            JurisdictionCode::CA => "CA",
            JurisdictionCode::AU => "AU",
            JurisdictionCode::CH => "CH",
            JurisdictionCode::Other(_) => "OTHER",
        }
    }
}

/// KYC (Know Your Customer) data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycData {
    pub id: Uuid,
    pub user_id: String,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: chrono::NaiveDate,
    pub nationality: String,
    pub address: Address,
    pub identity_document: IdentityDocument,
    pub verification_status: VerificationStatus,
    pub verification_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Identity document information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityDocument {
    pub document_type: DocumentType,
    pub document_number: String,
    pub issuing_country: String,
    pub issue_date: chrono::NaiveDate,
    pub expiry_date: Option<chrono::NaiveDate>,
}

/// Document type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    Passport,
    NationalId,
    DriversLicense,
    Other,
}

/// Verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    NotStarted,
    Pending,
    Verified,
    Failed,
    Expired,
}

/// AML (Anti-Money Laundering) check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlCheck {
    pub id: Uuid,
    pub user_id: String,
    pub check_type: AmlCheckType,
    pub result: AmlResult,
    pub risk_score: f64,
    pub matches: Vec<AmlMatch>,
    pub checked_at: DateTime<Utc>,
    pub provider: String,
}

/// AML check type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmlCheckType {
    Sanctions,
    PoliticallyExposed,
    AdverseMedia,
    Watchlist,
    Enhanced,
}

/// AML check result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmlResult {
    Clear,
    Review,
    Alert,
    Block,
}

/// KYC verification result (re-export for convenience)
pub type KycResult = crate::kyc::KycResult;

/// AML match information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmlMatch {
    pub match_type: AmlCheckType,
    pub confidence: f64,
    pub description: String,
    pub source: String,
}

/// Compliance report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: Uuid,
    pub report_type: ReportType,
    pub jurisdiction: JurisdictionCode,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub generated_at: DateTime<Utc>,
    pub data: serde_json::Value,
    pub status: ReportStatus,
}

/// Report type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// Suspicious Activity Report
    SAR,
    /// Currency Transaction Report
    CTR,
    /// Regulatory compliance summary
    ComplianceSummary,
    /// KYC statistics
    KycStats,
    /// AML monitoring report
    AmlMonitoring,
}

/// Report status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportStatus {
    Draft,
    Generated,
    Submitted,
    Acknowledged,
    Failed,
}

/// Audit event for compliance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub resource_id: Option<String>,
    pub action: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    KycSubmission,
    KycApproval,
    KycRejection,
    AmlCheck,
    ComplianceStatusChange,
    ReportGeneration,
    SystemAccess,
    DataAccess,
    ConfigurationChange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliance_level_ordering() {
        assert!(ComplianceLevel::Basic < ComplianceLevel::Standard);
        assert!(ComplianceLevel::Standard < ComplianceLevel::Enhanced);
        assert!(ComplianceLevel::Enhanced < ComplianceLevel::Premium);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Unknown < RiskLevel::Low);
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_jurisdiction_code_string_conversion() {
        assert_eq!(JurisdictionCode::US.as_str(), "US");
        assert_eq!(JurisdictionCode::EU.as_str(), "EU");
        assert_eq!(JurisdictionCode::Other(999).as_str(), "OTHER");
    }

    #[test]
    fn test_kyc_data_creation() {
        let kyc_data = KycData {
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
        };

        assert_eq!(kyc_data.user_id, "user123");
        assert_eq!(kyc_data.verification_status, VerificationStatus::NotStarted);
    }
}
