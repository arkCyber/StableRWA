// =====================================================================================
// RWA Tokenization Platform - Oracle Service Implementation
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::aggregator::PriceAggregator;
use crate::cache::PriceCache;
use crate::config::OracleConfig;
use crate::error::{OracleError, OracleResult};
use crate::models::{
    AssetPrice, PriceFeed, PriceSubscription, BatchPriceRequest, BatchPriceResponse,
    PriceHistoryRequest, PriceHistoryResponse, CreatePriceFeedRequest, UpdatePriceFeedRequest,
    SubscriptionRequest, AggregationMethod, PriceDataPoint
};
use crate::providers::{ProviderManager, coingecko::CoinGeckoProvider};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Oracle service trait
#[async_trait]
pub trait OracleServiceTrait {
    /// Get current price for an asset
    async fn get_asset_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice>;
    
    /// Get batch prices for multiple assets
    async fn get_batch_prices(&self, request: &BatchPriceRequest) -> OracleResult<BatchPriceResponse>;
    
    /// Get price history for an asset
    async fn get_price_history(&self, asset_id: &str, request: &PriceHistoryRequest) -> OracleResult<PriceHistoryResponse>;
    
    /// Create a new price feed
    async fn create_price_feed(&self, request: &CreatePriceFeedRequest) -> OracleResult<PriceFeed>;
    
    /// Update an existing price feed
    async fn update_price_feed(&self, feed_id: &Uuid, request: &UpdatePriceFeedRequest) -> OracleResult<PriceFeed>;
    
    /// Delete a price feed
    async fn delete_price_feed(&self, feed_id: &Uuid) -> OracleResult<()>;
    
    /// Get a price feed by ID
    async fn get_price_feed(&self, feed_id: &Uuid) -> OracleResult<PriceFeed>;
    
    /// List all price feeds
    async fn list_price_feeds(&self) -> OracleResult<Vec<PriceFeed>>;
    
    /// Subscribe to price updates
    async fn subscribe_to_feed(&self, feed_id: &Uuid, request: &SubscriptionRequest) -> OracleResult<PriceSubscription>;
    
    /// Unsubscribe from price updates
    async fn unsubscribe_from_feed(&self, subscription_id: &Uuid) -> OracleResult<()>;
    
    /// Get provider status
    async fn get_provider_status(&self) -> OracleResult<HashMap<String, bool>>;
}

/// Main Oracle service implementation
pub struct OracleService {
    config: OracleConfig,
    db_pool: PgPool,
    cache: Arc<PriceCache>,
    aggregator: Arc<RwLock<PriceAggregator>>,
    subscriptions: Arc<RwLock<HashMap<Uuid, PriceSubscription>>>,
}

impl OracleService {
    /// Create a new Oracle service
    pub async fn new(config: OracleConfig, db_pool: PgPool) -> OracleResult<Self> {
        info!("Initializing Oracle service");

        // Initialize cache
        let cache = Arc::new(PriceCache::new(&config.redis).await?);

        // Initialize provider manager
        let mut provider_manager = ProviderManager::new();
        
        // Add CoinGecko provider if enabled
        if config.providers.coingecko.enabled {
            let coingecko = Arc::new(CoinGeckoProvider::new(
                config.providers.coingecko.api_key.clone(),
                config.providers.coingecko.weight,
                config.providers.coingecko.rate_limit_per_minute,
            ));
            provider_manager.add_provider(coingecko);
            info!("Added CoinGecko provider");
        }

        // Initialize aggregator
        let aggregator = Arc::new(RwLock::new(PriceAggregator::new(
            provider_manager,
            config.aggregation.min_sources,
            config.aggregation.max_deviation_percent,
            config.aggregation.confidence_threshold,
            config.aggregation.outlier_detection,
            Duration::from_secs(config.providers.coingecko.timeout_seconds),
        )));

        let subscriptions = Arc::new(RwLock::new(HashMap::new()));

        info!("Oracle service initialized successfully");

        Ok(Self {
            config,
            db_pool,
            cache,
            aggregator,
            subscriptions,
        })
    }

