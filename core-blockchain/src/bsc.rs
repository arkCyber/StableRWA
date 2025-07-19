//!
//! Binance Smart Chain (BSC) integration module for StableRWA core-blockchain crate.
//! Provides BSC-specific blockchain operations, compatible with the project-wide Blockchain trait.
//! All functions include detailed logging, error handling, and are suitable for microservice use.
//!
//! # Features
//! - Fetch latest block number from BSC node
//! - Enterprise-level logging with timestamps
//! - Comprehensive error handling
//! - Unit tests included

use crate::traits::{Blockchain, BlockNumberProvider};
use log::{info, error};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// BSC client struct for interacting with Binance Smart Chain nodes.
#[derive(Debug, Clone)]
pub struct BscClient {
    pub rpc_url: String,
}

impl BscClient {
    /// Creates a new BscClient with the given RPC URL.
    /// Logs initialization with timestamp.
    pub fn new(rpc_url: String) -> Self {
        info!("{} - [BscClient] Initialized with RPC URL: {}", chrono::Utc::now(), rpc_url);
        Self { rpc_url }
    }
}

impl Blockchain for BscClient {
    /// Returns the name of the blockchain.
    fn name(&self) -> &str {
        "Binance Smart Chain"
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    method: &'a str,
    params: Vec<serde_json::Value>,
    id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    id: u32,
    result: Option<String>,
    error: Option<serde_json::Value>,
}

impl BlockNumberProvider for BscClient {
    /// Fetches the latest block number from the BSC node.
    /// Logs the result and handles errors with context.
    fn latest_block_number(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let now = chrono::Utc::now();
        let client = reqwest::blocking::Client::new();
        let payload = RpcRequest {
            jsonrpc: "2.0",
            method: "eth_blockNumber",
            params: vec![],
            id: 1,
        };
        let resp = client.post(&self.rpc_url)
            .json(&payload)
            .send();
        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                error!("{} - [BscClient] HTTP request error: {}", now, e);
                return Err(Box::new(e));
            }
        };
        let resp: RpcResponse = match resp.json() {
            Ok(r) => r,
            Err(e) => {
                error!("{} - [BscClient] JSON parse error: {}", now, e);
                return Err(Box::new(e));
            }
        };
        if let Some(result) = resp.result {
            match u64::from_str_radix(result.trim_start_matches("0x"), 16) {
                Ok(block_number) => {
                    info!("{} - [BscClient] Latest block number: {}", now, block_number);
                    Ok(block_number)
                },
                Err(e) => {
                    error!("{} - [BscClient] Block number parse error: {}", now, e);
                    Err(Box::new(e))
                }
            }
        } else {
            error!("{} - [BscClient] RPC error: {:?}", now, resp.error);
            Err("Failed to fetch block number".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use log::LevelFilter;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Info).try_init();
    }

    #[test]
    fn test_bsc_client_name() {
        init_logger();
        let client = BscClient::new("http://localhost:8545".to_string());
        assert_eq!(client.name(), "Binance Smart Chain");
    }

    #[test]
    fn test_bsc_block_number_error() {
        init_logger();
        // Use an invalid URL to force error
        let client = BscClient::new("http://invalid-url".to_string());
        let result = client.latest_block_number();
        assert!(result.is_err());
    }
} 