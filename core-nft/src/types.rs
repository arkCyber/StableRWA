// =====================================================================================
// File: core-nft/src/types.rs
// Description: Core types and data structures for NFT operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;
use validator::Validate;

/// Token ID type
pub type TokenId = String;

/// Contract address type
pub type ContractAddress = String;

/// Owner address type
pub type Owner = String;

/// Creator address type
pub type Creator = String;

/// NFT standards supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NFTStandard {
    ERC721,
    ERC1155,
    ERC721A,
    ERC721Enumerable,
    ERC1155Supply,
}

impl Default for NFTStandard {
    fn default() -> Self {
        Self::ERC721
    }
}

/// Metadata standards
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetadataStandard {
    OpenSea,
    ERC721Metadata,
    ERC1155Metadata,
    Custom,
}

impl Default for MetadataStandard {
    fn default() -> Self {
        Self::OpenSea
    }
}

/// NFT attribute
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NFTAttribute {
    #[validate(length(min = 1, max = 100))]
    pub trait_type: String,
    pub value: serde_json::Value,
    pub display_type: Option<String>,
    pub max_value: Option<serde_json::Value>,
}

/// NFT metadata
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NFTMetadata {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    #[validate(url)]
    pub image: Option<String>,
    #[validate(url)]
    pub external_url: Option<String>,
    #[validate(url)]
    pub animation_url: Option<String>,
    pub attributes: Vec<NFTAttribute>,
    pub background_color: Option<String>,
    pub youtube_url: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Royalty information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Royalty {
    pub recipient: String,
    pub percentage: Decimal,
    pub is_primary: bool,
}

/// NFT core structure
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NFT {
    pub id: Uuid,
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub standard: NFTStandard,
    pub owner: Owner,
    pub creator: Creator,
    pub metadata: NFTMetadata,
    pub royalties: Vec<Royalty>,
    pub supply: Option<u64>, // For ERC1155
    pub is_burned: bool,
    pub is_transferable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// NFT collection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NFTCollection {
    pub id: Uuid,
    pub contract_address: ContractAddress,
    pub standard: NFTStandard,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 10))]
    pub symbol: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    #[validate(url)]
    pub image: Option<String>,
    #[validate(url)]
    pub banner_image: Option<String>,
    #[validate(url)]
    pub external_url: Option<String>,
    pub creator: Creator,
    pub owner: Owner,
    pub royalties: Vec<Royalty>,
    pub total_supply: u64,
    pub max_supply: Option<u64>,
    pub is_verified: bool,
    pub is_featured: bool,
    pub floor_price: Option<Decimal>,
    pub volume_24h: Decimal,
    pub volume_total: Decimal,
    pub holders_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Token transfer event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub id: Uuid,
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub from_address: String,
    pub to_address: String,
    pub amount: Option<u64>, // For ERC1155
    pub transaction_hash: String,
    pub block_number: u64,
    pub log_index: u32,
    pub timestamp: DateTime<Utc>,
}

/// Token approval event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenApproval {
    pub id: Uuid,
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub owner: String,
    pub approved: String,
    pub transaction_hash: String,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
}

/// Marketplace listing
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MarketplaceListing {
    pub id: Uuid,
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub seller: String,
    pub price: Decimal,
    pub currency: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_active: bool,
    pub is_sold: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Marketplace offer
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MarketplaceOffer {
    pub id: Uuid,
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub buyer: String,
    pub price: Decimal,
    pub currency: String,
    pub expiry: DateTime<Utc>,
    pub is_active: bool,
    pub is_accepted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sale transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleTransaction {
    pub id: Uuid,
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub seller: String,
    pub buyer: String,
    pub price: Decimal,
    pub currency: String,
    pub platform_fee: Decimal,
    pub royalty_fee: Decimal,
    pub transaction_hash: String,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
}

/// Mint request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MintRequest {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub recipient: String,
    pub metadata: NFTMetadata,
    pub amount: Option<u64>, // For ERC1155
    pub royalties: Vec<Royalty>,
    pub requested_by: String,
    pub status: MintStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mint status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MintStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Batch mint request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BatchMintRequest {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub mint_requests: Vec<MintRequest>,
    pub requested_by: String,
    pub status: MintStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Storage provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageProvider {
    IPFS,
    Arweave,
    Filecoin,
    AWS,
    GoogleCloud,
    Azure,
}

/// Storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub provider: StorageProvider,
    pub hash: String,
    pub url: String,
    pub size: u64,
    pub content_type: String,
    pub is_pinned: bool,
    pub uploaded_at: DateTime<Utc>,
}

/// Collection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionStats {
    pub collection_id: Uuid,
    pub total_supply: u64,
    pub owners_count: u64,
    pub floor_price: Option<Decimal>,
    pub ceiling_price: Option<Decimal>,
    pub volume_24h: Decimal,
    pub volume_7d: Decimal,
    pub volume_30d: Decimal,
    pub volume_total: Decimal,
    pub sales_24h: u64,
    pub sales_7d: u64,
    pub sales_30d: u64,
    pub sales_total: u64,
    pub average_price_24h: Option<Decimal>,
    pub market_cap: Option<Decimal>,
    pub last_updated: DateTime<Utc>,
}

/// Price history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryEntry {
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub price: Decimal,
    pub currency: String,
    pub transaction_hash: String,
    pub timestamp: DateTime<Utc>,
}

/// Trait rarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRarity {
    pub trait_type: String,
    pub value: String,
    pub count: u64,
    pub rarity_percentage: f64,
}

/// Token rarity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRarity {
    pub token_id: TokenId,
    pub contract_address: ContractAddress,
    pub rarity_rank: u64,
    pub rarity_score: f64,
    pub traits: Vec<TraitRarity>,
    pub calculated_at: DateTime<Utc>,
}

/// Transfer request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TransferRequest {
    #[validate(length(min = 42, max = 42))]
    pub contract_address: String,
    pub token_id: String,
    #[validate(length(min = 42, max = 42))]
    pub from: String,
    #[validate(length(min = 42, max = 42))]
    pub to: String,
    pub amount: Option<u64>, // For ERC1155
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApprovalRequest {
    #[validate(length(min = 42, max = 42))]
    pub contract_address: String,
    pub token_id: String,
    #[validate(length(min = 42, max = 42))]
    pub owner: String,
    #[validate(length(min = 42, max = 42))]
    pub approved: String,
}

/// Create collection request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCollectionRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 10))]
    pub symbol: String,
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    #[validate(url)]
    pub image: Option<String>,
    #[validate(url)]
    pub external_url: Option<String>,
    pub standard: NFTStandard,
    #[validate(length(min = 42, max = 42))]
    pub creator: String,
    pub royalties: Vec<Royalty>,
}

/// Create listing request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateListingRequest {
    #[validate(length(min = 42, max = 42))]
    pub contract_address: String,
    pub token_id: String,
    #[validate(length(min = 42, max = 42))]
    pub seller: String,
    pub price: Decimal,
    pub currency: String,
    pub duration_hours: u32,
}

/// Create offer request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateOfferRequest {
    #[validate(length(min = 42, max = 42))]
    pub contract_address: String,
    pub token_id: String,
    #[validate(length(min = 42, max = 42))]
    pub buyer: String,
    pub price: Decimal,
    pub currency: String,
    pub expiration: DateTime<Utc>,
}
