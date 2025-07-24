// =====================================================================================
// File: core-analytics/src/service.rs
// Description: Main analytics service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    aggregation::{AggregationEngine, AggregationSpec, InMemoryAggregationEngine},
    metrics::{InMemoryMetricsCollector, MetricsCollector},
    reporting::{InMemoryReportGenerator, ReportGenerator},
    types::*,
    AlertConfig, AnalyticsConfig, AnalyticsError, AnalyticsEvent, AnalyticsQuery, AnalyticsResult,
    QueryResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Analytics service trait
#[async_trait]
pub trait AnalyticsService: Send + Sync {
    /// Execute an analytics query
    async fn execute_query(&self, query: AnalyticsQuery) -> AnalyticsResult<QueryResult>;

    /// Track an analytics event
    async fn track_event(&self, event: AnalyticsEvent) -> AnalyticsResult<()>;

    /// Generate a report
    async fn generate_report(
        &self,
        report_type: ReportType,
        parameters: std::collections::HashMap<String, serde_json::Value>,
    ) -> AnalyticsResult<AnalyticsReport>;

    /// Create or update a dashboard
    async fn save_dashboard(&self, dashboard: Dashboard) -> AnalyticsResult<()>;

    /// Get a dashboard by ID
    async fn get_dashboard(&self, dashboard_id: Uuid) -> AnalyticsResult<Option<Dashboard>>;

    /// List all dashboards
    async fn list_dashboards(&self) -> AnalyticsResult<Vec<Dashboard>>;

    /// Create or update an alert
    async fn save_alert(&self, alert: AlertConfig) -> AnalyticsResult<()>;

    /// Get alerts
    async fn get_alerts(&self) -> AnalyticsResult<Vec<AlertConfig>>;

    /// Check alerts and trigger notifications
    async fn check_alerts(&self) -> AnalyticsResult<Vec<AlertTrigger>>;

    /// Get analytics statistics
    async fn get_statistics(&self) -> AnalyticsResult<AnalyticsStatistics>;

    /// Health check
    async fn health_check(&self) -> AnalyticsResult<HealthStatus>;
}

/// Alert trigger information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AlertTrigger {
    pub alert_id: Uuid,
    pub alert_name: String,
    pub triggered_at: DateTime<Utc>,
    pub current_value: MetricValue,
    pub threshold_value: rust_decimal::Decimal,
    pub condition: crate::AlertCondition,
    pub message: String,
}

/// Analytics statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalyticsStatistics {
    pub total_queries: u64,
    pub total_events: u64,
    pub total_reports: u64,
    pub total_dashboards: u64,
    pub total_alerts: u64,
    pub active_alerts: u64,
    pub average_query_time_ms: f64,
    pub cache_hit_rate: f64,
    pub uptime_seconds: u64,
    pub last_updated: DateTime<Utc>,
}

/// Health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub active_connections: u64,
    pub last_check: DateTime<Utc>,
    pub components: std::collections::HashMap<String, ComponentHealth>,
}

/// Component health status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Main analytics service implementation
pub struct DefaultAnalyticsService {
    config: AnalyticsConfig,
    metrics_collector: Arc<dyn MetricsCollector>,
    aggregation_engine: Arc<dyn AggregationEngine>,
    report_generator: Arc<dyn ReportGenerator>,
    dashboards: Arc<RwLock<std::collections::HashMap<Uuid, Dashboard>>>,
    alerts: Arc<RwLock<std::collections::HashMap<Uuid, AlertConfig>>>,
    events: Arc<RwLock<Vec<AnalyticsEvent>>>,
    statistics: Arc<RwLock<AnalyticsStatistics>>,
    start_time: DateTime<Utc>,
}

