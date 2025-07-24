// =====================================================================================
// File: core-oracle/src/pyth.rs
// Description: Pyth Network oracle integration module
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

/// Pyth Network oracle client
pub struct PythOracle {
    config: ProviderConfig,
    client: reqwest::Client,
    feeds: HashMap<String, PythFeed>,
}

/// Pyth Network price feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythFeed {
    pub symbol: String,
    pub price_id: String,
    pub asset_type: String,
    pub base: String,
    pub quote_currency: String,
    pub description: String,
}

/// Pyth Network price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythPriceData {
    pub id: String,
    pub price: PythPrice,
    pub ema_price: PythPrice,
    pub metadata: PythMetadata,
}

/// Pyth Network price structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythPrice {
    pub price: String,
    pub conf: String,
    pub expo: i32,
    pub publish_time: i64,
}

/// Pyth Network EMA (Exponentially Weighted Moving Average) price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythEmaPrice {
    pub price: String,
    pub conf: String,
    pub expo: i32,
    pub publish_time: i64,
}

/// Pyth Network metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythMetadata {
    pub symbol: String,
    pub asset_type: String,
    pub base: String,
    pub quote_currency: String,
    pub tenor: Option<String>,
    pub description: String,
    pub country: Option<String>,
    pub quoter: Option<String>,
}

/// Pyth Network publisher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythPublisher {
    pub name: String,
    pub account: String,
    pub aggregate_price_info: PythAggregateInfo,
}

/// Pyth Network aggregate price info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythAggregateInfo {
    pub price: String,
    pub conf: String,
    pub status: String,
    pub corp_act: String,
    pub publish_slot: u64,
}

/// Pyth Network price feed response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythPriceFeedResponse {
    pub id: String,
    pub price: PythPrice,
    pub ema_price: PythEmaPrice,
    pub metadata: PythMetadata,
    pub vaa: Option<String>, // Verified Action Approval from Wormhole
}

/// Pyth Network batch price response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythBatchResponse {
    pub parsed: Vec<PythPriceFeedResponse>,
    pub binary: PythBinaryData,
}

/// Pyth Network binary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythBinaryData {
    pub encoding: String,
    pub data: Vec<String>,
}

impl PythOracle {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        let mut feeds = HashMap::new();

        // Add some common Pyth Network feeds
        feeds.insert(
            "ETH/USD".to_string(),
            PythFeed {
                symbol: "Crypto.ETH/USD".to_string(),
                price_id: "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace"
                    .to_string(),
                asset_type: "Crypto".to_string(),
                base: "ETH".to_string(),
                quote_currency: "USD".to_string(),
                description: "ETH/USD".to_string(),
            },
        );

        feeds.insert(
            "BTC/USD".to_string(),
            PythFeed {
                symbol: "Crypto.BTC/USD".to_string(),
                price_id: "0xe62df6c8b4c85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43"
                    .to_string(),
                asset_type: "Crypto".to_string(),
                base: "BTC".to_string(),
                quote_currency: "USD".to_string(),
                description: "BTC/USD".to_string(),
            },
        );

        feeds.insert(
            "SOL/USD".to_string(),
            PythFeed {
                symbol: "Crypto.SOL/USD".to_string(),
                price_id: "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d"
                    .to_string(),
                asset_type: "Crypto".to_string(),
                base: "SOL".to_string(),
                quote_currency: "USD".to_string(),
                description: "SOL/USD".to_string(),
            },
        );

