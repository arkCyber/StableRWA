// =====================================================================================
// File: core-oracle/benches/oracle_benchmarks.rs
// Description: Benchmark tests for oracle service performance
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use core_oracle::{
    AggregationConfig, AggregationMethod, BandOracle, ChainlinkOracle, FeedConfig,
    MultiSourceAggregator, OracleConfig, OracleProvider, OracleService, OracleServiceImpl,
    PriceAggregator, PriceData, ProviderClient, ProviderConfig, PythOracle, TimeInterval,
    TimeSeriesData,
};

/// Benchmark configuration
struct BenchmarkConfig {
    pub oracle_config: OracleConfig,
    pub aggregation_config: AggregationConfig,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        let providers = vec![
            ProviderConfig {
                provider: OracleProvider::Chainlink,
                enabled: true,
                api_key: None,
                endpoint: "https://api.chain.link".to_string(),
                timeout_seconds: 30,
                rate_limit_per_second: 100, // Higher for benchmarking
                weight: 1.0,
                priority: 1,
            },
            ProviderConfig {
                provider: OracleProvider::BandProtocol,
                enabled: true,
                api_key: None,
                endpoint: "https://laozi1.bandchain.org".to_string(),
                timeout_seconds: 30,
                rate_limit_per_second: 100,
                weight: 0.8,
                priority: 2,
            },
            ProviderConfig {
                provider: OracleProvider::PythNetwork,
                enabled: true,
                api_key: None,
                endpoint: "https://hermes.pyth.network".to_string(),
                timeout_seconds: 30,
                rate_limit_per_second: 100,
                weight: 0.9,
                priority: 3,
            },
        ];

        let feeds = (0..100) // Create 100 feeds for benchmarking
            .map(|i| FeedConfig {
                id: format!("BENCH{}/USD", i),
                symbol: format!("BENCH{}USD", i),
                base_asset: format!("BENCH{}", i),
                quote_asset: "USD".to_string(),
                providers: vec![OracleProvider::Chainlink, OracleProvider::BandProtocol],
                update_interval_seconds: 60,
                deviation_threshold_percent: 5.0,
                staleness_threshold_seconds: 300,
                min_sources: 2,
                enabled: true,
            })
            .collect();

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

/// Create test price data for benchmarking
fn create_benchmark_price_data(count: usize, provider: OracleProvider) -> Vec<PriceData> {
    (0..count)
        .map(|i| PriceData {
            price: Decimal::from_f64_retain(1000.0 + (i as f64 * 0.1)).unwrap(),
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
            source: provider,
            confidence: 0.95 - (i as f64 * 0.001), // Slightly decreasing confidence
            volume: Some(Decimal::new(1000000 + i as i64, 2)),
            market_cap: Some(Decimal::new(50000000000 + i as i64 * 1000, 0)),
            deviation: Some(1.5 + (i as f64 * 0.01)),
            round_id: Some(12345 + i as u64),
        })
        .collect()
}

/// Benchmark single price retrieval
fn bench_single_price_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = OracleServiceImpl::new(config.oracle_config);
        service.initialize().await.unwrap();
        service
    });

    c.bench_function("single_price_retrieval", |b| {
        b.to_async(&rt)
            .iter(|| async { service.get_price(black_box("ETH/USD")).await.unwrap() });
    });
}

/// Benchmark batch price retrieval
fn bench_batch_price_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = OracleServiceImpl::new(config.oracle_config);
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("batch_price_retrieval");

    for batch_size in [1, 10, 50, 100].iter() {
        let feed_ids: Vec<String> = (0..*batch_size)
            .map(|i| format!("BENCH{}/USD", i))
            .collect();

        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_prices", batch_size),
            batch_size,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let mut prices = Vec::new();
                    for feed_id in &feed_ids {
                        if let Ok(price) = service.get_price(black_box(feed_id)).await {
                            prices.push(price);
                        }
                    }
                    prices
                });
            },
        );
    }

    group.finish();
}

