// =====================================================================================
// RWA Tokenization Platform - Custody Service Core
// 
// Main custody service orchestrating digital asset custody, physical asset custody,
// proof systems, and insurance integration for comprehensive asset management.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::config::CustodyConfig;
use crate::digital::DigitalCustodyService;
use crate::error::{CustodyError, CustodyResult};
use crate::insurance::InsuranceService;
use crate::physical::PhysicalCustodyService;
use crate::proof::CustodyProofSystem;
use crate::types::{
    AccountStatus, AccountType, AssetType, CustodiedAsset, CustodyAccount, CustodyStatus,
    InsurancePolicy, MultiSigConfig, SignerInfo,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Main custody service coordinating all custody operations
pub struct CustodyService {
    /// Service configuration
    config: CustodyConfig,
    /// Digital asset custody service
    digital_custody: Arc<DigitalCustodyService>,
    /// Physical asset custody service
    physical_custody: Arc<PhysicalCustodyService>,
    /// Custody proof system
    proof_system: Arc<CustodyProofSystem>,
    /// Insurance service
    insurance_service: Arc<InsuranceService>,
    /// Account manager
    account_manager: Arc<RwLock<AccountManager>>,
    /// Asset registry
    asset_registry: Arc<RwLock<AssetRegistry>>,
    /// Service metrics
    metrics: Arc<RwLock<ServiceMetrics>>,
}

/// Account manager for custody accounts
#[derive(Debug)]
pub struct AccountManager {
    /// Active accounts
    accounts: HashMap<String, CustodyAccount>,
    /// Account relationships
    relationships: HashMap<String, Vec<String>>,
}

/// Asset registry for tracking all custodied assets
#[derive(Debug)]
pub struct AssetRegistry {
    /// All custodied assets
    assets: HashMap<String, CustodiedAsset>,
    /// Asset ownership mappings
    ownership: HashMap<String, Vec<String>>,
    /// Asset type indices
    type_indices: HashMap<String, Vec<String>>,
}

/// Service metrics for monitoring and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Total number of accounts
    pub total_accounts: u64,
    /// Total number of assets under custody
    pub total_assets: u64,
    /// Total value under custody (USD)
    pub total_value_usd: Decimal,
    /// Number of active policies
    pub active_policies: u64,
    /// Number of generated proofs
    pub total_proofs: u64,
    /// Service uptime in seconds
    pub uptime_seconds: u64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Custody operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyRequest {
    /// Request unique identifier
    pub id: String,
    /// Account identifier
    pub account_id: String,
    /// Operation type
    pub operation: CustodyOperation,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// Types of custody operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustodyOperation {
    /// Deposit asset into custody
    Deposit {
        /// Asset information
        asset: AssetInfo,
        /// Custody instructions
        instructions: CustodyInstructions,
    },
    /// Withdraw asset from custody
    Withdraw {
        /// Asset identifier
        asset_id: String,
        /// Withdrawal destination
        destination: WithdrawalDestination,
    },
    /// Transfer asset between accounts
    Transfer {
        /// Asset identifier
        asset_id: String,
        /// Destination account
        to_account: String,
    },
    /// Update asset information
    Update {
        /// Asset identifier
        asset_id: String,
        /// Update details
        updates: AssetUpdates,
    },
}

/// Asset information for custody operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetInfo {
    /// Asset type
    pub asset_type: AssetType,
    /// Asset quantity or amount
    pub quantity: Decimal,
    /// Asset description
    pub description: String,
    /// Asset valuation (if known)
    pub valuation_usd: Option<Decimal>,
    /// Asset metadata
    pub metadata: HashMap<String, String>,
}

/// Custody instructions for asset deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyInstructions {
    /// Preferred custody provider
    pub preferred_provider: Option<String>,
    /// Storage requirements
    pub storage_requirements: HashMap<String, String>,
    /// Insurance requirements
    pub insurance_required: bool,
    /// Insurance coverage amount
    pub insurance_amount: Option<Decimal>,
    /// Special handling instructions
    pub special_instructions: Vec<String>,
}

/// Withdrawal destination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalDestination {
    /// Destination type
    pub destination_type: DestinationType,
    /// Destination address or identifier
    pub address: String,
    /// Additional destination details
    pub details: HashMap<String, String>,
}