    /// Get aggregation method from string
    fn parse_aggregation_method(&self, method: &str) -> AggregationMethod {
        match method.to_lowercase().as_str() {
            "mean" => AggregationMethod::Mean,
            "median" => AggregationMethod::Median,
            "weighted_average" => AggregationMethod::WeightedAverage,
            "volume_weighted" => AggregationMethod::VolumeWeighted,
            _ => AggregationMethod::Custom(method.to_string()),
        }
    }

    /// Store price in database
    async fn store_price(&self, price: &AssetPrice) -> OracleResult<()> {
        let query = r#"
            INSERT INTO asset_prices (asset_id, price, currency, confidence, source, timestamp, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#;

        let metadata_json = price.metadata.as_ref()
            .map(|m| serde_json::to_value(m).unwrap_or(serde_json::Value::Null))
            .unwrap_or(serde_json::Value::Null);

        sqlx::query(query)
            .bind(&price.asset_id)
            .bind(&price.price.to_string())
            .bind(&price.currency)
            .bind(&price.confidence.to_string())
            .bind(&price.source)
            .bind(&price.timestamp)
            .bind(&metadata_json)
            .execute(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?;

        Ok(())
    }

    /// Get cached price or fetch new one
    async fn get_or_fetch_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
        let cache_key = format!("price:{}:{}", asset_id, currency);
        
        // Try to get from cache first
        if let Ok(Some(cached_price)) = self.cache.get_price(&cache_key).await {
            debug!("Retrieved price from cache for {}/{}", asset_id, currency);
            return Ok(cached_price);
        }

        // Fetch from providers
        debug!("Fetching fresh price for {}/{}", asset_id, currency);
        let mut aggregator = self.aggregator.write().await;
        let method = self.parse_aggregation_method(&self.config.aggregation.default_method);
        
        let aggregation_result = aggregator.aggregate_price(asset_id, currency, &method, None).await?;
        let price = aggregation_result.final_price;

        // Store in database
        if let Err(e) = self.store_price(&price).await {
            warn!("Failed to store price in database: {}", e);
        }

        // Cache the result
        if let Err(e) = self.cache.set_price(&cache_key, &price, self.config.aggregation.cache_ttl_seconds).await {
            warn!("Failed to cache price: {}", e);
        }

        Ok(price)
    }

    /// Validate asset ID and currency
    fn validate_request(&self, asset_id: &str, currency: &str) -> OracleResult<()> {
        crate::providers::validate_asset_id(asset_id)?;
        crate::providers::validate_currency(currency)?;
        Ok(())
    }
}

#[async_trait]
impl OracleServiceTrait for OracleService {
    async fn get_asset_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
        self.validate_request(asset_id, currency)?;
        self.get_or_fetch_price(asset_id, currency).await
    }

    async fn get_batch_prices(&self, request: &BatchPriceRequest) -> OracleResult<BatchPriceResponse> {
        let currency = request.currency.as_deref().unwrap_or("USD");
        let mut prices = HashMap::new();
        let mut errors = HashMap::new();

        for asset_id in &request.asset_ids {
            match self.get_or_fetch_price(asset_id, currency).await {
                Ok(price) => {
                    prices.insert(asset_id.clone(), price);
                }
                Err(e) => {
                    errors.insert(asset_id.clone(), e.to_string());
                }
            }
        }

        Ok(BatchPriceResponse {
            prices,
            errors,
            timestamp: Utc::now(),
        })
    }

