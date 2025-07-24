// =====================================================================================
// File: core-oracle/src/band.rs
// Description: Band Protocol oracle integration module
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

/// Band Protocol oracle client
pub struct BandOracle {
    config: ProviderConfig,
    client: reqwest::Client,
    feeds: HashMap<String, BandFeed>,
}

/// Band Protocol price feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandFeed {
    pub symbol: String,
    pub oracle_script_id: u64,
    pub multiplier: u64,
    pub min_count: u64,
    pub ask_count: u64,
    pub client_id: String,
}

/// Band Protocol data request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandRequest {
    pub oracle_script_id: u64,
    pub calldata: String,
    pub ask_count: u64,
    pub min_count: u64,
    pub client_id: String,
    pub fee_limit: Vec<BandCoin>,
    pub prepare_gas: u64,
    pub execute_gas: u64,
}

/// Band Protocol coin structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandCoin {
    pub denom: String,
    pub amount: String,
}

/// Band Protocol price response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandPriceResponse {
    pub price: String,
    pub multiplier: u64,
    pub px: u64,
    pub request_id: u64,
    pub resolve_time: u64,
}

/// Band Protocol oracle script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandOracleScript {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub schema: String,
    pub source_code_url: String,
}

/// Band Protocol data source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandDataSource {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub fee: Vec<BandCoin>,
}

/// Band Protocol validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandValidator {
    pub operator_address: String,
    pub consensus_pubkey: String,
    pub jailed: bool,
    pub status: String,
    pub tokens: String,
    pub delegator_shares: String,
    pub description: BandValidatorDescription,
    pub commission: BandCommission,
}

/// Band Protocol validator description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandValidatorDescription {
    pub moniker: String,
    pub identity: String,
    pub website: String,
    pub security_contact: String,
    pub details: String,
}

/// Band Protocol commission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandCommission {
    pub commission_rates: BandCommissionRates,
    pub update_time: String,
}

/// Band Protocol commission rates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandCommissionRates {
    pub rate: String,
    pub max_rate: String,
    pub max_change_rate: String,
}

impl BandOracle {
    pub fn new(config: ProviderConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        let mut feeds = HashMap::new();

        // Add some common Band Protocol feeds
        feeds.insert(
            "ETH".to_string(),
            BandFeed {
                symbol: "ETH".to_string(),
                oracle_script_id: 37,
                multiplier: 1000000000,
                min_count: 10,
                ask_count: 16,
                client_id: "from_pyband".to_string(),
            },
        );

        feeds.insert(
            "BTC".to_string(),
            BandFeed {
                symbol: "BTC".to_string(),
                oracle_script_id: 37,
                multiplier: 1000000000,
                min_count: 10,
                ask_count: 16,
                client_id: "from_pyband".to_string(),
            },
        );

        Self {
            config,
            client,
            feeds,
        }
    }

    /// Get feed configuration
    pub fn get_feed(&self, symbol: &str) -> Option<&BandFeed> {
        self.feeds.get(symbol)
    }

    /// Add new feed
    pub fn add_feed(&mut self, feed: BandFeed) {
        self.feeds.insert(feed.symbol.clone(), feed);
    }

    /// Remove feed
    pub fn remove_feed(&mut self, symbol: &str) -> Option<BandFeed> {
        self.feeds.remove(symbol)
    }

    /// Get latest price from Band Protocol
    pub async fn get_latest_price(&self, symbol: &str) -> OracleResult<BandPriceResponse> {
        let feed = self
            .get_feed(symbol)
            .ok_or_else(|| OracleError::feed_not_found(symbol.to_string()))?;

        // Mock Band Protocol API call
        let mock_response = self.mock_band_response(feed).await?;

        Ok(mock_response)
    }

    /// Create data request
    pub async fn create_request(&self, symbol: &str) -> OracleResult<BandRequest> {
        let feed = self
            .get_feed(symbol)
            .ok_or_else(|| OracleError::feed_not_found(symbol.to_string()))?;

        // Encode symbol for calldata
        let calldata = hex::encode(symbol.as_bytes());

        Ok(BandRequest {
            oracle_script_id: feed.oracle_script_id,
            calldata,
            ask_count: feed.ask_count,
            min_count: feed.min_count,
            client_id: feed.client_id.clone(),
            fee_limit: vec![BandCoin {
                denom: "uband".to_string(),
                amount: "100000".to_string(),
            }],
            prepare_gas: 50000,
            execute_gas: 300000,
        })
    }

