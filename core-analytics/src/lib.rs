// =====================================================================================
// File: core-analytics/src/lib.rs
// Description: Data analytics and reporting system for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Analytics Module
//!
//! This module provides comprehensive data analytics and reporting functionality for the
//! StableRWA platform, including real-time metrics, historical analysis, predictive
//! modeling, and automated report generation.

pub mod aggregation;
pub mod error;
pub mod metrics;
pub mod reporting;
pub mod service;
pub mod types;

// Re-export main types and traits
pub use aggregation::{AggregationEngine, AggregationType, InMemoryAggregationEngine};
pub use error::{AnalyticsError, AnalyticsResult};
pub use metrics::{InMemoryMetricsCollector, MetricDefinition, MetricsCollector};
pub use reporting::{InMemoryReportGenerator, ReportGenerator, ReportTemplate};
pub use service::{AnalyticsService, DefaultAnalyticsService};
pub use types::{
    AnalyticsReport, Dashboard, Metric, MetricType, MetricValue, ReportFormat, ReportType,
    TimeSeriesData, Widget, WidgetType,
};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Main analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Metrics collection configuration
    pub metrics_config: metrics::MetricsConfig,
    /// Data aggregation configuration
    pub aggregation_config: aggregation::AggregationConfig,
    /// Reporting configuration
    pub reporting_config: reporting::ReportingConfig,
    /// Data retention settings
    pub data_retention: DataRetentionConfig,
    /// Real-time processing settings
    pub real_time_processing: RealTimeConfig,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            metrics_config: metrics::MetricsConfig::default(),
            aggregation_config: aggregation::AggregationConfig::default(),
            reporting_config: reporting::ReportingConfig::default(),
            data_retention: DataRetentionConfig::default(),
            real_time_processing: RealTimeConfig::default(),
        }
    }
}

/// Data retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionConfig {
    /// Raw data retention period in days
    pub raw_data_retention_days: u32,
    /// Aggregated data retention period in days
    pub aggregated_data_retention_days: u32,
    /// Report retention period in days
    pub report_retention_days: u32,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
    /// Cleanup schedule (cron expression)
    pub cleanup_schedule: String,
    /// Archive old data instead of deleting
    pub archive_old_data: bool,
    /// Archive storage location
    pub archive_location: Option<String>,
}

impl Default for DataRetentionConfig {
    fn default() -> Self {
        Self {
            raw_data_retention_days: 90,          // 3 months
            aggregated_data_retention_days: 1095, // 3 years
            report_retention_days: 2555,          // 7 years
            auto_cleanup: true,
            cleanup_schedule: "0 2 * * *".to_string(), // Daily at 2 AM
            archive_old_data: true,
            archive_location: Some("s3://analytics-archive/".to_string()),
        }
    }
}

/// Real-time processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeConfig {
    /// Enable real-time processing
    pub enabled: bool,
    /// Processing window size in seconds
    pub window_size_seconds: u64,
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Processing timeout in seconds
    pub processing_timeout_seconds: u64,
    /// Enable streaming analytics
    pub streaming_analytics: bool,
    /// Stream buffer size
    pub stream_buffer_size: usize,
}

impl Default for RealTimeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_size_seconds: 60, // 1 minute windows
            max_batch_size: 1000,
            processing_timeout_seconds: 30,
            streaming_analytics: true,
            stream_buffer_size: 10000,
        }
    }
}

/// Analytics query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsQuery {
    pub id: Uuid,
    pub query_type: QueryType,
    pub metrics: Vec<String>,
    pub filters: Vec<QueryFilter>,
    pub time_range: types::TimeRange,
    pub aggregation: Option<aggregation::AggregationType>,
    pub group_by: Vec<String>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Query type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryType {
    /// Real-time data query
    RealTime,
    /// Historical data query
    Historical,
    /// Aggregated data query
    Aggregated,
    /// Comparative analysis query
    Comparative,
    /// Trend analysis query
    Trend,
    /// Forecast query
    Forecast,
}

/// Query filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
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
    StartsWith,
    EndsWith,
    Between,
    IsNull,
    IsNotNull,
}

// TimeRange is now defined in types module

/// Order by specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub field: String,
    pub direction: OrderDirection,
}

/// Order direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderDirection {
    Ascending,
    Descending,
}

/// Analytics query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query_id: Uuid,
    pub data: Vec<HashMap<String, serde_json::Value>>,
    pub metadata: QueryMetadata,
    pub execution_time_ms: u64,
    pub total_rows: usize,
    pub has_more: bool,
}

