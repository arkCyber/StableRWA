// =====================================================================================
// File: core-stablecoin/src/monitoring.rs
// Description: Enterprise monitoring and alerting system for stablecoin operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

use crate::{StablecoinResult, StablecoinError};
use crate::types::{Stablecoin, CollateralPosition};

/// Monitoring service trait
#[async_trait]
pub trait MonitoringService: Send + Sync {
    /// Start monitoring for a stablecoin
    async fn start_monitoring(&self, stablecoin_id: Uuid, config: MonitoringConfig) -> StablecoinResult<()>;
    
    /// Stop monitoring for a stablecoin
    async fn stop_monitoring(&self, stablecoin_id: Uuid) -> StablecoinResult<()>;
    
    /// Get current system health
    async fn get_system_health(&self) -> StablecoinResult<SystemHealth>;
    
    /// Get metrics for a specific stablecoin
    async fn get_stablecoin_metrics(&self, stablecoin_id: Uuid) -> StablecoinResult<StablecoinMetrics>;
    
    /// Get active alerts
    async fn get_active_alerts(&self) -> StablecoinResult<Vec<Alert>>;
    
    /// Acknowledge an alert
    async fn acknowledge_alert(&self, alert_id: Uuid, user_id: &str) -> StablecoinResult<()>;
    
    /// Get monitoring history
    async fn get_monitoring_history(&self, stablecoin_id: Uuid, period: TimePeriod) -> StablecoinResult<Vec<MonitoringEvent>>;
    
    /// Update monitoring configuration
    async fn update_monitoring_config(&self, stablecoin_id: Uuid, config: MonitoringConfig) -> StablecoinResult<()>;
}

/// Enterprise monitoring service implementation
pub struct EnterpriseMonitoringService {
    // Core monitoring data
    monitored_stablecoins: Arc<RwLock<HashMap<Uuid, MonitoringConfig>>>,
    metrics_cache: Arc<Mutex<HashMap<Uuid, StablecoinMetrics>>>,
    active_alerts: Arc<Mutex<HashMap<Uuid, Alert>>>,
    monitoring_history: Arc<Mutex<HashMap<Uuid, Vec<MonitoringEvent>>>>,
    
    // Alert handlers
    alert_handlers: Arc<RwLock<Vec<Box<dyn AlertHandler>>>>,
    
    // System health tracking
    system_health: Arc<Mutex<SystemHealth>>,
    
    // Configuration
    global_config: MonitoringGlobalConfig,
}

impl EnterpriseMonitoringService {
    pub fn new(global_config: MonitoringGlobalConfig) -> Self {
        Self {
            monitored_stablecoins: Arc::new(RwLock::new(HashMap::new())),
            metrics_cache: Arc::new(Mutex::new(HashMap::new())),
            active_alerts: Arc::new(Mutex::new(HashMap::new())),
            monitoring_history: Arc::new(Mutex::new(HashMap::new())),
            alert_handlers: Arc::new(RwLock::new(Vec::new())),
            system_health: Arc::new(Mutex::new(SystemHealth::default())),
            global_config,
        }
    }
    
    pub fn with_defaults() -> Self {
        Self::new(MonitoringGlobalConfig::default())
    }
    
    /// Add an alert handler
    pub async fn add_alert_handler(&self, handler: Box<dyn AlertHandler>) {
        let mut handlers = self.alert_handlers.write().await;
        handlers.push(handler);
    }
    
    /// Start the monitoring loop (requires Arc<Self>)
    pub async fn start_monitoring_loop(self: Arc<Self>) -> StablecoinResult<()> {
        let service = self.clone();

        // Spawn monitoring task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(service.global_config.monitoring_interval_seconds)
            );

