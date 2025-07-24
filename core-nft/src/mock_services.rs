// =====================================================================================
// File: core-nft/src/mock_services.rs
// Description: Mock service implementations for NFT operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    error::NFTResult,
    types::*,
};
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

// Mock ERC721 Service
pub struct MockERC721Service;

impl MockERC721Service {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl crate::erc721::ERC721Service for MockERC721Service {
    async fn get_token(&self, contract_address: &str, token_id: &str) -> NFTResult<NFT> {
        let mock_token = NFT {
            id: Uuid::new_v4(),
            token_id: token_id.to_string(),
            contract_address: contract_address.to_string(),
            standard: NFTStandard::ERC721,
            owner: "0x1234567890123456789012345678901234567890".to_string(),
            creator: "0x1234567890123456789012345678901234567890".to_string(),
            metadata: NFTMetadata {
                name: format!("Mock ERC721 Token #{}", token_id),
                description: Some("A mock ERC721 token".to_string()),
                image: Some("https://example.com/image.png".to_string()),
                external_url: None,
                attributes: vec![],
                animation_url: None,
                background_color: None,
                properties: HashMap::new(),
                youtube_url: None,
            },
            royalties: vec![],
            supply: None,
            is_burned: false,
            is_transferable: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        Ok(mock_token)
    }

    async fn get_tokens_by_owner(&self, _owner: &str) -> NFTResult<Vec<NFT>> {
        Ok(vec![])
    }

    async fn get_tokens_by_collection(&self, _contract_address: &str) -> NFTResult<Vec<NFT>> {
        Ok(vec![])
    }

    async fn transfer_token(&self, _request: TransferRequest) -> NFTResult<String> {
        Ok(format!("0x{:x}", 12345u64))
    }

    async fn approve_token(&self, _request: ApprovalRequest) -> NFTResult<String> {
        Ok(format!("0x{:x}", 67890u64))
    }

    async fn get_owner(&self, _contract_address: &str, _token_id: &str) -> NFTResult<String> {
        Ok("0x1234567890123456789012345678901234567890".to_string())
    }

    async fn get_approved(&self, _contract_address: &str, _token_id: &str) -> NFTResult<String> {
        Ok("0x0000000000000000000000000000000000000000".to_string())
    }

    async fn is_approved_for_all(&self, _contract_address: &str, _owner: &str, _operator: &str) -> NFTResult<bool> {
        Ok(false)
    }

    async fn get_balance(&self, _contract_address: &str, _owner: &str) -> NFTResult<u64> {
        Ok(0)
    }

    async fn get_total_supply(&self, _contract_address: &str) -> NFTResult<u64> {
        Ok(10000)
    }

    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// ERC1155 Service
#[async_trait]
pub trait ERC1155Service: Send + Sync {
    async fn get_token(&self, contract_address: &str, token_id: &str) -> NFTResult<NFT>;
    async fn get_tokens_by_owner(&self, owner: &str) -> NFTResult<Vec<NFT>>;
    async fn get_tokens_by_collection(&self, contract_address: &str) -> NFTResult<Vec<NFT>>;
    async fn transfer_token(&self, request: TransferRequest) -> NFTResult<String>;
    async fn approve_token(&self, request: ApprovalRequest) -> NFTResult<String>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockERC1155Service;

impl MockERC1155Service {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ERC1155Service for MockERC1155Service {
    async fn get_token(&self, contract_address: &str, token_id: &str) -> NFTResult<NFT> {
        let mock_token = NFT {
            id: Uuid::new_v4(),
            token_id: token_id.to_string(),
            contract_address: contract_address.to_string(),
            standard: NFTStandard::ERC1155,
            owner: "0x1234567890123456789012345678901234567890".to_string(),
            creator: "0x1234567890123456789012345678901234567890".to_string(),
            metadata: NFTMetadata {
                name: format!("ERC1155 Token #{}", token_id),
                description: Some("A sample ERC1155 token".to_string()),
                image: Some("https://example.com/image.png".to_string()),
                external_url: None,
                animation_url: None,
                attributes: vec![],
                background_color: None,
                youtube_url: None,
                properties: HashMap::new(),
            },
            royalties: vec![],
            supply: Some(100),
            is_burned: false,
            is_transferable: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        Ok(mock_token)
    }
    
    async fn get_tokens_by_owner(&self, _owner: &str) -> NFTResult<Vec<NFT>> {
        Ok(vec![])
    }
    
    async fn get_tokens_by_collection(&self, _contract_address: &str) -> NFTResult<Vec<NFT>> {
        Ok(vec![])
    }
    
    async fn transfer_token(&self, _request: TransferRequest) -> NFTResult<String> {
        Ok("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string())
    }
    
    async fn approve_token(&self, _request: ApprovalRequest) -> NFTResult<String> {
        Ok("0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string())
    }
    
    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Metadata Service
#[async_trait]
pub trait MetadataService: Send + Sync {
    async fn update_metadata(&self, token_id: &str, metadata: NFTMetadata) -> NFTResult<()>;
    async fn get_metadata(&self, token_id: &str) -> NFTResult<NFTMetadata>;
    async fn validate_metadata(&self, metadata: &NFTMetadata) -> NFTResult<()>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockMetadataService;

impl MockMetadataService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MetadataService for MockMetadataService {
    async fn update_metadata(&self, _token_id: &str, _metadata: NFTMetadata) -> NFTResult<()> {
        Ok(())
    }
    
    async fn get_metadata(&self, token_id: &str) -> NFTResult<NFTMetadata> {
        Ok(NFTMetadata {
            name: format!("Token #{}", token_id),
            description: Some("Mock metadata".to_string()),
            image: Some("https://example.com/image.png".to_string()),
            external_url: None,
            animation_url: None,
            attributes: vec![],
            background_color: None,
            youtube_url: None,
            properties: HashMap::new(),
        })
    }
    
    async fn validate_metadata(&self, _metadata: &NFTMetadata) -> NFTResult<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// IPFS Service
#[async_trait]
pub trait IPFSService: Send + Sync {
    async fn upload_file(&self, data: Vec<u8>) -> NFTResult<String>;
    async fn upload_json(&self, data: serde_json::Value) -> NFTResult<String>;
    async fn get_file(&self, hash: &str) -> NFTResult<Vec<u8>>;
    async fn pin_file(&self, hash: &str) -> NFTResult<()>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockIPFSService;

impl MockIPFSService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IPFSService for MockIPFSService {
    async fn upload_file(&self, _data: Vec<u8>) -> NFTResult<String> {
        Ok("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string())
    }
    
    async fn upload_json(&self, _data: serde_json::Value) -> NFTResult<String> {
        Ok("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string())
    }
    
    async fn get_file(&self, _hash: &str) -> NFTResult<Vec<u8>> {
        Ok(vec![])
    }
    
    async fn pin_file(&self, _hash: &str) -> NFTResult<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Marketplace Service
#[async_trait]
pub trait MarketplaceService: Send + Sync {
    async fn create_listing(&self, request: CreateListingRequest) -> NFTResult<MarketplaceListing>;
    async fn create_offer(&self, request: CreateOfferRequest) -> NFTResult<MarketplaceOffer>;
    async fn execute_sale(&self, listing_id: &Uuid, buyer: &str) -> NFTResult<SaleTransaction>;
    async fn cancel_listing(&self, listing_id: &Uuid) -> NFTResult<()>;
    async fn cancel_offer(&self, offer_id: &Uuid) -> NFTResult<()>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockMarketplaceService;

impl MockMarketplaceService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MarketplaceService for MockMarketplaceService {
    async fn create_listing(&self, request: CreateListingRequest) -> NFTResult<MarketplaceListing> {
        Ok(MarketplaceListing {
            id: Uuid::new_v4(),
            token_id: request.token_id,
            contract_address: request.contract_address,
            seller: request.seller,
            price: request.price,
            currency: request.currency,
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::hours(request.duration_hours as i64),
            is_active: true,
            is_sold: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    async fn create_offer(&self, request: CreateOfferRequest) -> NFTResult<MarketplaceOffer> {
        Ok(MarketplaceOffer {
            id: Uuid::new_v4(),
            token_id: request.token_id,
            contract_address: request.contract_address,
            buyer: request.buyer,
            price: request.price,
            currency: request.currency,
            expiry: request.expiration,
            is_active: true,
            is_accepted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    async fn execute_sale(&self, listing_id: &Uuid, buyer: &str) -> NFTResult<SaleTransaction> {
        Ok(SaleTransaction {
            id: Uuid::new_v4(),
            token_id: "1".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            seller: "0x1234567890123456789012345678901234567890".to_string(),
            buyer: buyer.to_string(),
            price: rust_decimal::Decimal::new(1, 0),
            currency: "ETH".to_string(),
            platform_fee: rust_decimal::Decimal::new(25, 3), // 2.5%
            royalty_fee: rust_decimal::Decimal::new(5, 2), // 5%
            transaction_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            block_number: 12345678,
            timestamp: Utc::now(),
        })
    }
    
    async fn cancel_listing(&self, _listing_id: &Uuid) -> NFTResult<()> {
        Ok(())
    }
    
    async fn cancel_offer(&self, _offer_id: &Uuid) -> NFTResult<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Royalty Service
#[async_trait]
pub trait RoyaltyService: Send + Sync {
    async fn calculate_royalties(&self, token_id: &str, sale_price: rust_decimal::Decimal) -> NFTResult<Vec<Royalty>>;
    async fn set_royalties(&self, token_id: &str, royalties: Vec<Royalty>) -> NFTResult<()>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockRoyaltyService;

impl MockRoyaltyService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RoyaltyService for MockRoyaltyService {
    async fn calculate_royalties(&self, _token_id: &str, _sale_price: rust_decimal::Decimal) -> NFTResult<Vec<Royalty>> {
        Ok(vec![])
    }
    
    async fn set_royalties(&self, _token_id: &str, _royalties: Vec<Royalty>) -> NFTResult<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Collection Service
#[async_trait]
pub trait CollectionService: Send + Sync {
    async fn get_collection(&self, collection_id: &Uuid) -> NFTResult<NFTCollection>;
    async fn get_collections_by_creator(&self, creator: &str) -> NFTResult<Vec<NFTCollection>>;
    async fn create_collection(&self, request: CreateCollectionRequest) -> NFTResult<NFTCollection>;
    async fn get_collection_stats(&self, collection_id: &Uuid) -> NFTResult<CollectionStats>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockCollectionService;

impl MockCollectionService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CollectionService for MockCollectionService {
    async fn get_collection(&self, collection_id: &Uuid) -> NFTResult<NFTCollection> {
        Ok(NFTCollection {
            id: *collection_id,
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            standard: NFTStandard::ERC721,
            name: "Mock Collection".to_string(),
            symbol: "MOCK".to_string(),
            description: Some("A mock NFT collection".to_string()),
            image: Some("https://example.com/collection.png".to_string()),
            banner_image: None,
            external_url: None,
            creator: "0x1234567890123456789012345678901234567890".to_string(),
            owner: "0x1234567890123456789012345678901234567890".to_string(),
            royalties: vec![],
            total_supply: 1000,
            max_supply: Some(10000),
            is_verified: true,
            is_featured: false,
            floor_price: Some(rust_decimal::Decimal::new(1, 1)), // 0.1 ETH
            volume_24h: rust_decimal::Decimal::new(10, 0),
            volume_total: rust_decimal::Decimal::new(1000, 0),
            holders_count: 500,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_collections_by_creator(&self, _creator: &str) -> NFTResult<Vec<NFTCollection>> {
        Ok(vec![])
    }

    async fn create_collection(&self, request: CreateCollectionRequest) -> NFTResult<NFTCollection> {
        Ok(NFTCollection {
            id: Uuid::new_v4(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            standard: request.standard,
            name: request.name,
            symbol: request.symbol,
            description: request.description,
            image: request.image,
            banner_image: None,
            external_url: None,
            creator: request.creator.clone(),
            owner: request.creator,
            royalties: request.royalties,
            total_supply: 0,
            max_supply: Some(10000), // Default max supply
            is_verified: false,
            is_featured: false,
            floor_price: None,
            volume_24h: rust_decimal::Decimal::new(0, 0),
            volume_total: rust_decimal::Decimal::new(0, 0),
            holders_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn get_collection_stats(&self, collection_id: &Uuid) -> NFTResult<CollectionStats> {
        Ok(CollectionStats {
            collection_id: *collection_id,
            total_supply: 1000,
            owners_count: 500,
            floor_price: Some(rust_decimal::Decimal::new(1, 1)),
            ceiling_price: Some(rust_decimal::Decimal::new(10, 0)),
            volume_24h: rust_decimal::Decimal::new(10, 0),
            volume_7d: rust_decimal::Decimal::new(70, 0),
            volume_30d: rust_decimal::Decimal::new(300, 0),
            volume_total: rust_decimal::Decimal::new(1000, 0),
            sales_24h: 5,
            sales_7d: 35,
            sales_30d: 150,
            sales_total: 500,
            average_price_24h: Some(rust_decimal::Decimal::new(2, 0)),
            market_cap: Some(rust_decimal::Decimal::new(100, 0)),
            last_updated: Utc::now(),
        })
    }

    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Minting Service
#[async_trait]
pub trait MintingService: Send + Sync {
    async fn mint_token(&self, request: MintRequest) -> NFTResult<NFT>;
    async fn batch_mint(&self, request: BatchMintRequest) -> NFTResult<Vec<NFT>>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockMintingService;

impl MockMintingService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MintingService for MockMintingService {
    async fn mint_token(&self, request: MintRequest) -> NFTResult<NFT> {
        Ok(NFT {
            id: request.id,
            token_id: "1".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            standard: NFTStandard::ERC721,
            owner: request.recipient,
            creator: request.requested_by,
            metadata: request.metadata,
            royalties: request.royalties,
            supply: request.amount,
            is_burned: false,
            is_transferable: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    async fn batch_mint(&self, request: BatchMintRequest) -> NFTResult<Vec<NFT>> {
        let mut tokens = Vec::new();
        for mint_request in request.mint_requests {
            let token = self.mint_token(mint_request).await?;
            tokens.push(token);
        }
        Ok(tokens)
    }

    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Trading Service
#[async_trait]
pub trait TradingService: Send + Sync {
    async fn get_price_history(&self, contract_address: &str, token_id: &str) -> NFTResult<Vec<PriceHistoryEntry>>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockTradingService;

impl MockTradingService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TradingService for MockTradingService {
    async fn get_price_history(&self, contract_address: &str, token_id: &str) -> NFTResult<Vec<PriceHistoryEntry>> {
        Ok(vec![
            PriceHistoryEntry {
                token_id: token_id.to_string(),
                contract_address: contract_address.to_string(),
                price: rust_decimal::Decimal::new(1, 0),
                currency: "ETH".to_string(),
                transaction_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
                timestamp: Utc::now(),
            }
        ])
    }

    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Storage Service
#[async_trait]
pub trait StorageService: Send + Sync {
    async fn upload_metadata(&self, metadata: NFTMetadata) -> NFTResult<String>;
    async fn upload_image(&self, image_data: Vec<u8>) -> NFTResult<String>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockStorageService;

impl MockStorageService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StorageService for MockStorageService {
    async fn upload_metadata(&self, _metadata: NFTMetadata) -> NFTResult<String> {
        Ok("https://ipfs.io/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string())
    }

    async fn upload_image(&self, _image_data: Vec<u8>) -> NFTResult<String> {
        Ok("https://ipfs.io/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string())
    }

    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}

// Validation Service
#[async_trait]
pub trait ValidationService: Send + Sync {
    async fn validate_contract_address(&self, address: &str) -> NFTResult<()>;
    async fn validate_token_id(&self, token_id: &str) -> NFTResult<()>;
    async fn validate_address(&self, address: &str) -> NFTResult<()>;
    async fn validate_metadata(&self, metadata: &NFTMetadata) -> NFTResult<()>;
    async fn validate_create_collection_request(&self, request: &CreateCollectionRequest) -> NFTResult<()>;
    async fn validate_mint_request(&self, request: &MintRequest) -> NFTResult<()>;
    async fn validate_batch_mint_request(&self, request: &BatchMintRequest) -> NFTResult<()>;
    async fn validate_transfer_request(&self, request: &TransferRequest) -> NFTResult<()>;
    async fn validate_approval_request(&self, request: &ApprovalRequest) -> NFTResult<()>;
    async fn validate_create_listing_request(&self, request: &CreateListingRequest) -> NFTResult<()>;
    async fn validate_create_offer_request(&self, request: &CreateOfferRequest) -> NFTResult<()>;
    async fn health_check(&self) -> NFTResult<()>;
}

pub struct MockValidationService;

impl MockValidationService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ValidationService for MockValidationService {
    async fn validate_contract_address(&self, _address: &str) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_token_id(&self, _token_id: &str) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_address(&self, _address: &str) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_metadata(&self, _metadata: &NFTMetadata) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_create_collection_request(&self, _request: &CreateCollectionRequest) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_mint_request(&self, _request: &MintRequest) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_batch_mint_request(&self, _request: &BatchMintRequest) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_transfer_request(&self, _request: &TransferRequest) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_approval_request(&self, _request: &ApprovalRequest) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_create_listing_request(&self, _request: &CreateListingRequest) -> NFTResult<()> {
        Ok(())
    }

    async fn validate_create_offer_request(&self, _request: &CreateOfferRequest) -> NFTResult<()> {
        Ok(())
    }

    async fn health_check(&self) -> NFTResult<()> {
        Ok(())
    }
}
