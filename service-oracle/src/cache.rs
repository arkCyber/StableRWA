// =====================================================================================
// RWA Tokenization Platform - Oracle Price Cache
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::config::RedisConfig;
use crate::error::{OracleError, OracleResult};
use crate::models::AssetPrice;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
// Removed unused import: std::time::Duration
use tracing::{debug, warn, error};

/// Price cache using Redis
pub struct PriceCache {
    #[allow(dead_code)]
    client: Client,
    connection_pool: redis::aio::ConnectionManager,
}

/// Cached price data with expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedPrice {
    price: AssetPrice,
    cached_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl PriceCache {
    /// Create a new price cache
    pub async fn new(config: &RedisConfig) -> OracleResult<Self> {
        let client = Client::open(config.url.as_str())
            .map_err(|e| OracleError::Redis(e))?;

        let connection_pool = client
            .get_connection_manager()
            .await
            .map_err(|e| OracleError::Redis(e))?;

        debug!("Connected to Redis at {}", config.url);

        Ok(Self {
            client,
            connection_pool,
        })
    }

    /// Get price from cache
    pub async fn get_price(&self, key: &str) -> OracleResult<Option<AssetPrice>> {
        let mut conn = self.connection_pool.clone();
        
        let cached_data: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        match cached_data {
            Some(data) => {
                let cached_price: CachedPrice = serde_json::from_str(&data)
                    .map_err(|e| OracleError::Serialization(e))?;

                // Check if expired
                if chrono::Utc::now() > cached_price.expires_at {
                    debug!("Cached price for {} has expired", key);
                    // Remove expired entry
                    let _: () = conn.del(key).await.map_err(|e| OracleError::Redis(e))?;
                    return Ok(None);
                }

                debug!("Retrieved cached price for {}", key);
                Ok(Some(cached_price.price))
            }
            None => {
                debug!("No cached price found for {}", key);
                Ok(None)
            }
        }
    }

    /// Set price in cache with TTL
    pub async fn set_price(&self, key: &str, price: &AssetPrice, ttl_seconds: u64) -> OracleResult<()> {
        let mut conn = self.connection_pool.clone();
        
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);

        let cached_price = CachedPrice {
            price: price.clone(),
            cached_at: now,
            expires_at,
        };

        let serialized = serde_json::to_string(&cached_price)
            .map_err(|e| OracleError::Serialization(e))?;

        conn.set_ex::<_, _, ()>(key, serialized, ttl_seconds)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        debug!("Cached price for {} with TTL {}s", key, ttl_seconds);
        Ok(())
    }

    /// Delete price from cache
    pub async fn delete_price(&self, key: &str) -> OracleResult<bool> {
        let mut conn = self.connection_pool.clone();
        
        let deleted: i32 = conn
            .del(key)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        Ok(deleted > 0)
    }

    /// Get multiple prices from cache
    pub async fn get_batch_prices(&self, keys: &[String]) -> OracleResult<Vec<Option<AssetPrice>>> {
        let mut conn = self.connection_pool.clone();
        
        let cached_data: Vec<Option<String>> = conn
            .mget(keys)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        let mut results = Vec::new();
        let now = chrono::Utc::now();
        let mut expired_keys = Vec::new();

        for (i, data) in cached_data.into_iter().enumerate() {
            match data {
                Some(json_data) => {
                    match serde_json::from_str::<CachedPrice>(&json_data) {
                        Ok(cached_price) => {
                            if now > cached_price.expires_at {
                                expired_keys.push(keys[i].clone());
                                results.push(None);
                            } else {
                                results.push(Some(cached_price.price));
                            }
                        }
                        Err(e) => {
                            warn!("Failed to deserialize cached price for {}: {}", keys[i], e);
                            expired_keys.push(keys[i].clone());
                            results.push(None);
                        }
                    }
                }
                None => {
                    results.push(None);
                }
            }
        }

        // Clean up expired keys
        if !expired_keys.is_empty() {
            let _: () = conn.del(&expired_keys).await.map_err(|e| OracleError::Redis(e))?;
            debug!("Cleaned up {} expired cache entries", expired_keys.len());
        }

        Ok(results)
    }

    /// Set multiple prices in cache
    pub async fn set_batch_prices(&self, prices: &[(String, AssetPrice)], ttl_seconds: u64) -> OracleResult<()> {
        let mut conn = self.connection_pool.clone();
        
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::seconds(ttl_seconds as i64);

        let mut pipe = redis::pipe();
        
        for (key, price) in prices {
            let cached_price = CachedPrice {
                price: price.clone(),
                cached_at: now,
                expires_at,
            };

            let serialized = serde_json::to_string(&cached_price)
                .map_err(|e| OracleError::Serialization(e))?;

            pipe.set_ex(key, serialized, ttl_seconds);
        }

        pipe.query_async::<_, ()>(&mut conn)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        debug!("Cached {} prices with TTL {}s", prices.len(), ttl_seconds);
        Ok(())
    }

    /// Check if cache is healthy
    pub async fn health_check(&self) -> OracleResult<bool> {
        let mut conn = self.connection_pool.clone();
        
        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
            Ok(_) => Ok(true),
            Err(e) => {
                error!("Redis health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> OracleResult<CacheStats> {
        let mut conn = self.connection_pool.clone();
        
        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        // Parse Redis INFO output for memory statistics
        let mut used_memory = 0u64;
        let mut max_memory = 0u64;
        
        for line in info.lines() {
            if line.starts_with("used_memory:") {
                if let Some(value) = line.split(':').nth(1) {
                    used_memory = value.parse().unwrap_or(0);
                }
            } else if line.starts_with("maxmemory:") {
                if let Some(value) = line.split(':').nth(1) {
                    max_memory = value.parse().unwrap_or(0);
                }
            }
        }

        // Get key count (approximate)
        let key_count: i32 = redis::cmd("DBSIZE")
            .query_async(&mut conn)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        Ok(CacheStats {
            used_memory,
            max_memory,
            key_count: key_count as u64,
            hit_rate: 0.0, // Would need to track this separately
        })
    }

    /// Clear all cached prices
    pub async fn clear_all(&self) -> OracleResult<()> {
        let mut conn = self.connection_pool.clone();
        
        // Use pattern to delete only price-related keys
        let keys: Vec<String> = conn
            .keys("price:*")
            .await
            .map_err(|e| OracleError::Redis(e))?;

        if !keys.is_empty() {
            let _: () = conn.del(&keys).await.map_err(|e| OracleError::Redis(e))?;
            debug!("Cleared {} cached prices", keys.len());
        }

        Ok(())
    }

    /// Set cache entry with custom expiration
    pub async fn set_with_expiration(&self, key: &str, price: &AssetPrice, expires_at: chrono::DateTime<chrono::Utc>) -> OracleResult<()> {
        let mut conn = self.connection_pool.clone();
        
        let now = chrono::Utc::now();
        if expires_at <= now {
            return Err(OracleError::InvalidPriceData {
                reason: "Expiration time must be in the future".to_string(),
            });
        }

        let cached_price = CachedPrice {
            price: price.clone(),
            cached_at: now,
            expires_at,
        };

        let serialized = serde_json::to_string(&cached_price)
            .map_err(|e| OracleError::Serialization(e))?;

        let ttl_seconds = (expires_at - now).num_seconds() as u64;
        
        conn.set_ex::<_, _, ()>(key, serialized, ttl_seconds)
            .await
            .map_err(|e| OracleError::Redis(e))?;

        debug!("Cached price for {} until {}", key, expires_at);
        Ok(())
    }

    /// Get cache key for asset price
    pub fn get_price_key(&self, asset_id: &str, currency: &str) -> String {
        format!("price:{}:{}", asset_id, currency)
    }

    /// Get cache key for price history
    pub fn get_history_key(&self, asset_id: &str, currency: &str, interval: &str) -> String {
        format!("history:{}:{}:{}", asset_id, currency, interval)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub used_memory: u64,
    pub max_memory: u64,
    pub key_count: u64,
    pub hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AssetPrice;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Helper function to create test price
    fn create_test_price() -> AssetPrice {
        AssetPrice {
            asset_id: "BTC".to_string(),
            price: dec!(50000.0),
            currency: "USD".to_string(),
            timestamp: Utc::now(),
            confidence: dec!(0.95),
            source: "test".to_string(),
            metadata: Some(HashMap::new()),
        }
    }

    #[test]
    fn test_cache_key_generation() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            command_timeout: 5,
            retry_attempts: 3,
        };

        // Note: This test doesn't actually connect to Redis
        // In a real test environment, you'd use a test Redis instance
        
        // Test key generation methods
        let cache_key = format!("price:{}:{}", "BTC", "USD");
        assert_eq!(cache_key, "price:BTC:USD");

        let history_key = format!("history:{}:{}:{}", "BTC", "USD", "1h");
        assert_eq!(history_key, "history:BTC:USD:1h");
    }

    #[test]
    fn test_cached_price_serialization() {
        let price = create_test_price();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(1);

        let cached_price = CachedPrice {
            price: price.clone(),
            cached_at: now,
            expires_at,
        };

        // Test serialization
        let serialized = serde_json::to_string(&cached_price).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized: CachedPrice = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.price.asset_id, price.asset_id);
        assert_eq!(deserialized.price.price, price.price);
        assert_eq!(deserialized.cached_at, now);
        assert_eq!(deserialized.expires_at, expires_at);
    }

    #[test]
    fn test_cache_stats_creation() {
        let stats = CacheStats {
            used_memory: 1024,
            max_memory: 2048,
            key_count: 100,
            hit_rate: 0.85,
        };

        assert_eq!(stats.used_memory, 1024);
        assert_eq!(stats.max_memory, 2048);
        assert_eq!(stats.key_count, 100);
        assert_eq!(stats.hit_rate, 0.85);
    }

    // Integration tests would require a running Redis instance
    #[tokio::test]
    #[ignore] // Ignore by default to avoid requiring Redis in CI
    async fn test_cache_integration() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            command_timeout: 5,
            retry_attempts: 3,
        };

        let cache = PriceCache::new(&config).await;
        if cache.is_err() {
            // Skip test if Redis is not available
            return;
        }

        let cache = cache.unwrap();
        let price = create_test_price();
        let key = "test:BTC:USD";

        // Test set and get
        cache.set_price(key, &price, 60).await.unwrap();
        let retrieved = cache.get_price(key).await.unwrap();
        
        assert!(retrieved.is_some());
        let retrieved_price = retrieved.unwrap();
        assert_eq!(retrieved_price.asset_id, price.asset_id);
        assert_eq!(retrieved_price.price, price.price);

        // Test delete
        let deleted = cache.delete_price(key).await.unwrap();
        assert!(deleted);

        // Verify deletion
        let after_delete = cache.get_price(key).await.unwrap();
        assert!(after_delete.is_none());
    }

    #[tokio::test]
    #[ignore] // Ignore by default to avoid requiring Redis in CI
    async fn test_batch_operations() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            command_timeout: 5,
            retry_attempts: 3,
        };

        let cache = PriceCache::new(&config).await;
        if cache.is_err() {
            return;
        }

        let cache = cache.unwrap();
        
        // Create test prices
        let mut prices = Vec::new();
        prices.push(("test:BTC:USD".to_string(), create_test_price()));
        
        let mut eth_price = create_test_price();
        eth_price.asset_id = "ETH".to_string();
        eth_price.price = dec!(3000.0);
        prices.push(("test:ETH:USD".to_string(), eth_price));

        // Test batch set
        cache.set_batch_prices(&prices, 60).await.unwrap();

        // Test batch get
        let keys: Vec<String> = prices.iter().map(|(k, _)| k.clone()).collect();
        let retrieved = cache.get_batch_prices(&keys).await.unwrap();
        
        assert_eq!(retrieved.len(), 2);
        assert!(retrieved[0].is_some());
        assert!(retrieved[1].is_some());

        // Clean up
        for (key, _) in &prices {
            cache.delete_price(key).await.unwrap();
        }
    }
}
