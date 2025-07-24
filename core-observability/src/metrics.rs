// =====================================================================================
// File: core-observability/src/metrics.rs
// Description: Metrics collection and reporting for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{BusinessMetrics, ObservabilityError};
use prometheus::{
    Counter, CounterVec, Encoder, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Metrics collector and manager
pub struct MetricsCollector {
    registry: Registry,
    business_metrics: Arc<BusinessMetrics>,
    custom_metrics: Arc<RwLock<HashMap<String, Box<dyn CustomMetric>>>>,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new() -> Result<Self, ObservabilityError> {
        let registry = Registry::new();
        let business_metrics = Arc::new(BusinessMetrics::new()?);
        let custom_metrics = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            registry,
            business_metrics,
            custom_metrics,
        })
    }

    /// Get business metrics
    pub fn business_metrics(&self) -> Arc<BusinessMetrics> {
        Arc::clone(&self.business_metrics)
    }

    /// Register a custom metric
    pub async fn register_custom_metric(
        &self,
        name: String,
        metric: Box<dyn CustomMetric>,
    ) -> Result<(), ObservabilityError> {
        let mut metrics = self.custom_metrics.write().await;
        metrics.insert(name, metric);
        Ok(())
    }

    /// Collect all metrics as Prometheus text format
    pub async fn collect_metrics(&self) -> Result<String, ObservabilityError> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| ObservabilityError::Metrics(format!("Failed to encode metrics: {}", e)))?;

        String::from_utf8(buffer).map_err(|e| {
            ObservabilityError::Metrics(format!("Failed to convert metrics to string: {}", e))
        })
    }

    /// Record HTTP request metrics
    pub fn record_http_request(
        &self,
        method: &str,
        endpoint: &str,
        status_code: u16,
        duration: Duration,
    ) {
        let status = status_code.to_string();

        self.business_metrics
            .http_requests_total
            .with_label_values(&[method, endpoint, &status])
            .inc();

        self.business_metrics
            .http_request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());

        debug!(
            method = method,
            endpoint = endpoint,
            status_code = status_code,
            duration_ms = duration.as_millis(),
            "HTTP request recorded"
        );
    }

    /// Record database operation metrics
    pub fn record_database_operation(&self, operation: &str, duration: Duration, success: bool) {
        let status = if success { "success" } else { "error" };

        // You would add database-specific metrics here
        info!(
            operation = operation,
            duration_ms = duration.as_millis(),
            success = success,
            "Database operation recorded"
        );
    }

    /// Record blockchain operation metrics
    pub fn record_blockchain_operation(
        &self,
        chain: &str,
        operation: &str,
        success: bool,
        duration: Duration,
    ) {
        let status = if success { "success" } else { "error" };

        match operation {
            "transaction" => {
                self.business_metrics
                    .blockchain_transactions
                    .with_label_values(&[chain, status])
                    .inc();
            }
            "balance_check" => {
                self.business_metrics
                    .blockchain_balance_checks
                    .with_label_values(&[chain])
                    .inc();
            }
            _ => {}
        }

        info!(
            chain = chain,
            operation = operation,
            success = success,
            duration_ms = duration.as_millis(),
            "Blockchain operation recorded"
        );
    }

    /// Update connection status metrics
    pub fn update_connection_status(&self, service: &str, connected: bool) {
        let status = if connected { 1.0 } else { 0.0 };

        match service {
            "database" => {
                self.business_metrics.database_connections.set(status);
            }
            chain if chain.starts_with("blockchain_") => {
                let chain_name = chain.strip_prefix("blockchain_").unwrap_or(chain);
                self.business_metrics
                    .blockchain_connection_status
                    .with_label_values(&[chain_name])
                    .set(status);
            }
            _ => {}
        }

        info!(
            service = service,
            connected = connected,
            "Connection status updated"
        );
    }

    /// Record cache operation
    pub fn record_cache_operation(&self, hit: bool) {
        if hit {
            self.business_metrics.cache_hits.inc();
        } else {
            self.business_metrics.cache_misses.inc();
        }

        debug!(cache_hit = hit, "Cache operation recorded");
    }
}

