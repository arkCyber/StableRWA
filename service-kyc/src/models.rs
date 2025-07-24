// =====================================================================================
// RWA Tokenization Platform - KYC/AML Service Models
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// KYC verification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KycStatus {
    NotStarted,
    InProgress,
    PendingReview,
    Approved,
    Rejected,
    Expired,
    Suspended,
}

/// AML risk level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Document types for KYC verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Passport,
    DriverLicense,
    NationalId,
    ProofOfAddress,
    BankStatement,
    UtilityBill,
    TaxDocument,
    BusinessRegistration,
    ArticlesOfIncorporation,
    Other(String),
}

/// Customer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub external_id: String, // Customer ID from main system
    pub customer_type: CustomerType,
    pub kyc_status: KycStatus,
    pub aml_risk_level: RiskLevel,
    pub personal_info: Option<PersonalInfo>,
    pub business_info: Option<BusinessInfo>,
    pub verification_level: VerificationLevel,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Customer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomerType {
    Individual,
    Business,
    Institution,
}

/// Verification levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationLevel {
    Basic,      // Email + Phone
    Standard,   // Basic + ID Document
    Enhanced,   // Standard + Proof of Address
    Premium,    // Enhanced + Additional Checks
}

/// Personal information for individuals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInfo {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub date_of_birth: chrono::NaiveDate,
    pub nationality: String,
    pub country_of_residence: String,
    pub address: Address,
    pub phone: String,
    pub email: String,
    pub occupation: Option<String>,
    pub source_of_funds: Option<String>,
    pub politically_exposed_person: bool,
}

/// Business information for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessInfo {
    pub legal_name: String,
    pub trading_name: Option<String>,
    pub registration_number: String,
    pub tax_id: Option<String>,
    pub incorporation_date: chrono::NaiveDate,
    pub jurisdiction: String,
    pub business_type: String,
    pub industry: String,
    pub address: Address,
    pub website: Option<String>,
    pub beneficial_owners: Vec<BeneficialOwner>,
    pub authorized_representatives: Vec<AuthorizedRepresentative>,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street_address: String,
    pub city: String,
    pub state_province: Option<String>,
    pub postal_code: String,
    pub country: String,
}

/// Beneficial owner information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficialOwner {
    pub name: String,
    pub ownership_percentage: f64,
    pub date_of_birth: chrono::NaiveDate,
    pub nationality: String,
    pub address: Address,
    pub politically_exposed_person: bool,
}

/// Authorized representative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedRepresentative {
    pub name: String,
    pub title: String,
    pub email: String,
    pub phone: String,
    pub authority_level: String,
}

/// KYC document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycDocument {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub document_type: DocumentType,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub status: DocumentStatus,
    pub extracted_data: Option<HashMap<String, serde_json::Value>>,
    pub verification_result: Option<DocumentVerificationResult>,
    pub uploaded_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Document verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentStatus {
    Uploaded,
    Processing,
    Verified,
    Rejected,
    Expired,
}

/// Document verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVerificationResult {
    pub is_authentic: bool,
    pub confidence_score: f64,
    pub extracted_fields: HashMap<String, String>,
    pub verification_checks: Vec<VerificationCheck>,
    pub fraud_indicators: Vec<String>,
}

/// Individual verification check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub check_type: String,
    pub status: CheckStatus,
    pub confidence: f64,
    pub details: Option<String>,
}

/// Check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckStatus {
    Pass,
    Fail,
    Warning,
    NotApplicable,
}

/// AML transaction monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMonitoring {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub transaction_id: String,
    pub transaction_type: String,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub counterparty: Option<String>,
    pub risk_score: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub status: MonitoringStatus,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

/// Risk factors for AML monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: RiskLevel,
    pub description: String,
    pub score: f64,
}

/// Monitoring status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringStatus {
    Clear,
    Flagged,
    UnderReview,
    Escalated,
    Resolved,
}

/// Sanctions screening result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsScreening {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub screening_type: ScreeningType,
    pub query_data: HashMap<String, String>,
    pub matches: Vec<SanctionsMatch>,
    pub risk_level: RiskLevel,
    pub screened_at: DateTime<Utc>,
    pub next_screening_at: DateTime<Utc>,
}

/// Screening types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreeningType {
    Individual,
    Business,
    Transaction,
    Ongoing,
}

/// Sanctions match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanctionsMatch {
    pub list_name: String,
    pub match_type: MatchType,
    pub confidence_score: f64,
    pub matched_fields: Vec<String>,
    pub entity_details: HashMap<String, String>,
}

/// Match types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Fuzzy,
    Phonetic,
    Alias,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: Uuid,
    pub report_type: ReportType,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub data: HashMap<String, serde_json::Value>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub file_path: Option<String>,
}

/// Report types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    SuspiciousActivity,
    CurrencyTransaction,
    KycSummary,
    AmlActivity,
    RegulatoryFiling,
}

/// KYC verification request
#[derive(Debug, Deserialize)]
pub struct KycVerificationRequest {
    pub customer_id: String,
    pub verification_level: VerificationLevel,
    pub personal_info: Option<PersonalInfo>,
    pub business_info: Option<BusinessInfo>,
}

/// Document upload request
#[derive(Debug, Deserialize)]
pub struct DocumentUploadRequest {
    pub customer_id: String,
    pub document_type: DocumentType,
    // File data handled separately in multipart form
}

/// AML screening request
#[derive(Debug, Deserialize)]
pub struct AmlScreeningRequest {
    pub customer_id: String,
    pub screening_type: ScreeningType,
    pub immediate: Option<bool>,
}

/// Transaction monitoring request
#[derive(Debug, Deserialize)]
pub struct TransactionMonitoringRequest {
    pub customer_id: String,
    pub transaction_id: String,
    pub transaction_type: String,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub counterparty: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Default for KycStatus {
    fn default() -> Self {
        KycStatus::NotStarted
    }
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Medium
    }
}

impl Default for VerificationLevel {
    fn default() -> Self {
        VerificationLevel::Basic
    }
}
