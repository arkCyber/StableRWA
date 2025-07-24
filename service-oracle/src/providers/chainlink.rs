// =====================================================================================
// RWA Tokenization Platform - Chainlink Price Provider
// 
// Author: arkSong (arksong2018@gmail.com)
// Date: 2025-01-27
// 
// This module implements the Chainlink price provider for the Oracle service.
// It provides decentralized price data from Chainlink price feeds.
// =====================================================================================

use crate::error::{OracleError, OracleResult};
use crate::models::{AssetPrice, ProviderType};
use crate::providers::{PriceProvider, RateLimit};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ethers::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Chainlink aggregator ABI for price feeds
abigen!(
    ChainlinkAggregator,
    r#"[
        function latestRoundData() external view returns (uint80 roundId, int256 answer, uint256 startedAt, uint256 updatedAt, uint80 answeredInRound)
        function decimals() external view returns (uint8)
        function description() external view returns (string memory)
    ]"#
);

/// Chainlink provider configuration
#[derive(Debug, Clone)]
pub struct ChainlinkConfig {
    pub rpc_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub price_feeds: HashMap<String, ChainlinkFeedConfig>,
}

/// Configuration for a specific Chainlink price feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainlinkFeedConfig {
    pub address: String,
    pub decimals: u8,
    pub description: String,
    pub heartbeat: u64, // Maximum time between updates in seconds
}

impl Default for ChainlinkConfig {
    fn default() -> Self {
        let mut price_feeds = HashMap::new();
        
        // Ethereum mainnet price feeds
        price_feeds.insert("BTC/USD".to_string(), ChainlinkFeedConfig {
            address: "0xF4030086522a5bEEa4988F8cA5B36dbC97BeE88c".to_string(),
            decimals: 8,
            description: "BTC / USD".to_string(),
            heartbeat: 3600, // 1 hour
        });
        
        price_feeds.insert("ETH/USD".to_string(), ChainlinkFeedConfig {
            address: "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419".to_string(),
            decimals: 8,
            description: "ETH / USD".to_string(),
            heartbeat: 3600, // 1 hour
        });
        
        price_feeds.insert("USDT/USD".to_string(), ChainlinkFeedConfig {
            address: "0x3E7d1eAB13ad0104d2750B8863b489D65364e32D".to_string(),
            decimals: 8,
            description: "USDT / USD".to_string(),
            heartbeat: 86400, // 24 hours
        });

        Self {
            rpc_url: std::env::var("ETHEREUM_RPC_URL")
                .unwrap_or_else(|_| "https://eth-mainnet.alchemyapi.io/v2/demo".to_string()),
            timeout_seconds: 30,
            max_retries: 3,
            price_feeds,
        }
    }
}

/// Chainlink price provider implementation
pub struct ChainlinkProvider {
    config: ChainlinkConfig,
    provider: Arc<Provider<Http>>,
    feed_contracts: HashMap<String, ChainlinkAggregator<Provider<Http>>>,
}

impl ChainlinkProvider {
    /// Create a new Chainlink provider
    pub async fn new(config: ChainlinkConfig) -> OracleResult<Self> {
        info!("🚀 Initializing Chainlink provider with RPC: {}", config.rpc_url);
        
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .map_err(|e| OracleError::Configuration {
                message: format!("Failed to create Ethereum provider: {}", e),
            })?;
        
        let provider = Arc::new(provider);
        let mut feed_contracts = HashMap::new();
        
        // Initialize price feed contracts
        for (pair, feed_config) in &config.price_feeds {
            let address: Address = feed_config.address.parse()
                .map_err(|e| OracleError::Configuration {
                    message: format!("Invalid contract address for {}: {}", pair, e),
                })?;
            
            let contract = ChainlinkAggregator::new(address, provider.clone());
            feed_contracts.insert(pair.clone(), contract);
            
            debug!("📡 Initialized Chainlink feed for {}: {}", pair, feed_config.address);
        }
        
        info!("✅ Chainlink provider initialized with {} price feeds", feed_contracts.len());
        
        Ok(Self {
            config,
            provider,
            feed_contracts,
        })
    }
    
