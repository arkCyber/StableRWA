// =====================================================================================
// RWA Tokenization Platform - CoinMarketCap Price Provider
// 
// Author: arkSong (arksong2018@gmail.com)
// Date: 2025-01-27
// 
// This module implements the CoinMarketCap price provider for the Oracle service.
// It provides real-time cryptocurrency price data from CoinMarketCap API.
// =====================================================================================

use crate::error::{OracleError, OracleResult};
use crate::models::{AssetPrice, ProviderType};
use crate::providers::{PriceProvider, RateLimit};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// CoinMarketCap API configuration
#[derive(Debug, Clone)]
pub struct CoinMarketCapConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub rate_limit: u32,
}

impl Default for CoinMarketCapConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("COINMARKETCAP_API_KEY").unwrap_or_default(),
            base_url: "https://pro-api.coinmarketcap.com/v1".to_string(),
            timeout: Duration::from_secs(10),
            rate_limit: 333, // 333 requests per minute for basic plan
        }
    }
}

/// CoinMarketCap price provider implementation
pub struct CoinMarketCapProvider {
    config: CoinMarketCapConfig,
    client: Client,
    symbol_map: HashMap<String, String>, // Maps our asset IDs to CMC symbols
}

/// CoinMarketCap API response structures
#[derive(Debug, Deserialize)]
struct CmcResponse<T> {
    status: CmcStatus,
    data: T,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CmcStatus {
    timestamp: String,
    error_code: i32,
    error_message: Option<String>,
    elapsed: i32,
    credit_count: i32,
}

#[derive(Debug, Deserialize)]
struct CmcQuote {
    #[serde(rename = "USD")]
    usd: Option<CmcQuoteData>,
    #[serde(rename = "EUR")]
    eur: Option<CmcQuoteData>,
    #[serde(rename = "GBP")]
    gbp: Option<CmcQuoteData>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CmcQuoteData {
    price: f64,
    volume_24h: Option<f64>,
    percent_change_1h: Option<f64>,
    percent_change_24h: Option<f64>,
    percent_change_7d: Option<f64>,
    market_cap: Option<f64>,
    last_updated: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CmcCryptocurrency {
    id: i32,
    name: String,
    symbol: String,
    slug: String,
    quote: CmcQuote,
}

impl CoinMarketCapProvider {
    /// Create a new CoinMarketCap provider
    pub fn new(config: CoinMarketCapConfig) -> OracleResult<Self> {
        info!("🚀 Initializing CoinMarketCap provider with base URL: {}", config.base_url);
        
        if config.api_key.is_empty() {
            error!("❌ CoinMarketCap API key is required");
            return Err(OracleError::Configuration {
                message: "CoinMarketCap API key is required".to_string(),
            });
        }

        let client = Client::builder()
            .timeout(config.timeout)
            .user_agent("RWA-Oracle/1.0")
            .build()
            .map_err(|e| OracleError::Network {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        let mut symbol_map = HashMap::new();
        // Common cryptocurrency mappings
        symbol_map.insert("BTC".to_string(), "BTC".to_string());
        symbol_map.insert("ETH".to_string(), "ETH".to_string());
        symbol_map.insert("USDT".to_string(), "USDT".to_string());
        symbol_map.insert("BNB".to_string(), "BNB".to_string());
        symbol_map.insert("ADA".to_string(), "ADA".to_string());
        symbol_map.insert("SOL".to_string(), "SOL".to_string());
        symbol_map.insert("XRP".to_string(), "XRP".to_string());
        symbol_map.insert("DOT".to_string(), "DOT".to_string());
        symbol_map.insert("DOGE".to_string(), "DOGE".to_string());
        symbol_map.insert("AVAX".to_string(), "AVAX".to_string());

        info!("✅ CoinMarketCap provider initialized successfully");
        Ok(Self {
            config,
            client,
            symbol_map,
        })
    }

    /// Map asset ID to CoinMarketCap symbol
    fn map_asset_to_symbol(&self, asset_id: &str) -> OracleResult<&str> {
        self.symbol_map.get(asset_id)
            .map(|s| s.as_str())
            .ok_or_else(|| OracleError::AssetNotSupported {
                asset_id: asset_id.to_string(),
                provider: "CoinMarketCap".to_string(),
            })
    }

    /// Get quote data for a specific currency
    fn get_quote_data<'a>(&self, quote: &'a CmcQuote, currency: &str) -> Option<&'a CmcQuoteData> {
        match currency.to_uppercase().as_str() {
            "USD" => quote.usd.as_ref(),
            "EUR" => quote.eur.as_ref(),
            "GBP" => quote.gbp.as_ref(),
            _ => None,
        }
    }

    /// Make authenticated request to CoinMarketCap API
    async fn make_request(&self, endpoint: &str, params: &[(&str, &str)]) -> OracleResult<reqwest::Response> {
        let url = format!("{}/{}", self.config.base_url, endpoint);
        
        debug!("📡 Making CoinMarketCap API request to: {}", url);
        
        let mut request = self.client
            .get(&url)
            .header("X-CMC_PRO_API_KEY", &self.config.api_key)
            .header("Accept", "application/json");

        for (key, value) in params {
            request = request.query(&[(key, value)]);
        }

        let response = request.send().await
            .map_err(|e| OracleError::Network {
                message: format!("CoinMarketCap API request failed: {}", e),
            })?;

        if !response.status().is_success() {
            error!("❌ CoinMarketCap API error: {}", response.status());
            return Err(OracleError::Provider {
                provider: "CoinMarketCap".to_string(),
                message: format!("API returned status: {}", response.status()),
            });
        }

        Ok(response)
    }
}

#[async_trait]
impl PriceProvider for CoinMarketCapProvider {
    fn name(&self) -> &str {
        "CoinMarketCap"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::CoinMarketCap
    }

    fn weight(&self) -> Decimal {
        Decimal::from(85) / Decimal::from(100) // 0.85 weight
    }

    async fn health_check(&self) -> OracleResult<bool> {
        debug!("🔍 Performing CoinMarketCap health check");
        
        match self.make_request("key/info", &[]).await {
            Ok(_) => {
                info!("✅ CoinMarketCap health check passed");
                Ok(true)
            }
            Err(e) => {
                warn!("⚠️ CoinMarketCap health check failed: {}", e);
                Ok(false)
            }
        }
    }

    async fn get_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
        info!("💰 Getting price for {} in {} from CoinMarketCap", asset_id, currency);
        
        let symbol = self.map_asset_to_symbol(asset_id)?;
        
        let params = [
            ("symbol", symbol),
            ("convert", currency),
        ];

        let response = self.make_request("cryptocurrency/quotes/latest", &params).await?;
        let cmc_response: CmcResponse<HashMap<String, CmcCryptocurrency>> = response.json().await
            .map_err(|e| OracleError::Parsing {
                message: format!("Failed to parse CoinMarketCap response: {}", e),
            })?;

        if cmc_response.status.error_code != 0 {
            error!("❌ CoinMarketCap API error: {:?}", cmc_response.status.error_message);
            return Err(OracleError::Provider {
                provider: "CoinMarketCap".to_string(),
                message: cmc_response.status.error_message.unwrap_or("Unknown error".to_string()),
            });
        }

        let crypto_data = cmc_response.data.get(symbol)
            .ok_or_else(|| OracleError::AssetNotFound {
                asset_id: asset_id.to_string(),
                provider: "CoinMarketCap".to_string(),
            })?;

        let quote_data = self.get_quote_data(&crypto_data.quote, currency)
            .ok_or_else(|| OracleError::CurrencyNotSupported {
                currency: currency.to_string(),
                provider: "CoinMarketCap".to_string(),
            })?;

        let price = Decimal::try_from(quote_data.price)
            .map_err(|e| OracleError::Parsing {
                message: format!("Failed to parse price: {}", e),
            })?;

        let timestamp = chrono::DateTime::parse_from_rfc3339(&quote_data.last_updated)
            .map_err(|e| OracleError::Parsing {
                message: format!("Failed to parse timestamp: {}", e),
            })?
            .with_timezone(&Utc);

        let mut metadata = HashMap::new();
        if let Some(volume) = quote_data.volume_24h {
            metadata.insert("volume_24h".to_string(), serde_json::Value::String(volume.to_string()));
        }
        if let Some(change_1h) = quote_data.percent_change_1h {
            metadata.insert("percent_change_1h".to_string(), serde_json::Value::String(change_1h.to_string()));
        }
        if let Some(change_24h) = quote_data.percent_change_24h {
            metadata.insert("percent_change_24h".to_string(), serde_json::Value::String(change_24h.to_string()));
        }
        if let Some(market_cap) = quote_data.market_cap {
            metadata.insert("market_cap".to_string(), serde_json::Value::String(market_cap.to_string()));
        }

        let asset_price = AssetPrice {
            asset_id: asset_id.to_string(),
            price,
            currency: currency.to_string(),
            timestamp,
            confidence: Decimal::from(90) / Decimal::from(100), // 0.90 confidence
            source: self.name().to_string(),
            metadata: Some(metadata),
        };

        info!("✅ Successfully retrieved price for {}: {} {}", asset_id, price, currency);
        Ok(asset_price)
    }

    async fn get_batch_prices(&self, asset_ids: &[String], currency: &str) -> OracleResult<Vec<AssetPrice>> {
        info!("📊 Getting batch prices for {} assets from CoinMarketCap", asset_ids.len());
        
        let symbols: Result<Vec<&str>, _> = asset_ids.iter()
            .map(|id| self.map_asset_to_symbol(id))
            .collect();
        let symbols = symbols?;
        let symbols_str = symbols.join(",");

        let params = [
            ("symbol", symbols_str.as_str()),
            ("convert", currency),
        ];

        let response = self.make_request("cryptocurrency/quotes/latest", &params).await?;
        let cmc_response: CmcResponse<HashMap<String, CmcCryptocurrency>> = response.json().await
            .map_err(|e| OracleError::Parsing {
                message: format!("Failed to parse CoinMarketCap batch response: {}", e),
            })?;

        if cmc_response.status.error_code != 0 {
            return Err(OracleError::Provider {
                provider: "CoinMarketCap".to_string(),
                message: cmc_response.status.error_message.unwrap_or("Unknown error".to_string()),
            });
        }

        let mut prices = Vec::new();
        for asset_id in asset_ids {
            let symbol = self.map_asset_to_symbol(asset_id)?;
            
            if let Some(crypto_data) = cmc_response.data.get(symbol) {
                if let Some(quote_data) = self.get_quote_data(&crypto_data.quote, currency) {
                    if let Ok(price) = Decimal::try_from(quote_data.price) {
                        if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&quote_data.last_updated) {
                            let mut metadata = HashMap::new();
                            if let Some(volume) = quote_data.volume_24h {
                                metadata.insert("volume_24h".to_string(), serde_json::Value::String(volume.to_string()));
                            }

                            prices.push(AssetPrice {
                                asset_id: asset_id.clone(),
                                price,
                                currency: currency.to_string(),
                                timestamp: timestamp.with_timezone(&Utc),
                                confidence: Decimal::from(90) / Decimal::from(100),
                                source: self.name().to_string(),
                                metadata: Some(metadata),
                            });
                        }
                    }
                }
            }
        }

        info!("✅ Successfully retrieved {} prices from CoinMarketCap", prices.len());
        Ok(prices)
    }

