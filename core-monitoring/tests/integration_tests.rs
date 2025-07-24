// =====================================================================================
// File: core-monitoring/tests/integration_tests.rs
// Description: Integration tests for monitoring service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use core_monitoring::{
    AlertCondition, AlertManager, AlertManagerImpl, AlertRule, AlertSeverity,
    ApplicationMetricsCollector, DashboardConfig, DashboardManager, DashboardManagerImpl,
    HealthCheckConfig, HealthCheckService, HealthCheckServiceImpl, LogAggregator,
    LogAggregatorImpl, LogConfig, LogEntry, LogLevel, Metric, MetricType, MetricValue,
    MetricsCollector, MonitoringConfig, MonitoringService, MonitoringServiceImpl,
    SystemMetricsCollector, TimeRange,
};

/// Test configuration for monitoring service
struct MonitoringTestConfig {
    pub monitoring_config: MonitoringConfig,
    pub dashboard_config: DashboardConfig,
    pub health_check_config: HealthCheckConfig,
    pub log_config: LogConfig,
}

impl Default for MonitoringTestConfig {
    fn default() -> Self {
        Self {
            monitoring_config: MonitoringConfig {
                enabled: true,
                collection_interval_seconds: 10,
                retention_days: 7,
                max_metrics_per_batch: 1000,
                alert_evaluation_interval_seconds: 30,
            },
            dashboard_config: DashboardConfig {
                enabled: true,
                refresh_interval_seconds: 30,
                default_time_range_hours: 24,
                max_widgets_per_dashboard: 20,
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
        }
    }
}

/// Test helper for creating monitoring service
async fn create_monitoring_service() -> MonitoringServiceImpl {
    let config = MonitoringTestConfig::default();
    let service = MonitoringServiceImpl::new(config.monitoring_config);
    service
        .initialize()
        .await
        .expect("Failed to initialize monitoring service");
    service
}

/// Test helper for creating metrics
fn create_test_metrics() -> Vec<Metric> {
    vec![
        Metric {
            id: Uuid::new_v4(),
            name: "cpu_usage_percent".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Float(45.5),
            labels: {
                let mut labels = HashMap::new();
                labels.insert("host".to_string(), "server-01".to_string());
                labels.insert("service".to_string(), "api-gateway".to_string());
                labels
            },
            timestamp: Utc::now(),
        },
        Metric {
            id: Uuid::new_v4(),
            name: "memory_usage_bytes".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Integer(1073741824), // 1GB
            labels: {
                let mut labels = HashMap::new();
                labels.insert("host".to_string(), "server-01".to_string());
                labels
            },
            timestamp: Utc::now(),
        },
        Metric {
            id: Uuid::new_v4(),
            name: "http_requests_total".to_string(),
            metric_type: MetricType::Counter,
            value: MetricValue::Integer(1000),
            labels: {
                let mut labels = HashMap::new();
                labels.insert("method".to_string(), "GET".to_string());
                labels.insert("status".to_string(), "200".to_string());
                labels
            },
            timestamp: Utc::now(),
        },
    ]
}

#[tokio::test]
async fn test_monitoring_service_initialization() {
    let service = create_monitoring_service().await;
    let health = service.get_health_status().await.unwrap();

    assert_eq!(
        health.overall_status,
        core_monitoring::HealthStatus::Healthy
    );
    assert!(health.uptime_seconds > 0);
}

#[tokio::test]
async fn test_metrics_collection_and_storage() {
    let service = create_monitoring_service().await;
    let metrics = create_test_metrics();

    // Store metrics
    for metric in &metrics {
        service.store_metric(metric).await.unwrap();
    }

    // Query metrics
    let query = core_monitoring::MetricQuery {
        name: Some("cpu_usage_percent".to_string()),
        labels: HashMap::new(),
        start_time: Utc::now() - chrono::Duration::hours(1),
        end_time: Utc::now(),
        aggregation: Some(core_monitoring::AggregationType::Average),
        limit: Some(100),
        offset: Some(0),
    };

    let results = service.query_metrics(&query).await.unwrap();
    assert!(!results.is_empty());

    let cpu_metric = results
        .iter()
        .find(|m| m.name == "cpu_usage_percent")
        .unwrap();
    assert_eq!(cpu_metric.metric_type, MetricType::Gauge);

    if let MetricValue::Float(value) = cpu_metric.value {
        assert_eq!(value, 45.5);
    } else {
        panic!("Expected float value for CPU metric");
    }
}

#[tokio::test]
async fn test_system_metrics_collector() {
    let collector = SystemMetricsCollector::new();
    let metrics = collector.collect_metrics().await.unwrap();

    assert!(!metrics.is_empty());

    // Check for expected system metrics
    let metric_names: Vec<String> = metrics.iter().map(|m| m.name.clone()).collect();
    assert!(metric_names.contains(&"cpu_usage_percent".to_string()));
    assert!(metric_names.contains(&"memory_usage_bytes".to_string()));
    assert!(metric_names.contains(&"disk_usage_bytes".to_string()));
}

#[tokio::test]
async fn test_application_metrics_collector() {
    let collector = ApplicationMetricsCollector::new("test-app".to_string());
    let metrics = collector.collect_metrics().await.unwrap();

    assert!(!metrics.is_empty());

    // Check for application-specific metrics
    let metric_names: Vec<String> = metrics.iter().map(|m| m.name.clone()).collect();
    assert!(metric_names.contains(&"http_requests_total".to_string()));
    assert!(metric_names.contains(&"response_time_seconds".to_string()));
}

#[tokio::test]
async fn test_alert_manager() {
    let config = core_monitoring::AlertConfig {
        enabled: true,
        evaluation_interval_seconds: 10,
        notification_channels: vec!["email".to_string(), "slack".to_string()],
        rules: Vec::new(),
    };

    let alert_manager = AlertManagerImpl::new(config);

    // Create alert rule
    let rule = AlertRule {
        id: Uuid::new_v4(),
        name: "High CPU Usage".to_string(),
        description: "Alert when CPU usage exceeds 80%".to_string(),
        condition: AlertCondition {
            metric_name: "cpu_usage_percent".to_string(),
            operator: core_monitoring::ComparisonOperator::GreaterThan,
            threshold: 80.0,
            duration_seconds: 300,
        },
        severity: AlertSeverity::Warning,
        enabled: true,
        labels: HashMap::new(),
        annotations: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rule_id = alert_manager.add_rule(&rule).await.unwrap();
    assert_eq!(rule_id, rule.id);

    // Test alert evaluation
    let metrics = vec![Metric {
        id: Uuid::new_v4(),
        name: "cpu_usage_percent".to_string(),
        metric_type: MetricType::Gauge,
        value: MetricValue::Float(85.0), // Above threshold
        labels: HashMap::new(),
        timestamp: Utc::now(),
    }];

    let alerts = alert_manager.evaluate_rules(&metrics).await.unwrap();
    assert!(!alerts.is_empty());

    let alert = &alerts[0];
    assert_eq!(alert.rule_name, "High CPU Usage");
    assert_eq!(alert.severity, AlertSeverity::Warning);
}

#[tokio::test]
async fn test_dashboard_manager() {
    let config = DashboardConfig {
        enabled: true,
        refresh_interval_seconds: 30,
        default_time_range_hours: 24,
        max_widgets_per_dashboard: 20,
    };

    let dashboard_manager = DashboardManagerImpl::new(config);

    // List dashboards
    let dashboards = dashboard_manager.list_dashboards().await.unwrap();
    assert!(!dashboards.is_empty());

    let dashboard_id = dashboards[0].id;

    // Get dashboard data
    let time_range = TimeRange {
        start: Utc::now() - chrono::Duration::hours(1),
        end: Utc::now(),
    };

    let dashboard_data = dashboard_manager
        .get_dashboard_data(&dashboard_id, &time_range)
        .await
        .unwrap();
    assert_eq!(dashboard_data.dashboard.id, dashboard_id);
    assert!(!dashboard_data.widget_data.is_empty());
}

#[tokio::test]
async fn test_health_check_service() {
    let config = HealthCheckConfig {
        enabled: true,
        check_interval_seconds: 30,
        timeout_seconds: 10,
        checks: Vec::new(),
    };

    let health_service = HealthCheckServiceImpl::new(config);

    // Perform all health checks
    let checks = health_service.perform_all_checks().await.unwrap();
    assert!(!checks.is_empty());

    // Get system health
    let system_health = health_service.get_system_health().await.unwrap();
    assert_eq!(system_health.total_checks, checks.len() as u32);
    assert!(system_health.healthy_checks > 0);
}

#[tokio::test]
async fn test_log_aggregator() {
    let config = LogConfig {
        enabled: true,
        log_level: LogLevel::Info,
        retention_days: 7,
        structured_logging: true,
    };

    let log_aggregator = LogAggregatorImpl::new(config);

    // Create test log entry
    let log_entry = LogEntry {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        level: LogLevel::Info,
        message: "Test log message".to_string(),
        source: "test-service".to_string(),
        labels: HashMap::new(),
        fields: HashMap::new(),
    };

    // Store log
    log_aggregator.store_log(&log_entry).await.unwrap();

    // Query logs
    let query = core_monitoring::LogQuery {
        level: Some(LogLevel::Info),
        source: Some("test-service".to_string()),
        message_contains: Some("Test".to_string()),
        labels: HashMap::new(),
        start_time: Utc::now() - chrono::Duration::hours(1),
        end_time: Utc::now(),
        limit: Some(100),
        offset: Some(0),
    };

    let logs = log_aggregator.query_logs(&query).await.unwrap();
    assert!(!logs.is_empty());

    let found_log = logs.iter().find(|l| l.message.contains("Test")).unwrap();
    assert_eq!(found_log.level, LogLevel::Info);
    assert_eq!(found_log.source, "test-service");
}

#[tokio::test]
async fn test_metrics_aggregation() {
    let service = create_monitoring_service().await;

    // Store multiple metrics with same name but different timestamps
    let base_time = Utc::now() - chrono::Duration::minutes(10);
    for i in 0..5 {
        let metric = Metric {
            id: Uuid::new_v4(),
            name: "test_metric".to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Float(10.0 + i as f64),
            labels: HashMap::new(),
            timestamp: base_time + chrono::Duration::minutes(i as i64 * 2),
        };
        service.store_metric(&metric).await.unwrap();
    }

    // Query with aggregation
    let query = core_monitoring::MetricQuery {
        name: Some("test_metric".to_string()),
        labels: HashMap::new(),
        start_time: base_time - chrono::Duration::minutes(1),
        end_time: Utc::now(),
        aggregation: Some(core_monitoring::AggregationType::Average),
        limit: Some(100),
        offset: Some(0),
    };

    let results = service.query_metrics(&query).await.unwrap();
    assert!(!results.is_empty());

    // Verify aggregation worked
    let aggregated_metric = &results[0];
    if let MetricValue::Float(value) = aggregated_metric.value {
        assert!(value >= 10.0 && value <= 14.0); // Should be average of 10,11,12,13,14
    }
}

#[tokio::test]
async fn test_concurrent_metrics_collection() {
    let service = create_monitoring_service().await;
    let mut handles = Vec::new();

    // Spawn multiple concurrent metric collection tasks
    for i in 0..10 {
        let service_clone = service.clone();
        let handle = tokio::spawn(async move {
            let metric = Metric {
                id: Uuid::new_v4(),
                name: format!("concurrent_metric_{}", i),
                metric_type: MetricType::Counter,
                value: MetricValue::Integer(i),
                labels: HashMap::new(),
                timestamp: Utc::now(),
            };
            service_clone.store_metric(&metric).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    // Verify all metrics were stored
    let query = core_monitoring::MetricQuery {
        name: None,
        labels: HashMap::new(),
        start_time: Utc::now() - chrono::Duration::minutes(1),
        end_time: Utc::now(),
        aggregation: None,
        limit: Some(100),
        offset: Some(0),
    };

    let results = service.query_metrics(&query).await.unwrap();
    let concurrent_metrics: Vec<_> = results
        .iter()
        .filter(|m| m.name.starts_with("concurrent_metric_"))
        .collect();

    assert_eq!(concurrent_metrics.len(), 10);
}

#[tokio::test]
async fn test_alert_notification_flow() {
    let config = core_monitoring::AlertConfig {
        enabled: true,
        evaluation_interval_seconds: 1,
        notification_channels: vec!["test-channel".to_string()],
        rules: Vec::new(),
    };

    let alert_manager = AlertManagerImpl::new(config);

    // Create critical alert rule
    let rule = AlertRule {
        id: Uuid::new_v4(),
        name: "Critical Error Rate".to_string(),
        description: "Alert when error rate exceeds 5%".to_string(),
        condition: AlertCondition {
            metric_name: "error_rate_percent".to_string(),
            operator: core_monitoring::ComparisonOperator::GreaterThan,
            threshold: 5.0,
            duration_seconds: 60,
        },
        severity: AlertSeverity::Critical,
        enabled: true,
        labels: HashMap::new(),
        annotations: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    alert_manager.add_rule(&rule).await.unwrap();

    // Create metric that triggers alert
    let metric = Metric {
        id: Uuid::new_v4(),
        name: "error_rate_percent".to_string(),
        metric_type: MetricType::Gauge,
        value: MetricValue::Float(7.5), // Above threshold
        labels: HashMap::new(),
        timestamp: Utc::now(),
    };

    let alerts = alert_manager.evaluate_rules(&vec![metric]).await.unwrap();
    assert!(!alerts.is_empty());

    let alert = &alerts[0];
    assert_eq!(alert.severity, AlertSeverity::Critical);
    assert_eq!(alert.rule_name, "Critical Error Rate");
}