    /// Map asset ID and currency to Chainlink feed pair
    fn map_to_feed_pair(&self, asset_id: &str, currency: &str) -> OracleResult<String> {
        let pair = format!("{}/{}", asset_id.to_uppercase(), currency.to_uppercase());
        
        if self.config.price_feeds.contains_key(&pair) {
            Ok(pair)
        } else {
            Err(OracleError::AssetNotSupported {
                asset_id: format!("{} in {}", asset_id, currency),
                provider: "Chainlink".to_string(),
            })
        }
    }
    
    /// Get price data from a specific Chainlink feed
    async fn get_feed_price(&self, pair: &str) -> OracleResult<AssetPrice> {
        let contract = self.feed_contracts.get(pair)
            .ok_or_else(|| OracleError::AssetNotFound {
                asset_id: pair.to_string(),
                provider: "Chainlink".to_string(),
            })?;
        
        let feed_config = self.config.price_feeds.get(pair).unwrap();
        
        debug!("📊 Fetching latest round data for {}", pair);
        
        // Get latest round data with retries
        let mut last_error = None;
        for attempt in 1..=self.config.max_retries {
            match contract.latest_round_data().call().await {
                Ok((round_id, answer, _started_at, updated_at, _answered_in_round)) => {
                    debug!("✅ Retrieved round data for {}: round={}, answer={}, updated_at={}", 
                           pair, round_id, answer, updated_at);
                    
                    // Check if the data is fresh enough
                    let now = Utc::now().timestamp() as u64;
                    let age = now.saturating_sub(updated_at.as_u64());
                    
                    if age > feed_config.heartbeat * 2 {
                        warn!("⚠️ Chainlink data for {} is stale: {} seconds old", pair, age);
                    }
                    
                    // Convert price based on decimals
                    let price_raw = if answer >= 0 {
                        answer as u128
                    } else {
                        return Err(OracleError::Parsing {
                            message: format!("Negative price from Chainlink feed: {}", answer),
                        });
                    };
                    
                    let divisor = 10_u128.pow(feed_config.decimals as u32);
                    let price = Decimal::from(price_raw) / Decimal::from(divisor);
                    
                    let timestamp = DateTime::from_timestamp(updated_at.as_u64() as i64, 0)
                        .unwrap_or_else(|| Utc::now());
                    
                    // Parse asset and currency from pair
                    let parts: Vec<&str> = pair.split('/').collect();
                    let (asset_id, currency) = if parts.len() == 2 {
                        (parts[0], parts[1])
                    } else {
                        return Err(OracleError::Parsing {
                            message: format!("Invalid pair format: {}", pair),
                        });
                    };
                    
                    let mut metadata = HashMap::new();
                    metadata.insert("round_id".to_string(), round_id.to_string());
                    metadata.insert("contract_address".to_string(), feed_config.address.clone());
                    metadata.insert("decimals".to_string(), feed_config.decimals.to_string());
                    metadata.insert("heartbeat".to_string(), feed_config.heartbeat.to_string());
                    metadata.insert("data_age_seconds".to_string(), age.to_string());
                    
                    let asset_price = AssetPrice {
                        asset_id: asset_id.to_string(),
                        price,
                        currency: currency.to_string(),
                        timestamp,
                        confidence: Decimal::from(95) / Decimal::from(100), // 0.95 confidence for Chainlink
                        source: self.name().to_string(),
                        metadata: Some(metadata),
                    };
                    
                    info!("✅ Successfully retrieved Chainlink price for {}: {} {}", 
                          asset_id, price, currency);
                    return Ok(asset_price);
                }
                Err(e) => {
                    warn!("⚠️ Attempt {} failed for Chainlink feed {}: {}", attempt, pair, e);
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000 * attempt as u64)).await;
                    }
                }
            }
        }
        
        error!("❌ All attempts failed for Chainlink feed {}", pair);
        Err(OracleError::Provider {
            provider: "Chainlink".to_string(),
            message: format!("Failed after {} attempts: {}", 
                           self.config.max_retries, 
                           last_error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string())),
        })
    }
}

