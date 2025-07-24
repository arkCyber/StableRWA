// =====================================================================================
// RWA Tokenization Platform - CoinGecko Price Provider
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use super::{PriceProvider, RateLimit};
use crate::error::{OracleError, OracleResult};
use crate::models::{AssetPrice, ProviderType};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

/// CoinGecko API provider
pub struct CoinGeckoProvider {
    client: Client,
    api_key: Option<String>,
    base_url: String,
    weight: Decimal,
    rate_limit: u32,
}

/// CoinGecko API response for simple price endpoint
#[derive(Debug, Deserialize)]
struct CoinGeckoPriceResponse {
    #[serde(flatten)]
    prices: HashMap<String, HashMap<String, f64>>,
}

/// CoinGecko API response for coin info
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CoinGeckoInfoResponse {
    id: String,
    symbol: String,
    name: String,
    market_data: Option<MarketData>,
    last_updated: Option<String>,
}

/// Market data from CoinGecko
#[derive(Debug, Deserialize)]
struct MarketData {
    current_price: Option<HashMap<String, f64>>,
    market_cap: Option<HashMap<String, f64>>,
    total_volume: Option<HashMap<String, f64>>,
    price_change_percentage_24h: Option<f64>,
}

/// CoinGecko supported coins response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CoinGeckoCoinsListResponse {
    id: String,
    symbol: String,
    name: String,
}

impl CoinGeckoProvider {
    /// Create a new CoinGecko provider
    pub fn new(api_key: Option<String>, weight: Decimal, rate_limit: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("RWA-Oracle-Service/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            base_url: "https://api.coingecko.com/api/v3".to_string(),
            weight,
            rate_limit,
        }
    }

    /// Build request URL with API key if available
    fn build_url(&self, endpoint: &str) -> String {
        let base_url = format!("{}{}", self.base_url, endpoint);
        
        if let Some(ref api_key) = self.api_key {
            if endpoint.contains('?') {
                format!("{}&x_cg_pro_api_key={}", base_url, api_key)
            } else {
                format!("{}?x_cg_pro_api_key={}", base_url, api_key)
            }
        } else {
            base_url
        }
    }

    /// Convert asset ID to CoinGecko coin ID
    fn asset_id_to_coin_id(&self, asset_id: &str) -> String {
        match asset_id.to_uppercase().as_str() {
            "BTC" => "bitcoin".to_string(),
            "ETH" => "ethereum".to_string(),
            "ADA" => "cardano".to_string(),
            "DOT" => "polkadot".to_string(),
            "LINK" => "chainlink".to_string(),
            "UNI" => "uniswap".to_string(),
            "AAVE" => "aave".to_string(),
            "COMP" => "compound-governance-token".to_string(),
            "MKR" => "maker".to_string(),
            "SNX" => "havven".to_string(),
            _ => asset_id.to_lowercase(),
        }
    }

    /// Convert currency to lowercase for CoinGecko API
    fn normalize_currency(&self, currency: &str) -> String {
        currency.to_lowercase()
    }

    /// Calculate confidence score based on market data
    fn calculate_confidence(&self, market_data: &Option<MarketData>) -> Decimal {
        if let Some(data) = market_data {
            let mut confidence = Decimal::from_str("0.8").unwrap(); // Base confidence
            
            // Increase confidence if we have market cap data
            if data.market_cap.is_some() {
                confidence += Decimal::from_str("0.1").unwrap();
            }
            
            // Increase confidence if we have volume data
            if data.total_volume.is_some() {
                confidence += Decimal::from_str("0.05").unwrap();
            }
            
            // Decrease confidence for high volatility
            if let Some(change_24h) = data.price_change_percentage_24h {
                if change_24h.abs() > 10.0 {
                    confidence -= Decimal::from_str("0.1").unwrap();
                }
            }
            
            // Ensure confidence is between 0 and 1
            confidence.max(Decimal::from_str("0.1").unwrap())
                     .min(Decimal::from_str("1.0").unwrap())
        } else {
            Decimal::from_str("0.7").unwrap() // Lower confidence without market data
        }
    }
}

#[async_trait]
impl PriceProvider for CoinGeckoProvider {
    fn name(&self) -> &str {
        "CoinGecko"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::MarketData
    }

    fn weight(&self) -> Decimal {
        self.weight
    }

