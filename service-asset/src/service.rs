// =====================================================================================
// File: service-asset/src/service.rs
// Description: Asset service business logic implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{AssetError, AssetRepository, Asset, AssetMetadata, AssetValuation, TokenizationRequest};
use async_trait::async_trait;
use core_blockchain::{BlockchainServiceFactory, NetworkConfig, RwaAssetToken, ContractManager};
use core_utils::{validation::RwaValidate, helpers::{Pagination, PaginatedResponse}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Asset service trait
#[async_trait]
pub trait AssetService: Send + Sync {
    /// Create a new asset
    async fn create_asset(&self, request: CreateAssetRequest) -> Result<AssetResponse, AssetError>;
    
    /// Get asset by ID
    async fn get_asset(&self, asset_id: &str) -> Result<Option<AssetResponse>, AssetError>;
    
    /// Update asset information
    async fn update_asset(&self, asset_id: &str, request: UpdateAssetRequest) -> Result<AssetResponse, AssetError>;
    
    /// Delete asset
    async fn delete_asset(&self, asset_id: &str) -> Result<(), AssetError>;
    
    /// List assets with pagination
    async fn list_assets(&self, pagination: Pagination, filters: AssetFilters) -> Result<PaginatedResponse<AssetResponse>, AssetError>;
    
    /// Get assets by owner
    async fn get_assets_by_owner(&self, owner_id: &str, pagination: Pagination) -> Result<PaginatedResponse<AssetResponse>, AssetError>;
    
    /// Tokenize asset on blockchain
    async fn tokenize_asset(&self, asset_id: &str, request: TokenizationRequest) -> Result<TokenizationResponse, AssetError>;
    
    /// Update asset valuation
    async fn update_valuation(&self, asset_id: &str, valuation: AssetValuation) -> Result<(), AssetError>;
    
    /// Get asset valuation history
    async fn get_valuation_history(&self, asset_id: &str) -> Result<Vec<AssetValuation>, AssetError>;
    
    /// Transfer asset ownership
    async fn transfer_ownership(&self, asset_id: &str, new_owner_id: &str, transfer_reason: &str) -> Result<(), AssetError>;
    
    /// Get asset metadata
    async fn get_metadata(&self, asset_id: &str) -> Result<Option<AssetMetadata>, AssetError>;
    
    /// Update asset metadata
    async fn update_metadata(&self, asset_id: &str, metadata: AssetMetadata) -> Result<(), AssetError>;
}

/// Asset service implementation
pub struct AssetServiceImpl {
    repository: Arc<dyn AssetRepository>,
    contract_manager: Arc<ContractManager>,
    blockchain_configs: HashMap<String, NetworkConfig>,
}

impl AssetServiceImpl {
    pub fn new(
        repository: Arc<dyn AssetRepository>,
        contract_manager: Arc<ContractManager>,
    ) -> Self {
        let mut blockchain_configs = HashMap::new();
        blockchain_configs.insert("ethereum".to_string(), NetworkConfig::ethereum_mainnet());
        blockchain_configs.insert("ethereum_testnet".to_string(), NetworkConfig::ethereum_testnet());

        Self {
            repository,
            contract_manager,
            blockchain_configs,
        }
    }

    /// Validate asset data
    fn validate_asset_data(&self, request: &CreateAssetRequest) -> Result<(), AssetError> {
        RwaValidate::asset_data(
            &request.name,
            &request.description,
            request.total_value,
            &request.currency,
        ).map_err(|e| AssetError::ValidationError(e.to_string()))
    }

    /// Generate asset metadata
    fn generate_metadata(&self, request: &CreateAssetRequest) -> AssetMetadata {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), serde_json::Value::String(request.asset_type.clone()));
        metadata.insert("created_by".to_string(), serde_json::Value::String("system".to_string()));
        
        if let Some(ref location) = request.location {
            metadata.insert("location".to_string(), serde_json::Value::String(location.clone()));
        }

        AssetMetadata {
            asset_id: String::new(), // Will be set later
            metadata,
            documents: Vec::new(),
            images: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

#[async_trait]
impl AssetService for AssetServiceImpl {
    async fn create_asset(&self, request: CreateAssetRequest) -> Result<AssetResponse, AssetError> {
        info!(
            name = %request.name,
            asset_type = %request.asset_type,
            owner_id = %request.owner_id,
            total_value = %request.total_value,
            "Creating new asset"
        );

        // Validate input
        self.validate_asset_data(&request)?;

        // Create asset
        let asset = Asset {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            description: request.description,
            asset_type: request.asset_type,
            total_value: request.total_value,
            currency: request.currency,
            owner_id: request.owner_id,
            is_tokenized: false,
            token_address: None,
            blockchain_network: None,
            token_supply: None,
            token_symbol: None,
            status: AssetStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_asset = self.repository.create(&asset).await?;

        // Create metadata
        let mut metadata = self.generate_metadata(&request);
        metadata.asset_id = created_asset.id.clone();
        self.repository.create_metadata(&metadata).await?;

        // Create initial valuation
        let valuation = AssetValuation {
            id: Uuid::new_v4().to_string(),
            asset_id: created_asset.id.clone(),
            value: created_asset.total_value,
            currency: created_asset.currency.clone(),
            valuation_date: chrono::Utc::now(),
            valuation_method: "initial".to_string(),
            appraiser: None,
            notes: Some("Initial asset valuation".to_string()),
            created_at: chrono::Utc::now(),
        };

        self.repository.create_valuation(&valuation).await?;

        info!(
            asset_id = %created_asset.id,
            name = %created_asset.name,
            "Asset created successfully"
        );

        Ok(AssetResponse::from_asset_and_metadata(created_asset, Some(metadata)))
    }

    async fn get_asset(&self, asset_id: &str) -> Result<Option<AssetResponse>, AssetError> {
        debug!(asset_id = %asset_id, "Getting asset");

        let asset = self.repository.find_by_id(asset_id).await?;
        if let Some(asset) = asset {
            let metadata = self.repository.get_metadata(asset_id).await?;
            Ok(Some(AssetResponse::from_asset_and_metadata(asset, metadata)))
        } else {
            Ok(None)
        }
    }

    async fn update_asset(&self, asset_id: &str, request: UpdateAssetRequest) -> Result<AssetResponse, AssetError> {
        info!(asset_id = %asset_id, "Updating asset");

        // Get existing asset
        let mut asset = self.repository.find_by_id(asset_id).await?
            .ok_or_else(|| AssetError::AssetNotFound(asset_id.to_string()))?;

        // Update fields
        if let Some(name) = request.name {
            asset.name = name;
        }
        if let Some(description) = request.description {
            asset.description = description;
        }
        if let Some(total_value) = request.total_value {
            asset.total_value = total_value;
        }
        if let Some(status) = request.status {
            asset.status = status;
        }

        asset.updated_at = chrono::Utc::now();

        // Validate updated data
        RwaValidate::asset_data(
            &asset.name,
            &asset.description,
            asset.total_value,
            &asset.currency,
        ).map_err(|e| AssetError::ValidationError(e.to_string()))?;

        // Update in repository
        let updated_asset = self.repository.update(&asset).await?;

        // Get metadata
        let metadata = self.repository.get_metadata(asset_id).await?;

        info!(asset_id = %asset_id, "Asset updated successfully");

        Ok(AssetResponse::from_asset_and_metadata(updated_asset, metadata))
    }

    async fn delete_asset(&self, asset_id: &str) -> Result<(), AssetError> {
        info!(asset_id = %asset_id, "Deleting asset");

        // Check if asset exists
        let asset = self.repository.find_by_id(asset_id).await?
            .ok_or_else(|| AssetError::AssetNotFound(asset_id.to_string()))?;

        // Check if asset is tokenized
        if asset.is_tokenized {
            return Err(AssetError::CannotDeleteTokenizedAsset);
        }

        // Delete asset and related data
        self.repository.delete(asset_id).await?;
        self.repository.delete_metadata(asset_id).await?;

        info!(asset_id = %asset_id, "Asset deleted successfully");
        Ok(())
    }

    async fn list_assets(&self, pagination: Pagination, filters: AssetFilters) -> Result<PaginatedResponse<AssetResponse>, AssetError> {
        debug!(
            page = pagination.page,
            per_page = pagination.per_page,
            "Listing assets with filters"
        );

        let (assets, total_count) = self.repository.list_with_filters(pagination.clone(), filters).await?;
        
        let mut asset_responses = Vec::new();
        for asset in assets {
            let metadata = self.repository.get_metadata(&asset.id).await?;
            asset_responses.push(AssetResponse::from_asset_and_metadata(asset, metadata));
        }

        let pagination_with_total = pagination.with_total(total_count);

        Ok(PaginatedResponse::new(asset_responses, pagination_with_total))
    }

    async fn get_assets_by_owner(&self, owner_id: &str, pagination: Pagination) -> Result<PaginatedResponse<AssetResponse>, AssetError> {
        debug!(
            owner_id = %owner_id,
            page = pagination.page,
            per_page = pagination.per_page,
            "Getting assets by owner"
        );

        let (assets, total_count) = self.repository.find_by_owner(owner_id, pagination.clone()).await?;
        
        let mut asset_responses = Vec::new();
        for asset in assets {
            let metadata = self.repository.get_metadata(&asset.id).await?;
            asset_responses.push(AssetResponse::from_asset_and_metadata(asset, metadata));
        }

        let pagination_with_total = pagination.with_total(total_count);

        Ok(PaginatedResponse::new(asset_responses, pagination_with_total))
    }

    async fn tokenize_asset(&self, asset_id: &str, request: TokenizationRequest) -> Result<TokenizationResponse, AssetError> {
        info!(
            asset_id = %asset_id,
            blockchain_network = %request.blockchain_network,
            token_supply = %request.token_supply,
            "Tokenizing asset"
        );

        // Get asset
        let mut asset = self.repository.find_by_id(asset_id).await?
            .ok_or_else(|| AssetError::AssetNotFound(asset_id.to_string()))?;

        // Check if already tokenized
        if asset.is_tokenized {
            return Err(AssetError::AssetAlreadyTokenized);
        }

        // Get blockchain configuration
        let network_config = self.blockchain_configs.get(&request.blockchain_network)
            .ok_or_else(|| AssetError::UnsupportedBlockchain(request.blockchain_network.clone()))?;

        // Deploy token contract
        let deployer_address = core_blockchain::Address::new(
            "0x1234567890123456789012345678901234567890".to_string(), // In production, use actual deployer
            network_config.network.clone(),
        );

        let token_contract = self.contract_manager.deploy_rwa_token(&deployer_address, network_config.network.clone()).await
            .map_err(|e| AssetError::BlockchainError(e.to_string()))?;

        // Mint initial tokens
        let owner_address = core_blockchain::Address::new(
            format!("0x{}", &asset.owner_id[..40]), // Simplified address derivation
            network_config.network.clone(),
        );

        token_contract.mint_tokens(&owner_address, &request.token_supply.to_string(), asset_id).await
            .map_err(|e| AssetError::BlockchainError(e.to_string()))?;

        // Update asset with tokenization info
        asset.is_tokenized = true;
        asset.token_address = Some(token_contract.contract_address.address.clone());
        asset.blockchain_network = Some(request.blockchain_network.clone());
        asset.token_supply = Some(request.token_supply);
        asset.token_symbol = Some(request.token_symbol.clone());
        asset.updated_at = chrono::Utc::now();

        let updated_asset = self.repository.update(&asset).await?;

        info!(
            asset_id = %asset_id,
            token_address = %token_contract.contract_address.address,
            "Asset tokenized successfully"
        );

        Ok(TokenizationResponse {
            asset_id: updated_asset.id,
            token_address: token_contract.contract_address.address,
            blockchain_network: request.blockchain_network,
            token_supply: request.token_supply,
            token_symbol: request.token_symbol,
            transaction_hash: "0x123...".to_string(), // Would be actual transaction hash
        })
    }

    async fn update_valuation(&self, asset_id: &str, valuation: AssetValuation) -> Result<(), AssetError> {
        info!(
            asset_id = %asset_id,
            value = %valuation.value,
            currency = %valuation.currency,
            "Updating asset valuation"
        );

        // Verify asset exists
        self.repository.find_by_id(asset_id).await?
            .ok_or_else(|| AssetError::AssetNotFound(asset_id.to_string()))?;

        // Create valuation record
        self.repository.create_valuation(&valuation).await?;

        // Update asset's current value
        let mut asset = self.repository.find_by_id(asset_id).await?.unwrap();
        asset.total_value = valuation.value;
        asset.updated_at = chrono::Utc::now();
        self.repository.update(&asset).await?;

        info!(asset_id = %asset_id, "Asset valuation updated successfully");
        Ok(())
    }

    async fn get_valuation_history(&self, asset_id: &str) -> Result<Vec<AssetValuation>, AssetError> {
        debug!(asset_id = %asset_id, "Getting valuation history");

        // Verify asset exists
        self.repository.find_by_id(asset_id).await?
            .ok_or_else(|| AssetError::AssetNotFound(asset_id.to_string()))?;

        self.repository.get_valuation_history(asset_id).await
    }

    async fn transfer_ownership(&self, asset_id: &str, new_owner_id: &str, transfer_reason: &str) -> Result<(), AssetError> {
        info!(
            asset_id = %asset_id,
            new_owner_id = %new_owner_id,
            transfer_reason = %transfer_reason,
            "Transferring asset ownership"
        );

        // Get asset
        let mut asset = self.repository.find_by_id(asset_id).await?
            .ok_or_else(|| AssetError::AssetNotFound(asset_id.to_string()))?;

        let old_owner_id = asset.owner_id.clone();

        // Update ownership
        asset.owner_id = new_owner_id.to_string();
        asset.updated_at = chrono::Utc::now();

        self.repository.update(&asset).await?;

        // Record ownership transfer
        self.repository.record_ownership_transfer(
            asset_id,
            &old_owner_id,
            new_owner_id,
            transfer_reason,
        ).await?;

        info!(
            asset_id = %asset_id,
            old_owner = %old_owner_id,
            new_owner = %new_owner_id,
            "Asset ownership transferred successfully"
        );

        Ok(())
    }

    async fn get_metadata(&self, asset_id: &str) -> Result<Option<AssetMetadata>, AssetError> {
        debug!(asset_id = %asset_id, "Getting asset metadata");
        self.repository.get_metadata(asset_id).await
    }

    async fn update_metadata(&self, asset_id: &str, mut metadata: AssetMetadata) -> Result<(), AssetError> {
        info!(asset_id = %asset_id, "Updating asset metadata");

        // Verify asset exists
        self.repository.find_by_id(asset_id).await?
            .ok_or_else(|| AssetError::AssetNotFound(asset_id.to_string()))?;

        metadata.asset_id = asset_id.to_string();
        metadata.updated_at = chrono::Utc::now();

        self.repository.update_metadata(&metadata).await?;

        info!(asset_id = %asset_id, "Asset metadata updated successfully");
        Ok(())
    }
}

/// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub total_value: f64,
    pub currency: String,
    pub owner_id: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssetRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub total_value: Option<f64>,
    pub status: Option<AssetStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFilters {
    pub asset_type: Option<String>,
    pub owner_id: Option<String>,
    pub is_tokenized: Option<bool>,
    pub status: Option<AssetStatus>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub total_value: f64,
    pub currency: String,
    pub owner_id: String,
    pub is_tokenized: bool,
    pub token_address: Option<String>,
    pub blockchain_network: Option<String>,
    pub token_supply: Option<u64>,
    pub token_symbol: Option<String>,
    pub status: AssetStatus,
    pub metadata: Option<AssetMetadata>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl AssetResponse {
    pub fn from_asset_and_metadata(asset: Asset, metadata: Option<AssetMetadata>) -> Self {
        Self {
            id: asset.id,
            name: asset.name,
            description: asset.description,
            asset_type: asset.asset_type,
            total_value: asset.total_value,
            currency: asset.currency,
            owner_id: asset.owner_id,
            is_tokenized: asset.is_tokenized,
            token_address: asset.token_address,
            blockchain_network: asset.blockchain_network,
            token_supply: asset.token_supply,
            token_symbol: asset.token_symbol,
            status: asset.status,
            metadata,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizationResponse {
    pub asset_id: String,
    pub token_address: String,
    pub blockchain_network: String,
    pub token_supply: u64,
    pub token_symbol: String,
    pub transaction_hash: String,
}

/// Asset status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetStatus {
    Active,
    Inactive,
    UnderReview,
    Sold,
    Deprecated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_utils::fixtures::AssetFixture;

    // Mock repository for testing
    struct MockAssetRepository;

    #[async_trait]
    impl AssetRepository for MockAssetRepository {
        async fn create(&self, asset: &Asset) -> Result<Asset, AssetError> {
            Ok(asset.clone())
        }

        async fn find_by_id(&self, _id: &str) -> Result<Option<Asset>, AssetError> {
            let fixture = AssetFixture::generate();
            Ok(Some(Asset {
                id: fixture.id,
                name: fixture.name,
                description: fixture.description,
                asset_type: fixture.asset_type,
                total_value: fixture.total_value,
                currency: fixture.currency,
                owner_id: fixture.owner_id,
                is_tokenized: fixture.is_tokenized,
                token_address: fixture.token_address,
                blockchain_network: fixture.blockchain_network,
                token_supply: None,
                token_symbol: None,
                status: AssetStatus::Active,
                created_at: fixture.created_at,
                updated_at: fixture.updated_at,
            }))
        }

        async fn update(&self, asset: &Asset) -> Result<Asset, AssetError> {
            Ok(asset.clone())
        }

        async fn delete(&self, _id: &str) -> Result<(), AssetError> {
            Ok(())
        }

        async fn list_with_filters(&self, _pagination: Pagination, _filters: AssetFilters) -> Result<(Vec<Asset>, u64), AssetError> {
            Ok((vec![], 0))
        }

        async fn find_by_owner(&self, _owner_id: &str, _pagination: Pagination) -> Result<(Vec<Asset>, u64), AssetError> {
            Ok((vec![], 0))
        }

        async fn create_metadata(&self, _metadata: &AssetMetadata) -> Result<AssetMetadata, AssetError> {
            Ok(_metadata.clone())
        }

        async fn get_metadata(&self, _asset_id: &str) -> Result<Option<AssetMetadata>, AssetError> {
            Ok(None)
        }

        async fn update_metadata(&self, metadata: &AssetMetadata) -> Result<AssetMetadata, AssetError> {
            Ok(metadata.clone())
        }

        async fn delete_metadata(&self, _asset_id: &str) -> Result<(), AssetError> {
            Ok(())
        }

        async fn create_valuation(&self, valuation: &AssetValuation) -> Result<AssetValuation, AssetError> {
            Ok(valuation.clone())
        }

        async fn get_valuation_history(&self, _asset_id: &str) -> Result<Vec<AssetValuation>, AssetError> {
            Ok(vec![])
        }

        async fn record_ownership_transfer(&self, _asset_id: &str, _old_owner: &str, _new_owner: &str, _reason: &str) -> Result<(), AssetError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_asset() {
        let repository = Arc::new(MockAssetRepository);
        let contract_manager = Arc::new(ContractManager::new());
        let service = AssetServiceImpl::new(repository, contract_manager);

        let request = CreateAssetRequest {
            name: "Test Asset".to_string(),
            description: "A test asset".to_string(),
            asset_type: "real_estate".to_string(),
            total_value: 1000000.0,
            currency: "USD".to_string(),
            owner_id: "user123".to_string(),
            location: Some("New York".to_string()),
        };

        let result = service.create_asset(request).await;
        assert!(result.is_ok());

        let asset_response = result.unwrap();
        assert_eq!(asset_response.name, "Test Asset");
        assert_eq!(asset_response.total_value, 1000000.0);
        assert!(!asset_response.is_tokenized);
    }

    #[tokio::test]
    async fn test_get_asset() {
        let repository = Arc::new(MockAssetRepository);
        let contract_manager = Arc::new(ContractManager::new());
        let service = AssetServiceImpl::new(repository, contract_manager);

        let result = service.get_asset("test_id").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
