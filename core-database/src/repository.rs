// =====================================================================================
// File: core-database/src/repository.rs
// Description: Repository pattern implementations for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{Cache, DatabaseError, Entity, Repository};
use crate::redis_client::RedisClient;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use std::marker::PhantomData;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Generic PostgreSQL repository with caching
pub struct PostgresRepository<T: Entity> {
    pool: Pool<Postgres>,
    cache: Option<RedisClient>,
    table_name: String,
    cache_prefix: String,
    cache_ttl: Option<u64>,
    _phantom: PhantomData<T>,
}

impl<T: Entity> PostgresRepository<T> {
    pub fn new(
        pool: Pool<Postgres>,
        table_name: String,
        cache_prefix: String,
    ) -> Self {
        Self {
            pool,
            cache: None,
            table_name,
            cache_prefix,
            cache_ttl: Some(3600), // 1 hour default
            _phantom: PhantomData,
        }
    }
    
    pub fn with_cache(
        mut self,
        cache: RedisClient,
        ttl: Option<u64>,
    ) -> Self {
        self.cache = Some(cache);
        self.cache_ttl = ttl;
        self
    }
    
    /// Generate cache key for an entity
    fn cache_key(&self, id: &T::Id) -> String
    where
        T::Id: std::fmt::Display,
    {
        format!("{}:{}", self.cache_prefix, id)
    }
    
    /// Get from cache if available
    async fn get_from_cache(&self, id: &T::Id) -> Result<Option<T>, DatabaseError>
    where
        T: for<'de> Deserialize<'de>,
        T::Id: std::fmt::Display,
    {
        if let Some(cache) = &self.cache {
            let key = self.cache_key(id);
            cache.get(&key).await
        } else {
            Ok(None)
        }
    }
    
    /// Set in cache if available
    async fn set_in_cache(&self, entity: &T) -> Result<(), DatabaseError>
    where
        T: Serialize,
        T::Id: std::fmt::Display,
    {
        if let Some(cache) = &self.cache {
            let key = self.cache_key(entity.id());
            cache.set(&key, entity, self.cache_ttl).await?;
        }
        Ok(())
    }
    
    /// Remove from cache if available
    async fn remove_from_cache(&self, id: &T::Id) -> Result<(), DatabaseError>
    where
        T::Id: std::fmt::Display,
    {
        if let Some(cache) = &self.cache {
            let key = self.cache_key(id);
            cache.delete(&key).await?;
        }
        Ok(())
    }
}

/// User entity for demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity for User {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

/// User repository implementation
#[async_trait]
impl Repository<User> for PostgresRepository<User> {
    type Error = DatabaseError;
    
    async fn create(&self, user: &User) -> Result<User, Self::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, is_active, is_verified)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, email, password_hash, first_name, last_name, is_active, is_verified, created_at, updated_at
            "#
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(user.is_active)
        .bind(user.is_verified)
        .fetch_one(&self.pool)
        .await?;
        
        let created_user = User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        // Cache the created user
        self.set_in_cache(&created_user).await?;
        
        info!("User created: {}", created_user.id);
        Ok(created_user)
    }
    
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, Self::Error> {
        // Try cache first
        if let Some(user) = self.get_from_cache(id).await? {
            return Ok(Some(user));
        }
        
        // Query database
        let row = sqlx::query(
            "SELECT id, email, password_hash, first_name, last_name, is_active, is_verified, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                is_active: row.get("is_active"),
                is_verified: row.get("is_verified"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            
            // Cache the result
            self.set_in_cache(&user).await?;
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, user: &User) -> Result<User, Self::Error> {
        let row = sqlx::query(
            r#"
            UPDATE users 
            SET email = $2, password_hash = $3, first_name = $4, last_name = $5, 
                is_active = $6, is_verified = $7, updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, first_name, last_name, is_active, is_verified, created_at, updated_at
            "#
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(user.is_active)
        .bind(user.is_verified)
        .fetch_one(&self.pool)
        .await?;
        
        let updated_user = User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };
        
        // Update cache
        self.set_in_cache(&updated_user).await?;
        
        info!("User updated: {}", updated_user.id);
        Ok(updated_user)
    }
    
    async fn delete(&self, id: &Uuid) -> Result<bool, Self::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        let deleted = result.rows_affected() > 0;
        
        if deleted {
            // Remove from cache
            self.remove_from_cache(id).await?;
            info!("User deleted: {}", id);
        }
        
        Ok(deleted)
    }
    
    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<User>, Self::Error> {
        let (limit, offset) = crate::utils::build_pagination(limit, offset);
        
        let rows = sqlx::query(
            r#"
            SELECT id, email, password_hash, first_name, last_name, is_active, is_verified, created_at, updated_at 
            FROM users 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;
        
        let users: Vec<User> = rows.into_iter().map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();
        
        Ok(users)
    }
}

/// Additional user repository methods
impl PostgresRepository<User> {
    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DatabaseError> {
        let row = sqlx::query(
            "SELECT id, email, password_hash, first_name, last_name, is_active, is_verified, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                is_active: row.get("is_active"),
                is_verified: row.get("is_verified"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            
            // Cache the result
            self.set_in_cache(&user).await?;
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    /// Count active users
    pub async fn count_active_users(&self) -> Result<i64, DatabaseError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE is_active = true")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(row.get("count"))
    }
    
    /// Find users created after a specific date
    pub async fn find_users_created_after(&self, date: DateTime<Utc>) -> Result<Vec<User>, DatabaseError> {
        let rows = sqlx::query(
            r#"
            SELECT id, email, password_hash, first_name, last_name, is_active, is_verified, created_at, updated_at 
            FROM users 
            WHERE created_at > $1 
            ORDER BY created_at DESC
            "#
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await?;
        
        let users: Vec<User> = rows.into_iter().map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            is_active: row.get("is_active"),
            is_verified: row.get("is_verified"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();
        
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    
    #[test]
    fn test_user_entity() {
        let user = User {
            id: utils::generate_id(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            is_active: true,
            is_verified: false,
            created_at: utils::now(),
            updated_at: utils::now(),
        };
        
        assert_eq!(user.id(), &user.id);
        assert!(user.is_active);
        assert!(!user.is_verified);
    }
    
    #[test]
    fn test_cache_key_generation() {
        // Test cache key generation without requiring a real database connection
        let id = utils::generate_id();
        let key = utils::cache_key("user", &id.to_string());
        assert_eq!(key, format!("user:{}", id));
    }
}
