// =====================================================================================
// File: core-monitoring/src/service.rs
// Description: Main monitoring service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    error::{MonitoringError, MonitoringResult},
    types::{
        Alert, AlertChannel, AlertRule, AlertSeverity, AlertStatus, Dashboard, HealthCheck,
        HealthStatus, LogEntry, LogLevel, Metric, MetricType, MonitoringConfig, TraceSpan,
    },
};

/// Main monitoring service trait
#[async_trait]
pub trait MonitoringService: Send + Sync {
    /// Collect and store metrics
    async fn collect_metric(&self, metric: &Metric) -> MonitoringResult<()>;

    /// Query metrics
    async fn query_metrics(&self, query: &MetricQuery) -> MonitoringResult<Vec<Metric>>;

    /// Create alert rule
    async fn create_alert_rule(&self, rule: &AlertRule) -> MonitoringResult<Uuid>;

    /// Process alerts
    async fn process_alerts(&self) -> MonitoringResult<Vec<Alert>>;

    /// Get active alerts
    async fn get_active_alerts(&self) -> MonitoringResult<Vec<Alert>>;

    /// Create dashboard
    async fn create_dashboard(&self, dashboard: &Dashboard) -> MonitoringResult<Uuid>;

    /// Get dashboard
    async fn get_dashboard(&self, dashboard_id: &Uuid) -> MonitoringResult<Option<Dashboard>>;

    /// Perform health check
    async fn perform_health_check(&self, check_id: &Uuid) -> MonitoringResult<HealthCheck>;

    /// Get service health status
    async fn get_health_status(&self) -> MonitoringResult<MonitoringHealthStatus>;

    /// Store log entry
    async fn store_log(&self, log_entry: &LogEntry) -> MonitoringResult<()>;

    /// Query logs
    async fn query_logs(&self, query: &LogQuery) -> MonitoringResult<Vec<LogEntry>>;

    /// Store trace span
    async fn store_trace(&self, span: &TraceSpan) -> MonitoringResult<()>;

    /// Query traces
    async fn query_traces(&self, query: &TraceQuery) -> MonitoringResult<Vec<TraceSpan>>;
}

/// Metric query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricQuery {
    pub metric_name: Option<String>,
    pub labels: HashMap<String, String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub step: chrono::Duration,
    pub aggregation: Option<AggregationType>,
}

/// Aggregation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Min,
    Max,
    Count,
    Rate,
}

/// Log query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQuery {
    pub level: Option<LogLevel>,
    pub source: Option<String>,
    pub message_contains: Option<String>,
    pub labels: HashMap<String, String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Trace query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceQuery {
    pub trace_id: Option<String>,
    pub operation_name: Option<String>,
    pub service_name: Option<String>,
    pub min_duration: Option<chrono::Duration>,
    pub max_duration: Option<chrono::Duration>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub limit: Option<u32>,
}

/// Monitoring service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringHealthStatus {
    pub status: String,
    pub metrics_collected: u64,
    pub active_alerts: u32,
    pub dashboards_count: u32,
    pub health_checks_passing: u32,
    pub health_checks_failing: u32,
    pub log_entries_stored: u64,
    pub traces_stored: u64,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// Main monitoring service implementation
pub struct MonitoringServiceImpl {
    config: MonitoringConfig,
    metrics: Arc<RwLock<HashMap<String, Vec<Metric>>>>,
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    alert_rules: Arc<RwLock<HashMap<Uuid, AlertRule>>>,
    dashboards: Arc<RwLock<HashMap<Uuid, Dashboard>>>,
    health_checks: Arc<RwLock<HashMap<Uuid, HealthCheck>>>,
    logs: Arc<RwLock<Vec<LogEntry>>>,
    traces: Arc<RwLock<Vec<TraceSpan>>>,
    start_time: chrono::DateTime<chrono::Utc>,
}

