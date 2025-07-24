// =====================================================================================
// File: core-observability/src/health.rs
// Description: Health check and readiness probe implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

// Removed unused import: crate::ObservabilityError
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Health check status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}

impl HealthCheckResult {
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: None,
            details: HashMap::new(),
            duration: Duration::from_millis(0),
            timestamp: Utc::now(),
        }
    }

    pub fn unhealthy(message: String) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message),
            details: HashMap::new(),
            duration: Duration::from_millis(0),
            timestamp: Utc::now(),
        }
    }

    pub fn degraded(message: String) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: Some(message),
            details: HashMap::new(),
            duration: Duration::from_millis(0),
            timestamp: Utc::now(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: serde_json::Value) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of the health check
    fn name(&self) -> &str;

    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;

    /// Whether this check is critical for overall health
    fn is_critical(&self) -> bool {
        true
    }

    /// Timeout for this health check
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

/// Database health check
pub struct DatabaseHealthCheck {
    name: String,
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl DatabaseHealthCheck {
    pub fn new(name: String, pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { name, pool }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => {
                let duration = start.elapsed();
                HealthCheckResult::healthy()
                    .with_duration(duration)
                    .with_detail(
                        "connection_pool_size",
                        serde_json::Value::Number(
                            serde_json::Number::from(self.pool.size() as u64),
                        ),
                    )
                    .with_detail(
                        "idle_connections",
                        serde_json::Value::Number(serde_json::Number::from(
                            self.pool.num_idle() as u64
                        )),
                    )
            }
            Err(e) => {
                let duration = start.elapsed();
                HealthCheckResult::unhealthy(format!("Database connection failed: {}", e))
                    .with_duration(duration)
            }
        }
    }

    fn is_critical(&self) -> bool {
        true
    }
}

/// Redis health check
pub struct RedisHealthCheck {
    name: String,
    client: redis::Client,
}

impl RedisHealthCheck {
    pub fn new(name: String, client: redis::Client) -> Self {
        Self { name, client }
    }
}

#[async_trait]
impl HealthCheck for RedisHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        match self.client.get_async_connection().await {
            Ok(mut conn) => match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                Ok(response) if response == "PONG" => {
                    let duration = start.elapsed();
                    HealthCheckResult::healthy().with_duration(duration)
                }
                Ok(response) => {
                    let duration = start.elapsed();
                    HealthCheckResult::degraded(format!("Unexpected Redis response: {}", response))
                        .with_duration(duration)
                }
                Err(e) => {
                    let duration = start.elapsed();
                    HealthCheckResult::unhealthy(format!("Redis ping failed: {}", e))
                        .with_duration(duration)
                }
            },
            Err(e) => {
                let duration = start.elapsed();
                HealthCheckResult::unhealthy(format!("Redis connection failed: {}", e))
                    .with_duration(duration)
            }
        }
    }

    fn is_critical(&self) -> bool {
        false // Redis is often used for caching, not critical
    }
}

/// External service health check
pub struct ExternalServiceHealthCheck {
    name: String,
    url: String,
    client: reqwest::Client,
    expected_status: u16,
}

impl ExternalServiceHealthCheck {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            client: reqwest::Client::new(),
            expected_status: 200,
        }
    }

    pub fn with_expected_status(mut self, status: u16) -> Self {
        self.expected_status = status;
        self
    }
}

#[async_trait]
impl HealthCheck for ExternalServiceHealthCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        match self.client.get(&self.url).send().await {
            Ok(response) => {
                let duration = start.elapsed();
                let status_code = response.status().as_u16();

                if status_code == self.expected_status {
                    HealthCheckResult::healthy()
                        .with_duration(duration)
                        .with_detail(
                            "status_code",
                            serde_json::Value::Number(serde_json::Number::from(status_code as u64)),
                        )
                } else {
                    HealthCheckResult::degraded(format!(
                        "Unexpected status code: {} (expected: {})",
                        status_code, self.expected_status
                    ))
                    .with_duration(duration)
                    .with_detail(
                        "status_code",
                        serde_json::Value::Number(serde_json::Number::from(status_code as u64)),
                    )
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                HealthCheckResult::unhealthy(format!("HTTP request failed: {}", e))
                    .with_duration(duration)
            }
        }
    }

    fn is_critical(&self) -> bool {
        false
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}

