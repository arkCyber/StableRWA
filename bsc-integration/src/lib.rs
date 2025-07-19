//!
//! bsc-integration crate for StableRWA platform.
//! Provides microservice-level API for Binance Smart Chain (BSC) operations.
//! Integrates with core-blockchain BscClient, includes logging, error handling, and tests.

use core_blockchain::bsc::BscClient;
use log::{info, error};
use std::env;

/// Returns a BscClient using the BSC_RPC_URL environment variable or default.
pub fn get_bsc_client() -> BscClient {
    let rpc_url = env::var("BSC_RPC_URL").unwrap_or_else(|_| "https://bsc-dataseed.binance.org/".to_string());
    info!("{} - [bsc-integration] Using BSC RPC URL: {}", chrono::Utc::now(), rpc_url);
    BscClient::new(rpc_url)
}

/// Fetches the latest BSC block number, with logging and error handling.
pub fn fetch_bsc_block_number() -> Result<u64, String> {
    let client = get_bsc_client();
    match client.latest_block_number() {
        Ok(num) => {
            info!("{} - [bsc-integration] Latest BSC block number: {}", chrono::Utc::now(), num);
            Ok(num)
        },
        Err(e) => {
            error!("{} - [bsc-integration] Error fetching BSC block number: {}", chrono::Utc::now(), e);
            Err(format!("Failed to fetch BSC block number: {}", e))
        }
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
    fn test_get_bsc_client() {
        init_logger();
        let client = get_bsc_client();
        assert_eq!(client.name(), "Binance Smart Chain");
    }

    #[test]
    fn test_fetch_bsc_block_number_error() {
        init_logger();
        // Set an invalid URL to force error
        std::env::set_var("BSC_RPC_URL", "http://invalid-url");
        let result = fetch_bsc_block_number();
        assert!(result.is_err());
    }
} 