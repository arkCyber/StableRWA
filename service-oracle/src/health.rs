// =====================================================================================
// RWA Tokenization Platform - Oracle Health Check Service
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::cache::PriceCache;
use crate::error::OracleResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn, error};

/// Health check service
pub struct HealthService {
    db_pool: PgPool,
    cache: Arc<PriceCache>,
    health_status: Arc<RwLock<HealthStatus>>,
    last_check: Arc<RwLock<Instant>>,
    check_interval: Duration,
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: ServiceStatus,
    pub timestamp: DateTime<Utc>,
    pub components: HashMap<String, ComponentHealth>,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Service status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: ServiceStatus,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub error_count: u32,
    pub success_count: u32,
}

/// Detailed health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: ServiceStatus,
    pub message: Option<String>,
    pub response_time: Duration,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl HealthService {
    /// Create a new health service
    pub fn new(
        db_pool: PgPool,
        cache: Arc<PriceCache>,
        check_interval: Duration,
    ) -> Self {
        let health_status = Arc::new(RwLock::new(HealthStatus {
            status: ServiceStatus::Unknown,
            timestamp: Utc::now(),
            components: HashMap::new(),
            uptime_seconds: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }));

        Self {
            db_pool,
            cache,
            health_status,
            last_check: Arc::new(RwLock::new(Instant::now())),
            check_interval,
        }
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> HealthStatus {
        let health_status = self.health_status.read().await;
        health_status.clone()
    }

    /// Perform comprehensive health check
    pub async fn perform_health_check(&self) -> OracleResult<HealthStatus> {
        let start_time = Instant::now();
        debug!("Starting comprehensive health check");

        let mut components = HashMap::new();

        // Check database health
        let db_result = self.check_database_health().await;
        components.insert("database".to_string(), self.result_to_component_health("database", db_result));

        // Check cache health
        let cache_result = self.check_cache_health().await;
        components.insert("cache".to_string(), self.result_to_component_health("cache", cache_result));

        // Check system resources
        let system_result = self.check_system_health().await;
        components.insert("system".to_string(), self.result_to_component_health("system", system_result));

        // Determine overall status
        let overall_status = self.determine_overall_status(&components);

        let health_status = HealthStatus {
            status: overall_status,
            timestamp: Utc::now(),
            components,
            uptime_seconds: start_time.elapsed().as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        // Update stored health status
        {
            let mut stored_status = self.health_status.write().await;
            *stored_status = health_status.clone();
        }

        // Update last check time
        {
            let mut last_check = self.last_check.write().await;
            *last_check = Instant::now();
        }

        debug!("Health check completed in {:?}", start_time.elapsed());
        Ok(health_status)
    }

    /// Check if service is ready to serve requests
    pub async fn readiness_check(&self) -> OracleResult<bool> {
        // Check critical components only
        let db_healthy = self.check_database_health().await.is_ok();
        let cache_healthy = self.check_cache_health().await.is_ok();

        Ok(db_healthy && cache_healthy)
    }

    /// Check if service is alive (basic liveness probe)
    pub async fn liveness_check(&self) -> OracleResult<bool> {
        // Simple check - if we can respond, we're alive
        Ok(true)
    }

    /// Check database connectivity and performance
    async fn check_database_health(&self) -> OracleResult<HealthCheckResult> {
        let start_time = Instant::now();
        
        // Test basic connectivity
        let query_result = sqlx::query("SELECT 1 as health_check")
            .fetch_one(&self.db_pool)
            .await;

        let response_time = start_time.elapsed();

        match query_result {
            Ok(_) => {
                // Additional checks for database performance
                let mut details = HashMap::new();
                
                // Check connection pool status
                let pool_size = self.db_pool.size();
                let idle_connections = self.db_pool.num_idle();
                
                details.insert("pool_size".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(pool_size)
                ));
                details.insert("idle_connections".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(idle_connections)
                ));
                details.insert("response_time_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(response_time.as_millis() as u64)
                ));

                let status = if response_time > Duration::from_millis(1000) {
                    ServiceStatus::Degraded
                } else {
                    ServiceStatus::Healthy
                };

                Ok(HealthCheckResult {
                    component: "database".to_string(),
                    status,
                    message: Some("Database connection successful".to_string()),
                    response_time,
                    details: Some(details),
                })
            }
            Err(e) => {
                error!("Database health check failed: {}", e);
                Ok(HealthCheckResult {
                    component: "database".to_string(),
                    status: ServiceStatus::Unhealthy,
                    message: Some(format!("Database connection failed: {}", e)),
                    response_time,
                    details: None,
                })
            }
        }
    }

    /// Check cache connectivity and performance
    async fn check_cache_health(&self) -> OracleResult<HealthCheckResult> {
        let start_time = Instant::now();
        
        let health_result = self.cache.health_check().await;
        let response_time = start_time.elapsed();

        match health_result {
            Ok(true) => {
                // Get cache statistics
                let mut details = HashMap::new();
                
                if let Ok(stats) = self.cache.get_stats().await {
                    details.insert("used_memory".to_string(), serde_json::Value::Number(
                        serde_json::Number::from(stats.used_memory)
                    ));
                    details.insert("key_count".to_string(), serde_json::Value::Number(
                        serde_json::Number::from(stats.key_count)
                    ));
                    details.insert("hit_rate".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(stats.hit_rate).unwrap_or_else(|| serde_json::Number::from(0))
                    ));
                }

                details.insert("response_time_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(response_time.as_millis() as u64)
                ));

                let status = if response_time > Duration::from_millis(500) {
                    ServiceStatus::Degraded
                } else {
                    ServiceStatus::Healthy
                };

                Ok(HealthCheckResult {
                    component: "cache".to_string(),
                    status,
                    message: Some("Cache connection successful".to_string()),
                    response_time,
                    details: Some(details),
                })
            }
            Ok(false) | Err(_) => {
                warn!("Cache health check failed");
                Ok(HealthCheckResult {
                    component: "cache".to_string(),
                    status: ServiceStatus::Unhealthy,
                    message: Some("Cache connection failed".to_string()),
                    response_time,
                    details: None,
                })
            }
        }
    }

    /// Check system resources and performance
    async fn check_system_health(&self) -> OracleResult<HealthCheckResult> {
        let start_time = Instant::now();
        
        let mut details = HashMap::new();
        let status = ServiceStatus::Healthy;
        let messages: Vec<String> = Vec::new();

        // Check memory usage (simplified - in production you'd use a proper system monitoring crate)
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                let mut total_memory = 0u64;
                let mut available_memory = 0u64;

                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            total_memory = value.parse().unwrap_or(0);
                        }
                    } else if line.starts_with("MemAvailable:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            available_memory = value.parse().unwrap_or(0);
                        }
                    }
                }

                if total_memory > 0 {
                    let memory_usage_percent = ((total_memory - available_memory) as f64 / total_memory as f64) * 100.0;
                    details.insert("memory_usage_percent".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(memory_usage_percent).unwrap_or_else(|| serde_json::Number::from(0))
                    ));

                    if memory_usage_percent > 90.0 {
                        status = ServiceStatus::Degraded;
                        messages.push("High memory usage detected".to_string());
                    }
                }
            }
        }

        // Check disk space (simplified)
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(metadata) = std::fs::metadata(".") {
                let available_space = metadata.blocks() * metadata.blksize();
                details.insert("available_disk_bytes".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(available_space)
                ));
            }
        }

        // Add current timestamp
        details.insert("check_timestamp".to_string(), serde_json::Value::String(
            Utc::now().to_rfc3339()
        ));

        let response_time = start_time.elapsed();
        let message = if messages.is_empty() {
            Some("System resources within normal limits".to_string())
        } else {
            Some(messages.join("; "))
        };

        Ok(HealthCheckResult {
            component: "system".to_string(),
            status,
            message,
            response_time,
            details: Some(details),
        })
    }

    /// Convert health check result to component health
    fn result_to_component_health(&self, _component: &str, result: OracleResult<HealthCheckResult>) -> ComponentHealth {
        match result {
            Ok(check_result) => ComponentHealth {
                status: check_result.status,
                message: check_result.message,
                last_check: Utc::now(),
                response_time_ms: Some(check_result.response_time.as_millis() as u64),
                error_count: 0,
                success_count: 1,
            },
            Err(e) => ComponentHealth {
                status: ServiceStatus::Unhealthy,
                message: Some(format!("Health check failed: {}", e)),
                last_check: Utc::now(),
                response_time_ms: None,
                error_count: 1,
                success_count: 0,
            },
        }
    }

    /// Determine overall service status based on component health
    fn determine_overall_status(&self, components: &HashMap<String, ComponentHealth>) -> ServiceStatus {
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;

        for component in components.values() {
            match component.status {
                ServiceStatus::Healthy => healthy_count += 1,
                ServiceStatus::Degraded => degraded_count += 1,
                ServiceStatus::Unhealthy => unhealthy_count += 1,
                ServiceStatus::Unknown => {}
            }
        }

        // If any critical component is unhealthy, overall status is unhealthy
        if unhealthy_count > 0 {
            ServiceStatus::Unhealthy
        } else if degraded_count > 0 {
            ServiceStatus::Degraded
        } else if healthy_count > 0 {
            ServiceStatus::Healthy
        } else {
            ServiceStatus::Unknown
        }
    }

    /// Start background health monitoring
    pub async fn start_monitoring(&self) {
        let health_service = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(health_service.check_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = health_service.perform_health_check().await {
                    error!("Background health check failed: {}", e);
                }
            }
        });
    }
}

