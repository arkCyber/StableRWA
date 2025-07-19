// =====================================================================================
// File: core-blockchain/src/traits.rs
// Description: Blockchain adapter traits and interfaces
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::types::*;
use crate::BlockchainError;
use async_trait::async_trait;
use std::collections::HashMap;

/// Core blockchain adapter trait
#[async_trait]
pub trait BlockchainAdapter: Send + Sync {
    /// Connect to the blockchain network
    async fn connect(&self) -> Result<(), BlockchainError>;
    
    /// Disconnect from the blockchain network
    async fn disconnect(&self) -> Result<(), BlockchainError>;
    
    /// Check if connected to the blockchain network
    async fn is_connected(&self) -> bool;
    
    /// Get the current network information
    fn network(&self) -> BlockchainNetwork;
    
    /// Get network statistics
    async fn get_network_stats(&self) -> Result<NetworkStats, BlockchainError>;
}

/// Wallet operations trait
#[async_trait]
pub trait WalletOperations: BlockchainAdapter {
    /// Get balance for an address
    async fn get_balance(&self, address: &Address) -> Result<Balance, BlockchainError>;
    
    /// Get multiple balances in a single call
    async fn get_balances(&self, addresses: &[Address]) -> Result<Vec<Balance>, BlockchainError>;
    
    /// Get wallet information
    async fn get_wallet_info(&self, address: &Address) -> Result<Wallet, BlockchainError>;
    
    /// Validate an address format
    fn validate_address(&self, address: &str) -> bool;
    
    /// Generate a new address (if supported)
    async fn generate_address(&self) -> Result<Address, BlockchainError>;
}

/// Transaction operations trait
#[async_trait]
pub trait TransactionOperations: BlockchainAdapter {
    /// Send a transaction
    async fn send_transaction(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
        options: Option<TransactionOptions>,
    ) -> Result<TransactionHash, BlockchainError>;
    
    /// Get transaction by hash
    async fn get_transaction(&self, hash: &TransactionHash) -> Result<Option<Transaction>, BlockchainError>;
    
    /// Get transaction status
    async fn get_transaction_status(&self, hash: &TransactionHash) -> Result<TransactionStatus, BlockchainError>;
    
    /// Get transaction history for an address
    async fn get_transaction_history(
        &self,
        address: &Address,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Transaction>, BlockchainError>;
    
    /// Estimate transaction fee
    async fn estimate_fee(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
    ) -> Result<FeeEstimate, BlockchainError>;
    
    /// Wait for transaction confirmation
    async fn wait_for_confirmation(
        &self,
        hash: &TransactionHash,
        confirmations: u32,
        timeout_seconds: u64,
    ) -> Result<Transaction, BlockchainError>;
}

/// Smart contract operations trait
#[async_trait]
pub trait ContractOperations: BlockchainAdapter {
    /// Deploy a smart contract
    async fn deploy_contract(
        &self,
        bytecode: &[u8],
        constructor_args: &[serde_json::Value],
        from: &Address,
        options: Option<TransactionOptions>,
    ) -> Result<Address, BlockchainError>;
    
    /// Call a contract function (read-only)
    async fn call_contract(
        &self,
        contract_call: &ContractCall,
    ) -> Result<serde_json::Value, BlockchainError>;
    
    /// Execute a contract function (state-changing)
    async fn execute_contract(
        &self,
        contract_call: &ContractCall,
        from: &Address,
        options: Option<TransactionOptions>,
    ) -> Result<TransactionHash, BlockchainError>;
    
    /// Get contract information
    async fn get_contract_info(&self, address: &Address) -> Result<ContractInfo, BlockchainError>;
}

/// Token operations trait
#[async_trait]
pub trait TokenOperations: BlockchainAdapter {
    /// Create/mint a new token
    async fn create_token(
        &self,
        token_info: &TokenCreationInfo,
        from: &Address,
        options: Option<TransactionOptions>,
    ) -> Result<Address, BlockchainError>;
    