/// Health check manager
pub struct HealthCheckManager {
    checks: Arc<RwLock<HashMap<String, Box<dyn HealthCheck>>>>,
    last_results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
}

impl HealthCheckManager {
    pub fn new() -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            last_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a health check
    pub async fn register(&self, check: Box<dyn HealthCheck>) {
        let name = check.name().to_string();
        let mut checks = self.checks.write().await;
        checks.insert(name, check);
    }

    /// Run all health checks
    pub async fn check_all(&self) -> OverallHealthResult {
        let checks = self.checks.read().await;
        let mut results = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;
        let start = Instant::now();

        for (name, check) in checks.iter() {
            debug!("Running health check: {}", name);

            let result = tokio::time::timeout(check.timeout(), check.check()).await;

            let check_result = match result {
                Ok(result) => result,
                Err(_) => HealthCheckResult::unhealthy(format!(
                    "Health check timed out after {:?}",
                    check.timeout()
                )),
            };

            // Update overall status based on individual check results
            match (&overall_status, &check_result.status, check.is_critical()) {
                (_, HealthStatus::Unhealthy, true) => overall_status = HealthStatus::Unhealthy,
                (HealthStatus::Healthy, HealthStatus::Degraded, _) => {
                    overall_status = HealthStatus::Degraded
                }
                (HealthStatus::Healthy, HealthStatus::Unhealthy, false) => {
                    overall_status = HealthStatus::Degraded
                }
                _ => {}
            }

            info!(
                check_name = name,
                status = %check_result.status,
                duration_ms = check_result.duration.as_millis(),
                "Health check completed"
            );

            results.insert(name.clone(), check_result);
        }

        // Update cached results
        {
            let mut last_results = self.last_results.write().await;
            *last_results = results.clone();
        }

        let overall_duration = start.elapsed();

        OverallHealthResult {
            status: overall_status,
            checks: results,
            duration: overall_duration,
            timestamp: Utc::now(),
        }
    }

    /// Get last health check results without running checks
    pub async fn get_last_results(&self) -> Option<OverallHealthResult> {
        let results = self.last_results.read().await;

        if results.is_empty() {
            return None;
        }

        let mut overall_status = HealthStatus::Healthy;
        let mut total_duration = Duration::from_millis(0);
        let mut oldest_timestamp = Utc::now();

        for result in results.values() {
            total_duration += result.duration;
            if result.timestamp < oldest_timestamp {
                oldest_timestamp = result.timestamp;
            }

            match &overall_status {
                HealthStatus::Healthy => {
                    if result.status == HealthStatus::Unhealthy
                        || result.status == HealthStatus::Degraded
                    {
                        overall_status = result.status.clone();
                    }
                }
                HealthStatus::Degraded => {
                    if result.status == HealthStatus::Unhealthy {
                        overall_status = HealthStatus::Unhealthy;
                    }
                }
                _ => {}
            }
        }

        Some(OverallHealthResult {
            status: overall_status,
            checks: results.clone(),
            duration: total_duration,
            timestamp: oldest_timestamp,
        })
    }

    /// Check if the service is ready (all critical checks pass)
    pub async fn is_ready(&self) -> bool {
        let result = self.check_all().await;
        result.status == HealthStatus::Healthy || result.status == HealthStatus::Degraded
    }

    /// Check if the service is alive (basic liveness check)
    pub async fn is_alive(&self) -> bool {
        // Simple liveness check - service is alive if it can respond
        true
    }
}

/// Overall health result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallHealthResult {
    pub status: HealthStatus,
    pub checks: HashMap<String, HealthCheckResult>,
    pub duration: Duration,
    pub timestamp: DateTime<Utc>,
}

impl OverallHealthResult {
    /// Convert to HTTP status code
    pub fn to_http_status(&self) -> u16 {
        match self.status {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still serving traffic
            HealthStatus::Unhealthy => 503,
            HealthStatus::Unknown => 503,
        }
    }

