// =====================================================================================
// File: core-institutional/src/types.rs
// Description: Common types for institutional services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Institution type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstitutionType {
    Bank,
    HedgeFund,
    AssetManager,
    InsuranceCompany,
    PensionFund,
    SovereignWealthFund,
    FamilyOffice,
    CorporateTreasury,
    Broker,
    Exchange,
    Custodian,
    Other,
}

/// Institution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Institution {
    pub id: Uuid,
    pub name: String,
    pub institution_type: InstitutionType,
    pub legal_entity_identifier: Option<String>, // LEI
    pub jurisdiction: String,
    pub regulatory_licenses: Vec<String>,
    pub contact_info: ContactInfo,
    pub compliance_tier: ComplianceTier,
    pub risk_rating: RiskRating,
    pub aum: Option<Decimal>, // Assets Under Management
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub primary_contact: Contact,
    pub compliance_contact: Option<Contact>,
    pub technical_contact: Option<Contact>,
    pub address: Address,
}

/// Contact details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub title: String,
    pub email: String,
    pub phone: String,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country: String,
}

/// Compliance tier enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceTier {
    Basic,
    Enhanced,
    Premium,
    Enterprise,
}

/// Risk rating enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskRating {
    Low,
    Medium,
    High,
    Critical,
}

/// Account type for institutional accounts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Custody,
    Trading,
    Settlement,
    Omnibus,
    Segregated,
}

/// Institutional account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalAccount {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub account_number: String,
    pub account_type: AccountType,
    pub account_name: String,
    pub base_currency: String,
    pub balances: HashMap<String, Decimal>,
    pub available_balances: HashMap<String, Decimal>,
    pub blocked_balances: HashMap<String, Decimal>,
    pub credit_limit: Option<Decimal>,
    pub margin_requirements: Option<MarginRequirements>,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Margin requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRequirements {
    pub initial_margin: Decimal,
    pub maintenance_margin: Decimal,
    pub variation_margin: Decimal,
    pub margin_call_threshold: Decimal,
}

/// Permission enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ViewBalances,
    ViewTransactions,
    ViewReports,
    PlaceOrders,
    CancelOrders,
    WithdrawFunds,
    DepositFunds,
    ManageUsers,
    ViewCompliance,
    ManageCompliance,
    AdminAccess,
}

/// User role in institutional context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstitutionalRole {
    Viewer,
    Trader,
    PortfolioManager,
    RiskManager,
    ComplianceOfficer,
    Administrator,
    SuperAdmin,
}

/// Institutional user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalUser {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub employee_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: InstitutionalRole,
    pub permissions: Vec<Permission>,
    pub department: String,
    pub manager_id: Option<Uuid>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transaction type for institutional operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Trade,
    Settlement,
    Transfer,
    Fee,
    Interest,
    Dividend,
    CorporateAction,
    Adjustment,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Rejected,
}

/// Institutional transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalTransaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub transaction_type: TransactionType,
    pub asset: String,
    pub amount: Decimal,
    pub price: Option<Decimal>,
    pub fee: Decimal,
    pub status: TransactionStatus,
    pub reference_id: Option<String>,
    pub counterparty: Option<String>,
    pub settlement_date: Option<DateTime<Utc>>,
    pub trade_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Reporting frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportingFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    OnDemand,
}

/// Report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    PositionReport,
    TransactionReport,
    PerformanceReport,
    RiskReport,
    ComplianceReport,
    SettlementReport,
    FeeReport,
    TaxReport,
}

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    JSON,
    CSV,
    Excel,
    PDF,
    XML,
}

/// Report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub report_type: ReportType,
    pub report_name: String,
    pub frequency: ReportingFrequency,
    pub format: ReportFormat,
    pub recipients: Vec<String>,
    pub filters: HashMap<String, serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub key_name: String,
    pub key_hash: String,
    pub permissions: Vec<Permission>,
    pub rate_limit: Option<u32>,
    pub ip_whitelist: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Service tier for institutional clients
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceTier {
    Basic,
    Professional,
    Enterprise,
    Premium,
}

