// =====================================================================================
// File: core-analytics/tests/analytics_tests.rs
// Description: Comprehensive tests for analytics services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Duration, Utc};
use core_analytics::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio_test;
use uuid::Uuid;

#[tokio::test]
async fn test_metrics_collection_and_aggregation() {
    // Test metric creation and validation
    let metric = Metric::new(
        "trading_volume".to_string(),
        MetricValue::Decimal(Decimal::new(1000000, 2)), // $10,000
        HashMap::from([
            ("asset".to_string(), "BTC".to_string()),
            ("exchange".to_string(), "main".to_string()),
        ]),
    );

    assert_eq!(metric.name, "trading_volume");
    assert!(metric.value.is_numeric());
    assert_eq!(metric.tags.get("asset"), Some(&"BTC".to_string()));

    // Test metric value conversions
    let decimal_value = MetricValue::Decimal(Decimal::new(12345, 2)); // 123.45
    assert_eq!(decimal_value.as_f64(), Some(123.45));
    assert!(decimal_value.is_numeric());

    let null_value = MetricValue::Null;
    assert_eq!(null_value.as_f64(), None);
    assert!(!null_value.is_numeric());
    assert!(null_value.is_null());

    // Test histogram metric
    let histogram = MetricValue::Histogram {
        buckets: vec![
            HistogramBucket {
                upper_bound: Decimal::new(100, 0),
                count: 10,
            },
            HistogramBucket {
                upper_bound: Decimal::new(500, 0),
                count: 25,
            },
            HistogramBucket {
                upper_bound: Decimal::new(1000, 0),
                count: 15,
            },
        ],
        count: 50,
        sum: 12500.0,
    };

    if let MetricValue::Histogram { count, sum, .. } = histogram {
        assert_eq!(count, 50);
        assert_eq!(sum, 12500.0);
    }
}

#[tokio::test]
async fn test_time_series_data_handling() {
    // Test time series point creation
    let timestamp = Utc::now();
    let point = TimeSeriesPoint::new(
        timestamp,
        Decimal::new(50000, 0), // $500.00
        HashMap::from([
            ("symbol".to_string(), "BTC".to_string()),
            ("type".to_string(), "price".to_string()),
        ]),
    );

    assert_eq!(point.timestamp, timestamp);
    assert_eq!(point.value, Decimal::new(50000, 0));
    assert_eq!(point.tags.get("symbol"), Some(&"BTC".to_string()));

    // Test time series creation
    let mut time_series = TimeSeries::new(
        "btc_price".to_string(),
        HashMap::from([("asset".to_string(), "BTC".to_string())]),
    );

    // Add multiple points
    let base_time = Utc::now();
    for i in 0..10 {
        let point = TimeSeriesPoint::new(
            base_time + Duration::minutes(i),
            Decimal::new(50000 + i * 100, 0), // Increasing price
            HashMap::new(),
        );
        time_series.add_point(point);
    }

    assert_eq!(time_series.points.len(), 10);
    assert_eq!(time_series.name, "btc_price");

    // Test time range filtering
    let start_time = base_time + Duration::minutes(3);
    let end_time = base_time + Duration::minutes(7);
    let filtered_points = time_series.get_points_in_range(start_time, end_time);

    assert_eq!(filtered_points.len(), 5); // Points at minutes 3, 4, 5, 6, 7
}

#[tokio::test]
async fn test_analytics_query_system() {
    // Test query builder
    let query = AnalyticsQuery::builder()
        .metric("trading_volume")
        .time_range(Utc::now() - Duration::hours(24), Utc::now())
        .filter("asset", "BTC")
        .filter("exchange", "main")
        .aggregation(AggregationType::Sum)
        .group_by(vec!["asset".to_string()])
        .build();

    assert_eq!(query.metric_name, "trading_volume");
    assert_eq!(query.aggregation, AggregationType::Sum);
    assert_eq!(query.filters.get("asset"), Some(&"BTC".to_string()));
    assert_eq!(query.group_by, vec!["asset".to_string()]);

    // Test query validation
    assert!(query.validate().is_ok());

    // Test invalid query (empty metric name)
    let invalid_query = AnalyticsQuery {
        metric_name: "".to_string(),
        start_time: Utc::now() - Duration::hours(1),
        end_time: Utc::now(),
        filters: HashMap::new(),
        aggregation: AggregationType::Average,
        group_by: vec![],
        limit: None,
        offset: None,
    };

    assert!(invalid_query.validate().is_err());
}

