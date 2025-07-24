// =====================================================================================
// File: core-analytics/src/aggregation.rs
// Description: Data aggregation engine for analytics
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{types::*, AnalyticsError, AnalyticsResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    /// Enable parallel processing
    pub parallel_processing: bool,
    /// Maximum parallel workers
    pub max_workers: usize,
    /// Batch size for aggregation
    pub batch_size: usize,
    /// Memory limit for aggregation operations (MB)
    pub memory_limit_mb: usize,
    /// Timeout for aggregation operations (seconds)
    pub timeout_seconds: u64,
    /// Enable result caching
    pub cache_results: bool,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable incremental aggregation
    pub incremental_aggregation: bool,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            parallel_processing: true,
            max_workers: num_cpus::get(),
            batch_size: 10000,
            memory_limit_mb: 1024,
            timeout_seconds: 300,
            cache_results: true,
            cache_ttl_seconds: 3600,
            incremental_aggregation: true,
        }
    }
}

/// Aggregation type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum aggregation
    Sum,
    /// Average aggregation
    Average,
    /// Count aggregation
    Count,
    /// Count distinct aggregation
    CountDistinct,
    /// Minimum value aggregation
    Min,
    /// Maximum value aggregation
    Max,
    /// Median aggregation
    Median,
    /// Percentile aggregation
    Percentile(u8),
    /// Standard deviation aggregation
    StandardDeviation,
    /// Variance aggregation
    Variance,
    /// First value aggregation
    First,
    /// Last value aggregation
    Last,
    /// Range aggregation (max - min)
    Range,
    /// Mode aggregation (most frequent value)
    Mode,
    /// Geometric mean aggregation
    GeometricMean,
    /// Harmonic mean aggregation
    HarmonicMean,
}

impl AggregationType {
    /// Check if aggregation type requires numeric values
    pub fn requires_numeric(&self) -> bool {
        matches!(
            self,
            AggregationType::Sum
                | AggregationType::Average
                | AggregationType::Min
                | AggregationType::Max
                | AggregationType::Median
                | AggregationType::Percentile(_)
                | AggregationType::StandardDeviation
                | AggregationType::Variance
                | AggregationType::Range
                | AggregationType::GeometricMean
                | AggregationType::HarmonicMean
        )
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            AggregationType::Sum => "Sum of all values",
            AggregationType::Average => "Average of all values",
            AggregationType::Count => "Count of all values",
            AggregationType::CountDistinct => "Count of distinct values",
            AggregationType::Min => "Minimum value",
            AggregationType::Max => "Maximum value",
            AggregationType::Median => "Median value",
            AggregationType::Percentile(p) => match p {
                50 => "50th percentile (median)",
                90 => "90th percentile",
                95 => "95th percentile",
                99 => "99th percentile",
                _ => "Percentile value",
            },
            AggregationType::StandardDeviation => "Standard deviation",
            AggregationType::Variance => "Variance",
            AggregationType::First => "First value",
            AggregationType::Last => "Last value",
            AggregationType::Range => "Range (max - min)",
            AggregationType::Mode => "Most frequent value",
            AggregationType::GeometricMean => "Geometric mean",
            AggregationType::HarmonicMean => "Harmonic mean",
        }
    }
}

/// Aggregation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationSpec {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub aggregation_type: AggregationType,
    pub field: String,
    pub group_by: Vec<String>,
    pub filters: Vec<AggregationFilter>,
    pub time_window: Option<TimeWindow>,
    pub output_field: Option<String>,
    pub weight_field: Option<String>,
    pub created_at: DateTime<Utc>,
    pub enabled: bool,
}

/// Aggregation filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
    pub case_sensitive: bool,
}

/// Filter operator enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    In,
    NotIn,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Between,
    IsNull,
    IsNotNull,
    Regex,
}

/// Time window for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub duration_seconds: u64,
    pub slide_seconds: Option<u64>,
    pub alignment: WindowAlignment,
}

/// Window alignment enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowAlignment {
    Start,
    End,
    Center,
}

/// Aggregation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub spec_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub groups: Vec<GroupResult>,
    pub execution_time_ms: u64,
    pub processed_records: u64,
    pub cache_hit: bool,
}

/// Group result for aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResult {
    pub group_key: String,
    pub aggregated_value: MetricValue,
    pub record_count: u64,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Aggregation engine trait
#[async_trait]
pub trait AggregationEngine: Send + Sync {
    /// Execute aggregation specification
    async fn execute_aggregation(
        &self,
        spec: &AggregationSpec,
        data: Vec<HashMap<String, serde_json::Value>>,
    ) -> AnalyticsResult<AggregationResult>;