        Self {
            config,
            client,
            feeds,
        }
    }

    /// Get feed configuration
    pub fn get_feed(&self, feed_id: &str) -> Option<&PythFeed> {
        self.feeds.get(feed_id)
    }

    /// Add new feed
    pub fn add_feed(&mut self, feed_id: String, feed: PythFeed) {
        self.feeds.insert(feed_id, feed);
    }

    /// Remove feed
    pub fn remove_feed(&mut self, feed_id: &str) -> Option<PythFeed> {
        self.feeds.remove(feed_id)
    }

    /// Get latest price from Pyth Network
    pub async fn get_latest_price(&self, feed_id: &str) -> OracleResult<PythPriceFeedResponse> {
        let feed = self
            .get_feed(feed_id)
            .ok_or_else(|| OracleError::feed_not_found(feed_id.to_string()))?;

        // Mock Pyth Network API call
        let mock_response = self.mock_pyth_response(feed).await?;

        Ok(mock_response)
    }

    /// Get batch prices from Pyth Network
    pub async fn get_batch_prices(&self, feed_ids: &[String]) -> OracleResult<PythBatchResponse> {
        let mut responses = Vec::new();

        for feed_id in feed_ids {
            if let Ok(response) = self.get_latest_price(feed_id).await {
                responses.push(response);
            }
        }

        Ok(PythBatchResponse {
            parsed: responses,
            binary: PythBinaryData {
                encoding: "base64".to_string(),
                data: vec!["mock_binary_data".to_string()],
            },
        })
    }

    /// Get price by price ID
    pub async fn get_price_by_id(&self, price_id: &str) -> OracleResult<PythPriceFeedResponse> {
        // Find feed by price ID
        let feed_entry = self
            .feeds
            .iter()
            .find(|(_, feed)| feed.price_id == price_id)
            .ok_or_else(|| OracleError::feed_not_found(price_id.to_string()))?;

        self.get_latest_price(feed_entry.0).await
    }

    /// Get all available price feeds
    pub fn get_available_feeds(&self) -> Vec<String> {
        self.feeds.keys().cloned().collect()
    }

    /// Get publishers for a feed
    pub async fn get_publishers(&self, feed_id: &str) -> OracleResult<Vec<PythPublisher>> {
        let _feed = self
            .get_feed(feed_id)
            .ok_or_else(|| OracleError::feed_not_found(feed_id.to_string()))?;

        // Mock publisher data
        Ok(vec![
            PythPublisher {
                name: "Jump Crypto".to_string(),
                account: "5uGZbzQRjKCk7CVKBqKZByd2pFqrRAZymCsVFcvnyQHu".to_string(),
                aggregate_price_info: PythAggregateInfo {
                    price: "2000000000".to_string(),
                    conf: "1000000".to_string(),
                    status: "trading".to_string(),
                    corp_act: "nocorpact".to_string(),
                    publish_slot: 123456789,
                },
            },
            PythPublisher {
                name: "DRW Cumberland".to_string(),
                account: "AwVeBaGWNb4CGoQRKjUKYqZzUr6QCBWBrjZE1xfgEWYr".to_string(),
                aggregate_price_info: PythAggregateInfo {
                    price: "2001000000".to_string(),
                    conf: "1500000".to_string(),
                    status: "trading".to_string(),
                    corp_act: "nocorpact".to_string(),
                    publish_slot: 123456790,
                },
            },
        ])
    }

    /// Mock Pyth Network API response
    async fn mock_pyth_response(&self, feed: &PythFeed) -> OracleResult<PythPriceFeedResponse> {
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(75)).await;

        let (base_price, expo) = match feed.base.as_str() {
            "ETH" => ("200000000000", -8),  // $2000.00 with -8 exponent
            "BTC" => ("5000000000000", -8), // $50000.00 with -8 exponent
            "SOL" => ("10000000000", -8),   // $100.00 with -8 exponent
            _ => ("100000000000", -8),      // $1000.00 with -8 exponent
        };

        let confidence = (base_price.parse::<u64>().unwrap_or(0) / 1000).to_string(); // 0.1% confidence interval

        Ok(PythPriceFeedResponse {
            id: feed.price_id.clone(),
            price: PythPrice {
                price: base_price.to_string(),
                conf: confidence.clone(),
                expo,
                publish_time: Utc::now().timestamp(),
            },
            ema_price: PythEmaPrice {
                price: base_price.to_string(),
                conf: confidence,
                expo,
                publish_time: Utc::now().timestamp(),
            },
            metadata: PythMetadata {
                symbol: feed.symbol.clone(),
                asset_type: feed.asset_type.clone(),
                base: feed.base.clone(),
                quote_currency: feed.quote_currency.clone(),
                tenor: None,
                description: feed.description.clone(),
                country: None,
                quoter: None,
            },
            vaa: Some("mock_vaa_data".to_string()),
        })
    }

    /// Validate price data
    pub fn validate_price_data(
        &self,
        feed_id: &str,
        response: &PythPriceFeedResponse,
    ) -> OracleResult<bool> {
        // Check if price is valid
        let price_value = response.price.price.parse::<i64>().map_err(|_| {
            OracleError::invalid_price(
                feed_id,
                response.price.price.clone(),
                "Invalid price format",
            )
        })?;

        if price_value <= 0 {
            return Err(OracleError::invalid_price(
                feed_id,
                response.price.price.clone(),
                "Price must be positive",
            ));
        }

        // Check data freshness (Pyth updates very frequently, so use shorter threshold)
        let current_time = Utc::now().timestamp();
        if current_time - response.price.publish_time > 60 {
            // 1 minute threshold
            return Err(OracleError::stale_data(
                feed_id,
                (current_time - response.price.publish_time) as u64,
                60,
            ));
        }

        Ok(true)
    }

    /// Convert Pyth price to decimal
    pub fn convert_price_to_decimal(&self, price: &PythPrice) -> OracleResult<Decimal> {
        let price_value = price
            .price
            .parse::<i64>()
            .map_err(|_| OracleError::serialization_error("Invalid price format"))?;

        // Apply exponent (Pyth uses negative exponents)
        let decimal_places = (-price.expo) as u32;
        Ok(Decimal::new(price_value, decimal_places))
    }

    /// Calculate confidence interval
    pub fn calculate_confidence(&self, price: &PythPrice) -> OracleResult<f64> {
        let price_value = price
            .price
            .parse::<f64>()
            .map_err(|_| OracleError::serialization_error("Invalid price format"))?;
        let conf_value = price
            .conf
            .parse::<f64>()
            .map_err(|_| OracleError::serialization_error("Invalid confidence format"))?;

        if price_value == 0.0 {
            return Ok(0.0);
        }

        // Calculate confidence as percentage
        let confidence_percentage = (1.0 - (conf_value / price_value)) * 100.0;
        Ok(confidence_percentage.max(0.0).min(100.0))
    }
}

