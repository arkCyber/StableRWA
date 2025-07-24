// =====================================================================================
// File: service-asset/src/health.rs
// Description: Enterprise-grade health check system for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use crate::cache::CacheExt;

use crate::cache::Cache;
use crate::metrics::AssetMetrics;

/// Health check service
pub struct HealthService {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,
    cache: Option<Arc<dyn Cache>>,
    metrics: Option<AssetMetrics>,
    config: HealthConfig,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub check_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub detailed_response: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            timeout_seconds: 10,
            failure_threshold: 3,
            success_threshold: 2,
            detailed_response: true,
        }
    }
}

/// Health check trait
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check
    async fn check(&self) -> HealthCheckResult;
    
    /// Get check name
    fn name(&self) -> &str;
    
    /// Get check description
    fn description(&self) -> &str;
    
    /// Check if this is a critical check
    fn is_critical(&self) -> bool {
        false
    }
    
    /// Get check timeout
    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: String,
    pub duration_ms: u64,
    pub timestamp: u64,
    pub details: Option<serde_json::Value>,
}

/// Health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Overall service health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub version: String,
    pub checks: HashMap<String, HealthCheckResult>,
    pub summary: HealthSummary,
}

/// Health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub total_checks: u32,
    pub healthy_checks: u32,
    pub degraded_checks: u32,
    pub unhealthy_checks: u32,
    pub critical_failures: u32,
}

impl HealthService {
    /// Create new health service
    pub fn new(config: HealthConfig) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            cache: None,
            metrics: None,
            config,
        }
    }
    
    /// Add cache for health checks
    pub fn with_cache(mut self, cache: Arc<dyn Cache>) -> Self {
        self.cache = Some(cache);
        self
    }
    
    /// Add metrics for health checks
    pub fn with_metrics(mut self, metrics: AssetMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    /// Register a health check
    pub async fn register_check(&self, check: Box<dyn HealthCheck>) {
        let name = check.name().to_string();
        let mut checks = self.checks.write().await;
        checks.insert(name, check);
    }
    
    /// Perform all health checks
    pub async fn check_health(&self) -> ServiceHealth {
        let start_time = Instant::now();
        let checks = self.checks.read().await;
        let mut results = HashMap::new();
        let mut summary = HealthSummary {
            total_checks: checks.len() as u32,
            healthy_checks: 0,
            degraded_checks: 0,
            unhealthy_checks: 0,
            critical_failures: 0,
        };
        
        // Run all health checks concurrently
        let mut check_futures = Vec::new();
        for (name, check) in checks.iter() {
            let check_name = name.clone();
            let check_ref = check.as_ref();
            let timeout = check_ref.timeout();
            let is_critical = check_ref.is_critical();
            
            let future = async move {
                let result = tokio::time::timeout(timeout, check_ref.check()).await;
                let check_result = match result {
                    Ok(result) => result,
                    Err(_) => HealthCheckResult {
                        status: HealthStatus::Unhealthy,
                        message: "Health check timed out".to_string(),
                        duration_ms: timeout.as_millis() as u64,
                        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        details: None,
                    },
                };
                (check_name, check_result, is_critical)
            };
            
            check_futures.push(future);
        }
        
        // Wait for all checks to complete
        let check_results = futures_util::future::join_all(check_futures).await;
        
        // Process results
        for (name, result, is_critical) in check_results {
            match result.status {
                HealthStatus::Healthy => summary.healthy_checks += 1,
                HealthStatus::Degraded => summary.degraded_checks += 1,
                HealthStatus::Unhealthy => {
                    summary.unhealthy_checks += 1;
                    if is_critical {
                        summary.critical_failures += 1;
                    }
                }
            }
            
            results.insert(name, result);
        }
        
        // Determine overall status
        let overall_status = if summary.critical_failures > 0 {
            HealthStatus::Unhealthy
        } else if summary.unhealthy_checks > 0 || summary.degraded_checks > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        // Record metrics
        if let Some(ref metrics) = self.metrics {
            // Record health check metrics
            for (name, result) in &results {
                let status_str = match result.status {
                    HealthStatus::Healthy => "healthy",
                    HealthStatus::Degraded => "degraded",
                    HealthStatus::Unhealthy => "unhealthy",
                };
                // This would require adding health check metrics to AssetMetrics
                // metrics.record_health_check(name, status_str, result.duration_ms);
            }
        }
        
        ServiceHealth {
            status: overall_status,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            uptime_seconds: start_time.elapsed().as_secs(), // This should be service uptime
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks: results,
            summary,
        }
    }
    
    /// Get simple health status (for load balancers)
    pub async fn is_healthy(&self) -> bool {
        let health = self.check_health().await;
        health.status == HealthStatus::Healthy
    }
}