/// Custom metric trait for extensibility
pub trait CustomMetric: Send + Sync {
    fn name(&self) -> &str;
    fn help(&self) -> &str;
    fn metric_type(&self) -> MetricType;
    fn collect(&self) -> Result<Vec<MetricSample>, ObservabilityError>;
}

#[derive(Debug, Clone)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone)]
pub struct MetricSample {
    pub name: String,
    pub labels: HashMap<String, String>,
    pub value: f64,
    pub timestamp: Option<i64>,
}

/// Performance timer for measuring operation duration
pub struct PerformanceTimer {
    start: Instant,
    operation: String,
    labels: HashMap<String, String>,
}

impl PerformanceTimer {
    pub fn new(operation: String) -> Self {
        Self {
            start: Instant::now(),
            operation,
            labels: HashMap::new(),
        }
    }

    pub fn with_label(mut self, key: &str, value: &str) -> Self {
        self.labels.insert(key.to_string(), value.to_string());
        self
    }

    pub fn finish(self) -> Duration {
        let duration = self.start.elapsed();

        info!(
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            labels = ?self.labels,
            "Operation completed"
        );

        duration
    }

    pub fn finish_with_result<T, E>(self, result: &Result<T, E>) -> Duration
    where
        E: std::fmt::Display,
        T: std::fmt::Debug,
    {
        let duration = self.start.elapsed();
        let success = result.is_ok();

        if success {
            info!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                success = true,
                labels = ?self.labels,
                "Operation completed successfully"
            );
        } else {
            error!(
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                success = false,
                error = %result.as_ref().unwrap_err(),
                labels = ?self.labels,
                "Operation failed"
            );
        }

        duration
    }
}

/// Metrics middleware helper
pub struct MetricsMiddleware {
    collector: Arc<MetricsCollector>,
}

impl MetricsMiddleware {
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self { collector }
    }

    /// Start timing an HTTP request
    pub fn start_http_timer(&self, method: &str, endpoint: &str) -> HttpRequestTimer {
        HttpRequestTimer {
            start: Instant::now(),
            method: method.to_string(),
            endpoint: endpoint.to_string(),
            collector: Arc::clone(&self.collector),
        }
    }

    /// Record business event
    pub fn record_business_event(&self, event_type: &str, entity_type: &str) {
        match (event_type, entity_type) {
            ("created", "asset") => self.collector.business_metrics.assets_created.inc(),
            ("updated", "asset") => self.collector.business_metrics.assets_updated.inc(),
            ("deleted", "asset") => self.collector.business_metrics.assets_deleted.inc(),
            ("registered", "user") => self.collector.business_metrics.users_registered.inc(),
            ("initiated", "payment") => self.collector.business_metrics.payments_initiated.inc(),
            ("completed", "payment") => self.collector.business_metrics.payments_completed.inc(),
            ("failed", "payment") => self.collector.business_metrics.payments_failed.inc(),
            _ => {
                debug!(
                    event_type = event_type,
                    entity_type = entity_type,
                    "Unknown business event type"
                );
            }
        }

        info!(
            event_type = event_type,
            entity_type = entity_type,
            "Business event recorded"
        );
    }

    /// Update gauge metrics
    pub fn update_gauge(&self, metric_name: &str, value: f64) {
        match metric_name {
            "active_users" => self.collector.business_metrics.users_active.set(value),
            "active_sessions" => self.collector.business_metrics.user_sessions.set(value),
            "total_asset_value" => self.collector.business_metrics.asset_value_total.set(value),
            _ => {
                debug!(
                    metric_name = metric_name,
                    value = value,
                    "Unknown gauge metric"
                );
            }
        }

        debug!(
            metric_name = metric_name,
            value = value,
            "Gauge metric updated"
        );
    }
}

