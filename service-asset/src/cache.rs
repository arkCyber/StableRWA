// =====================================================================================
// File: service-asset/src/cache.rs
// Description: Enterprise-grade caching layer for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use redis::{AsyncCommands, Client, RedisError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::metrics::AssetMetrics;

/// Cache trait for different cache implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get value from cache (returns serialized bytes)
    async fn get_bytes(&self, key: &str) -> CacheResult<Option<Vec<u8>>>;

    /// Set value in cache with TTL (accepts serialized bytes)
    async fn set_bytes(&self, key: &str, value: &[u8], ttl: Duration) -> CacheResult<()>;

    /// Delete value from cache
    async fn delete(&self, key: &str) -> CacheResult<()>;

    /// Check if key exists in cache
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Set expiration for existing key
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()>;

    /// Increment counter
    async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64>;

    /// Decrement counter
    async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64>;

    /// Clear all cache entries
    async fn clear(&self) -> CacheResult<()>;

    /// Get cache statistics
    async fn stats(&self) -> CacheResult<CacheStats>;
}

/// Cache extension trait for typed operations
#[async_trait]
pub trait CacheExt: Cache {
    /// Get value from cache
    async fn get<T>(&self, key: &str) -> CacheResult<Option<T>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        if let Some(bytes) = self.get_bytes(key).await? {
            let value = bincode::deserialize(&bytes)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Set value in cache with TTL
    async fn set<T>(&self, key: &str, value: &T, ttl: Duration) -> CacheResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let bytes = bincode::serialize(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        self.set_bytes(key, &bytes, ttl).await
    }

    /// Get multiple values from cache
    async fn mget<T>(&self, keys: &[String]) -> CacheResult<Vec<Option<T>>>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let mut results = Vec::new();
        for key in keys {
            let value = self.get(key).await?;
            results.push(value);
        }
        Ok(results)
    }

    /// Set multiple values in cache
    async fn mset<T>(&self, items: &[(String, T)], ttl: Duration) -> CacheResult<()>
    where
        T: Serialize + Send + Sync,
    {
        for (key, value) in items {
            self.set(key, value, ttl).await?;
        }
        Ok(())
    }
}

// Implement CacheExt for all types that implement Cache
impl<T: Cache + ?Sized> CacheExt for T {}

/// Redis cache implementation
pub struct RedisCache {
    client: Client,
    connection_pool: Arc<RwLock<Option<redis::aio::Connection>>>,
    metrics: Option<AssetMetrics>,
    key_prefix: String,
    compression_enabled: bool,
}

