// =====================================================================================
// File: core-monitoring/src/dashboards.rs
// Description: Dashboard management and visualization module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{MonitoringError, MonitoringResult},
    types::{
        Dashboard, DashboardConfig, DashboardLayout, DashboardWidget, GaugeThreshold, TimeRange,
        WidgetConfig, WidgetPosition, WidgetSize, WidgetType,
    },
};

/// Dashboard manager trait
#[async_trait]
pub trait DashboardManager: Send + Sync {
    /// Create a new dashboard
    async fn create_dashboard(&self, dashboard: &Dashboard) -> MonitoringResult<Uuid>;

    /// Update existing dashboard
    async fn update_dashboard(&self, dashboard: &Dashboard) -> MonitoringResult<()>;

    /// Delete dashboard
    async fn delete_dashboard(&self, dashboard_id: &Uuid) -> MonitoringResult<()>;

    /// Get dashboard by ID
    async fn get_dashboard(&self, dashboard_id: &Uuid) -> MonitoringResult<Option<Dashboard>>;

    /// List all dashboards
    async fn list_dashboards(&self) -> MonitoringResult<Vec<Dashboard>>;

    /// Add widget to dashboard
    async fn add_widget(
        &self,
        dashboard_id: &Uuid,
        widget: &DashboardWidget,
    ) -> MonitoringResult<Uuid>;

    /// Update widget
    async fn update_widget(&self, widget: &DashboardWidget) -> MonitoringResult<()>;

    /// Remove widget from dashboard
    async fn remove_widget(&self, dashboard_id: &Uuid, widget_id: &Uuid) -> MonitoringResult<()>;

    /// Get dashboard data for rendering
    async fn get_dashboard_data(
        &self,
        dashboard_id: &Uuid,
        time_range: &TimeRange,
    ) -> MonitoringResult<DashboardData>;
}

/// Dashboard data for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub dashboard: Dashboard,
    pub widget_data: HashMap<Uuid, WidgetData>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetData {
    Chart {
        series: Vec<ChartSeries>,
        x_axis: Vec<String>,
        y_axis: Vec<f64>,
    },
    Gauge {
        value: f64,
        min: f64,
        max: f64,
        thresholds: Vec<GaugeThreshold>,
    },
    SingleStat {
        value: f64,
        unit: String,
        trend: Option<f64>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Text {
        content: String,
        rendered_html: Option<String>,
    },
}

/// Chart series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub data: Vec<ChartDataPoint>,
    pub color: Option<String>,
}

/// Chart data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub x: String,
    pub y: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Chart widget implementation
pub struct ChartWidget {
    config: WidgetConfig,
}

/// Table widget implementation
pub struct TableWidget {
    config: WidgetConfig,
}

/// Stat widget implementation
pub struct StatWidget {
    config: WidgetConfig,
}

/// Dashboard manager implementation
pub struct DashboardManagerImpl {
    config: DashboardConfig,
    dashboards: HashMap<Uuid, Dashboard>,
}

impl ChartWidget {
    pub fn new(config: WidgetConfig) -> Self {
        Self { config }
    }

    pub async fn render_data(
        &self,
        query: &str,
        time_range: &TimeRange,
    ) -> MonitoringResult<WidgetData> {
        // Mock chart data generation
        let mut series = Vec::new();

        // Generate sample time series data
        let start_time = time_range.start;
        let end_time = time_range.end;
        let duration = end_time - start_time;
        let points = 100;
        let step = duration / points;

        let mut data_points = Vec::new();
        for i in 0..points {
            let timestamp = start_time + step * i;
            let value = 50.0 + 20.0 * (i as f64 / 10.0).sin();

            data_points.push(ChartDataPoint {
                x: timestamp.format("%H:%M").to_string(),
                y: value,
                timestamp,
            });
        }

        series.push(ChartSeries {
            name: "CPU Usage".to_string(),
            data: data_points,
            color: Some("#3498db".to_string()),
        });

        Ok(WidgetData::Chart {
            series,
            x_axis: vec!["Time".to_string()],
            y_axis: vec![0.0, 100.0],
        })
    }
}

impl TableWidget {
    pub fn new(config: WidgetConfig) -> Self {
        Self { config }
    }

    pub async fn render_data(&self, query: &str) -> MonitoringResult<WidgetData> {
        // Mock table data generation
        let headers = vec![
            "Service".to_string(),
            "Status".to_string(),
            "Response Time".to_string(),
            "Error Rate".to_string(),
        ];

        let rows = vec![
            vec![
                "API Gateway".to_string(),
                "Healthy".to_string(),
                "45ms".to_string(),
                "0.1%".to_string(),
            ],
            vec![
                "Database".to_string(),
                "Healthy".to_string(),
                "12ms".to_string(),
                "0.0%".to_string(),
            ],
            vec![
                "Cache".to_string(),
                "Warning".to_string(),
                "2ms".to_string(),
                "0.5%".to_string(),
            ],
        ];

        Ok(WidgetData::Table { headers, rows })
    }
}

