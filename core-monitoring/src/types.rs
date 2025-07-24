// =====================================================================================
// File: core-monitoring/src/types.rs
// Description: Core types for monitoring and alerting operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Monitoring service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_config: MetricsConfig,
    pub alert_config: AlertConfig,
    pub dashboard_config: DashboardConfig,
    pub anomaly_config: AnomalyConfig,
    pub performance_config: PerformanceConfig,
    pub health_check_config: HealthCheckConfig,
    pub log_config: LogConfig,
    pub tracing_config: TracingConfig,
    pub notification_config: NotificationConfig,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub collection_interval_seconds: u64,
    pub retention_days: u32,
    pub exporters: Vec<ExporterConfig>,
    pub custom_metrics: Vec<CustomMetricConfig>,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub rules: Vec<AlertRule>,
    pub channels: Vec<AlertChannel>,
    pub escalation_rules: Vec<EscalationRule>,
    pub suppression_rules: Vec<SuppressionRule>,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub refresh_interval_seconds: u32,
    pub default_time_range_hours: u32,
    pub max_widgets_per_dashboard: u32,
}

/// Anomaly detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    pub enabled: bool,
    pub models: Vec<AnomalyModelConfig>,
    pub sensitivity: f64,
    pub min_data_points: u32,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub latency_thresholds: HashMap<String, f64>,
    pub throughput_thresholds: HashMap<String, f64>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub check_interval_seconds: u64,
    pub timeout_seconds: u32,
    pub checks: Vec<HealthCheckDefinition>,
}

/// Log aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub enabled: bool,
    pub log_level: LogLevel,
    pub retention_days: u32,
    pub structured_logging: bool,
}

/// Distributed tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub jaeger_endpoint: Option<String>,
    pub zipkin_endpoint: Option<String>,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub channels: Vec<NotificationChannel>,
    pub rate_limits: HashMap<String, u32>,
}

/// Metric data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: Uuid,
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// Metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(HistogramValue),
    Summary(SummaryValue),
}

/// Histogram value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramValue {
    pub buckets: Vec<HistogramBucket>,
    pub count: u64,
    pub sum: f64,
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Summary value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryValue {
    pub quantiles: Vec<Quantile>,
    pub count: u64,
    pub sum: f64,
}

/// Quantile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    pub quantile: f64,
    pub value: f64,
}

/// Alert data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    pub generator_url: Option<String>,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub query: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub duration: chrono::Duration,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub enabled: bool,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Threshold {
        operator: ComparisonOperator,
        value: f64,
    },
    Range {
        min: f64,
        max: f64,
    },
    Anomaly {
        sensitivity: f64,
    },
    Custom {
        expression: String,
    },
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertStatus {
    Firing,
    Resolved,
    Suppressed,
    Acknowledged,
}

/// Alert channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannel {
    pub id: Uuid,
    pub name: String,
    pub channel_type: AlertChannelType,
    pub config: AlertChannelConfig,
    pub enabled: bool,
}

/// Alert channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertChannelType {
    Email,
    Slack,
    Discord,
    Webhook,
    SMS,
    PagerDuty,
    OpsGenie,
}

/// Alert channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannelConfig {
    Email {
        recipients: Vec<String>,
    },
    Slack {
        webhook_url: String,
        channel: String,
    },
    Discord {
        webhook_url: String,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
    SMS {
        phone_numbers: Vec<String>,
    },
    PagerDuty {
        integration_key: String,
    },
    OpsGenie {
        api_key: String,
        team: String,
    },
}

/// Dashboard data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub widgets: Vec<DashboardWidget>,
    pub layout: DashboardLayout,
    pub time_range: TimeRange,
    pub refresh_interval: chrono::Duration,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: Uuid,
    pub title: String,
    pub widget_type: WidgetType,
    pub query: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub config: WidgetConfig,
}

/// Widget types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    Gauge,
    SingleStat,
    Table,
    Heatmap,
    Text,
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetConfig {
    Chart {
        x_axis: String,
        y_axis: String,
        legend: bool,
    },
    Gauge {
        min: f64,
        max: f64,
        thresholds: Vec<GaugeThreshold>,
    },
    SingleStat {
        unit: String,
        decimals: u32,
    },
    Table {
        columns: Vec<String>,
        sortable: bool,
    },
    Text {
        content: String,
        markdown: bool,
    },
}

/// Gauge threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeThreshold {
    pub value: f64,
    pub color: String,
}

/// Dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub grid_size: u32,
    pub auto_arrange: bool,
}

/// Time range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Health check definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckDefinition {
    pub id: Uuid,
    pub name: String,
    pub check_type: HealthCheckType,
    pub target: String,
    pub timeout: chrono::Duration,
    pub interval: chrono::Duration,
    pub retries: u32,
}

/// Health check types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthCheckType {
    HTTP,
    TCP,
    Database,
    Custom,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub id: Uuid,
    pub definition_id: Uuid,
    pub status: HealthStatus,
    pub response_time: chrono::Duration,
    pub message: String,
    pub checked_at: DateTime<Utc>,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub labels: HashMap<String, String>,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Trace span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<chrono::Duration>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
}

