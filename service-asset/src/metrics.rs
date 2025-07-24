// =====================================================================================
// File: service-asset/src/metrics.rs
// Description: Enterprise-grade metrics and monitoring for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec, Registry,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Metrics collector for the asset service
#[derive(Clone)]
pub struct AssetMetrics {
    registry: Arc<Registry>,
    
    // Request metrics
    pub http_requests_total: IntCounterVec,
    pub http_request_duration: HistogramVec,
    pub http_requests_in_flight: IntGaugeVec,
    
    // Asset operation metrics
    pub assets_created_total: IntCounter,
    pub assets_updated_total: IntCounter,
    pub assets_deleted_total: IntCounter,
    pub assets_tokenized_total: IntCounter,
    pub asset_valuations_total: IntCounter,
    
    // Database metrics
    pub db_connections_active: IntGauge,
    pub db_connections_idle: IntGauge,
    pub db_query_duration: HistogramVec,
    pub db_queries_total: IntCounterVec,
    
    // Blockchain metrics
    pub blockchain_requests_total: IntCounterVec,
    pub blockchain_request_duration: HistogramVec,
    pub blockchain_gas_used: CounterVec,
    pub blockchain_confirmations: HistogramVec,
    
    // Business metrics
    pub total_asset_value: GaugeVec,
    pub assets_by_type: IntGaugeVec,
    pub tokenized_assets_ratio: Gauge,
    
    // Error metrics
    pub errors_total: IntCounterVec,
    pub validation_errors_total: IntCounterVec,
    
    // Performance metrics
    pub cache_hits_total: IntCounter,
    pub cache_misses_total: IntCounter,
    pub rate_limit_exceeded_total: IntCounter,
    
    // Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, f64>>>,
}

impl AssetMetrics {
    /// Create new metrics instance
    pub fn new() -> Result<Self, MetricsError> {
        let registry = Arc::new(Registry::new());
        
        // HTTP request metrics
        let http_requests_total = IntCounterVec::new(
            prometheus::Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace("asset_service"),
            &["method", "endpoint", "status"],
        )?;
        
        let http_request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("http_request_duration_seconds", "HTTP request duration")
                .namespace("asset_service")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
            &["method", "endpoint"],
        )?;
        
        let http_requests_in_flight = IntGaugeVec::new(
            prometheus::Opts::new("http_requests_in_flight", "Current HTTP requests in flight")
                .namespace("asset_service"),
            &["method", "endpoint"],
        )?;
        
        // Asset operation metrics
        let assets_created_total = IntCounter::new(
            "asset_service_assets_created_total",
            "Total number of assets created",
        )?;
        
        let assets_updated_total = IntCounter::new(
            "asset_service_assets_updated_total",
            "Total number of assets updated",
        )?;
        
        let assets_deleted_total = IntCounter::new(
            "asset_service_assets_deleted_total",
            "Total number of assets deleted",
        )?;
        
        let assets_tokenized_total = IntCounter::new(
            "asset_service_assets_tokenized_total",
            "Total number of assets tokenized",
        )?;
        
        let asset_valuations_total = IntCounter::new(
            "asset_service_asset_valuations_total",
            "Total number of asset valuations",
        )?;
        
        // Database metrics
        let db_connections_active = IntGauge::new(
            "asset_service_db_connections_active",
            "Number of active database connections",
        )?;
        
        let db_connections_idle = IntGauge::new(
            "asset_service_db_connections_idle",
            "Number of idle database connections",
        )?;
        
