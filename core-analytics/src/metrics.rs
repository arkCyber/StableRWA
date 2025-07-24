// =====================================================================================
// File: core-analytics/src/metrics.rs
// Description: Metrics collection and management system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{types::*, AnalyticsError, AnalyticsResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Collection interval in seconds
    pub collection_interval_seconds: u64,
    /// Batch size for metric collection
    pub batch_size: usize,
    /// Enable real-time metrics
    pub real_time_enabled: bool,
    /// Metric retention period in days
    pub retention_days: u32,
    /// Enable metric aggregation
    pub aggregation_enabled: bool,
    /// Aggregation intervals
    pub aggregation_intervals: Vec<AggregationInterval>,
    /// Maximum metrics per collection
    pub max_metrics_per_collection: usize,
    /// Enable metric validation
    pub validation_enabled: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: 60,
            batch_size: 1000,
            real_time_enabled: true,
            retention_days: 90,
            aggregation_enabled: true,
            aggregation_intervals: vec![
                AggregationInterval::Minute,
                AggregationInterval::Hour,
                AggregationInterval::Day,
            ],
            max_metrics_per_collection: 10000,
            validation_enabled: true,
        }
    }
}

/// Aggregation interval enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationInterval {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl AggregationInterval {
    /// Get the duration in seconds
    pub fn duration_seconds(&self) -> u64 {
        match self {
            AggregationInterval::Minute => 60,
            AggregationInterval::Hour => 3600,
            AggregationInterval::Day => 86400,
            AggregationInterval::Week => 604800,
            AggregationInterval::Month => 2592000,   // 30 days
            AggregationInterval::Quarter => 7776000, // 90 days
            AggregationInterval::Year => 31536000,   // 365 days
        }
    }
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub unit: String,
    pub tags: HashMap<String, String>,
    pub collection_method: CollectionMethod,
    pub validation_rules: Vec<ValidationRule>,
    pub aggregation_functions: Vec<AggregationFunction>,
    pub retention_policy: RetentionPolicy,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled: bool,
}

/// Collection method enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionMethod {
    /// Push-based collection (metrics sent to collector)
    Push,
    /// Pull-based collection (collector fetches metrics)
    Pull,
    /// Event-driven collection
    Event,
    /// Scheduled collection
    Scheduled,
}

/// Validation rule for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub error_action: ValidationErrorAction,
}

/// Validation rule type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    Range,
    Pattern,
    Required,
    DataType,
    Custom,
}

/// Validation error action enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationErrorAction {
    Reject,
    Warn,
    Correct,
    Ignore,
}

/// Aggregation function enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationFunction {
    Sum,
    Average,
    Min,
    Max,
    Count,
    CountDistinct,
    Median,
    Percentile(u8),
    StandardDeviation,
    Variance,
}

/// Retention policy for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub raw_data_days: u32,
    pub aggregated_data_days: u32,
    pub archive_after_days: Option<u32>,
    pub delete_after_days: Option<u32>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            raw_data_days: 30,
            aggregated_data_days: 365,
            archive_after_days: Some(90),
            delete_after_days: Some(2555), // 7 years
        }
    }
}

/// Metrics collector trait
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Collect a single metric
    async fn collect_metric(&self, metric: Metric) -> AnalyticsResult<()>;

    /// Collect multiple metrics in batch
    async fn collect_metrics(&self, metrics: Vec<Metric>) -> AnalyticsResult<()>;

    /// Get metric by name and time range
    async fn get_metric(
        &self,
        name: &str,
        time_range: TimeRange,
        tags: Option<HashMap<String, String>>,
    ) -> AnalyticsResult<Vec<Metric>>;

    /// Get aggregated metrics
    async fn get_aggregated_metrics(
        &self,
        name: &str,
        time_range: TimeRange,
        interval: AggregationInterval,
        function: AggregationFunction,
        tags: Option<HashMap<String, String>>,
    ) -> AnalyticsResult<Vec<AggregatedMetric>>;

    /// Register a metric definition
    async fn register_metric_definition(&self, definition: MetricDefinition)
        -> AnalyticsResult<()>;

    /// Get metric definitions
    async fn get_metric_definitions(&self) -> AnalyticsResult<Vec<MetricDefinition>>;

    /// Delete metrics by criteria
    async fn delete_metrics(
        &self,
        name: &str,
        time_range: Option<TimeRange>,
        tags: Option<HashMap<String, String>>,
    ) -> AnalyticsResult<u64>;

    /// Get metric statistics
    async fn get_metric_statistics(&self, name: &str) -> AnalyticsResult<MetricStatistics>;
}

