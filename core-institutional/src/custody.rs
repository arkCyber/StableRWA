// =====================================================================================
// File: core-institutional/src/custody.rs
// Description: Institutional custody service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{InstitutionalError, InstitutionalResult},
    types::{AccountType, InstitutionalAccount, TransactionStatus},
};

/// Custody service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyConfig {
    /// Multi-signature threshold
    pub multisig_threshold: u32,
    /// Required signers for high-value transactions
    pub high_value_signers: u32,
    /// High-value transaction threshold
    pub high_value_threshold: Decimal,
    /// Cold storage percentage
    pub cold_storage_percentage: Decimal,
    /// Hot wallet maximum balance
    pub hot_wallet_max_balance: Decimal,
    /// Insurance coverage amount
    pub insurance_coverage: Decimal,
    /// Enable hardware security modules
    pub enable_hsm: bool,
    /// Audit trail retention days
    pub audit_retention_days: u32,
    /// Enable real-time monitoring
    pub enable_monitoring: bool,
}

impl Default for CustodyConfig {
    fn default() -> Self {
        Self {
            multisig_threshold: 3,
            high_value_signers: 5,
            high_value_threshold: Decimal::new(100000000, 2), // $1,000,000
            cold_storage_percentage: Decimal::new(95, 2),     // 95%
            hot_wallet_max_balance: Decimal::new(500000000, 2), // $5,000,000
            insurance_coverage: Decimal::new(10000000000, 2), // $100,000,000
            enable_hsm: true,
            audit_retention_days: 2555, // 7 years
            enable_monitoring: true,
        }
    }
}

/// Custody account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyAccount {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub account_number: String,
    pub account_name: String,
    pub account_type: CustodyAccountType,
    pub base_currency: String,
    pub custodian: String,
    pub sub_custodians: Vec<String>,
    pub segregation_type: SegregationType,
    pub insurance_policy: Option<InsurancePolicy>,
    pub authorized_signers: Vec<AuthorizedSigner>,
    pub balances: HashMap<String, AssetBalance>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Custody account type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyAccountType {
    Segregated,
    Omnibus,
    Prime,
    Clearing,
}

/// Asset segregation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegregationType {
    FullySegregated,
    PartiallySegregated,
    Omnibus,
}

/// Insurance policy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsurancePolicy {
    pub policy_number: String,
    pub insurer: String,
    pub coverage_amount: Decimal,
    pub deductible: Decimal,
    pub policy_type: InsurancePolicyType,
    pub effective_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub covered_assets: Vec<String>,
}

/// Insurance policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsurancePolicyType {
    CrimeInsurance,
    CyberInsurance,
    ProfessionalIndemnity,
    DirectorsAndOfficers,
    Comprehensive,
}

/// Authorized signer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedSigner {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub email: String,
    pub public_key: String,
    pub signing_authority: SigningAuthority,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Signing authority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningAuthority {
    Limited,   // Up to $100K
    Standard,  // Up to $1M
    Senior,    // Up to $10M
    Executive, // Unlimited
}

impl SigningAuthority {
    /// Get maximum signing amount
    pub fn max_amount(&self) -> Option<Decimal> {
        match self {
            SigningAuthority::Limited => Some(Decimal::new(10000000, 2)), // $100K
            SigningAuthority::Standard => Some(Decimal::new(100000000, 2)), // $1M
            SigningAuthority::Senior => Some(Decimal::new(1000000000, 2)), // $10M
            SigningAuthority::Executive => None,                          // Unlimited
        }
    }
}