    /// Execute multiple aggregations in batch
    async fn execute_batch_aggregation(
        &self,
        specs: Vec<AggregationSpec>,
        data: Vec<HashMap<String, serde_json::Value>>,
    ) -> AnalyticsResult<Vec<AggregationResult>>;

    /// Execute streaming aggregation
    async fn execute_streaming_aggregation(
        &self,
        spec: &AggregationSpec,
        data_stream: tokio::sync::mpsc::Receiver<HashMap<String, serde_json::Value>>,
    ) -> AnalyticsResult<tokio::sync::mpsc::Receiver<AggregationResult>>;

    /// Get aggregation statistics
    async fn get_aggregation_statistics(
        &self,
        spec_id: Uuid,
    ) -> AnalyticsResult<AggregationStatistics>;
}

/// Aggregation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationStatistics {
    pub spec_id: Uuid,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub total_records_processed: u64,
    pub cache_hit_rate: f64,
    pub last_execution: Option<DateTime<Utc>>,
}

/// In-memory aggregation engine implementation
pub struct InMemoryAggregationEngine {
    config: AggregationConfig,
    statistics: tokio::sync::RwLock<HashMap<Uuid, AggregationStatistics>>,
    cache: tokio::sync::RwLock<HashMap<String, (AggregationResult, DateTime<Utc>)>>,
}

