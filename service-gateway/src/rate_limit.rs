// =====================================================================================
// File: service-gateway/src/rate_limit.rs
// Description: Rate limiting implementation for API Gateway
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::GatewayError;
use actix_web::HttpMessage;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, warn};

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub window_size: Duration,
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
            window_size: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        if elapsed > 0.0 {
            let new_tokens = elapsed * self.refill_rate;
            self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
            self.last_refill = now;
        }
    }

    fn tokens_available(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

/// Rate limiter implementation
pub struct RateLimiter {
    config: RateLimitConfig,
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            buckets: Arc::new(Mutex::new(HashMap::new())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Check if request is allowed for the given identifier
    pub async fn check_rate_limit(&self, identifier: &str) -> Result<(), GatewayError> {
        self.cleanup_if_needed().await;

        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets.entry(identifier.to_string()).or_insert_with(|| {
            TokenBucket::new(
                self.config.burst_size as f64,
                self.config.requests_per_minute as f64 / 60.0, // convert to per second
            )
        });

        if bucket.try_consume(1.0) {
            debug!("Rate limit check passed for identifier: {}", identifier);
            Ok(())
        } else {
            warn!("Rate limit exceeded for identifier: {}", identifier);
            Err(GatewayError::RateLimitExceeded)
        }
    }

    /// Get current rate limit status for identifier
    pub async fn get_rate_limit_status(&self, identifier: &str) -> RateLimitStatus {
        let mut buckets = self.buckets.lock().unwrap();

        if let Some(bucket) = buckets.get_mut(identifier) {
            let tokens_available = bucket.tokens_available();
            let requests_remaining = tokens_available.floor() as u32;
            let reset_time = if tokens_available < self.config.burst_size as f64 {
                let tokens_needed = self.config.burst_size as f64 - tokens_available;
                let seconds_to_wait = tokens_needed / bucket.refill_rate;
                Some(Instant::now() + Duration::from_secs_f64(seconds_to_wait))
            } else {
                None
            };

            RateLimitStatus {
                requests_remaining,
                reset_time,
                limit: self.config.requests_per_minute,
                window_size: self.config.window_size,
            }
        } else {
            RateLimitStatus {
                requests_remaining: self.config.burst_size,
                reset_time: None,
                limit: self.config.requests_per_minute,
                window_size: self.config.window_size,
            }
        }
    }

    /// Extract identifier from request (IP address, user ID, API key, etc.)
    pub fn extract_identifier(&self, req: &actix_web::dev::ServiceRequest) -> String {
        // Try to get user ID from authentication first
        if let Some(user_claims) = req.extensions().get::<core_security::UserClaims>() {
            return format!("user:{}", user_claims.sub);
        }

        // Try to get API key from headers
        if let Some(api_key) = req.headers().get("X-API-Key") {
            if let Ok(key_str) = api_key.to_str() {
                return format!("api_key:{}", key_str);
            }
        }

        // Fall back to IP address
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        format!("ip:{}", ip)
    }

    /// Get rate limit configuration for different user types
    pub fn get_rate_limit_config(&self, identifier: &str) -> RateLimitConfig {
        if identifier.starts_with("user:") {
            // Authenticated users get higher limits
            RateLimitConfig {
                requests_per_minute: 120,
                burst_size: 20,
                ..self.config.clone()
            }
        } else if identifier.starts_with("api_key:") {
            // API key users get even higher limits
            RateLimitConfig {
                requests_per_minute: 300,
                burst_size: 50,
                ..self.config.clone()
            }
        } else {
            // Anonymous users get default limits
            self.config.clone()
        }
    }

    /// Cleanup old buckets periodically
    async fn cleanup_if_needed(&self) {
        let should_cleanup = {
            let last_cleanup = self.last_cleanup.lock().unwrap();
            last_cleanup.elapsed() > self.config.cleanup_interval
        };

        if should_cleanup {
            self.cleanup_old_buckets().await;
            *self.last_cleanup.lock().unwrap() = Instant::now();
        }
    }

    /// Remove old, unused buckets to prevent memory leaks
    async fn cleanup_old_buckets(&self) {
        let mut buckets = self.buckets.lock().unwrap();
        let cutoff = Instant::now() - self.config.cleanup_interval;

        buckets.retain(|_, bucket| bucket.last_refill > cutoff);

        debug!(
            "Cleaned up old rate limit buckets, {} remaining",
            buckets.len()
        );
    }

    /// Reset rate limit for specific identifier (admin function)
    pub async fn reset_rate_limit(&self, identifier: &str) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.remove(identifier);
        debug!("Reset rate limit for identifier: {}", identifier);
    }

    /// Get all current rate limit statuses (admin function)
    pub async fn get_all_statuses(&self) -> HashMap<String, RateLimitStatus> {
        let mut buckets = self.buckets.lock().unwrap();
        let mut statuses = HashMap::new();

        for (identifier, bucket) in buckets.iter_mut() {
            let tokens_available = bucket.tokens_available();
            let requests_remaining = tokens_available.floor() as u32;
            let reset_time = if tokens_available < self.config.burst_size as f64 {
                let tokens_needed = self.config.burst_size as f64 - tokens_available;
                let seconds_to_wait = tokens_needed / bucket.refill_rate;
                Some(Instant::now() + Duration::from_secs_f64(seconds_to_wait))
            } else {
                None
            };

            statuses.insert(
                identifier.clone(),
                RateLimitStatus {
                    requests_remaining,
                    reset_time,
                    limit: self.config.requests_per_minute,
                    window_size: self.config.window_size,
                },
            );
        }

        statuses
    }
}

/// Rate limit status information
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub requests_remaining: u32,
    pub reset_time: Option<Instant>,
    pub limit: u32,
    pub window_size: Duration,
}

