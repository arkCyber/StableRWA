// =====================================================================================
// RWA Tokenization Platform - Binance Price Provider
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
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

/// Binance API provider
pub struct BinanceProvider {
    client: Client,
    base_url: String,
    weight: Decimal,
    rate_limit: u32,
}

/// Binance API response for ticker price
#[derive(Debug, Deserialize)]
struct BinanceTickerResponse {
    symbol: String,
    price: String,
}

/// Binance API response for 24hr ticker statistics
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Binance24hrTickerResponse {
    symbol: String,
    #[serde(rename = "priceChange")]
    price_change: String,
    #[serde(rename = "priceChangePercent")]
    price_change_percent: String,
    #[serde(rename = "weightedAvgPrice")]
    weighted_avg_price: String,
    #[serde(rename = "prevClosePrice")]
    prev_close_price: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
    #[serde(rename = "lastQty")]
    last_qty: String,
    #[serde(rename = "bidPrice")]
    bid_price: String,
    #[serde(rename = "askPrice")]
    ask_price: String,
    #[serde(rename = "openPrice")]
    open_price: String,
    #[serde(rename = "highPrice")]
    high_price: String,
    #[serde(rename = "lowPrice")]
    low_price: String,
    volume: String,
    #[serde(rename = "quoteVolume")]
    quote_volume: String,
    #[serde(rename = "openTime")]
    open_time: u64,
    #[serde(rename = "closeTime")]
    close_time: u64,
    #[serde(rename = "firstId")]
    first_id: u64,
    #[serde(rename = "lastId")]
    last_id: u64,
    count: u64,
}

/// Binance exchange info response
#[derive(Debug, Deserialize)]
struct BinanceExchangeInfoResponse {
    symbols: Vec<BinanceSymbolInfo>,
}

/// Binance symbol information
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BinanceSymbolInfo {
    symbol: String,
    status: String,
    #[serde(rename = "baseAsset")]
    base_asset: String,
    #[serde(rename = "quoteAsset")]
    quote_asset: String,
}

impl BinanceProvider {
    /// Create a new Binance provider
    pub fn new(weight: Decimal, rate_limit: u32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("RWA-Oracle-Service/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://api.binance.com/api/v3".to_string(),
            weight,
            rate_limit,
        }
    }

    /// Convert asset ID and currency to Binance symbol format
    fn to_binance_symbol(&self, asset_id: &str, currency: &str) -> String {
        format!("{}{}", asset_id.to_uppercase(), currency.to_uppercase())
    }

    /// Parse Binance symbol to get base and quote assets
    #[allow(dead_code)]
    fn parse_binance_symbol(&self, symbol: &str) -> Option<(String, String)> {
        // Common quote assets in order of preference
        let quote_assets = vec!["USDT", "BUSD", "USD", "BTC", "ETH", "BNB"];
        
        for quote in &quote_assets {
            if symbol.ends_with(quote) && symbol.len() > quote.len() {
                let base = &symbol[..symbol.len() - quote.len()];
                return Some((base.to_string(), quote.to_string()));
            }
        }
        
        None
    }

    /// Calculate confidence based on market data
    fn calculate_confidence(&self, ticker_data: &Binance24hrTickerResponse) -> Decimal {
        let mut confidence = Decimal::from_str("0.85").unwrap(); // Base confidence for Binance
        
        // Increase confidence based on volume
        if let Ok(volume) = Decimal::from_str(&ticker_data.volume) {
            if volume > Decimal::from_str("1000").unwrap() {
                confidence += Decimal::from_str("0.05").unwrap();
            }
        }
        
        // Decrease confidence for high volatility
        if let Ok(change_percent) = Decimal::from_str(&ticker_data.price_change_percent) {
            if change_percent.abs() > Decimal::from_str("10").unwrap() {
                confidence -= Decimal::from_str("0.1").unwrap();
            }
        }
        
        // Ensure confidence is between 0 and 1
        confidence.max(Decimal::from_str("0.1").unwrap())
                 .min(Decimal::from_str("1.0").unwrap())
    }
}

#[async_trait]
impl PriceProvider for BinanceProvider {
    fn name(&self) -> &str {
        "Binance"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::CentralizedExchange
    }

    fn weight(&self) -> Decimal {
        self.weight
    }

