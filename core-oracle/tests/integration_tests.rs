// =====================================================================================
// File: core-oracle/tests/integration_tests.rs
// Description: Integration tests for oracle service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use core_oracle::{
    AggregationConfig, AggregationMethod, BandOracle, ChainlinkOracle, FeedConfig, HealthStatus,
    MultiSourceAggregator, OracleConfig, OracleHealthStatus, OracleProvider, OracleService,
    OracleServiceImpl, PriceAggregator, PriceData, ProviderConfig, PythOracle, TimeInterval,
    TimeSeriesData,
};

/// Test configuration for oracle service
struct OracleTestConfig {
    pub oracle_config: OracleConfig,
    pub aggregation_config: AggregationConfig,
}

impl Default for OracleTestConfig {
    fn default() -> Self {
        let providers = vec![
            ProviderConfig {
                provider: OracleProvider::Chainlink,
                enabled: true,
                api_key: None,
                endpoint: "https://api.chain.link".to_string(),
                timeout_seconds: 30,
                rate_limit_per_second: 10,
                weight: 1.0,
                priority: 1,
            },
            ProviderConfig {
                provider: OracleProvider::BandProtocol,
                enabled: true,
                api_key: None,
                endpoint: "https://laozi1.bandchain.org".to_string(),
                timeout_seconds: 30,
                rate_limit_per_second: 10,
                weight: 0.8,
                priority: 2,
            },
            ProviderConfig {
                provider: OracleProvider::PythNetwork,
                enabled: true,
                api_key: None,
                endpoint: "https://hermes.pyth.network".to_string(),
                timeout_seconds: 30,
                rate_limit_per_second: 10,
                weight: 0.9,
                priority: 3,
            },
        ];

        let feeds = vec![
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

        Self {
            oracle_config: OracleConfig {
                providers,
                feeds,
                aggregation: AggregationConfig {
                    default_method: AggregationMethod::Median,
                    outlier_detection: true,
                    outlier_threshold_percent: 10.0,
                    min_sources_for_consensus: 2,
                    confidence_threshold: 0.8,
                },
                validation: core_oracle::ValidationConfig {
                    enable_deviation_check: true,
                    enable_freshness_check: true,
                    enable_sanity_check: true,
                    max_price_change_percent: 50.0,
                    min_price: Decimal::new(1, 8),
                    max_price: Decimal::new(1_000_000_000, 0),
                },
                update_intervals: HashMap::new(),
                circuit_breaker: core_oracle::CircuitBreakerConfig {
                    enabled: true,
                    failure_threshold: 5,
                    recovery_timeout_seconds: 300,
                    half_open_max_calls: 3,
                },
            },
            aggregation_config: AggregationConfig {
                default_method: AggregationMethod::Median,
                outlier_detection: true,
                outlier_threshold_percent: 10.0,
                min_sources_for_consensus: 2,
                confidence_threshold: 0.8,
            },
        }
    }
}

/// Test helper for creating oracle service
async fn create_oracle_service() -> OracleServiceImpl {
    let config = OracleTestConfig::default();
    let service = OracleServiceImpl::new(config.oracle_config);
    service
        .initialize()
        .await
        .expect("Failed to initialize oracle service");
    service
}

/// Test helper for creating test price data
fn create_test_price_data(provider: OracleProvider, price: f64) -> PriceData {
    PriceData {
        price: Decimal::from_f64_retain(price).unwrap(),
        timestamp: Utc::now(),
        source: provider,
        confidence: 0.95,
        volume: Some(Decimal::new(1000000, 2)),
        market_cap: None,
        deviation: Some(1.5),
        round_id: Some(12345),
    }
}

#[tokio::test]
async fn test_oracle_service_initialization() {
    let service = create_oracle_service().await;
    let health = service.get_health_status().await.unwrap();

    assert_eq!(health.overall_status, HealthStatus::Healthy);
    assert!(!health.provider_status.is_empty());
    assert!(!health.feed_status.is_empty());
}

#[tokio::test]
async fn test_price_retrieval() {
    let service = create_oracle_service().await;

    // Test ETH/USD price retrieval
    let eth_price = service.get_price("ETH/USD").await.unwrap();
    assert!(eth_price.price > Decimal::ZERO);
    assert!(eth_price.confidence > 0.0);
    assert!(eth_price.timestamp <= Utc::now());

    // Test BTC/USD price retrieval
    let btc_price = service.get_price("BTC/USD").await.unwrap();
    assert!(btc_price.price > Decimal::ZERO);
    assert!(btc_price.confidence > 0.0);
}

#[tokio::test]
async fn test_provider_specific_price_retrieval() {
    let service = create_oracle_service().await;

    // Test Chainlink price
    let chainlink_price = service
        .get_price_from_provider("ETH/USD", OracleProvider::Chainlink)
        .await
        .unwrap();
    assert_eq!(chainlink_price.source, OracleProvider::Chainlink);
    assert!(chainlink_price.price > Decimal::ZERO);

    // Test Band Protocol price
    let band_price = service
        .get_price_from_provider("ETH/USD", OracleProvider::BandProtocol)
        .await
        .unwrap();
    assert_eq!(band_price.source, OracleProvider::BandProtocol);
    assert!(band_price.price > Decimal::ZERO);
}

#[tokio::test]
async fn test_historical_price_data() {
    let service = create_oracle_service().await;

    let start_time = Utc::now() - chrono::Duration::hours(24);
    let end_time = Utc::now();

    let historical_data = service
        .get_historical_prices("ETH/USD", start_time, end_time)
        .await
        .unwrap();

    assert_eq!(historical_data.symbol, "ETH/USD");
    assert_eq!(historical_data.interval, TimeInterval::OneHour);
    assert!(!historical_data.points.is_empty());
    assert_eq!(historical_data.start_time, start_time);
    assert_eq!(historical_data.end_time, end_time);

    // Verify price points have valid data
    for point in &historical_data.points {
        assert!(point.open > Decimal::ZERO);
        assert!(point.high >= point.open);
        assert!(point.low <= point.open);
        assert!(point.close > Decimal::ZERO);
        assert!(point.volume > Decimal::ZERO);
        assert!(point.source_count > 0);
    }
}

#[tokio::test]
async fn test_feed_management() {
    let service = create_oracle_service().await;

    // List existing feeds
    let feeds = service.list_feeds().await.unwrap();
    assert!(!feeds.is_empty());
    assert!(feeds.contains(&"ETH/USD".to_string()));
    assert!(feeds.contains(&"BTC/USD".to_string()));

    // Add new feed
    let new_feed_config = FeedConfig {
        id: "SOL/USD".to_string(),
        symbol: "SOLUSD".to_string(),
        base_asset: "SOL".to_string(),
        quote_asset: "USD".to_string(),
        providers: vec![OracleProvider::PythNetwork],
        update_interval_seconds: 60,
        deviation_threshold_percent: 5.0,
        staleness_threshold_seconds: 300,
        min_sources: 1,
        enabled: true,
    };

    let feed_id = service.add_feed(&new_feed_config).await.unwrap();
    assert_eq!(feed_id, "SOL/USD");

    // Verify feed was added
    let updated_feeds = service.list_feeds().await.unwrap();
    assert!(updated_feeds.contains(&"SOL/USD".to_string()));

    // Get feed info
    let feed_info = service.get_feed_info("SOL/USD").await.unwrap();
    assert_eq!(feed_info.id, "SOL/USD");
    assert_eq!(feed_info.base_asset, "SOL");
    assert_eq!(feed_info.quote_asset, "USD");
}

#[tokio::test]
async fn test_chainlink_oracle() {
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

    // Test price retrieval
    let price = oracle.get_price("ETH/USD").await.unwrap();
    assert_eq!(price.source, OracleProvider::Chainlink);
    assert!(price.price > Decimal::ZERO);
    assert!(price.confidence > 0.0);
    assert!(price.round_id.is_some());

    // Test health check
    let health = oracle.get_health().await.unwrap();
    assert_eq!(health.status, core_oracle::HealthStatus::Healthy);
    assert!(health.success_rate > 0.0);
}

#[tokio::test]
async fn test_band_oracle() {
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

    // Test price retrieval
    let price = oracle.get_price("ETH/USD").await.unwrap();
    assert_eq!(price.source, OracleProvider::BandProtocol);
    assert!(price.price > Decimal::ZERO);
    assert!(price.confidence > 0.0);

    // Test health check
    let health = oracle.get_health().await.unwrap();
    assert_eq!(health.status, core_oracle::HealthStatus::Healthy);
}

#[tokio::test]
async fn test_pyth_oracle() {
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

    // Test price retrieval
    let price = oracle.get_price("ETH/USD").await.unwrap();
    assert_eq!(price.source, OracleProvider::PythNetwork);
    assert!(price.price > Decimal::ZERO);
    assert!(price.confidence > 0.0);

    // Test health check
    let health = oracle.get_health().await.unwrap();
    assert_eq!(health.status, core_oracle::HealthStatus::Healthy);
}

#[tokio::test]
async fn test_price_aggregation() {
    let config = OracleTestConfig::default();
    let aggregator = MultiSourceAggregator::new(config.aggregation_config, HashMap::new());

    // Create test price data from multiple sources
    let prices = vec![
        create_test_price_data(OracleProvider::Chainlink, 2000.0),
        create_test_price_data(OracleProvider::BandProtocol, 2005.0),
        create_test_price_data(OracleProvider::PythNetwork, 1995.0),
    ];

    // Test median aggregation
    let aggregated = aggregator
        .aggregate_prices(prices.clone(), AggregationMethod::Median)
        .await
        .unwrap();
    assert_eq!(aggregated.price, Decimal::new(2000, 0)); // Median of 1995, 2000, 2005

    // Test mean aggregation
    let mean_aggregated = aggregator
        .aggregate_prices(prices.clone(), AggregationMethod::Mean)
        .await
        .unwrap();
    assert_eq!(mean_aggregated.price, Decimal::new(2000, 0)); // Mean of 1995, 2000, 2005

    // Test weighted mean aggregation
    let weighted_aggregated = aggregator
        .aggregate_prices(prices, AggregationMethod::WeightedMean)
        .await
        .unwrap();
    assert!(weighted_aggregated.price > Decimal::ZERO);
}

#[tokio::test]
async fn test_consensus_calculation() {
    let config = OracleTestConfig::default();
    let aggregator = MultiSourceAggregator::new(config.aggregation_config, HashMap::new());

    // Test high consensus (similar prices)
    let similar_prices = vec![
        create_test_price_data(OracleProvider::Chainlink, 2000.0),
        create_test_price_data(OracleProvider::BandProtocol, 2001.0),
        create_test_price_data(OracleProvider::PythNetwork, 1999.0),
    ];

    let high_consensus = aggregator
        .calculate_consensus(&similar_prices)
        .await
        .unwrap();
    assert!(high_consensus > 0.9);

    // Test low consensus (divergent prices)
    let divergent_prices = vec![
        create_test_price_data(OracleProvider::Chainlink, 2000.0),
        create_test_price_data(OracleProvider::BandProtocol, 2200.0),
        create_test_price_data(OracleProvider::PythNetwork, 1800.0),
    ];

    let low_consensus = aggregator
        .calculate_consensus(&divergent_prices)
        .await
        .unwrap();
    assert!(low_consensus < 0.8);
}

#[tokio::test]
async fn test_outlier_detection() {
    let config = OracleTestConfig::default();
    let aggregator = MultiSourceAggregator::new(config.aggregation_config, HashMap::new());

    // Create prices with one outlier
    let prices_with_outlier = vec![
        create_test_price_data(OracleProvider::Chainlink, 2000.0),
        create_test_price_data(OracleProvider::BandProtocol, 2005.0),
        create_test_price_data(OracleProvider::PythNetwork, 3000.0), // Outlier
    ];

    let filtered_prices = aggregator
        .filter_outliers(prices_with_outlier, 15.0)
        .await
        .unwrap();

    // Should filter out the outlier
    assert_eq!(filtered_prices.len(), 2);
    assert!(filtered_prices
        .iter()
        .all(|p| p.price.to_f64().unwrap() < 2100.0));
}

#[tokio::test]
async fn test_price_validation() {
    let service = create_oracle_service().await;

    // Test valid price data
    let valid_price = create_test_price_data(OracleProvider::Chainlink, 2000.0);
    let is_valid = service
        .validate_price_data("ETH/USD", &valid_price)
        .await
        .unwrap();
    assert!(is_valid);

    // Test invalid price data (zero price)
    let mut invalid_price = create_test_price_data(OracleProvider::Chainlink, 0.0);
    invalid_price.price = Decimal::ZERO;

    let validation_result = service.validate_price_data("ETH/USD", &invalid_price).await;
    assert!(validation_result.is_err());
}

#[tokio::test]
async fn test_concurrent_price_requests() {
    let service = create_oracle_service().await;
    let mut handles = Vec::new();

    // Spawn multiple concurrent price requests
    for _ in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move { service_clone.get_price("ETH/USD").await });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap().unwrap();
        results.push(result);
    }

    // Verify all requests succeeded
    assert_eq!(results.len(), 10);
    for price in results {
        assert!(price.price > Decimal::ZERO);
        assert!(price.confidence > 0.0);
    }
}

#[tokio::test]
async fn test_oracle_health_monitoring() {
    let service = create_oracle_service().await;

    let health_status = service.get_health_status().await.unwrap();

    assert_eq!(health_status.overall_status, HealthStatus::Healthy);
    assert!(health_status.uptime_percentage > 0.0);
    assert!(health_status.total_requests > 0);
    assert!(health_status.successful_requests > 0);
    assert_eq!(health_status.failed_requests, 0);

    // Check provider health
    assert!(!health_status.provider_status.is_empty());
    for (provider, health) in &health_status.provider_status {
        assert_eq!(health.status, core_oracle::HealthStatus::Healthy);
        assert!(health.success_rate > 0.0);
        assert_eq!(health.consecutive_failures, 0);
    }

    // Check feed health
    assert!(!health_status.feed_status.is_empty());
    for (feed_id, health) in &health_status.feed_status {
        assert_eq!(health.status, core_oracle::HealthStatus::Healthy);
        assert!(health.source_count > 0);
        assert!(health.consensus_score > 0.0);
    }
}
