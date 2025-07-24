// =====================================================================================
// File: core-security/src/rate_limit.rs
// Description: Rate limiting implementation for API protection
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::SecurityError;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{interval, Interval};
use tracing::{debug, info, warn};

/// Rate limiter trait
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if request is allowed for the given key
    async fn is_allowed(
        &self,
        key: &str,
        limit: u32,
        window_seconds: u64,
    ) -> Result<bool, SecurityError>;

    /// Get current usage for a key
    async fn get_usage(&self, key: &str) -> Result<Option<RateLimitInfo>, SecurityError>;

    /// Reset rate limit for a key
    async fn reset(&self, key: &str) -> Result<(), SecurityError>;

    /// Clean up expired entries
    async fn cleanup(&self) -> Result<(), SecurityError>;
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub key: String,
    pub requests: u32,
    pub limit: u32,
    pub window_start: DateTime<Utc>,
    pub window_seconds: u64,
    pub reset_time: DateTime<Utc>,
}

impl RateLimitInfo {
    pub fn remaining(&self) -> u32 {
        if self.requests >= self.limit {
            0
        } else {
            self.limit - self.requests
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.reset_time
    }

    pub fn seconds_until_reset(&self) -> i64 {
        let now = Utc::now();
        if now >= self.reset_time {
            0
        } else {
            (self.reset_time - now).num_seconds()
        }
    }
}

/// In-memory rate limiter using sliding window
pub struct InMemoryRateLimiter {
    storage: Arc<RwLock<HashMap<String, RateLimitInfo>>>,
    cleanup_interval: Interval,
}

impl InMemoryRateLimiter {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: interval(tokio::time::Duration::from_secs(60)), // Cleanup every minute
        }
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(self: Arc<Self>) {
        let limiter = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(limiter.cleanup_interval.period());
            loop {
                interval.tick().await;
                if let Err(e) = limiter.cleanup().await {
                    warn!("Rate limiter cleanup failed: {}", e);
                }
            }
        });
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn is_allowed(
        &self,
        key: &str,
        limit: u32,
        window_seconds: u64,
    ) -> Result<bool, SecurityError> {
        let now = Utc::now();
        let window_duration = Duration::seconds(window_seconds as i64);

        let mut storage = self
            .storage
            .write()
            .map_err(|e| SecurityError::ValidationError(format!("Lock error: {}", e)))?;

        let entry = storage
            .entry(key.to_string())
            .or_insert_with(|| RateLimitInfo {
                key: key.to_string(),
                requests: 0,
                limit,
                window_start: now,
                window_seconds,
                reset_time: now + window_duration,
            });

        // Check if window has expired
        if entry.is_expired() {
            // Reset the window
            entry.requests = 0;
            entry.window_start = now;
            entry.reset_time = now + window_duration;
            entry.limit = limit;
            entry.window_seconds = window_seconds;
        }

        // Check if request is allowed
        if entry.requests >= entry.limit {
            debug!("Rate limit exceeded for key: {}", key);
            return Ok(false);
        }

        // Increment request count
        entry.requests += 1;
        debug!(
            "Rate limit check for key: {} - {}/{}",
            key, entry.requests, entry.limit
        );

        Ok(true)
    }

    async fn get_usage(&self, key: &str) -> Result<Option<RateLimitInfo>, SecurityError> {
        let storage = self
            .storage
            .read()
            .map_err(|e| SecurityError::ValidationError(format!("Lock error: {}", e)))?;

        Ok(storage.get(key).cloned())
    }

    async fn reset(&self, key: &str) -> Result<(), SecurityError> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SecurityError::ValidationError(format!("Lock error: {}", e)))?;

        storage.remove(key);
        info!("Rate limit reset for key: {}", key);
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), SecurityError> {
        let mut storage = self
            .storage
            .write()
            .map_err(|e| SecurityError::ValidationError(format!("Lock error: {}", e)))?;

        let now = Utc::now();
        let initial_count = storage.len();

        storage.retain(|_, info| {
            // Keep entries that are not expired or have been used recently
            !info.is_expired() || (now - info.window_start).num_seconds() < 3600
            // Keep for 1 hour
        });

        let removed_count = initial_count - storage.len();
        if removed_count > 0 {
            info!("Cleaned up {} expired rate limit entries", removed_count);
        }

        Ok(())
    }
}

/// Rate limit configuration for different endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub rules: HashMap<String, RateLimitRule>,
    pub default_rule: RateLimitRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut rules = HashMap::new();

        // Authentication endpoints - stricter limits
        rules.insert(
            "/auth/login".to_string(),
            RateLimitRule {
                requests_per_minute: 5,
                requests_per_hour: 20,
                requests_per_day: 100,
                burst_size: 2,
            },
        );

        // Asset endpoints - moderate limits
        rules.insert(
            "/api/v1/assets".to_string(),
            RateLimitRule {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                requests_per_day: 10000,
                burst_size: 10,
            },
        );

        // Payment endpoints - strict limits
        rules.insert(
            "/api/v1/payments".to_string(),
            RateLimitRule {
                requests_per_minute: 10,
                requests_per_hour: 100,
                requests_per_day: 500,
                burst_size: 3,
            },
        );

        Self {
            rules,
            default_rule: RateLimitRule {
                requests_per_minute: 100,
                requests_per_hour: 2000,
                requests_per_day: 20000,
                burst_size: 20,
            },
        }
    }
}

impl RateLimitConfig {
    pub fn get_rule(&self, endpoint: &str) -> &RateLimitRule {
        self.rules.get(endpoint).unwrap_or(&self.default_rule)
    }
}

/// Rate limit key generators
pub struct RateLimitKeyGenerator;

impl RateLimitKeyGenerator {
    /// Generate key based on IP address
    pub fn by_ip(ip: &str) -> String {
        format!("ip:{}", ip)
    }

