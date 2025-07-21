// =====================================================================================
// File: core-nft/src/lib.rs
// Description: NFT and metadata management for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core NFT Module
//! 
//! This module provides comprehensive NFT (Non-Fungible Token) and metadata management
//! services for the StableRWA platform, supporting ERC-721, ERC-1155, and various
//! metadata standards with IPFS integration.

pub mod error;
pub mod types;
pub mod erc721;
pub mod erc1155;
pub mod metadata;
pub mod ipfs;
pub mod marketplace;
pub mod royalties;
pub mod collections;
pub mod minting;
pub mod trading;
pub mod storage;
pub mod validation;
pub mod service;

// Re-export main types and traits
pub use error::{NFTError, NFTResult};
pub use types::{
    NFT, NFTCollection, NFTMetadata, NFTAttribute, NFTStandard,
    TokenId, ContractAddress, Owner, Creator, Royalty
};
pub use erc721::{
    ERC721Service, ERC721Token, ERC721Metadata,
    ERC721Transfer, ERC721Approval
};
pub use erc1155::{
    ERC1155Service, ERC1155Token, ERC1155Metadata,
    ERC1155Transfer, ERC1155Batch
};
pub use metadata::{
    MetadataService, MetadataStandard, MetadataValidator,
    OpenSeaMetadata, JSONMetadata, OnChainMetadata
};
pub use ipfs::{
    IPFSService, IPFSClient, IPFSHash,
    IPFSUpload, IPFSDownload, IPFSPin
};
pub use marketplace::{
    MarketplaceService, Listing, Offer, Sale,
    MarketplaceConfig, MarketplaceFee
};
pub use royalties::{
    RoyaltyService, RoyaltyInfo, RoyaltyDistribution,
    EIP2981Royalty, CustomRoyalty
};
pub use collections::{
    CollectionService, CollectionMetadata, CollectionStats,
    CollectionVerification, CollectionRoyalty
};
pub use minting::{
    MintingService, MintRequest, MintResult,
    BatchMint, LazyMint, MintingPolicy
};
pub use trading::{
    TradingService, TradeOrder, TradeExecution,
    OrderBook, PriceHistory
};
pub use storage::{
    StorageService, DecentralizedStorage, CentralizedStorage,
    StorageProvider, StorageBackup
};
pub use validation::{
    ValidationService, MetadataValidator as MetaValidator,
    ImageValidator, ContentValidator
};
pub use service::NFTService;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main NFT service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTServiceConfig {
    /// ERC-721 configuration
    pub erc721_config: erc721::ERC721Config,
    /// ERC-1155 configuration
    pub erc1155_config: erc1155::ERC1155Config,
    /// Metadata configuration
    pub metadata_config: metadata::MetadataConfig,
    /// IPFS configuration
    pub ipfs_config: ipfs::IPFSConfig,
    /// Marketplace configuration
    pub marketplace_config: marketplace::MarketplaceConfig,
    /// Storage configuration
    pub storage_config: storage::StorageConfig,
    /// Global NFT settings
    pub global_settings: GlobalNFTSettings,
}

impl Default for NFTServiceConfig {
    fn default() -> Self {
        Self {
            erc721_config: erc721::ERC721Config::default(),
            erc1155_config: erc1155::ERC1155Config::default(),
            metadata_config: metadata::MetadataConfig::default(),
            ipfs_config: ipfs::IPFSConfig::default(),
            marketplace_config: marketplace::MarketplaceConfig::default(),
            storage_config: storage::StorageConfig::default(),
            global_settings: GlobalNFTSettings::default(),
        }
    }
}

/// Global NFT settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalNFTSettings {
    /// Default metadata standard
    pub default_metadata_standard: MetadataStandard,
    /// Enable IPFS storage
    pub enable_ipfs_storage: bool,
    /// Enable metadata validation
    pub enable_metadata_validation: bool,
    /// Enable image validation
    pub enable_image_validation: bool,
    /// Maximum file size in MB
    pub max_file_size_mb: u32,
    /// Supported image formats
    pub supported_image_formats: Vec<String>,
    /// Enable royalty enforcement
    pub enable_royalty_enforcement: bool,
    /// Default royalty percentage
    pub default_royalty_percentage: Decimal,
    /// Enable marketplace integration
    pub enable_marketplace_integration: bool,
    /// Enable batch operations
    pub enable_batch_operations: bool,
}

impl Default for GlobalNFTSettings {
    fn default() -> Self {
        Self {
            default_metadata_standard: MetadataStandard::OpenSea,
            enable_ipfs_storage: true,
            enable_metadata_validation: true,
            enable_image_validation: true,
            max_file_size_mb: 100,
            supported_image_formats: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "webp".to_string(),
            ],
            enable_royalty_enforcement: true,
            default_royalty_percentage: Decimal::new(250, 4), // 2.5%
            enable_marketplace_integration: true,
            enable_batch_operations: true,
        }
    }
}

/// NFT metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTMetrics {
    pub total_nfts: u64,
    pub total_collections: u64,
    pub total_owners: u64,
    pub total_volume_24h: Decimal,
    pub total_sales_24h: u64,
    pub average_price_24h: Decimal,
    pub floor_price_change_24h: Decimal,
    pub mints_24h: u64,
    pub transfers_24h: u64,
    pub marketplace_volume_24h: Decimal,
    pub royalties_paid_24h: Decimal,
    pub standard_breakdown: HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

/// NFT health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTHealthStatus {
    pub overall_status: String,
    pub erc721_status: String,
    pub erc1155_status: String,
    pub metadata_status: String,
    pub ipfs_status: String,
    pub marketplace_status: String,
    pub storage_status: String,
    pub validation_status: String,
    pub last_check: DateTime<Utc>,
}

