// =====================================================================================
// File: core-oracle/src/chainlink.rs
// Description: Chainlink oracle integration module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{OracleError, OracleResult},
    service::ProviderClient,
    types::{OracleProvider, PriceData, ProviderConfig},
};

/// Chainlink oracle client
pub struct ChainlinkOracle {
    config: ProviderConfig,
    client: reqwest::Client,
    feeds: HashMap<String, ChainlinkFeed>,
}

/// Chainlink price feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainlinkFeed {
    pub feed_id: String,
    pub contract_address: String,
    pub decimals: u8,
    pub description: String,
    pub heartbeat: u64,
    pub deviation_threshold: f64,
    pub network: ChainlinkNetwork,
}

/// Chainlink aggregator interface
pub struct ChainlinkAggregator {
    oracle: ChainlinkOracle,
    feeds: Vec<ChainlinkFeed>,
}

/// Chainlink networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainlinkNetwork {
    Ethereum,
    Polygon,
    BSC,
    Avalanche,
    Arbitrum,
    Optimism,
    Fantom,
}

/// Chainlink price response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainlinkPriceResponse {
    pub answer: String,
    pub updated_at: i64,
    pub round_id: u64,
    pub started_at: i64,
    pub answered_in_round: u64,
}

/// Chainlink feed metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainlinkFeedMetadata {
    pub feed_id: String,
    pub name: String,
    pub asset_name: String,
    pub asset_type: String,
    pub status: String,
    pub decimals: u8,
    pub heartbeat: u64,
    pub deviation_threshold: f64,
    pub multiply: String,
    pub proxy_address: String,
}

/// Chainlink round data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainlinkRoundData {
    pub round_id: u64,
    pub answer: Decimal,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub answered_in_round: u64,
}

impl ChainlinkOracle {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        let mut feeds = HashMap::new();

        // Add some common Chainlink feeds
        feeds.insert(
            "ETH/USD".to_string(),
            ChainlinkFeed {
                feed_id: "ETH/USD".to_string(),
                contract_address: "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419".to_string(),
                decimals: 8,
                description: "ETH / USD".to_string(),
                heartbeat: 3600,
                deviation_threshold: 0.5,
                network: ChainlinkNetwork::Ethereum,
            },
        );

        feeds.insert(
            "BTC/USD".to_string(),
            ChainlinkFeed {
                feed_id: "BTC/USD".to_string(),
                contract_address: "0xF4030086522a5bEEa4988F8cA5B36dbC97BeE88c".to_string(),
                decimals: 8,
                description: "BTC / USD".to_string(),
                heartbeat: 3600,
                deviation_threshold: 0.5,
                network: ChainlinkNetwork::Ethereum,
            },
        );

