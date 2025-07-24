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
pub mod service;
pub mod erc721;
pub mod mock_services;

// Re-export main types and traits
pub use error::{NFTError, NFTResult};
pub use types::*;
pub use service::NFTService;
pub use erc721::{ERC721Service, ERC721ServiceImpl, MockERC721Service};
pub use mock_services::*;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main NFT service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTServiceConfig {
    pub network: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_price_gwei: Decimal,
    pub max_gas_limit: u64,
    pub confirmation_blocks: u64,
    pub ipfs_gateway: String,
    pub ipfs_api_url: String,
    pub storage_provider: StorageProvider,
    pub marketplace_fee_percentage: Decimal,
    pub max_royalty_percentage: Decimal,
    pub enable_batch_operations: bool,
    pub enable_lazy_minting: bool,
    pub enable_gasless_transactions: bool,
    pub rate_limit_per_minute: u64,
    pub max_file_size_mb: u64,
    pub supported_image_formats: Vec<String>,
    pub metadata_cache_ttl_seconds: u64,
}

impl Default for NFTServiceConfig {
    fn default() -> Self {
        Self {
            network: "ethereum".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: 1,
            gas_price_gwei: Decimal::new(20, 0),
            max_gas_limit: 500000,
            confirmation_blocks: 12,
            ipfs_gateway: "https://ipfs.io/ipfs/".to_string(),
            ipfs_api_url: "https://api.pinata.cloud".to_string(),
            storage_provider: StorageProvider::IPFS,
            marketplace_fee_percentage: Decimal::new(25, 3), // 2.5%
            max_royalty_percentage: Decimal::new(10, 2), // 10%
            enable_batch_operations: true,
            enable_lazy_minting: true,
            enable_gasless_transactions: false,
            rate_limit_per_minute: 100,
            max_file_size_mb: 100,
            supported_image_formats: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "image/svg+xml".to_string(),
            ],
            metadata_cache_ttl_seconds: 3600, // 1 hour
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_config_default() {
        let config = NFTServiceConfig::default();
        assert_eq!(config.network, "ethereum");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.gas_price_gwei, Decimal::new(20, 0));
        assert_eq!(config.max_gas_limit, 500000);
        assert_eq!(config.confirmation_blocks, 12);
        assert_eq!(config.storage_provider, StorageProvider::IPFS);
        assert_eq!(config.marketplace_fee_percentage, Decimal::new(25, 3));
        assert_eq!(config.max_royalty_percentage, Decimal::new(10, 2));
        assert!(config.enable_batch_operations);
        assert!(config.enable_lazy_minting);
        assert!(!config.enable_gasless_transactions);
        assert_eq!(config.rate_limit_per_minute, 100);
        assert_eq!(config.max_file_size_mb, 100);
        assert_eq!(config.supported_image_formats.len(), 5);
        assert!(config.supported_image_formats.contains(&"image/png".to_string()));
        assert_eq!(config.metadata_cache_ttl_seconds, 3600);
    }

    #[test]
    fn test_storage_provider_enum() {
        let provider = StorageProvider::IPFS;
        assert_eq!(provider, StorageProvider::IPFS);

        let provider = StorageProvider::Arweave;
        assert_eq!(provider, StorageProvider::Arweave);
    }

    #[test]
    fn test_nft_standard_enum() {
        let standard = NFTStandard::ERC721;
        assert_eq!(standard, NFTStandard::ERC721);

        let standard = NFTStandard::ERC1155;
        assert_eq!(standard, NFTStandard::ERC1155);
    }
}