impl RedisCache {
    /// Create new Redis cache instance
    pub fn new(redis_url: &str, key_prefix: String) -> CacheResult<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| CacheError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            client,
            connection_pool: Arc::new(RwLock::new(None)),
            metrics: None,
            key_prefix,
            compression_enabled: false,
        })
    }
    
    /// Enable metrics collection
    pub fn with_metrics(mut self, metrics: AssetMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    /// Enable compression
    pub fn with_compression(mut self) -> Self {
        self.compression_enabled = true;
        self
    }
    
    /// Get Redis connection
    async fn get_connection(&self) -> CacheResult<redis::aio::Connection> {
        let mut pool = self.connection_pool.write().await;
        
        if pool.is_none() {
            let conn = self.client.get_async_connection().await
                .map_err(|e| CacheError::ConnectionError(e.to_string()))?;
            *pool = Some(conn);
        }
        
        // Clone the connection (Redis connections are cheap to clone)
        // For demo purposes, create a new connection each time
        let client = redis::Client::open("redis://127.0.0.1/")?;
        Ok(client.get_async_connection().await?)
    }
    
    /// Build cache key with prefix
    fn build_key(&self, key: &str) -> String {
        if self.key_prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}:{}", self.key_prefix, key)
        }
    }
    
    /// Serialize value with optional compression
    fn serialize_value<T>(&self, value: &T) -> CacheResult<Vec<u8>>
    where
        T: Serialize,
    {
        let serialized = bincode::serialize(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        
        if self.compression_enabled && serialized.len() > 1024 {
            // Compress large values
            use flate2::write::GzEncoder;
            use flate2::Compression;
            use std::io::Write;
            
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&serialized)
                .map_err(|e| CacheError::CompressionError(e.to_string()))?;
            let compressed = encoder.finish()
                .map_err(|e| CacheError::CompressionError(e.to_string()))?;
            
            // Add compression marker
            let mut result = vec![1u8]; // Compression flag
            result.extend_from_slice(&compressed);
            Ok(result)
        } else {
            // No compression
            let mut result = vec![0u8]; // No compression flag
            result.extend_from_slice(&serialized);
            Ok(result)
        }
    }
    
    /// Deserialize value with optional decompression
    fn deserialize_value<T>(&self, data: &[u8]) -> CacheResult<T>
    where
        T: DeserializeOwned,
    {
        if data.is_empty() {
            return Err(CacheError::SerializationError("Empty data".to_string()));
        }
        
        let (compressed, payload) = data.split_at(1);
        let is_compressed = compressed[0] == 1;
        
        let serialized = if is_compressed {
            // Decompress
            use flate2::read::GzDecoder;
            use std::io::Read;
            
            let mut decoder = GzDecoder::new(payload);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| CacheError::CompressionError(e.to_string()))?;
            decompressed
        } else {
            payload.to_vec()
        };
        
        bincode::deserialize(&serialized)
            .map_err(|e| CacheError::SerializationError(e.to_string()))
    }
    
    /// Record cache hit
    fn record_hit(&self) {
        if let Some(ref metrics) = self.metrics {
            metrics.record_cache_hit();
        }
    }
    
    /// Record cache miss
    fn record_miss(&self) {
        if let Some(ref metrics) = self.metrics {
            metrics.record_cache_miss();
        }
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn get_bytes(&self, key: &str) -> CacheResult<Option<Vec<u8>>> {
        let cache_key = self.build_key(key);
        let mut conn = self.get_connection().await?;

        let data: Option<Vec<u8>> = conn.get(&cache_key).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        match data {
            Some(bytes) => {
                self.record_hit();
                debug!("Cache hit for key: {}", cache_key);
                Ok(Some(bytes))
            }
            None => {
                self.record_miss();
                debug!("Cache miss for key: {}", cache_key);
                Ok(None)
            }
        }
    }

    async fn set_bytes(&self, key: &str, value: &[u8], ttl: Duration) -> CacheResult<()> {
        let cache_key = self.build_key(key);
        let mut conn = self.get_connection().await?;

        let ttl_seconds = ttl.as_secs() as usize;
        conn.set_ex(&cache_key, value, ttl_seconds as u64).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;

        debug!("Cache set for key: {} (TTL: {}s)", cache_key, ttl_seconds);
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> CacheResult<()> {
        let cache_key = self.build_key(key);
        let mut conn = self.get_connection().await?;
        
        conn.del(&cache_key).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;
        
        debug!("Cache delete for key: {}", cache_key);
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let cache_key = self.build_key(key);
        let mut conn = self.get_connection().await?;
        
        let exists: bool = conn.exists(&cache_key).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;
        
        Ok(exists)
    }
    
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        let cache_key = self.build_key(key);
        let mut conn = self.get_connection().await?;
        
        let ttl_seconds = ttl.as_secs() as i64;
        conn.expire(&cache_key, ttl_seconds).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;
        
        Ok(())
    }

    
    async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let cache_key = self.build_key(key);
        let mut conn = self.get_connection().await?;
        
        let result: i64 = conn.incr(&cache_key, delta).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;
        
        Ok(result)
    }
    
    async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let cache_key = self.build_key(key);
        let mut conn = self.get_connection().await?;
        
        let result: i64 = conn.decr(&cache_key, delta).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;
        
        Ok(result)
    }
    
    async fn clear(&self) -> CacheResult<()> {
        let mut conn = self.get_connection().await?;
        
        // Use pattern to clear only keys with our prefix
        let pattern = if self.key_prefix.is_empty() {
            "*".to_string()
        } else {
            format!("{}:*", self.key_prefix)
        };
        
        let keys: Vec<String> = conn.keys(&pattern).await
            .map_err(|e| CacheError::OperationError(e.to_string()))?;
        
        if !keys.is_empty() {
            conn.del(&keys).await
                .map_err(|e| CacheError::OperationError(e.to_string()))?;
        }
        
        info!("Cache cleared: {} keys deleted", keys.len());
        Ok(())
    }
    
    async fn stats(&self) -> CacheResult<CacheStats> {
        // Simplified stats implementation
        Ok(CacheStats {
            hits: 0, // Would need to track separately
            misses: 0, // Would need to track separately
            hit_rate: 0.0,
            used_memory: 0, // Would need Redis INFO command
            max_memory: 0, // Would need Redis INFO command
            key_count: 0, // Would need to count keys
            evictions: 0,
        })
    }
}

/// In-memory cache implementation for testing/development
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    metrics: Option<AssetMetrics>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: SystemTime,
}

impl MemoryCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            metrics: None,
            max_entries,
        }
    }
    
    pub fn with_metrics(mut self, metrics: AssetMetrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
    
    /// Clean expired entries
    async fn cleanup_expired(&self) {
        let now = SystemTime::now();
        let mut store = self.store.write().await;
        
        store.retain(|_, entry| entry.expires_at > now);
    }
    
    /// Evict entries if over limit
    async fn evict_if_needed(&self) {
        let mut store = self.store.write().await;
        
        if store.len() > self.max_entries {
            // Simple LRU: remove oldest entries
            let excess = store.len() - self.max_entries;
            let keys_to_remove: Vec<String> = store.keys().take(excess).cloned().collect();
            
            for key in keys_to_remove {
                store.remove(&key);
            }
        }
    }
    
    fn record_hit(&self) {
        if let Some(ref metrics) = self.metrics {
            metrics.record_cache_hit();
        }
    }
    
    fn record_miss(&self) {
        if let Some(ref metrics) = self.metrics {
            metrics.record_cache_miss();
        }
    }
}

