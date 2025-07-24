// =====================================================================================
// File: core-analytics/src/types.rs
// Description: Core types for analytics system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc, Datelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub unit: String,
    pub tags: HashMap<String, String>,
    pub value: MetricValue,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

impl Metric {
    /// Create a new metric
    pub fn new<S: Into<String>>(
        name: S,
        metric_type: MetricType,
        value: MetricValue,
        source: S,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            metric_type,
            unit: String::new(),
            tags: HashMap::new(),
            value,
            timestamp: Utc::now(),
            source: source.into(),
        }
    }

    /// Add a tag to the metric
    pub fn with_tag<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Set the unit
    pub fn with_unit<S: Into<String>>(mut self, unit: S) -> Self {
        self.unit = unit.into();
        self
    }

    /// Set the description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = description.into();
        self
    }
}

/// Metric type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter that only increases
    Counter,
    /// Gauge that can go up and down
    Gauge,
    /// Histogram for distribution of values
    Histogram,
    /// Summary with quantiles
    Summary,
    /// Timer for measuring durations
    Timer,
    /// Set for counting unique values
    Set,
    /// Rate for measuring events per time unit
    Rate,
}

impl MetricType {
    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            MetricType::Counter => "Counter",
            MetricType::Gauge => "Gauge",
            MetricType::Histogram => "Histogram",
            MetricType::Summary => "Summary",
            MetricType::Timer => "Timer",
            MetricType::Set => "Set",
            MetricType::Rate => "Rate",
        }
    }

    /// Check if metric type supports aggregation
    pub fn supports_aggregation(&self) -> bool {
        matches!(
            self,
            MetricType::Counter | MetricType::Gauge | MetricType::Histogram | MetricType::Summary
        )
    }
}

/// Metric value enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    /// Floating point value
    Float(f64),
    /// Decimal value for precise calculations
    Decimal(Decimal),
    /// Boolean value
    Boolean(bool),
    /// String value
    String(String),
    /// Null value
    Null,
    /// Histogram buckets
    Histogram {
        buckets: Vec<HistogramBucket>,
        count: u64,
        sum: f64,
    },
    /// Summary with quantiles
    Summary {
        quantiles: Vec<Quantile>,
        count: u64,
        sum: f64,
    },
    /// Set of unique values
    Set(Vec<String>),
}

impl MetricValue {
    /// Convert to f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Integer(i) => Some(*i as f64),
            MetricValue::Float(f) => Some(*f),
            MetricValue::Decimal(d) => d.to_f64(),
            MetricValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            MetricValue::Null => None,
            MetricValue::Histogram { sum, .. } => Some(*sum),
            MetricValue::Summary { sum, .. } => Some(*sum),
            _ => None,
        }
    }

    /// Check if value is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            MetricValue::Integer(_) | MetricValue::Float(_) | MetricValue::Decimal(_)
        )
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, MetricValue::Null)
    }
}

/// Histogram bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Quantile for summary metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantile {
    pub quantile: f64,
    pub value: f64,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub tags: HashMap<String, String>,
}

/// Time series data collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub metric_name: String,
    pub points: Vec<TimeSeriesPoint>,
    pub metadata: TimeSeriesMetadata,
}

/// Time series metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesMetadata {
    pub unit: String,
    pub description: String,
    pub aggregation_method: Option<String>,
    pub resolution: Option<String>,
    pub data_source: String,
}

/// Analytics report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub report_type: ReportType,
    pub format: ReportFormat,
    pub sections: Vec<ReportSection>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub valid_until: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub file_size: Option<u64>,
}

/// Report type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportType {
    /// Daily operational report
    Daily,
    /// Weekly summary report
    Weekly,
    /// Monthly business report
    Monthly,
    /// Quarterly financial report
    Quarterly,
    /// Annual comprehensive report
    Annual,
    /// Ad-hoc custom report
    Custom,
    /// Summary report
    Summary,
    /// Detailed report
    Detailed,
    /// Real-time dashboard report
    RealTime,
    /// Compliance report
    Compliance,
    /// Risk assessment report
    Risk,
    /// Performance analysis report
    Performance,
}

/// Report format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// PDF document
    PDF,
    /// Excel spreadsheet
    Excel,
    /// CSV data file
    CSV,
    /// JSON data format
    JSON,
    /// HTML web page
    HTML,
    /// PowerPoint presentation
    PowerPoint,
    /// Interactive dashboard
    Dashboard,
}

/// Report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub id: String,
    pub title: String,
    pub section_type: SectionType,
    pub content: SectionContent,
    pub order: u32,
    pub visible: bool,
}

