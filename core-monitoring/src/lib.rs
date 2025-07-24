// =====================================================================================
// File: core-monitoring/src/lib.rs
// Description: Real-time monitoring and alerting system for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Monitoring Module
//!
//! This module provides comprehensive real-time monitoring and alerting capabilities
//! for the StableRWA platform, including metrics collection, performance monitoring,
//! anomaly detection, and multi-channel alerting.

pub mod alerting;
pub mod anomaly_detection;
pub mod dashboards;
pub mod distributed_tracing;
pub mod error;
pub mod health_checks;
pub mod log_aggregation;
pub mod logging;
pub mod metrics;
pub mod notification;
pub mod performance;
pub mod scheduler;
pub mod service;
pub mod types;

// Re-export main types and traits
pub use alerting::{
    AlertEscalation, AlertManager, AlertManagerImpl, AlertNotificationDispatcher, AlertProcessor,
    AlertSuppression,
};
pub use anomaly_detection::{
    Anomaly, AnomalyDetector, AnomalyModel, MLDetector, StatisticalDetector, ThresholdDetector,
};
pub use dashboards::{
    ChartDataPoint, ChartSeries, ChartWidget, DashboardData, DashboardManager,
    DashboardManagerImpl, StatWidget, TableWidget, WidgetData,
};
pub use distributed_tracing::{
    JaegerExporter, SpanProcessor, TraceCollector, TraceExporter, TracingManager,
};
pub use error::{MonitoringError, MonitoringResult};
pub use health_checks::{
    CustomHealthChecker, DatabaseHealthChecker, HTTPHealthChecker, HealthCheckService,
    HealthCheckServiceImpl, SystemHealthStatus, TCPHealthChecker,
};
pub use logging::{
    LogAggregator, LogAggregatorImpl, LogIndexer, LogParser, LogSourceStats, LogStatistics,
    ParsedLogData,
};
pub use metrics::{
    ApplicationMetricsCollector, CustomMetric, InfluxDBExporter, MetricsCollector, MetricsExporter,
    PrometheusExporter, SystemMetricsCollector,
};
pub use notification::{
    EmailNotifier, NotificationManager, NotificationManagerImpl, PushNotifier, SMSNotifier,
    SlackNotifier,
};
pub use performance::{
    LatencyTracker, PerformanceMetrics, PerformanceMonitor, ResourceTracker, ThroughputTracker,
};
pub use scheduler::{
    CronScheduler, IntervalScheduler, OneTimeScheduler, ScheduledTask, SchedulerConfig,
    TaskSchedule, TaskScheduler, TaskType,
};
pub use service::{
    AggregationType, LogQuery, MetricQuery, MonitoringHealthStatus, MonitoringService,
    MonitoringServiceImpl, TraceQuery,
};
pub use types::{
    Alert, AlertChannel, AlertRule, AlertSeverity, Dashboard, HealthCheck, LogEntry, Metric,
    MetricType, TraceSpan,
};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main Monitoring service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringServiceConfig {
    /// Core monitoring configuration
    pub monitoring_config: types::MonitoringConfig,
    /// Global monitoring settings
    pub global_settings: GlobalMonitoringSettings,
}

impl Default for MonitoringServiceConfig {
    fn default() -> Self {
        Self {
            monitoring_config: types::MonitoringConfig::default(),
            global_settings: GlobalMonitoringSettings::default(),
        }
    }
}

/// Global monitoring settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMonitoringSettings {
    /// Enable real-time monitoring
    pub enable_real_time_monitoring: bool,
    /// Metrics collection interval in seconds
    pub metrics_collection_interval: u64,
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    /// Enable distributed tracing
    pub enable_distributed_tracing: bool,
    /// Data retention period in days
    pub data_retention_days: u32,
    /// Enable high availability mode
    pub enable_high_availability: bool,
    /// Alert escalation timeout in minutes
    pub alert_escalation_timeout_minutes: u32,
    /// Enable alert suppression
    pub enable_alert_suppression: bool,
    /// Maximum concurrent alerts
    pub max_concurrent_alerts: u32,
    /// Enable performance profiling
    pub enable_performance_profiling: bool,
}

impl Default for GlobalMonitoringSettings {
    fn default() -> Self {
        Self {
            enable_real_time_monitoring: true,
            metrics_collection_interval: 60,
            enable_anomaly_detection: true,
            enable_distributed_tracing: true,
            data_retention_days: 90,
            enable_high_availability: true,
            alert_escalation_timeout_minutes: 15,
            enable_alert_suppression: true,
            max_concurrent_alerts: 1000,
            enable_performance_profiling: true,
        }
    }
}

/// Monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    pub total_metrics_collected: u64,
    pub active_alerts: u64,
    pub resolved_alerts_24h: u64,
    pub false_positive_alerts_24h: u64,
    pub anomalies_detected_24h: u64,
    pub average_alert_response_time_minutes: f64,
    pub system_uptime_percentage: Decimal,
    pub data_ingestion_rate_per_second: f64,
    pub storage_utilization_percentage: Decimal,
    pub alert_channel_breakdown: HashMap<String, u64>,
    pub service_health_status: HashMap<String, String>,
    pub last_updated: DateTime<Utc>,
}



// Re-export types from modules for convenience
pub use crate::types::*;




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringServiceConfig::default();
        assert_eq!(config.metrics_config.collection_interval_seconds, 60);
        assert!(config.alert_config.enable_email_alerts);
        assert!(config.dashboard_config.enable_real_time_updates);
        assert!(config.anomaly_config.enable_ml_detection);
    }

    #[test]
    fn test_global_monitoring_settings() {
        let settings = GlobalMonitoringSettings::default();
        assert!(settings.enable_real_time_monitoring);
        assert_eq!(settings.metrics_collection_interval, 60);
        assert!(settings.enable_anomaly_detection);
        assert!(settings.enable_distributed_tracing);
        assert_eq!(settings.data_retention_days, 90);
        assert!(settings.enable_high_availability);
    }
}