            loop {
                interval.tick().await;

                if let Err(e) = service.perform_monitoring_cycle().await {
                    eprintln!("Monitoring cycle error: {}", e);
                }
            }
        });

        Ok(())
    }
    
    /// Perform a single monitoring cycle
    async fn perform_monitoring_cycle(&self) -> StablecoinResult<()> {
        let monitored = self.monitored_stablecoins.read().await.clone();
        
        for (stablecoin_id, config) in monitored {
            if let Err(e) = self.monitor_stablecoin(stablecoin_id, &config).await {
                eprintln!("Error monitoring stablecoin {}: {}", stablecoin_id, e);
            }
        }
        
        // Update system health
        self.update_system_health().await?;
        
        Ok(())
    }
    
    /// Monitor a specific stablecoin
    async fn monitor_stablecoin(&self, stablecoin_id: Uuid, config: &MonitoringConfig) -> StablecoinResult<()> {
        // Collect metrics
        let metrics = self.collect_stablecoin_metrics(stablecoin_id).await?;
        
        // Store metrics in cache
        {
            let mut cache = self.metrics_cache.lock().await;
            cache.insert(stablecoin_id, metrics.clone());
        }
        
        // Check for alerts
        self.check_alerts(stablecoin_id, &metrics, config).await?;
        
        // Record monitoring event
        let event = MonitoringEvent {
            id: Uuid::new_v4(),
            stablecoin_id,
            event_type: MonitoringEventType::MetricsCollected,
            timestamp: Utc::now(),
            data: serde_json::to_value(&metrics).unwrap_or_default(),
        };
        
        {
            let mut history = self.monitoring_history.lock().await;
            history.entry(stablecoin_id)
                .or_insert_with(Vec::new)
                .push(event);
        }
        
        Ok(())
    }
    
    /// Collect metrics for a stablecoin
    async fn collect_stablecoin_metrics(&self, stablecoin_id: Uuid) -> StablecoinResult<StablecoinMetrics> {
        // In a real implementation, this would collect actual metrics from various sources
        // For now, we'll simulate some metrics
        
        Ok(StablecoinMetrics {
            stablecoin_id,
            timestamp: Utc::now(),
            
            // Supply metrics
            total_supply: Decimal::new(10_000_000, 0), // $10M
            circulating_supply: Decimal::new(9_500_000, 0), // $9.5M
            
            // Price metrics
            current_price: Decimal::new(100, 2), // $1.00
            price_deviation: Decimal::new(5, 3), // 0.5%
            
            // Collateral metrics
            total_collateral_value: Decimal::new(15_000_000, 0), // $15M
            collateral_ratio: Decimal::new(158, 2), // 158%
            
            // Transaction metrics
            daily_volume: Decimal::new(1_000_000, 0), // $1M
            transaction_count_24h: 1500,
            
            // Performance metrics
            avg_transaction_time: Duration::seconds(15),
            success_rate: Decimal::new(9995, 4), // 99.95%
            
            // Risk metrics
            risk_score: 25, // Low risk
            volatility: Decimal::new(2, 2), // 2%
        })
    }
    
    /// Check for alert conditions
    async fn check_alerts(&self, stablecoin_id: Uuid, metrics: &StablecoinMetrics, config: &MonitoringConfig) -> StablecoinResult<()> {
        let mut new_alerts = Vec::new();
        
        // Price deviation alert
        if metrics.price_deviation > config.max_price_deviation {
            new_alerts.push(Alert {
                id: Uuid::new_v4(),
                stablecoin_id,
                alert_type: AlertType::PriceDeviation,
                severity: AlertSeverity::High,
                title: "Price Deviation Alert".to_string(),
                description: format!("Price deviation {}% exceeds threshold {}%", 
                    metrics.price_deviation, config.max_price_deviation),
                triggered_at: Utc::now(),
                acknowledged_at: None,
                acknowledged_by: None,
                resolved_at: None,
                metadata: HashMap::new(),
            });
        }
        
        // Collateral ratio alert
        if metrics.collateral_ratio < config.min_collateral_ratio {
            new_alerts.push(Alert {
                id: Uuid::new_v4(),
                stablecoin_id,
                alert_type: AlertType::LowCollateral,
                severity: AlertSeverity::Critical,
                title: "Low Collateral Alert".to_string(),
                description: format!("Collateral ratio {}% below minimum {}%", 
                    metrics.collateral_ratio, config.min_collateral_ratio),
                triggered_at: Utc::now(),
                acknowledged_at: None,
                acknowledged_by: None,
                resolved_at: None,
                metadata: HashMap::new(),
            });
        }
        
        // High volume alert
        if metrics.daily_volume > config.max_daily_volume {
            new_alerts.push(Alert {
                id: Uuid::new_v4(),
                stablecoin_id,
                alert_type: AlertType::HighVolume,
                severity: AlertSeverity::Medium,
                title: "High Volume Alert".to_string(),
                description: format!("Daily volume {} exceeds threshold {}", 
                    metrics.daily_volume, config.max_daily_volume),
                triggered_at: Utc::now(),
                acknowledged_at: None,
                acknowledged_by: None,
                resolved_at: None,
                metadata: HashMap::new(),
            });
        }
        
        // Process new alerts
        for alert in new_alerts {
            self.trigger_alert(alert).await?;
        }
        
        Ok(())
    }
    
    /// Trigger an alert
    async fn trigger_alert(&self, alert: Alert) -> StablecoinResult<()> {
        // Store alert
        {
            let mut alerts = self.active_alerts.lock().await;
            alerts.insert(alert.id, alert.clone());
        }
        
        // Notify alert handlers
        let handlers = self.alert_handlers.read().await;
        for handler in handlers.iter() {
            if let Err(e) = handler.handle_alert(&alert).await {
                eprintln!("Alert handler error: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Update system health
    async fn update_system_health(&self) -> StablecoinResult<()> {
        let monitored = self.monitored_stablecoins.read().await;
        let alerts = self.active_alerts.lock().await;
        
        let total_stablecoins = monitored.len();
        let critical_alerts = alerts.values()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .count();
        
        let health_status = if critical_alerts > 0 {
            HealthStatus::Critical
        } else if alerts.len() > total_stablecoins {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
        
        let mut health = self.system_health.lock().await;
        health.status = health_status;
        health.total_stablecoins = total_stablecoins;
        health.active_alerts = alerts.len();
        health.critical_alerts = critical_alerts;
        health.last_updated = Utc::now();
        
        Ok(())
    }
}

#[async_trait]
impl MonitoringService for EnterpriseMonitoringService {
    async fn start_monitoring(&self, stablecoin_id: Uuid, config: MonitoringConfig) -> StablecoinResult<()> {
        let mut monitored = self.monitored_stablecoins.write().await;
        monitored.insert(stablecoin_id, config);

        // Initialize history
        {
            let mut history = self.monitoring_history.lock().await;
            history.insert(stablecoin_id, Vec::new());
        }

        Ok(())
    }

    async fn stop_monitoring(&self, stablecoin_id: Uuid) -> StablecoinResult<()> {
        let mut monitored = self.monitored_stablecoins.write().await;
        monitored.remove(&stablecoin_id);

        // Clear related alerts
        {
            let mut alerts = self.active_alerts.lock().await;
            alerts.retain(|_, alert| alert.stablecoin_id != stablecoin_id);
        }

        Ok(())
    }

    async fn get_system_health(&self) -> StablecoinResult<SystemHealth> {
        let health = self.system_health.lock().await;
        Ok(health.clone())
    }

    async fn get_stablecoin_metrics(&self, stablecoin_id: Uuid) -> StablecoinResult<StablecoinMetrics> {
        let cache = self.metrics_cache.lock().await;
        cache.get(&stablecoin_id)
            .cloned()
            .ok_or_else(|| StablecoinError::InvalidRequest("Stablecoin not monitored".to_string()))
    }

    async fn get_active_alerts(&self) -> StablecoinResult<Vec<Alert>> {
        let alerts = self.active_alerts.lock().await;
        Ok(alerts.values().cloned().collect())
    }

    async fn acknowledge_alert(&self, alert_id: Uuid, user_id: &str) -> StablecoinResult<()> {
        let mut alerts = self.active_alerts.lock().await;

        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.acknowledged_at = Some(Utc::now());
            alert.acknowledged_by = Some(user_id.to_string());
            Ok(())
        } else {
            Err(StablecoinError::InvalidRequest("Alert not found".to_string()))
        }
    }

    async fn get_monitoring_history(&self, stablecoin_id: Uuid, period: TimePeriod) -> StablecoinResult<Vec<MonitoringEvent>> {
        let history = self.monitoring_history.lock().await;

        if let Some(events) = history.get(&stablecoin_id) {
            let filtered: Vec<MonitoringEvent> = events.iter()
                .filter(|event| event.timestamp >= period.start && event.timestamp <= period.end)
                .cloned()
                .collect();
            Ok(filtered)
        } else {
            Ok(Vec::new())
        }
    }

    async fn update_monitoring_config(&self, stablecoin_id: Uuid, config: MonitoringConfig) -> StablecoinResult<()> {
        let mut monitored = self.monitored_stablecoins.write().await;

        if monitored.contains_key(&stablecoin_id) {
            monitored.insert(stablecoin_id, config);
            Ok(())
        } else {
            Err(StablecoinError::InvalidRequest("Stablecoin not monitored".to_string()))
        }
    }
}

/// Alert handler trait
#[async_trait]
pub trait AlertHandler: Send + Sync {
    async fn handle_alert(&self, alert: &Alert) -> StablecoinResult<()>;
}

/// Email alert handler
pub struct EmailAlertHandler {
    smtp_config: SmtpConfig,
    recipients: Vec<String>,
}

impl EmailAlertHandler {
    pub fn new(smtp_config: SmtpConfig, recipients: Vec<String>) -> Self {
        Self { smtp_config, recipients }
    }
}

#[async_trait]
impl AlertHandler for EmailAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> StablecoinResult<()> {
        // Simulate sending email
        println!("📧 EMAIL ALERT: {} - {}", alert.title, alert.description);
        println!("   Recipients: {:?}", self.recipients);
        println!("   Severity: {:?}", alert.severity);
        Ok(())
    }
}

/// Slack alert handler
pub struct SlackAlertHandler {
    webhook_url: String,
    channel: String,
}

impl SlackAlertHandler {
    pub fn new(webhook_url: String, channel: String) -> Self {
        Self { webhook_url, channel }
    }
}

#[async_trait]
impl AlertHandler for SlackAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> StablecoinResult<()> {
        // Simulate sending Slack message
        let emoji = match alert.severity {
            AlertSeverity::Critical => "🚨",
            AlertSeverity::High => "⚠️",
            AlertSeverity::Medium => "⚡",
            AlertSeverity::Low => "ℹ️",
        };

        println!("{} SLACK ALERT: {} - {}", emoji, alert.title, alert.description);
        println!("   Channel: {}", self.channel);
        Ok(())
    }
}

/// PagerDuty alert handler
pub struct PagerDutyAlertHandler {
    integration_key: String,
}

impl PagerDutyAlertHandler {
    pub fn new(integration_key: String) -> Self {
        Self { integration_key }
    }
}

#[async_trait]
impl AlertHandler for PagerDutyAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> StablecoinResult<()> {
        // Only trigger PagerDuty for critical alerts
        if alert.severity == AlertSeverity::Critical {
            println!("📟 PAGERDUTY ALERT: {} - {}", alert.title, alert.description);
            println!("   Integration Key: {}", self.integration_key);
        }
        Ok(())
    }
}