impl InMemoryAggregationEngine {
    pub fn new(config: AggregationConfig) -> Self {
        Self {
            config,
            statistics: tokio::sync::RwLock::new(HashMap::new()),
            cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Apply filters to data
    fn apply_filters(
        &self,
        data: &[HashMap<String, serde_json::Value>],
        filters: &[AggregationFilter],
    ) -> Vec<HashMap<String, serde_json::Value>> {
        data.iter()
            .filter(|record| {
                filters
                    .iter()
                    .all(|filter| self.apply_filter(record, filter))
            })
            .cloned()
            .collect()
    }

    /// Apply a single filter to a record
    fn apply_filter(
        &self,
        record: &HashMap<String, serde_json::Value>,
        filter: &AggregationFilter,
    ) -> bool {
        let field_value = record.get(&filter.field);

        match filter.operator {
            FilterOperator::Equal => field_value == Some(&filter.value),
            FilterOperator::NotEqual => field_value != Some(&filter.value),
            FilterOperator::IsNull => {
                field_value.is_none() || field_value == Some(&serde_json::Value::Null)
            }
            FilterOperator::IsNotNull => {
                field_value.is_some() && field_value != Some(&serde_json::Value::Null)
            }
            FilterOperator::GreaterThan => {
                if let (Some(field_val), Some(filter_val)) =
                    (field_value.and_then(|v| v.as_f64()), filter.value.as_f64())
                {
                    field_val > filter_val
                } else {
                    false
                }
            }
            FilterOperator::LessThan => {
                if let (Some(field_val), Some(filter_val)) =
                    (field_value.and_then(|v| v.as_f64()), filter.value.as_f64())
                {
                    field_val < filter_val
                } else {
                    false
                }
            }
            FilterOperator::Contains => {
                if let (Some(field_str), Some(filter_str)) =
                    (field_value.and_then(|v| v.as_str()), filter.value.as_str())
                {
                    if filter.case_sensitive {
                        field_str.contains(filter_str)
                    } else {
                        field_str
                            .to_lowercase()
                            .contains(&filter_str.to_lowercase())
                    }
                } else {
                    false
                }
            }
            FilterOperator::In => {
                if let Some(filter_array) = filter.value.as_array() {
                    filter_array.contains(field_value.unwrap_or(&serde_json::Value::Null))
                } else {
                    false
                }
            }
            _ => false, // Other operators not implemented in this example
        }
    }

    /// Group data by specified fields
    fn group_data(
        &self,
        data: &[HashMap<String, serde_json::Value>],
        group_by: &[String],
    ) -> HashMap<String, Vec<HashMap<String, serde_json::Value>>> {
        let mut groups = HashMap::new();

        for record in data {
            let mut group_key_parts = Vec::new();
            for field in group_by {
                let value = record
                    .get(field)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                group_key_parts.push(format!("{}:{}", field, value));
            }
            let group_key = group_key_parts.join("|");

            groups
                .entry(group_key)
                .or_insert_with(Vec::new)
                .push(record.clone());
        }

        groups
    }

    /// Apply aggregation function to a group of records
    fn apply_aggregation(
        &self,
        records: &[HashMap<String, serde_json::Value>],
        aggregation_type: AggregationType,
        field: &str,
    ) -> AnalyticsResult<MetricValue> {
        if records.is_empty() {
            return Ok(MetricValue::Null);
        }

        match aggregation_type {
            AggregationType::Count => Ok(MetricValue::Integer(records.len() as i64)),
            AggregationType::CountDistinct => {
                let mut unique_values = std::collections::HashSet::new();
                for record in records {
                    if let Some(value) = record.get(field) {
                        unique_values.insert(value);
                    }
                }
                Ok(MetricValue::Integer(unique_values.len() as i64))
            }
            AggregationType::Sum => {
                let sum = records.iter().fold(0.0, |acc, record| {
                    acc + record.get(field).and_then(|v| v.as_f64()).unwrap_or(0.0)
                });
                Ok(MetricValue::Float(sum))
            }
            AggregationType::Average => {
                let sum = records.iter().fold(0.0, |acc, record| {
                    acc + record.get(field).and_then(|v| v.as_f64()).unwrap_or(0.0)
                });
                Ok(MetricValue::Float(sum / records.len() as f64))
            }
            AggregationType::Min => {
                let min = records
                    .iter()
                    .filter_map(|record| record.get(field).and_then(|v| v.as_f64()))
                    .fold(f64::INFINITY, f64::min);

                if min.is_finite() {
                    Ok(MetricValue::Float(min))
                } else {
                    Ok(MetricValue::Null)
                }
            }
            AggregationType::Max => {
                let max = records
                    .iter()
                    .filter_map(|record| record.get(field).and_then(|v| v.as_f64()))
                    .fold(f64::NEG_INFINITY, f64::max);

                if max.is_finite() {
                    Ok(MetricValue::Float(max))
                } else {
                    Ok(MetricValue::Null)
                }
            }
            AggregationType::First => Ok(records
                .first()
                .and_then(|record| record.get(field))
                .map(|v| match v {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            MetricValue::Integer(i)
                        } else if let Some(f) = n.as_f64() {
                            MetricValue::Float(f)
                        } else {
                            MetricValue::String(v.to_string())
                        }
                    }
                    serde_json::Value::String(s) => MetricValue::String(s.clone()),
                    serde_json::Value::Bool(b) => MetricValue::Boolean(*b),
                    _ => MetricValue::String(v.to_string()),
                })
                .unwrap_or(MetricValue::Null)),
            AggregationType::Last => Ok(records
                .last()
                .and_then(|record| record.get(field))
                .map(|v| match v {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            MetricValue::Integer(i)
                        } else if let Some(f) = n.as_f64() {
                            MetricValue::Float(f)
                        } else {
                            MetricValue::String(v.to_string())
                        }
                    }
                    serde_json::Value::String(s) => MetricValue::String(s.clone()),
                    serde_json::Value::Bool(b) => MetricValue::Boolean(*b),
                    _ => MetricValue::String(v.to_string()),
                })
                .unwrap_or(MetricValue::Null)),
            _ => Err(AnalyticsError::aggregation_error(format!(
                "Aggregation type {:?} not implemented",
                aggregation_type
            ))),
        }
    }

    /// Generate cache key for aggregation
    fn generate_cache_key(&self, spec: &AggregationSpec, data_hash: u64) -> String {
        format!("agg_{}_{}", spec.id, data_hash)
    }

    /// Update aggregation statistics
    async fn update_statistics(
        &self,
        spec_id: Uuid,
        execution_time_ms: u64,
        records_processed: u64,
        success: bool,
        cache_hit: bool,
    ) {
        let mut stats = self.statistics.write().await;
        let stat = stats
            .entry(spec_id)
            .or_insert_with(|| AggregationStatistics {
                spec_id,
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                average_execution_time_ms: 0.0,
                total_records_processed: 0,
                cache_hit_rate: 0.0,
                last_execution: None,
            });

        stat.total_executions += 1;
        if success {
            stat.successful_executions += 1;
        } else {
            stat.failed_executions += 1;
        }

        stat.average_execution_time_ms = (stat.average_execution_time_ms
            * (stat.total_executions - 1) as f64
            + execution_time_ms as f64)
            / stat.total_executions as f64;
        stat.total_records_processed += records_processed;

        let cache_hits = if cache_hit { 1 } else { 0 };
        stat.cache_hit_rate = (stat.cache_hit_rate * (stat.total_executions - 1) as f64
            + cache_hits as f64)
            / stat.total_executions as f64;
        stat.last_execution = Some(Utc::now());
    }
}