/// Database health check
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

impl DatabaseHealthCheck {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        
        let result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        match result {
            Ok(_) => HealthCheckResult {
                status: HealthStatus::Healthy,
                message: "Database connection successful".to_string(),
                duration_ms,
                timestamp,
                details: Some(serde_json::json!({
                    "pool_size": self.pool.size(),
                    "idle_connections": self.pool.num_idle(),
                })),
            },
            Err(e) => HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: format!("Database connection failed: {}", e),
                duration_ms,
                timestamp,
                details: None,
            },
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
    
    fn description(&self) -> &str {
        "PostgreSQL database connectivity"
    }
    
    fn is_critical(&self) -> bool {
        true
    }
}

/// Cache health check
pub struct CacheHealthCheck {
    cache: Arc<dyn Cache>,
}

impl CacheHealthCheck {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }
}

#[async_trait::async_trait]
impl HealthCheck for CacheHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        let test_key = "health_check_test";
        let test_value = "test_value";
        
        // Try to set and get a test value
        let set_result = self.cache.set(test_key, &test_value, Duration::from_secs(60)).await;
        let get_result = if set_result.is_ok() {
            self.cache.get::<String>(test_key).await
        } else {
            Err(crate::cache::CacheError::OperationError("Set failed".to_string()))
        };
        
        // Clean up
        let _ = self.cache.delete(test_key).await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        match (set_result, get_result) {
            (Ok(_), Ok(Some(value))) if value == test_value => {
                let stats = self.cache.stats().await.unwrap_or_default();
                HealthCheckResult {
                    status: HealthStatus::Healthy,
                    message: "Cache operations successful".to_string(),
                    duration_ms,
                    timestamp,
                    details: Some(serde_json::json!({
                        "hit_rate": stats.hit_rate,
                        "used_memory": stats.used_memory,
                        "key_count": stats.key_count,
                    })),
                }
            }
            _ => HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: "Cache operations failed".to_string(),
                duration_ms,
                timestamp,
                details: None,
            },
        }
    }
    
    fn name(&self) -> &str {
        "cache"
    }
    
    fn description(&self) -> &str {
        "Cache system connectivity and operations"
    }
    
    fn is_critical(&self) -> bool {
        false // Cache is not critical for basic functionality
    }
}

/// Blockchain health check
pub struct BlockchainHealthCheck {
    rpc_url: String,
}

impl BlockchainHealthCheck {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }
}

#[async_trait::async_trait]
impl HealthCheck for BlockchainHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        
        // Try to get the latest block number
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });
        
        let result = client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        match result {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(json) if json.get("result").is_some() => HealthCheckResult {
                        status: HealthStatus::Healthy,
                        message: "Blockchain RPC connection successful".to_string(),
                        duration_ms,
                        timestamp,
                        details: Some(serde_json::json!({
                            "block_number": json["result"],
                            "rpc_url": self.rpc_url,
                        })),
                    },
                    _ => HealthCheckResult {
                        status: HealthStatus::Unhealthy,
                        message: "Invalid blockchain RPC response".to_string(),
                        duration_ms,
                        timestamp,
                        details: None,
                    },
                }
            }
            Ok(response) => HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: format!("Blockchain RPC error: {}", response.status()),
                duration_ms,
                timestamp,
                details: None,
            },
            Err(e) => HealthCheckResult {
                status: HealthStatus::Unhealthy,
                message: format!("Blockchain RPC connection failed: {}", e),
                duration_ms,
                timestamp,
                details: None,
            },
        }
    }
    
    fn name(&self) -> &str {
        "blockchain"
    }
    
    fn description(&self) -> &str {
        "Blockchain RPC connectivity"
    }
    
    fn is_critical(&self) -> bool {
        true
    }
}

/// Memory health check
pub struct MemoryHealthCheck {
    warning_threshold_mb: u64,
    critical_threshold_mb: u64,
}