    /// Get oracle script information
    pub async fn get_oracle_script(&self, script_id: u64) -> OracleResult<BandOracleScript> {
        // Mock oracle script data
        Ok(BandOracleScript {
            id: script_id,
            name: "Cryptocurrency Price in USD".to_string(),
            description: "Oracle script for getting cryptocurrency prices in USD".to_string(),
            filename: "crypto_price.py".to_string(),
            schema: "{symbol:string}/{px:u64}".to_string(),
            source_code_url: "https://ipfs.io/ipfs/QmQqzMTavQgT4f4T5v6PWBp7XNKtoPmC9jvn12WPT3gkSE"
                .to_string(),
        })
    }

    /// Get data sources
    pub async fn get_data_sources(&self) -> OracleResult<Vec<BandDataSource>> {
        // Mock data sources
        Ok(vec![
            BandDataSource {
                id: 1,
                name: "CoinGecko".to_string(),
                description: "Cryptocurrency data from CoinGecko".to_string(),
                filename: "coingecko.py".to_string(),
                fee: vec![BandCoin {
                    denom: "uband".to_string(),
                    amount: "1000".to_string(),
                }],
            },
            BandDataSource {
                id: 2,
                name: "CoinMarketCap".to_string(),
                description: "Cryptocurrency data from CoinMarketCap".to_string(),
                filename: "coinmarketcap.py".to_string(),
                fee: vec![BandCoin {
                    denom: "uband".to_string(),
                    amount: "1000".to_string(),
                }],
            },
        ])
    }

    /// Get validators
    pub async fn get_validators(&self) -> OracleResult<Vec<BandValidator>> {
        // Mock validator data
        Ok(vec![BandValidator {
            operator_address: "bandvaloper1p40yh3zkmhcv0ecqp3mcazy83sa57rgjde6wec".to_string(),
            consensus_pubkey:
                "bandvalconspub1zcjduepq0vu2zgkgk26ehru95gs4c3q9mqzurmkn02119kmqx4n89qwv2f3qrqsdf7"
                    .to_string(),
            jailed: false,
            status: "BOND_STATUS_BONDED".to_string(),
            tokens: "1000000000000".to_string(),
            delegator_shares: "1000000000000.000000000000000000".to_string(),
            description: BandValidatorDescription {
                moniker: "BandChain Validator".to_string(),
                identity: "".to_string(),
                website: "https://bandprotocol.com".to_string(),
                security_contact: "".to_string(),
                details: "Official Band Protocol validator".to_string(),
            },
            commission: BandCommission {
                commission_rates: BandCommissionRates {
                    rate: "0.100000000000000000".to_string(),
                    max_rate: "0.200000000000000000".to_string(),
                    max_change_rate: "0.010000000000000000".to_string(),
                },
                update_time: "2021-01-01T00:00:00Z".to_string(),
            },
        }])
    }

    /// Mock Band Protocol API response
    async fn mock_band_response(&self, feed: &BandFeed) -> OracleResult<BandPriceResponse> {
        // Simulate network delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let base_price = match feed.symbol.as_str() {
            "ETH" => 2000_000000000_u64,  // $2000 with 9 decimals
            "BTC" => 50000_000000000_u64, // $50000 with 9 decimals
            "USDC" => 1_000000000_u64,    // $1 with 9 decimals
            _ => 100_000000000_u64,       // $100 with 9 decimals
        };

        Ok(BandPriceResponse {
            price: base_price.to_string(),
            multiplier: feed.multiplier,
            px: base_price,
            request_id: 12345,
            resolve_time: Utc::now().timestamp() as u64,
        })
    }

    /// Validate price data
    pub fn validate_price_data(
        &self,
        symbol: &str,
        response: &BandPriceResponse,
    ) -> OracleResult<bool> {
        let feed = self
            .get_feed(symbol)
            .ok_or_else(|| OracleError::feed_not_found(symbol.to_string()))?;

        // Check if price is valid
        if response.px == 0 {
            return Err(OracleError::invalid_price(
                symbol,
                response.px.to_string(),
                "Price cannot be zero",
            ));
        }

        // Check resolve time (data freshness)
        let current_time = Utc::now().timestamp() as u64;
        if current_time - response.resolve_time > 3600 {
            // 1 hour threshold
            return Err(OracleError::stale_data(
                symbol,
                current_time - response.resolve_time,
                3600,
            ));
        }

        Ok(true)
    }
}