impl StatWidget {
    pub fn new(config: WidgetConfig) -> Self {
        Self { config }
    }

    pub async fn render_data(&self, query: &str) -> MonitoringResult<WidgetData> {
        // Mock single stat data generation
        Ok(WidgetData::SingleStat {
            value: 42.5,
            unit: "%".to_string(),
            trend: Some(2.3), // Positive trend
        })
    }
}

impl DashboardManagerImpl {
    pub fn new(config: DashboardConfig) -> Self {
        Self {
            config,
            dashboards: HashMap::new(),
        }
    }

    fn create_default_dashboard(&self) -> Dashboard {
        let widget1 = DashboardWidget {
            id: Uuid::new_v4(),
            title: "CPU Usage".to_string(),
            widget_type: WidgetType::LineChart,
            query: "cpu_usage_percent".to_string(),
            position: WidgetPosition { x: 0, y: 0 },
            size: WidgetSize {
                width: 6,
                height: 4,
            },
            config: WidgetConfig::Chart {
                x_axis: "Time".to_string(),
                y_axis: "Percentage".to_string(),
                legend: true,
            },
        };

        let widget2 = DashboardWidget {
            id: Uuid::new_v4(),
            title: "Memory Usage".to_string(),
            widget_type: WidgetType::Gauge,
            query: "memory_usage_percent".to_string(),
            position: WidgetPosition { x: 6, y: 0 },
            size: WidgetSize {
                width: 3,
                height: 4,
            },
            config: WidgetConfig::Gauge {
                min: 0.0,
                max: 100.0,
                thresholds: vec![
                    GaugeThreshold {
                        value: 70.0,
                        color: "#f39c12".to_string(),
                    },
                    GaugeThreshold {
                        value: 90.0,
                        color: "#e74c3c".to_string(),
                    },
                ],
            },
        };

        let widget3 = DashboardWidget {
            id: Uuid::new_v4(),
            title: "Active Connections".to_string(),
            widget_type: WidgetType::SingleStat,
            query: "active_connections".to_string(),
            position: WidgetPosition { x: 9, y: 0 },
            size: WidgetSize {
                width: 3,
                height: 2,
            },
            config: WidgetConfig::SingleStat {
                unit: "connections".to_string(),
                decimals: 0,
            },
        };

        Dashboard {
            id: Uuid::new_v4(),
            name: "System Overview".to_string(),
            description: "Overview of system metrics and health".to_string(),
            widgets: vec![widget1, widget2, widget3],
            layout: DashboardLayout {
                grid_size: 12,
                auto_arrange: false,
            },
            time_range: TimeRange {
                start: Utc::now() - chrono::Duration::hours(1),
                end: Utc::now(),
            },
            refresh_interval: chrono::Duration::seconds(30),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    async fn render_widget_data(
        &self,
        widget: &DashboardWidget,
        time_range: &TimeRange,
    ) -> MonitoringResult<WidgetData> {
        match widget.widget_type {
            WidgetType::LineChart | WidgetType::BarChart => {
                let chart_widget = ChartWidget::new(widget.config.clone());
                chart_widget.render_data(&widget.query, time_range).await
            }
            WidgetType::Table => {
                let table_widget = TableWidget::new(widget.config.clone());
                table_widget.render_data(&widget.query).await
            }
            WidgetType::SingleStat => {
                let stat_widget = StatWidget::new(widget.config.clone());
                stat_widget.render_data(&widget.query).await
            }
            WidgetType::Gauge => Ok(WidgetData::Gauge {
                value: 65.5,
                min: 0.0,
                max: 100.0,
                thresholds: vec![
                    GaugeThreshold {
                        value: 70.0,
                        color: "#f39c12".to_string(),
                    },
                    GaugeThreshold {
                        value: 90.0,
                        color: "#e74c3c".to_string(),
                    },
                ],
            }),
            WidgetType::Text => {
                if let WidgetConfig::Text { content, markdown } = &widget.config {
                    Ok(WidgetData::Text {
                        content: content.clone(),
                        rendered_html: if *markdown {
                            Some(format!("<p>{}</p>", content))
                        } else {
                            None
                        },
                    })
                } else {
                    Ok(WidgetData::Text {
                        content: "Default text content".to_string(),
                        rendered_html: None,
                    })
                }
            }
            _ => {
                // Default fallback
                Ok(WidgetData::SingleStat {
                    value: 0.0,
                    unit: "".to_string(),
                    trend: None,
                })
            }
        }
    }
}

#[async_trait]
impl DashboardManager for DashboardManagerImpl {
    async fn create_dashboard(&self, dashboard: &Dashboard) -> MonitoringResult<Uuid> {
        // Mock dashboard creation
        Ok(dashboard.id)
    }

    async fn update_dashboard(&self, dashboard: &Dashboard) -> MonitoringResult<()> {
        // Mock dashboard update
        Ok(())
    }

    async fn delete_dashboard(&self, dashboard_id: &Uuid) -> MonitoringResult<()> {
        // Mock dashboard deletion
        Ok(())
    }

    async fn get_dashboard(&self, dashboard_id: &Uuid) -> MonitoringResult<Option<Dashboard>> {
        // Return default dashboard for demo
        Ok(Some(self.create_default_dashboard()))
    }

    async fn list_dashboards(&self) -> MonitoringResult<Vec<Dashboard>> {
        // Return list with default dashboard
        Ok(vec![self.create_default_dashboard()])
    }

    async fn add_widget(
        &self,
        dashboard_id: &Uuid,
        widget: &DashboardWidget,
    ) -> MonitoringResult<Uuid> {
        // Mock widget addition
        Ok(widget.id)
    }

    async fn update_widget(&self, widget: &DashboardWidget) -> MonitoringResult<()> {
        // Mock widget update
        Ok(())
    }

    async fn remove_widget(&self, dashboard_id: &Uuid, widget_id: &Uuid) -> MonitoringResult<()> {
        // Mock widget removal
        Ok(())
    }

    async fn get_dashboard_data(
        &self,
        dashboard_id: &Uuid,
        time_range: &TimeRange,
    ) -> MonitoringResult<DashboardData> {
        let dashboard = self
            .get_dashboard(dashboard_id)
            .await?
            .ok_or_else(|| MonitoringError::dashboard_error("Dashboard not found"))?;

        let mut widget_data = HashMap::new();

        for widget in &dashboard.widgets {
            let data = self.render_widget_data(widget, time_range).await?;
            widget_data.insert(widget.id, data);
        }

        Ok(DashboardData {
            dashboard,
            widget_data,
            last_updated: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_manager_creation() {
        let config = DashboardConfig {
            enabled: true,
            refresh_interval_seconds: 30,
            default_time_range_hours: 24,
            max_widgets_per_dashboard: 20,
        };

        let manager = DashboardManagerImpl::new(config);
        let dashboards = manager.list_dashboards().await.unwrap();

        assert_eq!(dashboards.len(), 1);
        assert_eq!(dashboards[0].name, "System Overview");
    }

    #[tokio::test]
    async fn test_dashboard_data_rendering() {
        let config = DashboardConfig {
            enabled: true,
            refresh_interval_seconds: 30,
            default_time_range_hours: 24,
            max_widgets_per_dashboard: 20,
        };

        let manager = DashboardManagerImpl::new(config);
        let dashboard = manager.create_default_dashboard();

        let time_range = TimeRange {
            start: Utc::now() - chrono::Duration::hours(1),
            end: Utc::now(),
        };

        let dashboard_data = manager
            .get_dashboard_data(&dashboard.id, &time_range)
            .await
            .unwrap();

        assert_eq!(dashboard_data.dashboard.id, dashboard.id);
        assert_eq!(dashboard_data.widget_data.len(), dashboard.widgets.len());
    }

    #[tokio::test]
    async fn test_chart_widget_rendering() {
        let config = WidgetConfig::Chart {
            x_axis: "Time".to_string(),
            y_axis: "Value".to_string(),
            legend: true,
        };

        let widget = ChartWidget::new(config);
        let time_range = TimeRange {
            start: Utc::now() - chrono::Duration::hours(1),
            end: Utc::now(),
        };

        let data = widget.render_data("test_query", &time_range).await.unwrap();

        match data {
            WidgetData::Chart { series, .. } => {
                assert!(!series.is_empty());
                assert_eq!(series[0].name, "CPU Usage");
                assert!(!series[0].data.is_empty());
            }
            _ => panic!("Expected chart data"),
        }
    }

    #[tokio::test]
    async fn test_table_widget_rendering() {
        let config = WidgetConfig::Table {
            columns: vec!["Service".to_string(), "Status".to_string()],
            sortable: true,
        };

        let widget = TableWidget::new(config);
        let data = widget.render_data("test_query").await.unwrap();

        match data {
            WidgetData::Table { headers, rows } => {
                assert_eq!(headers.len(), 4);
                assert_eq!(rows.len(), 3);
                assert_eq!(headers[0], "Service");
                assert_eq!(rows[0][0], "API Gateway");
            }
            _ => panic!("Expected table data"),
        }
    }

    #[tokio::test]
    async fn test_stat_widget_rendering() {
        let config = WidgetConfig::SingleStat {
            unit: "%".to_string(),
            decimals: 1,
        };

        let widget = StatWidget::new(config);
        let data = widget.render_data("test_query").await.unwrap();

        match data {
            WidgetData::SingleStat { value, unit, trend } => {
                assert_eq!(value, 42.5);
                assert_eq!(unit, "%");
                assert!(trend.is_some());
            }
            _ => panic!("Expected single stat data"),
        }
    }
}