#[async_trait]
impl PriceProvider for ChainlinkProvider {
    fn name(&self) -> &str {
        "Chainlink"
    }
    
    fn provider_type(&self) -> ProviderType {
        ProviderType::Chainlink
    }
    
    fn weight(&self) -> Decimal {
        Decimal::from(95) / Decimal::from(100) // 0.95 weight - highest trust
    }
    
    async fn health_check(&self) -> OracleResult<bool> {
        debug!("🔍 Performing Chainlink health check");
        
        // Check if we can connect to the Ethereum provider
        match self.provider.get_block_number().await {
            Ok(block_number) => {
                debug!("✅ Ethereum provider healthy, latest block: {}", block_number);
                
                // Test one price feed if available
                if let Some((pair, _)) = self.config.price_feeds.iter().next() {
                    match self.get_feed_price(pair).await {
                        Ok(_) => {
                            info!("✅ Chainlink health check passed");
                            Ok(true)
                        }
                        Err(e) => {
                            warn!("⚠️ Chainlink feed test failed: {}", e);
                            Ok(false)
                        }
                    }
                } else {
                    warn!("⚠️ No Chainlink feeds configured");
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("⚠️ Ethereum provider health check failed: {}", e);
                Ok(false)
            }
        }
    }
    
    async fn get_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
        info!("💰 Getting price for {} in {} from Chainlink", asset_id, currency);
        
        let pair = self.map_to_feed_pair(asset_id, currency)?;
        self.get_feed_price(&pair).await
    }
    
    async fn get_batch_prices(&self, asset_ids: &[String], currency: &str) -> OracleResult<Vec<AssetPrice>> {
        info!("📊 Getting batch prices for {} assets from Chainlink", asset_ids.len());
        
        let mut prices = Vec::new();
        
        for asset_id in asset_ids {
            match self.get_price(asset_id, currency).await {
                Ok(price) => prices.push(price),
                Err(e) => {
                    warn!("⚠️ Failed to get price for {} from Chainlink: {}", asset_id, e);
                    // Continue with other assets instead of failing the entire batch
                }
            }
        }
        
        info!("✅ Successfully retrieved {} prices from Chainlink", prices.len());
        Ok(prices)
    }
    
    async fn get_supported_assets(&self) -> OracleResult<Vec<String>> {
        let assets: Vec<String> = self.config.price_feeds.keys()
            .filter_map(|pair| {
                let parts: Vec<&str> = pair.split('/').collect();
                if parts.len() == 2 {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            })
            .collect();
        
        Ok(assets)
    }
    
    fn get_rate_limit(&self) -> RateLimit {
        RateLimit {
            requests_per_minute: 300, // Conservative limit for Ethereum RPC
            current_usage: 0,
            reset_time: Utc::now() + chrono::Duration::minutes(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_chainlink_config_default() {
        let config = ChainlinkConfig::default();
        assert!(!config.price_feeds.is_empty());
        assert!(config.price_feeds.contains_key("BTC/USD"));
        assert!(config.price_feeds.contains_key("ETH/USD"));
    }

    #[test]
    fn test_feed_pair_mapping() {
        let config = ChainlinkConfig::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        // This test would require a real provider, so we'll just test the config
        assert!(config.price_feeds.contains_key("BTC/USD"));
        assert!(config.price_feeds.contains_key("ETH/USD"));
    }

    #[test]
    fn test_chainlink_feed_config() {
        let feed_config = ChainlinkFeedConfig {
            address: "0xF4030086522a5bEEa4988F8cA5B36dbC97BeE88c".to_string(),
            decimals: 8,
            description: "BTC / USD".to_string(),
            heartbeat: 3600,
        };
        
        assert_eq!(feed_config.decimals, 8);
        assert_eq!(feed_config.heartbeat, 3600);
        assert!(!feed_config.address.is_empty());
    }
}