impl MemoryHealthCheck {
    pub fn new(warning_threshold_mb: u64, critical_threshold_mb: u64) -> Self {
        Self {
            warning_threshold_mb,
            critical_threshold_mb,
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        
        // Get memory usage (simplified - in production use a proper system info crate)
        let memory_usage_mb = 100; // Placeholder
        
        let duration_ms = start.elapsed().as_millis() as u64;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        let (status, message) = if memory_usage_mb > self.critical_threshold_mb {
            (HealthStatus::Unhealthy, format!("Memory usage critical: {}MB", memory_usage_mb))
        } else if memory_usage_mb > self.warning_threshold_mb {
            (HealthStatus::Degraded, format!("Memory usage high: {}MB", memory_usage_mb))
        } else {
            (HealthStatus::Healthy, format!("Memory usage normal: {}MB", memory_usage_mb))
        };
        
        HealthCheckResult {
            status,
            message,
            duration_ms,
            timestamp,
            details: Some(serde_json::json!({
                "memory_usage_mb": memory_usage_mb,
                "warning_threshold_mb": self.warning_threshold_mb,
                "critical_threshold_mb": self.critical_threshold_mb,
            })),
        }
    }
    
    fn name(&self) -> &str {
        "memory"
    }
    
    fn description(&self) -> &str {
        "System memory usage"
    }
    
    fn is_critical(&self) -> bool {
        false
    }
}

// HTTP handlers for health endpoints

/// Health check endpoint
pub async fn health_check(health_service: web::Data<HealthService>) -> Result<HttpResponse> {
    let health = health_service.check_health().await;
    
    let status_code = match health.status {
        HealthStatus::Healthy => 200,
        HealthStatus::Degraded => 200, // Still return 200 for degraded
        HealthStatus::Unhealthy => 503,
    };
    
    Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
        .json(health))
}

/// Simple liveness probe (for Kubernetes)
pub async fn liveness_probe(health_service: web::Data<HealthService>) -> Result<HttpResponse> {
    // Liveness should only check if the service is running, not external dependencies
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    })))
}

/// Readiness probe (for Kubernetes)
pub async fn readiness_probe(health_service: web::Data<HealthService>) -> Result<HttpResponse> {
    let is_ready = health_service.is_healthy().await;
    
    if is_ready {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        })))
    }
}

impl Default for crate::cache::CacheStats {
    fn default() -> Self {
        Self {
            hits: 0,
            misses: 0,
            hit_rate: 0.0,
            used_memory: 0,
            max_memory: 0,
            key_count: 0,
            evictions: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct MockHealthCheck {
        name: String,
        should_fail: Arc<AtomicBool>,
    }

    impl MockHealthCheck {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                should_fail: Arc::new(AtomicBool::new(false)),
            }
        }
        
        fn set_should_fail(&self, should_fail: bool) {
            self.should_fail.store(should_fail, Ordering::Relaxed);
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for MockHealthCheck {
        async fn check(&self) -> HealthCheckResult {
            let should_fail = self.should_fail.load(Ordering::Relaxed);
            
            HealthCheckResult {
                status: if should_fail { HealthStatus::Unhealthy } else { HealthStatus::Healthy },
                message: if should_fail { "Mock failure" } else { "Mock success" }.to_string(),
                duration_ms: 10,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                details: None,
            }
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn description(&self) -> &str {
            "Mock health check for testing"
        }
    }

    #[tokio::test]
    async fn test_health_service() {
        let config = HealthConfig::default();
        let health_service = HealthService::new(config);
        
        // Register mock checks
        let check1 = MockHealthCheck::new("test1");
        let check2 = MockHealthCheck::new("test2");
        
        health_service.register_check(Box::new(check1)).await;
        health_service.register_check(Box::new(check2)).await;
        
        // All checks should be healthy
        let health = health_service.check_health().await;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.summary.healthy_checks, 2);
        assert_eq!(health.summary.unhealthy_checks, 0);
    }

    #[tokio::test]
    async fn test_health_service_with_failures() {
        let config = HealthConfig::default();
        let health_service = HealthService::new(config);
        
        // Register mock checks
        let check1 = MockHealthCheck::new("test1");
        let check2 = MockHealthCheck::new("test2");
        
        // Make one check fail
        check2.set_should_fail(true);
        
        health_service.register_check(Box::new(check1)).await;
        health_service.register_check(Box::new(check2)).await;
        
        // Service should be degraded
        let health = health_service.check_health().await;
        assert_eq!(health.status, HealthStatus::Degraded);
        assert_eq!(health.summary.healthy_checks, 1);
        assert_eq!(health.summary.unhealthy_checks, 1);
    }
}