#[async_trait]
impl ProviderClient for PythOracle {
    async fn get_price(&self, feed_id: &str) -> OracleResult<PriceData> {
        let response = self.get_latest_price(feed_id).await?;

        // Validate the data
        self.validate_price_data(feed_id, &response)?;

        // Convert price to decimal
        let price = self.convert_price_to_decimal(&response.price)?;

        // Calculate confidence
        let confidence = self.calculate_confidence(&response.price)? / 100.0; // Convert to 0-1 range

        let publish_time =
            chrono::DateTime::from_timestamp(response.price.publish_time, 0).unwrap_or(Utc::now());

        Ok(PriceData {
            price,
            timestamp: publish_time,
            source: OracleProvider::PythNetwork,
            confidence,
            volume: None,
            market_cap: None,
            deviation: None,
            round_id: None, // Pyth doesn't use round IDs like Chainlink
        })
    }

    async fn get_health(&self) -> OracleResult<crate::types::ProviderHealth> {
        // Mock health check - in reality, this would check Pyth Network status
        Ok(crate::types::ProviderHealth {
            status: crate::types::HealthStatus::Healthy,
            last_successful_update: Utc::now(),
            consecutive_failures: 0,
            response_time_ms: 75,
            success_rate: 99.8,
        })
    }

    fn provider(&self) -> OracleProvider {
        OracleProvider::PythNetwork
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pyth_oracle_creation() {
        let config = ProviderConfig {
            provider: OracleProvider::PythNetwork,
            enabled: true,
            api_key: None,
            endpoint: "https://hermes.pyth.network".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 3,
        };

        let oracle = PythOracle::new(config);
        assert!(oracle.get_feed("ETH/USD").is_some());
        assert!(oracle.get_feed("BTC/USD").is_some());
        assert!(oracle.get_feed("SOL/USD").is_some());
    }

    #[tokio::test]
    async fn test_pyth_price_fetch() {
        let config = ProviderConfig {
            provider: OracleProvider::PythNetwork,
            enabled: true,
            api_key: None,
            endpoint: "https://hermes.pyth.network".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 3,
        };

        let oracle = PythOracle::new(config);
        let price = oracle.get_price("ETH/USD").await.unwrap();

        assert!(price.price > Decimal::ZERO);
        assert_eq!(price.source, OracleProvider::PythNetwork);
        assert!(price.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_pyth_latest_price() {
        let config = ProviderConfig {
            provider: OracleProvider::PythNetwork,
            enabled: true,
            api_key: None,
            endpoint: "https://hermes.pyth.network".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 3,
        };

        let oracle = PythOracle::new(config);
        let response = oracle.get_latest_price("ETH/USD").await.unwrap();

        assert!(!response.id.is_empty());
        assert!(response.price.price.parse::<i64>().unwrap() > 0);
        assert!(response.price.publish_time > 0);
    }

    #[tokio::test]
    async fn test_pyth_batch_prices() {
        let config = ProviderConfig {
            provider: OracleProvider::PythNetwork,
            enabled: true,
            api_key: None,
            endpoint: "https://hermes.pyth.network".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 3,
        };

        let oracle = PythOracle::new(config);
        let feed_ids = vec!["ETH/USD".to_string(), "BTC/USD".to_string()];
        let batch_response = oracle.get_batch_prices(&feed_ids).await.unwrap();

        assert!(!batch_response.parsed.is_empty());
        assert_eq!(batch_response.binary.encoding, "base64");
    }

    #[tokio::test]
    async fn test_pyth_price_conversion() {
        let config = ProviderConfig {
            provider: OracleProvider::PythNetwork,
            enabled: true,
            api_key: None,
            endpoint: "https://hermes.pyth.network".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 3,
        };

        let oracle = PythOracle::new(config);

        let price = PythPrice {
            price: "200000000000".to_string(),
            conf: "1000000".to_string(),
            expo: -8,
            publish_time: Utc::now().timestamp(),
        };

        let decimal_price = oracle.convert_price_to_decimal(&price).unwrap();
        assert_eq!(decimal_price, Decimal::new(2000, 0)); // $2000.00

        let confidence = oracle.calculate_confidence(&price).unwrap();
        assert!(confidence > 99.0); // Should be very high confidence
    }

    #[tokio::test]
    async fn test_pyth_publishers() {
        let config = ProviderConfig {
            provider: OracleProvider::PythNetwork,
            enabled: true,
            api_key: None,
            endpoint: "https://hermes.pyth.network".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 3,
        };

        let oracle = PythOracle::new(config);
        let publishers = oracle.get_publishers("ETH/USD").await.unwrap();

        assert!(!publishers.is_empty());
        assert!(publishers.iter().any(|p| p.name == "Jump Crypto"));
        assert!(publishers.iter().any(|p| p.name == "DRW Cumberland"));
    }
}