    /// Generate key based on user ID
    pub fn by_user(user_id: &str) -> String {
        format!("user:{}", user_id)
    }

    /// Generate key based on API key
    pub fn by_api_key(api_key: &str) -> String {
        format!("api_key:{}", api_key)
    }

    /// Generate key based on IP and endpoint
    pub fn by_ip_endpoint(ip: &str, endpoint: &str) -> String {
        format!("ip:{}:endpoint:{}", ip, endpoint)
    }

    /// Generate key based on user and endpoint
    pub fn by_user_endpoint(user_id: &str, endpoint: &str) -> String {
        format!("user:{}:endpoint:{}", user_id, endpoint)
    }
}

/// Rate limit middleware helper
pub struct RateLimitMiddleware {
    limiter: Arc<dyn RateLimiter>,
    config: RateLimitConfig,
}

impl RateLimitMiddleware {
    pub fn new(limiter: Arc<dyn RateLimiter>, config: RateLimitConfig) -> Self {
        Self { limiter, config }
    }

    /// Check rate limit for a request
    pub async fn check_rate_limit(
        &self,
        key: &str,
        endpoint: &str,
        window: RateLimitWindow,
    ) -> Result<RateLimitResult, SecurityError> {
        let rule = self.config.get_rule(endpoint);

        let (limit, window_seconds) = match window {
            RateLimitWindow::Minute => (rule.requests_per_minute, 60),
            RateLimitWindow::Hour => (rule.requests_per_hour, 3600),
            RateLimitWindow::Day => (rule.requests_per_day, 86400),
        };

        let allowed = self.limiter.is_allowed(key, limit, window_seconds).await?;
        let usage = self
            .limiter
            .get_usage(key)
            .await?
            .unwrap_or_else(|| RateLimitInfo {
                key: key.to_string(),
                requests: 0,
                limit,
                window_start: Utc::now(),
                window_seconds,
                reset_time: Utc::now() + Duration::seconds(window_seconds as i64),
            });

        Ok(RateLimitResult {
            allowed,
            usage,
            window,
        })
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitWindow {
    Minute,
    Hour,
    Day,
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub usage: RateLimitInfo,
    pub window: RateLimitWindow,
}

impl RateLimitResult {
    /// Get headers for HTTP response
    pub fn to_headers(&self) -> Vec<(String, String)> {
        vec![
            (
                "X-RateLimit-Limit".to_string(),
                self.usage.limit.to_string(),
            ),
            (
                "X-RateLimit-Remaining".to_string(),
                self.usage.remaining().to_string(),
            ),
            (
                "X-RateLimit-Reset".to_string(),
                self.usage.reset_time.timestamp().to_string(),
            ),
            (
                "X-RateLimit-Window".to_string(),
                format!("{:?}", self.window),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_in_memory_rate_limiter() {
        let limiter = InMemoryRateLimiter::new();
        let key = "test_key";

        // First request should be allowed
        assert!(limiter.is_allowed(key, 2, 60).await.unwrap());

        // Second request should be allowed
        assert!(limiter.is_allowed(key, 2, 60).await.unwrap());

        // Third request should be denied
        assert!(!limiter.is_allowed(key, 2, 60).await.unwrap());

        // Check usage
        let usage = limiter.get_usage(key).await.unwrap().unwrap();
        assert_eq!(usage.requests, 2);
        assert_eq!(usage.limit, 2);
        assert_eq!(usage.remaining(), 0);
    }

    #[tokio::test]
    async fn test_rate_limit_reset() {
        let limiter = InMemoryRateLimiter::new();
        let key = "test_key";

        // Use up the limit
        assert!(limiter.is_allowed(key, 1, 1).await.unwrap());
        assert!(!limiter.is_allowed(key, 1, 1).await.unwrap());

        // Wait for window to expire
        sleep(TokioDuration::from_secs(2)).await;

        // Should be allowed again
        assert!(limiter.is_allowed(key, 1, 1).await.unwrap());
    }

    #[tokio::test]
    async fn test_rate_limit_manual_reset() {
        let limiter = InMemoryRateLimiter::new();
        let key = "test_key";

        // Use up the limit
        assert!(limiter.is_allowed(key, 1, 60).await.unwrap());
        assert!(!limiter.is_allowed(key, 1, 60).await.unwrap());

        // Manual reset
        limiter.reset(key).await.unwrap();

        // Should be allowed again
        assert!(limiter.is_allowed(key, 1, 60).await.unwrap());
    }

    #[test]
    fn test_rate_limit_key_generator() {
        assert_eq!(
            RateLimitKeyGenerator::by_ip("192.168.1.1"),
            "ip:192.168.1.1"
        );
        assert_eq!(RateLimitKeyGenerator::by_user("user123"), "user:user123");
        assert_eq!(
            RateLimitKeyGenerator::by_ip_endpoint("192.168.1.1", "/api/v1/assets"),
            "ip:192.168.1.1:endpoint:/api/v1/assets"
        );
    }

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::default();

        let auth_rule = config.get_rule("/auth/login");
        assert_eq!(auth_rule.requests_per_minute, 5);

        let unknown_rule = config.get_rule("/unknown/endpoint");
        assert_eq!(unknown_rule.requests_per_minute, 100); // Default rule
    }

    #[test]
    fn test_rate_limit_info() {
        let info = RateLimitInfo {
            key: "test".to_string(),
            requests: 5,
            limit: 10,
            window_start: Utc::now(),
            window_seconds: 60,
            reset_time: Utc::now() + Duration::seconds(60),
        };

        assert_eq!(info.remaining(), 5);
        assert!(!info.is_expired());
        assert!(info.seconds_until_reset() > 0);
    }
}
