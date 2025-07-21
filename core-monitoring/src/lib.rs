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

pub mod error;
pub mod types;
pub mod metrics;
pub mod alerting;
pub mod dashboards;
pub mod anomaly_detection;
pub mod performance;
pub mod health_checks;
pub mod log_aggregation;
pub mod distributed_tracing;
pub mod notification;
pub mod scheduler;
pub mod service;

// Re-export main types and traits
pub use error::{MonitoringError, MonitoringResult};
pub use types::{
    Metric, Alert, Dashboard, HealthCheck, LogEntry, TraceSpan,
    AlertRule, AlertChannel, MetricType, AlertSeverity
};
pub use metrics::{
    MetricsCollector, MetricsConfig, MetricsExporter,
    PrometheusExporter, InfluxDBExporter, CustomMetric
};
pub use alerting::{
    AlertManager, AlertConfig, AlertRule as AlertRuleType,
    AlertProcessor, AlertEscalation, AlertSuppression
};
pub use dashboards::{
    DashboardManager, DashboardConfig, DashboardWidget,
    ChartWidget, TableWidget, StatWidget
};
pub use anomaly_detection::{
    AnomalyDetector, AnomalyConfig, AnomalyModel,
    StatisticalDetector, MLDetector, ThresholdDetector
};
pub use performance::{
    PerformanceMonitor, PerformanceConfig, PerformanceMetrics,
    LatencyTracker, ThroughputTracker, ResourceTracker
};
pub use health_checks::{
    HealthChecker, HealthConfig, HealthStatus,
    ServiceHealthCheck, DatabaseHealthCheck, ExternalServiceHealthCheck
};
pub use log_aggregation::{
    LogAggregator, LogConfig, LogProcessor,
    LogParser, LogFilter, LogForwarder
};
pub use distributed_tracing::{
    TracingManager, TracingConfig, TraceCollector,
    SpanProcessor, TraceExporter, JaegerExporter
};
pub use notification::{
    NotificationManager, NotificationConfig, NotificationChannel,
    EmailNotifier, SlackNotifier, SMSNotifier, PushNotifier
};
pub use scheduler::{
    TaskScheduler, SchedulerConfig, ScheduledTask,
    CronScheduler, IntervalScheduler, OneTimeScheduler
};
pub use service::MonitoringService;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main Monitoring service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringServiceConfig {
    /// Metrics configuration
    pub metrics_config: metrics::MetricsConfig,
    /// Alerting configuration
    pub alert_config: alerting::AlertConfig,
    /// Dashboard configuration
    pub dashboard_config: dashboards::DashboardConfig,
    /// Anomaly detection configuration
    pub anomaly_config: anomaly_detection::AnomalyConfig,
    /// Performance monitoring configuration
    pub performance_config: performance::PerformanceConfig,
    /// Health check configuration
    pub health_config: health_checks::HealthConfig,
    /// Log aggregation configuration
    pub log_config: log_aggregation::LogConfig,
    /// Distributed tracing configuration
    pub tracing_config: distributed_tracing::TracingConfig,
    /// Notification configuration
    pub notification_config: notification::NotificationConfig,
    /// Global monitoring settings
    pub global_settings: GlobalMonitoringSettings,
}

