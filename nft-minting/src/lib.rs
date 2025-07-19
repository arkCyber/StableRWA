// =====================================================================================
// File: nft-minting/src/lib.rs
// Description: Core NFT minting logic for enterprise-grade NFT microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// NFT metadata structure (ERC-721 compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nft {
    pub id: u64,
    pub owner: String,
    pub metadata_uri: String,
    pub minted_at: String,
}

/// NFT minting error type
#[derive(Debug, Error)]
pub enum NftError {
    #[error("Owner address is empty")] 
    EmptyOwner,
    #[error("Metadata URI is empty")] 
    EmptyMetadata,
    #[error("NFT not found")] 
    NotFound,
    #[error("Internal error: {0}")]
    Internal(String),
}

/// NFT storage (in-memory, for demo; replace with DB in production)
#[derive(Default, Clone)]
pub struct NftStore {
    pub nfts: Arc<Mutex<HashMap<u64, Nft>>>,
    pub next_id: Arc<Mutex<u64>>,
}

impl NftStore {
    /// Creates a new NFT store
    pub fn new() -> Self {
        Self::default()
    }

    /// Mints a new NFT
    pub fn mint_nft(&self, owner: String, metadata_uri: String) -> Result<Nft, NftError> {
        let now = Utc::now();
        if owner.trim().is_empty() {
            error!("{} - [NftStore] Mint failed: owner empty", now);
            return Err(NftError::EmptyOwner);
        }
        if metadata_uri.trim().is_empty() {
            error!("{} - [NftStore] Mint failed: metadata_uri empty", now);
            return Err(NftError::EmptyMetadata);
        }
        let mut id_lock = self.next_id.lock().unwrap();
        let id = *id_lock;
        *id_lock += 1;
        let nft = Nft {
            id,
            owner: owner.clone(),
            metadata_uri: metadata_uri.clone(),
            minted_at: now.to_rfc3339(),
        };
        self.nfts.lock().unwrap().insert(id, nft.clone());
        info!("{} - [NftStore] Minted NFT id={} owner={}", now, id, owner);
        Ok(nft)
    }

    /// Gets an NFT by id
    pub fn get_nft(&self, id: u64) -> Result<Nft, NftError> {
        let now = Utc::now();
        let nfts = self.nfts.lock().unwrap();
        nfts.get(&id).cloned().ok_or_else(|| {
            error!("{} - [NftStore] NFT not found: {}", now, id);
            NftError::NotFound
        })
    }

    /// Gets all NFTs owned by a given address
    pub fn get_nfts_by_owner(&self, owner: &str) -> Vec<Nft> {
        let nfts = self.nfts.lock().unwrap();
        nfts.values().filter(|n| n.owner == owner).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Info).try_init();
    }

    #[test]
    fn test_mint_and_get_nft() {
        init_logger();
        let store = NftStore::new();
        let nft = store.mint_nft("0xabc".to_string(), "ipfs://meta1".to_string()).unwrap();
        let fetched = store.get_nft(nft.id).unwrap();
        assert_eq!(nft.id, fetched.id);
        assert_eq!(nft.owner, fetched.owner);
    }

    #[test]
    fn test_get_nfts_by_owner() {
        init_logger();
        let store = NftStore::new();
        store.mint_nft("0xabc".to_string(), "ipfs://meta1".to_string()).unwrap();
        store.mint_nft("0xabc".to_string(), "ipfs://meta2".to_string()).unwrap();
        store.mint_nft("0xdef".to_string(), "ipfs://meta3".to_string()).unwrap();
        let nfts = store.get_nfts_by_owner("0xabc");
        assert_eq!(nfts.len(), 2);
    }

    #[test]
    fn test_mint_error() {
        init_logger();
        let store = NftStore::new();
        assert!(store.mint_nft("".to_string(), "meta".to_string()).is_err());
        assert!(store.mint_nft("0xabc".to_string(), "".to_string()).is_err());
    }
} 