impl DefaultAnalyticsService {
    /// Create a new analytics service
    pub fn new(config: AnalyticsConfig) -> Self {
        let metrics_collector =
            Arc::new(InMemoryMetricsCollector::new(config.metrics_config.clone()));
        let aggregation_engine = Arc::new(InMemoryAggregationEngine::new(
            config.aggregation_config.clone(),
        ));
        let report_generator = Arc::new(InMemoryReportGenerator::new(
            config.reporting_config.clone(),
        ));

        Self {
            config,
            metrics_collector,
            aggregation_engine,
            report_generator,
            dashboards: Arc::new(RwLock::new(std::collections::HashMap::new())),
            alerts: Arc::new(RwLock::new(std::collections::HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(AnalyticsStatistics {
                total_queries: 0,
                total_events: 0,
                total_reports: 0,
                total_dashboards: 0,
                total_alerts: 0,
                active_alerts: 0,
                average_query_time_ms: 0.0,
                cache_hit_rate: 0.0,
                uptime_seconds: 0,
                last_updated: Utc::now(),
            })),
            start_time: Utc::now(),
        }
    }

    /// Update statistics
    async fn update_statistics<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut AnalyticsStatistics),
    {
        let mut stats = self.statistics.write().await;
        update_fn(&mut stats);
        stats.last_updated = Utc::now();
        stats.uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;
    }

    /// Convert query to aggregation specs
    fn query_to_aggregation_specs(&self, query: &AnalyticsQuery) -> Vec<AggregationSpec> {
        // This is a simplified conversion - in a real implementation,
        // this would be more sophisticated
        query
            .metrics
            .iter()
            .map(|metric_name| AggregationSpec {
                id: Uuid::new_v4(),
                name: format!("query_{}_{}", query.id, metric_name),
                description: format!("Aggregation for metric {}", metric_name),
                aggregation_type: query
                    .aggregation
                    .unwrap_or(crate::aggregation::AggregationType::Sum),
                field: metric_name.clone(),
                group_by: query.group_by.clone(),
                filters: query
                    .filters
                    .iter()
                    .map(|f| crate::aggregation::AggregationFilter {
                        field: f.field.clone(),
                        operator: match f.operator {
                            crate::FilterOperator::Equal => {
                                crate::aggregation::FilterOperator::Equal
                            }
                            crate::FilterOperator::GreaterThan => {
                                crate::aggregation::FilterOperator::GreaterThan
                            }
                            crate::FilterOperator::LessThan => {
                                crate::aggregation::FilterOperator::LessThan
                            }
                            _ => crate::aggregation::FilterOperator::Equal,
                        },
                        value: f.value.clone(),
                        case_sensitive: true,
                    })
                    .collect(),
                time_window: None,
                output_field: None,
                weight_field: None,
                created_at: Utc::now(),
                enabled: true,
            })
            .collect()
    }