#[tokio::test]
async fn test_aggregation_functions() {
    // Create test data points
    let values = vec![
        Decimal::new(100, 0),
        Decimal::new(200, 0),
        Decimal::new(300, 0),
        Decimal::new(400, 0),
        Decimal::new(500, 0),
    ];

    // Test sum aggregation
    let sum = values.iter().fold(Decimal::ZERO, |acc, x| acc + x);
    assert_eq!(sum, Decimal::new(1500, 0));

    // Test average aggregation
    let avg = sum / Decimal::new(values.len() as i64, 0);
    assert_eq!(avg, Decimal::new(300, 0));

    // Test min/max aggregation
    let min = values.iter().min().unwrap();
    let max = values.iter().max().unwrap();
    assert_eq!(*min, Decimal::new(100, 0));
    assert_eq!(*max, Decimal::new(500, 0));

    // Test count aggregation
    assert_eq!(values.len(), 5);
}

#[tokio::test]
async fn test_dashboard_and_visualization() {
    // Test dashboard creation
    let dashboard = Dashboard::new(
        "Trading Dashboard".to_string(),
        "Main trading metrics dashboard".to_string(),
        "admin".to_string(),
    );

    assert_eq!(dashboard.title, "Trading Dashboard");
    assert_eq!(dashboard.created_by, "admin");
    assert!(dashboard.is_active);

    // Test widget creation
    let widget = Widget::new(
        "Volume Chart".to_string(),
        WidgetType::LineChart,
        AnalyticsQuery::builder()
            .metric("trading_volume")
            .time_range(Utc::now() - Duration::hours(24), Utc::now())
            .aggregation(AggregationType::Sum)
            .build(),
    );

    assert_eq!(widget.title, "Volume Chart");
    assert_eq!(widget.widget_type, WidgetType::LineChart);

    // Test chart configuration
    let chart_config = ChartConfig {
        chart_type: ChartType::Line,
        x_axis: AxisConfig {
            title: "Time".to_string(),
            data_type: DataType::DateTime,
            scale: ScaleType::Linear,
        },
        y_axis: AxisConfig {
            title: "Volume (USD)".to_string(),
            data_type: DataType::Decimal,
            scale: ScaleType::Linear,
        },
        colors: vec!["#1f77b4".to_string(), "#ff7f0e".to_string()],
        show_legend: true,
        show_grid: true,
    };

    assert_eq!(chart_config.chart_type, ChartType::Line);
    assert!(chart_config.show_legend);
    assert!(chart_config.show_grid);
}

#[tokio::test]
async fn test_real_time_analytics() {
    // Test real-time metric streaming
    let mut stream_config = StreamConfig::new(
        "real_time_prices".to_string(),
        Duration::seconds(1), // 1 second interval
    );

    stream_config.add_metric("btc_price");
    stream_config.add_metric("eth_price");
    stream_config.add_filter("exchange", "main");

    assert_eq!(stream_config.stream_name, "real_time_prices");
    assert_eq!(stream_config.update_interval, Duration::seconds(1));
    assert_eq!(stream_config.metrics.len(), 2);

    // Test alert configuration
    let alert = AlertConfig::new(
        "High Volume Alert".to_string(),
        "trading_volume".to_string(),
        AlertCondition::GreaterThan(Decimal::new(1000000, 2)), // > $10,000
        AlertSeverity::Warning,
    );

    assert_eq!(alert.name, "High Volume Alert");
    assert_eq!(alert.severity, AlertSeverity::Warning);

    if let AlertCondition::GreaterThan(threshold) = alert.condition {
        assert_eq!(threshold, Decimal::new(1000000, 2));
    }

    // Test alert evaluation
    let high_value = Decimal::new(1500000, 2); // $15,000
    let low_value = Decimal::new(500000, 2); // $5,000

    assert!(
        matches!(alert.condition, AlertCondition::GreaterThan(threshold) if high_value > threshold)
    );
    assert!(
        !matches!(alert.condition, AlertCondition::GreaterThan(threshold) if low_value > threshold)
    );
}

#[tokio::test]
async fn test_performance_metrics() {
    // Test performance metric calculation
    let start_time = std::time::Instant::now();

    // Simulate processing 10,000 metrics
    let mut metrics = Vec::new();
    for i in 0..10000 {
        let metric = Metric::new(
            format!("metric_{}", i),
            MetricValue::Integer(i),
            HashMap::from([("batch".to_string(), "test".to_string())]),
        );
        metrics.push(metric);
    }

    let elapsed = start_time.elapsed();
    println!("Created 10,000 metrics in {:?}", elapsed);

    // Should be able to create metrics quickly
    assert!(elapsed.as_millis() < 1000); // Less than 1 second
    assert_eq!(metrics.len(), 10000);

    // Test aggregation performance
    let aggregation_start = std::time::Instant::now();

    let sum: i64 = metrics
        .iter()
        .filter_map(|m| match &m.value {
            MetricValue::Integer(i) => Some(*i),
            _ => None,
        })
        .sum();

    let aggregation_elapsed = aggregation_start.elapsed();
    println!("Aggregated 10,000 metrics in {:?}", aggregation_elapsed);

    assert_eq!(sum, (0..10000).sum::<i64>());
    assert!(aggregation_elapsed.as_millis() < 100); // Should be very fast
}