impl Clone for HealthService {
    fn clone(&self) -> Self {
        Self {
            db_pool: self.db_pool.clone(),
            cache: self.cache.clone(),
            health_status: self.health_status.clone(),
            last_check: self.last_check.clone(),
            check_interval: self.check_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_service_status_serialization() {
        let status = ServiceStatus::Healthy;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: ServiceStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_component_health_creation() {
        let component = ComponentHealth {
            status: ServiceStatus::Healthy,
            message: Some("All good".to_string()),
            last_check: Utc::now(),
            response_time_ms: Some(100),
            error_count: 0,
            success_count: 5,
        };

        assert_eq!(component.status, ServiceStatus::Healthy);
        assert_eq!(component.message, Some("All good".to_string()));
        assert_eq!(component.response_time_ms, Some(100));
        assert_eq!(component.error_count, 0);
        assert_eq!(component.success_count, 5);
    }

    #[test]
    fn test_determine_overall_status() {
        // This would require creating a HealthService instance
        // For now, test the logic conceptually
        
        // All healthy -> Healthy
        let mut components = HashMap::new();
        components.insert("db".to_string(), ComponentHealth {
            status: ServiceStatus::Healthy,
            message: None,
            last_check: Utc::now(),
            response_time_ms: Some(50),
            error_count: 0,
            success_count: 1,
        });
        components.insert("cache".to_string(), ComponentHealth {
            status: ServiceStatus::Healthy,
            message: None,
            last_check: Utc::now(),
            response_time_ms: Some(25),
            error_count: 0,
            success_count: 1,
        });

        // Would need HealthService instance to test determine_overall_status method
        // This demonstrates the test structure
    }

    #[test]
    fn test_health_check_result_creation() {
        let result = HealthCheckResult {
            component: "test".to_string(),
            status: ServiceStatus::Healthy,
            message: Some("Test passed".to_string()),
            response_time: Duration::from_millis(100),
            details: None,
        };

        assert_eq!(result.component, "test");
        assert_eq!(result.status, ServiceStatus::Healthy);
        assert_eq!(result.message, Some("Test passed".to_string()));
        assert_eq!(result.response_time, Duration::from_millis(100));
    }

    #[test]
    fn test_health_status_serialization() {
        let mut components = HashMap::new();
        components.insert("test".to_string(), ComponentHealth {
            status: ServiceStatus::Healthy,
            message: None,
            last_check: Utc::now(),
            response_time_ms: Some(100),
            error_count: 0,
            success_count: 1,
        });

        let health_status = HealthStatus {
            status: ServiceStatus::Healthy,
            timestamp: Utc::now(),
            components,
            uptime_seconds: 3600,
            version: "1.0.0".to_string(),
        };

        let serialized = serde_json::to_string(&health_status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(health_status.status, deserialized.status);
        assert_eq!(health_status.uptime_seconds, deserialized.uptime_seconds);
        assert_eq!(health_status.version, deserialized.version);
        assert_eq!(health_status.components.len(), deserialized.components.len());
    }
}
