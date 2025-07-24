// =====================================================================================
// File: core-blockchain/src/types.rs
// Description: Common blockchain types and data structures
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use ethers::types::H160;
use ethers::abi::{Tokenizable, Token as EthToken, Detokenize, InvalidOutputType};

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockchainNetwork {
    Ethereum,
    EthereumMainnet,
    EthereumTestnet,
    Solana,
    SolanaMainnet,
    SolanaDevnet,
    SolanaTestnet,
    Polkadot,
    PolkadotMainnet,
    PolkadotTestnet,
}

/// Type alias for backward compatibility
pub type Network = BlockchainNetwork;

/// Block number type
pub type BlockNumber = u64;

/// U256 type alias for large numbers (simplified for now)
pub type U256 = u64;

/// Transaction receipt type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: TransactionHash,
    pub block_number: BlockNumber,
    pub block_hash: String,
    pub transaction_index: u64,
    pub from: Address,
    pub to: Option<Address>,
    pub cumulative_gas_used: u64,
    pub gas_used: u64,
    pub contract_address: Option<Address>,
    pub logs: Vec<Log>,
    pub status: TransactionStatus,
    pub network: BlockchainNetwork,
}

/// Log entry in transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: BlockNumber,
    pub transaction_hash: TransactionHash,
    pub log_index: u64,
}

/// Contract argument type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractArg {
    String(String),
    Uint256(String),
    Address(Address),
    Bool(bool),
    Bytes(Vec<u8>),
    Array(Vec<ContractArg>),
}

/// Contract event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub name: String,
    pub signature: String,
    pub inputs: Vec<EventInput>,
    pub address: Address,
    pub block_number: BlockNumber,
    pub transaction_hash: TransactionHash,
    pub log_index: u64,
    pub data: HashMap<String, ContractArg>,
}

/// Event input parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInput {
    pub name: String,
    pub type_name: String,
    pub indexed: bool,
}

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub network: BlockchainNetwork,
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub confirmation_blocks: u32,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub enable_mempool_monitoring: bool,
    pub enable_event_listening: bool,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            network: BlockchainNetwork::Ethereum,
            rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: 1,
            gas_limit: 21000,
            gas_price: 20_000_000_000, // 20 gwei
            confirmation_blocks: 12,
            timeout_seconds: 300,
            retry_attempts: 3,
            enable_mempool_monitoring: false,
            enable_event_listening: true,
        }
    }
}

impl BlockchainNetwork {
    pub fn as_str(&self) -> &'static str {
        match self {
            BlockchainNetwork::Ethereum => "ethereum",
            BlockchainNetwork::EthereumMainnet => "ethereum_mainnet",
            BlockchainNetwork::EthereumTestnet => "ethereum_testnet",
            BlockchainNetwork::Solana => "solana",
            BlockchainNetwork::SolanaMainnet => "solana_mainnet",
            BlockchainNetwork::SolanaDevnet => "solana_devnet",
            BlockchainNetwork::SolanaTestnet => "solana_testnet",
            BlockchainNetwork::Polkadot => "polkadot",
            BlockchainNetwork::PolkadotMainnet => "polkadot_mainnet",
            BlockchainNetwork::PolkadotTestnet => "polkadot_testnet",
        }
    }

    pub fn is_mainnet(&self) -> bool {
        matches!(self,
            BlockchainNetwork::Ethereum |
            BlockchainNetwork::EthereumMainnet |
            BlockchainNetwork::Solana |
            BlockchainNetwork::SolanaMainnet |
            BlockchainNetwork::Polkadot |
            BlockchainNetwork::PolkadotMainnet
        )
    }
}

/// Generic blockchain address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    pub value: String,
    pub network: BlockchainNetwork,
}

// Note: Address cannot implement Copy because it contains String

impl Address {
    pub fn new(value: String, network: BlockchainNetwork) -> Self {
        Self { value, network }
    }

    pub fn ethereum(value: String) -> Self {
        Self::new(value, BlockchainNetwork::Ethereum)
    }

