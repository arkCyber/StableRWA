// =====================================================================================
// RWA Tokenization Platform - Oracle Service Models
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Asset price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPrice {
    pub asset_id: String,
    pub price: Decimal,
    pub currency: String,
    pub timestamp: DateTime<Utc>,
    pub confidence: Decimal, // 0.0 to 1.0
    pub source: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Historical price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceDataPoint {
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub volume: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub open: Option<Decimal>,
    pub close: Option<Decimal>,
}

/// Price feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeed {
    pub id: Uuid,
    pub asset_id: String,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub update_interval: i32, // seconds
    pub providers: Vec<String>,
    pub aggregation_method: AggregationMethod,
    pub deviation_threshold: Decimal, // percentage
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price aggregation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationMethod {
    Mean,
    Median,
    WeightedAverage,
    VolumeWeighted,
    Custom(String),
}

/// Price provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceProvider {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub api_endpoint: String,
    pub api_key: Option<String>,
    pub weight: Decimal, // for weighted aggregation
    pub timeout_seconds: u64,
    pub rate_limit: Option<RateLimit>,
    pub is_active: bool,
    pub last_update: Option<DateTime<Utc>>,
    pub error_count: i32,
    pub success_rate: Decimal,
}

/// Provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    CentralizedExchange,
    DecentralizedExchange,
    PriceOracle,
    MarketData,
    RealEstate,
    Commodity,
    CoinMarketCap,
    Chainlink,
    Binance,
    CoinGecko,
    Custom(String),
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

/// Price subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSubscription {
    pub id: Uuid,
    pub feed_id: Uuid,
    pub subscriber_id: String,
    pub webhook_url: Option<String>,
    pub filters: Option<SubscriptionFilters>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Subscription filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilters {
    pub min_price_change: Option<Decimal>,
    pub max_update_frequency: Option<i32>, // seconds
    pub confidence_threshold: Option<Decimal>,
}

/// Price alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: Uuid,
    pub asset_id: String,
    pub alert_type: AlertType,
    pub threshold: Decimal,
    pub condition: AlertCondition,
    pub recipient: String,
    pub is_active: bool,
    pub triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    PriceThreshold,
    PercentageChange,
    VolumeSpike,
    LowConfidence,
    ProviderFailure,
}

/// Alert conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
    Equal,
    PercentageIncrease(Decimal),
    PercentageDecrease(Decimal),
}

/// Batch price request
#[derive(Debug, Deserialize)]
pub struct BatchPriceRequest {
    pub asset_ids: Vec<String>,
    pub currency: Option<String>,
    pub include_metadata: Option<bool>,
}

/// Batch price response
#[derive(Debug, Serialize)]
pub struct BatchPriceResponse {
    pub prices: HashMap<String, AssetPrice>,
    pub errors: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Price history request parameters
#[derive(Debug, Deserialize)]
pub struct PriceHistoryRequest {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub interval: Option<String>, // "1m", "5m", "1h", "1d", etc.
    pub limit: Option<i32>,
}

/// Price history response
#[derive(Debug, Serialize)]
pub struct PriceHistoryResponse {
    pub asset_id: String,
    pub currency: String,
    pub data: Vec<PriceDataPoint>,
    pub interval: String,
    pub total_count: i64,
}

/// Provider status information
#[derive(Debug, Serialize)]
pub struct ProviderStatus {
    pub provider_id: String,
    pub is_healthy: bool,
    pub last_successful_update: Option<DateTime<Utc>>,
    pub error_count: i32,
    pub success_rate: Decimal,
    pub average_response_time: Option<f64>, // milliseconds
    pub current_error: Option<String>,
}

/// Oracle health status
#[derive(Debug, Serialize)]
pub struct OracleHealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub active_feeds: i32,
    pub healthy_providers: i32,
    pub total_providers: i32,
    pub last_price_update: Option<DateTime<Utc>>,
    pub system_load: Option<f64>,
}

/// Price feed creation request
#[derive(Debug, Deserialize)]
pub struct CreatePriceFeedRequest {
    pub asset_id: String,
    pub name: String,
    pub description: Option<String>,
    pub currency: String,
    pub update_interval: i32,
    pub providers: Vec<String>,
    pub aggregation_method: AggregationMethod,
    pub deviation_threshold: Decimal,
}