/// Benchmark provider-specific price retrieval
fn bench_provider_price_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = OracleServiceImpl::new(config.oracle_config);
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("provider_price_retrieval");

    let providers = vec![
        OracleProvider::Chainlink,
        OracleProvider::BandProtocol,
        OracleProvider::PythNetwork,
    ];

    for provider in providers {
        group.bench_function(&format!("{:?}", provider), |b| {
            b.to_async(&rt).iter(|| async {
                service
                    .get_price_from_provider(black_box("ETH/USD"), black_box(provider))
                    .await
                    .unwrap()
            });
        });
    }

    group.finish();
}

/// Benchmark price aggregation methods
fn bench_price_aggregation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let aggregator = MultiSourceAggregator::new(config.aggregation_config, HashMap::new());

    let mut group = c.benchmark_group("price_aggregation");

    let methods = vec![
        AggregationMethod::Mean,
        AggregationMethod::Median,
        AggregationMethod::WeightedMean,
    ];

    for method in methods {
        for price_count in [3, 10, 50, 100].iter() {
            let prices = create_benchmark_price_data(*price_count, OracleProvider::Chainlink);

            group.throughput(Throughput::Elements(*price_count as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{:?}", method), price_count),
                price_count,
                |b, _| {
                    b.to_async(&rt).iter(|| async {
                        aggregator
                            .aggregate_prices(black_box(prices.clone()), black_box(method))
                            .await
                            .unwrap()
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark consensus calculation
fn bench_consensus_calculation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let aggregator = MultiSourceAggregator::new(config.aggregation_config, HashMap::new());

    let mut group = c.benchmark_group("consensus_calculation");

    for price_count in [3, 10, 50, 100].iter() {
        let prices = create_benchmark_price_data(*price_count, OracleProvider::Chainlink);

        group.throughput(Throughput::Elements(*price_count as u64));
        group.bench_with_input(
            BenchmarkId::new("consensus", price_count),
            price_count,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    aggregator
                        .calculate_consensus(black_box(&prices))
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark outlier detection
fn bench_outlier_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let aggregator = MultiSourceAggregator::new(config.aggregation_config, HashMap::new());

    let mut group = c.benchmark_group("outlier_detection");

    for price_count in [10, 50, 100, 500].iter() {
        let mut prices = create_benchmark_price_data(*price_count, OracleProvider::Chainlink);

        // Add some outliers
        if *price_count > 10 {
            prices[5].price = Decimal::from_f64_retain(10000.0).unwrap(); // Outlier
            prices[*price_count - 1].price = Decimal::from_f64_retain(100.0).unwrap();
            // Another outlier
        }

        group.throughput(Throughput::Elements(*price_count as u64));
        group.bench_with_input(
            BenchmarkId::new("outlier_detection", price_count),
            price_count,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    aggregator
                        .filter_outliers(black_box(prices.clone()), black_box(15.0))
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark historical data retrieval
fn bench_historical_data_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = OracleServiceImpl::new(config.oracle_config);
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("historical_data_retrieval");

    let time_ranges = vec![
        ("1_hour", chrono::Duration::hours(1)),
        ("1_day", chrono::Duration::days(1)),
        ("1_week", chrono::Duration::weeks(1)),
    ];

    for (name, duration) in time_ranges {
        let start_time = Utc::now() - duration;
        let end_time = Utc::now();

        group.bench_function(name, |b| {
            b.to_async(&rt).iter(|| async {
                service
                    .get_historical_prices(
                        black_box("ETH/USD"),
                        black_box(start_time),
                        black_box(end_time),
                    )
                    .await
                    .unwrap()
            });
        });
    }

    group.finish();
}

/// Benchmark concurrent price requests
fn bench_concurrent_price_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = OracleServiceImpl::new(config.oracle_config);
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("concurrent_price_requests");

    for concurrency in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_requests", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for i in 0..concurrency {
                        let service_clone = service.clone();
                        let feed_id = format!("BENCH{}/USD", i % 10);

                        let handle = tokio::spawn(async move {
                            service_clone.get_price(&feed_id).await.unwrap()
                        });
                        handles.push(handle);
                    }

                    let mut results = Vec::new();
                    for handle in handles {
                        results.push(handle.await.unwrap());
                    }
                    results
                });
            },
        );
    }

    group.finish();
}

/// Benchmark oracle health monitoring
fn bench_health_monitoring(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = OracleServiceImpl::new(config.oracle_config);
        service.initialize().await.unwrap();
        service
    });

    c.bench_function("health_status_check", |b| {
        b.to_async(&rt)
            .iter(|| async { service.get_health_status().await.unwrap() });
    });
}

/// Benchmark feed management operations
fn bench_feed_management(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = OracleServiceImpl::new(config.oracle_config);
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("feed_management");

    group.bench_function("list_feeds", |b| {
        b.to_async(&rt)
            .iter(|| async { service.list_feeds().await.unwrap() });
    });

    group.bench_function("get_feed_info", |b| {
        b.to_async(&rt)
            .iter(|| async { service.get_feed_info(black_box("ETH/USD")).await.unwrap() });
    });

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();

    c.bench_function("service_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let service = OracleServiceImpl::new(black_box(config.oracle_config.clone()));
            service.initialize().await.unwrap();
            service
        });
    });
}