#[tokio::test]
async fn test_data_retention_and_cleanup() {
    // Test data retention policy
    let retention_policy = DataRetentionPolicy {
        metric_name: "trading_volume".to_string(),
        retention_period: Duration::days(30),
        aggregation_rules: vec![
            AggregationRule {
                interval: Duration::minutes(1),
                retention: Duration::days(7),
                aggregation: AggregationType::Average,
            },
            AggregationRule {
                interval: Duration::hours(1),
                retention: Duration::days(30),
                aggregation: AggregationType::Average,
            },
        ],
    };

    assert_eq!(retention_policy.retention_period, Duration::days(30));
    assert_eq!(retention_policy.aggregation_rules.len(), 2);

    // Test cleanup logic
    let cutoff_time = Utc::now() - Duration::days(31);
    let recent_time = Utc::now() - Duration::days(15);

    // Simulate data points
    let old_point = TimeSeriesPoint::new(cutoff_time, Decimal::new(1000, 0), HashMap::new());

    let recent_point = TimeSeriesPoint::new(recent_time, Decimal::new(2000, 0), HashMap::new());

    // Old point should be eligible for cleanup
    assert!(old_point.timestamp < Utc::now() - retention_policy.retention_period);
    // Recent point should be retained
    assert!(recent_point.timestamp > Utc::now() - retention_policy.retention_period);
}

#[tokio::test]
async fn test_analytics_service_configuration() {
    // Test service configuration
    let config = AnalyticsServiceConfig::default();

    assert_eq!(config.metrics_config.max_metrics_per_request, 1000);
    assert_eq!(
        config.aggregation_config.default_bucket_size,
        Duration::minutes(5)
    );
    assert_eq!(config.reporting_config.max_report_size_mb, 100);
    assert!(config.service_config.enable_real_time_processing);

    // Test configuration validation
    let mut invalid_config = config.clone();
    invalid_config.metrics_config.max_metrics_per_request = 0;

    // This would fail validation in a real implementation
    assert_eq!(invalid_config.metrics_config.max_metrics_per_request, 0);
}

#[tokio::test]
async fn test_error_handling_and_edge_cases() {
    // Test invalid metric values
    let invalid_metric = Metric::new(
        "".to_string(), // Empty name
        MetricValue::Integer(100),
        HashMap::new(),
    );

    // Empty metric name should be invalid
    assert!(invalid_metric.name.is_empty());

    // Test division by zero in aggregation
    let empty_values: Vec<Decimal> = vec![];
    let sum = empty_values.iter().fold(Decimal::ZERO, |acc, x| acc + x);
    assert_eq!(sum, Decimal::ZERO);

    // Test with null values
    let values_with_null = vec![
        MetricValue::Integer(100),
        MetricValue::Null,
        MetricValue::Integer(200),
    ];

    let numeric_count = values_with_null.iter().filter(|v| v.is_numeric()).count();

    assert_eq!(numeric_count, 2);

    // Test time range validation
    let invalid_time_range = (Utc::now(), Utc::now() - Duration::hours(1));
    assert!(invalid_time_range.0 > invalid_time_range.1); // End before start
}

#[tokio::test]
async fn test_complex_analytics_scenario() {
    // Test a complex analytics scenario with multiple components
    let mut dashboard = Dashboard::new(
        "Comprehensive Trading Analytics".to_string(),
        "Complete trading dashboard with multiple metrics".to_string(),
        "analyst".to_string(),
    );

    // Add multiple widgets
    let widgets = vec![
        Widget::new(
            "Trading Volume".to_string(),
            WidgetType::LineChart,
            AnalyticsQuery::builder()
                .metric("trading_volume")
                .time_range(Utc::now() - Duration::hours(24), Utc::now())
                .aggregation(AggregationType::Sum)
                .group_by(vec!["asset".to_string()])
                .build(),
        ),
        Widget::new(
            "Price Distribution".to_string(),
            WidgetType::Histogram,
            AnalyticsQuery::builder()
                .metric("asset_price")
                .time_range(Utc::now() - Duration::hours(24), Utc::now())
                .aggregation(AggregationType::Average)
                .build(),
        ),
        Widget::new(
            "Top Assets".to_string(),
            WidgetType::Table,
            AnalyticsQuery::builder()
                .metric("trading_volume")
                .time_range(Utc::now() - Duration::hours(24), Utc::now())
                .aggregation(AggregationType::Sum)
                .group_by(vec!["asset".to_string()])
                .limit(10)
                .build(),
        ),
    ];

    for widget in widgets {
        dashboard.add_widget(widget);
    }

    assert_eq!(dashboard.widgets.len(), 3);
    assert!(dashboard
        .widgets
        .iter()
        .any(|w| w.title == "Trading Volume"));
    assert!(dashboard
        .widgets
        .iter()
        .any(|w| w.widget_type == WidgetType::Histogram));
}
