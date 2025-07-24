// =====================================================================================
// File: core-layer2/src/starknet.rs
// Description: StarkNet Layer 2 integration (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// StarkNet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarkNetConfig {
    pub network: StarkNetNetwork,
    pub rpc_url: String,
    pub chain_id: String,
}

/// StarkNet network types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StarkNetNetwork {
    Mainnet,
    Goerli,
    Sepolia,
}

impl Default for StarkNetConfig {
    fn default() -> Self {
        Self {
            network: StarkNetNetwork::Goerli,
            rpc_url: "https://alpha4.starknet.io".to_string(),
            chain_id: "SN_GOERLI".to_string(),
        }
    }
}
