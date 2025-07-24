// =====================================================================================
// RWA Tokenization Platform - Custody Service Types
// 
// Core type definitions for custody operations including digital assets, physical assets,
// custody proofs, insurance policies, and multi-signature configurations.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Asset types supported by the custody service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    /// Digital assets (cryptocurrencies, tokens, NFTs)
    Digital {
        /// Blockchain network (e.g., "ethereum", "bitcoin", "solana")
        network: String,
        /// Token contract address (if applicable)
        contract_address: Option<String>,
        /// Token standard (e.g., "ERC20", "ERC721", "SPL")
        standard: Option<String>,
    },
    /// Physical assets (real estate, commodities, art, etc.)
    Physical {
        /// Asset category (e.g., "real_estate", "precious_metals", "art")
        category: String,
        /// Physical location or storage facility
        location: String,
        /// Asset condition assessment
        condition: AssetCondition,
    },
}

/// Physical asset condition assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetCondition {
    /// Excellent condition
    Excellent,
    /// Good condition with minor wear
    Good,
    /// Fair condition with noticeable wear
    Fair,
    /// Poor condition requiring attention
    Poor,
    /// Damaged and requiring repair
    Damaged,
}

/// Custody account information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodyAccount {
    /// Unique account identifier
    pub id: Uuid,
    /// Account owner identifier
    pub owner_id: String,
    /// Account type and permissions
    pub account_type: AccountType,
    /// Account status
    pub status: AccountStatus,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Account metadata
    pub metadata: HashMap<String, String>,
}

/// Account types with different permission levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    /// Individual retail account
    Individual,
    /// Corporate account
    Corporate,
    /// Institutional account with enhanced features
    Institutional,
    /// Service account for automated operations
    Service,
}

/// Account status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    /// Account is active and operational
    Active,
    /// Account is temporarily suspended
    Suspended,
    /// Account is frozen due to security concerns
    Frozen,
    /// Account is closed and no longer operational
    Closed,
    /// Account is pending activation
    Pending,
}

/// Custodied asset information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodiedAsset {
    /// Unique asset identifier
    pub id: Uuid,
    /// Account that owns this asset
    pub account_id: Uuid,
    /// Asset type and details
    pub asset_type: AssetType,
    /// Asset quantity or amount
    pub quantity: Decimal,
    /// Asset valuation in USD
    pub valuation_usd: Option<Decimal>,
    /// Custody status
    pub status: CustodyStatus,
    /// Custody provider information
    pub provider: CustodyProvider,
    /// Asset custody start date
    pub custody_start: DateTime<Utc>,
    /// Asset custody end date (if applicable)
    pub custody_end: Option<DateTime<Utc>>,
    /// Insurance policy information
    pub insurance: Option<InsurancePolicy>,
    /// Asset metadata and additional information
    pub metadata: HashMap<String, String>,
}

/// Custody status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyStatus {
    /// Asset is securely held in custody
    Held,
    /// Asset is in transit to custody
    InTransit,
    /// Asset is being transferred out of custody
    Transferring,
    /// Asset custody is temporarily suspended
    Suspended,
    /// Asset has been released from custody
    Released,
}

/// Custody provider information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodyProvider {
    /// Provider unique identifier
    pub id: String,
    /// Provider name
    pub name: String,
    /// Provider type
    pub provider_type: ProviderType,
    /// Provider contact information
    pub contact_info: ContactInfo,
    /// Provider certifications and licenses
    pub certifications: Vec<String>,
    /// Provider reputation score (0.0 to 1.0)
    pub reputation_score: Decimal,
}

/// Types of custody providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    /// Internal custody managed by the platform
    Internal,
    /// Third-party digital asset custodian
    DigitalCustodian,
    /// Physical asset storage facility
    PhysicalStorage,
    /// Bank or financial institution
    Bank,
    /// Specialized custody service
    Specialized,
}

/// Contact information for custody providers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactInfo {
    /// Primary contact email
    pub email: String,
    /// Contact phone number
    pub phone: Option<String>,
    /// Physical address
    pub address: Option<String>,
    /// Website URL
    pub website: Option<String>,
    /// Emergency contact information
    pub emergency_contact: Option<String>,
}

/// Insurance policy information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsurancePolicy {
    /// Policy unique identifier
    pub id: String,
    /// Insurance provider name
    pub provider: String,
    /// Policy number
    pub policy_number: String,
    /// Coverage amount in USD
    pub coverage_amount: Decimal,
    /// Policy effective date
    pub effective_date: DateTime<Utc>,
    /// Policy expiration date
    pub expiration_date: DateTime<Utc>,
    /// Policy status
    pub status: PolicyStatus,
    /// Premium amount
    pub premium: Decimal,
    /// Deductible amount
    pub deductible: Decimal,
    /// Coverage details and terms
    pub coverage_details: HashMap<String, String>,
}

/// Insurance policy status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyStatus {
    /// Policy is active and providing coverage
    Active,
    /// Policy is pending activation
    Pending,
    /// Policy has expired
    Expired,
    /// Policy has been cancelled
    Cancelled,
    /// Policy is suspended
    Suspended,
}

