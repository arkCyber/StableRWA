// =====================================================================================
// File: core-blockchain/src/types.rs
// Description: Common blockchain types and data structures
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported blockchain networks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockchainNetwork {
    Ethereum,
    Solana,
    Polkadot,
    EthereumTestnet,
    SolanaDevnet,
    PolkadotTestnet,
}

impl BlockchainNetwork {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockchainNetwork::Ethereum => "ethereum",
            BlockchainNetwork::Solana => "solana",
            BlockchainNetwork::Polkadot => "polkadot",
            BlockchainNetwork::EthereumTestnet => "ethereum_testnet",
            BlockchainNetwork::SolanaDevnet => "solana_devnet",
            BlockchainNetwork::PolkadotTestnet => "polkadot_testnet",
        }
    }
    
    pub fn is_mainnet(&self) -> bool {
        matches!(self, 
            BlockchainNetwork::Ethereum | 
            BlockchainNetwork::Solana | 
            BlockchainNetwork::Polkadot
        )
    }
}

/// Generic blockchain address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    pub value: String,
    pub network: BlockchainNetwork,
}

impl Address {
    pub fn new(value: String, network: BlockchainNetwork) -> Self {
        Self { value, network }
    }
    
    pub fn is_valid(&self) -> bool {
        match self.network {
            BlockchainNetwork::Ethereum | BlockchainNetwork::EthereumTestnet => {
                self.value.starts_with("0x") && self.value.len() == 42
            }
            BlockchainNetwork::Solana | BlockchainNetwork::SolanaDevnet => {
                self.value.len() >= 32 && self.value.len() <= 44
            }
            BlockchainNetwork::Polkadot | BlockchainNetwork::PolkadotTestnet => {
                self.value.len() >= 47 && self.value.len() <= 48
            }
        }
    }
}

/// Transaction hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionHash {
    pub value: String,
    pub network: BlockchainNetwork,
}

impl std::fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Dropped,
}

/// Generic blockchain transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: TransactionHash,
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: u64,
    pub status: TransactionStatus,
    pub block_number: Option<u64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub confirmations: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Account balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub address: Address,
    pub amount: u64,
    pub token_balances: HashMap<String, u64>, // token_address -> balance
    pub last_updated: DateTime<Utc>,
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: DateTime<Utc>,
    pub transaction_count: u32,
    pub network: BlockchainNetwork,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: Address,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: Option<u64>,
    pub network: BlockchainNetwork,
}

/// Smart contract interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract_address: Address,
    pub function_name: String,
    pub parameters: Vec<serde_json::Value>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
}

/// Asset tokenization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizedAsset {
    pub asset_id: String,
    pub token_address: Address,
    pub total_tokens: u64,
    pub token_symbol: String,
    pub token_name: String,
    pub decimals: u8,
    pub metadata_uri: Option<String>,
    pub created_at: DateTime<Utc>,
    pub network: BlockchainNetwork,
}

/// Wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub address: Address,
    pub balance: Balance,
    pub nonce: Option<u64>,
    pub is_contract: bool,
    pub last_activity: Option<DateTime<Utc>>,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub network: BlockchainNetwork,
    pub latest_block: u64,
    pub average_block_time: f64, // seconds
    pub gas_price: Option<u64>,
    pub total_transactions: u64,
    pub active_addresses: u64,
    pub last_updated: DateTime<Utc>,
}

/// Transaction fee estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub network: BlockchainNetwork,
    pub slow: u64,
    pub standard: u64,
    pub fast: u64,
    pub estimated_time_seconds: HashMap<String, u64>, // fee_level -> time
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_address_validation() {
        let eth_addr = Address::new("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(), BlockchainNetwork::Ethereum);
        assert!(eth_addr.is_valid());
        
        let invalid_eth_addr = Address::new("invalid".to_string(), BlockchainNetwork::Ethereum);
        assert!(!invalid_eth_addr.is_valid());
        
        let sol_addr = Address::new("11111111111111111111111111111112".to_string(), BlockchainNetwork::Solana);
        assert!(sol_addr.is_valid());
    }
    
    #[test]
    fn test_network_properties() {
        assert_eq!(BlockchainNetwork::Ethereum.as_str(), "ethereum");
        assert!(BlockchainNetwork::Ethereum.is_mainnet());
        assert!(!BlockchainNetwork::EthereumTestnet.is_mainnet());
    }
    
    #[test]
    fn test_transaction_hash() {
        let tx_hash = TransactionHash {
            value: "0x123".to_string(),
            network: BlockchainNetwork::Ethereum,
        };
        assert_eq!(tx_hash.value, "0x123");
        assert_eq!(tx_hash.network, BlockchainNetwork::Ethereum);
    }
}