        let db_query_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("db_query_duration_seconds", "Database query duration")
                .namespace("asset_service")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            &["query_type", "table"],
        )?;
        
        let db_queries_total = IntCounterVec::new(
            prometheus::Opts::new("db_queries_total", "Total database queries")
                .namespace("asset_service"),
            &["query_type", "table", "status"],
        )?;
        
        // Blockchain metrics
        let blockchain_requests_total = IntCounterVec::new(
            prometheus::Opts::new("blockchain_requests_total", "Total blockchain requests")
                .namespace("asset_service"),
            &["network", "method", "status"],
        )?;
        
        let blockchain_request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("blockchain_request_duration_seconds", "Blockchain request duration")
                .namespace("asset_service")
                .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]),
            &["network", "method"],
        )?;
        
        let blockchain_gas_used = CounterVec::new(
            prometheus::Opts::new("blockchain_gas_used_total", "Total gas used for blockchain transactions")
                .namespace("asset_service"),
            &["network", "contract"],
        )?;
        
        let blockchain_confirmations = HistogramVec::new(
            prometheus::HistogramOpts::new("blockchain_confirmations", "Number of confirmations for transactions")
                .namespace("asset_service")
                .buckets(vec![1.0, 3.0, 6.0, 12.0, 24.0, 50.0, 100.0]),
            &["network"],
        )?;
        
        // Business metrics
        let total_asset_value = GaugeVec::new(
            prometheus::Opts::new("total_asset_value", "Total value of assets")
                .namespace("asset_service"),
            &["currency", "asset_type"],
        )?;
        
        let assets_by_type = IntGaugeVec::new(
            prometheus::Opts::new("assets_by_type", "Number of assets by type")
                .namespace("asset_service"),
            &["asset_type", "tokenized"],
        )?;
        
        let tokenized_assets_ratio = Gauge::new(
            "asset_service_tokenized_assets_ratio",
            "Ratio of tokenized assets to total assets",
        )?;
        
        // Error metrics
        let errors_total = IntCounterVec::new(
            prometheus::Opts::new("errors_total", "Total number of errors")
                .namespace("asset_service"),
            &["error_type", "operation"],
        )?;
        
        let validation_errors_total = IntCounterVec::new(
            prometheus::Opts::new("validation_errors_total", "Total validation errors")
                .namespace("asset_service"),
            &["field", "error_type"],
        )?;
        
        // Performance metrics
        let cache_hits_total = IntCounter::new(
            "asset_service_cache_hits_total",
            "Total cache hits",
        )?;
        
        let cache_misses_total = IntCounter::new(
            "asset_service_cache_misses_total",
            "Total cache misses",
        )?;
        
        let rate_limit_exceeded_total = IntCounter::new(
            "asset_service_rate_limit_exceeded_total",
            "Total rate limit exceeded events",
        )?;
        
        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        registry.register(Box::new(assets_created_total.clone()))?;
        registry.register(Box::new(assets_updated_total.clone()))?;
        registry.register(Box::new(assets_deleted_total.clone()))?;
        registry.register(Box::new(assets_tokenized_total.clone()))?;
        registry.register(Box::new(asset_valuations_total.clone()))?;
        registry.register(Box::new(db_connections_active.clone()))?;
        registry.register(Box::new(db_connections_idle.clone()))?;
        registry.register(Box::new(db_query_duration.clone()))?;
        registry.register(Box::new(db_queries_total.clone()))?;
        registry.register(Box::new(blockchain_requests_total.clone()))?;
        registry.register(Box::new(blockchain_request_duration.clone()))?;
        registry.register(Box::new(blockchain_gas_used.clone()))?;
        registry.register(Box::new(blockchain_confirmations.clone()))?;
        registry.register(Box::new(total_asset_value.clone()))?;
        registry.register(Box::new(assets_by_type.clone()))?;
        registry.register(Box::new(tokenized_assets_ratio.clone()))?;
        registry.register(Box::new(errors_total.clone()))?;
        registry.register(Box::new(validation_errors_total.clone()))?;
        registry.register(Box::new(cache_hits_total.clone()))?;
        registry.register(Box::new(cache_misses_total.clone()))?;
        registry.register(Box::new(rate_limit_exceeded_total.clone()))?;
        
        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_requests_in_flight,
            assets_created_total,
            assets_updated_total,
            assets_deleted_total,
            assets_tokenized_total,
            asset_valuations_total,
            db_connections_active,
            db_connections_idle,
            db_query_duration,
            db_queries_total,
            blockchain_requests_total,
            blockchain_request_duration,
            blockchain_gas_used,
            blockchain_confirmations,
            total_asset_value,
            assets_by_type,
            tokenized_assets_ratio,
            errors_total,
            validation_errors_total,
            cache_hits_total,
            cache_misses_total,
            rate_limit_exceeded_total,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Record HTTP request
    pub fn record_http_request(&self, method: &str, endpoint: &str, status: u16, duration: Duration) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();
        
        self.http_request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }
    
    /// Record HTTP request start
    pub fn record_http_request_start(&self, method: &str, endpoint: &str) {
        self.http_requests_in_flight
            .with_label_values(&[method, endpoint])
            .inc();
    }
    
    /// Record HTTP request end
    pub fn record_http_request_end(&self, method: &str, endpoint: &str) {
        self.http_requests_in_flight
            .with_label_values(&[method, endpoint])
            .dec();
    }
    
    /// Record asset creation
    pub fn record_asset_created(&self) {
        self.assets_created_total.inc();
    }
    
    /// Record asset update
    pub fn record_asset_updated(&self) {
        self.assets_updated_total.inc();
    }
    
    /// Record asset deletion
    pub fn record_asset_deleted(&self) {
        self.assets_deleted_total.inc();
    }
    
    /// Record asset tokenization
    pub fn record_asset_tokenized(&self) {
        self.assets_tokenized_total.inc();
    }
    
    /// Record asset valuation
    pub fn record_asset_valuation(&self) {
        self.asset_valuations_total.inc();
    }
    
    /// Record database query
    pub fn record_db_query(&self, query_type: &str, table: &str, duration: Duration, success: bool) {
        let status = if success { "success" } else { "error" };
        
        self.db_queries_total
            .with_label_values(&[query_type, table, status])
            .inc();
        
        self.db_query_duration
            .with_label_values(&[query_type, table])
            .observe(duration.as_secs_f64());
    }
    
    /// Record blockchain request
    pub fn record_blockchain_request(&self, network: &str, method: &str, duration: Duration, success: bool) {
        let status = if success { "success" } else { "error" };
        
        self.blockchain_requests_total
            .with_label_values(&[network, method, status])
            .inc();
        
        self.blockchain_request_duration
            .with_label_values(&[network, method])
            .observe(duration.as_secs_f64());
    }
    
    /// Record gas usage
    pub fn record_gas_used(&self, network: &str, contract: &str, gas_used: u64) {
        self.blockchain_gas_used
            .with_label_values(&[network, contract])
            .inc_by(gas_used as f64);
    }
    
    /// Record error
    pub fn record_error(&self, error_type: &str, operation: &str) {
        self.errors_total
            .with_label_values(&[error_type, operation])
            .inc();
    }
    
    /// Record validation error
    pub fn record_validation_error(&self, field: &str, error_type: &str) {
        self.validation_errors_total
            .with_label_values(&[field, error_type])
            .inc();
    }
    
    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits_total.inc();
    }
    
    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses_total.inc();
    }
    
    /// Record rate limit exceeded
    pub fn record_rate_limit_exceeded(&self) {
        self.rate_limit_exceeded_total.inc();
    }
    
    /// Update database connection metrics
    pub fn update_db_connections(&self, active: i64, idle: i64) {
        self.db_connections_active.set(active);
        self.db_connections_idle.set(idle);
    }
    
    /// Update asset statistics
    pub fn update_asset_stats(&self, stats: &AssetStats) {
        // Update total asset value by currency and type
        for ((currency, asset_type), value) in &stats.total_value_by_currency_and_type {
            self.total_asset_value
                .with_label_values(&[currency, asset_type])
                .set(*value);
        }
        
        // Update assets by type
        for ((asset_type, tokenized), count) in &stats.assets_by_type_and_tokenization {
            let tokenized_str = if *tokenized { "true" } else { "false" };
            self.assets_by_type
                .with_label_values(&[asset_type, tokenized_str])
                .set(*count as i64);
        }
        
        // Update tokenization ratio
        if stats.total_assets > 0 {
            let ratio = stats.tokenized_assets as f64 / stats.total_assets as f64;
            self.tokenized_assets_ratio.set(ratio);
        }
    }
    
    /// Set custom metric
    pub async fn set_custom_metric(&self, name: String, value: f64) {
        let mut metrics = self.custom_metrics.write().await;
        metrics.insert(name, value);
    }
    
    /// Get custom metric
    pub async fn get_custom_metric(&self, name: &str) -> Option<f64> {
        let metrics = self.custom_metrics.read().await;
        metrics.get(name).copied()
    }
    
    /// Export Prometheus metrics
    pub fn export_prometheus_metrics(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families).unwrap_or_default()
    }
    
    /// Get metrics summary
    pub async fn get_summary(&self) -> MetricsSummary {
        let custom_metrics = self.custom_metrics.read().await.clone();
        
        MetricsSummary {
            assets_created: self.assets_created_total.get(),
            assets_updated: self.assets_updated_total.get(),
            assets_deleted: self.assets_deleted_total.get(),
            assets_tokenized: self.assets_tokenized_total.get(),
            asset_valuations: self.asset_valuations_total.get(),
            cache_hit_rate: self.calculate_cache_hit_rate(),
            error_rate: self.calculate_error_rate(),
            custom_metrics,
        }
    }
    
    fn calculate_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits_total.get() as f64;
        let misses = self.cache_misses_total.get() as f64;
        let total = hits + misses;
        
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
    
    fn calculate_error_rate(&self) -> f64 {
        // This would need to be calculated based on total requests vs errors
        // For now, return 0.0 as a placeholder
        0.0
    }
}

