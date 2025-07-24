// =====================================================================================
// RWA Tokenization Platform - Oracle Metrics Collection
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use prometheus::{
    Counter, Gauge, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec, Registry, TextEncoder
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// Oracle service metrics collector
pub struct OracleMetrics {
    registry: Registry,
    
    // Request metrics
    pub requests_total: IntCounterVec,
    pub request_duration: HistogramVec,
    pub active_requests: IntGauge,
    
    // Price metrics
    pub prices_fetched_total: IntCounterVec,
    pub price_fetch_duration: HistogramVec,
    pub price_aggregation_duration: HistogramVec,
    pub price_cache_hits: IntCounter,
    pub price_cache_misses: IntCounter,
    
    // Provider metrics
    pub provider_requests_total: IntCounterVec,
    pub provider_errors_total: IntCounterVec,
    pub provider_response_time: HistogramVec,
    pub provider_health_status: IntGaugeVec,
    
    // Database metrics
    pub db_connections_active: IntGauge,
    pub db_connections_idle: IntGauge,
    pub db_query_duration: HistogramVec,
    pub db_errors_total: IntCounterVec,
    
    // Cache metrics
    pub cache_operations_total: IntCounterVec,
    pub cache_memory_usage: Gauge,
    pub cache_key_count: IntGauge,
    pub cache_hit_rate: Gauge,
    
    // Business metrics
    pub feeds_active: IntGauge,
    pub subscriptions_active: IntGauge,
    pub price_deviations_total: IntCounter,
    pub outliers_detected_total: IntCounter,
    
    // System metrics
    pub memory_usage_bytes: Gauge,
    pub cpu_usage_percent: Gauge,
    pub uptime_seconds: Counter,
    
    // Custom metrics storage (simplified for now)
    #[allow(dead_code)]
    custom_metrics: Arc<RwLock<HashMap<String, String>>>,
}

impl OracleMetrics {
    /// Create new metrics collector
    pub fn new() -> prometheus::Result<Self> {
        let registry = Registry::new();
        
        // Request metrics
        let requests_total = IntCounterVec::new(
            prometheus::Opts::new("oracle_requests_total", "Total number of requests")
                .namespace("rwa")
                .subsystem("oracle"),
            &["method", "endpoint", "status"]
        )?;
        
        let request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("oracle_request_duration_seconds", "Request duration in seconds")
                .namespace("rwa")
                .subsystem("oracle")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "endpoint"]
        )?;
        
        let active_requests = IntGauge::new(
            "oracle_active_requests",
            "Number of active requests"
        )?;
        
        // Price metrics
        let prices_fetched_total = IntCounterVec::new(
            prometheus::Opts::new("oracle_prices_fetched_total", "Total number of prices fetched")
                .namespace("rwa")
                .subsystem("oracle"),
            &["asset_id", "currency", "source"]
        )?;
        
        let price_fetch_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("oracle_price_fetch_duration_seconds", "Price fetch duration")
                .namespace("rwa")
                .subsystem("oracle")
                .buckets(vec![0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]),
            &["asset_id", "source"]
        )?;
        
        let price_aggregation_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("oracle_price_aggregation_duration_seconds", "Price aggregation duration")
                .namespace("rwa")
                .subsystem("oracle")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5]),
            &["asset_id", "method"]
        )?;
        
        let price_cache_hits = IntCounter::new(
            "oracle_price_cache_hits_total",
            "Total cache hits"
        )?;
        
        let price_cache_misses = IntCounter::new(
            "oracle_price_cache_misses_total",
            "Total cache misses"
        )?;
        
        // Provider metrics
        let provider_requests_total = IntCounterVec::new(
            prometheus::Opts::new("oracle_provider_requests_total", "Total provider requests")
                .namespace("rwa")
                .subsystem("oracle"),
            &["provider", "status"]
        )?;
        
        let provider_errors_total = IntCounterVec::new(
            prometheus::Opts::new("oracle_provider_errors_total", "Total provider errors")
                .namespace("rwa")
                .subsystem("oracle"),
            &["provider", "error_type"]
        )?;
        
        let provider_response_time = HistogramVec::new(
            prometheus::HistogramOpts::new("oracle_provider_response_time_seconds", "Provider response time")
                .namespace("rwa")
                .subsystem("oracle")
                .buckets(vec![0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0]),
            &["provider"]
        )?;
        
        let provider_health_status = IntGaugeVec::new(
            prometheus::Opts::new("oracle_provider_health_status", "Provider health status (1=healthy, 0=unhealthy)")
                .namespace("rwa")
                .subsystem("oracle"),
            &["provider"]
        )?;
        
        // Database metrics
        let db_connections_active = IntGauge::new(
            "oracle_db_connections_active",
            "Active database connections"
        )?;
        
        let db_connections_idle = IntGauge::new(
            "oracle_db_connections_idle",
            "Idle database connections"
        )?;
        
        let db_query_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("oracle_db_query_duration_seconds", "Database query duration")
                .namespace("rwa")
                .subsystem("oracle")
                .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["query_type"]
        )?;
        
        let db_errors_total = IntCounterVec::new(
            prometheus::Opts::new("oracle_db_errors_total", "Total database errors")
                .namespace("rwa")
                .subsystem("oracle"),
            &["error_type"]
        )?;
        
        // Cache metrics
        let cache_operations_total = IntCounterVec::new(
            prometheus::Opts::new("oracle_cache_operations_total", "Total cache operations")
                .namespace("rwa")
                .subsystem("oracle"),
            &["operation", "status"]
        )?;
        
        let cache_memory_usage = Gauge::new(
            "oracle_cache_memory_usage_bytes",
            "Cache memory usage in bytes"
        )?;
        
        let cache_key_count = IntGauge::new(
            "oracle_cache_key_count",
            "Number of keys in cache"
        )?;
        
        let cache_hit_rate = Gauge::new(
            "oracle_cache_hit_rate",
            "Cache hit rate (0.0 to 1.0)"
        )?;
        
        // Business metrics
        let feeds_active = IntGauge::new(
            "oracle_feeds_active",
            "Number of active price feeds"
        )?;
        
        let subscriptions_active = IntGauge::new(
            "oracle_subscriptions_active",
            "Number of active subscriptions"
        )?;
        
        let price_deviations_total = IntCounter::new(
            "oracle_price_deviations_total",
            "Total price deviations detected"
        )?;
        
        let outliers_detected_total = IntCounter::new(
            "oracle_outliers_detected_total",
            "Total outliers detected"
        )?;
        
        // System metrics
        let memory_usage_bytes = Gauge::new(
            "oracle_memory_usage_bytes",
            "Memory usage in bytes"
        )?;
        
        let cpu_usage_percent = Gauge::new(
            "oracle_cpu_usage_percent",
            "CPU usage percentage"
        )?;
        
        let uptime_seconds = Counter::new(
            "oracle_uptime_seconds_total",
            "Service uptime in seconds"
        )?;
        
        // Register all metrics
        registry.register(Box::new(requests_total.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;
        registry.register(Box::new(active_requests.clone()))?;
        registry.register(Box::new(prices_fetched_total.clone()))?;
        registry.register(Box::new(price_fetch_duration.clone()))?;
        registry.register(Box::new(price_aggregation_duration.clone()))?;
        registry.register(Box::new(price_cache_hits.clone()))?;
        registry.register(Box::new(price_cache_misses.clone()))?;
        registry.register(Box::new(provider_requests_total.clone()))?;
        registry.register(Box::new(provider_errors_total.clone()))?;
        registry.register(Box::new(provider_response_time.clone()))?;
        registry.register(Box::new(provider_health_status.clone()))?;
        registry.register(Box::new(db_connections_active.clone()))?;
        registry.register(Box::new(db_connections_idle.clone()))?;
        registry.register(Box::new(db_query_duration.clone()))?;
        registry.register(Box::new(db_errors_total.clone()))?;
        registry.register(Box::new(cache_operations_total.clone()))?;
        registry.register(Box::new(cache_memory_usage.clone()))?;
        registry.register(Box::new(cache_key_count.clone()))?;
        registry.register(Box::new(cache_hit_rate.clone()))?;
        registry.register(Box::new(feeds_active.clone()))?;
        registry.register(Box::new(subscriptions_active.clone()))?;
        registry.register(Box::new(price_deviations_total.clone()))?;
        registry.register(Box::new(outliers_detected_total.clone()))?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;
        registry.register(Box::new(cpu_usage_percent.clone()))?;
        registry.register(Box::new(uptime_seconds.clone()))?;
        
        Ok(Self {
            registry,
            requests_total,
            request_duration,
            active_requests,
            prices_fetched_total,
            price_fetch_duration,
            price_aggregation_duration,
            price_cache_hits,
            price_cache_misses,
            provider_requests_total,
            provider_errors_total,
            provider_response_time,
            provider_health_status,
            db_connections_active,
            db_connections_idle,
            db_query_duration,
            db_errors_total,
            cache_operations_total,
            cache_memory_usage,
            cache_key_count,
            cache_hit_rate,
            feeds_active,
            subscriptions_active,
            price_deviations_total,
            outliers_detected_total,
            memory_usage_bytes,
            cpu_usage_percent,
            uptime_seconds,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Record HTTP request metrics
    pub fn record_request(&self, method: &str, endpoint: &str, status: u16, duration: Duration) {
        self.requests_total
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();
        
        self.request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }

    /// Record price fetch metrics
    pub fn record_price_fetch(&self, asset_id: &str, currency: &str, source: &str, duration: Duration, success: bool) {
        if success {
            self.prices_fetched_total
                .with_label_values(&[asset_id, currency, source])
                .inc();
        }
        
        self.price_fetch_duration
            .with_label_values(&[asset_id, source])
            .observe(duration.as_secs_f64());
    }

    /// Record price aggregation metrics
    pub fn record_price_aggregation(&self, asset_id: &str, method: &str, duration: Duration) {
        self.price_aggregation_duration
            .with_label_values(&[asset_id, method])
            .observe(duration.as_secs_f64());
    }

    /// Record cache hit/miss
    pub fn record_cache_hit(&self, hit: bool) {
        if hit {
            self.price_cache_hits.inc();
        } else {
            self.price_cache_misses.inc();
        }
    }

    /// Record provider metrics
    pub fn record_provider_request(&self, provider: &str, success: bool, duration: Duration) {
        let status = if success { "success" } else { "error" };
        
        self.provider_requests_total
            .with_label_values(&[provider, status])
            .inc();
        
        self.provider_response_time
            .with_label_values(&[provider])
            .observe(duration.as_secs_f64());
    }

    /// Update provider health status
    pub fn update_provider_health(&self, provider: &str, healthy: bool) {
        let status = if healthy { 1 } else { 0 };
        self.provider_health_status
            .with_label_values(&[provider])
            .set(status);
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families)
    }

    /// Get registry for custom metrics
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Start background metrics collection
    pub async fn start_background_collection(&self) {
        let metrics = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                metrics.collect_system_metrics().await;
            }
        });
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) {
        // Update uptime
        self.uptime_seconds.inc_by(30.0);

        // Collect memory usage (simplified - in production use proper system monitoring)
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(value_str) = line.split_whitespace().nth(1) {
                            if let Ok(value) = value_str.parse::<f64>() {
                                self.memory_usage_bytes.set(value * 1024.0); // Convert KB to bytes
                            }
                        }
                        break;
                    }
                }
            }
        }

        debug!("System metrics collected");
    }
}