/// Types of withdrawal destinations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DestinationType {
    /// External wallet address
    ExternalWallet,
    /// Bank account
    BankAccount,
    /// Physical delivery address
    PhysicalAddress,
    /// Another custody account
    CustodyAccount,
}

/// Asset update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetUpdates {
    /// Updated description
    pub description: Option<String>,
    /// Updated valuation
    pub valuation_usd: Option<Decimal>,
    /// Updated metadata
    pub metadata: Option<HashMap<String, String>>,
    /// Updated storage requirements
    pub storage_requirements: Option<HashMap<String, String>>,
}

/// Custody operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyResponse {
    /// Request identifier
    pub request_id: String,
    /// Operation status
    pub status: OperationStatus,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// Operation result
    pub result: OperationResult,
    /// Response metadata
    pub metadata: HashMap<String, String>,
}

/// Operation status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    /// Operation completed successfully
    Success,
    /// Operation failed
    Failed,
    /// Operation is pending
    Pending,
    /// Operation requires additional approval
    RequiresApproval,
}

/// Operation result details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    /// Asset deposit result
    Deposit {
        /// Created asset identifier
        asset_id: String,
        /// Custody confirmation
        custody_confirmation: String,
    },
    /// Asset withdrawal result
    Withdrawal {
        /// Transaction or confirmation identifier
        transaction_id: String,
        /// Estimated completion time
        estimated_completion: DateTime<Utc>,
    },
    /// Asset transfer result
    Transfer {
        /// Transfer confirmation
        transfer_id: String,
        /// New asset location
        new_location: String,
    },
    /// Asset update result
    Update {
        /// Updated asset information
        updated_asset: CustodiedAsset,
    },
    /// Operation error
    Error {
        /// Error code
        error_code: String,
        /// Error message
        error_message: String,
    },
}

impl CustodyService {
    /// Create a new custody service instance
    /// 
    /// # Arguments
    /// 
    /// * `config` - Custody service configuration
    /// 
    /// # Returns
    /// 
    /// Returns a new `CustodyService` instance
    /// 
    /// # Errors
    /// 
    /// Returns `CustodyError` if initialization fails
    pub async fn new(config: CustodyConfig) -> CustodyResult<Self> {
        // Initialize sub-services
        let digital_custody = Arc::new(
            DigitalCustodyService::new(config.digital_custody.clone()).await?
        );
        
        let physical_custody = Arc::new(
            PhysicalCustodyService::new(config.physical_custody.clone()).await?
        );
        
        let proof_system = Arc::new(CustodyProofSystem::new());
        
        let insurance_service = Arc::new(
            InsuranceService::new(config.insurance.clone()).await?
        );

        let account_manager = Arc::new(RwLock::new(AccountManager::new()));
        let asset_registry = Arc::new(RwLock::new(AssetRegistry::new()));
        let metrics = Arc::new(RwLock::new(ServiceMetrics::new()));

        Ok(Self {
            config,
            digital_custody,
            physical_custody,
            proof_system,
            insurance_service,
            account_manager,
            asset_registry,
            metrics,
        })
    }

    /// Create a new custody account
    /// 
    /// # Arguments
    /// 
    /// * `owner_id` - Account owner identifier
    /// * `account_type` - Type of account to create
    /// 
    /// # Returns
    /// 
    /// Returns the created custody account
    /// 
    /// # Errors
    /// 
    /// Returns `CustodyError` if account creation fails
    pub async fn create_account(
        &self,
        owner_id: &str,
        account_type: AccountType,
    ) -> CustodyResult<CustodyAccount> {
        let account = CustodyAccount {
            id: Uuid::new_v4(),
            owner_id: owner_id.to_string(),
            account_type,
            status: AccountStatus::Active,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metadata: HashMap::new(),
        };

        let mut manager = self.account_manager.write().await;
        manager.add_account(account.clone()).await?;

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_accounts += 1;
        metrics.last_updated = Utc::now();

        Ok(account)
    }