/// Span log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    pub timestamp: DateTime<Utc>,
    pub fields: HashMap<String, String>,
}

/// Exporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterConfig {
    pub name: String,
    pub exporter_type: ExporterType,
    pub endpoint: String,
    pub credentials: Option<ExporterCredentials>,
    pub batch_size: u32,
    pub flush_interval_seconds: u64,
}

/// Exporter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExporterType {
    Prometheus,
    InfluxDB,
    Elasticsearch,
    Jaeger,
    Zipkin,
    Custom,
}

/// Exporter credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterCredentials {
    pub username: String,
    pub password: String,
    pub token: Option<String>,
}

/// Custom metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricConfig {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub labels: Vec<String>,
    pub collection_script: Option<String>,
}

/// Anomaly model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyModelConfig {
    pub name: String,
    pub model_type: AnomalyModelType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub training_data_days: u32,
}

/// Anomaly model types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnomalyModelType {
    Statistical,
    MachineLearning,
    Threshold,
    Seasonal,
}

/// Escalation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    pub id: Uuid,
    pub name: String,
    pub conditions: Vec<EscalationCondition>,
    pub actions: Vec<EscalationAction>,
    pub delay: chrono::Duration,
}

/// Escalation condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationCondition {
    Severity(AlertSeverity),
    Duration(chrono::Duration),
    UnacknowledgedFor(chrono::Duration),
    Label { key: String, value: String },
}

/// Escalation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationAction {
    NotifyChannel(Uuid),
    IncreaseSeverity,
    CreateTicket { system: String, priority: String },
    RunScript { script: String },
}

/// Suppression rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionRule {
    pub id: Uuid,
    pub name: String,
    pub conditions: Vec<SuppressionCondition>,
    pub duration: chrono::Duration,
    pub enabled: bool,
}

/// Suppression condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuppressionCondition {
    Label { key: String, value: String },
    TimeWindow { start: String, end: String },
    Maintenance { system: String },
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: Uuid,
    pub name: String,
    pub channel_type: NotificationChannelType,
    pub config: NotificationChannelConfig,
    pub enabled: bool,
}

/// Notification channel types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannelType {
    Email,
    SMS,
    Push,
    Webhook,
    Slack,
    Discord,
    Teams,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannelConfig {
    Email {
        smtp_server: String,
        recipients: Vec<String>,
    },
    SMS {
        provider: String,
        phone_numbers: Vec<String>,
    },
    Push {
        service: String,
        tokens: Vec<String>,
    },
    Webhook {
        url: String,
        headers: HashMap<String, String>,
    },
    Slack {
        webhook_url: String,
        channel: String,
    },
    Discord {
        webhook_url: String,
    },
    Teams {
        webhook_url: String,
    },
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_config: MetricsConfig {
                collection_interval_seconds: 60,
                retention_days: 30,
                exporters: Vec::new(),
                custom_metrics: Vec::new(),
            },
            alert_config: AlertConfig {
                enabled: true,
                rules: Vec::new(),
                channels: Vec::new(),
                escalation_rules: Vec::new(),
                suppression_rules: Vec::new(),
            },
            dashboard_config: DashboardConfig {
                enabled: true,
                refresh_interval_seconds: 30,
                default_time_range_hours: 24,
                max_widgets_per_dashboard: 20,
            },
            anomaly_config: AnomalyConfig {
                enabled: true,
                models: Vec::new(),
                sensitivity: 0.8,
                min_data_points: 100,
            },
            performance_config: PerformanceConfig {
                enabled: true,
                sampling_rate: 0.1,
                latency_thresholds: HashMap::new(),
                throughput_thresholds: HashMap::new(),
            },
            health_check_config: HealthCheckConfig {
                enabled: true,
                check_interval_seconds: 30,
                timeout_seconds: 10,
                checks: Vec::new(),
            },
            log_config: LogConfig {
                enabled: true,
                log_level: LogLevel::Info,
                retention_days: 7,
                structured_logging: true,
            },
            tracing_config: TracingConfig {
                enabled: true,
                sampling_rate: 0.1,
                jaeger_endpoint: None,
                zipkin_endpoint: None,
            },
            notification_config: NotificationConfig {
                enabled: true,
                channels: Vec::new(),
                rate_limits: HashMap::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        assert!(config.alert_config.enabled);
        assert!(config.dashboard_config.enabled);
        assert_eq!(config.metrics_config.collection_interval_seconds, 60);
        assert_eq!(config.health_check_config.check_interval_seconds, 30);
    }

    #[test]
    fn test_metric_creation() {
        let metric = Metric {
            id: Uuid::new_v4(),
            name: "test_metric".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Counter(100),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            source: "test".to_string(),
        };

        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert!(matches!(metric.value, MetricValue::Counter(100)));
    }

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Info < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::Critical);
        assert!(AlertSeverity::Critical < AlertSeverity::Emergency);
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Fatal);
    }
}