/// Aggregated metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub interval: AggregationInterval,
    pub function: AggregationFunction,
    pub value: MetricValue,
    pub count: u64,
    pub tags: HashMap<String, String>,
}

/// Metric statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStatistics {
    pub name: String,
    pub total_count: u64,
    pub first_timestamp: Option<DateTime<Utc>>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub unique_tag_combinations: u64,
    pub average_value: Option<Decimal>,
    pub min_value: Option<MetricValue>,
    pub max_value: Option<MetricValue>,
    pub data_points_per_day: HashMap<String, u64>,
}

/// In-memory metrics collector implementation
pub struct InMemoryMetricsCollector {
    metrics: tokio::sync::RwLock<Vec<Metric>>,
    definitions: tokio::sync::RwLock<HashMap<String, MetricDefinition>>,
    config: MetricsConfig,
}

impl InMemoryMetricsCollector {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            metrics: tokio::sync::RwLock::new(Vec::new()),
            definitions: tokio::sync::RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Validate a metric against its definition
    async fn validate_metric(&self, metric: &Metric) -> AnalyticsResult<()> {
        if !self.config.validation_enabled {
            return Ok(());
        }

        let definitions = self.definitions.read().await;
        if let Some(definition) = definitions.get(&metric.name) {
            for rule in &definition.validation_rules {
                self.apply_validation_rule(metric, rule)?;
            }
        }

        Ok(())
    }

    /// Apply a validation rule to a metric
    fn apply_validation_rule(&self, metric: &Metric, rule: &ValidationRule) -> AnalyticsResult<()> {
        match rule.rule_type {
            ValidationRuleType::Range => {
                if let (Some(min), Some(max)) = (
                    rule.parameters.get("min").and_then(|v| v.as_f64()),
                    rule.parameters.get("max").and_then(|v| v.as_f64()),
                ) {
                    let value = match &metric.value {
                        MetricValue::Integer(i) => *i as f64,
                        MetricValue::Float(f) => *f,
                        MetricValue::Decimal(d) => d.to_f64().unwrap_or(0.0),
                        _ => return Ok(()), // Skip validation for non-numeric values
                    };

                    if value < min || value > max {
                        match rule.error_action {
                            ValidationErrorAction::Reject => {
                                return Err(AnalyticsError::validation_error(
                                    metric.name.clone(),
                                    format!("Value {} is outside range [{}, {}]", value, min, max),
                                ));
                            }
                            ValidationErrorAction::Warn => {
                                tracing::warn!(
                                    "Metric {} value {} is outside range [{}, {}]",
                                    metric.name,
                                    value,
                                    min,
                                    max
                                );
                            }
                            _ => {} // Other actions not implemented in this example
                        }
                    }
                }
            }
            ValidationRuleType::Required => {
                if matches!(metric.value, MetricValue::Null) {
                    return Err(AnalyticsError::validation_error(
                        metric.name.clone(),
                        "Required metric value is null".to_string(),
                    ));
                }
            }
            _ => {} // Other validation rules not implemented in this example
        }

        Ok(())
    }
}

#[async_trait]
impl MetricsCollector for InMemoryMetricsCollector {
    async fn collect_metric(&self, metric: Metric) -> AnalyticsResult<()> {
        self.validate_metric(&metric).await?;

        let mut metrics = self.metrics.write().await;
        metrics.push(metric);

        // Apply retention policy
        let metrics_len = metrics.len();
        if metrics_len > self.config.max_metrics_per_collection {
            metrics.drain(0..metrics_len - self.config.max_metrics_per_collection);
        }

        Ok(())
    }

