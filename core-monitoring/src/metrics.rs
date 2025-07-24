// =====================================================================================
// File: core-monitoring/src/metrics.rs
// Description: Metrics collection and export module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{MonitoringError, MonitoringResult},
    types::{ExporterConfig, ExporterType, Metric, MetricType, MetricValue, MetricsConfig},
};

/// Metrics collector trait
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Collect system metrics
    async fn collect_system_metrics(&self) -> MonitoringResult<Vec<Metric>>;

    /// Collect application metrics
    async fn collect_application_metrics(&self) -> MonitoringResult<Vec<Metric>>;

    /// Collect custom metrics
    async fn collect_custom_metrics(&self) -> MonitoringResult<Vec<Metric>>;

    /// Start metrics collection
    async fn start_collection(&self) -> MonitoringResult<()>;

    /// Stop metrics collection
    async fn stop_collection(&self) -> MonitoringResult<()>;
}

/// Metrics exporter trait
#[async_trait]
pub trait MetricsExporter: Send + Sync {
    /// Export metrics to external system
    async fn export_metrics(&self, metrics: &[Metric]) -> MonitoringResult<()>;

    /// Get exporter health status
    async fn health_check(&self) -> MonitoringResult<bool>;
}

/// Custom metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub labels: HashMap<String, String>,
    pub value: MetricValue,
}

/// System metrics collector
pub struct SystemMetricsCollector {
    config: MetricsConfig,
}

/// Application metrics collector
pub struct ApplicationMetricsCollector {
    config: MetricsConfig,
    custom_metrics: HashMap<String, CustomMetric>,
}

/// Prometheus exporter
pub struct PrometheusExporter {
    config: ExporterConfig,
    endpoint: String,
}

/// InfluxDB exporter
pub struct InfluxDBExporter {
    config: ExporterConfig,
    endpoint: String,
    database: String,
}

impl SystemMetricsCollector {
    pub fn new(config: MetricsConfig) -> Self {
        Self { config }
    }

    fn collect_cpu_metrics(&self) -> Vec<Metric> {
        vec![Metric {
            id: Uuid::new_v4(),
            name: "system_cpu_usage_percent".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(15.2),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            source: "system".to_string(),
        }]
    }

    fn collect_memory_metrics(&self) -> Vec<Metric> {
        let mut labels = HashMap::new();
        labels.insert("type".to_string(), "used".to_string());

        vec![Metric {
            id: Uuid::new_v4(),
            name: "system_memory_bytes".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(1024.0 * 1024.0 * 512.0), // 512 MB
            labels,
            timestamp: Utc::now(),
            source: "system".to_string(),
        }]
    }

    fn collect_disk_metrics(&self) -> Vec<Metric> {
        let mut labels = HashMap::new();
        labels.insert("device".to_string(), "/dev/sda1".to_string());
        labels.insert("mountpoint".to_string(), "/".to_string());

        vec![Metric {
            id: Uuid::new_v4(),
            name: "system_disk_usage_percent".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(45.8),
            labels,
            timestamp: Utc::now(),
            source: "system".to_string(),
        }]
    }

    fn collect_network_metrics(&self) -> Vec<Metric> {
        let mut labels = HashMap::new();
        labels.insert("interface".to_string(), "eth0".to_string());

        vec![
            Metric {
                id: Uuid::new_v4(),
                name: "system_network_bytes_received_total".to_string(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(1024 * 1024 * 100), // 100 MB
                labels: labels.clone(),
                timestamp: Utc::now(),
                source: "system".to_string(),
            },
            Metric {
                id: Uuid::new_v4(),
                name: "system_network_bytes_transmitted_total".to_string(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(1024 * 1024 * 50), // 50 MB
                labels,
                timestamp: Utc::now(),
                source: "system".to_string(),
            },
        ]
    }
}

#[async_trait]
impl MetricsCollector for SystemMetricsCollector {
    async fn collect_system_metrics(&self) -> MonitoringResult<Vec<Metric>> {
        let mut metrics = Vec::new();

        metrics.extend(self.collect_cpu_metrics());
        metrics.extend(self.collect_memory_metrics());
        metrics.extend(self.collect_disk_metrics());
        metrics.extend(self.collect_network_metrics());

        Ok(metrics)
    }

    async fn collect_application_metrics(&self) -> MonitoringResult<Vec<Metric>> {
        // System collector doesn't collect application metrics
        Ok(Vec::new())
    }

    async fn collect_custom_metrics(&self) -> MonitoringResult<Vec<Metric>> {
        // System collector doesn't collect custom metrics
        Ok(Vec::new())
    }

    async fn start_collection(&self) -> MonitoringResult<()> {
        // Mock implementation - in reality, this would start background collection
        Ok(())
    }

    async fn stop_collection(&self) -> MonitoringResult<()> {
        // Mock implementation - in reality, this would stop background collection
        Ok(())
    }
}

impl ApplicationMetricsCollector {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            custom_metrics: HashMap::new(),
        }
    }