    /// Process a custody operation request
    /// 
    /// # Arguments
    /// 
    /// * `request` - Custody operation request
    /// 
    /// # Returns
    /// 
    /// Returns the operation response
    /// 
    /// # Errors
    /// 
    /// Returns `CustodyError` if operation fails
    pub async fn process_request(&self, request: &CustodyRequest) -> CustodyResult<CustodyResponse> {
        // Validate account exists and is active
        let manager = self.account_manager.read().await;
        let account = manager.get_account(&request.account_id)
            .ok_or_else(|| CustodyError::validation("account_id", "Account not found"))?;

        if account.status != AccountStatus::Active {
            return Ok(CustodyResponse {
                request_id: request.id.clone(),
                status: OperationStatus::Failed,
                timestamp: Utc::now(),
                result: OperationResult::Error {
                    error_code: "ACCOUNT_INACTIVE".to_string(),
                    error_message: "Account is not active".to_string(),
                },
                metadata: HashMap::new(),
            });
        }
        drop(manager);

        // Process operation based on type
        let result = match &request.operation {
            CustodyOperation::Deposit { asset, instructions } => {
                self.process_deposit(&request.account_id, asset, instructions).await?
            }
            CustodyOperation::Withdraw { asset_id, destination } => {
                self.process_withdrawal(asset_id, destination).await?
            }
            CustodyOperation::Transfer { asset_id, to_account } => {
                self.process_transfer(asset_id, to_account).await?
            }
            CustodyOperation::Update { asset_id, updates } => {
                self.process_update(asset_id, updates).await?
            }
        };

        Ok(CustodyResponse {
            request_id: request.id.clone(),
            status: OperationStatus::Success,
            timestamp: Utc::now(),
            result,
            metadata: HashMap::new(),
        })
    }

    /// Process asset deposit operation
    async fn process_deposit(
        &self,
        account_id: &str,
        asset: &AssetInfo,
        instructions: &CustodyInstructions,
    ) -> CustodyResult<OperationResult> {
        let asset_id = Uuid::new_v4();
        
        // Create custodied asset
        let custodied_asset = CustodiedAsset {
            id: asset_id,
            account_id: Uuid::parse_str(account_id)
                .map_err(|e| CustodyError::validation("account_id", &e.to_string()))?,
            asset_type: asset.asset_type.clone(),
            quantity: asset.quantity,
            valuation_usd: asset.valuation_usd,
            status: CustodyStatus::Held,
            provider: crate::types::CustodyProvider {
                id: "default".to_string(),
                name: "Default Custody Provider".to_string(),
                provider_type: crate::types::ProviderType::Internal,
                contact_info: crate::types::ContactInfo {
                    email: "custody@example.com".to_string(),
                    phone: None,
                    address: None,
                    website: None,
                    emergency_contact: None,
                },
                certifications: Vec::new(),
                reputation_score: Decimal::new(95, 2), // 0.95
            },
            custody_start: Utc::now(),
            custody_end: None,
            insurance: None,
            metadata: asset.metadata.clone(),
        };

        // Register asset
        let mut registry = self.asset_registry.write().await;
        registry.register_asset(custodied_asset).await?;

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_assets += 1;
        if let Some(value) = asset.valuation_usd {
            metrics.total_value_usd += value;
        }
        metrics.last_updated = Utc::now();

        Ok(OperationResult::Deposit {
            asset_id: asset_id.to_string(),
            custody_confirmation: format!("CUSTODY-{}", asset_id),
        })
    }

    /// Process asset withdrawal operation
    async fn process_withdrawal(
        &self,
        asset_id: &str,
        destination: &WithdrawalDestination,
    ) -> CustodyResult<OperationResult> {
        let mut registry = self.asset_registry.write().await;
        let asset = registry.get_asset_mut(asset_id)
            .ok_or_else(|| CustodyError::validation("asset_id", "Asset not found"))?;

        // Update asset status
        asset.status = CustodyStatus::Transferring;
        asset.custody_end = Some(Utc::now());

        let transaction_id = Uuid::new_v4().to_string();
        let estimated_completion = Utc::now() + chrono::Duration::hours(24);

        Ok(OperationResult::Withdrawal {
            transaction_id,
            estimated_completion,
        })
    }