    async fn get_supported_assets(&self) -> OracleResult<Vec<String>> {
        Ok(self.symbol_map.keys().cloned().collect())
    }

    fn get_rate_limit(&self) -> RateLimit {
        RateLimit {
            requests_per_minute: self.config.rate_limit,
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
    fn test_coinmarketcap_provider_creation() {
        let config = CoinMarketCapConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };
        
        let provider = CoinMarketCapProvider::new(config);
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "CoinMarketCap");
        assert_eq!(provider.weight(), dec!(0.85));
    }

    #[test]
    fn test_asset_mapping() {
        let config = CoinMarketCapConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };
        let provider = CoinMarketCapProvider::new(config).unwrap();
        
        assert_eq!(provider.map_asset_to_symbol("BTC").unwrap(), "BTC");
        assert_eq!(provider.map_asset_to_symbol("ETH").unwrap(), "ETH");
        assert!(provider.map_asset_to_symbol("UNKNOWN").is_err());
    }

    #[test]
    fn test_supported_assets() {
        let config = CoinMarketCapConfig {
            api_key: "test_key".to_string(),
            ..Default::default()
        };
        let provider = CoinMarketCapProvider::new(config).unwrap();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let assets = rt.block_on(provider.get_supported_assets()).unwrap();
        
        assert!(!assets.is_empty());
        assert!(assets.contains(&"BTC".to_string()));
        assert!(assets.contains(&"ETH".to_string()));
    }
}