    /// Get summary information
    pub fn summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        summary.insert(
            "status".to_string(),
            serde_json::Value::String(self.status.to_string()),
        );
        summary.insert(
            "timestamp".to_string(),
            serde_json::Value::String(self.timestamp.to_rfc3339()),
        );
        summary.insert(
            "duration_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.duration.as_millis() as u64)),
        );
        summary.insert(
            "total_checks".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.checks.len() as u64)),
        );

        let healthy_count = self
            .checks
            .values()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count();
        summary.insert(
            "healthy_checks".to_string(),
            serde_json::Value::Number(serde_json::Number::from(healthy_count as u64)),
        );

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHealthCheck {
        name: String,
        result: HealthCheckResult,
        is_critical: bool,
    }

    impl MockHealthCheck {
        fn new(name: &str, status: HealthStatus, is_critical: bool) -> Self {
            let result = match status {
                HealthStatus::Healthy => HealthCheckResult::healthy(),
                HealthStatus::Unhealthy => {
                    HealthCheckResult::unhealthy("Mock unhealthy".to_string())
                }
                HealthStatus::Degraded => HealthCheckResult::degraded("Mock degraded".to_string()),
                HealthStatus::Unknown => HealthCheckResult::healthy(), // Default to healthy for unknown
            };

            Self {
                name: name.to_string(),
                result,
                is_critical,
            }
        }
    }

    #[async_trait]
    impl HealthCheck for MockHealthCheck {
        fn name(&self) -> &str {
            &self.name
        }

        async fn check(&self) -> HealthCheckResult {
            self.result.clone()
        }

        fn is_critical(&self) -> bool {
            self.is_critical
        }
    }

    #[tokio::test]
    async fn test_health_check_manager() {
        let manager = HealthCheckManager::new();

        // Register health checks
        manager
            .register(Box::new(MockHealthCheck::new(
                "db",
                HealthStatus::Healthy,
                true,
            )))
            .await;
        manager
            .register(Box::new(MockHealthCheck::new(
                "cache",
                HealthStatus::Degraded,
                false,
            )))
            .await;

        // Run health checks
        let result = manager.check_all().await;

        assert_eq!(result.status, HealthStatus::Degraded); // Degraded because cache is degraded
        assert_eq!(result.checks.len(), 2);
        assert!(result.checks.contains_key("db"));
        assert!(result.checks.contains_key("cache"));
    }

    #[tokio::test]
    async fn test_critical_unhealthy_check() {
        let manager = HealthCheckManager::new();

        manager
            .register(Box::new(MockHealthCheck::new(
                "db",
                HealthStatus::Unhealthy,
                true,
            )))
            .await;
        manager
            .register(Box::new(MockHealthCheck::new(
                "cache",
                HealthStatus::Healthy,
                false,
            )))
            .await;

        let result = manager.check_all().await;

        assert_eq!(result.status, HealthStatus::Unhealthy); // Unhealthy because critical check failed
    }

    #[tokio::test]
    async fn test_non_critical_unhealthy_check() {
        let manager = HealthCheckManager::new();

        manager
            .register(Box::new(MockHealthCheck::new(
                "db",
                HealthStatus::Healthy,
                true,
            )))
            .await;
        manager
            .register(Box::new(MockHealthCheck::new(
                "cache",
                HealthStatus::Unhealthy,
                false,
            )))
            .await;

        let result = manager.check_all().await;

        assert_eq!(result.status, HealthStatus::Degraded); // Degraded because non-critical check failed
    }

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult::healthy()
            .with_detail("key", serde_json::Value::String("value".to_string()))
            .with_duration(Duration::from_millis(100));

        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.duration, Duration::from_millis(100));
        assert!(result.details.contains_key("key"));
    }

    #[test]
    fn test_overall_health_result_http_status() {
        let result = OverallHealthResult {
            status: HealthStatus::Healthy,
            checks: HashMap::new(),
            duration: Duration::from_millis(0),
            timestamp: Utc::now(),
        };
        assert_eq!(result.to_http_status(), 200);

        let result = OverallHealthResult {
            status: HealthStatus::Unhealthy,
            checks: HashMap::new(),
            duration: Duration::from_millis(0),
            timestamp: Utc::now(),
        };
        assert_eq!(result.to_http_status(), 503);
    }
}
