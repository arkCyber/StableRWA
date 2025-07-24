// =====================================================================================
// File: core-monitoring/benches/monitoring_benchmarks.rs
// Description: Benchmark tests for monitoring service performance
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use core_monitoring::{
    AggregationType, AlertConfig, AlertManager, AlertManagerImpl, ApplicationMetricsCollector,
    DashboardConfig, DashboardManager, DashboardManagerImpl, LogAggregator, LogAggregatorImpl,
    LogConfig, LogEntry, LogLevel, Metric, MetricQuery, MetricType, MetricValue, MetricsCollector,
    MonitoringConfig, MonitoringService, MonitoringServiceImpl, SystemMetricsCollector,
};

/// Benchmark configuration
struct BenchmarkConfig {
    pub monitoring_config: MonitoringConfig,
    pub alert_config: AlertConfig,
    pub dashboard_config: DashboardConfig,
    pub log_config: LogConfig,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            monitoring_config: MonitoringConfig {
                enabled: true,
                collection_interval_seconds: 10,
                retention_days: 7,
                max_metrics_per_batch: 1000,
                alert_evaluation_interval_seconds: 30,
            },
            alert_config: AlertConfig {
                enabled: true,
                evaluation_interval_seconds: 10,
                notification_channels: vec!["benchmark".to_string()],
                rules: Vec::new(),
            },
            dashboard_config: DashboardConfig {
                enabled: true,
                refresh_interval_seconds: 30,
                default_time_range_hours: 24,
                max_widgets_per_dashboard: 20,
            },
            log_config: LogConfig {
                enabled: true,
                log_level: LogLevel::Info,
                retention_days: 7,
                structured_logging: true,
            },
        }
    }
}

/// Create test metrics for benchmarking
fn create_benchmark_metrics(count: usize) -> Vec<Metric> {
    (0..count)
        .map(|i| Metric {
            id: Uuid::new_v4(),
            name: format!("benchmark_metric_{}", i % 10), // Cycle through 10 different metric names
            metric_type: if i % 2 == 0 {
                MetricType::Counter
            } else {
                MetricType::Gauge
            },
            value: if i % 2 == 0 {
                MetricValue::Integer(i as i64)
            } else {
                MetricValue::Float(i as f64 * 1.5)
            },
            labels: {
                let mut labels = HashMap::new();
                labels.insert("service".to_string(), format!("service_{}", i % 5));
                labels.insert("host".to_string(), format!("host_{}", i % 3));
                labels.insert("environment".to_string(), "benchmark".to_string());
                labels
            },
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
        })
        .collect()
}

/// Create test log entries for benchmarking
fn create_benchmark_logs(count: usize) -> Vec<LogEntry> {
    (0..count)
        .map(|i| LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
            level: match i % 4 {
                0 => LogLevel::Debug,
                1 => LogLevel::Info,
                2 => LogLevel::Warning,
                3 => LogLevel::Error,
                _ => LogLevel::Info,
            },
            message: format!("Benchmark log message {}", i),
            source: format!("benchmark_service_{}", i % 5),
            labels: {
                let mut labels = HashMap::new();
                labels.insert("request_id".to_string(), Uuid::new_v4().to_string());
                labels.insert("user_id".to_string(), format!("user_{}", i % 100));
                labels
            },
            fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    "duration_ms".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(i % 1000)),
                );
                fields.insert(
                    "status_code".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(200 + (i % 5))),
                );
                fields
            },
        })
        .collect()
}

/// Benchmark metric storage performance
fn bench_metric_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = MonitoringServiceImpl::new(config.monitoring_config);
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("metric_storage");

    for size in [10, 100, 1000, 10000].iter() {
        let metrics = create_benchmark_metrics(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("store_metrics", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                for metric in &metrics {
                    service.store_metric(black_box(metric)).await.unwrap();
                }
            });
        });
    }

    group.finish();
}

/// Benchmark metric querying performance
fn bench_metric_querying(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = MonitoringServiceImpl::new(config.monitoring_config);
        service.initialize().await.unwrap();

        // Pre-populate with test data
        let metrics = create_benchmark_metrics(10000);
        for metric in &metrics {
            service.store_metric(metric).await.unwrap();
        }

        service
    });

    let mut group = c.benchmark_group("metric_querying");

    // Benchmark different query patterns
    let queries = vec![
        (
            "simple_name_query",
            MetricQuery {
                name: Some("benchmark_metric_0".to_string()),
                labels: HashMap::new(),
                start_time: Utc::now() - chrono::Duration::hours(1),
                end_time: Utc::now(),
                aggregation: None,
                limit: Some(1000),
                offset: Some(0),
            },
        ),
        (
            "label_filter_query",
            MetricQuery {
                name: None,
                labels: {
                    let mut labels = HashMap::new();
                    labels.insert("service".to_string(), "service_0".to_string());
                    labels
                },
                start_time: Utc::now() - chrono::Duration::hours(1),
                end_time: Utc::now(),
                aggregation: None,
                limit: Some(1000),
                offset: Some(0),
            },
        ),
        (
            "aggregated_query",
            MetricQuery {
                name: Some("benchmark_metric_1".to_string()),
                labels: HashMap::new(),
                start_time: Utc::now() - chrono::Duration::hours(1),
                end_time: Utc::now(),
                aggregation: Some(AggregationType::Average),
                limit: Some(1000),
                offset: Some(0),
            },
        ),
    ];

    for (query_name, query) in queries {
        group.bench_function(query_name, |b| {
            b.to_async(&rt)
                .iter(|| async { service.query_metrics(black_box(&query)).await.unwrap() });
        });
    }

    group.finish();
}