    async fn health_check(&self) -> OracleResult<bool> {
        let url = format!("{}/ping", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    async fn get_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
        let symbol = self.to_binance_symbol(asset_id, currency);
        
        // Get 24hr ticker statistics for more comprehensive data
        let url = format!("{}/ticker/24hr?symbol={}", self.base_url, symbol);

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

        let ticker_data: Binance24hrTickerResponse = response
            .json()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        let price = Decimal::from_str(&ticker_data.last_price)
            .map_err(|_| OracleError::InvalidPriceData {
                reason: "Invalid price value from Binance".to_string(),
            })?;

        let confidence = self.calculate_confidence(&ticker_data);

        let mut metadata = HashMap::new();
        metadata.insert("symbol".to_string(), serde_json::Value::String(ticker_data.symbol));
        metadata.insert("volume".to_string(), serde_json::Value::String(ticker_data.volume));
        metadata.insert("quote_volume".to_string(), serde_json::Value::String(ticker_data.quote_volume));
        metadata.insert("price_change_24h".to_string(), serde_json::Value::String(ticker_data.price_change));
        metadata.insert("price_change_percent_24h".to_string(), serde_json::Value::String(ticker_data.price_change_percent));
        metadata.insert("high_24h".to_string(), serde_json::Value::String(ticker_data.high_price));
        metadata.insert("low_24h".to_string(), serde_json::Value::String(ticker_data.low_price));
        metadata.insert("weighted_avg_price".to_string(), serde_json::Value::String(ticker_data.weighted_avg_price));

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
        let symbols: Vec<String> = asset_ids.iter()
            .map(|id| self.to_binance_symbol(id, currency))
            .collect();

        // Binance allows batch requests for ticker prices
        let url = format!("{}/ticker/price", self.base_url);

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

        let all_tickers: Vec<BinanceTickerResponse> = response
            .json()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        let mut prices = Vec::new();
        
        for (i, asset_id) in asset_ids.iter().enumerate() {
            let symbol = &symbols[i];
            
            if let Some(ticker) = all_tickers.iter().find(|t| t.symbol == *symbol) {
                if let Ok(price) = Decimal::from_str(&ticker.price) {
                    let mut metadata = HashMap::new();
                    metadata.insert("symbol".to_string(), serde_json::Value::String(ticker.symbol.clone()));

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

        Ok(prices)
    }

    async fn get_supported_assets(&self) -> OracleResult<Vec<String>> {
        let url = format!("{}/exchangeInfo", self.base_url);

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

        let exchange_info: BinanceExchangeInfoResponse = response
            .json()
            .await
            .map_err(|e| OracleError::Provider {
                provider: self.name().to_string(),
                message: format!("Failed to parse response: {}", e),
            })?;

        let mut assets = std::collections::HashSet::new();
        
        for symbol_info in exchange_info.symbols {
            if symbol_info.status == "TRADING" {
                assets.insert(symbol_info.base_asset);
            }
        }

        Ok(assets.into_iter().collect())
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

    #[test]
    fn test_binance_provider_creation() {
        let provider = BinanceProvider::new(dec!(1.1), 1200);
        
        assert_eq!(provider.name(), "Binance");
        assert_eq!(provider.weight(), dec!(1.1));
        assert!(matches!(provider.provider_type(), ProviderType::CentralizedExchange));
    }

    #[test]
    fn test_to_binance_symbol() {
        let provider = BinanceProvider::new(dec!(1.0), 1200);
        
        assert_eq!(provider.to_binance_symbol("BTC", "USDT"), "BTCUSDT");
        assert_eq!(provider.to_binance_symbol("eth", "usd"), "ETHUSD");
        assert_eq!(provider.to_binance_symbol("ADA", "BTC"), "ADABTC");
    }

    #[test]
    fn test_parse_binance_symbol() {
        let provider = BinanceProvider::new(dec!(1.0), 1200);
        
        assert_eq!(provider.parse_binance_symbol("BTCUSDT"), Some(("BTC".to_string(), "USDT".to_string())));
        assert_eq!(provider.parse_binance_symbol("ETHBUSD"), Some(("ETH".to_string(), "BUSD".to_string())));
        assert_eq!(provider.parse_binance_symbol("ADABTC"), Some(("ADA".to_string(), "BTC".to_string())));
        assert_eq!(provider.parse_binance_symbol("INVALID"), None);
    }

    #[test]
    fn test_calculate_confidence() {
        let provider = BinanceProvider::new(dec!(1.0), 1200);
        
        let ticker_data = Binance24hrTickerResponse {
            symbol: "BTCUSDT".to_string(),
            price_change: "1000.0".to_string(),
            price_change_percent: "2.5".to_string(), // Low volatility
            weighted_avg_price: "50000.0".to_string(),
            prev_close_price: "49000.0".to_string(),
            last_price: "50000.0".to_string(),
            last_qty: "0.1".to_string(),
            bid_price: "49999.0".to_string(),
            ask_price: "50001.0".to_string(),
            open_price: "49000.0".to_string(),
            high_price: "51000.0".to_string(),
            low_price: "48000.0".to_string(),
            volume: "10000.0".to_string(), // High volume
            quote_volume: "500000000.0".to_string(),
            open_time: 1640995200000,
            close_time: 1641081600000,
            first_id: 1,
            last_id: 1000,
            count: 1000,
        };
        
        let confidence = provider.calculate_confidence(&ticker_data);
        assert!(confidence >= dec!(0.85)); // Should be high confidence
    }

    #[test]
    fn test_get_rate_limit() {
        let provider = BinanceProvider::new(dec!(1.0), 1200);
        
        let rate_limit = provider.get_rate_limit();
        assert_eq!(rate_limit.requests_per_minute, 1200);
        assert_eq!(rate_limit.current_usage, 0);
    }

    // Integration tests (these would require actual API calls)
    #[tokio::test]
    #[ignore] // Ignore by default to avoid hitting real API during tests
    async fn test_health_check_integration() {
        let provider = BinanceProvider::new(dec!(1.0), 1200);
        
        let result = provider.health_check().await;
        assert!(result.is_ok());
        // Note: This test requires internet connection and working Binance API
    }

    #[tokio::test]
    #[ignore] // Ignore by default to avoid hitting real API during tests
    async fn test_get_price_integration() {
        let provider = BinanceProvider::new(dec!(1.0), 1200);
        
        let result = provider.get_price("BTC", "USDT").await;
        match result {
            Ok(price) => {
                assert_eq!(price.asset_id, "BTC");
                assert_eq!(price.currency, "USDT");
                assert!(price.price > dec!(0));
                assert_eq!(price.source, "Binance");
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
        let provider = BinanceProvider::new(dec!(1.0), 1200);
        assert!(matches!(provider.provider_type(), ProviderType::CentralizedExchange));
    }

    #[test]
    fn test_provider_weight() {
        let provider = BinanceProvider::new(dec!(1.5), 1200);
        assert_eq!(provider.weight(), dec!(1.5));
    }
}