/// Section type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionType {
    /// Executive summary
    Summary,
    /// Key metrics overview
    Metrics,
    /// Charts and visualizations
    Charts,
    /// Data tables
    Tables,
    /// Text analysis
    Analysis,
    /// Recommendations
    Recommendations,
    /// Appendix with raw data
    Appendix,
    /// Raw data section
    Data,
}

/// Section content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SectionContent {
    /// Text content
    Text(String),
    /// Metrics data
    Metrics(Vec<Metric>),
    /// Chart specification
    Chart(ChartSpec),
    /// Data table
    Table(TableData),
    /// Mixed content
    Mixed(Vec<ContentItem>),
    /// JSON data
    Json(String),
}

/// Content item for mixed sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentItem {
    Text(String),
    Metric(Metric),
    Chart(ChartSpec),
    Table(TableData),
}

/// Chart specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSpec {
    pub chart_type: ChartType,
    pub title: String,
    pub data: ChartData,
    pub options: ChartOptions,
}

/// Chart type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    /// Line chart for time series
    Line,
    /// Bar chart for categorical data
    Bar,
    /// Pie chart for proportions
    Pie,
    /// Area chart for cumulative data
    Area,
    /// Scatter plot for correlations
    Scatter,
    /// Histogram for distributions
    Histogram,
    /// Heatmap for matrix data
    Heatmap,
    /// Candlestick for financial data
    Candlestick,
    /// Gauge for single values
    Gauge,
    /// Treemap for hierarchical data
    Treemap,
}

/// Chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub datasets: Vec<Dataset>,
    pub labels: Vec<String>,
}

/// Dataset for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: Option<String>,
    pub fill: Option<bool>,
}

/// Chart options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub show_legend: bool,
    pub show_grid: bool,
    pub x_axis_label: Option<String>,
    pub y_axis_label: Option<String>,
    pub custom_options: HashMap<String, serde_json::Value>,
}

/// Table data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: Option<usize>,
    pub pagination: Option<PaginationInfo>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub widgets: Vec<Widget>,
    pub layout: DashboardLayout,
    pub refresh_interval: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub shared: bool,
    pub permissions: Vec<DashboardPermission>,
}

/// Dashboard widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: Uuid,
    pub title: String,
    pub widget_type: WidgetType,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub configuration: WidgetConfiguration,
    pub data_source: String,
    pub refresh_interval: Option<u32>,
}

/// Widget type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetType {
    /// Single metric display
    Metric,
    /// Chart widget
    Chart,
    /// Data table widget
    Table,
    /// Text/markdown widget
    Text,
    /// KPI indicator
    KPI,
    /// Progress bar
    Progress,
    /// Alert list
    Alerts,
    /// Custom widget
    Custom,
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
pub struct WidgetConfiguration {
    pub query: Option<String>,
    pub chart_config: Option<ChartSpec>,
    pub display_options: HashMap<String, serde_json::Value>,
    pub thresholds: Option<Vec<Threshold>>,
}

/// Threshold for alerts and indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    pub value: f64,
    pub operator: String,
    pub color: String,
    pub label: String,
}

/// Dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub grid_size: GridSize,
    pub auto_arrange: bool,
    pub responsive: bool,
}

/// Grid size for dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSize {
    pub columns: u32,
    pub rows: u32,
    pub cell_width: u32,
    pub cell_height: u32,
}

/// Dashboard permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPermission {
    pub user_id: String,
    pub permission_type: PermissionType,
    pub granted_at: DateTime<Utc>,
    pub granted_by: String,
}

/// Permission type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionType {
    View,
    Edit,
    Admin,
    Share,
}