#[async_trait]
impl AggregationEngine for InMemoryAggregationEngine {
    async fn execute_aggregation(
        &self,
        spec: &AggregationSpec,
        data: Vec<HashMap<String, serde_json::Value>>,
    ) -> AnalyticsResult<AggregationResult> {
        let start_time = std::time::Instant::now();

        // Check cache if enabled
        if self.config.cache_results {
            let data_hash = fxhash::hash64(&format!("{:?}", data));
            let cache_key = self.generate_cache_key(spec, data_hash);

            let cache = self.cache.read().await;
            if let Some((cached_result, cached_at)) = cache.get(&cache_key) {
                let cache_age = Utc::now().signed_duration_since(*cached_at);
                if cache_age.num_seconds() < self.config.cache_ttl_seconds as i64 {
                    self.update_statistics(spec.id, 0, data.len() as u64, true, true)
                        .await;
                    return Ok(cached_result.clone());
                }
            }
        }

        // Apply filters
        let filtered_data = self.apply_filters(&data, &spec.filters);

        // Group data
        let filtered_data_len = filtered_data.len();
        let groups = if spec.group_by.is_empty() {
            let mut single_group = HashMap::new();
            single_group.insert("default".to_string(), filtered_data);
            single_group
        } else {
            self.group_data(&filtered_data, &spec.group_by)
        };

        // Apply aggregation to each group
        let mut group_results = Vec::new();
        for (group_key, group_records) in groups {
            let aggregated_value =
                self.apply_aggregation(&group_records, spec.aggregation_type, &spec.field)?;

            group_results.push(GroupResult {
                group_key,
                aggregated_value,
                record_count: group_records.len() as u64,
                metadata: None,
            });
        }

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        let result = AggregationResult {
            spec_id: spec.id,
            timestamp: Utc::now(),
            groups: group_results,
            execution_time_ms,
            processed_records: filtered_data_len as u64,
            cache_hit: false,
        };

        // Cache result if enabled
        if self.config.cache_results {
            let data_hash = fxhash::hash64(&format!("{:?}", data));
            let cache_key = self.generate_cache_key(spec, data_hash);
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, (result.clone(), Utc::now()));
        }

        self.update_statistics(spec.id, execution_time_ms, data.len() as u64, true, false)
            .await;

        Ok(result)
    }

    async fn execute_batch_aggregation(
        &self,
        specs: Vec<AggregationSpec>,
        data: Vec<HashMap<String, serde_json::Value>>,
    ) -> AnalyticsResult<Vec<AggregationResult>> {
        let mut results = Vec::new();

        for spec in specs {
            let result = self.execute_aggregation(&spec, data.clone()).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn execute_streaming_aggregation(
        &self,
        _spec: &AggregationSpec,
        _data_stream: tokio::sync::mpsc::Receiver<HashMap<String, serde_json::Value>>,
    ) -> AnalyticsResult<tokio::sync::mpsc::Receiver<AggregationResult>> {
        // Streaming aggregation not implemented in this example
        Err(AnalyticsError::aggregation_error(
            "Streaming aggregation not implemented",
        ))
    }

    async fn get_aggregation_statistics(
        &self,
        spec_id: Uuid,
    ) -> AnalyticsResult<AggregationStatistics> {
        let stats = self.statistics.read().await;
        stats.get(&spec_id).cloned().ok_or_else(|| {
            AnalyticsError::metric_not_found(format!("aggregation_stats_{}", spec_id))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aggregation_engine() {
        let config = AggregationConfig::default();
        let engine = InMemoryAggregationEngine::new(config);

        let spec = AggregationSpec {
            id: Uuid::new_v4(),
            name: "test_aggregation".to_string(),
            description: "Test aggregation".to_string(),
            aggregation_type: AggregationType::Sum,
            field: "value".to_string(),
            group_by: vec![],
            filters: vec![],
            time_window: None,
            output_field: None,
            weight_field: None,
            created_at: Utc::now(),
            enabled: true,
        };

        let data = vec![
            {
                let mut record = HashMap::new();
                record.insert("value".to_string(), serde_json::json!(10));
                record
            },
            {
                let mut record = HashMap::new();
                record.insert("value".to_string(), serde_json::json!(20));
                record
            },
            {
                let mut record = HashMap::new();
                record.insert("value".to_string(), serde_json::json!(30));
                record
            },
        ];

        let result = engine.execute_aggregation(&spec, data).await.unwrap();

        assert_eq!(result.groups.len(), 1);
        if let MetricValue::Float(sum) = result.groups[0].aggregated_value {
            assert_eq!(sum, 60.0);
        } else {
            panic!("Expected float value");
        }
    }
}