// Stub modules for compilation
pub mod erc721 {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ERC721Config {
        pub default_gas_limit: u64,
        pub enable_enumerable: bool,
        pub enable_metadata: bool,
    }
    
    impl Default for ERC721Config {
        fn default() -> Self {
            Self {
                default_gas_limit: 200000,
                enable_enumerable: true,
                enable_metadata: true,
            }
        }
    }
    
    pub struct ERC721Service;
    pub struct ERC721Token;
    pub struct ERC721Metadata;
    pub struct ERC721Transfer;
    pub struct ERC721Approval;
}

pub mod erc1155 {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ERC1155Config {
        pub default_gas_limit: u64,
        pub enable_batch_operations: bool,
        pub max_batch_size: u32,
    }
    
    impl Default for ERC1155Config {
        fn default() -> Self {
            Self {
                default_gas_limit: 300000,
                enable_batch_operations: true,
                max_batch_size: 100,
            }
        }
    }
    
    pub struct ERC1155Service;
    pub struct ERC1155Token;
    pub struct ERC1155Metadata;
    pub struct ERC1155Transfer;
    pub struct ERC1155Batch;
}

pub mod metadata {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MetadataConfig {
        pub enable_validation: bool,
        pub max_attributes: u32,
        pub enable_caching: bool,
    }
    
    impl Default for MetadataConfig {
        fn default() -> Self {
            Self {
                enable_validation: true,
                max_attributes: 100,
                enable_caching: true,
            }
        }
    }
    
    pub struct MetadataService;
    pub struct MetadataValidator;
    pub struct OpenSeaMetadata;
    pub struct JSONMetadata;
    pub struct OnChainMetadata;
}

pub mod ipfs {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IPFSConfig {
        pub node_url: String,
        pub api_key: Option<String>,
        pub enable_pinning: bool,
    }
    
    impl Default for IPFSConfig {
        fn default() -> Self {
            Self {
                node_url: "https://ipfs.infura.io:5001".to_string(),
                api_key: None,
                enable_pinning: true,
            }
        }
    }
    
    pub struct IPFSService;
    pub struct IPFSClient;
    pub struct IPFSHash;
    pub struct IPFSUpload;
    pub struct IPFSDownload;
    pub struct IPFSPin;
}

pub mod marketplace {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MarketplaceConfig {
        pub platform_fee_percentage: Decimal,
        pub min_listing_price: Decimal,
        pub max_listing_duration_days: u32,
    }
    
    impl Default for MarketplaceConfig {
        fn default() -> Self {
            Self {
                platform_fee_percentage: Decimal::new(250, 4), // 2.5%
                min_listing_price: Decimal::new(1, 18), // 0.001 ETH
                max_listing_duration_days: 365,
            }
        }
    }
    
    pub struct MarketplaceService;
    pub struct Listing;
    pub struct Offer;
    pub struct Sale;
    pub struct MarketplaceFee;
}

pub mod royalties {
    use super::*;
    
    pub struct RoyaltyService;
    pub struct RoyaltyInfo;
    pub struct RoyaltyDistribution;
    pub struct EIP2981Royalty;
    pub struct CustomRoyalty;
}

pub mod collections {
    use super::*;
    
    pub struct CollectionService;
    pub struct CollectionMetadata;
    pub struct CollectionStats;
    pub struct CollectionVerification;
    pub struct CollectionRoyalty;
}

pub mod minting {
    use super::*;
    
    pub struct MintingService;
    pub struct MintRequest;
    pub struct MintResult;
    pub struct BatchMint;
    pub struct LazyMint;
    pub struct MintingPolicy;
}

pub mod trading {
    use super::*;
    
    pub struct TradingService;
    pub struct TradeOrder;
    pub struct TradeExecution;
    pub struct OrderBook;
    pub struct PriceHistory;
}

pub mod storage {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StorageConfig {
        pub primary_provider: String,
        pub backup_providers: Vec<String>,
        pub enable_redundancy: bool,
    }
    
    impl Default for StorageConfig {
        fn default() -> Self {
            Self {
                primary_provider: "IPFS".to_string(),
                backup_providers: vec!["Arweave".to_string(), "Filecoin".to_string()],
                enable_redundancy: true,
            }
        }
    }
    
    pub struct StorageService;
    pub struct DecentralizedStorage;
    pub struct CentralizedStorage;
    pub struct StorageProvider;
    pub struct StorageBackup;
}

pub mod validation {
    use super::*;
    
    pub struct ValidationService;
    pub struct ImageValidator;
    pub struct ContentValidator;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_config_default() {
        let config = NFTServiceConfig::default();
        assert!(config.erc721_config.enable_enumerable);
        assert!(config.erc721_config.enable_metadata);
        assert!(config.erc1155_config.enable_batch_operations);
        assert_eq!(config.erc1155_config.max_batch_size, 100);
        assert!(config.global_settings.enable_ipfs_storage);
        assert!(config.global_settings.enable_metadata_validation);
    }

    #[test]
    fn test_global_nft_settings() {
        let settings = GlobalNFTSettings::default();
        assert_eq!(settings.default_metadata_standard, MetadataStandard::OpenSea);
        assert_eq!(settings.max_file_size_mb, 100);
        assert_eq!(settings.supported_image_formats.len(), 6);
        assert!(settings.supported_image_formats.contains(&"png".to_string()));
        assert_eq!(settings.default_royalty_percentage, Decimal::new(250, 4));
    }

    #[test]
    fn test_marketplace_config() {
        let config = marketplace::MarketplaceConfig::default();
        assert_eq!(config.platform_fee_percentage, Decimal::new(250, 4));
        assert_eq!(config.min_listing_price, Decimal::new(1, 18));
        assert_eq!(config.max_listing_duration_days, 365);
    }
}