impl Default for MonitoringServiceConfig {
    fn default() -> Self {
        Self {
            metrics_config: metrics::MetricsConfig::default(),
            alert_config: alerting::AlertConfig::default(),
            dashboard_config: dashboards::DashboardConfig::default(),
            anomaly_config: anomaly_detection::AnomalyConfig::default(),
            performance_config: performance::PerformanceConfig::default(),
            health_config: health_checks::HealthConfig::default(),
            log_config: log_aggregation::LogConfig::default(),
            tracing_config: distributed_tracing::TracingConfig::default(),
            notification_config: notification::NotificationConfig::default(),
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

/// Monitoring health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHealthStatus {
    pub overall_status: String,
    pub metrics_collector_status: String,
    pub alert_manager_status: String,
    pub dashboard_status: String,
    pub anomaly_detector_status: String,
    pub performance_monitor_status: String,
    pub health_checker_status: String,
    pub log_aggregator_status: String,
    pub tracing_manager_status: String,
    pub notification_manager_status: String,
    pub last_check: DateTime<Utc>,
}

// Stub modules for compilation
pub mod metrics {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MetricsConfig {
        pub collection_interval_seconds: u64,
        pub retention_days: u32,
        pub enable_prometheus: bool,
        pub enable_influxdb: bool,
        pub custom_metrics_enabled: bool,
    }
    
    impl Default for MetricsConfig {
        fn default() -> Self {
            Self {
                collection_interval_seconds: 60,
                retention_days: 90,
                enable_prometheus: true,
                enable_influxdb: true,
                custom_metrics_enabled: true,
            }
        }
    }
    
    pub struct MetricsCollector;
    pub struct MetricsExporter;
    pub struct PrometheusExporter;
    pub struct InfluxDBExporter;
    pub struct CustomMetric;
}

pub mod alerting {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AlertConfig {
        pub enable_email_alerts: bool,
        pub enable_slack_alerts: bool,
        pub enable_sms_alerts: bool,
        pub alert_suppression_window_minutes: u32,
        pub max_alerts_per_minute: u32,
    }
    
    impl Default for AlertConfig {
        fn default() -> Self {
            Self {
                enable_email_alerts: true,
                enable_slack_alerts: true,
                enable_sms_alerts: false,
                alert_suppression_window_minutes: 5,
                max_alerts_per_minute: 10,
            }
        }
    }
    
    pub struct AlertManager;
    pub struct AlertRule;
    pub struct AlertProcessor;
    pub struct AlertEscalation;
    pub struct AlertSuppression;
}

pub mod dashboards {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DashboardConfig {
        pub enable_real_time_updates: bool,
        pub update_interval_seconds: u64,
        pub max_data_points: u32,
        pub enable_custom_dashboards: bool,
    }
    
    impl Default for DashboardConfig {
        fn default() -> Self {
            Self {
                enable_real_time_updates: true,
                update_interval_seconds: 30,
                max_data_points: 1000,
                enable_custom_dashboards: true,
            }
        }
    }
    
    pub struct DashboardManager;
    pub struct DashboardWidget;
    pub struct ChartWidget;
    pub struct TableWidget;
    pub struct StatWidget;
}

pub mod anomaly_detection {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AnomalyConfig {
        pub detection_algorithm: String,
        pub sensitivity_level: f64,
        pub training_window_days: u32,
        pub enable_ml_detection: bool,
    }
    
    impl Default for AnomalyConfig {
        fn default() -> Self {
            Self {
                detection_algorithm: "isolation_forest".to_string(),
                sensitivity_level: 0.8,
                training_window_days: 30,
                enable_ml_detection: true,
            }
        }
    }
    
    pub struct AnomalyDetector;
    pub struct AnomalyModel;
    pub struct StatisticalDetector;
    pub struct MLDetector;
    pub struct ThresholdDetector;
}

pub mod performance {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PerformanceConfig {
        pub enable_latency_tracking: bool,
        pub enable_throughput_tracking: bool,
        pub enable_resource_tracking: bool,
        pub sampling_rate: f64,
    }
    
    impl Default for PerformanceConfig {
        fn default() -> Self {
            Self {
                enable_latency_tracking: true,
                enable_throughput_tracking: true,
                enable_resource_tracking: true,
                sampling_rate: 0.1, // 10% sampling
            }
        }
    }
    
    pub struct PerformanceMonitor;
    pub struct PerformanceMetrics;
    pub struct LatencyTracker;
    pub struct ThroughputTracker;
    pub struct ResourceTracker;
}

pub mod health_checks {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HealthConfig {
        pub check_interval_seconds: u64,
        pub timeout_seconds: u32,
        pub enable_deep_health_checks: bool,
        pub enable_dependency_checks: bool,
    }
    
    impl Default for HealthConfig {
        fn default() -> Self {
            Self {
                check_interval_seconds: 30,
                timeout_seconds: 10,
                enable_deep_health_checks: true,
                enable_dependency_checks: true,
            }
        }
    }
    
    pub struct HealthChecker;
    pub struct HealthStatus;
    pub struct ServiceHealthCheck;
    pub struct DatabaseHealthCheck;
    pub struct ExternalServiceHealthCheck;
}

pub mod log_aggregation {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LogConfig {
        pub log_level: String,
        pub enable_structured_logging: bool,
        pub retention_days: u32,
        pub max_log_size_mb: u32,
    }
    
    impl Default for LogConfig {
        fn default() -> Self {
            Self {
                log_level: "info".to_string(),
                enable_structured_logging: true,
                retention_days: 30,
                max_log_size_mb: 100,
            }
        }
    }
    
    pub struct LogAggregator;
    pub struct LogProcessor;
    pub struct LogParser;
    pub struct LogFilter;
    pub struct LogForwarder;
}

pub mod distributed_tracing {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TracingConfig {
        pub enable_jaeger: bool,
        pub sampling_rate: f64,
        pub max_span_attributes: u32,
        pub trace_timeout_seconds: u64,
    }
    
    impl Default for TracingConfig {
        fn default() -> Self {
            Self {
                enable_jaeger: true,
                sampling_rate: 0.1, // 10% sampling
                max_span_attributes: 100,
                trace_timeout_seconds: 300,
            }
        }
    }
    
    pub struct TracingManager;
    pub struct TraceCollector;
    pub struct SpanProcessor;
    pub struct TraceExporter;
    pub struct JaegerExporter;
}

pub mod notification {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NotificationConfig {
        pub email_enabled: bool,
        pub slack_enabled: bool,
        pub sms_enabled: bool,
        pub push_enabled: bool,
        pub rate_limit_per_minute: u32,
    }
    
    impl Default for NotificationConfig {
        fn default() -> Self {
            Self {
                email_enabled: true,
                slack_enabled: true,
                sms_enabled: false,
                push_enabled: false,
                rate_limit_per_minute: 60,
            }
        }
    }
    
    pub struct NotificationManager;
    pub struct NotificationChannel;
    pub struct EmailNotifier;
    pub struct SlackNotifier;
    pub struct SMSNotifier;
    pub struct PushNotifier;
}

pub mod scheduler {
    use super::*;
    
    pub struct TaskScheduler;
    pub struct SchedulerConfig;
    pub struct ScheduledTask;
    pub struct CronScheduler;
    pub struct IntervalScheduler;
    pub struct OneTimeScheduler;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringServiceConfig::default();
        assert_eq!(config.metrics_config.collection_interval_seconds, 60);
        assert!(config.metrics_config.enable_prometheus);
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

    #[test]
    fn test_metrics_config() {
        let config = metrics::MetricsConfig::default();
        assert_eq!(config.collection_interval_seconds, 60);
        assert_eq!(config.retention_days, 90);
        assert!(config.enable_prometheus);
        assert!(config.enable_influxdb);
        assert!(config.custom_metrics_enabled);
    }

    #[test]
    fn test_alert_config() {
        let config = alerting::AlertConfig::default();
        assert!(config.enable_email_alerts);
        assert!(config.enable_slack_alerts);
        assert!(!config.enable_sms_alerts);
        assert_eq!(config.alert_suppression_window_minutes, 5);
        assert_eq!(config.max_alerts_per_minute, 10);
    }
}