impl Clone for OracleMetrics {
    fn clone(&self) -> Self {
        // Note: This creates a new metrics instance with the same registry
        // In practice, you'd want to share the same metrics instance
        Self::new().expect("Failed to create metrics clone")
    }
}

/// Request timing helper
pub struct RequestTimer {
    start: Instant,
    metrics: Arc<OracleMetrics>,
    method: String,
    endpoint: String,
}

impl RequestTimer {
    pub fn new(metrics: Arc<OracleMetrics>, method: String, endpoint: String) -> Self {
        metrics.active_requests.inc();
        
        Self {
            start: Instant::now(),
            metrics,
            method,
            endpoint,
        }
    }

    pub fn finish(self, status: u16) {
        let duration = self.start.elapsed();
        self.metrics.record_request(&self.method, &self.endpoint, status, duration);
        self.metrics.active_requests.dec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = OracleMetrics::new().unwrap();
        
        // Test that metrics are properly initialized
        assert_eq!(metrics.active_requests.get(), 0);
        assert_eq!(metrics.price_cache_hits.get(), 0);
        assert_eq!(metrics.price_cache_misses.get(), 0);
    }

    #[test]
    fn test_request_metrics() {
        let metrics = OracleMetrics::new().unwrap();
        
        // Record a request
        metrics.record_request("GET", "/api/v1/prices/BTC", 200, Duration::from_millis(100));
        
        // Verify metrics were recorded
        let metric_families = metrics.registry.gather();
        assert!(!metric_families.is_empty());
        
        // Find the requests_total metric
        let requests_metric = metric_families.iter()
            .find(|mf| mf.get_name() == "rwa_oracle_requests_total")
            .expect("requests_total metric not found");
        
        assert_eq!(requests_metric.get_metric().len(), 1);
        assert_eq!(requests_metric.get_metric()[0].get_counter().get_value(), 1.0);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = OracleMetrics::new().unwrap();
        
        // Record cache hits and misses
        metrics.record_cache_hit(true);
        metrics.record_cache_hit(true);
        metrics.record_cache_hit(false);
        
        assert_eq!(metrics.price_cache_hits.get(), 2);
        assert_eq!(metrics.price_cache_misses.get(), 1);
    }

