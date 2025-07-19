// =====================================================================================
// File: smart-contract/src/lib.rs
// Description: Core EVM smart contract management logic for enterprise-grade microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use std::fs;
use std::sync::Arc;
use ethers::prelude::*;
use ethers::contract::abigen;
use thiserror::Error;

/// Error type for smart contract operations
#[derive(Debug, Error)]
pub enum ContractError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Signer error: {0}")]
    Signer(String),
    #[error("ABI error: {0}")]
    Abi(String),
    #[error("Contract error: {0}")]
    Contract(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Other error: {0}")]
    Other(String),
}

/// ContractManager for EVM smart contract operations
pub struct ContractManager {
    pub provider: Provider<Http>,
    pub signer: Option<Wallet<SigningKey>>,
    pub chain_id: u64,
}

impl ContractManager {
    /// Create a new ContractManager from environment variables
    pub fn from_env() -> Result<Self, ContractError> {
        let node_url = std::env::var("EVM_NODE_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());
        let provider = Provider::<Http>::try_from(node_url.clone())
            .map_err(|e| ContractError::Provider(e.to_string()))?;
        let chain_id = std::env::var("EVM_CHAIN_ID").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
        let signer = match std::env::var("EVM_PRIVATE_KEY") {
            Ok(pk) => {
                let wallet = pk.parse::<LocalWallet>().map_err(|e| ContractError::Signer(e.to_string()))?;
                Some(wallet.with_chain_id(chain_id))
            },
            Err(_) => None,
        };
        info!("{} - [ContractManager] Initialized with node {} chain_id {}", Utc::now(), node_url, chain_id);
        Ok(Self { provider, signer, chain_id })
    }

    /// Load contract ABI from file
    pub fn load_abi(path: &str) -> Result<Abi, ContractError> {
        let abi_str = fs::read_to_string(path).map_err(|e| ContractError::Io(e.to_string()))?;
        serde_json::from_str(&abi_str).map_err(|e| ContractError::Abi(e.to_string()))
    }

    /// Deploy a contract (bytecode, ABI, constructor args)
    pub async fn deploy_contract(&self, abi: Abi, bytecode: Bytes, constructor_args: Vec<Token>) -> Result<Address, ContractError> {
        let signer = self.signer.clone().ok_or(ContractError::Signer("No private key set".to_string()))?;
        let client = SignerMiddleware::new(self.provider.clone(), signer);
        let factory = ContractFactory::new(abi, bytecode, Arc::new(client));
        let deployer = factory.deploy_tokens(constructor_args).map_err(|e| ContractError::Contract(e.to_string()))?;
        let contract = deployer.send().await.map_err(|e| ContractError::Contract(e.to_string()))?;
        let addr = contract.address();
        info!("{} - [ContractManager] Deployed contract at {}", Utc::now(), addr);
        Ok(addr)
    }

    /// Call a contract function (read-only)
    pub async fn call_function(&self, abi: Abi, address: Address, func: &str, args: Vec<Token>) -> Result<Token, ContractError> {
        let client = self.provider.clone();
        let contract = Contract::new(address, abi, client);
        let result = contract.method::<_, Token>(func, args)
            .map_err(|e| ContractError::Contract(e.to_string()))?
            .call().await.map_err(|e| ContractError::Contract(e.to_string()))?;
        info!("{} - [ContractManager] Called function {} on {}", Utc::now(), func, address);
        Ok(result)
    }

    /// Send a contract transaction (write)
    pub async fn send_transaction(&self, abi: Abi, address: Address, func: &str, args: Vec<Token>) -> Result<TxHash, ContractError> {
        let signer = self.signer.clone().ok_or(ContractError::Signer("No private key set".to_string()))?;
        let client = SignerMiddleware::new(self.provider.clone(), signer);
        let contract = Contract::new(address, abi, Arc::new(client));
        let tx = contract.method::<_, H256>(func, args)
            .map_err(|e| ContractError::Contract(e.to_string()))?;
        let pending = tx.send().await.map_err(|e| ContractError::Contract(e.to_string()))?;
        let tx_hash = *pending;
        info!("{} - [ContractManager] Sent transaction {} on {}", Utc::now(), func, address);
        Ok(tx_hash)
    }

    /// Query contract events (logs)
    pub async fn query_events(&self, abi: Abi, address: Address, event: &str) -> Result<Vec<Log>, ContractError> {
        let client = self.provider.clone();
        let contract = Contract::new(address, abi, client);
        let logs = contract.event::<Log>(event)
            .map_err(|e| ContractError::Contract(e.to_string()))?
            .query().await.map_err(|e| ContractError::Contract(e.to_string()))?;
        info!("{} - [ContractManager] Queried event {} on {}", Utc::now(), event, address);
        Ok(logs)
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
    fn test_load_abi_fail() {
        init_logger();
        let res = ContractManager::load_abi("nonexistent.json");
        assert!(res.is_err());
    }
} 