    pub fn solana(value: String) -> Self {
        Self::new(value, BlockchainNetwork::Solana)
    }

    pub fn polkadot(value: String) -> Self {
        Self::new(value, BlockchainNetwork::Polkadot)
    }

    /// Validates the address format according to the blockchain network
    ///
    /// # Returns
    ///
    /// `true` if the address is valid for the specified network, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use core_blockchain::types::{Address, BlockchainNetwork};
    ///
    /// let eth_addr = Address::ethereum("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string());
    /// assert!(eth_addr.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        match self.network {
            BlockchainNetwork::Ethereum | BlockchainNetwork::EthereumTestnet | BlockchainNetwork::EthereumMainnet => {
                self.value.starts_with("0x") && self.value.len() == 42
            }
            BlockchainNetwork::Solana | BlockchainNetwork::SolanaDevnet | BlockchainNetwork::SolanaMainnet | BlockchainNetwork::SolanaTestnet => {
                self.value.len() >= 32 && self.value.len() <= 44
            }
            BlockchainNetwork::Polkadot | BlockchainNetwork::PolkadotTestnet | BlockchainNetwork::PolkadotMainnet => {
                self.value.len() >= 47 && self.value.len() <= 48
            }
        }
    }

    /// Converts the address to an Ethereum H160 address
    ///
    /// # Errors
    ///
    /// Returns an error if the address is not a valid Ethereum address
    pub fn to_h160(&self) -> Result<H160, String> {
        if !matches!(self.network,
            BlockchainNetwork::Ethereum |
            BlockchainNetwork::EthereumTestnet |
            BlockchainNetwork::EthereumMainnet
        ) {
            return Err("Address is not an Ethereum address".to_string());
        }

        self.value.parse::<H160>()
            .map_err(|e| format!("Invalid Ethereum address format: {}", e))
    }
}

// Enterprise-grade trait implementations for Address
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Default to Ethereum for parsing from string
        let address = Self::ethereum(s.to_string());
        if address.is_valid() {
            Ok(address)
        } else {
            Err(format!("Invalid address format: {}", s))
        }
    }
}

impl From<H160> for Address {
    fn from(h160: H160) -> Self {
        Self::ethereum(format!("{:?}", h160))
    }
}

impl From<Address> for H160 {
    fn from(addr: Address) -> Self {
        addr.to_h160().unwrap_or_default()
    }
}

impl Tokenizable for Address {
    fn from_token(token: EthToken) -> Result<Self, InvalidOutputType> {
        match token {
            EthToken::Address(addr) => Ok(Self::from(addr)),
            _ => Err(InvalidOutputType("Expected address token".to_string())),
        }
    }

    fn into_token(self) -> EthToken {
        EthToken::Address(self.into())
    }
}

// Detokenize is automatically implemented for types that implement Tokenizable

/// Transaction hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionHash {
    pub value: String,
    pub network: BlockchainNetwork,
}

impl fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl FromStr for TransactionHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Default to Ethereum for parsing from string
        Ok(Self {
            value: s.to_string(),
            network: BlockchainNetwork::Ethereum,
        })
    }
}

impl From<ethers::types::H256> for TransactionHash {
    fn from(h256: ethers::types::H256) -> Self {
        Self {
            value: format!("{:?}", h256),
            network: BlockchainNetwork::Ethereum,
        }
    }
}

impl From<TransactionHash> for String {
    fn from(hash: TransactionHash) -> Self {
        hash.value
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Dropped,
    Cancelled,
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
    // Additional fields for compatibility
    pub id: String,
    pub from_address: Address,
    pub to_address: Address,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    // Blockchain-specific fields
    pub gas_limit: u64,
    pub gas_price: u64,
    pub nonce: u64,
    pub data: Vec<u8>,
}

/// Account balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub address: Address,
    pub amount: u64,
    pub token_balances: HashMap<String, u64>, // token_address -> balance
    pub last_updated: DateTime<Utc>,
    pub currency: String,
    pub network: BlockchainNetwork,
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
    pub transactions: Vec<TransactionHash>,
    pub gas_used: u64,
    pub gas_limit: u64,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
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