impl MonitoringServiceImpl {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Arc::new(RwLock::new(HashMap::new())),
            dashboards: Arc::new(RwLock::new(HashMap::new())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            traces: Arc::new(RwLock::new(Vec::new())),
            start_time: Utc::now(),
        }
    }

    /// Initialize service with default data
    pub async fn initialize(&self) -> MonitoringResult<()> {
        // Initialize default alert rules, dashboards, etc.
        Ok(())
    }

    fn evaluate_alert_rule(&self, rule: &AlertRule, metrics: &[Metric]) -> bool {
        // Mock alert rule evaluation
        // In reality, this would parse and evaluate the query against metrics
        !metrics.is_empty()
    }

    fn create_alert_from_rule(&self, rule: &AlertRule) -> Alert {
        Alert {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            name: rule.name.clone(),
            description: format!("Alert triggered by rule: {}", rule.name),
            severity: rule.severity,
            status: AlertStatus::Firing,
            labels: rule.labels.clone(),
            annotations: rule.annotations.clone(),
            starts_at: Utc::now(),
            ends_at: None,
            generator_url: None,
        }
    }

    async fn cleanup_old_data(&self) -> MonitoringResult<()> {
        let retention_cutoff =
            Utc::now() - chrono::Duration::days(self.config.metrics_config.retention_days as i64);

        // Clean up old metrics
        let mut metrics = self.metrics.write().await;
        for metric_series in metrics.values_mut() {
            metric_series.retain(|metric| metric.timestamp > retention_cutoff);
        }

        // Clean up old logs
        let log_retention_cutoff =
            Utc::now() - chrono::Duration::days(self.config.log_config.retention_days as i64);
        let mut logs = self.logs.write().await;
        logs.retain(|log| log.timestamp > log_retention_cutoff);

        Ok(())
    }

    fn calculate_memory_usage(&self) -> f64 {
        // Mock memory usage calculation
        // In reality, this would use system APIs to get actual memory usage
        128.5 // MB
    }

    fn calculate_cpu_usage(&self) -> f64 {
        // Mock CPU usage calculation
        // In reality, this would use system APIs to get actual CPU usage
        15.2 // Percent
    }
}

#[async_trait]
impl MonitoringService for MonitoringServiceImpl {
    async fn collect_metric(&self, metric: &Metric) -> MonitoringResult<()> {
        let mut metrics = self.metrics.write().await;
        let metric_series = metrics.entry(metric.name.clone()).or_insert_with(Vec::new);
        metric_series.push(metric.clone());

        // Keep only recent metrics to prevent memory bloat
        let retention_cutoff =
            Utc::now() - chrono::Duration::days(self.config.metrics_config.retention_days as i64);
        metric_series.retain(|m| m.timestamp > retention_cutoff);

        Ok(())
    }

    async fn query_metrics(&self, query: &MetricQuery) -> MonitoringResult<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        let mut results = Vec::new();

        for (metric_name, metric_series) in metrics.iter() {
            // Filter by metric name if specified
            if let Some(ref name) = query.metric_name {
                if metric_name != name {
                    continue;
                }
            }

            for metric in metric_series {
                // Filter by time range
                if metric.timestamp < query.start_time || metric.timestamp > query.end_time {
                    continue;
                }

                // Filter by labels
                let mut matches_labels = true;
                for (key, value) in &query.labels {
                    if metric.labels.get(key) != Some(value) {
                        matches_labels = false;
                        break;
                    }
                }

                if matches_labels {
                    results.push(metric.clone());
                }
            }
        }

        // Sort by timestamp
        results.sort_by_key(|m| m.timestamp);

