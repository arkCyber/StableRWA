// =====================================================================================
// File: core-blockchain/src/lib.rs
// Description: Core blockchain integration framework for StableRWA Platform
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

//! # Core Blockchain Library
//!
//! This library provides enterprise-grade blockchain integration for the RWA platform.
//! It includes support for multiple blockchain networks, wallet management, smart contracts,
//! and transaction processing with comprehensive error handling and monitoring.

pub mod adapters;
pub mod types;
pub mod wallet;
pub mod contracts;
pub mod transaction_processor;

// Re-export main types and traits
pub use adapters::*;
pub use types::*;
pub use wallet::*;
pub use contracts::*;
pub use transaction_processor::*;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Comprehensive blockchain error types for enterprise operations
#[derive(Error, Debug, Clone)]
pub enum BlockchainError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: String, available: String },

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("Transaction timeout: {0}")]
    TransactionTimeout(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Wallet locked: {0}")]
    WalletLocked(String),

    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Contract call failed: {0}")]
    ContractCallFailed(String),

    #[error("Contract deployment failed: {0}")]
    ContractDeploymentFailed(String),

    #[error("Invalid gas price: {0}")]
    InvalidGasPrice(String),

    #[error("Insufficient gas: {0}")]
    InsufficientGas(String),

    #[error("Nonce error: {0}")]
    NonceError(String),

    #[error("Signature error: {0}")]
    SignatureError(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for blockchain operations
pub type BlockchainResult<T> = Result<T, BlockchainError>;

/// Common blockchain operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub operation_id: String,
}

impl<T> OperationResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            operation_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
            operation_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Blockchain network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub network: BlockchainNetwork,
    pub rpc_url: String,
    pub chain_id: Option<u64>,
    pub gas_price_multiplier: f64,
    pub confirmation_blocks: u32,
    pub timeout_seconds: u64,
}

impl NetworkConfig {
    pub fn ethereum_mainnet() -> Self {
        Self {
            name: "Ethereum Mainnet".to_string(),
            network: BlockchainNetwork::EthereumMainnet,
            rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: Some(1),
            gas_price_multiplier: 1.0,
            confirmation_blocks: 12,
            timeout_seconds: 300,
        }
    }

    pub fn ethereum_testnet() -> Self {
        Self {
            name: "Ethereum Goerli".to_string(),
            network: BlockchainNetwork::EthereumTestnet,
            rpc_url: "https://goerli.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: Some(5),
            gas_price_multiplier: 1.0,
            confirmation_blocks: 6,
            timeout_seconds: 180,
        }
    }
}

/// Blockchain service factory for creating network-specific services
pub struct BlockchainServiceFactory;

impl BlockchainServiceFactory {
    /// Create a blockchain adapter for the specified network
    pub fn create_adapter(config: &NetworkConfig) -> BlockchainResult<Box<dyn BlockchainAdapter>> {
        match config.network {
            BlockchainNetwork::EthereumMainnet | BlockchainNetwork::EthereumTestnet => {
                Ok(Box::new(EthereumAdapter::new(config.clone())))
            }
            BlockchainNetwork::SolanaMainnet | BlockchainNetwork::SolanaTestnet => {
                Ok(Box::new(SolanaAdapter::new(config.clone())))
            }
            BlockchainNetwork::PolkadotMainnet | BlockchainNetwork::PolkadotTestnet => {
                Ok(Box::new(PolkadotAdapter::new(config.clone())))
            }
        }
    }

    /// Create a wallet manager for the specified network
    pub fn create_wallet_manager(_config: &NetworkConfig) -> BlockchainResult<Box<dyn WalletManager>> {
        Ok(Box::new(InMemoryWalletManager::new()))
    }

    /// Create a transaction processor for the specified network
    pub fn create_transaction_processor(config: &NetworkConfig) -> BlockchainResult<Box<dyn TransactionProcessor>> {
        match config.network {
            BlockchainNetwork::EthereumMainnet | BlockchainNetwork::EthereumTestnet => {
                Ok(Box::new(EthereumTransactionProcessor::new(
                    config.network.clone(),
                    config.rpc_url.clone(),
                )))
            }
            _ => Err(BlockchainError::UnsupportedOperation(
                format!("Transaction processor not implemented for {:?}", config.network)
            )),
        }
    }
}

/// Blockchain integration utilities
pub mod utils {
    use super::*;

