// =====================================================================================
// File: core-oracle/src/service.rs
// Description: Main oracle service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    error::{OracleError, OracleResult},
    types::{
        AggregationMethod, FeedConfig, FeedSubscription, OracleConfig, OracleHealthStatus,
        OracleProvider, OracleRequest, OracleResponse, PriceAlert, PriceData, PriceFeed,
        TimeSeriesData, ValidationRule,
    },
};

/// Main oracle service trait
#[async_trait]
pub trait OracleService: Send + Sync {
    /// Get current price for a feed
    async fn get_price(&self, feed_id: &str) -> OracleResult<PriceData>;

    /// Get price from specific provider
    async fn get_price_from_provider(
        &self,
        feed_id: &str,
        provider: OracleProvider,
    ) -> OracleResult<PriceData>;

    /// Get historical prices
    async fn get_historical_prices(
        &self,
        feed_id: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> OracleResult<TimeSeriesData>;

    /// Subscribe to price feed updates
    async fn subscribe_to_feed(&self, subscription: &FeedSubscription) -> OracleResult<Uuid>;

    /// Unsubscribe from price feed
    async fn unsubscribe_from_feed(&self, subscription_id: &Uuid) -> OracleResult<()>;

    /// Add new price feed
    async fn add_feed(&self, config: &FeedConfig) -> OracleResult<String>;

    /// Remove price feed
    async fn remove_feed(&self, feed_id: &str) -> OracleResult<()>;

    /// Update feed configuration
    async fn update_feed_config(&self, feed_id: &str, config: &FeedConfig) -> OracleResult<()>;

    /// Get feed information
    async fn get_feed_info(&self, feed_id: &str) -> OracleResult<PriceFeed>;

    /// List all available feeds
    async fn list_feeds(&self) -> OracleResult<Vec<String>>;

    /// Get oracle health status
    async fn get_health_status(&self) -> OracleResult<OracleHealthStatus>;

    /// Process oracle request
    async fn process_request(&self, request: &OracleRequest) -> OracleResult<OracleResponse>;

    /// Validate price data
    async fn validate_price_data(
        &self,
        feed_id: &str,
        price_data: &PriceData,
    ) -> OracleResult<bool>;
}

/// Price query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuery {
    pub feed_id: String,
    pub providers: Option<Vec<OracleProvider>>,
    pub max_age_seconds: Option<u64>,
    pub min_confidence: Option<f64>,
    pub aggregation_method: Option<AggregationMethod>,
}

/// Historical data query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalQuery {
    pub feed_id: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub interval: crate::types::TimeInterval,
    pub providers: Option<Vec<OracleProvider>>,
}

/// Oracle service implementation
pub struct OracleServiceImpl {
    config: OracleConfig,
    feeds: Arc<RwLock<HashMap<String, PriceFeed>>>,
    subscriptions: Arc<RwLock<HashMap<Uuid, FeedSubscription>>>,
    price_cache: Arc<RwLock<HashMap<String, PriceData>>>,
    provider_clients: HashMap<OracleProvider, Box<dyn ProviderClient>>,
    start_time: chrono::DateTime<chrono::Utc>,
}

/// Provider client trait
#[async_trait]
pub trait ProviderClient: Send + Sync {
    /// Get price from provider
    async fn get_price(&self, feed_id: &str) -> OracleResult<PriceData>;

    /// Get provider health
    async fn get_health(&self) -> OracleResult<crate::types::ProviderHealth>;

    /// Get provider name
    fn provider(&self) -> OracleProvider;
}

/// Mock provider client for demonstration
pub struct MockProviderClient {
    provider: OracleProvider,
}