/// Benchmark individual oracle providers
fn bench_oracle_providers(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("oracle_providers");

    // Benchmark Chainlink
    let chainlink_config = ProviderConfig {
        provider: OracleProvider::Chainlink,
        enabled: true,
        api_key: None,
        endpoint: "https://api.chain.link".to_string(),
        timeout_seconds: 30,
        rate_limit_per_second: 100,
        weight: 1.0,
        priority: 1,
    };
    let chainlink_oracle = ChainlinkOracle::new(chainlink_config);

    group.bench_function("chainlink_price_fetch", |b| {
        b.to_async(&rt).iter(|| async {
            chainlink_oracle
                .get_price(black_box("ETH/USD"))
                .await
                .unwrap()
        });
    });

    // Benchmark Band Protocol
    let band_config = ProviderConfig {
        provider: OracleProvider::BandProtocol,
        enabled: true,
        api_key: None,
        endpoint: "https://laozi1.bandchain.org".to_string(),
        timeout_seconds: 30,
        rate_limit_per_second: 100,
        weight: 1.0,
        priority: 2,
    };
    let band_oracle = BandOracle::new(band_config);

    group.bench_function("band_price_fetch", |b| {
        b.to_async(&rt)
            .iter(|| async { band_oracle.get_price(black_box("ETH/USD")).await.unwrap() });
    });

    // Benchmark Pyth Network
    let pyth_config = ProviderConfig {
        provider: OracleProvider::PythNetwork,
        enabled: true,
        api_key: None,
        endpoint: "https://hermes.pyth.network".to_string(),
        timeout_seconds: 30,
        rate_limit_per_second: 100,
        weight: 1.0,
        priority: 3,
    };
    let pyth_oracle = PythOracle::new(pyth_config);

    group.bench_function("pyth_price_fetch", |b| {
        b.to_async(&rt)
            .iter(|| async { pyth_oracle.get_price(black_box("ETH/USD")).await.unwrap() });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_price_retrieval,
    bench_batch_price_retrieval,
    bench_provider_price_retrieval,
    bench_price_aggregation,
    bench_consensus_calculation,
    bench_outlier_detection,
    bench_historical_data_retrieval,
    bench_concurrent_price_requests,
    bench_health_monitoring,
    bench_feed_management,
    bench_memory_usage,
    bench_oracle_providers
);

criterion_main!(benches);