    pub fn register_custom_metric(&mut self, metric: CustomMetric) {
        self.custom_metrics.insert(metric.name.clone(), metric);
    }

    fn collect_http_metrics(&self) -> Vec<Metric> {
        let mut labels = HashMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("status".to_string(), "200".to_string());

        vec![
            Metric {
                id: Uuid::new_v4(),
                name: "http_requests_total".to_string(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(1000),
                labels: labels.clone(),
                timestamp: Utc::now(),
                source: "application".to_string(),
            },
            Metric {
                id: Uuid::new_v4(),
                name: "http_request_duration_seconds".to_string(),
                metric_type: MetricType::Histogram,
                value: MetricValue::Histogram(crate::types::HistogramValue {
                    buckets: vec![
                        crate::types::HistogramBucket {
                            upper_bound: 0.1,
                            count: 100,
                        },
                        crate::types::HistogramBucket {
                            upper_bound: 0.5,
                            count: 200,
                        },
                        crate::types::HistogramBucket {
                            upper_bound: 1.0,
                            count: 300,
                        },
                    ],
                    count: 300,
                    sum: 150.0,
                }),
                labels,
                timestamp: Utc::now(),
                source: "application".to_string(),
            },
        ]
    }

    fn collect_database_metrics(&self) -> Vec<Metric> {
        let mut labels = HashMap::new();
        labels.insert("database".to_string(), "postgres".to_string());

        vec![
            Metric {
                id: Uuid::new_v4(),
                name: "database_connections_active".to_string(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(25.0),
                labels: labels.clone(),
                timestamp: Utc::now(),
                source: "application".to_string(),
            },
            Metric {
                id: Uuid::new_v4(),
                name: "database_query_duration_seconds".to_string(),
                metric_type: MetricType::Summary,
                value: MetricValue::Summary(crate::types::SummaryValue {
                    quantiles: vec![
                        crate::types::Quantile {
                            quantile: 0.5,
                            value: 0.01,
                        },
                        crate::types::Quantile {
                            quantile: 0.9,
                            value: 0.05,
                        },
                        crate::types::Quantile {
                            quantile: 0.99,
                            value: 0.1,
                        },
                    ],
                    count: 1000,
                    sum: 50.0,
                }),
                labels,
                timestamp: Utc::now(),
                source: "application".to_string(),
            },
        ]
    }
}

#[async_trait]
impl MetricsCollector for ApplicationMetricsCollector {
    async fn collect_system_metrics(&self) -> MonitoringResult<Vec<Metric>> {
        // Application collector doesn't collect system metrics
        Ok(Vec::new())
    }

    async fn collect_application_metrics(&self) -> MonitoringResult<Vec<Metric>> {
        let mut metrics = Vec::new();

        metrics.extend(self.collect_http_metrics());
        metrics.extend(self.collect_database_metrics());

        Ok(metrics)
    }

    async fn collect_custom_metrics(&self) -> MonitoringResult<Vec<Metric>> {
        let metrics: Vec<Metric> = self
            .custom_metrics
            .values()
            .map(|custom_metric| Metric {
                id: Uuid::new_v4(),
                name: custom_metric.name.clone(),
                metric_type: custom_metric.metric_type,
                value: custom_metric.value.clone(),
                labels: custom_metric.labels.clone(),
                timestamp: Utc::now(),
                source: "custom".to_string(),
            })
            .collect();

        Ok(metrics)
    }

    async fn start_collection(&self) -> MonitoringResult<()> {
        // Mock implementation - in reality, this would start background collection
        Ok(())
    }

    async fn stop_collection(&self) -> MonitoringResult<()> {
        // Mock implementation - in reality, this would stop background collection
        Ok(())
    }
}

impl PrometheusExporter {
    pub fn new(config: ExporterConfig) -> Self {
        Self {
            endpoint: config.endpoint.clone(),
            config,
        }
    }

    fn format_metric_for_prometheus(&self, metric: &Metric) -> String {
        let labels_str = if metric.labels.is_empty() {
            String::new()
        } else {
            let labels: Vec<String> = metric
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            format!("{{{}}}", labels.join(","))
        };

        let value_str = match &metric.value {
            MetricValue::Counter(v) => v.to_string(),
            MetricValue::Gauge(v) => v.to_string(),
            MetricValue::Histogram(h) => h.sum.to_string(), // Simplified
            MetricValue::Summary(s) => s.sum.to_string(),   // Simplified
        };

        format!(
            "{}{} {} {}",
            metric.name,
            labels_str,
            value_str,
            metric.timestamp.timestamp_millis()
        )
    }
}

#[async_trait]
impl MetricsExporter for PrometheusExporter {
    async fn export_metrics(&self, metrics: &[Metric]) -> MonitoringResult<()> {
        // Mock Prometheus export
        let _prometheus_format: Vec<String> = metrics
            .iter()
            .map(|metric| self.format_metric_for_prometheus(metric))
            .collect();

        // In reality, this would send the formatted metrics to Prometheus
        Ok(())
    }

    async fn health_check(&self) -> MonitoringResult<bool> {
        // Mock health check - in reality, this would ping the Prometheus endpoint
        Ok(true)
    }
}

impl InfluxDBExporter {
    pub fn new(config: ExporterConfig, database: String) -> Self {
        Self {
            endpoint: config.endpoint.clone(),
            database,
            config,
        }
    }

    fn format_metric_for_influxdb(&self, metric: &Metric) -> String {
        let tags_str = if metric.labels.is_empty() {
            String::new()
        } else {
            let tags: Vec<String> = metric
                .labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!(",{}", tags.join(","))
        };

        let value_str = match &metric.value {
            MetricValue::Counter(v) => format!("value={}i", v),
            MetricValue::Gauge(v) => format!("value={}", v),
            MetricValue::Histogram(h) => format!("sum={},count={}i", h.sum, h.count),
            MetricValue::Summary(s) => format!("sum={},count={}i", s.sum, s.count),
        };

        format!(
            "{}{} {} {}",
            metric.name,
            tags_str,
            value_str,
            metric.timestamp.timestamp_nanos()
        )
    }
}

#[async_trait]
impl MetricsExporter for InfluxDBExporter {
    async fn export_metrics(&self, metrics: &[Metric]) -> MonitoringResult<()> {
        // Mock InfluxDB export
        let _influxdb_format: Vec<String> = metrics
            .iter()
            .map(|metric| self.format_metric_for_influxdb(metric))
            .collect();

        // In reality, this would send the formatted metrics to InfluxDB
        Ok(())
    }

    async fn health_check(&self) -> MonitoringResult<bool> {
        // Mock health check - in reality, this would ping the InfluxDB endpoint
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_metrics_collector() {
        let config = MetricsConfig {
            collection_interval_seconds: 60,
            retention_days: 30,
            exporters: Vec::new(),
            custom_metrics: Vec::new(),
        };

        let collector = SystemMetricsCollector::new(config);
        let metrics = collector.collect_system_metrics().await.unwrap();

        assert!(!metrics.is_empty());
        assert!(metrics.iter().any(|m| m.name.contains("cpu")));
        assert!(metrics.iter().any(|m| m.name.contains("memory")));
        assert!(metrics.iter().any(|m| m.name.contains("disk")));
        assert!(metrics.iter().any(|m| m.name.contains("network")));
    }

    #[tokio::test]
    async fn test_application_metrics_collector() {
        let config = MetricsConfig {
            collection_interval_seconds: 60,
            retention_days: 30,
            exporters: Vec::new(),
            custom_metrics: Vec::new(),
        };

        let collector = ApplicationMetricsCollector::new(config);
        let metrics = collector.collect_application_metrics().await.unwrap();

        assert!(!metrics.is_empty());
        assert!(metrics.iter().any(|m| m.name.contains("http")));
        assert!(metrics.iter().any(|m| m.name.contains("database")));
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let config = MetricsConfig {
            collection_interval_seconds: 60,
            retention_days: 30,
            exporters: Vec::new(),
            custom_metrics: Vec::new(),
        };

        let mut collector = ApplicationMetricsCollector::new(config);

        let custom_metric = CustomMetric {
            name: "custom_business_metric".to_string(),
            metric_type: MetricType::Gauge,
            description: "A custom business metric".to_string(),
            labels: HashMap::new(),
            value: MetricValue::Gauge(42.0),
        };

        collector.register_custom_metric(custom_metric);

        let metrics = collector.collect_custom_metrics().await.unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "custom_business_metric");
    }

    #[tokio::test]
    async fn test_prometheus_exporter() {
        let config = ExporterConfig {
            name: "prometheus".to_string(),
            exporter_type: ExporterType::Prometheus,
            endpoint: "http://localhost:9090".to_string(),
            credentials: None,
            batch_size: 100,
            flush_interval_seconds: 60,
        };

        let exporter = PrometheusExporter::new(config);

        let metric = Metric {
            id: Uuid::new_v4(),
            name: "test_metric".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(100),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            source: "test".to_string(),
        };

        let result = exporter.export_metrics(&[metric]).await;
        assert!(result.is_ok());

        let health = exporter.health_check().await.unwrap();
        assert!(health);
    }

    #[test]
    fn test_prometheus_formatting() {
        let config = ExporterConfig {
            name: "prometheus".to_string(),
            exporter_type: ExporterType::Prometheus,
            endpoint: "http://localhost:9090".to_string(),
            credentials: None,
            batch_size: 100,
            flush_interval_seconds: 60,
        };

        let exporter = PrometheusExporter::new(config);

        let mut labels = HashMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("status".to_string(), "200".to_string());

        let metric = Metric {
            id: Uuid::new_v4(),
            name: "http_requests_total".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(100),
            labels,
            timestamp: Utc::now(),
            source: "test".to_string(),
        };

        let formatted = exporter.format_metric_for_prometheus(&metric);
        assert!(formatted.contains("http_requests_total"));
        assert!(formatted.contains("method=\"GET\""));
        assert!(formatted.contains("status=\"200\""));
        assert!(formatted.contains("100"));
    }
}