        Ok(results)
    }

    async fn create_alert_rule(&self, rule: &AlertRule) -> MonitoringResult<Uuid> {
        let mut alert_rules = self.alert_rules.write().await;
        alert_rules.insert(rule.id, rule.clone());
        Ok(rule.id)
    }

    async fn process_alerts(&self) -> MonitoringResult<Vec<Alert>> {
        let alert_rules = self.alert_rules.read().await;
        let metrics = self.metrics.read().await;
        let mut new_alerts = Vec::new();

        for rule in alert_rules.values() {
            if !rule.enabled {
                continue;
            }

            // Get metrics for this rule (simplified)
            let rule_metrics: Vec<Metric> = metrics.values().flatten().cloned().collect();

            if self.evaluate_alert_rule(rule, &rule_metrics) {
                let alert = self.create_alert_from_rule(rule);
                new_alerts.push(alert.clone());

                // Store the alert
                let mut alerts = self.alerts.write().await;
                alerts.insert(alert.id, alert);
            }
        }

        Ok(new_alerts)
    }

    async fn get_active_alerts(&self) -> MonitoringResult<Vec<Alert>> {
        let alerts = self.alerts.read().await;
        let active_alerts: Vec<Alert> = alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Firing)
            .cloned()
            .collect();

        Ok(active_alerts)
    }

    async fn create_dashboard(&self, dashboard: &Dashboard) -> MonitoringResult<Uuid> {
        let mut dashboards = self.dashboards.write().await;
        dashboards.insert(dashboard.id, dashboard.clone());
        Ok(dashboard.id)
    }

    async fn get_dashboard(&self, dashboard_id: &Uuid) -> MonitoringResult<Option<Dashboard>> {
        let dashboards = self.dashboards.read().await;
        Ok(dashboards.get(dashboard_id).cloned())
    }

    async fn perform_health_check(&self, check_id: &Uuid) -> MonitoringResult<HealthCheck> {
        // Mock health check implementation
        let health_check = HealthCheck {
            id: Uuid::new_v4(),
            definition_id: *check_id,
            status: HealthStatus::Healthy,
            response_time: chrono::Duration::milliseconds(50),
            message: "Service is healthy".to_string(),
            checked_at: Utc::now(),
        };

        // Store the health check result
        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(health_check.id, health_check.clone());

        Ok(health_check)
    }

    async fn get_health_status(&self) -> MonitoringResult<MonitoringHealthStatus> {
        let metrics = self.metrics.read().await;
        let alerts = self.alerts.read().await;
        let dashboards = self.dashboards.read().await;
        let health_checks = self.health_checks.read().await;
        let logs = self.logs.read().await;
        let traces = self.traces.read().await;

        let metrics_collected = metrics.values().map(|series| series.len() as u64).sum();
        let active_alerts = alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Firing)
            .count() as u32;

        let health_checks_passing = health_checks
            .values()
            .filter(|check| check.status == HealthStatus::Healthy)
            .count() as u32;

        let health_checks_failing = health_checks
            .values()
            .filter(|check| check.status == HealthStatus::Unhealthy)
            .count() as u32;

        let uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;

        Ok(MonitoringHealthStatus {
            status: "healthy".to_string(),
            metrics_collected,
            active_alerts,
            dashboards_count: dashboards.len() as u32,
            health_checks_passing,
            health_checks_failing,
            log_entries_stored: logs.len() as u64,
            traces_stored: traces.len() as u64,
            uptime_seconds,
            memory_usage_mb: self.calculate_memory_usage(),
            cpu_usage_percent: self.calculate_cpu_usage(),
        })
    }

    async fn store_log(&self, log_entry: &LogEntry) -> MonitoringResult<()> {
        let mut logs = self.logs.write().await;
        logs.push(log_entry.clone());

        // Keep only recent logs to prevent memory bloat
        let retention_cutoff =
            Utc::now() - chrono::Duration::days(self.config.log_config.retention_days as i64);
        logs.retain(|log| log.timestamp > retention_cutoff);

        Ok(())
    }

    async fn query_logs(&self, query: &LogQuery) -> MonitoringResult<Vec<LogEntry>> {
        let logs = self.logs.read().await;
        let mut results: Vec<LogEntry> = logs
            .iter()
            .filter(|log| {
                // Filter by time range
                if log.timestamp < query.start_time || log.timestamp > query.end_time {
                    return false;
                }

                // Filter by log level
                if let Some(level) = query.level {
                    if log.level != level {
                        return false;
                    }
                }

                // Filter by source
                if let Some(ref source) = query.source {
                    if log.source != *source {
                        return false;
                    }
                }

                // Filter by message content
                if let Some(ref contains) = query.message_contains {
                    if !log.message.contains(contains) {
                        return false;
                    }
                }

                // Filter by labels
                for (key, value) in &query.labels {
                    if log.labels.get(key) != Some(value) {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(1000) as usize;

        if offset < results.len() {
            let end = (offset + limit).min(results.len());
            results = results[offset..end].to_vec();
        } else {
            results.clear();
        }

        Ok(results)
    }

    async fn store_trace(&self, span: &TraceSpan) -> MonitoringResult<()> {
        let mut traces = self.traces.write().await;
        traces.push(span.clone());

        // Keep only recent traces to prevent memory bloat
        let retention_cutoff = Utc::now() - chrono::Duration::days(7); // Keep traces for 7 days
        traces.retain(|trace| trace.start_time > retention_cutoff);

        Ok(())
    }

    async fn query_traces(&self, query: &TraceQuery) -> MonitoringResult<Vec<TraceSpan>> {
        let traces = self.traces.read().await;
        let mut results: Vec<TraceSpan> = traces
            .iter()
            .filter(|trace| {
                // Filter by time range
                if trace.start_time < query.start_time || trace.start_time > query.end_time {
                    return false;
                }

                // Filter by trace ID
                if let Some(ref trace_id) = query.trace_id {
                    if trace.trace_id != *trace_id {
                        return false;
                    }
                }

                // Filter by operation name
                if let Some(ref operation_name) = query.operation_name {
                    if trace.operation_name != *operation_name {
                        return false;
                    }
                }

                // Filter by service name
                if let Some(ref service_name) = query.service_name {
                    if trace.tags.get("service.name") != Some(service_name) {
                        return false;
                    }
                }

                // Filter by duration
                if let Some(duration) = trace.duration {
                    if let Some(min_duration) = query.min_duration {
                        if duration < min_duration {
                            return false;
                        }
                    }
                    if let Some(max_duration) = query.max_duration {
                        if duration > max_duration {
                            return false;
                        }
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by start time (newest first)
        results.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit as usize);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        let config = MonitoringConfig::default();
        let service = MonitoringServiceImpl::new(config);

        assert!(service.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_metric_collection() {
        let config = MonitoringConfig::default();
        let service = MonitoringServiceImpl::new(config);

        let metric = Metric {
            id: Uuid::new_v4(),
            name: "test_metric".to_string(),
            metric_type: MetricType::Counter,
            value: crate::types::MetricValue::Counter(100),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            source: "test".to_string(),
        };

        let result = service.collect_metric(&metric).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metric_query() {
        let config = MonitoringConfig::default();
        let service = MonitoringServiceImpl::new(config);

        // First collect a metric
        let metric = Metric {
            id: Uuid::new_v4(),
            name: "test_metric".to_string(),
            metric_type: MetricType::Counter,
            value: crate::types::MetricValue::Counter(100),
            labels: HashMap::new(),
            timestamp: Utc::now(),
            source: "test".to_string(),
        };

        service.collect_metric(&metric).await.unwrap();

        // Then query for it
        let query = MetricQuery {
            metric_name: Some("test_metric".to_string()),
            labels: HashMap::new(),
            start_time: Utc::now() - chrono::Duration::hours(1),
            end_time: Utc::now() + chrono::Duration::hours(1),
            step: chrono::Duration::minutes(1),
            aggregation: None,
        };

        let results = service.query_metrics(&query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_metric");
    }

    #[tokio::test]
    async fn test_health_status() {
        let config = MonitoringConfig::default();
        let service = MonitoringServiceImpl::new(config);

        let health = service.get_health_status().await.unwrap();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.metrics_collected, 0);
        assert_eq!(health.active_alerts, 0);
    }
}