    #[test]
    fn test_provider_metrics() {
        let metrics = OracleMetrics::new().unwrap();
        
        // Record provider requests
        metrics.record_provider_request("CoinGecko", true, Duration::from_millis(500));
        metrics.record_provider_request("CoinGecko", false, Duration::from_millis(1000));
        
        // Update provider health
        metrics.update_provider_health("CoinGecko", true);
        
        // Verify metrics
        let metric_families = metrics.registry.gather();
        let provider_requests = metric_families.iter()
            .find(|mf| mf.get_name() == "rwa_oracle_provider_requests_total")
            .expect("provider_requests_total metric not found");
        
        assert_eq!(provider_requests.get_metric().len(), 2); // success and error
    }

    #[test]
    fn test_price_fetch_metrics() {
        let metrics = OracleMetrics::new().unwrap();
        
        metrics.record_price_fetch("BTC", "USD", "CoinGecko", Duration::from_millis(200), true);
        metrics.record_price_fetch("ETH", "USD", "Binance", Duration::from_millis(150), true);
        
        let metric_families = metrics.registry.gather();
        let prices_fetched = metric_families.iter()
            .find(|mf| mf.get_name() == "rwa_oracle_prices_fetched_total")
            .expect("prices_fetched_total metric not found");
        
        assert_eq!(prices_fetched.get_metric().len(), 2); // BTC and ETH
    }