/// Asset statistics for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStats {
    pub total_assets: u64,
    pub tokenized_assets: u64,
    pub total_value_by_currency_and_type: HashMap<(String, String), f64>,
    pub assets_by_type_and_tokenization: HashMap<(String, bool), u64>,
}

/// Metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub assets_created: u64,
    pub assets_updated: u64,
    pub assets_deleted: u64,
    pub assets_tokenized: u64,
    pub asset_valuations: u64,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub custom_metrics: HashMap<String, f64>,
}

/// Metrics errors
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Prometheus error: {0}")]
    PrometheusError(#[from] prometheus::Error),
    
    #[error("Metrics collection error: {0}")]
    CollectionError(String),
}

/// Request timer for measuring durations
pub struct RequestTimer {
    start: Instant,
    method: String,
    endpoint: String,
    metrics: AssetMetrics,
}

impl RequestTimer {
    pub fn new(metrics: AssetMetrics, method: String, endpoint: String) -> Self {
        metrics.record_http_request_start(&method, &endpoint);
        
        Self {
            start: Instant::now(),
            method,
            endpoint,
            metrics,
        }
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.metrics.record_http_request_end(&self.method, &self.endpoint);
        // Note: We don't record the full request here as we don't have the status code
        // The status code should be recorded separately when the response is ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = AssetMetrics::new().unwrap();
        
        // Test basic operations
        metrics.record_asset_created();
        metrics.record_asset_updated();
        metrics.record_asset_deleted();
        
        assert_eq!(metrics.assets_created_total.get(), 1);
        assert_eq!(metrics.assets_updated_total.get(), 1);
        assert_eq!(metrics.assets_deleted_total.get(), 1);
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let metrics = AssetMetrics::new().unwrap();
        
        metrics.set_custom_metric("test_metric".to_string(), 42.0).await;
        let value = metrics.get_custom_metric("test_metric").await;
        
        assert_eq!(value, Some(42.0));
    }

    #[tokio::test]
    async fn test_cache_hit_rate() {
        let metrics = AssetMetrics::new().unwrap();
        
        // Record some cache hits and misses
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        let hit_rate = metrics.calculate_cache_hit_rate();
        assert!((hit_rate - 0.666).abs() < 0.01); // 2/3 ≈ 0.666
    }

    #[test]
    fn test_prometheus_export() {
        let metrics = AssetMetrics::new().unwrap();
        metrics.record_asset_created();
        
        let exported = metrics.export_prometheus_metrics();
        assert!(exported.contains("asset_service_assets_created_total"));
    }
}