/// Query metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    pub columns: Vec<ColumnInfo>,
    pub data_sources: Vec<String>,
    pub cache_hit: bool,
    pub query_plan: Option<String>,
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub description: Option<String>,
}

/// Data type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    String,
    Integer,
    Float,
    Decimal,
    Boolean,
    DateTime,
    Json,
    Array,
}

/// Analytics alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric: String,
    pub condition: AlertCondition,
    pub threshold: Decimal,
    pub time_window_minutes: u32,
    pub notification_channels: Vec<NotificationChannel>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Alert condition enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    PercentageChange,
    StandardDeviation,
    Anomaly,
}

/// Notification channel enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email,
    Slack,
    Webhook,
    SMS,
    Dashboard,
}

/// Analytics event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: Uuid,
    pub event_type: String,
    pub entity_id: Option<String>,
    pub entity_type: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub source: String,
}

impl AnalyticsEvent {
    /// Create a new analytics event
    pub fn new(event_type: String, source: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            entity_id: None,
            entity_type: None,
            properties: HashMap::new(),
            timestamp: Utc::now(),
            user_id: None,
            session_id: None,
            source,
        }
    }

    /// Add a property to the event
    pub fn with_property<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Set the entity information
    pub fn with_entity<S: Into<String>>(mut self, entity_type: S, entity_id: S) -> Self {
        self.entity_type = Some(entity_type.into());
        self.entity_id = Some(entity_id.into());
        self
    }

    /// Set the user information
    pub fn with_user<S: Into<String>>(mut self, user_id: S, session_id: Option<S>) -> Self {
        self.user_id = Some(user_id.into());
        self.session_id = session_id.map(|s| s.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_config_default() {
        let config = AnalyticsConfig::default();
        assert!(config.real_time_processing.enabled);
        assert_eq!(config.data_retention.raw_data_retention_days, 90);
        assert!(config.data_retention.auto_cleanup);
    }

    #[test]
    fn test_time_range_creation() {
        let last_7_days = types::TimeRange::last_days(7);
        assert!(last_7_days.end > last_7_days.start);

        let duration = last_7_days.end - last_7_days.start;
        assert_eq!(duration.num_days(), 7);

        let today = types::TimeRange::today();
        assert!(today.end >= today.start);
    }

    #[test]
    fn test_analytics_event_creation() {
        let event = AnalyticsEvent::new("user_login".to_string(), "web_app".to_string())
            .with_property("ip_address", "192.168.1.1")
            .with_property("user_agent", "Mozilla/5.0")
            .with_entity("user", "user123")
            .with_user("user123", Some("session456"));

        assert_eq!(event.event_type, "user_login");
        assert_eq!(event.source, "web_app");
        assert_eq!(event.entity_type, Some("user".to_string()));
        assert_eq!(event.entity_id, Some("user123".to_string()));
        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.session_id, Some("session456".to_string()));
        assert_eq!(event.properties.len(), 2);
    }

    #[test]
    fn test_query_filter_creation() {
        let filter = QueryFilter {
            field: "amount".to_string(),
            operator: FilterOperator::GreaterThan,
            value: serde_json::json!(1000),
        };

        assert_eq!(filter.field, "amount");
        assert_eq!(filter.operator, FilterOperator::GreaterThan);
    }

    #[test]
    fn test_alert_config_creation() {
        let alert = AlertConfig {
            id: Uuid::new_v4(),
            name: "High Transaction Volume".to_string(),
            description: "Alert when transaction volume exceeds threshold".to_string(),
            metric: "transaction_volume".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: Decimal::new(100000000, 2), // $1,000,000
            time_window_minutes: 60,
            notification_channels: vec![NotificationChannel::Email, NotificationChannel::Slack],
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(alert.condition, AlertCondition::GreaterThan);
        assert_eq!(alert.notification_channels.len(), 2);
        assert!(alert.enabled);
    }

    #[test]
    fn test_data_retention_config_default() {
        let retention = DataRetentionConfig::default();
        assert_eq!(retention.raw_data_retention_days, 90);
        assert_eq!(retention.aggregated_data_retention_days, 1095);
        assert_eq!(retention.report_retention_days, 2555);
        assert!(retention.auto_cleanup);
        assert!(retention.archive_old_data);
    }

    #[test]
    fn test_real_time_config_default() {
        let real_time = RealTimeConfig::default();
        assert!(real_time.enabled);
        assert_eq!(real_time.window_size_seconds, 60);
        assert_eq!(real_time.max_batch_size, 1000);
        assert!(real_time.streaming_analytics);
    }
}