        Self {
            config,
            client,
            feeds,
        }
    }

    /// Get feed configuration
    pub fn get_feed(&self, feed_id: &str) -> Option<&ChainlinkFeed> {
        self.feeds.get(feed_id)
    }

    /// Add new feed
    pub fn add_feed(&mut self, feed: ChainlinkFeed) {
        self.feeds.insert(feed.feed_id.clone(), feed);
    }

    /// Remove feed
    pub fn remove_feed(&mut self, feed_id: &str) -> Option<ChainlinkFeed> {
        self.feeds.remove(feed_id)
    }

    /// Get latest round data from Chainlink
    pub async fn get_latest_round_data(&self, feed_id: &str) -> OracleResult<ChainlinkRoundData> {
        let feed = self
            .get_feed(feed_id)
            .ok_or_else(|| OracleError::feed_not_found(feed_id.to_string()))?;

        // Mock Chainlink API call - in reality, this would call the actual Chainlink contract
        let mock_response = self.mock_chainlink_response(feed).await?;

        let answer = Decimal::from_str_exact(&mock_response.answer).map_err(|e| {
            OracleError::serialization_error(format!("Invalid price format: {}", e))
        })?;

        let updated_at =
            chrono::DateTime::from_timestamp(mock_response.updated_at, 0).unwrap_or(Utc::now());
        let started_at =
            chrono::DateTime::from_timestamp(mock_response.started_at, 0).unwrap_or(Utc::now());

        Ok(ChainlinkRoundData {
            round_id: mock_response.round_id,
            answer: answer / Decimal::new(10_i64.pow(feed.decimals as u32), 0),
            started_at,
            updated_at,
            answered_in_round: mock_response.answered_in_round,
        })
    }

    /// Get historical round data
    pub async fn get_round_data(
        &self,
        feed_id: &str,
        round_id: u64,
    ) -> OracleResult<ChainlinkRoundData> {
        let feed = self
            .get_feed(feed_id)
            .ok_or_else(|| OracleError::feed_not_found(feed_id.to_string()))?;

        // Mock historical data - in reality, this would query specific round
        let mock_response = ChainlinkPriceResponse {
            answer: match feed_id {
                "ETH/USD" => "200000000000".to_string(), // $2000.00 with 8 decimals
                "BTC/USD" => "5000000000000".to_string(), // $50000.00 with 8 decimals
                _ => "10000000000".to_string(),          // $100.00 with 8 decimals
            },
            updated_at: Utc::now().timestamp() - (round_id as i64 * 3600),
            round_id,
            started_at: Utc::now().timestamp() - (round_id as i64 * 3600) - 60,
            answered_in_round: round_id,
        };

        let answer = Decimal::from_str_exact(&mock_response.answer).map_err(|e| {
            OracleError::serialization_error(format!("Invalid price format: {}", e))
        })?;

        let updated_at =
            chrono::DateTime::from_timestamp(mock_response.updated_at, 0).unwrap_or(Utc::now());
        let started_at =
            chrono::DateTime::from_timestamp(mock_response.started_at, 0).unwrap_or(Utc::now());

        Ok(ChainlinkRoundData {
            round_id: mock_response.round_id,
            answer: answer / Decimal::new(10_i64.pow(feed.decimals as u32), 0),
            started_at,
            updated_at,
            answered_in_round: mock_response.answered_in_round,
        })
    }

    /// Get feed metadata
    pub async fn get_feed_metadata(&self, feed_id: &str) -> OracleResult<ChainlinkFeedMetadata> {
        let feed = self
            .get_feed(feed_id)
            .ok_or_else(|| OracleError::feed_not_found(feed_id.to_string()))?;

        Ok(ChainlinkFeedMetadata {
            feed_id: feed.feed_id.clone(),
            name: feed.description.clone(),
            asset_name: feed
                .feed_id
                .split('/')
                .next()
                .unwrap_or("Unknown")
                .to_string(),
            asset_type: "crypto".to_string(),
            status: "live".to_string(),
            decimals: feed.decimals,
            heartbeat: feed.heartbeat,
            deviation_threshold: feed.deviation_threshold,
            multiply: "1".to_string(),
            proxy_address: feed.contract_address.clone(),
        })
    }

    /// Mock Chainlink API response
    async fn mock_chainlink_response(
        &self,
        feed: &ChainlinkFeed,
    ) -> OracleResult<ChainlinkPriceResponse> {
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let base_price = match feed.feed_id.as_str() {
            "ETH/USD" => "200000000000",  // $2000.00 with 8 decimals
            "BTC/USD" => "5000000000000", // $50000.00 with 8 decimals
            "USDC/USD" => "100000000",    // $1.00 with 8 decimals
            _ => "10000000000",           // $100.00 with 8 decimals
        };

        Ok(ChainlinkPriceResponse {
            answer: base_price.to_string(),
            updated_at: Utc::now().timestamp(),
            round_id: 12345,
            started_at: Utc::now().timestamp() - 60,
            answered_in_round: 12345,
        })
    }

    /// Validate feed data
    pub fn validate_feed_data(
        &self,
        feed_id: &str,
        round_data: &ChainlinkRoundData,
    ) -> OracleResult<bool> {
        let feed = self
            .get_feed(feed_id)
            .ok_or_else(|| OracleError::feed_not_found(feed_id.to_string()))?;

        // Check data freshness
        let age = Utc::now() - round_data.updated_at;
        if age.num_seconds() > feed.heartbeat as i64 * 2 {
            return Err(OracleError::stale_data(
                feed_id,
                age.num_seconds() as u64,
                feed.heartbeat * 2,
            ));
        }

        // Check price validity
        if round_data.answer <= Decimal::ZERO {
            return Err(OracleError::invalid_price(
                feed_id,
                round_data.answer.to_string(),
                "Price must be positive",
            ));
        }

        Ok(true)
    }
}

#[async_trait]
impl ProviderClient for ChainlinkOracle {
    async fn get_price(&self, feed_id: &str) -> OracleResult<PriceData> {
        let round_data = self.get_latest_round_data(feed_id).await?;

        // Validate the data
        self.validate_feed_data(feed_id, &round_data)?;

        Ok(PriceData {
            price: round_data.answer,
            timestamp: round_data.updated_at,
            source: OracleProvider::Chainlink,
            confidence: 0.95, // Chainlink typically has high confidence
            volume: None,
            market_cap: None,
            deviation: None,
            round_id: Some(round_data.round_id),
        })
    }

    async fn get_health(&self) -> OracleResult<crate::types::ProviderHealth> {
        // Mock health check - in reality, this would check Chainlink node status
        Ok(crate::types::ProviderHealth {
            status: crate::types::HealthStatus::Healthy,
            last_successful_update: Utc::now(),
            consecutive_failures: 0,
            response_time_ms: 50,
            success_rate: 99.9,
        })
    }

    fn provider(&self) -> OracleProvider {
        OracleProvider::Chainlink
    }
}

impl ChainlinkAggregator {
    pub fn new(oracle: ChainlinkOracle, feeds: Vec<ChainlinkFeed>) -> Self {
        Self { oracle, feeds }
    }