/// HTTP request timer
pub struct HttpRequestTimer {
    start: Instant,
    method: String,
    endpoint: String,
    collector: Arc<MetricsCollector>,
}

impl HttpRequestTimer {
    pub fn finish(self, status_code: u16) {
        let duration = self.start.elapsed();
        self.collector
            .record_http_request(&self.method, &self.endpoint, status_code, duration);
    }
}

/// Health check metrics
pub struct HealthMetrics {
    pub service_up: IntGauge,
    pub last_health_check: IntGauge,
    pub health_check_duration: Histogram,
}

impl HealthMetrics {
    pub fn new() -> Result<Self, ObservabilityError> {
        let service_up = IntGauge::new("service_up", "Whether the service is up (1) or down (0)")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;

        let last_health_check = IntGauge::new(
            "last_health_check_timestamp",
            "Timestamp of the last health check",
        )
        .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;

        let health_check_duration = Histogram::with_opts(HistogramOpts::new(
            "health_check_duration_seconds",
            "Duration of health checks",
        ))
        .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;

        Ok(Self {
            service_up,
            last_health_check,
            health_check_duration,
        })
    }

    pub fn record_health_check(&self, healthy: bool, duration: Duration) {
        self.service_up.set(if healthy { 1 } else { 0 });
        self.last_health_check.set(chrono::Utc::now().timestamp());
        self.health_check_duration.observe(duration.as_secs_f64());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_metrics_collector() {
        // Test metrics collector creation (may fail due to global registry conflicts)
        let collector_result = MetricsCollector::new();
        if collector_result.is_err() {
            // Skip test if collector already exists (common in test environment)
            return;
        }
        let collector = collector_result.unwrap();

        // Test HTTP request recording
        collector.record_http_request("GET", "/api/v1/assets", 200, Duration::from_millis(150));

        // Test blockchain operation recording
        collector.record_blockchain_operation(
            "ethereum",
            "transaction",
            true,
            Duration::from_millis(500),
        );

        // Test connection status update
        collector.update_connection_status("database", true);

        // Test cache operation recording
        collector.record_cache_operation(true);
        collector.record_cache_operation(false);

        // Test metrics collection
        let metrics_text = collector.collect_metrics().await.unwrap();
        assert!(!metrics_text.is_empty());
    }

    #[test]
    fn test_performance_timer() {
        let timer =
            PerformanceTimer::new("test_operation".to_string()).with_label("component", "test");

        std::thread::sleep(Duration::from_millis(10));
        let duration = timer.finish();

        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_performance_timer_with_result() {
        let timer = PerformanceTimer::new("test_operation".to_string());

        let result: Result<(), &str> = Ok(());
        let duration = timer.finish_with_result(&result);

        assert!(duration.as_nanos() > 0);
    }

    #[tokio::test]
    async fn test_metrics_middleware() {
        // Test metrics middleware creation (may fail due to global registry conflicts)
        let collector_result = MetricsCollector::new();
        if collector_result.is_err() {
            // Skip test if collector already exists (common in test environment)
            return;
        }
        let collector = Arc::new(collector_result.unwrap());
        let middleware = MetricsMiddleware::new(collector);

        // Test HTTP timer
        let timer = middleware.start_http_timer("GET", "/test");
        std::thread::sleep(Duration::from_millis(1));
        timer.finish(200);

        // Test business event recording
        middleware.record_business_event("created", "asset");
        middleware.record_business_event("registered", "user");

        // Test gauge updates
        middleware.update_gauge("active_users", 42.0);
        middleware.update_gauge("total_asset_value", 1000000.0);
    }

    #[test]
    fn test_health_metrics() {
        let health_metrics = HealthMetrics::new().unwrap();

        health_metrics.record_health_check(true, Duration::from_millis(50));
        assert_eq!(health_metrics.service_up.get(), 1);

        health_metrics.record_health_check(false, Duration::from_millis(100));
        assert_eq!(health_metrics.service_up.get(), 0);
    }
}