    /// Validate blockchain address format
    pub fn validate_address(address: &str, network: &BlockchainNetwork) -> BlockchainResult<()> {
        match network {
            BlockchainNetwork::EthereumMainnet | BlockchainNetwork::EthereumTestnet => {
                if address.len() != 42 || !address.starts_with("0x") {
                    return Err(BlockchainError::InvalidAddress(
                        "Ethereum address must be 42 characters starting with 0x".to_string()
                    ));
                }
            }
            BlockchainNetwork::SolanaMainnet | BlockchainNetwork::SolanaTestnet => {
                if address.len() < 32 || address.len() > 44 {
                    return Err(BlockchainError::InvalidAddress(
                        "Solana address must be 32-44 characters".to_string()
                    ));
                }
            }
            BlockchainNetwork::PolkadotMainnet | BlockchainNetwork::PolkadotTestnet => {
                if address.len() < 47 || address.len() > 48 {
                    return Err(BlockchainError::InvalidAddress(
                        "Polkadot address must be 47-48 characters".to_string()
                    ));
                }
            }
        }
                Ok(())
    }

    /// Convert amount to blockchain-specific format
    pub fn format_amount(amount: &str, network: &BlockchainNetwork) -> BlockchainResult<String> {
        let parsed_amount: f64 = amount.parse()
            .map_err(|_| BlockchainError::InvalidTransaction("Invalid amount format".to_string()))?;

        match network {
            BlockchainNetwork::EthereumMainnet | BlockchainNetwork::EthereumTestnet => {
                // Convert to wei (18 decimals)
                let wei = (parsed_amount * 1e18) as u64;
                Ok(wei.to_string())
            }
            BlockchainNetwork::SolanaMainnet | BlockchainNetwork::SolanaTestnet => {
                // Convert to lamports (9 decimals)
                let lamports = (parsed_amount * 1e9) as u64;
                Ok(lamports.to_string())
            }
            BlockchainNetwork::PolkadotMainnet | BlockchainNetwork::PolkadotTestnet => {
                // Convert to planck (10 decimals)
                let planck = (parsed_amount * 1e10) as u64;
                Ok(planck.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_error_display() {
        let error = BlockchainError::ConnectionError("Test error".to_string());
        assert_eq!(error.to_string(), "Connection error: Test error");
    }

    #[test]
    fn test_operation_result_success() {
        let result = OperationResult::success("test data");
        assert!(result.success);
        assert_eq!(result.data, Some("test data"));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_operation_result_error() {
        let result: OperationResult<String> = OperationResult::error("test error".to_string());
        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("test error".to_string()));
    }

    #[test]
    fn test_network_config_ethereum() {
        let config = NetworkConfig::ethereum_mainnet();
        assert_eq!(config.network, BlockchainNetwork::EthereumMainnet);
        assert_eq!(config.chain_id, Some(1));
        assert_eq!(config.confirmation_blocks, 12);
    }

    #[test]
    fn test_address_validation() {
        // Valid Ethereum address
        assert!(utils::validate_address(
            "0x1234567890123456789012345678901234567890",
            &BlockchainNetwork::EthereumMainnet
        ).is_ok());

        // Invalid Ethereum address (too short)
        assert!(utils::validate_address(
            "0x123",
            &BlockchainNetwork::EthereumMainnet
        ).is_err());

        // Invalid Ethereum address (no 0x prefix)
        assert!(utils::validate_address(
            "1234567890123456789012345678901234567890",
            &BlockchainNetwork::EthereumMainnet
        ).is_err());
    }

    #[test]
    fn test_amount_formatting() {
        // Ethereum (18 decimals)
        let result = utils::format_amount("1.0", &BlockchainNetwork::EthereumMainnet).unwrap();
        assert_eq!(result, "1000000000000000000");

        // Solana (9 decimals)
        let result = utils::format_amount("1.0", &BlockchainNetwork::SolanaMainnet).unwrap();
        assert_eq!(result, "1000000000");

        // Polkadot (10 decimals)
        let result = utils::format_amount("1.0", &BlockchainNetwork::PolkadotMainnet).unwrap();
        assert_eq!(result, "10000000000");
    }

    #[test]
    fn test_blockchain_service_factory() {
        let config = NetworkConfig::ethereum_testnet();

        // Test adapter creation
        let adapter = BlockchainServiceFactory::create_adapter(&config);
        assert!(adapter.is_ok());

        // Test wallet manager creation
        let wallet_manager = BlockchainServiceFactory::create_wallet_manager(&config);
        assert!(wallet_manager.is_ok());

        // Test transaction processor creation
        let tx_processor = BlockchainServiceFactory::create_transaction_processor(&config);
        assert!(tx_processor.is_ok());
    }
}



