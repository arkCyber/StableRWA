// =====================================================================================
// File: core-utils/src/testing.rs
// Description: Testing utilities and helpers for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::UtilError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::info;

/// Test environment configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
    pub test_timeout: Duration,
    pub cleanup_on_drop: bool,
    pub log_level: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://test:test@localhost:5432/test_db".to_string(),
            redis_url: "redis://localhost:6379/1".to_string(),
            test_timeout: Duration::from_secs(30),
            cleanup_on_drop: true,
            log_level: "debug".to_string(),
        }
    }
}

/// Test context for managing test resources
pub struct TestContext {
    pub config: TestConfig,
    pub cleanup_tasks: Arc<RwLock<Vec<Box<dyn CleanupTask>>>>,
    pub start_time: Instant,
}

impl TestContext {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            cleanup_tasks: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// Add a cleanup task to be executed when the test context is dropped
    pub async fn add_cleanup_task(&self, task: Box<dyn CleanupTask>) {
        let mut tasks = self.cleanup_tasks.write().await;
        tasks.push(task);
    }

    /// Get elapsed time since test start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Create a test database connection
    pub async fn create_test_db_pool(&self) -> Result<sqlx::Pool<sqlx::Postgres>, UtilError> {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.config.database_url)
            .await
            .map_err(|e| {
                UtilError::ProcessingError(format!("Failed to create test DB pool: {}", e))
            })
    }

    /// Create a test Redis connection
    pub async fn create_test_redis_client(&self) -> Result<redis::Client, UtilError> {
        redis::Client::open(self.config.redis_url.as_str()).map_err(|e| {
            UtilError::ProcessingError(format!("Failed to create test Redis client: {}", e))
        })
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if self.config.cleanup_on_drop {
            info!("Test context cleanup initiated");
            // Note: In a real implementation, you'd want to handle async cleanup properly
            // This is a simplified version for demonstration
        }
    }
}

/// Trait for cleanup tasks
#[async_trait]
pub trait CleanupTask: Send + Sync {
    async fn cleanup(&self) -> Result<(), UtilError>;
}

/// HTTP test client for API testing
pub struct TestHttpClient {
    client: Client,
    base_url: String,
    default_headers: HashMap<String, String>,
}

impl TestHttpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            default_headers: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.default_headers
            .insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_auth_token(self, token: &str) -> Self {
        self.with_header("Authorization", &format!("Bearer {}", token))
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> Result<TestResponse, UtilError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);

        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| UtilError::NetworkError(e.to_string()))?;

        Ok(TestResponse::new(response).await?)
    }

    /// Make a POST request
    pub async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<TestResponse, UtilError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.post(&url).json(body);

        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| UtilError::NetworkError(e.to_string()))?;

        Ok(TestResponse::new(response).await?)
    }

    /// Make a PUT request
    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> Result<TestResponse, UtilError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.put(&url).json(body);

        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| UtilError::NetworkError(e.to_string()))?;

        Ok(TestResponse::new(response).await?)
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> Result<TestResponse, UtilError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.delete(&url);

        for (key, value) in &self.default_headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| UtilError::NetworkError(e.to_string()))?;

        Ok(TestResponse::new(response).await?)
    }
}

/// Test response wrapper
pub struct TestResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl TestResponse {
    async fn new(response: reqwest::Response) -> Result<Self, UtilError> {
        let status = response.status().as_u16();

        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        let body = response
            .text()
            .await
            .map_err(|e| UtilError::NetworkError(e.to_string()))?;

        Ok(Self {
            status,
            headers,
            body,
        })
    }

    /// Parse response body as JSON
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, UtilError> {
        serde_json::from_str(&self.body).map_err(|e| UtilError::SerializationError(e.to_string()))
    }

    /// Check if response is successful (2xx status)
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Assert response status
    pub fn assert_status(&self, expected: u16) -> &Self {
        assert_eq!(
            self.status, expected,
            "Expected status {}, got {}",
            expected, self.status
        );
        self
    }

    /// Assert response contains text
    pub fn assert_contains(&self, text: &str) -> &Self {
        assert!(
            self.body.contains(text),
            "Response body does not contain '{}'",
            text
        );
        self
    }