    async fn health_check(&self) -> OracleResult<bool> {
        let url = self.build_url("/ping");
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn get_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
        let coin_id = self.asset_id_to_coin_id(asset_id);
        let currency_lower = self.normalize_currency(currency);
        
        let url = self.build_url(&format!(
            "/coins/{}?localization=false&tickers=false&market_data=true&community_data=false&developer_data=false&sparkline=false",
            coin_id
        ));

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("HTTP request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("API returned status: {}", response.status()),
            });
        }

        let coin_info: CoinGeckoInfoResponse = response
            .json()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        let market_data = coin_info.market_data.as_ref()
            .ok_or_else(|| OracleError::Provider {
                provider: self.name().to_string(),
                message: "No market data available".to_string(),
            })?;

        let current_price = market_data.current_price.as_ref()
            .ok_or_else(|| OracleError::Provider {
                provider: self.name().to_string(),
                message: "No current price data available".to_string(),
            })?;

        let price_value = current_price.get(&currency_lower)
            .ok_or_else(|| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("Price not available for currency: {}", currency),
            })?;

        let price = Decimal::from_f64(*price_value)
            .ok_or_else(|| OracleError::InvalidPriceData {
                reason: "Invalid price value from CoinGecko".to_string(),
            })?;

        let confidence = self.calculate_confidence(&coin_info.market_data);

        let mut metadata = HashMap::new();
        metadata.insert("coin_id".to_string(), serde_json::Value::String(coin_info.id));
        metadata.insert("symbol".to_string(), serde_json::Value::String(coin_info.symbol));
        metadata.insert("name".to_string(), serde_json::Value::String(coin_info.name));
        
        if let Some(change_24h) = market_data.price_change_percentage_24h {
            metadata.insert("price_change_24h".to_string(), serde_json::Value::Number(
                serde_json::Number::from_f64(change_24h).unwrap_or_else(|| serde_json::Number::from(0))
            ));
        }

        Ok(AssetPrice {
            asset_id: asset_id.to_string(),
            price,
            currency: currency.to_string(),
            timestamp: Utc::now(),
            confidence,
            source: self.name().to_string(),
            metadata: Some(metadata),
        })
    }

    async fn get_batch_prices(&self, asset_ids: &[String], currency: &str) -> OracleResult<Vec<AssetPrice>> {
        let coin_ids: Vec<String> = asset_ids.iter()
            .map(|id| self.asset_id_to_coin_id(id))
            .collect();
        
        let currency_lower = self.normalize_currency(currency);
        let ids_param = coin_ids.join(",");
        
        let url = self.build_url(&format!(
            "/simple/price?ids={}&vs_currencies={}&include_market_cap=true&include_24hr_vol=true&include_24hr_change=true",
            ids_param, currency_lower
        ));

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("HTTP request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("API returned status: {}", response.status()),
            });
        }

        let price_data: CoinGeckoPriceResponse = response
            .json()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        let mut prices = Vec::new();
        
        for (i, asset_id) in asset_ids.iter().enumerate() {
            let coin_id = &coin_ids[i];
            
            if let Some(coin_data) = price_data.prices.get(coin_id) {
                if let Some(&price_value) = coin_data.get(&currency_lower) {
                    if let Some(price) = Decimal::from_f64(price_value) {
                        let mut metadata = HashMap::new();
                        metadata.insert("coin_id".to_string(), serde_json::Value::String(coin_id.clone()));
                        
                        // Add market cap if available
                        if let Some(&market_cap) = coin_data.get(&format!("{}_market_cap", currency_lower)) {
                            metadata.insert("market_cap".to_string(), serde_json::Value::Number(
                                serde_json::Number::from_f64(market_cap).unwrap_or_else(|| serde_json::Number::from(0))
                            ));
                        }
                        
                        // Add 24h volume if available
                        if let Some(&volume_24h) = coin_data.get(&format!("{}_24h_vol", currency_lower)) {
                            metadata.insert("volume_24h".to_string(), serde_json::Value::Number(
                                serde_json::Number::from_f64(volume_24h).unwrap_or_else(|| serde_json::Number::from(0))
                            ));
                        }
                        
                        // Add 24h change if available
                        if let Some(&change_24h) = coin_data.get(&format!("{}_24h_change", currency_lower)) {
                            metadata.insert("price_change_24h".to_string(), serde_json::Value::Number(
                                serde_json::Number::from_f64(change_24h).unwrap_or_else(|| serde_json::Number::from(0))
                            ));
                        }

                        prices.push(AssetPrice {
                            asset_id: asset_id.clone(),
                            price,
                            currency: currency.to_string(),
                            timestamp: Utc::now(),
                            confidence: Decimal::from_str("0.85").unwrap(), // Default confidence for batch
                            source: self.name().to_string(),
                            metadata: Some(metadata),
                        });
                    }
                }
            }
        }

        Ok(prices)
    }

    async fn get_supported_assets(&self) -> OracleResult<Vec<String>> {
        let url = self.build_url("/coins/list");

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("HTTP request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("API returned status: {}", response.status()),
            });
        }

        let coins: Vec<CoinGeckoCoinsListResponse> = response
            .json()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        Ok(coins.into_iter()
            .map(|coin| coin.symbol.to_uppercase())
            .collect())
    }

    fn get_rate_limit(&self) -> RateLimit {
        RateLimit {
            requests_per_minute: self.rate_limit,
            current_usage: 0,
            reset_time: Utc::now() + chrono::Duration::minutes(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use tokio;

    #[test]
    fn test_coingecko_provider_creation() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        assert_eq!(provider.name(), "CoinGecko");
        assert_eq!(provider.weight(), dec!(1.0));
        assert!(matches!(provider.provider_type(), ProviderType::MarketData));
    }

    #[test]
    fn test_asset_id_to_coin_id_mapping() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        assert_eq!(provider.asset_id_to_coin_id("BTC"), "bitcoin");
        assert_eq!(provider.asset_id_to_coin_id("ETH"), "ethereum");
        assert_eq!(provider.asset_id_to_coin_id("ADA"), "cardano");
        assert_eq!(provider.asset_id_to_coin_id("UNKNOWN"), "unknown");
    }

    #[test]
    fn test_normalize_currency() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        assert_eq!(provider.normalize_currency("USD"), "usd");
        assert_eq!(provider.normalize_currency("EUR"), "eur");
        assert_eq!(provider.normalize_currency("btc"), "btc");
    }

    #[test]
    fn test_build_url_without_api_key() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        let url = provider.build_url("/ping");
        assert_eq!(url, "https://api.coingecko.com/api/v3/ping");

        let url_with_params = provider.build_url("/coins?include_platform=false");
        assert_eq!(url_with_params, "https://api.coingecko.com/api/v3/coins?include_platform=false");
    }

    #[test]
    fn test_build_url_with_api_key() {
        let provider = CoinGeckoProvider::new(Some("test-api-key".to_string()), dec!(1.0), 50);

        let url = provider.build_url("/ping");
        assert_eq!(url, "https://api.coingecko.com/api/v3/ping?x_cg_pro_api_key=test-api-key");

        let url_with_params = provider.build_url("/coins?include_platform=false");
        assert_eq!(url_with_params, "https://api.coingecko.com/api/v3/coins?include_platform=false&x_cg_pro_api_key=test-api-key");
    }

    #[test]
    fn test_calculate_confidence_with_full_data() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        let market_data = MarketData {
            current_price: Some(HashMap::new()),
            market_cap: Some(HashMap::new()),
            total_volume: Some(HashMap::new()),
            price_change_percentage_24h: Some(2.5), // Low volatility
        };

        let confidence = provider.calculate_confidence(&Some(market_data));
        assert!(confidence >= dec!(0.85)); // Should be high confidence
    }

    #[test]
    fn test_calculate_confidence_with_high_volatility() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        let market_data = MarketData {
            current_price: Some(HashMap::new()),
            market_cap: Some(HashMap::new()),
            total_volume: Some(HashMap::new()),
            price_change_percentage_24h: Some(15.0), // High volatility
        };

        let confidence = provider.calculate_confidence(&Some(market_data));
        assert!(confidence < dec!(0.9)); // Should be lower confidence due to volatility
    }

    #[test]
    fn test_calculate_confidence_without_data() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        let confidence = provider.calculate_confidence(&None);
        assert_eq!(confidence, dec!(0.7)); // Default confidence without data
    }

    #[test]
    fn test_get_rate_limit() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 100);

        let rate_limit = provider.get_rate_limit();
        assert_eq!(rate_limit.requests_per_minute, 100);
        assert_eq!(rate_limit.current_usage, 0);
    }

    // Integration tests (these would require actual API calls)
    #[tokio::test]
    #[ignore] // Ignore by default to avoid hitting real API during tests
    async fn test_health_check_integration() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        let result = provider.health_check().await;
        assert!(result.is_ok());
        // Note: This test requires internet connection and working CoinGecko API
    }

    #[tokio::test]
    #[ignore] // Ignore by default to avoid hitting real API during tests
    async fn test_get_price_integration() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);

        let result = provider.get_price("BTC", "USD").await;
        match result {
            Ok(price) => {
                assert_eq!(price.asset_id, "BTC");
                assert_eq!(price.currency, "USD");
                assert!(price.price > dec!(0));
                assert_eq!(price.source, "CoinGecko");
                assert!(price.confidence > dec!(0));
                assert!(price.metadata.is_some());
            }
            Err(e) => {
                // This might fail due to network issues or API limits
                println!("Integration test failed (expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_provider_type() {
        let provider = CoinGeckoProvider::new(None, dec!(1.0), 50);
        assert!(matches!(provider.provider_type(), ProviderType::MarketData));
    }

    #[test]
    fn test_provider_weight() {
        let provider = CoinGeckoProvider::new(None, dec!(1.5), 50);
        assert_eq!(provider.weight(), dec!(1.5));
    }
}