    #[test]
    fn test_metrics_export() {
        let metrics = OracleMetrics::new().unwrap();
        
        // Record some metrics
        metrics.record_request("GET", "/health", 200, Duration::from_millis(10));
        metrics.record_cache_hit(true);
        
        // Export metrics
        let exported = metrics.export_metrics().unwrap();
        
        assert!(!exported.is_empty());
        assert!(exported.contains("rwa_oracle_requests_total"));
        assert!(exported.contains("rwa_oracle_price_cache_hits_total"));
    }

    #[test]
    fn test_request_timer() {
        let metrics = Arc::new(OracleMetrics::new().unwrap());
        
        // Initial active requests should be 0
        assert_eq!(metrics.active_requests.get(), 0);
        
        // Create timer (should increment active requests)
        let timer = RequestTimer::new(
            metrics.clone(),
            "GET".to_string(),
            "/test".to_string()
        );
        
        assert_eq!(metrics.active_requests.get(), 1);
        
        // Finish timer (should decrement active requests and record metrics)
        timer.finish(200);
        
        assert_eq!(metrics.active_requests.get(), 0);
        
        // Verify request was recorded
        let metric_families = metrics.registry.gather();
        let requests_metric = metric_families.iter()
            .find(|mf| mf.get_name() == "rwa_oracle_requests_total")
            .expect("requests_total metric not found");
        
        assert_eq!(requests_metric.get_metric()[0].get_counter().get_value(), 1.0);
    }

    #[test]
    fn test_aggregation_metrics() {
        let metrics = OracleMetrics::new().unwrap();
        
        metrics.record_price_aggregation("BTC", "weighted_average", Duration::from_millis(50));
        metrics.record_price_aggregation("ETH", "median", Duration::from_millis(30));
        
        let metric_families = metrics.registry.gather();
        let aggregation_metric = metric_families.iter()
            .find(|mf| mf.get_name() == "rwa_oracle_price_aggregation_duration_seconds")
            .expect("price_aggregation_duration_seconds metric not found");
        
        assert_eq!(aggregation_metric.get_metric().len(), 2); // BTC and ETH
    }

    #[test]
    fn test_business_metrics() {
        let metrics = OracleMetrics::new().unwrap();
        
        // Update business metrics
        metrics.feeds_active.set(5);
        metrics.subscriptions_active.set(10);
        metrics.price_deviations_total.inc();
        metrics.outliers_detected_total.inc_by(3);
        
        assert_eq!(metrics.feeds_active.get(), 5);
        assert_eq!(metrics.subscriptions_active.get(), 10);
        assert_eq!(metrics.price_deviations_total.get(), 1);
        assert_eq!(metrics.outliers_detected_total.get(), 3);
    }
}