#[async_trait]
impl ProviderClient for BandOracle {
    async fn get_price(&self, feed_id: &str) -> OracleResult<PriceData> {
        // Extract symbol from feed_id (e.g., "ETH/USD" -> "ETH")
        let symbol = feed_id.split('/').next().unwrap_or(feed_id);

        let response = self.get_latest_price(symbol).await?;

        // Validate the data
        self.validate_price_data(symbol, &response)?;

        let feed = self
            .get_feed(symbol)
            .ok_or_else(|| OracleError::feed_not_found(symbol.to_string()))?;

        // Convert price with proper decimals
        let price = Decimal::new(response.px as i64, 9); // Band uses 9 decimals

        let resolve_time =
            chrono::DateTime::from_timestamp(response.resolve_time as i64, 0).unwrap_or(Utc::now());

        Ok(PriceData {
            price,
            timestamp: resolve_time,
            source: OracleProvider::BandProtocol,
            confidence: 0.90, // Band Protocol typically has good confidence
            volume: None,
            market_cap: None,
            deviation: None,
            round_id: Some(response.request_id),
        })
    }

    async fn get_health(&self) -> OracleResult<crate::types::ProviderHealth> {
        // Mock health check - in reality, this would check Band Protocol node status
        Ok(crate::types::ProviderHealth {
            status: crate::types::HealthStatus::Healthy,
            last_successful_update: Utc::now(),
            consecutive_failures: 0,
            response_time_ms: 100,
            success_rate: 99.5,
        })
    }

    fn provider(&self) -> OracleProvider {
        OracleProvider::BandProtocol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_band_oracle_creation() {
        let config = ProviderConfig {
            provider: OracleProvider::BandProtocol,
            enabled: true,
            api_key: None,
            endpoint: "https://laozi1.bandchain.org".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 2,
        };

        let oracle = BandOracle::new(config);
        assert!(oracle.get_feed("ETH").is_some());
        assert!(oracle.get_feed("BTC").is_some());
    }

    #[tokio::test]
    async fn test_band_price_fetch() {
        let config = ProviderConfig {
            provider: OracleProvider::BandProtocol,
            enabled: true,
            api_key: None,
            endpoint: "https://laozi1.bandchain.org".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 2,
        };

        let oracle = BandOracle::new(config);
        let price = oracle.get_price("ETH/USD").await.unwrap();

        assert!(price.price > Decimal::ZERO);
        assert_eq!(price.source, OracleProvider::BandProtocol);
        assert!(price.confidence > 0.0);
        assert!(price.round_id.is_some());
    }

    #[tokio::test]
    async fn test_band_latest_price() {
        let config = ProviderConfig {
            provider: OracleProvider::BandProtocol,
            enabled: true,
            api_key: None,
            endpoint: "https://laozi1.bandchain.org".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 2,
        };

        let oracle = BandOracle::new(config);
        let response = oracle.get_latest_price("ETH").await.unwrap();

        assert!(response.px > 0);
        assert!(response.request_id > 0);
        assert!(response.resolve_time > 0);
    }

    #[tokio::test]
    async fn test_band_request_creation() {
        let config = ProviderConfig {
            provider: OracleProvider::BandProtocol,
            enabled: true,
            api_key: None,
            endpoint: "https://laozi1.bandchain.org".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 2,
        };

        let oracle = BandOracle::new(config);
        let request = oracle.create_request("ETH").await.unwrap();

        assert_eq!(request.oracle_script_id, 37);
        assert_eq!(request.ask_count, 16);
        assert_eq!(request.min_count, 10);
        assert!(!request.calldata.is_empty());
    }

    #[tokio::test]
    async fn test_band_oracle_script() {
        let config = ProviderConfig {
            provider: OracleProvider::BandProtocol,
            enabled: true,
            api_key: None,
            endpoint: "https://laozi1.bandchain.org".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 2,
        };

        let oracle = BandOracle::new(config);
        let script = oracle.get_oracle_script(37).await.unwrap();

        assert_eq!(script.id, 37);
        assert!(!script.name.is_empty());
        assert!(!script.description.is_empty());
    }

    #[tokio::test]
    async fn test_band_data_sources() {
        let config = ProviderConfig {
            provider: OracleProvider::BandProtocol,
            enabled: true,
            api_key: None,
            endpoint: "https://laozi1.bandchain.org".to_string(),
            timeout_seconds: 30,
            rate_limit_per_second: 10,
            weight: 1.0,
            priority: 2,
        };

        let oracle = BandOracle::new(config);
        let sources = oracle.get_data_sources().await.unwrap();

        assert!(!sources.is_empty());
        assert!(sources.iter().any(|s| s.name == "CoinGecko"));
        assert!(sources.iter().any(|s| s.name == "CoinMarketCap"));
    }
}
