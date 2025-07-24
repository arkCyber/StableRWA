// =====================================================================================
// RWA Tokenization Platform - Custody Service Metrics
// 
// Comprehensive metrics collection and monitoring for custody service operations
// including performance metrics, business metrics, and health indicators.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec, Registry,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Metrics collector for custody service
#[derive(Debug)]
pub struct MetricsCollector {
    /// Prometheus registry
    registry: Registry,
    /// HTTP request metrics
    http_metrics: HttpMetrics,
    /// Business metrics
    business_metrics: BusinessMetrics,
    /// System metrics
    system_metrics: SystemMetrics,
    /// Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, CustomMetric>>>,
}

/// HTTP request and response metrics
#[derive(Debug)]
pub struct HttpMetrics {
    /// Total HTTP requests
    pub requests_total: IntCounterVec,
    /// HTTP request duration
    pub request_duration: HistogramVec,
    /// HTTP response size
    pub response_size: HistogramVec,
    /// Active HTTP connections
    pub active_connections: IntGauge,
    /// HTTP errors by type
    pub errors_total: IntCounterVec,
}

/// Business-specific metrics
#[derive(Debug)]
pub struct BusinessMetrics {
    /// Total accounts created
    pub accounts_total: IntCounter,
    /// Active accounts
    pub accounts_active: IntGauge,
    /// Total assets under custody
    pub assets_total: IntCounter,
    /// Total value under custody (USD)
    pub custody_value_usd: Gauge,
    /// Assets by type
    pub assets_by_type: IntGaugeVec,
    /// Insurance policies active
    pub insurance_policies_active: IntGauge,
    /// Proofs generated
    pub proofs_generated_total: IntCounter,
    /// Multi-sig transactions
    pub multisig_transactions_total: IntCounterVec,
}

/// System performance metrics
#[derive(Debug)]
pub struct SystemMetrics {
    /// Memory usage
    pub memory_usage_bytes: IntGauge,
    /// CPU usage percentage
    pub cpu_usage_percent: Gauge,
    /// Database connections
    pub database_connections: IntGaugeVec,
    /// Cache hit rate
    pub cache_hit_rate: Gauge,
    /// Service uptime
    pub uptime_seconds: IntGauge,
    /// Background tasks
    pub background_tasks: IntGaugeVec,
}

/// Custom metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: MetricValue,
    /// Metric labels
    pub labels: HashMap<String, String>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Metric description
    pub description: String,
}

/// Types of metrics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric (monotonically increasing)
    Counter,
    /// Gauge metric (can increase or decrease)
    Gauge,
    /// Histogram metric (distribution of values)
    Histogram,
    /// Summary metric (quantiles)
    Summary,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Decimal value
    Decimal(Decimal),
    /// Boolean value
    Boolean(bool),
}

/// Performance measurement helper
#[derive(Debug)]
pub struct PerformanceTimer {
    /// Timer start time
    start_time: Instant,
    /// Timer name
    name: String,
    /// Associated labels
    labels: HashMap<String, String>,
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// HTTP metrics summary
    pub http_summary: HttpMetricsSummary,
    /// Business metrics summary
    pub business_summary: BusinessMetricsSummary,
    /// System metrics summary
    pub system_summary: SystemMetricsSummary,
    /// Custom metrics
    pub custom_metrics: Vec<CustomMetric>,
}

/// HTTP metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMetricsSummary {
    /// Total requests processed
    pub total_requests: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Active connections
    pub active_connections: i64,
}

/// Business metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetricsSummary {
    /// Total accounts
    pub total_accounts: u64,
    /// Active accounts
    pub active_accounts: i64,
    /// Total assets
    pub total_assets: u64,
    /// Total custody value (USD)
    pub total_custody_value_usd: Decimal,
    /// Assets by type breakdown
    pub assets_by_type: HashMap<String, i64>,
    /// Active insurance policies
    pub active_insurance_policies: i64,
    /// Total proofs generated
    pub total_proofs: u64,
}