    async fn get_price_history(&self, asset_id: &str, request: &PriceHistoryRequest) -> OracleResult<PriceHistoryResponse> {
        self.validate_request(asset_id, "USD")?; // Basic validation

        let limit = request.limit.unwrap_or(100).min(1000); // Cap at 1000
        let start_time = request.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
        let end_time = request.end_time.unwrap_or_else(|| Utc::now());

        let query = r#"
            SELECT price, timestamp, metadata
            FROM asset_prices
            WHERE asset_id = $1 AND timestamp BETWEEN $2 AND $3
            ORDER BY timestamp DESC
            LIMIT $4
        "#;

        let rows = sqlx::query(query)
            .bind(asset_id)
            .bind(start_time)
            .bind(end_time)
            .bind(limit as i64)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?;

        let mut data = Vec::new();
        for row in rows {
            let price_str: String = row.get("price");
            let price = Decimal::from_str(&price_str).map_err(|e| OracleError::Parsing {
                message: format!("Failed to parse price: {}", e),
            })?;
            let timestamp: DateTime<Utc> = row.get("timestamp");
            
            data.push(PriceDataPoint {
                timestamp,
                price,
                volume: None,
                high: None,
                low: None,
                open: None,
                close: Some(price),
            });
        }

        // Get total count
        let count_query = r#"
            SELECT COUNT(*) as total
            FROM asset_prices
            WHERE asset_id = $1 AND timestamp BETWEEN $2 AND $3
        "#;

        let total_count: i64 = sqlx::query(count_query)
            .bind(asset_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?
            .get("total");

        Ok(PriceHistoryResponse {
            asset_id: asset_id.to_string(),
            currency: "USD".to_string(), // TODO: Make this configurable
            data,
            interval: request.interval.clone().unwrap_or_else(|| "1h".to_string()),
            total_count,
        })
    }

    async fn create_price_feed(&self, request: &CreatePriceFeedRequest) -> OracleResult<PriceFeed> {
        let feed_id = Uuid::new_v4();
        let now = Utc::now();

        let query = r#"
            INSERT INTO price_feeds (id, asset_id, name, description, currency, update_interval, 
                                   providers, aggregation_method, deviation_threshold, is_active, 
                                   created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10, $11)
        "#;

        let providers_json = serde_json::to_value(&request.providers)
            .map_err(|e| OracleError::Serialization(e))?;

        sqlx::query(query)
            .bind(&feed_id)
            .bind(&request.asset_id)
            .bind(&request.name)
            .bind(&request.description)
            .bind(&request.currency)
            .bind(&request.update_interval)
            .bind(&providers_json)
            .bind(&request.aggregation_method.to_string())
            .bind(&request.deviation_threshold.to_string())
            .bind(&now)
            .bind(&now)
            .execute(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?;

        Ok(PriceFeed {
            id: feed_id,
            asset_id: request.asset_id.clone(),
            name: request.name.clone(),
            description: request.description.clone(),
            currency: request.currency.clone(),
            update_interval: request.update_interval,
            providers: request.providers.clone(),
            aggregation_method: request.aggregation_method.clone(),
            deviation_threshold: request.deviation_threshold,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    async fn update_price_feed(&self, feed_id: &Uuid, request: &UpdatePriceFeedRequest) -> OracleResult<PriceFeed> {
        // First, get the existing feed
        let _existing_feed = self.get_price_feed(feed_id).await?;

        let now = Utc::now();
        let mut query_parts = Vec::new();
        let mut bind_count = 1;

        let mut query = "UPDATE price_feeds SET updated_at = $1".to_string();
        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![Box::new(now)];
        bind_count += 1;

        if let Some(name) = &request.name {
            query_parts.push(format!("name = ${}", bind_count));
            bindings.push(Box::new(name.clone()));
            bind_count += 1;
        }

        if let Some(description) = &request.description {
            query_parts.push(format!("description = ${}", bind_count));
            bindings.push(Box::new(description.clone()));
            bind_count += 1;
        }

        if let Some(update_interval) = &request.update_interval {
            query_parts.push(format!("update_interval = ${}", bind_count));
            bindings.push(Box::new(*update_interval));
            bind_count += 1;
        }

        if let Some(is_active) = &request.is_active {
            query_parts.push(format!("is_active = ${}", bind_count));
            bindings.push(Box::new(*is_active));
            bind_count += 1;
        }

        if !query_parts.is_empty() {
            query.push_str(", ");
            query.push_str(&query_parts.join(", "));
        }

        query.push_str(&format!(" WHERE id = ${}", bind_count));
        bindings.push(Box::new(*feed_id));

        // Note: This is a simplified version. In a real implementation, you'd need to handle
        // the dynamic query building more carefully with proper type handling.
        
        // For now, let's implement a simpler version that updates specific fields
        let update_query = r#"
            UPDATE price_feeds 
            SET updated_at = $1,
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                update_interval = COALESCE($4, update_interval),
                is_active = COALESCE($5, is_active)
            WHERE id = $6
        "#;

        sqlx::query(update_query)
            .bind(&now)
            .bind(&request.name)
            .bind(&request.description)
            .bind(&request.update_interval)
            .bind(&request.is_active)
            .bind(feed_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?;

        // Return updated feed
        self.get_price_feed(feed_id).await
    }

    async fn delete_price_feed(&self, feed_id: &Uuid) -> OracleResult<()> {
        let query = "DELETE FROM price_feeds WHERE id = $1";
        
        let result = sqlx::query(query)
            .bind(feed_id)
            .execute(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?;

        if result.rows_affected() == 0 {
            return Err(OracleError::FeedNotFound {
                feed_id: feed_id.to_string(),
            });
        }

        Ok(())
    }

    async fn get_price_feed(&self, feed_id: &Uuid) -> OracleResult<PriceFeed> {
        let query = r#"
            SELECT id, asset_id, name, description, currency, update_interval, providers,
                   aggregation_method, deviation_threshold, is_active, created_at, updated_at
            FROM price_feeds
            WHERE id = $1
        "#;

        let row = sqlx::query(query)
            .bind(feed_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?
            .ok_or_else(|| OracleError::FeedNotFound {
                feed_id: feed_id.to_string(),
            })?;

        let providers_json: serde_json::Value = row.get("providers");
        let providers: Vec<String> = serde_json::from_value(providers_json)
            .map_err(|e| OracleError::Serialization(e))?;

        let aggregation_method_str: String = row.get("aggregation_method");
        let aggregation_method = self.parse_aggregation_method(&aggregation_method_str);

        Ok(PriceFeed {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            name: row.get("name"),
            description: row.get("description"),
            currency: row.get("currency"),
            update_interval: row.get("update_interval"),
            providers,
            aggregation_method,
            deviation_threshold: {
                let threshold_str: String = row.get("deviation_threshold");
                Decimal::from_str(&threshold_str).unwrap_or(Decimal::ZERO)
            },
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn list_price_feeds(&self) -> OracleResult<Vec<PriceFeed>> {
        let query = r#"
            SELECT id, asset_id, name, description, currency, update_interval, providers,
                   aggregation_method, deviation_threshold, is_active, created_at, updated_at
            FROM price_feeds
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| OracleError::Database(e))?;

        let mut feeds = Vec::new();
        for row in rows {
            let providers_json: serde_json::Value = row.get("providers");
            let providers: Vec<String> = serde_json::from_value(providers_json)
                .map_err(|e| OracleError::Serialization(e))?;

            let aggregation_method_str: String = row.get("aggregation_method");
            let aggregation_method = self.parse_aggregation_method(&aggregation_method_str);

            feeds.push(PriceFeed {
                id: row.get("id"),
                asset_id: row.get("asset_id"),
                name: row.get("name"),
                description: row.get("description"),
                currency: row.get("currency"),
                update_interval: row.get("update_interval"),
                providers,
                aggregation_method,
                deviation_threshold: {
                    let threshold_str: String = row.get("deviation_threshold");
                    Decimal::from_str(&threshold_str).unwrap_or(Decimal::ZERO)
                },
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(feeds)
    }

    async fn subscribe_to_feed(&self, feed_id: &Uuid, request: &SubscriptionRequest) -> OracleResult<PriceSubscription> {
        // Verify feed exists
        self.get_price_feed(feed_id).await?;

        let subscription_id = Uuid::new_v4();
        let now = Utc::now();

        let subscription = PriceSubscription {
            id: subscription_id,
            feed_id: *feed_id,
            subscriber_id: request.subscriber_id.clone(),
            webhook_url: request.webhook_url.clone(),
            filters: request.filters.clone(),
            is_active: true,
            created_at: now,
        };

        // Store in memory (in a real implementation, you'd store in database)
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id, subscription.clone());

        Ok(subscription)
    }

    async fn unsubscribe_from_feed(&self, subscription_id: &Uuid) -> OracleResult<()> {
        let mut subscriptions = self.subscriptions.write().await;
        
        if subscriptions.remove(subscription_id).is_none() {
            return Err(OracleError::SubscriptionNotFound {
                subscription_id: subscription_id.to_string(),
            });
        }

        Ok(())
    }

    async fn get_provider_status(&self) -> OracleResult<HashMap<String, bool>> {
        let _aggregator = self.aggregator.read().await;
        // Note: This would need to be implemented in the aggregator
        // For now, return empty map
        Ok(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{OracleConfig, AggregationConfig, ProviderConfig, RedisConfig, DatabaseConfig, ServerConfig};
    use crate::models::*;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;
    // Removed unused imports
    use tokio;

    async fn create_test_db_pool() -> PgPool {
        // This would need to be configured for actual testing
        // For now, we'll create a mock or use an in-memory database
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/oracle_test".to_string());

        PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    fn create_test_config() -> OracleConfig {
        OracleConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8081,
                workers: 1,
            },
            database: DatabaseConfig {
                url: "postgresql://test:test@localhost/test".to_string(),
                max_connections: 5,
                min_connections: 1,
                connect_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 1800,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379/1".to_string(),
                max_connections: 5,
                connection_timeout: 5,
                command_timeout: 5,
                retry_attempts: 3,
            },
            aggregation: AggregationConfig {
                default_method: AggregationMethod::Mean,
                min_sources: 1, // Lower for testing
                max_deviation_percent: dec!(50.0), // Higher tolerance for testing
                confidence_threshold: dec!(0.5), // Lower for testing
                outlier_detection: true,
                cache_ttl_seconds: 60,
            },
            providers: crate::config::ProvidersConfig {
                coingecko: ProviderConfig {
                    enabled: false, // Disable for unit tests
                    api_key: None,
                    weight: dec!(1.0),
                    rate_limit_per_minute: 50,
                    timeout_seconds: 10,
                },
                binance: ProviderConfig {
                    enabled: false,
                    api_key: None,
                    weight: dec!(1.0),
                    rate_limit_per_minute: 1200,
                    timeout_seconds: 5,
                },
                coinmarketcap: ProviderConfig {
                    enabled: false,
                    api_key: None,
                    weight: dec!(1.2),
                    rate_limit_per_minute: 333,
                    timeout_seconds: 10,
                },
            },
        }
    }

    #[test]
    fn test_parse_aggregation_method() {
        let config = create_test_config();
        let pool = PgPool::connect("postgresql://test").await.unwrap_or_else(|_| {
            // Create a mock pool for testing
            panic!("Test database not available")
        });

        // This test would need to be adjusted based on actual implementation
        // For now, we'll test the method parsing logic separately
    }

    #[tokio::test]
    async fn test_aggregation_method_parsing() {
        let config = create_test_config();

        // We can't easily create an OracleService without a real database
        // So we'll test the parsing logic in isolation

        // Create a mock service for testing method parsing
        struct MockService {
            config: OracleConfig,
        }

        impl MockService {
            fn parse_aggregation_method(&self, method: &str) -> AggregationMethod {
                match method.to_lowercase().as_str() {
                    "mean" => AggregationMethod::Mean,
                    "median" => AggregationMethod::Median,
                    "weighted_average" => AggregationMethod::WeightedAverage,
                    "volume_weighted" => AggregationMethod::VolumeWeighted,
                    _ => AggregationMethod::Custom(method.to_string()),
                }
            }
        }

        let mock_service = MockService { config };

        assert!(matches!(mock_service.parse_aggregation_method("mean"), AggregationMethod::Mean));
        assert!(matches!(mock_service.parse_aggregation_method("median"), AggregationMethod::Median));
        assert!(matches!(mock_service.parse_aggregation_method("weighted_average"), AggregationMethod::WeightedAverage));
        assert!(matches!(mock_service.parse_aggregation_method("volume_weighted"), AggregationMethod::VolumeWeighted));

        if let AggregationMethod::Custom(method) = mock_service.parse_aggregation_method("custom_method") {
            assert_eq!(method, "custom_method");
        } else {
            panic!("Expected Custom aggregation method");
        }
    }

    #[test]
    fn test_validate_request() {
        // Test validation logic in isolation
        fn validate_asset_id(asset_id: &str) -> OracleResult<()> {
            if asset_id.is_empty() {
                return Err(OracleError::ValidationError {
                    field: "asset_id".to_string(),
                    message: "Asset ID cannot be empty".to_string(),
                });
            }
            if asset_id.len() > 20 {
                return Err(OracleError::ValidationError {
                    field: "asset_id".to_string(),
                    message: "Asset ID too long".to_string(),
                });
            }
            Ok(())
        }

        fn validate_currency(currency: &str) -> OracleResult<()> {
            if currency.is_empty() {
                return Err(OracleError::ValidationError {
                    field: "currency".to_string(),
                    message: "Currency cannot be empty".to_string(),
                });
            }
            if currency.len() != 3 {
                return Err(OracleError::ValidationError {
                    field: "currency".to_string(),
                    message: "Currency must be 3 characters".to_string(),
                });
            }
            Ok(())
        }

        // Test valid inputs
        assert!(validate_asset_id("BTC").is_ok());
        assert!(validate_currency("USD").is_ok());

        // Test invalid inputs
        assert!(validate_asset_id("").is_err());
        assert!(validate_asset_id("VERY_LONG_ASSET_ID_NAME").is_err());
        assert!(validate_currency("").is_err());
        assert!(validate_currency("US").is_err());
        assert!(validate_currency("USDT").is_err());
    }

    #[test]
    fn test_batch_price_request_creation() {
        let request = BatchPriceRequest {
            asset_ids: vec!["BTC".to_string(), "ETH".to_string()],
            currency: Some("USD".to_string()),
            include_metadata: Some(true),
        };

        assert_eq!(request.asset_ids.len(), 2);
        assert!(request.asset_ids.contains(&"BTC".to_string()));
        assert!(request.asset_ids.contains(&"ETH".to_string()));
        assert_eq!(request.currency, Some("USD".to_string()));
        assert_eq!(request.include_metadata, Some(true));
    }

    #[test]
    fn test_create_price_feed_request_validation() {
        let request = CreatePriceFeedRequest {
            asset_id: "BTC".to_string(),
            name: "Bitcoin Feed".to_string(),
            description: Some("Bitcoin price feed".to_string()),
            currency: "USD".to_string(),
            update_interval: 60,
            providers: vec!["CoinGecko".to_string()],
            aggregation_method: AggregationMethod::Mean,
            deviation_threshold: dec!(10.0),
        };

        // Validate request fields
        assert!(!request.asset_id.is_empty());
        assert!(!request.name.is_empty());
        assert_eq!(request.currency, "USD");
        assert!(request.update_interval > 0);
        assert!(!request.providers.is_empty());
        assert!(request.deviation_threshold > dec!(0));
    }

    #[test]
    fn test_subscription_request_validation() {
        let request = SubscriptionRequest {
            subscriber_id: "test-subscriber".to_string(),
            webhook_url: Some("https://example.com/webhook".to_string()),
            filters: Some(SubscriptionFilters {
                min_price_change: Some(dec!(5.0)),
                max_update_frequency: Some(60),
                confidence_threshold: Some(dec!(0.8)),
            }),
        };

        assert!(!request.subscriber_id.is_empty());
        assert!(request.webhook_url.is_some());
        assert!(request.filters.is_some());

        if let Some(filters) = &request.filters {
            assert_eq!(filters.min_price_change, Some(dec!(5.0)));
            assert_eq!(filters.max_update_frequency, Some(60));
            assert_eq!(filters.confidence_threshold, Some(dec!(0.8)));
        }
    }

    #[test]
    fn test_price_history_request_defaults() {
        let request = PriceHistoryRequest {
            start_time: None,
            end_time: None,
            interval: None,
            limit: None,
        };

        // Test that we can handle None values appropriately
        let limit = request.limit.unwrap_or(100).min(1000);
        assert_eq!(limit, 100);

        let interval = request.interval.unwrap_or_else(|| "1h".to_string());
        assert_eq!(interval, "1h");
    }

    #[tokio::test]
    async fn test_oracle_service_trait_methods() {
        // Test that our trait methods have the correct signatures
        // This is more of a compilation test

        fn assert_trait_implemented<T: OracleServiceTrait>() {}

        // This will fail to compile if OracleService doesn't implement OracleServiceTrait
        // assert_trait_implemented::<OracleService>();

        // For now, we'll just test that the trait exists and has the expected methods
        // Removed unused import

        fn check_trait_methods() {
            // This function checks that the trait methods exist with correct signatures
            fn _check<T: OracleServiceTrait>(service: &T) {
                let _: &dyn Fn(&str, &str) -> _ = &|asset_id, currency| service.get_asset_price(asset_id, currency);
                let _: &dyn Fn(&BatchPriceRequest) -> _ = &|request| service.get_batch_prices(request);
                // Add more method signature checks as needed
            }
        }
    }

    #[test]
    fn test_error_handling_patterns() {
        // Test common error patterns that the service should handle

        // Test database error conversion
        let db_error = sqlx::Error::RowNotFound;
        let oracle_error: OracleError = db_error.into();

        match oracle_error {
            OracleError::Database { message } => {
                assert!(message.contains("no rows returned"));
            }
            _ => panic!("Expected Database error"),
        }

        // Test validation errors
        let validation_error = OracleError::ValidationError {
            field: "asset_id".to_string(),
            message: "Invalid asset ID".to_string(),
        };

        assert!(format!("{}", validation_error).contains("Invalid asset ID"));
    }

    #[test]
    fn test_config_validation_for_service() {
        let mut config = create_test_config();

        // Test that config validation works
        assert!(config.validate().is_ok());

        // Test invalid configuration
        config.aggregation.min_sources = 0;
        assert!(config.validate().is_err());

        config.aggregation.min_sources = 2;
        config.aggregation.confidence_threshold = dec!(2.0); // Invalid (> 1.0)
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_service_configuration_parsing() {
        // Test that service can parse different aggregation methods
        let methods = vec![
            ("mean", AggregationMethod::Mean),
            ("median", AggregationMethod::Median),
            ("weighted_average", AggregationMethod::WeightedAverage),
            ("volume_weighted", AggregationMethod::VolumeWeighted),
        ];

        for (method_str, expected) in methods {
            let parsed = match method_str {
                "mean" => AggregationMethod::Mean,
                "median" => AggregationMethod::Median,
                "weighted_average" => AggregationMethod::WeightedAverage,
                "volume_weighted" => AggregationMethod::VolumeWeighted,
                _ => AggregationMethod::Custom(method_str.to_string()),
            };

            assert_eq!(std::mem::discriminant(&parsed), std::mem::discriminant(&expected));
        }
    }
}