/// Price feed update request
#[derive(Debug, Deserialize)]
pub struct UpdatePriceFeedRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub update_interval: Option<i32>,
    pub providers: Option<Vec<String>>,
    pub aggregation_method: Option<AggregationMethod>,
    pub deviation_threshold: Option<Decimal>,
    pub is_active: Option<bool>,
}

/// Subscription request
#[derive(Debug, Deserialize)]
pub struct SubscriptionRequest {
    pub subscriber_id: String,
    pub webhook_url: Option<String>,
    pub filters: Option<SubscriptionFilters>,
}

impl Default for AggregationMethod {
    fn default() -> Self {
        AggregationMethod::Mean
    }
}

impl std::fmt::Display for AggregationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregationMethod::Mean => write!(f, "mean"),
            AggregationMethod::Median => write!(f, "median"),
            AggregationMethod::WeightedAverage => write!(f, "weighted_average"),
            AggregationMethod::VolumeWeighted => write!(f, "volume_weighted"),
            AggregationMethod::Custom(method) => write!(f, "custom_{}", method),
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::CentralizedExchange => write!(f, "centralized_exchange"),
            ProviderType::DecentralizedExchange => write!(f, "decentralized_exchange"),
            ProviderType::PriceOracle => write!(f, "price_oracle"),
            ProviderType::MarketData => write!(f, "market_data"),
            ProviderType::RealEstate => write!(f, "real_estate"),
            ProviderType::Commodity => write!(f, "commodity"),
            ProviderType::CoinMarketCap => write!(f, "coinmarketcap"),
            ProviderType::Chainlink => write!(f, "chainlink"),
            ProviderType::Binance => write!(f, "binance"),
            ProviderType::CoinGecko => write!(f, "coingecko"),
            ProviderType::Custom(type_name) => write!(f, "custom_{}", type_name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use serde_json::json;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_asset_price_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("volume".to_string(), json!("1000000"));

        let price = AssetPrice {
            asset_id: "BTC".to_string(),
            price: dec!(50000.0),
            currency: "USD".to_string(),
            timestamp: Utc::now(),
            confidence: dec!(0.95),
            source: "CoinGecko".to_string(),
            metadata: Some(metadata),
        };

        assert_eq!(price.asset_id, "BTC");
        assert_eq!(price.price, dec!(50000.0));
        assert_eq!(price.currency, "USD");
        assert_eq!(price.confidence, dec!(0.95));
        assert_eq!(price.source, "CoinGecko");
        assert!(price.metadata.is_some());
    }

    #[test]
    fn test_asset_price_serialization() {
        let price = AssetPrice {
            asset_id: "ETH".to_string(),
            price: dec!(3000.0),
            currency: "USD".to_string(),
            timestamp: Utc::now(),
            confidence: dec!(0.90),
            source: "Binance".to_string(),
            metadata: None,
        };

        let json = serde_json::to_string(&price).expect("Failed to serialize");
        let deserialized: AssetPrice = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(price.asset_id, deserialized.asset_id);
        assert_eq!(price.price, deserialized.price);
        assert_eq!(price.currency, deserialized.currency);
        assert_eq!(price.confidence, deserialized.confidence);
        assert_eq!(price.source, deserialized.source);
    }

    #[test]
    fn test_aggregation_method_serialization() {
        let methods = vec![
            AggregationMethod::Mean,
            AggregationMethod::Median,
            AggregationMethod::WeightedAverage,
            AggregationMethod::VolumeWeighted,
            AggregationMethod::Custom("custom_method".to_string()),
        ];

        for method in methods {
            let json = serde_json::to_string(&method).expect("Failed to serialize");
            let deserialized: AggregationMethod = serde_json::from_str(&json)
                .expect("Failed to deserialize");
            assert_eq!(method, deserialized);
        }
    }

    #[test]
    fn test_aggregation_method_display() {
        assert_eq!(format!("{}", AggregationMethod::Mean), "mean");
        assert_eq!(format!("{}", AggregationMethod::Median), "median");
        assert_eq!(format!("{}", AggregationMethod::WeightedAverage), "weighted_average");
        assert_eq!(format!("{}", AggregationMethod::VolumeWeighted), "volume_weighted");
        assert_eq!(format!("{}", AggregationMethod::Custom("test".to_string())), "custom_test");
    }

    #[test]
    fn test_aggregation_method_default() {
        let default_method = AggregationMethod::default();
        assert!(matches!(default_method, AggregationMethod::Mean));
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(format!("{}", ProviderType::MarketData), "market_data");
        assert_eq!(format!("{}", ProviderType::CentralizedExchange), "centralized_exchange");
        assert_eq!(format!("{}", ProviderType::DecentralizedExchange), "decentralized_exchange");
        assert_eq!(format!("{}", ProviderType::PriceOracle), "price_oracle");
        assert_eq!(format!("{}", ProviderType::RealEstate), "real_estate");
        assert_eq!(format!("{}", ProviderType::Commodity), "commodity");
        assert_eq!(format!("{}", ProviderType::Custom("test".to_string())), "custom_test");
    }

    #[test]
    fn test_price_feed_creation() {
        let feed = PriceFeed {
            id: Uuid::new_v4(),
            asset_id: "BTC".to_string(),
            name: "Bitcoin USD Feed".to_string(),
            description: Some("Bitcoin price feed in USD".to_string()),
            currency: "USD".to_string(),
            update_interval: 60,
            providers: vec!["CoinGecko".to_string(), "Binance".to_string()],
            aggregation_method: AggregationMethod::WeightedAverage,
            deviation_threshold: dec!(10.0),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(feed.asset_id, "BTC");
        assert_eq!(feed.currency, "USD");
        assert_eq!(feed.update_interval, 60);
        assert_eq!(feed.providers.len(), 2);
        assert!(feed.is_active);
        assert!(matches!(feed.aggregation_method, AggregationMethod::WeightedAverage));
    }

    #[test]
    fn test_price_provider_creation() {
        let provider = PriceProvider {
            id: "coingecko".to_string(),
            name: "CoinGecko".to_string(),
            provider_type: ProviderType::MarketData,
            api_endpoint: "https://api.coingecko.com/api/v3".to_string(),
            api_key: Some("test-key".to_string()),
            weight: dec!(1.0),
            timeout_seconds: 30,
            rate_limit: Some(RateLimit {
                requests_per_minute: 50,
                burst_size: 10,
            }),
            is_active: true,
            last_update: Some(Utc::now()),
            error_count: 0,
            success_rate: dec!(0.99),
        };

        assert_eq!(provider.id, "coingecko");
        assert_eq!(provider.name, "CoinGecko");
        assert!(matches!(provider.provider_type, ProviderType::MarketData));
        assert!(provider.is_active);
        assert_eq!(provider.error_count, 0);
        assert!(provider.rate_limit.is_some());
    }

    #[test]
    fn test_rate_limit_creation() {
        let rate_limit = RateLimit {
            requests_per_minute: 100,
            burst_size: 20,
        };

        assert_eq!(rate_limit.requests_per_minute, 100);
        assert_eq!(rate_limit.burst_size, 20);
    }

    #[test]
    fn test_price_subscription_creation() {
        let subscription = PriceSubscription {
            id: Uuid::new_v4(),
            feed_id: Uuid::new_v4(),
            subscriber_id: "test-subscriber".to_string(),
            webhook_url: Some("https://example.com/webhook".to_string()),
            filters: Some(SubscriptionFilters {
                min_price_change: Some(dec!(5.0)),
                max_update_frequency: Some(60),
                confidence_threshold: Some(dec!(0.8)),
            }),
            is_active: true,
            created_at: Utc::now(),
        };

        assert_eq!(subscription.subscriber_id, "test-subscriber");
        assert!(subscription.is_active);
        assert!(subscription.webhook_url.is_some());
        assert!(subscription.filters.is_some());
    }

    #[test]
    fn test_subscription_filters_creation() {
        let filters = SubscriptionFilters {
            min_price_change: Some(dec!(2.5)),
            max_update_frequency: Some(30),
            confidence_threshold: Some(dec!(0.9)),
        };

        assert_eq!(filters.min_price_change, Some(dec!(2.5)));
        assert_eq!(filters.max_update_frequency, Some(30));
        assert_eq!(filters.confidence_threshold, Some(dec!(0.9)));
    }

    #[test]
    fn test_price_alert_creation() {
        let alert = PriceAlert {
            id: Uuid::new_v4(),
            asset_id: "BTC".to_string(),
            alert_type: AlertType::PriceThreshold,
            threshold: dec!(60000.0),
            condition: AlertCondition::Above,
            recipient: "user@example.com".to_string(),
            is_active: true,
            triggered_at: None,
            created_at: Utc::now(),
        };

        assert_eq!(alert.asset_id, "BTC");
        assert!(matches!(alert.alert_type, AlertType::PriceThreshold));
        assert!(matches!(alert.condition, AlertCondition::Above));
        assert_eq!(alert.threshold, dec!(60000.0));
        assert!(alert.is_active);
        assert!(alert.triggered_at.is_none());
    }

    #[test]
    fn test_alert_condition_serialization() {
        let conditions = vec![
            AlertCondition::Above,
            AlertCondition::Below,
            AlertCondition::Equal,
            AlertCondition::PercentageIncrease(dec!(10.0)),
            AlertCondition::PercentageDecrease(dec!(5.0)),
        ];

        for condition in conditions {
            let json = serde_json::to_string(&condition).expect("Failed to serialize");
            let deserialized: AlertCondition = serde_json::from_str(&json)
                .expect("Failed to deserialize");
            assert_eq!(condition, deserialized);
        }
    }

    #[test]
    fn test_batch_price_request_validation() {
        let request = BatchPriceRequest {
            asset_ids: vec!["BTC".to_string(), "ETH".to_string()],
            currency: Some("USD".to_string()),
            include_metadata: Some(true),
        };

        assert_eq!(request.asset_ids.len(), 2);
        assert_eq!(request.currency, Some("USD".to_string()));
        assert_eq!(request.include_metadata, Some(true));
    }

    #[test]
    fn test_batch_price_response_creation() {
        let mut prices = HashMap::new();
        prices.insert("BTC".to_string(), AssetPrice {
            asset_id: "BTC".to_string(),
            price: dec!(50000.0),
            currency: "USD".to_string(),
            timestamp: Utc::now(),
            confidence: dec!(0.95),
            source: "test".to_string(),
            metadata: None,
        });

        let mut errors = HashMap::new();
        errors.insert("ETH".to_string(), "Provider unavailable".to_string());

        let response = BatchPriceResponse {
            prices,
            errors,
            timestamp: Utc::now(),
        };

        assert_eq!(response.prices.len(), 1);
        assert_eq!(response.errors.len(), 1);
        assert!(response.prices.contains_key("BTC"));
        assert!(response.errors.contains_key("ETH"));
    }

    #[test]
    fn test_price_history_request_validation() {
        let request = PriceHistoryRequest {
            start_time: Some(Utc::now() - chrono::Duration::hours(24)),
            end_time: Some(Utc::now()),
            interval: Some("1h".to_string()),
            limit: Some(100),
        };

        assert!(request.start_time.is_some());
        assert!(request.end_time.is_some());
        assert_eq!(request.interval, Some("1h".to_string()));
        assert_eq!(request.limit, Some(100));
    }

    #[test]
    fn test_price_data_point_creation() {
        let data_point = PriceDataPoint {
            timestamp: Utc::now(),
            price: dec!(50000.0),
            volume: Some(dec!(1000.0)),
            high: Some(dec!(51000.0)),
            low: Some(dec!(49000.0)),
            open: Some(dec!(49500.0)),
            close: Some(dec!(50000.0)),
        };

        assert_eq!(data_point.price, dec!(50000.0));
        assert_eq!(data_point.volume, Some(dec!(1000.0)));
        assert_eq!(data_point.high, Some(dec!(51000.0)));
        assert_eq!(data_point.low, Some(dec!(49000.0)));
    }

    #[test]
    fn test_provider_status_creation() {
        let status = ProviderStatus {
            provider_id: "coingecko".to_string(),
            is_healthy: true,
            last_successful_update: Some(Utc::now()),
            error_count: 0,
            success_rate: dec!(0.99),
            average_response_time: Some(250.0),
            current_error: None,
        };

        assert_eq!(status.provider_id, "coingecko");
        assert!(status.is_healthy);
        assert_eq!(status.error_count, 0);
        assert_eq!(status.success_rate, dec!(0.99));
        assert!(status.current_error.is_none());
    }

    #[test]
    fn test_oracle_health_status_creation() {
        let health = OracleHealthStatus {
            status: "healthy".to_string(),
            timestamp: Utc::now(),
            active_feeds: 5,
            healthy_providers: 3,
            total_providers: 3,
            last_price_update: Some(Utc::now()),
            system_load: Some(0.75),
        };

        assert_eq!(health.status, "healthy");
        assert_eq!(health.active_feeds, 5);
        assert_eq!(health.healthy_providers, 3);
        assert_eq!(health.total_providers, 3);
        assert!(health.last_price_update.is_some());
        assert_eq!(health.system_load, Some(0.75));
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

        assert_eq!(request.asset_id, "BTC");
        assert_eq!(request.update_interval, 60);
        assert!(!request.providers.is_empty());
        assert!(request.deviation_threshold > dec!(0));
        assert!(matches!(request.aggregation_method, AggregationMethod::Mean));
    }

    #[test]
    fn test_update_price_feed_request_validation() {
        let request = UpdatePriceFeedRequest {
            name: Some("Updated Bitcoin Feed".to_string()),
            description: Some("Updated description".to_string()),
            update_interval: Some(120),
            providers: Some(vec!["CoinGecko".to_string(), "Binance".to_string()]),
            aggregation_method: Some(AggregationMethod::WeightedAverage),
            deviation_threshold: Some(dec!(15.0)),
            is_active: Some(false),
        };

        assert_eq!(request.name, Some("Updated Bitcoin Feed".to_string()));
        assert_eq!(request.update_interval, Some(120));
        assert_eq!(request.providers.as_ref().unwrap().len(), 2);
        assert_eq!(request.is_active, Some(false));
    }

    #[test]
    fn test_subscription_request_validation() {
        let request = SubscriptionRequest {
            subscriber_id: "test-subscriber".to_string(),
            webhook_url: Some("https://example.com/webhook".to_string()),
            filters: Some(SubscriptionFilters {
                min_price_change: Some(dec!(5.0)),
                max_update_frequency: None,
                confidence_threshold: Some(dec!(0.8)),
            }),
        };

        assert_eq!(request.subscriber_id, "test-subscriber");
        assert!(request.webhook_url.is_some());
        assert!(request.filters.is_some());
    }

    #[test]
    fn test_model_serialization_roundtrip() {
        let original_feed = PriceFeed {
            id: Uuid::new_v4(),
            asset_id: "BTC".to_string(),
            name: "Bitcoin Feed".to_string(),
            description: Some("Test feed".to_string()),
            currency: "USD".to_string(),
            update_interval: 60,
            providers: vec!["test".to_string()],
            aggregation_method: AggregationMethod::Mean,
            deviation_threshold: dec!(10.0),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&original_feed).expect("Failed to serialize");
        let deserialized: PriceFeed = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(original_feed.id, deserialized.id);
        assert_eq!(original_feed.asset_id, deserialized.asset_id);
        assert_eq!(original_feed.name, deserialized.name);
        assert_eq!(original_feed.currency, deserialized.currency);
    }

    #[test]
    fn test_model_clone_functionality() {
        let original = AssetPrice {
            asset_id: "BTC".to_string(),
            price: dec!(50000.0),
            currency: "USD".to_string(),
            timestamp: Utc::now(),
            confidence: dec!(0.95),
            source: "test".to_string(),
            metadata: None,
        };

        let cloned = original.clone();
        assert_eq!(original.asset_id, cloned.asset_id);
        assert_eq!(original.price, cloned.price);
        assert_eq!(original.currency, cloned.currency);
        assert_eq!(original.confidence, cloned.confidence);
    }
}