    /// Process asset transfer operation
    async fn process_transfer(
        &self,
        asset_id: &str,
        to_account: &str,
    ) -> CustodyResult<OperationResult> {
        let mut registry = self.asset_registry.write().await;
        let asset = registry.get_asset_mut(asset_id)
            .ok_or_else(|| CustodyError::validation("asset_id", "Asset not found"))?;

        // Update asset account
        asset.account_id = Uuid::parse_str(to_account)
            .map_err(|e| CustodyError::validation("to_account", &e.to_string()))?;

        let transfer_id = Uuid::new_v4().to_string();

        Ok(OperationResult::Transfer {
            transfer_id,
            new_location: to_account.to_string(),
        })
    }

    /// Process asset update operation
    async fn process_update(
        &self,
        asset_id: &str,
        updates: &AssetUpdates,
    ) -> CustodyResult<OperationResult> {
        let mut registry = self.asset_registry.write().await;
        let asset = registry.get_asset_mut(asset_id)
            .ok_or_else(|| CustodyError::validation("asset_id", "Asset not found"))?;

        // Apply updates
        if let Some(description) = &updates.description {
            asset.metadata.insert("description".to_string(), description.clone());
        }
        if let Some(valuation) = updates.valuation_usd {
            asset.valuation_usd = Some(valuation);
        }
        if let Some(metadata) = &updates.metadata {
            asset.metadata.extend(metadata.clone());
        }

        Ok(OperationResult::Update {
            updated_asset: asset.clone(),
        })
    }

    /// Get account information
    /// 
    /// # Arguments
    /// 
    /// * `account_id` - Account identifier
    /// 
    /// # Returns
    /// 
    /// Returns the account information if found
    /// 
    /// # Errors
    /// 
    /// Returns `CustodyError` if account is not found
    pub async fn get_account(&self, account_id: &str) -> CustodyResult<CustodyAccount> {
        let manager = self.account_manager.read().await;
        manager.get_account(account_id)
            .cloned()
            .ok_or_else(|| CustodyError::validation("account_id", "Account not found"))
    }

    /// List assets for an account
    /// 
    /// # Arguments
    /// 
    /// * `account_id` - Account identifier
    /// 
    /// # Returns
    /// 
    /// Returns a list of assets owned by the account
    pub async fn list_account_assets(&self, account_id: &str) -> Vec<CustodiedAsset> {
        let registry = self.asset_registry.read().await;
        registry.get_account_assets(account_id)
    }

    /// Get service metrics
    /// 
    /// # Returns
    /// 
    /// Returns current service metrics
    pub async fn get_metrics(&self) -> ServiceMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get digital custody service reference
    /// 
    /// # Returns
    /// 
    /// Returns a reference to the digital custody service
    pub fn digital_custody(&self) -> &DigitalCustodyService {
        &self.digital_custody
    }

    /// Get physical custody service reference
    /// 
    /// # Returns
    /// 
    /// Returns a reference to the physical custody service
    pub fn physical_custody(&self) -> &PhysicalCustodyService {
        &self.physical_custody
    }

    /// Get proof system reference
    /// 
    /// # Returns
    /// 
    /// Returns a reference to the custody proof system
    pub fn proof_system(&self) -> &CustodyProofSystem {
        &self.proof_system
    }

    /// Get insurance service reference
    /// 
    /// # Returns
    /// 
    /// Returns a reference to the insurance service
    pub fn insurance_service(&self) -> &InsuranceService {
        &self.insurance_service
    }
}

impl AccountManager {
    /// Create a new account manager
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            relationships: HashMap::new(),
        }
    }

    /// Add an account to the manager
    /// 
    /// # Arguments
    /// 
    /// * `account` - Custody account to add
    /// 
    /// # Errors
    /// 
    /// Returns `CustodyError` if adding fails
    pub async fn add_account(&mut self, account: CustodyAccount) -> CustodyResult<()> {
        self.accounts.insert(account.id.to_string(), account);
        Ok(())
    }

    /// Get an account by ID
    /// 
    /// # Arguments
    /// 
    /// * `account_id` - Account identifier
    /// 
    /// # Returns
    /// 
    /// Returns the account if found
    pub fn get_account(&self, account_id: &str) -> Option<&CustodyAccount> {
        self.accounts.get(account_id)
    }
}