/// SLA (Service Level Agreement) metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAMetrics {
    pub service_tier: ServiceTier,
    pub uptime_guarantee: Decimal, // Percentage
    pub response_time_ms: u64,
    pub throughput_tps: u32, // Transactions per second
    pub support_response_hours: u32,
    pub data_retention_days: u32,
    pub backup_frequency_hours: u32,
}

/// White label configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelConfig {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub brand_name: String,
    pub domain: String,
    pub logo_url: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub custom_css: Option<String>,
    pub custom_terms_url: Option<String>,
    pub custom_privacy_url: Option<String>,
    pub features_enabled: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Institution {
    /// Check if institution has specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        // This would be implemented based on the institution's compliance tier
        // and specific agreements
        match self.compliance_tier {
            ComplianceTier::Basic => matches!(
                permission,
                Permission::ViewBalances | Permission::ViewTransactions
            ),
            ComplianceTier::Enhanced => !matches!(
                permission,
                Permission::AdminAccess | Permission::ManageCompliance
            ),
            ComplianceTier::Premium | ComplianceTier::Enterprise => true,
        }
    }

    /// Get maximum transaction amount based on compliance tier
    pub fn max_transaction_amount(&self) -> Decimal {
        match self.compliance_tier {
            ComplianceTier::Basic => Decimal::new(100000, 2), // $1,000
            ComplianceTier::Enhanced => Decimal::new(10000000, 2), // $100,000
            ComplianceTier::Premium => Decimal::new(1000000000, 2), // $10,000,000
            ComplianceTier::Enterprise => Decimal::new(100000000000, 2), // $1,000,000,000
        }
    }
}

impl InstitutionalUser {
    /// Check if user has specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Get user's full name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_institution_permissions() {
        let institution = Institution {
            id: Uuid::new_v4(),
            name: "Test Bank".to_string(),
            institution_type: InstitutionType::Bank,
            legal_entity_identifier: Some("123456789012345678".to_string()),
            jurisdiction: "US".to_string(),
            regulatory_licenses: vec!["FDIC".to_string()],
            contact_info: ContactInfo {
                primary_contact: Contact {
                    name: "John Doe".to_string(),
                    title: "CEO".to_string(),
                    email: "john@testbank.com".to_string(),
                    phone: "+1-555-0123".to_string(),
                },
                compliance_contact: None,
                technical_contact: None,
                address: Address {
                    street: "123 Main St".to_string(),
                    city: "New York".to_string(),
                    state_province: "NY".to_string(),
                    postal_code: "10001".to_string(),
                    country: "US".to_string(),
                },
            },
            compliance_tier: ComplianceTier::Enhanced,
            risk_rating: RiskRating::Low,
            aum: Some(Decimal::new(100000000000, 2)), // $1B
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        };

        assert!(institution.has_permission(Permission::ViewBalances));
        assert!(institution.has_permission(Permission::PlaceOrders));
        assert!(!institution.has_permission(Permission::AdminAccess));

        assert_eq!(
            institution.max_transaction_amount(),
            Decimal::new(10000000, 2)
        );
    }

    #[test]
    fn test_institutional_user() {
        let user = InstitutionalUser {
            id: Uuid::new_v4(),
            institution_id: Uuid::new_v4(),
            employee_id: "EMP001".to_string(),
            email: "trader@testbank.com".to_string(),
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            role: InstitutionalRole::Trader,
            permissions: vec![Permission::ViewBalances, Permission::PlaceOrders],
            department: "Trading".to_string(),
            manager_id: None,
            is_active: true,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(user.full_name(), "Jane Smith");
        assert!(user.has_permission(Permission::PlaceOrders));
        assert!(!user.has_permission(Permission::AdminAccess));
    }
}