/// Monitoring configuration for a stablecoin
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Maximum allowed price deviation (percentage)
    pub max_price_deviation: Decimal,
    /// Minimum collateral ratio
    pub min_collateral_ratio: Decimal,
    /// Maximum daily volume threshold
    pub max_daily_volume: Decimal,
    /// Maximum transaction failure rate (percentage)
    pub max_failure_rate: Decimal,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Monitoring frequency (seconds)
    pub monitoring_frequency: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            max_price_deviation: Decimal::new(2, 2), // 2%
            min_collateral_ratio: Decimal::new(120, 2), // 120%
            max_daily_volume: Decimal::new(10_000_000, 0), // $10M
            max_failure_rate: Decimal::new(1, 2), // 1%
            alert_thresholds: AlertThresholds::default(),
            monitoring_frequency: 60, // 1 minute
        }
    }
}

/// Global monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringGlobalConfig {
    /// Monitoring interval in seconds
    pub monitoring_interval_seconds: u64,
    /// Maximum alerts to keep in memory
    pub max_alerts_in_memory: usize,
    /// Maximum history events per stablecoin
    pub max_history_events: usize,
    /// Enable detailed logging
    pub enable_detailed_logging: bool,
}

impl Default for MonitoringGlobalConfig {
    fn default() -> Self {
        Self {
            monitoring_interval_seconds: 30,
            max_alerts_in_memory: 1000,
            max_history_events: 10000,
            enable_detailed_logging: true,
        }
    }
}

