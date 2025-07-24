// =====================================================================================
// File: core-layer2/src/zksync.rs
// Description: zkSync Layer 2 integration (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// zkSync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkSyncConfig {
    pub network: ZkSyncNetwork,
    pub rpc_url: String,
    pub chain_id: u64,
}

/// zkSync network types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZkSyncNetwork {
    Era,
    Lite,
    Testnet,
}

impl Default for ZkSyncConfig {
    fn default() -> Self {
        Self {
            network: ZkSyncNetwork::Testnet,
            rpc_url: "https://testnet.era.zksync.dev".to_string(),
            chain_id: 280,
        }
    }
}