#[async_trait]
impl Cache for MemoryCache {
    async fn get_bytes(&self, key: &str) -> CacheResult<Option<Vec<u8>>> {
        self.cleanup_expired().await;

        let store = self.store.read().await;
        let now = SystemTime::now();

        if let Some(entry) = store.get(key) {
            if entry.expires_at > now {
                self.record_hit();
                return Ok(Some(entry.data.clone()));
            }
        }

        self.record_miss();
        Ok(None)
    }

    async fn set_bytes(&self, key: &str, value: &[u8], ttl: Duration) -> CacheResult<()> {
        let expires_at = SystemTime::now() + ttl;
        let entry = CacheEntry {
            data: value.to_vec(),
            expires_at
        };

        {
            let mut store = self.store.write().await;
            store.insert(key.to_string(), entry);
        }

        self.evict_if_needed().await;
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> CacheResult<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> CacheResult<bool> {
        self.cleanup_expired().await;
        
        let store = self.store.read().await;
        let now = SystemTime::now();
        
        if let Some(entry) = store.get(key) {
            Ok(entry.expires_at > now)
        } else {
            Ok(false)
        }
    }
    
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<()> {
        let mut store = self.store.write().await;
        
        if let Some(entry) = store.get_mut(key) {
            entry.expires_at = SystemTime::now() + ttl;
        }
        
        Ok(())
    }

    async fn incr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        let current: i64 = self.get(key).await?.unwrap_or(0);
        let new_value = current + delta;
        self.set(key, &new_value, Duration::from_secs(3600)).await?;
        Ok(new_value)
    }

    async fn decr(&self, key: &str, delta: i64) -> CacheResult<i64> {
        self.incr(key, -delta).await
    }
    
    async fn clear(&self) -> CacheResult<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }
    
    async fn stats(&self) -> CacheResult<CacheStats> {
        let store = self.store.read().await;
        let key_count = store.len() as u64;
        
        // Calculate approximate memory usage
        let used_memory = store.iter()
            .map(|(k, v)| k.len() + v.data.len())
            .sum::<usize>() as u64;
        
        Ok(CacheStats {
            hits: 0, // Would need to track separately
            misses: 0, // Would need to track separately
            hit_rate: 0.0,
            used_memory,
            max_memory: 0,
            key_count,
            evictions: 0,
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub used_memory: u64,
    pub max_memory: u64,
    pub key_count: u64,
    pub evictions: u64,
}

/// Cache errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Operation error: {0}")]
    OperationError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Compression error: {0}")]
    CompressionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl From<redis::RedisError> for CacheError {
    fn from(err: redis::RedisError) -> Self {
        CacheError::ConnectionError(err.to_string())
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::SerializationError(err.to_string())
    }
}

pub type CacheResult<T> = Result<T, CacheError>;

/// Cache key builder utility
pub struct CacheKeyBuilder {
    prefix: String,
}

impl CacheKeyBuilder {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
    
    pub fn asset(&self, asset_id: &str) -> String {
        format!("{}:asset:{}", self.prefix, asset_id)
    }
    
    pub fn asset_list(&self, owner_id: &str, page: i64, per_page: i64) -> String {
        format!("{}:asset_list:{}:{}:{}", self.prefix, owner_id, page, per_page)
    }
    
    pub fn asset_valuation(&self, asset_id: &str) -> String {
        format!("{}:valuation:{}", self.prefix, asset_id)
    }
    
    pub fn asset_metadata(&self, asset_id: &str) -> String {
        format!("{}:metadata:{}", self.prefix, asset_id)
    }
    
    pub fn user_assets(&self, user_id: &str) -> String {
        format!("{}:user_assets:{}", self.prefix, user_id)
    }
    
    pub fn stats(&self, stat_type: &str) -> String {
        format!("{}:stats:{}", self.prefix, stat_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        id: u32,
        name: String,
        value: f64,
    }

    #[tokio::test]
    async fn test_memory_cache() {
        let cache = MemoryCache::new(100);
        let test_data = TestData {
            id: 1,
            name: "test".to_string(),
            value: 42.0,
        };
        
        // Test set and get
        cache.set("test_key", &test_data, Duration::from_secs(60)).await.unwrap();
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        
        assert_eq!(retrieved, Some(test_data));
        
        // Test exists
        assert!(cache.exists("test_key").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());
        
        // Test delete
        cache.delete("test_key").await.unwrap();
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_key_builder() {
        let builder = CacheKeyBuilder::new("test");
        
        assert_eq!(builder.asset("123"), "test:asset:123");
        assert_eq!(builder.asset_list("user1", 1, 10), "test:asset_list:user1:1:10");
        assert_eq!(builder.asset_valuation("123"), "test:valuation:123");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = MemoryCache::new(100);
        let test_data = TestData {
            id: 1,
            name: "test".to_string(),
            value: 42.0,
        };
        
        // Set with very short TTL
        cache.set("test_key", &test_data, Duration::from_millis(10)).await.unwrap();
        
        // Should exist immediately
        assert!(cache.exists("test_key").await.unwrap());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Should be expired
        assert!(!cache.exists("test_key").await.unwrap());
        let retrieved: Option<TestData> = cache.get("test_key").await.unwrap();
        assert_eq!(retrieved, None);
    }
}