impl MockProviderClient {
    pub fn new(provider: OracleProvider) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl ProviderClient for MockProviderClient {
    async fn get_price(&self, feed_id: &str) -> OracleResult<PriceData> {
        // Mock price data generation
        let base_price = match feed_id {
            "ETH/USD" => Decimal::new(200000, 2),  // $2000.00
            "BTC/USD" => Decimal::new(5000000, 2), // $50000.00
            "USDC/USD" => Decimal::new(100, 2),    // $1.00
            _ => Decimal::new(10000, 2),           // $100.00
        };

        // Add some provider-specific variation
        let variation = match self.provider {
            OracleProvider::Chainlink => Decimal::new(0, 0),
            OracleProvider::BandProtocol => Decimal::new(50, 2), // +$0.50
            OracleProvider::PythNetwork => Decimal::new(-25, 2), // -$0.25
            _ => Decimal::new(0, 0),
        };

        Ok(PriceData {
            price: base_price + variation,
            timestamp: Utc::now(),
            source: self.provider,
            confidence: 0.95,
            volume: Some(Decimal::new(1000000, 2)),
            market_cap: None,
            deviation: Some(0.5),
            round_id: Some(12345),
        })
    }

    async fn get_health(&self) -> OracleResult<crate::types::ProviderHealth> {
        Ok(crate::types::ProviderHealth {
            status: crate::types::HealthStatus::Healthy,
            last_successful_update: Utc::now(),
            consecutive_failures: 0,
            response_time_ms: 50,
            success_rate: 99.5,
        })
    }

    fn provider(&self) -> OracleProvider {
        self.provider
    }
}

impl OracleServiceImpl {
    pub fn new(config: OracleConfig) -> Self {
        let mut provider_clients: HashMap<OracleProvider, Box<dyn ProviderClient>> = HashMap::new();

        // Initialize mock provider clients
        for provider_config in &config.providers {
            if provider_config.enabled {
                let client = Box::new(MockProviderClient::new(provider_config.provider));
                provider_clients.insert(provider_config.provider, client);
            }
        }

        Self {
            config,
            feeds: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            provider_clients,
            start_time: Utc::now(),
        }
    }

    /// Initialize service with default feeds
    pub async fn initialize(&self) -> OracleResult<()> {
        // Add some default feeds
        let default_feeds = vec![
            FeedConfig {
                id: "ETH/USD".to_string(),
                symbol: "ETHUSD".to_string(),
                base_asset: "ETH".to_string(),
                quote_asset: "USD".to_string(),
                providers: vec![OracleProvider::Chainlink, OracleProvider::BandProtocol],
                update_interval_seconds: 60,
                deviation_threshold_percent: 5.0,
                staleness_threshold_seconds: 300,
                min_sources: 2,
                enabled: true,
            },
            FeedConfig {
                id: "BTC/USD".to_string(),
                symbol: "BTCUSD".to_string(),
                base_asset: "BTC".to_string(),
                quote_asset: "USD".to_string(),
                providers: vec![OracleProvider::Chainlink, OracleProvider::PythNetwork],
                update_interval_seconds: 60,
                deviation_threshold_percent: 5.0,
                staleness_threshold_seconds: 300,
                min_sources: 2,
                enabled: true,
            },
        ];

        for feed_config in default_feeds {
            self.add_feed(&feed_config).await?;
        }

        Ok(())
    }

    async fn aggregate_prices(
        &self,
        prices: Vec<PriceData>,
        method: AggregationMethod,
    ) -> OracleResult<PriceData> {
        if prices.is_empty() {
            return Err(OracleError::insufficient_data("aggregation", 1, 0));
        }

        let aggregated_price = match method {
            AggregationMethod::Mean => {
                let sum: Decimal = prices.iter().map(|p| p.price).sum();
                sum / Decimal::new(prices.len() as i64, 0)
            }
            AggregationMethod::Median => {
                let mut sorted_prices: Vec<Decimal> = prices.iter().map(|p| p.price).collect();
                sorted_prices.sort();
                let mid = sorted_prices.len() / 2;
                if sorted_prices.len() % 2 == 0 {
                    (sorted_prices[mid - 1] + sorted_prices[mid]) / Decimal::new(2, 0)
                } else {
                    sorted_prices[mid]
                }
            }
            AggregationMethod::WeightedMean => {
                let total_weight: f64 = prices.iter().map(|p| p.confidence).sum();
                let weighted_sum: Decimal = prices
                    .iter()
                    .map(|p| {
                        p.price * Decimal::from_f64_retain(p.confidence).unwrap_or(Decimal::ONE)
                    })
                    .sum();
                weighted_sum / Decimal::from_f64_retain(total_weight).unwrap_or(Decimal::ONE)
            }
            _ => {
                // Default to median for other methods
                let mut sorted_prices: Vec<Decimal> = prices.iter().map(|p| p.price).collect();
                sorted_prices.sort();
                let mid = sorted_prices.len() / 2;
                sorted_prices[mid]
            }
        };

        let avg_confidence = prices.iter().map(|p| p.confidence).sum::<f64>() / prices.len() as f64;
        let latest_timestamp = prices
            .iter()
            .map(|p| p.timestamp)
            .max()
            .unwrap_or(Utc::now());

        Ok(PriceData {
            price: aggregated_price,
            timestamp: latest_timestamp,
            source: OracleProvider::Custom(0), // Aggregated source
            confidence: avg_confidence,
            volume: None,
            market_cap: None,
            deviation: None,
            round_id: None,
        })
    }

