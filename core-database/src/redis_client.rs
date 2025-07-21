// =====================================================================================
// File: core-database/src/redis_client.rs
// Description: Redis client and caching utilities
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{Cache, DatabaseError};
use async_trait::async_trait;
use core_config::RedisConfig;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

/// Redis client wrapper with connection pooling
#[derive(Clone)]
pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    /// Create a new Redis client
    pub async fn new(config: &RedisConfig) -> Result<Self, DatabaseError> {
        info!("Creating Redis client");
        
        let client = Client::open(config.url.as_str())
            .map_err(|e| DatabaseError::Connection(format!("Failed to create Redis client: {}", e)))?;
            
        info!("Redis client created successfully");
        Ok(Self { client })
    }
    
    /// Ping Redis server
    pub async fn ping(&self) -> Result<(), DatabaseError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DatabaseError::Connection(format!("Failed to get Redis connection: {}", e)))?;
        redis::cmd("PING").query_async(&mut conn).await
            .map_err(|e| DatabaseError::Connection(format!("Redis ping failed: {}", e)))?;
        Ok(())
    }
    

}

#[async_trait]
impl Cache for RedisClient {
    type Error = DatabaseError;
    
    async fn get<T>(&self, key: &str) -> Result<Option<T>, Self::Error>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DatabaseError::Connection(format!("Failed to get Redis connection: {}", e)))?;
        let value: Option<String> = conn.get(key).await
            .map_err(|e| DatabaseError::Connection(format!("Redis get failed: {}", e)))?;

        match value {
            Some(json_str) => {
                let deserialized = serde_json::from_str(&json_str)
                    .map_err(|e| DatabaseError::Serialization(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }
    
    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> Result<(), Self::Error>
    where
        T: Serialize + Send + Sync,
    {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DatabaseError::Connection(format!("Failed to get Redis connection: {}", e)))?;
        let json_str = serde_json::to_string(value)
            .map_err(|e| DatabaseError::Serialization(e.to_string()))?;

        match ttl {
            Some(seconds) => {
                let _: () = conn.set_ex(key, json_str, seconds).await
                    .map_err(|e| DatabaseError::Connection(format!("Redis set_ex failed: {}", e)))?;
            }
            None => {
                let _: () = conn.set(key, json_str).await
                    .map_err(|e| DatabaseError::Connection(format!("Redis set failed: {}", e)))?;
            }
        }

        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<bool, Self::Error> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DatabaseError::Connection(format!("Failed to get Redis connection: {}", e)))?;
        let deleted: i32 = conn.del(key).await
            .map_err(|e| DatabaseError::Connection(format!("Redis delete failed: {}", e)))?;
        Ok(deleted > 0)
    }

    async fn exists(&self, key: &str) -> Result<bool, Self::Error> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DatabaseError::Connection(format!("Failed to get Redis connection: {}", e)))?;
        let exists: bool = conn.exists(key).await
            .map_err(|e| DatabaseError::Connection(format!("Redis exists failed: {}", e)))?;
        Ok(exists)
    }

    async fn expire(&self, key: &str, ttl: u64) -> Result<bool, Self::Error> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DatabaseError::Connection(format!("Failed to get Redis connection: {}", e)))?;
        let result: bool = conn.expire(key, ttl as i64).await
            .map_err(|e| DatabaseError::Connection(format!("Redis expire failed: {}", e)))?;
        Ok(result)
    }
}

/// Redis utilities for common caching patterns
pub mod redis_utils {
    use super::*;
    
    /// Cache key patterns
    pub struct CacheKeys;
    
    impl CacheKeys {
        pub fn user(user_id: &str) -> String {
            format!("user:{}", user_id)
        }
        
        pub fn asset(asset_id: &str) -> String {
            format!("asset:{}", asset_id)
        }
        
        pub fn session(session_id: &str) -> String {
            format!("session:{}", session_id)
        }
        
        pub fn rate_limit(identifier: &str) -> String {
            format!("rate_limit:{}", identifier)
        }
        
        pub fn blockchain_cache(chain: &str, key: &str) -> String {
            format!("blockchain:{}:{}", chain, key)
        }
    }
    
    /// Common TTL values in seconds
    pub struct CacheTTL;
    
    impl CacheTTL {
        pub const MINUTE: u64 = 60;
        pub const HOUR: u64 = 3600;
        pub const DAY: u64 = 86400;
        pub const WEEK: u64 = 604800;
        
        // Application-specific TTLs
        pub const USER_SESSION: u64 = Self::HOUR * 24; // 24 hours
        pub const ASSET_DATA: u64 = Self::MINUTE * 15; // 15 minutes
        pub const BLOCKCHAIN_DATA: u64 = Self::MINUTE * 5; // 5 minutes
        pub const RATE_LIMIT: u64 = Self::MINUTE; // 1 minute
    }
}

#[cfg(test)]
mod tests {
    use super::redis_utils::*;
    
    #[test]
    fn test_cache_keys() {
        assert_eq!(CacheKeys::user("123"), "user:123");
        assert_eq!(CacheKeys::asset("abc"), "asset:abc");
        assert_eq!(CacheKeys::session("sess_123"), "session:sess_123");
        assert_eq!(CacheKeys::rate_limit("192.168.1.1"), "rate_limit:192.168.1.1");
        assert_eq!(CacheKeys::blockchain_cache("ethereum", "block_123"), "blockchain:ethereum:block_123");
    }
    
    #[test]
    fn test_cache_ttl() {
        assert_eq!(CacheTTL::MINUTE, 60);
        assert_eq!(CacheTTL::HOUR, 3600);
        assert_eq!(CacheTTL::DAY, 86400);
        assert_eq!(CacheTTL::USER_SESSION, 86400);
    }
}