/// Benchmark system metrics collection
fn bench_system_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let collector = SystemMetricsCollector::new();

    c.bench_function("system_metrics_collection", |b| {
        b.to_async(&rt)
            .iter(|| async { collector.collect_metrics().await.unwrap() });
    });
}

/// Benchmark application metrics collection
fn bench_application_metrics_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let collector = ApplicationMetricsCollector::new("benchmark_app".to_string());

    c.bench_function("application_metrics_collection", |b| {
        b.to_async(&rt)
            .iter(|| async { collector.collect_metrics().await.unwrap() });
    });
}

/// Benchmark alert evaluation performance
fn bench_alert_evaluation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let alert_manager = AlertManagerImpl::new(config.alert_config);

    let mut group = c.benchmark_group("alert_evaluation");

    for metric_count in [10, 100, 1000].iter() {
        let metrics = create_benchmark_metrics(*metric_count);

        group.throughput(Throughput::Elements(*metric_count as u64));
        group.bench_with_input(
            BenchmarkId::new("evaluate_rules", metric_count),
            metric_count,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    alert_manager
                        .evaluate_rules(black_box(&metrics))
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark log storage performance
fn bench_log_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let log_aggregator = LogAggregatorImpl::new(config.log_config);

    let mut group = c.benchmark_group("log_storage");

    for size in [10, 100, 1000, 10000].iter() {
        let logs = create_benchmark_logs(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("store_logs", size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                for log in &logs {
                    log_aggregator.store_log(black_box(log)).await.unwrap();
                }
            });
        });
    }

    group.finish();
}

/// Benchmark log querying performance
fn bench_log_querying(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let log_aggregator = rt.block_on(async {
        let aggregator = LogAggregatorImpl::new(config.log_config);

        // Pre-populate with test data
        let logs = create_benchmark_logs(10000);
        for log in &logs {
            aggregator.store_log(log).await.unwrap();
        }

        aggregator
    });

    let query = core_monitoring::LogQuery {
        level: Some(LogLevel::Info),
        source: Some("benchmark_service_0".to_string()),
        message_contains: Some("Benchmark".to_string()),
        labels: HashMap::new(),
        start_time: Utc::now() - chrono::Duration::hours(1),
        end_time: Utc::now(),
        limit: Some(1000),
        offset: Some(0),
    };

    c.bench_function("log_querying", |b| {
        b.to_async(&rt)
            .iter(|| async { log_aggregator.query_logs(black_box(&query)).await.unwrap() });
    });
}

/// Benchmark dashboard data generation
fn bench_dashboard_data_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let dashboard_manager = rt.block_on(async {
        let manager = DashboardManagerImpl::new(config.dashboard_config);
        manager
    });

    let time_range = core_monitoring::TimeRange {
        start: Utc::now() - chrono::Duration::hours(1),
        end: Utc::now(),
    };

    c.bench_function("dashboard_data_generation", |b| {
        b.to_async(&rt).iter(|| async {
            let dashboards = dashboard_manager.list_dashboards().await.unwrap();
            if !dashboards.is_empty() {
                dashboard_manager
                    .get_dashboard_data(black_box(&dashboards[0].id), black_box(&time_range))
                    .await
                    .unwrap()
            }
        });
    });
}

/// Benchmark concurrent metric operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();
    let service = rt.block_on(async {
        let service = MonitoringServiceImpl::new(config.monitoring_config);
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("concurrent_operations");

    for concurrency in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_metric_storage", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for _ in 0..concurrency {
                        let service_clone = service.clone();
                        let metric = create_benchmark_metrics(1)[0].clone();

                        let handle = tokio::spawn(async move {
                            service_clone.store_metric(&metric).await.unwrap();
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = BenchmarkConfig::default();

    c.bench_function("service_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let service = MonitoringServiceImpl::new(black_box(config.monitoring_config.clone()));
            service.initialize().await.unwrap();
            service
        });
    });
}

criterion_group!(
    benches,
    bench_metric_storage,
    bench_metric_querying,
    bench_system_metrics_collection,
    bench_application_metrics_collection,
    bench_alert_evaluation,
    bench_log_storage,
    bench_log_querying,
    bench_dashboard_data_generation,
    bench_concurrent_operations,
    bench_memory_usage
);

criterion_main!(benches);