    async fn validate_price(&self, feed_id: &str, price_data: &PriceData) -> OracleResult<bool> {
        // Check if price is within reasonable bounds
        if price_data.price <= Decimal::ZERO {
            return Err(OracleError::invalid_price(
                feed_id,
                price_data.price.to_string(),
                "Price must be positive",
            ));
        }

        // Check data freshness
        let age = Utc::now() - price_data.timestamp;
        if age.num_seconds() > 300 {
            // 5 minutes
            return Err(OracleError::stale_data(
                feed_id,
                age.num_seconds() as u64,
                300,
            ));
        }

        // Check confidence level
        if price_data.confidence < 0.5 {
            return Err(OracleError::validation_error(
                "confidence",
                "Confidence too low",
            ));
        }

        Ok(true)
    }
}

#[async_trait]
impl OracleService for OracleServiceImpl {
    async fn get_price(&self, feed_id: &str) -> OracleResult<PriceData> {
        // Check cache first
        {
            let cache = self.price_cache.read().await;
            if let Some(cached_price) = cache.get(feed_id) {
                let age = Utc::now() - cached_price.timestamp;
                if age.num_seconds() < 60 {
                    // Cache for 1 minute
                    return Ok(cached_price.clone());
                }
            }
        }

        // Get prices from all available providers
        let mut prices = Vec::new();
        for (provider, client) in &self.provider_clients {
            match client.get_price(feed_id).await {
                Ok(price_data) => {
                    if self.validate_price(feed_id, &price_data).await.is_ok() {
                        prices.push(price_data);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get price from {:?}: {}", provider, e);
                }
            }
        }

        if prices.is_empty() {
            return Err(OracleError::feed_not_found(feed_id.to_string()));
        }

        // Aggregate prices
        let aggregated_price = self
            .aggregate_prices(prices, self.config.aggregation.default_method)
            .await?;

        // Update cache
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(feed_id.to_string(), aggregated_price.clone());
        }

        Ok(aggregated_price)
    }

    async fn get_price_from_provider(
        &self,
        feed_id: &str,
        provider: OracleProvider,
    ) -> OracleResult<PriceData> {
        let client = self.provider_clients.get(&provider).ok_or_else(|| {
            OracleError::provider_error(provider.name(), "Provider not available")
        })?;

        let price_data = client.get_price(feed_id).await?;
        self.validate_price(feed_id, &price_data).await?;

        Ok(price_data)
    }

    async fn get_historical_prices(
        &self,
        feed_id: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> OracleResult<TimeSeriesData> {
        // Mock historical data generation
        let mut points = Vec::new();
        let mut current_time = start_time;
        let interval = chrono::Duration::hours(1);

        while current_time <= end_time {
            let base_price = Decimal::new(200000, 2); // $2000.00
            let variation = (current_time.timestamp() % 100) as i64;
            let price = base_price + Decimal::new(variation, 0);

            points.push(crate::types::PricePoint {
                timestamp: current_time,
                open: price,
                high: price + Decimal::new(50, 0),
                low: price - Decimal::new(30, 0),
                close: price + Decimal::new(10, 0),
                volume: Decimal::new(1000000, 2),
                source_count: 3,
            });

            current_time += interval;
        }

        Ok(TimeSeriesData {
            symbol: feed_id.to_string(),
            interval: crate::types::TimeInterval::OneHour,
            points,
            start_time,
            end_time,
        })
    }

    async fn subscribe_to_feed(&self, subscription: &FeedSubscription) -> OracleResult<Uuid> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription.id, subscription.clone());
        Ok(subscription.id)
    }