/// Asset balance in custody
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    pub asset: String,
    pub total_balance: Decimal,
    pub available_balance: Decimal,
    pub pending_balance: Decimal,
    pub blocked_balance: Decimal,
    pub cold_storage_balance: Decimal,
    pub hot_wallet_balance: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Custody transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransactionRequest {
    pub id: Uuid,
    pub account_id: Uuid,
    pub transaction_type: CustodyTransactionType,
    pub asset: String,
    pub amount: Decimal,
    pub destination: Option<String>,
    pub purpose: String,
    pub requested_by: Uuid,
    pub approvers: Vec<Uuid>,
    pub required_signatures: u32,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Custody transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyTransactionType {
    Deposit,
    Withdrawal,
    InternalTransfer,
    ColdStorageMove,
    HotWalletMove,
    Rebalancing,
}

/// Custody transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyTransactionResult {
    pub request_id: Uuid,
    pub transaction_id: Option<String>,
    pub status: TransactionStatus,
    pub signatures_collected: u32,
    pub signatures_required: u32,
    pub approvals: Vec<TransactionApproval>,
    pub executed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Transaction approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionApproval {
    pub signer_id: Uuid,
    pub signature: String,
    pub approved_at: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
}

/// Custody service trait
#[async_trait]
pub trait CustodyService: Send + Sync {
    /// Create a new custody account
    async fn create_account(&self, account: CustodyAccount) -> InstitutionalResult<CustodyAccount>;

    /// Get custody account by ID
    async fn get_account(&self, account_id: Uuid) -> InstitutionalResult<Option<CustodyAccount>>;

    /// Get all accounts for an institution
    async fn get_institution_accounts(
        &self,
        institution_id: Uuid,
    ) -> InstitutionalResult<Vec<CustodyAccount>>;

    /// Update account information
    async fn update_account(&self, account: CustodyAccount) -> InstitutionalResult<CustodyAccount>;

    /// Get asset balances for an account
    async fn get_balances(
        &self,
        account_id: Uuid,
    ) -> InstitutionalResult<HashMap<String, AssetBalance>>;

    /// Submit custody transaction request
    async fn submit_transaction(
        &self,
        request: CustodyTransactionRequest,
    ) -> InstitutionalResult<CustodyTransactionResult>;

    /// Approve custody transaction
    async fn approve_transaction(
        &self,
        request_id: Uuid,
        signer_id: Uuid,
        signature: String,
    ) -> InstitutionalResult<CustodyTransactionResult>;

    /// Get transaction status
    async fn get_transaction_status(
        &self,
        request_id: Uuid,
    ) -> InstitutionalResult<Option<CustodyTransactionResult>>;

    /// Get transaction history
    async fn get_transaction_history(
        &self,
        account_id: Uuid,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> InstitutionalResult<Vec<CustodyTransactionResult>>;

    /// Add authorized signer
    async fn add_authorized_signer(
        &self,
        account_id: Uuid,
        signer: AuthorizedSigner,
    ) -> InstitutionalResult<()>;

    /// Remove authorized signer
    async fn remove_authorized_signer(
        &self,
        account_id: Uuid,
        signer_id: Uuid,
    ) -> InstitutionalResult<()>;

    /// Generate custody report
    async fn generate_custody_report(
        &self,
        account_id: Uuid,
        report_type: CustodyReportType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> InstitutionalResult<CustodyReport>;

    /// Health check
    async fn health_check(&self) -> InstitutionalResult<CustodyHealthStatus>;
}

/// Custody report type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyReportType {
    PositionReport,
    TransactionReport,
    ReconciliationReport,
    AuditReport,
    ComplianceReport,
}

/// Custody report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyReport {
    pub id: Uuid,
    pub account_id: Uuid,
    pub report_type: CustodyReportType,
    pub report_data: serde_json::Value,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Uuid,
    pub file_url: Option<String>,
}

/// Custody health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyHealthStatus {
    pub status: String,
    pub total_accounts: u64,
    pub active_accounts: u64,
    pub total_assets_under_custody: Decimal,
    pub cold_storage_percentage: Decimal,
    pub pending_transactions: u64,
    pub failed_transactions_24h: u64,
    pub insurance_coverage_ratio: Decimal,
    pub last_reconciliation: DateTime<Utc>,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_authority_limits() {
        assert_eq!(
            SigningAuthority::Limited.max_amount(),
            Some(Decimal::new(10000000, 2))
        );
        assert_eq!(
            SigningAuthority::Standard.max_amount(),
            Some(Decimal::new(100000000, 2))
        );
        assert_eq!(
            SigningAuthority::Senior.max_amount(),
            Some(Decimal::new(1000000000, 2))
        );
        assert_eq!(SigningAuthority::Executive.max_amount(), None);
    }

    #[test]
    fn test_custody_account_creation() {
        let account = CustodyAccount {
            id: Uuid::new_v4(),
            institution_id: Uuid::new_v4(),
            account_number: "CUST001".to_string(),
            account_name: "Test Custody Account".to_string(),
            account_type: CustodyAccountType::Segregated,
            base_currency: "USD".to_string(),
            custodian: "Prime Custodian Inc.".to_string(),
            sub_custodians: vec!["Sub Custodian A".to_string()],
            segregation_type: SegregationType::FullySegregated,
            insurance_policy: None,
            authorized_signers: vec![],
            balances: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        };

        assert_eq!(account.account_type, CustodyAccountType::Segregated);
        assert_eq!(account.segregation_type, SegregationType::FullySegregated);
        assert!(account.is_active);
    }

    #[test]
    fn test_custody_config_default() {
        let config = CustodyConfig::default();
        assert_eq!(config.multisig_threshold, 3);
        assert_eq!(config.cold_storage_percentage, Decimal::new(95, 2));
        assert!(config.enable_hsm);
        assert!(config.enable_monitoring);
    }
}