    /// Get token information
    async fn get_token_info(&self, token_address: &Address) -> Result<Token, BlockchainError>;
    
    /// Get token balance for an address
    async fn get_token_balance(
        &self,
        token_address: &Address,
        holder_address: &Address,
    ) -> Result<u64, BlockchainError>;
    
    /// Transfer tokens
    async fn transfer_token(
        &self,
        token_address: &Address,
        from: &Address,
        to: &Address,
        amount: u64,
        options: Option<TransactionOptions>,
    ) -> Result<TransactionHash, BlockchainError>;
    
    /// Get token holders
    async fn get_token_holders(
        &self,
        token_address: &Address,
        limit: Option<u32>,
    ) -> Result<Vec<TokenHolder>, BlockchainError>;
}

/// Asset tokenization trait
#[async_trait]
pub trait AssetTokenization: TokenOperations {
    /// Tokenize a real-world asset
    async fn tokenize_asset(
        &self,
        asset_info: &AssetTokenizationInfo,
        from: &Address,
        options: Option<TransactionOptions>,
    ) -> Result<TokenizedAsset, BlockchainError>;
    
    /// Update asset metadata
    async fn update_asset_metadata(
        &self,
        token_address: &Address,
        metadata_uri: &str,
        from: &Address,
        options: Option<TransactionOptions>,
    ) -> Result<TransactionHash, BlockchainError>;
    
    /// Fractionalize asset (split into smaller tokens)
    async fn fractionalize_asset(
        &self,
        token_address: &Address,
        total_fractions: u64,
        from: &Address,
        options: Option<TransactionOptions>,
    ) -> Result<TransactionHash, BlockchainError>;
    
    /// Get asset tokenization history
    async fn get_tokenization_history(
        &self,
        asset_id: &str,
    ) -> Result<Vec<TokenizedAsset>, BlockchainError>;
}

/// Block and event monitoring trait
#[async_trait]
pub trait BlockchainMonitor: BlockchainAdapter {
    /// Get latest block number
    async fn get_latest_block_number(&self) -> Result<u64, BlockchainError>;
    
    /// Get block by number
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>, BlockchainError>;
    
    /// Subscribe to new blocks
    async fn subscribe_to_blocks(&self) -> Result<BlockSubscription, BlockchainError>;
    
    /// Subscribe to address events
    async fn subscribe_to_address_events(
        &self,
        address: &Address,
    ) -> Result<AddressSubscription, BlockchainError>;
    
    /// Get events for a contract
    async fn get_contract_events(
        &self,
        contract_address: &Address,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> Result<Vec<ContractEvent>, BlockchainError>;
}

/// Additional types for trait implementations

#[derive(Debug, Clone)]
pub struct TransactionOptions {
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
    pub nonce: Option<u64>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub address: Address,
    pub bytecode: Option<String>,
    pub abi: Option<serde_json::Value>,
    pub is_verified: bool,
    pub creation_transaction: Option<TransactionHash>,
}

#[derive(Debug, Clone)]
pub struct TokenCreationInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub metadata_uri: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TokenHolder {
    pub address: Address,
    pub balance: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct AssetTokenizationInfo {
    pub asset_id: String,
    pub asset_name: String,
    pub asset_description: String,
    pub total_value: u64,
    pub token_symbol: String,
    pub token_name: String,
    pub total_tokens: u64,
    pub decimals: u8,
    pub metadata_uri: String,
}

#[derive(Debug)]
pub struct BlockSubscription {
    // Implementation-specific subscription handle
    pub id: String,
}

#[derive(Debug)]
pub struct AddressSubscription {
    // Implementation-specific subscription handle
    pub id: String,
    pub address: Address,
}

#[derive(Debug, Clone)]
pub struct ContractEvent {
    pub transaction_hash: TransactionHash,
    pub block_number: u64,
    pub event_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