/// Multi-signature wallet configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiSigConfig {
    /// Required number of signatures
    pub required_signatures: u32,
    /// Total number of signers
    pub total_signers: u32,
    /// List of authorized signers
    pub signers: Vec<SignerInfo>,
    /// Wallet creation timestamp
    pub created_at: DateTime<Utc>,
    /// Configuration metadata
    pub metadata: HashMap<String, String>,
}

/// Signer information for multi-signature wallets
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignerInfo {
    /// Signer unique identifier
    pub id: String,
    /// Signer public key
    pub public_key: String,
    /// Signer role and permissions
    pub role: SignerRole,
    /// Signer status
    pub status: SignerStatus,
    /// Signer addition timestamp
    pub added_at: DateTime<Utc>,
}

/// Signer roles in multi-signature wallets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignerRole {
    /// Primary signer with full permissions
    Primary,
    /// Secondary signer with limited permissions
    Secondary,
    /// Emergency signer for recovery operations
    Emergency,
    /// Audit signer for compliance purposes
    Audit,
}

/// Signer status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignerStatus {
    /// Signer is active and can sign transactions
    Active,
    /// Signer is temporarily suspended
    Suspended,
    /// Signer has been revoked
    Revoked,
    /// Signer is pending activation
    Pending,
}

/// Custody proof information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodyProof {
    /// Proof unique identifier
    pub id: Uuid,
    /// Asset being proven
    pub asset_id: Uuid,
    /// Proof type
    pub proof_type: ProofType,
    /// Proof data and evidence
    pub proof_data: ProofData,
    /// Proof generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Proof validity period
    pub valid_until: DateTime<Utc>,
    /// Proof verification status
    pub verification_status: VerificationStatus,
    /// Digital signature of the proof
    pub signature: String,
}

/// Types of custody proofs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    /// Proof of asset existence
    Existence,
    /// Proof of asset ownership
    Ownership,
    /// Proof of asset reserves
    Reserves,
    /// Proof of asset location
    Location,
    /// Proof of asset condition
    Condition,
}

/// Proof data containing evidence and verification information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofData {
    /// Cryptographic hash of the proof
    pub hash: String,
    /// Merkle tree root (if applicable)
    pub merkle_root: Option<String>,
    /// Blockchain transaction hash (if applicable)
    pub transaction_hash: Option<String>,
    /// Additional proof metadata
    pub metadata: HashMap<String, String>,
    /// External verification references
    pub external_refs: Vec<String>,
}

/// Proof verification status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Proof is verified and valid
    Verified,
    /// Proof is pending verification
    Pending,
    /// Proof verification failed
    Failed,
    /// Proof has expired
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_asset_type_creation() {
        let digital_asset = AssetType::Digital {
            network: "ethereum".to_string(),
            contract_address: Some("0x123...".to_string()),
            standard: Some("ERC20".to_string()),
        };
        
        let physical_asset = AssetType::Physical {
            category: "real_estate".to_string(),
            location: "New York, NY".to_string(),
            condition: AssetCondition::Excellent,
        };
        
        assert!(matches!(digital_asset, AssetType::Digital { .. }));
        assert!(matches!(physical_asset, AssetType::Physical { .. }));
    }

    #[test]
    fn test_custody_account_creation() {
        let account = CustodyAccount {
            id: Uuid::new_v4(),
            owner_id: "user123".to_string(),
            account_type: AccountType::Individual,
            status: AccountStatus::Active,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metadata: HashMap::new(),
        };
        
        assert_eq!(account.account_type, AccountType::Individual);
        assert_eq!(account.status, AccountStatus::Active);
    }

    #[test]
    fn test_insurance_policy_creation() {
        let policy = InsurancePolicy {
            id: "policy123".to_string(),
            provider: "InsureCorp".to_string(),
            policy_number: "POL-2024-001".to_string(),
            coverage_amount: dec!(1000000),
            effective_date: Utc::now(),
            expiration_date: Utc::now(),
            status: PolicyStatus::Active,
            premium: dec!(5000),
            deductible: dec!(10000),
            coverage_details: HashMap::new(),
        };
        
        assert_eq!(policy.coverage_amount, dec!(1000000));
        assert_eq!(policy.status, PolicyStatus::Active);
    }

    #[test]
    fn test_multi_sig_config_validation() {
        let config = MultiSigConfig {
            required_signatures: 2,
            total_signers: 3,
            signers: vec![],
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        assert!(config.required_signatures <= config.total_signers);
        assert!(config.required_signatures > 0);
    }

    #[test]
    fn test_custody_proof_creation() {
        let proof = CustodyProof {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            proof_type: ProofType::Existence,
            proof_data: ProofData {
                hash: "0xabc123...".to_string(),
                merkle_root: None,
                transaction_hash: None,
                metadata: HashMap::new(),
                external_refs: vec![],
            },
            generated_at: Utc::now(),
            valid_until: Utc::now(),
            verification_status: VerificationStatus::Verified,
            signature: "signature123".to_string(),
        };
        
        assert_eq!(proof.proof_type, ProofType::Existence);
        assert_eq!(proof.verification_status, VerificationStatus::Verified);
    }
}
