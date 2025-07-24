// =====================================================================================
// File: service-asset/src/models.rs
// Description: Data models for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Asset type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    RealEstate,
    Commodity,
    Artwork,
    Collectible,
    IntellectualProperty,
    Equipment,
    Vehicle,
    Other,
}

impl std::str::FromStr for AssetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "real_estate" | "realestate" => Ok(AssetType::RealEstate),
            "commodity" => Ok(AssetType::Commodity),
            "artwork" => Ok(AssetType::Artwork),
            "collectible" => Ok(AssetType::Collectible),
            "intellectual_property" | "intellectualproperty" => Ok(AssetType::IntellectualProperty),
            "equipment" => Ok(AssetType::Equipment),
            "vehicle" => Ok(AssetType::Vehicle),
            "other" => Ok(AssetType::Other),
            _ => Err(format!("Invalid asset type: {}", s)),
        }
    }
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::RealEstate => write!(f, "real_estate"),
            AssetType::Commodity => write!(f, "commodity"),
            AssetType::Artwork => write!(f, "artwork"),
            AssetType::Collectible => write!(f, "collectible"),
            AssetType::IntellectualProperty => write!(f, "intellectual_property"),
            AssetType::Equipment => write!(f, "equipment"),
            AssetType::Vehicle => write!(f, "vehicle"),
            AssetType::Other => write!(f, "other"),
        }
    }
}

/// Asset status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetStatus {
    Active,
    Inactive,
    UnderReview,
    Sold,
    Deprecated,
}

impl std::str::FromStr for AssetStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(AssetStatus::Active),
            "inactive" => Ok(AssetStatus::Inactive),
            "under_review" | "underreview" => Ok(AssetStatus::UnderReview),
            "sold" => Ok(AssetStatus::Sold),
            "deprecated" => Ok(AssetStatus::Deprecated),
            _ => Err(format!("Invalid asset status: {}", s)),
        }
    }
}

impl std::fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetStatus::Active => write!(f, "active"),
            AssetStatus::Inactive => write!(f, "inactive"),
            AssetStatus::UnderReview => write!(f, "under_review"),
            AssetStatus::Sold => write!(f, "sold"),
            AssetStatus::Deprecated => write!(f, "deprecated"),
        }
    }
}

/// Asset model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub status: AssetStatus,
    pub total_value: Decimal,
    pub tokenized_value: Option<Decimal>,
    pub currency: String,
    pub location: Option<String>,
    pub is_tokenized: bool,
    pub token_address: Option<String>,
    pub blockchain_network: Option<String>,
    pub token_supply: Option<i64>,
    pub token_symbol: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset metadata model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub metadata_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset valuation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValuation {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub value: Decimal,
    pub currency: String,
    pub valuation_method: String,
    pub valuation_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Tokenization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizationResult {
    pub asset_id: Uuid,
    pub token_address: String,
    pub blockchain_network: String,
    pub token_supply: u64,
    pub token_symbol: String,
    pub transaction_hash: String,
    pub block_number: Option<u64>,
}

/// Asset repository trait for dependency injection
#[async_trait]
pub trait AssetRepository: Send + Sync {
    async fn create(&self, asset: &Asset) -> Result<Asset, crate::AssetError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Asset>, crate::AssetError>;
    async fn find_by_owner(&self, owner_id: &str) -> Result<Vec<Asset>, crate::AssetError>;
    async fn update(&self, asset: &Asset) -> Result<Asset, crate::AssetError>;
    async fn delete(&self, id: &str) -> Result<(), crate::AssetError>;
    async fn list_with_filters(
        &self,
        filters: &AssetFilters,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Asset>, i64), crate::AssetError>;
}

/// Asset filters for querying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFilters {
    pub asset_type: Option<AssetType>,
    pub owner_id: Option<Uuid>,
    pub status: Option<AssetStatus>,
    pub min_value: Option<Decimal>,
    pub max_value: Option<Decimal>,
    pub is_tokenized: Option<bool>,
}

/// In-memory repository implementation for demo purposes
pub struct InMemoryAssetRepository {
    assets: std::sync::Arc<tokio::sync::RwLock<HashMap<Uuid, Asset>>>,
}

impl InMemoryAssetRepository {
    pub fn new() -> Self {
        Self {
            assets: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AssetRepository for InMemoryAssetRepository {
    async fn create(&self, asset: &Asset) -> Result<Asset, crate::AssetError> {
        let mut assets = self.assets.write().await;
        let asset_clone = asset.clone();
        assets.insert(asset.id, asset_clone.clone());
        Ok(asset_clone)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Asset>, crate::AssetError> {
        let asset_id = Uuid::parse_str(id)
            .map_err(|_| crate::AssetError::InvalidAssetId(id.to_string()))?;
        let assets = self.assets.read().await;
        Ok(assets.get(&asset_id).cloned())
    }

    async fn find_by_owner(&self, owner_id: &str) -> Result<Vec<Asset>, crate::AssetError> {
        let owner_uuid = Uuid::parse_str(owner_id)
            .map_err(|_| crate::AssetError::InvalidAssetId(owner_id.to_string()))?;
        let assets = self.assets.read().await;
        let owned_assets: Vec<Asset> = assets
            .values()
            .filter(|asset| asset.owner_id == owner_uuid)
            .cloned()
            .collect();
        Ok(owned_assets)
    }

    async fn update(&self, asset: &Asset) -> Result<Asset, crate::AssetError> {
        let mut assets = self.assets.write().await;
        if assets.contains_key(&asset.id) {
            assets.insert(asset.id, asset.clone());
            Ok(asset.clone())
        } else {
            Err(crate::AssetError::AssetNotFound(asset.id.to_string()))
        }
    }

    async fn delete(&self, id: &str) -> Result<(), crate::AssetError> {
        let asset_id = Uuid::parse_str(id)
            .map_err(|_| crate::AssetError::InvalidAssetId(id.to_string()))?;
        let mut assets = self.assets.write().await;
        if assets.remove(&asset_id).is_some() {
            Ok(())
        } else {
            Err(crate::AssetError::AssetNotFound(id.to_string()))
        }
    }

    async fn list_with_filters(
        &self,
        filters: &AssetFilters,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Asset>, i64), crate::AssetError> {
        let assets = self.assets.read().await;
        let mut filtered_assets: Vec<Asset> = assets
            .values()
            .filter(|asset| {
                // Apply filters
                if let Some(asset_type) = &filters.asset_type {
                    if asset.asset_type != *asset_type {
                        return false;
                    }
                }
                if let Some(owner_id) = &filters.owner_id {
                    if asset.owner_id != *owner_id {
                        return false;
                    }
                }
                if let Some(status) = &filters.status {
                    if asset.status != *status {
                        return false;
                    }
                }
                if let Some(min_value) = &filters.min_value {
                    if asset.total_value < *min_value {
                        return false;
                    }
                }
                if let Some(max_value) = &filters.max_value {
                    if asset.total_value > *max_value {
                        return false;
                    }
                }
                if let Some(is_tokenized) = filters.is_tokenized {
                    if asset.is_tokenized != is_tokenized {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by created_at descending
        filtered_assets.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let total = filtered_assets.len() as i64;
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, filtered_assets.len());

        let paginated_assets = if start < filtered_assets.len() {
            filtered_assets[start..end].to_vec()
        } else {
            vec![]
        };

        Ok((paginated_assets, total))
    }
}