/// Time range for queries and data filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// Create a time range for the last N hours
    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::hours(hours);
        Self { start, end }
    }

    /// Create a time range for the last N days
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - chrono::Duration::days(days);
        Self { start, end }
    }

    /// Create a time range for today
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = now.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc();
        Self { start, end }
    }

    /// Create a time range for this week
    pub fn this_week() -> Self {
        let now = Utc::now();
        let days_since_monday = now.weekday().num_days_from_monday();
        let start = (now - chrono::Duration::days(days_since_monday as i64))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let end = start + chrono::Duration::days(6);
        Self { start, end }
    }

    /// Create a time range for this month
    pub fn this_month() -> Self {
        let now = Utc::now();
        let start = now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let end = if now.month() == 12 {
            chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap()
        } else {
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1).unwrap()
        }
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
            - chrono::Duration::seconds(1);
        Self { start, end }
    }

    /// Check if the time range contains a specific timestamp
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }

    /// Get the duration of the time range
    pub fn duration(&self) -> chrono::Duration {
        self.end - self.start
    }

    /// Check if the time range is valid (start <= end)
    pub fn is_valid(&self) -> bool {
        self.start <= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new(
            "transaction_count",
            MetricType::Counter,
            MetricValue::Integer(100),
            "api_server",
        )
        .with_tag("environment", "production")
        .with_unit("count")
        .with_description("Total number of transactions");

        assert_eq!(metric.name, "transaction_count");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.unit, "count");
        assert_eq!(
            metric.tags.get("environment"),
            Some(&"production".to_string())
        );
    }

    #[test]
    fn test_metric_value_conversion() {
        let int_value = MetricValue::Integer(42);
        assert_eq!(int_value.as_f64(), Some(42.0));
        assert!(int_value.is_numeric());

        let float_value = MetricValue::Float(3.14);
        assert_eq!(float_value.as_f64(), Some(3.14));
        assert!(float_value.is_numeric());

        let bool_value = MetricValue::Boolean(true);
        assert_eq!(bool_value.as_f64(), Some(1.0));
        assert!(!bool_value.is_numeric());

        let string_value = MetricValue::String("test".to_string());
        assert_eq!(string_value.as_f64(), None);
        assert!(!string_value.is_numeric());
    }

    #[test]
    fn test_metric_type_properties() {
        assert_eq!(MetricType::Counter.display_name(), "Counter");
        assert!(MetricType::Counter.supports_aggregation());

        assert_eq!(MetricType::Timer.display_name(), "Timer");
        assert!(!MetricType::Timer.supports_aggregation());
    }

    #[test]
    fn test_time_series_data_creation() {
        let points = vec![
            TimeSeriesPoint {
                timestamp: Utc::now(),
                value: 100.0,
                tags: HashMap::new(),
            },
            TimeSeriesPoint {
                timestamp: Utc::now(),
                value: 150.0,
                tags: HashMap::new(),
            },
        ];

        let time_series = TimeSeriesData {
            metric_name: "cpu_usage".to_string(),
            points,
            metadata: TimeSeriesMetadata {
                unit: "percent".to_string(),
                description: "CPU usage percentage".to_string(),
                aggregation_method: Some("average".to_string()),
                resolution: Some("1m".to_string()),
                data_source: "system_monitor".to_string(),
            },
        };

        assert_eq!(time_series.metric_name, "cpu_usage");
        assert_eq!(time_series.points.len(), 2);
        assert_eq!(time_series.metadata.unit, "percent");
    }

    #[test]
    fn test_chart_spec_creation() {
        let chart = ChartSpec {
            chart_type: ChartType::Line,
            title: "Transaction Volume".to_string(),
            data: ChartData {
                datasets: vec![Dataset {
                    label: "Volume".to_string(),
                    data: vec![100.0, 150.0, 200.0],
                    color: Some("#007bff".to_string()),
                    fill: Some(false),
                }],
                labels: vec!["Jan".to_string(), "Feb".to_string(), "Mar".to_string()],
            },
            options: ChartOptions {
                width: Some(800),
                height: Some(400),
                show_legend: true,
                show_grid: true,
                x_axis_label: Some("Month".to_string()),
                y_axis_label: Some("Volume".to_string()),
                custom_options: HashMap::new(),
            },
        };

        assert_eq!(chart.chart_type, ChartType::Line);
        assert_eq!(chart.title, "Transaction Volume");
        assert_eq!(chart.data.datasets.len(), 1);
        assert_eq!(chart.data.labels.len(), 3);
    }

    #[test]
    fn test_dashboard_creation() {
        let dashboard = Dashboard {
            id: Uuid::new_v4(),
            name: "Main Dashboard".to_string(),
            description: "Primary analytics dashboard".to_string(),
            widgets: vec![],
            layout: DashboardLayout {
                grid_size: GridSize {
                    columns: 12,
                    rows: 8,
                    cell_width: 100,
                    cell_height: 100,
                },
                auto_arrange: false,
                responsive: true,
            },
            refresh_interval: Some(60), // 1 minute
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: "admin".to_string(),
            shared: true,
            permissions: vec![],
        };

        assert_eq!(dashboard.name, "Main Dashboard");
        assert!(dashboard.shared);
        assert_eq!(dashboard.refresh_interval, Some(60));
        assert_eq!(dashboard.layout.grid_size.columns, 12);
    }
}