    async fn collect_metrics(&self, metrics: Vec<Metric>) -> AnalyticsResult<()> {
        for metric in metrics {
            self.collect_metric(metric).await?;
        }
        Ok(())
    }

    async fn get_metric(
        &self,
        name: &str,
        time_range: TimeRange,
        tags: Option<HashMap<String, String>>,
    ) -> AnalyticsResult<Vec<Metric>> {
        let metrics = self.metrics.read().await;
        let filtered_metrics: Vec<Metric> = metrics
            .iter()
            .filter(|m| {
                m.name == name
                    && m.timestamp >= time_range.start
                    && m.timestamp <= time_range.end
                    && tags.as_ref().map_or(true, |filter_tags| {
                        filter_tags.iter().all(|(k, v)| m.tags.get(k) == Some(v))
                    })
            })
            .cloned()
            .collect();

        Ok(filtered_metrics)
    }

    async fn get_aggregated_metrics(
        &self,
        name: &str,
        time_range: TimeRange,
        interval: AggregationInterval,
        function: AggregationFunction,
        tags: Option<HashMap<String, String>>,
    ) -> AnalyticsResult<Vec<AggregatedMetric>> {
        let metrics = self.get_metric(name, time_range, tags).await?;

        // Group metrics by time interval
        let mut grouped_metrics: HashMap<DateTime<Utc>, Vec<&Metric>> = HashMap::new();
        let interval_seconds = interval.duration_seconds() as i64;

        for metric in &metrics {
            let interval_start = DateTime::from_timestamp(
                (metric.timestamp.timestamp() / interval_seconds) * interval_seconds,
                0,
            )
            .unwrap_or(metric.timestamp);

            grouped_metrics
                .entry(interval_start)
                .or_default()
                .push(metric);
        }

        // Apply aggregation function
        let mut aggregated = Vec::new();
        for (timestamp, group_metrics) in grouped_metrics {
            if let Some(aggregated_value) =
                self.apply_aggregation_function(&group_metrics, function)
            {
                aggregated.push(AggregatedMetric {
                    name: name.to_string(),
                    timestamp,
                    interval,
                    function,
                    value: aggregated_value,
                    count: group_metrics.len() as u64,
                    tags: group_metrics
                        .first()
                        .map(|m| m.tags.clone())
                        .unwrap_or_default(),
                });
            }
        }

        aggregated.sort_by_key(|m| m.timestamp);
        Ok(aggregated)
    }

    async fn register_metric_definition(
        &self,
        definition: MetricDefinition,
    ) -> AnalyticsResult<()> {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.name.clone(), definition);
        Ok(())
    }

    async fn get_metric_definitions(&self) -> AnalyticsResult<Vec<MetricDefinition>> {
        let definitions = self.definitions.read().await;
        Ok(definitions.values().cloned().collect())
    }

    async fn delete_metrics(
        &self,
        name: &str,
        time_range: Option<TimeRange>,
        tags: Option<HashMap<String, String>>,
    ) -> AnalyticsResult<u64> {
        let mut metrics = self.metrics.write().await;
        let initial_len = metrics.len();

        metrics.retain(|m| {
            !(m.name == name
                && time_range
                    .as_ref()
                    .map_or(true, |tr| m.timestamp >= tr.start && m.timestamp <= tr.end)
                && tags.as_ref().map_or(true, |filter_tags| {
                    filter_tags.iter().all(|(k, v)| m.tags.get(k) == Some(v))
                }))
        });

        Ok((initial_len - metrics.len()) as u64)
    }

    async fn get_metric_statistics(&self, name: &str) -> AnalyticsResult<MetricStatistics> {
        let metrics = self.metrics.read().await;
        let filtered_metrics: Vec<&Metric> = metrics.iter().filter(|m| m.name == name).collect();

        if filtered_metrics.is_empty() {
            return Ok(MetricStatistics {
                name: name.to_string(),
                total_count: 0,
                first_timestamp: None,
                last_timestamp: None,
                unique_tag_combinations: 0,
                average_value: None,
                min_value: None,
                max_value: None,
                data_points_per_day: HashMap::new(),
            });
        }

        let total_count = filtered_metrics.len() as u64;
        let first_timestamp = filtered_metrics.iter().map(|m| m.timestamp).min();
        let last_timestamp = filtered_metrics.iter().map(|m| m.timestamp).max();

        // Calculate unique tag combinations
        let mut unique_tags = std::collections::HashSet::new();
        for metric in &filtered_metrics {
            let tag_string = format!("{:?}", metric.tags);
            unique_tags.insert(tag_string);
        }
        let unique_tag_combinations = unique_tags.len() as u64;

        // Calculate data points per day
        let mut data_points_per_day = HashMap::new();
        for metric in &filtered_metrics {
            let date = metric.timestamp.date_naive().to_string();
            *data_points_per_day.entry(date).or_insert(0) += 1;
        }

        Ok(MetricStatistics {
            name: name.to_string(),
            total_count,
            first_timestamp,
            last_timestamp,
            unique_tag_combinations,
            average_value: None, // Would need to implement based on metric values
            min_value: None,     // Would need to implement based on metric values
            max_value: None,     // Would need to implement based on metric values
            data_points_per_day,
        })
    }
}