    /// Assert response header exists
    pub fn assert_header(&self, key: &str, value: &str) -> &Self {
        let actual = self
            .headers
            .get(key)
            .unwrap_or_else(|| panic!("Header '{}' not found", key));
        assert_eq!(
            actual, value,
            "Expected header '{}' to be '{}', got '{}'",
            key, value, actual
        );
        self
    }
}

/// Test database utilities
pub struct TestDatabase {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl TestDatabase {
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }

    /// Execute SQL and return affected rows
    pub async fn execute(&self, sql: &str) -> Result<u64, UtilError> {
        sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|e| UtilError::ProcessingError(e.to_string()))
    }

    /// Truncate all tables (for test cleanup)
    pub async fn truncate_all_tables(&self) -> Result<(), UtilError> {
        // Get all table names
        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT tablename FROM pg_tables WHERE schemaname = 'public'")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| UtilError::ProcessingError(e.to_string()))?;

        // Truncate each table
        for (table_name,) in tables {
            let sql = format!("TRUNCATE TABLE {} RESTART IDENTITY CASCADE", table_name);
            self.execute(&sql).await?;
        }

        Ok(())
    }

    /// Insert test data from SQL file
    pub async fn load_fixtures(&self, sql_content: &str) -> Result<(), UtilError> {
        sqlx::query(sql_content)
            .execute(&self.pool)
            .await
            .map_err(|e| UtilError::ProcessingError(e.to_string()))?;

        Ok(())
    }
}

/// Performance testing utilities
pub struct PerformanceTest {
    name: String,
    start_time: Instant,
    measurements: Vec<Duration>,
}

impl PerformanceTest {
    pub fn new(name: String) -> Self {
        Self {
            name,
            start_time: Instant::now(),
            measurements: Vec::new(),
        }
    }

    /// Record a measurement
    pub fn record(&mut self, duration: Duration) {
        self.measurements.push(duration);
    }

    /// Record current elapsed time as a measurement
    pub fn record_elapsed(&mut self) {
        let elapsed = self.start_time.elapsed();
        self.measurements.push(elapsed);
        self.start_time = Instant::now(); // Reset for next measurement
    }

    /// Get performance statistics
    pub fn stats(&self) -> PerformanceStats {
        if self.measurements.is_empty() {
            return PerformanceStats::default();
        }

        let total: Duration = self.measurements.iter().sum();
        let count = self.measurements.len();
        let average = total / count as u32;

        let mut sorted = self.measurements.clone();
        sorted.sort();

        let min = sorted[0];
        let max = sorted[count - 1];
        let median = sorted[count / 2];

        PerformanceStats {
            name: self.name.clone(),
            count,
            total,
            average,
            min,
            max,
            median,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub name: String,
    pub count: usize,
    pub total: Duration,
    pub average: Duration,
    pub min: Duration,
    pub max: Duration,
    pub median: Duration,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            count: 0,
            total: Duration::from_millis(0),
            average: Duration::from_millis(0),
            min: Duration::from_millis(0),
            max: Duration::from_millis(0),
            median: Duration::from_millis(0),
        }
    }
}

impl std::fmt::Display for PerformanceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Performance Stats for '{}': {} measurements, avg: {:?}, min: {:?}, max: {:?}, median: {:?}",
            self.name, self.count, self.average, self.min, self.max, self.median
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TestConfig::default();
        assert!(config.database_url.contains("test_db"));
        assert_eq!(config.test_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_context_creation() {
        let config = TestConfig::default();
        let context = TestContext::new(config);
        assert!(context.elapsed().as_millis() >= 0);
    }

    #[test]
    fn test_http_client_creation() {
        let client = TestHttpClient::new("http://localhost:8080".to_string())
            .with_header("Content-Type", "application/json")
            .with_auth_token("test-token");

        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(
            client.default_headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            client.default_headers.get("Authorization"),
            Some(&"Bearer test-token".to_string())
        );
    }

    #[test]
    fn test_performance_test() {
        let mut perf_test = PerformanceTest::new("test_operation".to_string());

        perf_test.record(Duration::from_millis(100));
        perf_test.record(Duration::from_millis(200));
        perf_test.record(Duration::from_millis(150));

        let stats = perf_test.stats();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, Duration::from_millis(100));
        assert_eq!(stats.max, Duration::from_millis(200));
        assert_eq!(stats.median, Duration::from_millis(150));
    }
}