    /// Check if an alert should be triggered
    async fn should_trigger_alert(
        &self,
        alert: &AlertConfig,
    ) -> AnalyticsResult<Option<AlertTrigger>> {
        // Get recent metrics for the alert
        let time_range = TimeRange {
            start: Utc::now() - chrono::Duration::minutes(alert.time_window_minutes as i64),
            end: Utc::now(),
        };

        let metrics = self
            .metrics_collector
            .get_metric(&alert.metric, time_range, None)
            .await?;

        if metrics.is_empty() {
            return Ok(None);
        }

        // Get the latest metric value
        let latest_metric = metrics.last().unwrap();
        let current_value = &latest_metric.value;

        // Check if the condition is met
        let should_trigger = match alert.condition {
            crate::AlertCondition::GreaterThan => {
                if let Some(value) = current_value.as_f64() {
                    rust_decimal::Decimal::from_f64_retain(value).unwrap_or_default()
                        > alert.threshold
                } else {
                    false
                }
            }
            crate::AlertCondition::LessThan => {
                if let Some(value) = current_value.as_f64() {
                    rust_decimal::Decimal::from_f64_retain(value).unwrap_or_default()
                        < alert.threshold
                } else {
                    false
                }
            }
            _ => false, // Other conditions not implemented in this example
        };

        if should_trigger {
            Ok(Some(AlertTrigger {
                alert_id: alert.id,
                alert_name: alert.name.clone(),
                triggered_at: Utc::now(),
                current_value: current_value.clone(),
                threshold_value: alert.threshold,
                condition: alert.condition,
                message: format!(
                    "Alert '{}' triggered: current value {:?} {} threshold {}",
                    alert.name,
                    current_value,
                    match alert.condition {
                        crate::AlertCondition::GreaterThan => ">",
                        crate::AlertCondition::LessThan => "<",
                        _ => "?",
                    },
                    alert.threshold
                ),
            }))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl AnalyticsService for DefaultAnalyticsService {
    async fn execute_query(&self, query: AnalyticsQuery) -> AnalyticsResult<QueryResult> {
        let start_time = std::time::Instant::now();

        // Convert query to aggregation specifications
        let aggregation_specs = self.query_to_aggregation_specs(&query);

        // For this example, we'll create some mock data
        let mock_data = vec![
            {
                let mut record = std::collections::HashMap::new();
                record.insert("metric1".to_string(), serde_json::json!(100));
                record.insert("category".to_string(), serde_json::json!("A"));
                record
            },
            {
                let mut record = std::collections::HashMap::new();
                record.insert("metric1".to_string(), serde_json::json!(200));
                record.insert("category".to_string(), serde_json::json!("B"));
                record
            },
        ];

        // Execute aggregations
        let mut all_results = Vec::new();
        for spec in aggregation_specs {
            let result = self
                .aggregation_engine
                .execute_aggregation(&spec, mock_data.clone())
                .await?;
            all_results.push(result);
        }

        // Convert aggregation results to query result format
        let mut data = Vec::new();
        for result in all_results {
            for group in result.groups {
                let mut record = HashMap::new();
                record.insert("group_key".to_string(), serde_json::json!(group.group_key));
                record.insert(
                    "value".to_string(),
                    match group.aggregated_value {
                        MetricValue::Integer(i) => serde_json::json!(i),
                        MetricValue::Float(f) => serde_json::json!(f),
                        MetricValue::Decimal(d) => serde_json::json!(d.to_f64()),
                        MetricValue::String(s) => serde_json::json!(s),
                        MetricValue::Boolean(b) => serde_json::json!(b),
                        _ => serde_json::json!(null),
                    },
                );
                data.push(record);
            }
        }

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        // Update statistics
        self.update_statistics(|stats| {
            stats.total_queries += 1;
            stats.average_query_time_ms = (stats.average_query_time_ms
                * (stats.total_queries - 1) as f64
                + execution_time_ms as f64)
                / stats.total_queries as f64;
        })
        .await;

        let total_rows = data.len();
        Ok(QueryResult {
            query_id: query.id,
            data,
            metadata: crate::QueryMetadata {
                columns: vec![crate::ColumnInfo {
                    name: "value".to_string(),
                    data_type: crate::DataType::Float,
                    nullable: false,
                    description: Some("Aggregated value".to_string()),
                }],
                data_sources: vec!["in_memory".to_string()],
                cache_hit: false,
                query_plan: None,
            },
            execution_time_ms,
            total_rows,
            has_more: false,
        })
    }

    async fn track_event(&self, event: AnalyticsEvent) -> AnalyticsResult<()> {
        let mut events = self.events.write().await;
        events.push(event);

        // Apply retention policy
        let events_len = events.len();
        if events_len > 100000 {
            events.drain(0..events_len - 100000);
        }

        self.update_statistics(|stats| {
            stats.total_events += 1;
        })
        .await;

        Ok(())
    }

    async fn generate_report(
        &self,
        report_type: ReportType,
        parameters: std::collections::HashMap<String, serde_json::Value>,
    ) -> AnalyticsResult<AnalyticsReport> {
        let report = self
            .report_generator
            .generate_report(report_type, parameters)
            .await?;

        self.update_statistics(|stats| {
            stats.total_reports += 1;
        })
        .await;

        Ok(report)
    }

    async fn save_dashboard(&self, dashboard: Dashboard) -> AnalyticsResult<()> {
        let mut dashboards = self.dashboards.write().await;
        let is_new = !dashboards.contains_key(&dashboard.id);
        dashboards.insert(dashboard.id, dashboard);

        if is_new {
            self.update_statistics(|stats| {
                stats.total_dashboards += 1;
            })
            .await;
        }

        Ok(())
    }

    async fn get_dashboard(&self, dashboard_id: Uuid) -> AnalyticsResult<Option<Dashboard>> {
        let dashboards = self.dashboards.read().await;
        Ok(dashboards.get(&dashboard_id).cloned())
    }

    async fn list_dashboards(&self) -> AnalyticsResult<Vec<Dashboard>> {
        let dashboards = self.dashboards.read().await;
        Ok(dashboards.values().cloned().collect())
    }

    async fn save_alert(&self, alert: AlertConfig) -> AnalyticsResult<()> {
        let mut alerts = self.alerts.write().await;
        let is_new = !alerts.contains_key(&alert.id);
        let alert_enabled = alert.enabled;
        alerts.insert(alert.id, alert);

        if is_new {
            self.update_statistics(|stats| {
                stats.total_alerts += 1;
                if alert_enabled {
                    stats.active_alerts += 1;
                }
            })
            .await;
        }

        Ok(())
    }

    async fn get_alerts(&self) -> AnalyticsResult<Vec<AlertConfig>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.values().cloned().collect())
    }

    async fn check_alerts(&self) -> AnalyticsResult<Vec<AlertTrigger>> {
        let alerts = self.alerts.read().await;
        let mut triggers = Vec::new();

        for alert in alerts.values() {
            if alert.enabled {
                if let Some(trigger) = self.should_trigger_alert(alert).await? {
                    triggers.push(trigger);
                }
            }
        }

        Ok(triggers)
    }

    async fn get_statistics(&self) -> AnalyticsResult<AnalyticsStatistics> {
        let stats = self.statistics.read().await;
        Ok(stats.clone())
    }

    async fn health_check(&self) -> AnalyticsResult<HealthStatus> {
        let stats = self.statistics.read().await;
        let uptime = (Utc::now() - self.start_time).num_seconds() as u64;

        let mut components = std::collections::HashMap::new();

        // Check metrics collector health
        components.insert(
            "metrics_collector".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                last_check: Utc::now(),
                error_message: None,
                response_time_ms: Some(1),
            },
        );

        // Check aggregation engine health
        components.insert(
            "aggregation_engine".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                last_check: Utc::now(),
                error_message: None,
                response_time_ms: Some(2),
            },
        );

        // Check report generator health
        components.insert(
            "report_generator".to_string(),
            ComponentHealth {
                status: "healthy".to_string(),
                last_check: Utc::now(),
                error_message: None,
                response_time_ms: Some(3),
            },
        );

        Ok(HealthStatus {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: uptime,
            memory_usage_mb: 128,    // Mock value
            cpu_usage_percent: 15.5, // Mock value
            active_connections: 10,  // Mock value
            last_check: Utc::now(),
            components,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnalyticsConfig;

    #[tokio::test]
    async fn test_analytics_service() {
        let config = AnalyticsConfig::default();
        let service = DefaultAnalyticsService::new(config);

        // Test health check
        let health = service.health_check().await.unwrap();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "1.0.0");

        // Test statistics
        let stats = service.get_statistics().await.unwrap();
        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.total_events, 0);
    }

    #[tokio::test]
    async fn test_event_tracking() {
        let config = AnalyticsConfig::default();
        let service = DefaultAnalyticsService::new(config);

        let event = AnalyticsEvent::new("test_event".to_string(), "test_source".to_string());
        service.track_event(event).await.unwrap();

        let stats = service.get_statistics().await.unwrap();
        assert_eq!(stats.total_events, 1);
    }
}