impl InMemoryMetricsCollector {
    /// Apply aggregation function to a group of metrics
    fn apply_aggregation_function(
        &self,
        metrics: &[&Metric],
        function: AggregationFunction,
    ) -> Option<MetricValue> {
        if metrics.is_empty() {
            return None;
        }

        match function {
            AggregationFunction::Count => Some(MetricValue::Integer(metrics.len() as i64)),
            AggregationFunction::Sum => {
                let sum = metrics.iter().fold(0.0, |acc, m| {
                    acc + match &m.value {
                        MetricValue::Integer(i) => *i as f64,
                        MetricValue::Float(f) => *f,
                        MetricValue::Decimal(d) => d.to_f64().unwrap_or(0.0),
                        _ => 0.0,
                    }
                });
                Some(MetricValue::Float(sum))
            }
            AggregationFunction::Average => {
                let sum = metrics.iter().fold(0.0, |acc, m| {
                    acc + match &m.value {
                        MetricValue::Integer(i) => *i as f64,
                        MetricValue::Float(f) => *f,
                        MetricValue::Decimal(d) => d.to_f64().unwrap_or(0.0),
                        _ => 0.0,
                    }
                });
                Some(MetricValue::Float(sum / metrics.len() as f64))
            }
            _ => None, // Other functions not implemented in this example
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimeRange;

    #[tokio::test]
    async fn test_metrics_collector() {
        let config = MetricsConfig::default();
        let collector = InMemoryMetricsCollector::new(config);

        let metric = Metric {
            name: "test_metric".to_string(),
            value: MetricValue::Integer(100),
            timestamp: Utc::now(),
            tags: HashMap::new(),
        };

        collector.collect_metric(metric.clone()).await.unwrap();

        let time_range = TimeRange::last_hours(1);
        let retrieved_metrics = collector
            .get_metric("test_metric", time_range, None)
            .await
            .unwrap();

        assert_eq!(retrieved_metrics.len(), 1);
        assert_eq!(retrieved_metrics[0].name, "test_metric");
    }

    #[tokio::test]
    async fn test_metric_aggregation() {
        let config = MetricsConfig::default();
        let collector = InMemoryMetricsCollector::new(config);

        // Add multiple metrics
        for i in 1..=5 {
            let metric = Metric {
                name: "test_metric".to_string(),
                value: MetricValue::Integer(i * 10),
                timestamp: Utc::now(),
                tags: HashMap::new(),
            };
            collector.collect_metric(metric).await.unwrap();
        }

        let time_range = TimeRange::last_hours(1);
        let aggregated = collector
            .get_aggregated_metrics(
                "test_metric",
                time_range,
                AggregationInterval::Hour,
                AggregationFunction::Sum,
                None,
            )
            .await
            .unwrap();

        assert_eq!(aggregated.len(), 1);
        if let MetricValue::Float(sum) = aggregated[0].value {
            assert_eq!(sum, 150.0); // 10 + 20 + 30 + 40 + 50
        } else {
            panic!("Expected float value");
        }
    }
}