    async fn unsubscribe_from_feed(&self, subscription_id: &Uuid) -> OracleResult<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id);
        Ok(())
    }

    async fn add_feed(&self, config: &FeedConfig) -> OracleResult<String> {
        let feed = PriceFeed {
            id: config.id.clone(),
            symbol: config.symbol.clone(),
            base_asset: config.base_asset.clone(),
            quote_asset: config.quote_asset.clone(),
            current_price: PriceData {
                price: Decimal::ZERO,
                timestamp: Utc::now(),
                source: OracleProvider::Custom(0),
                confidence: 0.0,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
            historical_prices: Vec::new(),
            sources: Vec::new(),
            metadata: crate::types::FeedMetadata {
                description: format!("{} price feed", config.symbol),
                decimals: 8,
                heartbeat: config.update_interval_seconds,
                deviation_threshold: config.deviation_threshold_percent,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                tags: Vec::new(),
            },
            status: crate::types::FeedStatus::Active,
        };

        let mut feeds = self.feeds.write().await;
        feeds.insert(config.id.clone(), feed);

        Ok(config.id.clone())
    }

    async fn remove_feed(&self, feed_id: &str) -> OracleResult<()> {
        let mut feeds = self.feeds.write().await;
        feeds.remove(feed_id);
        Ok(())
    }

    async fn update_feed_config(&self, feed_id: &str, config: &FeedConfig) -> OracleResult<()> {
        let mut feeds = self.feeds.write().await;
        if let Some(feed) = feeds.get_mut(feed_id) {
            feed.metadata.updated_at = Utc::now();
            feed.metadata.heartbeat = config.update_interval_seconds;
            feed.metadata.deviation_threshold = config.deviation_threshold_percent;
        }
        Ok(())
    }

    async fn get_feed_info(&self, feed_id: &str) -> OracleResult<PriceFeed> {
        let feeds = self.feeds.read().await;
        feeds
            .get(feed_id)
            .cloned()
            .ok_or_else(|| OracleError::feed_not_found(feed_id.to_string()))
    }

    async fn list_feeds(&self) -> OracleResult<Vec<String>> {
        let feeds = self.feeds.read().await;
        Ok(feeds.keys().cloned().collect())
    }

    async fn get_health_status(&self) -> OracleResult<OracleHealthStatus> {
        let mut provider_status = HashMap::new();

        for (provider, client) in &self.provider_clients {
            match client.get_health().await {
                Ok(health) => {
                    provider_status.insert(*provider, health);
                }
                Err(_) => {
                    provider_status.insert(
                        *provider,
                        crate::types::ProviderHealth {
                            status: crate::types::HealthStatus::Unhealthy,
                            last_successful_update: Utc::now() - chrono::Duration::hours(1),
                            consecutive_failures: 5,
                            response_time_ms: 0,
                            success_rate: 0.0,
                        },
                    );
                }
            }
        }

        let feeds = self.feeds.read().await;
        let mut feed_status = HashMap::new();

        for (feed_id, _feed) in feeds.iter() {
            feed_status.insert(
                feed_id.clone(),
                crate::types::FeedHealth {
                    status: crate::types::HealthStatus::Healthy,
                    last_update: Utc::now(),
                    data_age_seconds: 30,
                    source_count: 3,
                    consensus_score: 0.95,
                    price_deviation: 1.2,
                },
            );
        }

        let uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;

        Ok(OracleHealthStatus {
            overall_status: crate::types::HealthStatus::Healthy,
            provider_status,
            feed_status,
            last_updated: Utc::now(),
            uptime_percentage: 99.9,
            total_requests: 10000,
            successful_requests: 9990,
            failed_requests: 10,
        })
    }

    async fn process_request(&self, request: &OracleRequest) -> OracleResult<OracleResponse> {
        let start_time = std::time::Instant::now();

        let price_data = self.get_price(&request.feed_id).await?;

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(OracleResponse {
            request_id: request.id,
            feed_id: request.feed_id.clone(),
            price_data,
            sources_used: vec![OracleProvider::Chainlink, OracleProvider::BandProtocol],
            aggregation_method: self.config.aggregation.default_method,
            confidence_score: 0.95,
            response_time_ms,
            responded_at: Utc::now(),
        })
    }

    async fn validate_price_data(
        &self,
        feed_id: &str,
        price_data: &PriceData,
    ) -> OracleResult<bool> {
        self.validate_price(feed_id, price_data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_oracle_service_creation() {
        let config = OracleConfig::default();
        let service = OracleServiceImpl::new(config);

        assert!(service.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_mock_provider_client() {
        let client = MockProviderClient::new(OracleProvider::Chainlink);

        let price = client.get_price("ETH/USD").await.unwrap();
        assert!(price.price > Decimal::ZERO);
        assert_eq!(price.source, OracleProvider::Chainlink);

        let health = client.get_health().await.unwrap();
        assert_eq!(health.status, crate::types::HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_price_aggregation() {
        let config = OracleConfig::default();
        let service = OracleServiceImpl::new(config);

        let prices = vec![
            PriceData {
                price: Decimal::new(100, 0),
                timestamp: Utc::now(),
                source: OracleProvider::Chainlink,
                confidence: 0.9,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
            PriceData {
                price: Decimal::new(102, 0),
                timestamp: Utc::now(),
                source: OracleProvider::BandProtocol,
                confidence: 0.8,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
        ];

        let aggregated = service
            .aggregate_prices(prices, AggregationMethod::Mean)
            .await
            .unwrap();
        assert_eq!(aggregated.price, Decimal::new(101, 0));
    }
}
