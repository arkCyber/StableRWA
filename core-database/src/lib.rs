// =====================================================================================
// File: core-database/src/lib.rs
// Description: Database abstraction layer for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod migrations;
pub mod postgres;
pub mod redis_client;
pub mod repository;

pub use postgres::*;
pub use redis_client::*;
pub use repository::*;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::fmt::Debug;
use thiserror::Error;
use uuid::Uuid;

/// Database error types
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Query error: {0}")]
    Query(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    #[error("Transaction error: {0}")]
    Transaction(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    DatabaseError::ConstraintViolation(db_err.to_string())
                } else {
                    DatabaseError::Query(db_err.to_string())
                }
            }
            _ => DatabaseError::Query(err.to_string()),
        }
    }
}

impl From<redis::RedisError> for DatabaseError {
    fn from(err: redis::RedisError) -> Self {
        DatabaseError::Connection(err.to_string())
    }
}

/// Base entity trait for all database entities
pub trait Entity: Debug + Clone + Send + Sync {
    type Id: Debug + Clone + Send + Sync;

    fn id(&self) -> &Self::Id;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Generic repository trait for CRUD operations
#[async_trait]
pub trait Repository<T: Entity>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn create(&self, entity: &T) -> Result<T, Self::Error>;
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, Self::Error>;
    async fn update(&self, entity: &T) -> Result<T, Self::Error>;
    async fn delete(&self, id: &T::Id) -> Result<bool, Self::Error>;
    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<T>, Self::Error>;
}

/// Cache trait for caching operations
#[async_trait]
pub trait Cache: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get<T>(&self, key: &str) -> Result<Option<T>, Self::Error>
    where
        T: for<'de> Deserialize<'de> + Send;

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> Result<(), Self::Error>
    where
        T: Serialize + Send + Sync;

    async fn delete(&self, key: &str) -> Result<bool, Self::Error>;
    async fn exists(&self, key: &str) -> Result<bool, Self::Error>;
    async fn expire(&self, key: &str, ttl: u64) -> Result<bool, Self::Error>;
}

/// Database connection manager
pub struct DatabaseManager {
    pub postgres_pool: Pool<Postgres>,
    pub redis_client: redis_client::RedisClient,
}

impl DatabaseManager {
    /// Create a new database manager with connection pools
    pub async fn new(
        config: &core_config::DatabaseConfig,
        redis_config: &core_config::RedisConfig,
    ) -> Result<Self, DatabaseError> {
        let postgres_pool = postgres::create_pool(config).await?;
        let redis_client = redis_client::RedisClient::new(redis_config).await?;

        Ok(Self {
            postgres_pool,
            redis_client,
        })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        migrations::run_migrations(&self.postgres_pool).await
    }

    /// Health check for database connections
    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        // Check PostgreSQL connection
        sqlx::query("SELECT 1")
            .execute(&self.postgres_pool)
            .await
            .map_err(DatabaseError::from)?;

        // Check Redis connection
        self.redis_client.ping().await?;

        Ok(())
    }
}

/// Common database utilities
pub mod utils {
    use super::*;

    /// Generate a new UUID v4
    pub fn generate_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Get current UTC timestamp
    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }

    /// Build cache key with prefix
    pub fn cache_key(prefix: &str, id: &str) -> String {
        format!("{}:{}", prefix, id)
    }

    /// Build pagination query
    pub fn build_pagination(limit: Option<u32>, offset: Option<u32>) -> (u32, u32) {
        let limit = limit.unwrap_or(50).min(1000); // Max 1000 records per query
        let offset = offset.unwrap_or(0);
        (limit, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = utils::generate_id();
        let id2 = utils::generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_cache_key() {
        let key = utils::cache_key("user", "123");
        assert_eq!(key, "user:123");
    }

    #[test]
    fn test_pagination() {
        let (limit, offset) = utils::build_pagination(Some(10), Some(20));
        assert_eq!(limit, 10);
        assert_eq!(offset, 20);

        let (limit, offset) = utils::build_pagination(None, None);
        assert_eq!(limit, 50);
        assert_eq!(offset, 0);

        // Test max limit
        let (limit, _) = utils::build_pagination(Some(2000), None);
        assert_eq!(limit, 1000);
    }
}