impl RateLimitStatus {
    /// Convert to HTTP headers
    pub fn to_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("X-RateLimit-Limit".to_string(), self.limit.to_string()),
            (
                "X-RateLimit-Remaining".to_string(),
                self.requests_remaining.to_string(),
            ),
            (
                "X-RateLimit-Window".to_string(),
                self.window_size.as_secs().to_string(),
            ),
        ];

        if let Some(reset_time) = self.reset_time {
            let reset_seconds = reset_time.duration_since(Instant::now()).as_secs();
            headers.push(("X-RateLimit-Reset".to_string(), reset_seconds.to_string()));
        }

        headers
    }
}

/// Adaptive rate limiter that adjusts limits based on system load
pub struct AdaptiveRateLimiter {
    base_limiter: RateLimiter,
    system_load_threshold: f64,
    load_reduction_factor: f64,
}

impl AdaptiveRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            base_limiter: RateLimiter::new(config),
            system_load_threshold: 0.8, // 80% CPU usage
            load_reduction_factor: 0.5, // Reduce limits by 50% under high load
        }
    }

    /// Check rate limit with adaptive adjustment based on system load
    pub async fn check_adaptive_rate_limit(&self, identifier: &str) -> Result<(), GatewayError> {
        let system_load = self.get_system_load().await;

        if system_load > self.system_load_threshold {
            // Under high load, apply stricter rate limiting
            let adjusted_identifier = format!("high_load:{}", identifier);
            self.base_limiter
                .check_rate_limit(&adjusted_identifier)
                .await
        } else {
            self.base_limiter.check_rate_limit(identifier).await
        }
    }

    /// Get current system load (simplified implementation)
    async fn get_system_load(&self) -> f64 {
        // In a real implementation, this would check actual system metrics
        // For now, return a mock value
        0.3 // 30% load
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10.0, 1.0); // 10 tokens, 1 token per second

        // Should be able to consume initial tokens
        assert!(bucket.try_consume(5.0));
        assert_eq!(bucket.tokens, 5.0);

        // Should not be able to consume more than available
        assert!(!bucket.try_consume(10.0));

        // Wait and check refill
        sleep(Duration::from_millis(1100)).await;
        bucket.refill();
        assert!(bucket.tokens > 5.0);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_minute: 60,
            burst_size: 5,
            window_size: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(300),
        };

        let limiter = RateLimiter::new(config);
        let identifier = "test_user";

        // Should allow initial requests up to burst size
        for _ in 0..5 {
            assert!(limiter.check_rate_limit(identifier).await.is_ok());
        }

        // Should reject additional requests
        assert!(limiter.check_rate_limit(identifier).await.is_err());

        // Check status
        let status = limiter.get_rate_limit_status(identifier).await;
        assert_eq!(status.requests_remaining, 0);
    }

    #[tokio::test]
    async fn test_rate_limit_status_headers() {
        let status = RateLimitStatus {
            requests_remaining: 45,
            reset_time: Some(Instant::now() + Duration::from_secs(30)),
            limit: 60,
            window_size: Duration::from_secs(60),
        };

        let headers = status.to_headers();
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Limit" && v == "60"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "X-RateLimit-Remaining" && v == "45"));
    }
}
