// =====================================================================================
// File: core-database/src/postgres.rs
// Description: PostgreSQL connection and utilities
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::DatabaseError;
use core_config::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use tracing::info;

/// Create a PostgreSQL connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<Pool<Postgres>, DatabaseError> {
    info!("Creating PostgreSQL connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout))
        .idle_timeout(Duration::from_secs(config.idle_timeout))
        .max_lifetime(Duration::from_secs(config.max_lifetime))
        .connect(&config.url)
        .await
        .map_err(|e| {
            DatabaseError::Connection(format!("Failed to create PostgreSQL pool: {}", e))
        })?;

    // Test the connection
    let _conn = pool.acquire().await.map_err(|e| {
        DatabaseError::Connection(format!("Failed to acquire connection from pool: {}", e))
    })?;

    info!("PostgreSQL connection pool created successfully");
    Ok(pool)
}

/// PostgreSQL-specific utilities
pub mod pg_utils {
    // Test imports would go here when needed

    /// Build a dynamic WHERE clause for filtering
    pub struct WhereClause {
        conditions: Vec<String>,
        bind_count: usize,
    }

    impl WhereClause {
        pub fn new() -> Self {
            Self {
                conditions: Vec::new(),
                bind_count: 0,
            }
        }

        pub fn add_condition(&mut self, condition: &str) -> &mut Self {
            self.conditions.push(condition.to_string());
            self.bind_count += 1;
            self
        }

        pub fn build(&self) -> String {
            if self.conditions.is_empty() {
                String::new()
            } else {
                format!("WHERE {}", self.conditions.join(" AND "))
            }
        }

        pub fn is_empty(&self) -> bool {
            self.conditions.is_empty()
        }
    }

    /// Build a dynamic ORDER BY clause
    pub fn build_order_by(sort_by: Option<&str>, sort_order: Option<&str>) -> String {
        match (sort_by, sort_order) {
            (Some(field), Some(order)) => {
                let order = if order.to_lowercase() == "desc" {
                    "DESC"
                } else {
                    "ASC"
                };
                format!("ORDER BY {} {}", field, order)
            }
            (Some(field), None) => format!("ORDER BY {} ASC", field),
            _ => "ORDER BY created_at DESC".to_string(),
        }
    }

    /// Build pagination clause
    pub fn build_pagination(limit: u32, offset: u32) -> String {
        format!("LIMIT {} OFFSET {}", limit, offset)
    }

    /// Escape SQL identifier (table/column names)
    pub fn escape_identifier(identifier: &str) -> String {
        format!("\"{}\"", identifier.replace("\"", "\"\""))
    }
}

#[cfg(test)]
mod tests {
    use super::pg_utils::*;

    #[test]
    fn test_where_clause() {
        let mut where_clause = WhereClause::new();
        assert!(where_clause.is_empty());
        assert_eq!(where_clause.build(), "");

        where_clause.add_condition("name = $1");
        where_clause.add_condition("active = $2");

        assert!(!where_clause.is_empty());
        assert_eq!(where_clause.build(), "WHERE name = $1 AND active = $2");
    }

    #[test]
    fn test_order_by() {
        assert_eq!(
            build_order_by(Some("name"), Some("asc")),
            "ORDER BY name ASC"
        );
        assert_eq!(
            build_order_by(Some("name"), Some("desc")),
            "ORDER BY name DESC"
        );
        assert_eq!(build_order_by(Some("name"), None), "ORDER BY name ASC");
        assert_eq!(build_order_by(None, None), "ORDER BY created_at DESC");
    }

    #[test]
    fn test_pagination() {
        assert_eq!(build_pagination(10, 20), "LIMIT 10 OFFSET 20");
    }

    #[test]
    fn test_escape_identifier() {
        assert_eq!(escape_identifier("user"), "\"user\"");
        assert_eq!(escape_identifier("user\"name"), "\"user\"\"name\"");
    }
}
