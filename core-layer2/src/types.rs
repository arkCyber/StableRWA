// =====================================================================================
// File: core-layer2/src/types.rs
// Description: Common types for Layer 2 services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;

/// Layer 2 network enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer2Network {
    Polygon,
    Arbitrum,
    Optimism,
    Base,
    ZkSync,
    StarkNet,
    Avalanche,
}

impl Layer2Network {
    pub fn name(&self) -> &'static str {
        match self {
            Layer2Network::Polygon => "Polygon",
            Layer2Network::Arbitrum => "Arbitrum",
            Layer2Network::Optimism => "Optimism",
            Layer2Network::Base => "Base",
            Layer2Network::ZkSync => "zkSync",
            Layer2Network::StarkNet => "StarkNet",
            Layer2Network::Avalanche => "Avalanche",
        }
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            Layer2Network::Polygon => 137,
            Layer2Network::Arbitrum => 42161,
            Layer2Network::Optimism => 10,
            Layer2Network::Base => 8453,
            Layer2Network::ZkSync => 324,
            Layer2Network::StarkNet => 0, // StarkNet uses different addressing
            Layer2Network::Avalanche => 43114,
        }
    }
}

/// Cross-chain message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    pub id: Uuid,
    pub source_network: Layer2Network,
    pub destination_network: Layer2Network,
    pub sender: String,
    pub recipient: String,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub created_at: DateTime<Utc>,
    pub status: MessageStatus,
}

/// Message status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Relayed,
    Executed,
    Failed,
}

/// Bridge transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: Uuid,
    pub user_id: String,
    pub source_network: Layer2Network,
    pub destination_network: Layer2Network,
    pub token_address: String,
    pub amount: Decimal,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub status: BridgeStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Bridge status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeStatus {
    Initiated,
    Locked,
    Relayed,
    Minted,
    Completed,
    Failed,
}

/// State update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub id: Uuid,
    pub network: Layer2Network,
    pub block_number: u64,
    pub state_root: String,
    pub timestamp: DateTime<Utc>,
    pub transactions_count: u32,
}

/// Gas estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub network: Layer2Network,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub total_cost: Decimal,
    pub estimated_time_seconds: u32,
}

/// Network status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub network: Layer2Network,
    pub is_online: bool,
    pub block_height: u64,
    pub gas_price: u64,
    pub tps: f64,
    pub last_updated: DateTime<Utc>,
}

/// Chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub network: Layer2Network,
    pub rpc_url: String,
    pub chain_id: u64,
    pub block_explorer_url: String,
    pub native_token_symbol: String,
    pub bridge_contract_address: String,
    pub is_testnet: bool,
}