/// Alert thresholds configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub price_deviation_warning: Decimal,
    pub price_deviation_critical: Decimal,
    pub collateral_ratio_warning: Decimal,
    pub collateral_ratio_critical: Decimal,
    pub volume_warning: Decimal,
    pub volume_critical: Decimal,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            price_deviation_warning: Decimal::new(1, 2), // 1%
            price_deviation_critical: Decimal::new(3, 2), // 3%
            collateral_ratio_warning: Decimal::new(130, 2), // 130%
            collateral_ratio_critical: Decimal::new(110, 2), // 110%
            volume_warning: Decimal::new(5_000_000, 0), // $5M
            volume_critical: Decimal::new(20_000_000, 0), // $20M
        }
    }
}

/// System health status
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub total_stablecoins: usize,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub last_updated: DateTime<Utc>,
    pub uptime: Duration,
    pub performance_metrics: PerformanceMetrics,
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Healthy,
            total_stablecoins: 0,
            active_alerts: 0,
            critical_alerts: 0,
            last_updated: Utc::now(),
            uptime: Duration::zero(),
            performance_metrics: PerformanceMetrics::default(),
        }
    }
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Maintenance,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_response_time: Duration,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time: Duration::milliseconds(100),
            requests_per_second: 100.0,
            error_rate: 0.01, // 1%
            memory_usage: 512 * 1024 * 1024, // 512MB
            cpu_usage: 25.0, // 25%
        }
    }
}