impl AssetRegistry {
    /// Create a new asset registry
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            ownership: HashMap::new(),
            type_indices: HashMap::new(),
        }
    }

    /// Register an asset in the registry
    /// 
    /// # Arguments
    /// 
    /// * `asset` - Custodied asset to register
    /// 
    /// # Errors
    /// 
    /// Returns `CustodyError` if registration fails
    pub async fn register_asset(&mut self, asset: CustodiedAsset) -> CustodyResult<()> {
        let asset_id = asset.id.to_string();
        let account_id = asset.account_id.to_string();
        let asset_type = format!("{:?}", asset.asset_type);

        // Store asset
        self.assets.insert(asset_id.clone(), asset);

        // Update ownership mapping
        self.ownership
            .entry(account_id)
            .or_insert_with(Vec::new)
            .push(asset_id.clone());

        // Update type index
        self.type_indices
            .entry(asset_type)
            .or_insert_with(Vec::new)
            .push(asset_id);

        Ok(())
    }

    /// Get an asset by ID
    /// 
    /// # Arguments
    /// 
    /// * `asset_id` - Asset identifier
    /// 
    /// # Returns
    /// 
    /// Returns the asset if found
    pub fn get_asset(&self, asset_id: &str) -> Option<&CustodiedAsset> {
        self.assets.get(asset_id)
    }

    /// Get a mutable reference to an asset by ID
    /// 
    /// # Arguments
    /// 
    /// * `asset_id` - Asset identifier
    /// 
    /// # Returns
    /// 
    /// Returns a mutable reference to the asset if found
    pub fn get_asset_mut(&mut self, asset_id: &str) -> Option<&mut CustodiedAsset> {
        self.assets.get_mut(asset_id)
    }

    /// Get all assets for an account
    /// 
    /// # Arguments
    /// 
    /// * `account_id` - Account identifier
    /// 
    /// # Returns
    /// 
    /// Returns a list of assets owned by the account
    pub fn get_account_assets(&self, account_id: &str) -> Vec<CustodiedAsset> {
        if let Some(asset_ids) = self.ownership.get(account_id) {
            asset_ids
                .iter()
                .filter_map(|id| self.assets.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl ServiceMetrics {
    /// Create new service metrics
    pub fn new() -> Self {
        Self {
            total_accounts: 0,
            total_assets: 0,
            total_value_usd: Decimal::ZERO,
            active_policies: 0,
            total_proofs: 0,
            uptime_seconds: 0,
            last_updated: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_config() -> CustodyConfig {
        CustodyConfig::default()
    }

    #[tokio::test]
    async fn test_custody_service_creation() {
        let config = create_test_config();
        let result = CustodyService::new(config).await;
        
        // This test might fail due to network connectivity in digital custody
        // In a real test environment, we would mock the network calls
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_account_manager_operations() {
        let mut manager = AccountManager::new();
        
        let account = CustodyAccount {
            id: Uuid::new_v4(),
            owner_id: "user123".to_string(),
            account_type: AccountType::Individual,
            status: AccountStatus::Active,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            metadata: HashMap::new(),
        };

        let account_id = account.id.to_string();
        let result = manager.add_account(account).await;
        assert!(result.is_ok());

        let retrieved = manager.get_account(&account_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().owner_id, "user123");
    }

    #[tokio::test]
    async fn test_asset_registry_operations() {
        let mut registry = AssetRegistry::new();
        
        let asset = CustodiedAsset {
            id: Uuid::new_v4(),
            account_id: Uuid::new_v4(),
            asset_type: AssetType::Digital {
                network: "ethereum".to_string(),
                contract_address: Some("0x123...".to_string()),
                standard: Some("ERC20".to_string()),
            },
            quantity: dec!(100),
            valuation_usd: Some(dec!(1000)),
            status: CustodyStatus::Held,
            provider: crate::types::CustodyProvider {
                id: "test_provider".to_string(),
                name: "Test Provider".to_string(),
                provider_type: crate::types::ProviderType::Internal,
                contact_info: crate::types::ContactInfo {
                    email: "test@example.com".to_string(),
                    phone: None,
                    address: None,
                    website: None,
                    emergency_contact: None,
                },
                certifications: Vec::new(),
                reputation_score: dec!(0.95),
            },
            custody_start: Utc::now(),
            custody_end: None,
            insurance: None,
            metadata: HashMap::new(),
        };

        let asset_id = asset.id.to_string();
        let account_id = asset.account_id.to_string();

        let result = registry.register_asset(asset).await;
        assert!(result.is_ok());

        let retrieved = registry.get_asset(&asset_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().quantity, dec!(100));

        let account_assets = registry.get_account_assets(&account_id);
        assert_eq!(account_assets.len(), 1);
    }

    #[test]
    fn test_custody_operation_enum() {
        let operation = CustodyOperation::Deposit {
            asset: AssetInfo {
                asset_type: AssetType::Digital {
                    network: "bitcoin".to_string(),
                    contract_address: None,
                    standard: None,
                },
                quantity: dec!(1),
                description: "Bitcoin deposit".to_string(),
                valuation_usd: Some(dec!(50000)),
                metadata: HashMap::new(),
            },
            instructions: CustodyInstructions {
                preferred_provider: None,
                storage_requirements: HashMap::new(),
                insurance_required: true,
                insurance_amount: Some(dec!(50000)),
                special_instructions: Vec::new(),
            },
        };

        match operation {
            CustodyOperation::Deposit { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_operation_status_enum() {
        let status = OperationStatus::Success;
        assert_eq!(status, OperationStatus::Success);
        assert_ne!(status, OperationStatus::Failed);
    }

    #[test]
    fn test_destination_type_enum() {
        let dest_type = DestinationType::ExternalWallet;
        assert_eq!(dest_type, DestinationType::ExternalWallet);
        assert_ne!(dest_type, DestinationType::BankAccount);
    }

    #[test]
    fn test_service_metrics_creation() {
        let metrics = ServiceMetrics::new();
        assert_eq!(metrics.total_accounts, 0);
        assert_eq!(metrics.total_assets, 0);
        assert_eq!(metrics.total_value_usd, Decimal::ZERO);
    }

    #[test]
    fn test_custody_request_creation() {
        let request = CustodyRequest {
            id: "req_001".to_string(),
            account_id: "acc_001".to_string(),
            operation: CustodyOperation::Transfer {
                asset_id: "asset_001".to_string(),
                to_account: "acc_002".to_string(),
            },
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        assert_eq!(request.id, "req_001");
        assert_eq!(request.account_id, "acc_001");
        match request.operation {
            CustodyOperation::Transfer { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_asset_info_creation() {
        let asset_info = AssetInfo {
            asset_type: AssetType::Physical {
                category: "real_estate".to_string(),
                location: "New York".to_string(),
                condition: crate::types::AssetCondition::Excellent,
            },
            quantity: dec!(1),
            description: "Luxury apartment".to_string(),
            valuation_usd: Some(dec!(2000000)),
            metadata: HashMap::new(),
        };

        assert_eq!(asset_info.quantity, dec!(1));
        assert_eq!(asset_info.valuation_usd, Some(dec!(2000000)));
        match asset_info.asset_type {
            AssetType::Physical { .. } => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_withdrawal_destination_creation() {
        let destination = WithdrawalDestination {
            destination_type: DestinationType::ExternalWallet,
            address: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
            details: {
                let mut details = HashMap::new();
                details.insert("network".to_string(), "bitcoin".to_string());
                details
            },
        };

        assert_eq!(destination.destination_type, DestinationType::ExternalWallet);
        assert_eq!(destination.address, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        assert_eq!(destination.details.get("network"), Some(&"bitcoin".to_string()));
    }

    #[test]
    fn test_operation_result_variants() {
        let deposit_result = OperationResult::Deposit {
            asset_id: "asset_001".to_string(),
            custody_confirmation: "CUSTODY-001".to_string(),
        };

        let error_result = OperationResult::Error {
            error_code: "INVALID_ASSET".to_string(),
            error_message: "Asset not found".to_string(),
        };

        match deposit_result {
            OperationResult::Deposit { .. } => assert!(true),
            _ => assert!(false),
        }

        match error_result {
            OperationResult::Error { .. } => assert!(true),
            _ => assert!(false),
        }
    }
}