    /// Get aggregated price from multiple feeds
    pub async fn get_aggregated_price(&self, base_feeds: &[String]) -> OracleResult<PriceData> {
        let mut prices = Vec::new();

        for feed_id in base_feeds {
            match self.oracle.get_price(feed_id).await {
                Ok(price_data) => prices.push(price_data),
                Err(e) => {
                    eprintln!("Failed to get price for {}: {}", feed_id, e);
                }
            }
        }

        if prices.is_empty() {
            return Err(OracleError::insufficient_data("aggregation", 1, 0));
        }

        // Simple median aggregation
        prices.sort_by(|a, b| a.price.cmp(&b.price));
        let median_price = if prices.len() % 2 == 0 {
            let mid = prices.len() / 2;
            (prices[mid - 1].price + prices[mid].price) / Decimal::new(2, 0)
        } else {
            prices[prices.len() / 2].price
        };

        let avg_confidence = prices.iter().map(|p| p.confidence).sum::<f64>() / prices.len() as f64;

        Ok(PriceData {
            price: median_price,
            timestamp: Utc::now(),
            source: OracleProvider::Chainlink,
            confidence: avg_confidence,
            volume: None,
            market_cap: None,
            deviation: None,
            round_id: None,
        })
    }

    /// Get all supported feeds
    pub fn get_supported_feeds(&self) -> Vec<String> {
        self.feeds.iter().map(|feed| feed.feed_id.clone()).collect()
    }
}

impl ChainlinkNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            Self::Ethereum => 1,
            Self::Polygon => 137,
            Self::BSC => 56,
            Self::Avalanche => 43114,
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Fantom => 250,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Ethereum => "Ethereum",
            Self::Polygon => "Polygon",
            Self::BSC => "Binance Smart Chain",
            Self::Avalanche => "Avalanche",
            Self::Arbitrum => "Arbitrum",
            Self::Optimism => "Optimism",
            Self::Fantom => "Fantom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chainlink_oracle_creation() {
        let config = ProviderConfig {
            provider: OracleProvider::Chainlink,
            enabled: true,
            api_key: None,
            endpoint: "https://api.chain.link".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 1,
        };

        let oracle = ChainlinkOracle::new(config);
        assert!(oracle.get_feed("ETH/USD").is_some());
        assert!(oracle.get_feed("BTC/USD").is_some());
    }

    #[tokio::test]
    async fn test_chainlink_price_fetch() {
        let config = ProviderConfig {
            provider: OracleProvider::Chainlink,
            enabled: true,
            api_key: None,
            endpoint: "https://api.chain.link".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 1,
        };

        let oracle = ChainlinkOracle::new(config);
        let price = oracle.get_price("ETH/USD").await.unwrap();

        assert!(price.price > Decimal::ZERO);
        assert_eq!(price.source, OracleProvider::Chainlink);
        assert!(price.confidence > 0.0);
        assert!(price.round_id.is_some());
    }

    #[tokio::test]
    async fn test_chainlink_round_data() {
        let config = ProviderConfig {
            provider: OracleProvider::Chainlink,
            enabled: true,
            api_key: None,
            endpoint: "https://api.chain.link".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 1,
        };

        let oracle = ChainlinkOracle::new(config);
        let round_data = oracle.get_latest_round_data("ETH/USD").await.unwrap();

        assert!(round_data.answer > Decimal::ZERO);
        assert!(round_data.round_id > 0);
        assert!(round_data.updated_at <= Utc::now());
    }

    #[tokio::test]
    async fn test_chainlink_feed_metadata() {
        let config = ProviderConfig {
            provider: OracleProvider::Chainlink,
            enabled: true,
            api_key: None,
            endpoint: "https://api.chain.link".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 1,
        };

        let oracle = ChainlinkOracle::new(config);
        let metadata = oracle.get_feed_metadata("ETH/USD").await.unwrap();

        assert_eq!(metadata.feed_id, "ETH/USD");
        assert_eq!(metadata.decimals, 8);
        assert_eq!(metadata.status, "live");
    }

    #[tokio::test]
    async fn test_chainlink_aggregator() {
        let config = ProviderConfig {
            provider: OracleProvider::Chainlink,
            enabled: true,
            api_key: None,
            endpoint: "https://api.chain.link".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 1,
        };

        let oracle = ChainlinkOracle::new(config);
        let feeds = vec![ChainlinkFeed {
            feed_id: "ETH/USD".to_string(),
            contract_address: "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419".to_string(),
            decimals: 8,
            description: "ETH / USD".to_string(),
            heartbeat: 3600,
            deviation_threshold: 0.5,
            network: ChainlinkNetwork::Ethereum,
        }];

        let aggregator = ChainlinkAggregator::new(oracle, feeds);
        let supported_feeds = aggregator.get_supported_feeds();

        assert!(!supported_feeds.is_empty());
        assert!(supported_feeds.contains(&"ETH/USD".to_string()));
    }

    #[test]
    fn test_chainlink_network() {
        assert_eq!(ChainlinkNetwork::Ethereum.chain_id(), 1);
        assert_eq!(ChainlinkNetwork::Polygon.chain_id(), 137);
        assert_eq!(ChainlinkNetwork::Ethereum.name(), "Ethereum");
        assert_eq!(ChainlinkNetwork::Polygon.name(), "Polygon");
    }
}