/// Stablecoin metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StablecoinMetrics {
    pub stablecoin_id: Uuid,
    pub timestamp: DateTime<Utc>,

    // Supply metrics
    pub total_supply: Decimal,
    pub circulating_supply: Decimal,

    // Price metrics
    pub current_price: Decimal,
    pub price_deviation: Decimal,

    // Collateral metrics
    pub total_collateral_value: Decimal,
    pub collateral_ratio: Decimal,

    // Transaction metrics
    pub daily_volume: Decimal,
    pub transaction_count_24h: u64,

    // Performance metrics
    pub avg_transaction_time: Duration,
    pub success_rate: Decimal,

    // Risk metrics
    pub risk_score: u8,
    pub volatility: Decimal,
}

/// Alert structure
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub triggered_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Alert types
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    PriceDeviation,
    LowCollateral,
    HighVolume,
    SystemFailure,
    SecurityBreach,
    ComplianceViolation,
    PerformanceDegradation,
    MaintenanceRequired,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Monitoring event
#[derive(Debug, Clone)]
pub struct MonitoringEvent {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub event_type: MonitoringEventType,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

/// Monitoring event types
#[derive(Debug, Clone, PartialEq)]
pub enum MonitoringEventType {
    MetricsCollected,
    AlertTriggered,
    AlertResolved,
    ConfigurationChanged,
    SystemHealthUpdated,
}

/// Time period for queries
#[derive(Debug, Clone)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// SMTP configuration for email alerts
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enterprise_monitoring_service_creation() {
        let config = MonitoringGlobalConfig::default();
        let service = EnterpriseMonitoringService::new(config);

        let health = service.get_system_health().await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.total_stablecoins, 0);
    }

    #[tokio::test]
    async fn test_start_stop_monitoring() {
        let service = EnterpriseMonitoringService::with_defaults();
        let stablecoin_id = Uuid::new_v4();
        let config = MonitoringConfig::default();

        // Start monitoring
        service.start_monitoring(stablecoin_id, config).await.unwrap();

        // Verify monitoring started
        let monitored = service.monitored_stablecoins.read().await;
        assert!(monitored.contains_key(&stablecoin_id));
        drop(monitored);

        // Stop monitoring
        service.stop_monitoring(stablecoin_id).await.unwrap();

        // Verify monitoring stopped
        let monitored = service.monitored_stablecoins.read().await;
        assert!(!monitored.contains_key(&stablecoin_id));
    }

    #[tokio::test]
    async fn test_monitoring_config() {
        let config = MonitoringConfig::default();

        assert_eq!(config.max_price_deviation, Decimal::new(2, 2)); // 2%
        assert_eq!(config.min_collateral_ratio, Decimal::new(120, 2)); // 120%
        assert_eq!(config.monitoring_frequency, 60); // 1 minute
    }

    #[tokio::test]
    async fn test_alert_creation() {
        let alert = Alert {
            id: Uuid::new_v4(),
            stablecoin_id: Uuid::new_v4(),
            alert_type: AlertType::PriceDeviation,
            severity: AlertSeverity::High,
            title: "Test Alert".to_string(),
            description: "Test alert description".to_string(),
            triggered_at: Utc::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            metadata: HashMap::new(),
        };

        assert_eq!(alert.alert_type, AlertType::PriceDeviation);
        assert_eq!(alert.severity, AlertSeverity::High);
        assert!(alert.acknowledged_at.is_none());
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let service = EnterpriseMonitoringService::with_defaults();
        let alert_id = Uuid::new_v4();

        let alert = Alert {
            id: alert_id,
            stablecoin_id: Uuid::new_v4(),
            alert_type: AlertType::LowCollateral,
            severity: AlertSeverity::Critical,
            title: "Low Collateral".to_string(),
            description: "Collateral ratio below threshold".to_string(),
            triggered_at: Utc::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            metadata: HashMap::new(),
        };

        // Add alert to active alerts
        {
            let mut alerts = service.active_alerts.lock().await;
            alerts.insert(alert_id, alert);
        }

        // Acknowledge alert
        service.acknowledge_alert(alert_id, "admin").await.unwrap();

        // Verify acknowledgment
        let alerts = service.active_alerts.lock().await;
        let acknowledged_alert = alerts.get(&alert_id).unwrap();
        assert!(acknowledged_alert.acknowledged_at.is_some());
        assert_eq!(acknowledged_alert.acknowledged_by, Some("admin".to_string()));
    }

    #[tokio::test]
    async fn test_stablecoin_metrics() {
        let metrics = StablecoinMetrics {
            stablecoin_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            total_supply: Decimal::new(1_000_000, 0),
            circulating_supply: Decimal::new(950_000, 0),
            current_price: Decimal::new(100, 2), // $1.00
            price_deviation: Decimal::new(5, 3), // 0.5%
            total_collateral_value: Decimal::new(1_500_000, 0),
            collateral_ratio: Decimal::new(158, 2), // 158%
            daily_volume: Decimal::new(100_000, 0),
            transaction_count_24h: 500,
            avg_transaction_time: Duration::seconds(15),
            success_rate: Decimal::new(9995, 4), // 99.95%
            risk_score: 25,
            volatility: Decimal::new(2, 2), // 2%
        };

        assert_eq!(metrics.current_price, Decimal::new(100, 2));
        assert_eq!(metrics.collateral_ratio, Decimal::new(158, 2));
        assert_eq!(metrics.risk_score, 25);
    }

    #[tokio::test]
    async fn test_email_alert_handler() {
        let smtp_config = SmtpConfig {
            server: "smtp.example.com".to_string(),
            port: 587,
            username: "alerts@example.com".to_string(),
            password: "password".to_string(),
            use_tls: true,
        };

        let handler = EmailAlertHandler::new(
            smtp_config,
            vec!["admin@example.com".to_string()]
        );

        let alert = Alert {
            id: Uuid::new_v4(),
            stablecoin_id: Uuid::new_v4(),
            alert_type: AlertType::SystemFailure,
            severity: AlertSeverity::Critical,
            title: "System Failure".to_string(),
            description: "Critical system failure detected".to_string(),
            triggered_at: Utc::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            metadata: HashMap::new(),
        };

        // This should not fail (just prints to console in test)
        handler.handle_alert(&alert).await.unwrap();
    }

    #[tokio::test]
    async fn test_slack_alert_handler() {
        let handler = SlackAlertHandler::new(
            "https://hooks.slack.com/services/test".to_string(),
            "#alerts".to_string()
        );

        let alert = Alert {
            id: Uuid::new_v4(),
            stablecoin_id: Uuid::new_v4(),
            alert_type: AlertType::PerformanceDegradation,
            severity: AlertSeverity::Medium,
            title: "Performance Issue".to_string(),
            description: "System performance degraded".to_string(),
            triggered_at: Utc::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            resolved_at: None,
            metadata: HashMap::new(),
        };

        handler.handle_alert(&alert).await.unwrap();
    }

    #[tokio::test]
    async fn test_system_health() {
        let mut health = SystemHealth::default();

        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.active_alerts, 0);
        assert_eq!(health.critical_alerts, 0);

        // Simulate some alerts
        health.active_alerts = 5;
        health.critical_alerts = 1;
        health.status = HealthStatus::Warning;

        assert_eq!(health.status, HealthStatus::Warning);
        assert_eq!(health.active_alerts, 5);
    }

    #[tokio::test]
    async fn test_monitoring_history() {
        let service = EnterpriseMonitoringService::with_defaults();
        let stablecoin_id = Uuid::new_v4();

        let event = MonitoringEvent {
            id: Uuid::new_v4(),
            stablecoin_id,
            event_type: MonitoringEventType::MetricsCollected,
            timestamp: Utc::now(),
            data: serde_json::json!({"test": "data"}),
        };

        // Add event to history
        {
            let mut history = service.monitoring_history.lock().await;
            history.insert(stablecoin_id, vec![event.clone()]);
        }

        // Query history
        let period = TimePeriod {
            start: Utc::now() - Duration::hours(1),
            end: Utc::now() + Duration::hours(1),
        };

        let events = service.get_monitoring_history(stablecoin_id, period).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, MonitoringEventType::MetricsCollected);
    }
}