/// System metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetricsSummary {
    /// Memory usage (bytes)
    pub memory_usage_bytes: i64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Database connection pool status
    pub database_connections: HashMap<String, i64>,
    /// Cache hit rate percentage
    pub cache_hit_rate_percent: f64,
    /// Service uptime (seconds)
    pub uptime_seconds: i64,
    /// Background task status
    pub background_tasks: HashMap<String, i64>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    /// 
    /// # Returns
    /// 
    /// Returns a new metrics collector instance
    pub fn new() -> Self {
        let registry = Registry::new();
        
        let http_metrics = HttpMetrics::new(&registry);
        let business_metrics = BusinessMetrics::new(&registry);
        let system_metrics = SystemMetrics::new(&registry);
        let custom_metrics = Arc::new(RwLock::new(HashMap::new()));

        Self {
            registry,
            http_metrics,
            business_metrics,
            system_metrics,
            custom_metrics,
        }
    }

    /// Record HTTP request
    /// 
    /// # Arguments
    /// 
    /// * `method` - HTTP method
    /// * `path` - Request path
    /// * `status_code` - Response status code
    /// * `duration` - Request duration
    /// * `response_size` - Response size in bytes
    pub fn record_http_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: Duration,
        response_size: usize,
    ) {
        let labels = &[method, path, &status_code.to_string()];
        
        self.http_metrics.requests_total.with_label_values(labels).inc();
        self.http_metrics
            .request_duration
            .with_label_values(&[method, path])
            .observe(duration.as_secs_f64());
        self.http_metrics
            .response_size
            .with_label_values(&[method, path])
            .observe(response_size as f64);

        // Record errors for non-2xx status codes
        if status_code >= 400 {
            let error_type = match status_code {
                400..=499 => "client_error",
                500..=599 => "server_error",
                _ => "unknown_error",
            };
            self.http_metrics
                .errors_total
                .with_label_values(&[error_type, &status_code.to_string()])
                .inc();
        }
    }

    /// Record account creation
    pub fn record_account_created(&self) {
        self.business_metrics.accounts_total.inc();
        self.business_metrics.accounts_active.inc();
    }

    /// Record asset custody
    /// 
    /// # Arguments
    /// 
    /// * `asset_type` - Type of asset
    /// * `value_usd` - Asset value in USD
    pub fn record_asset_custody(&self, asset_type: &str, value_usd: Option<Decimal>) {
        self.business_metrics.assets_total.inc();
        self.business_metrics
            .assets_by_type
            .with_label_values(&[asset_type])
            .inc();

        if let Some(value) = value_usd {
            self.business_metrics
                .custody_value_usd
                .add(value.to_f64().unwrap_or(0.0));
        }
    }

    /// Record proof generation
    pub fn record_proof_generated(&self) {
        self.business_metrics.proofs_generated_total.inc();
    }

    /// Record multi-signature transaction
    /// 
    /// # Arguments
    /// 
    /// * `transaction_type` - Type of transaction
    /// * `status` - Transaction status
    pub fn record_multisig_transaction(&self, transaction_type: &str, status: &str) {
        self.business_metrics
            .multisig_transactions_total
            .with_label_values(&[transaction_type, status])
            .inc();
    }

    /// Update system metrics
    /// 
    /// # Arguments
    /// 
    /// * `memory_usage` - Memory usage in bytes
    /// * `cpu_usage` - CPU usage percentage
    /// * `uptime` - Service uptime in seconds
    pub fn update_system_metrics(&self, memory_usage: i64, cpu_usage: f64, uptime: i64) {
        self.system_metrics.memory_usage_bytes.set(memory_usage);
        self.system_metrics.cpu_usage_percent.set(cpu_usage);
        self.system_metrics.uptime_seconds.set(uptime);
    }

    /// Add custom metric
    /// 
    /// # Arguments
    /// 
    /// * `metric` - Custom metric to add
    pub async fn add_custom_metric(&self, metric: CustomMetric) {
        let mut metrics = self.custom_metrics.write().await;
        metrics.insert(metric.name.clone(), metric);
    }

    /// Start performance timer
    /// 
    /// # Arguments
    /// 
    /// * `name` - Timer name
    /// * `labels` - Associated labels
    /// 
    /// # Returns
    /// 
    /// Returns a performance timer
    pub fn start_timer(&self, name: &str, labels: HashMap<String, String>) -> PerformanceTimer {
        PerformanceTimer::new(name.to_string(), labels)
    }

    /// Get metrics snapshot
    /// 
    /// # Returns
    /// 
    /// Returns current metrics snapshot
    pub async fn get_snapshot(&self) -> MetricsSnapshot {
        let custom_metrics = self.custom_metrics.read().await;
        
        MetricsSnapshot {
            timestamp: Utc::now(),
            http_summary: self.get_http_summary(),
            business_summary: self.get_business_summary(),
            system_summary: self.get_system_summary(),
            custom_metrics: custom_metrics.values().cloned().collect(),
        }
    }

    /// Get HTTP metrics summary
    fn get_http_summary(&self) -> HttpMetricsSummary {
        // This is a simplified implementation
        // In a real implementation, this would aggregate actual metric values
        HttpMetricsSummary {
            total_requests: 0,
            avg_response_time_ms: 0.0,
            error_rate_percent: 0.0,
            requests_per_second: 0.0,
            active_connections: self.http_metrics.active_connections.get(),
        }
    }

    /// Get business metrics summary
    fn get_business_summary(&self) -> BusinessMetricsSummary {
        BusinessMetricsSummary {
            total_accounts: self.business_metrics.accounts_total.get() as u64,
            active_accounts: self.business_metrics.accounts_active.get(),
            total_assets: self.business_metrics.assets_total.get() as u64,
            total_custody_value_usd: Decimal::new(
                self.business_metrics.custody_value_usd.get() as i64,
                2,
            ),
            assets_by_type: HashMap::new(), // Simplified
            active_insurance_policies: self.business_metrics.insurance_policies_active.get(),
            total_proofs: self.business_metrics.proofs_generated_total.get() as u64,
        }
    }

    /// Get system metrics summary
    fn get_system_summary(&self) -> SystemMetricsSummary {
        SystemMetricsSummary {
            memory_usage_bytes: self.system_metrics.memory_usage_bytes.get(),
            cpu_usage_percent: self.system_metrics.cpu_usage_percent.get(),
            database_connections: HashMap::new(), // Simplified
            cache_hit_rate_percent: self.system_metrics.cache_hit_rate.get(),
            uptime_seconds: self.system_metrics.uptime_seconds.get(),
            background_tasks: HashMap::new(), // Simplified
        }
    }

    /// Get Prometheus registry
    /// 
    /// # Returns
    /// 
    /// Returns the Prometheus registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl HttpMetrics {
    /// Create new HTTP metrics
    fn new(registry: &Registry) -> Self {
        let requests_total = IntCounterVec::new(
            prometheus::Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "path", "status"],
        ).unwrap();

        let request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new("http_request_duration_seconds", "HTTP request duration"),
            &["method", "path"],
        ).unwrap();

        let response_size = HistogramVec::new(
            prometheus::HistogramOpts::new("http_response_size_bytes", "HTTP response size"),
            &["method", "path"],
        ).unwrap();

        let active_connections = IntGauge::new(
            "http_active_connections", "Active HTTP connections"
        ).unwrap();

        let errors_total = IntCounterVec::new(
            prometheus::Opts::new("http_errors_total", "Total HTTP errors"),
            &["error_type", "status"],
        ).unwrap();

        registry.register(Box::new(requests_total.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(response_size.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();
        registry.register(Box::new(errors_total.clone())).unwrap();

        Self {
            requests_total,
            request_duration,
            response_size,
            active_connections,
            errors_total,
        }
    }
}

impl BusinessMetrics {
    /// Create new business metrics
    fn new(registry: &Registry) -> Self {
        let accounts_total = IntCounter::new(
            "custody_accounts_total", "Total custody accounts created"
        ).unwrap();

        let accounts_active = IntGauge::new(
            "custody_accounts_active", "Active custody accounts"
        ).unwrap();

        let assets_total = IntCounter::new(
            "custody_assets_total", "Total assets under custody"
        ).unwrap();

        let custody_value_usd = Gauge::new(
            "custody_value_usd_total", "Total value under custody in USD"
        ).unwrap();

        let assets_by_type = IntGaugeVec::new(
            prometheus::Opts::new("custody_assets_by_type", "Assets by type"),
            &["asset_type"],
        ).unwrap();

        let insurance_policies_active = IntGauge::new(
            "custody_insurance_policies_active", "Active insurance policies"
        ).unwrap();

        let proofs_generated_total = IntCounter::new(
            "custody_proofs_generated_total", "Total proofs generated"
        ).unwrap();

        let multisig_transactions_total = IntCounterVec::new(
            prometheus::Opts::new("custody_multisig_transactions_total", "Multi-signature transactions"),
            &["transaction_type", "status"],
        ).unwrap();

        registry.register(Box::new(accounts_total.clone())).unwrap();
        registry.register(Box::new(accounts_active.clone())).unwrap();
        registry.register(Box::new(assets_total.clone())).unwrap();
        registry.register(Box::new(custody_value_usd.clone())).unwrap();
        registry.register(Box::new(assets_by_type.clone())).unwrap();
        registry.register(Box::new(insurance_policies_active.clone())).unwrap();
        registry.register(Box::new(proofs_generated_total.clone())).unwrap();
        registry.register(Box::new(multisig_transactions_total.clone())).unwrap();

        Self {
            accounts_total,
            accounts_active,
            assets_total,
            custody_value_usd,
            assets_by_type,
            insurance_policies_active,
            proofs_generated_total,
            multisig_transactions_total,
        }
    }
}

impl SystemMetrics {
    /// Create new system metrics
    fn new(registry: &Registry) -> Self {
        let memory_usage_bytes = IntGauge::new(
            "system_memory_usage_bytes", "System memory usage in bytes"
        ).unwrap();

        let cpu_usage_percent = Gauge::new(
            "system_cpu_usage_percent", "System CPU usage percentage"
        ).unwrap();

        let database_connections = IntGaugeVec::new(
            prometheus::Opts::new("database_connections", "Database connections"),
            &["pool", "status"],
        ).unwrap();

        let cache_hit_rate = Gauge::new(
            "cache_hit_rate", "Cache hit rate"
        ).unwrap();

        let uptime_seconds = IntGauge::new(
            "service_uptime_seconds", "Service uptime in seconds"
        ).unwrap();

        let background_tasks = IntGaugeVec::new(
            prometheus::Opts::new("background_tasks", "Background tasks"),
            &["task_type", "status"],
        ).unwrap();

        registry.register(Box::new(memory_usage_bytes.clone())).unwrap();
        registry.register(Box::new(cpu_usage_percent.clone())).unwrap();
        registry.register(Box::new(database_connections.clone())).unwrap();
        registry.register(Box::new(cache_hit_rate.clone())).unwrap();
        registry.register(Box::new(uptime_seconds.clone())).unwrap();
        registry.register(Box::new(background_tasks.clone())).unwrap();

        Self {
            memory_usage_bytes,
            cpu_usage_percent,
            database_connections,
            cache_hit_rate,
            uptime_seconds,
            background_tasks,
        }
    }
}

impl PerformanceTimer {
    /// Create a new performance timer
    /// 
    /// # Arguments
    /// 
    /// * `name` - Timer name
    /// * `labels` - Associated labels
    /// 
    /// # Returns
    /// 
    /// Returns a new performance timer
    pub fn new(name: String, labels: HashMap<String, String>) -> Self {
        Self {
            start_time: Instant::now(),
            name,
            labels,
        }
    }

    /// Stop the timer and get elapsed duration
    /// 
    /// # Returns
    /// 
    /// Returns elapsed duration
    pub fn stop(self) -> Duration {
        let duration = self.start_time.elapsed();
        println!(
            "Timer '{}' completed in {:?} with labels: {:?}",
            self.name, duration, self.labels
        );
        duration
    }
}

impl CustomMetric {
    /// Create a new custom metric
    /// 
    /// # Arguments
    /// 
    /// * `name` - Metric name
    /// * `metric_type` - Metric type
    /// * `value` - Metric value
    /// * `description` - Metric description
    /// 
    /// # Returns
    /// 
    /// Returns a new custom metric
    pub fn new(
        name: String,
        metric_type: MetricType,
        value: MetricValue,
        description: String,
    ) -> Self {
        Self {
            name,
            metric_type,
            value,
            labels: HashMap::new(),
            updated_at: Utc::now(),
            description,
        }
    }

    /// Add label to metric
    /// 
    /// # Arguments
    /// 
    /// * `key` - Label key
    /// * `value` - Label value
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::time::Duration;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.registry().gather().is_empty());
    }

    #[test]
    fn test_http_request_recording() {
        let collector = MetricsCollector::new();
        
        collector.record_http_request(
            "GET",
            "/api/accounts",
            200,
            Duration::from_millis(150),
            1024,
        );

        // Verify metrics were recorded (simplified check)
        let metric_families = collector.registry().gather();
        assert!(!metric_families.is_empty());
    }

    #[test]
    fn test_business_metrics_recording() {
        let collector = MetricsCollector::new();
        
        collector.record_account_created();
        collector.record_asset_custody("Digital", Some(dec!(1000)));
        collector.record_proof_generated();
        collector.record_multisig_transaction("deposit", "completed");

        // Verify metrics were recorded
        let metric_families = collector.registry().gather();
        assert!(!metric_families.is_empty());
    }

    #[test]
    fn test_system_metrics_update() {
        let collector = MetricsCollector::new();
        
        collector.update_system_metrics(1024 * 1024 * 512, 45.5, 3600);

        // Verify metrics were updated
        assert_eq!(collector.system_metrics.memory_usage_bytes.get(), 1024 * 1024 * 512);
        assert_eq!(collector.system_metrics.cpu_usage_percent.get(), 45.5);
        assert_eq!(collector.system_metrics.uptime_seconds.get(), 3600);
    }

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::new(
            "test_operation".to_string(),
            HashMap::new(),
        );

        std::thread::sleep(Duration::from_millis(10));
        let duration = timer.stop();
        
        assert!(duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_custom_metric_creation() {
        let metric = CustomMetric::new(
            "test_metric".to_string(),
            MetricType::Counter,
            MetricValue::Integer(42),
            "Test metric description".to_string(),
        ).with_label("environment".to_string(), "test".to_string());

        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert!(matches!(metric.value, MetricValue::Integer(42)));
        assert_eq!(metric.labels.get("environment"), Some(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_custom_metric_storage() {
        let collector = MetricsCollector::new();
        
        let metric = CustomMetric::new(
            "custom_counter".to_string(),
            MetricType::Counter,
            MetricValue::Integer(100),
            "Custom counter metric".to_string(),
        );

        collector.add_custom_metric(metric).await;

        let snapshot = collector.get_snapshot().await;
        assert_eq!(snapshot.custom_metrics.len(), 1);
        assert_eq!(snapshot.custom_metrics[0].name, "custom_counter");
    }

    #[test]
    fn test_metric_type_enum() {
        assert_eq!(MetricType::Counter, MetricType::Counter);
        assert_ne!(MetricType::Counter, MetricType::Gauge);
    }

    #[test]
    fn test_metric_value_variants() {
        let int_value = MetricValue::Integer(42);
        let float_value = MetricValue::Float(3.14);
        let decimal_value = MetricValue::Decimal(dec!(123.45));
        let bool_value = MetricValue::Boolean(true);

        match int_value {
            MetricValue::Integer(42) => assert!(true),
            _ => assert!(false),
        }

        match float_value {
            MetricValue::Float(f) => assert!((f - 3.14).abs() < f64::EPSILON),
            _ => assert!(false),
        }

        match decimal_value {
            MetricValue::Decimal(d) => assert_eq!(d, dec!(123.45)),
            _ => assert!(false),
        }

        match bool_value {
            MetricValue::Boolean(true) => assert!(true),
            _ => assert!(false),
        }
    }

    #[tokio::test]
    async fn test_metrics_snapshot() {
        let collector = MetricsCollector::new();
        
        // Record some metrics
        collector.record_account_created();
        collector.record_asset_custody("Digital", Some(dec!(5000)));
        collector.update_system_metrics(1024 * 1024, 25.0, 1800);

        let snapshot = collector.get_snapshot().await;
        
        assert!(snapshot.timestamp <= Utc::now());
        assert_eq!(snapshot.business_summary.total_accounts, 1);
        assert_eq!(snapshot.business_summary.active_accounts, 1);
        assert_eq!(snapshot.system_summary.memory_usage_bytes, 1024 * 1024);
        assert_eq!(snapshot.system_summary.cpu_usage_percent, 25.0);
    }

    #[test]
    fn test_http_metrics_summary_creation() {
        let summary = HttpMetricsSummary {
            total_requests: 1000,
            avg_response_time_ms: 150.5,
            error_rate_percent: 2.5,
            requests_per_second: 10.0,
            active_connections: 25,
        };

        assert_eq!(summary.total_requests, 1000);
        assert_eq!(summary.avg_response_time_ms, 150.5);
        assert_eq!(summary.error_rate_percent, 2.5);
        assert_eq!(summary.requests_per_second, 10.0);
        assert_eq!(summary.active_connections, 25);
    }

    #[test]
    fn test_business_metrics_summary_creation() {
        let mut assets_by_type = HashMap::new();
        assets_by_type.insert("Digital".to_string(), 100);
        assets_by_type.insert("Physical".to_string(), 50);

        let summary = BusinessMetricsSummary {
            total_accounts: 500,
            active_accounts: 450,
            total_assets: 150,
            total_custody_value_usd: dec!(10000000),
            assets_by_type,
            active_insurance_policies: 75,
            total_proofs: 1000,
        };

        assert_eq!(summary.total_accounts, 500);
        assert_eq!(summary.active_accounts, 450);
        assert_eq!(summary.total_assets, 150);
        assert_eq!(summary.total_custody_value_usd, dec!(10000000));
        assert_eq!(summary.assets_by_type.len(), 2);
        assert_eq!(summary.active_insurance_policies, 75);
        assert_eq!(summary.total_proofs, 1000);
    }
}
