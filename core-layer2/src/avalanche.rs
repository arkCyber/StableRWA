// =====================================================================================
// File: core-layer2/src/avalanche.rs
// Description: Avalanche subnet integration (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Avalanche configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvalancheConfig {
    pub network: AvalancheNetwork,
    pub rpc_url: String,
    pub chain_id: u64,
}

/// Avalanche network types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AvalancheNetwork {
    Mainnet,
    Fuji,
    Subnet,
}

impl Default for AvalancheConfig {
    fn default() -> Self {
        Self {
            network: AvalancheNetwork::Fuji,
            rpc_url: "https://api.avax-test.network/ext/bc/C/rpc".to_string(),
            chain_id: 43113,
        }
    }
}
