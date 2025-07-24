// =====================================================================================
// File: core-database/src/migrations.rs
// Description: Database migrations for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::DatabaseError;
use sqlx::{Pool, Postgres, Row};
use tracing::{info, warn};

/// Run all database migrations
pub async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), DatabaseError> {
    info!("Running database migrations");

    // Create migrations table if it doesn't exist
    create_migrations_table(pool).await?;

    // Get current migration version
    let current_version = get_current_migration_version(pool).await?;
    info!("Current migration version: {}", current_version);

    // Run migrations in order
    let migrations = get_migrations();

    for migration in migrations {
        if migration.version > current_version {
            info!(
                "Running migration {}: {}",
                migration.version, migration.name
            );

            // Start transaction
            let mut tx = pool.begin().await.map_err(DatabaseError::from)?;

            // Execute migration
            for statement in &migration.up_sql {
                sqlx::query(statement)
                    .execute(&mut *tx)
                    .await
                    .map_err(DatabaseError::from)?;
            }

            // Update migration version
            sqlx::query(
                "INSERT INTO migrations (version, name, applied_at) VALUES ($1, $2, NOW())",
            )
            .bind(migration.version)
            .bind(&migration.name)
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::from)?;

            // Commit transaction
            tx.commit().await.map_err(DatabaseError::from)?;

            info!("Migration {} completed successfully", migration.version);
        }
    }

    info!("All migrations completed");
    Ok(())
}

/// Create the migrations tracking table
async fn create_migrations_table(pool: &Pool<Postgres>) -> Result<(), DatabaseError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            applied_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(DatabaseError::from)?;

    Ok(())
}

/// Get the current migration version
async fn get_current_migration_version(pool: &Pool<Postgres>) -> Result<i32, DatabaseError> {
    let row = sqlx::query("SELECT COALESCE(MAX(version), 0) as version FROM migrations")
        .fetch_one(pool)
        .await
        .map_err(DatabaseError::from)?;

    Ok(row.get("version"))
}

/// Migration definition
#[derive(Debug)]
struct Migration {
    version: i32,
    name: String,
    up_sql: Vec<String>,
    down_sql: Vec<String>,
}

/// Get all migrations in order
fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            name: "create_users_table".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE users (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    email VARCHAR(255) UNIQUE NOT NULL,
                    password_hash VARCHAR(255) NOT NULL,
                    first_name VARCHAR(100),
                    last_name VARCHAR(100),
                    is_active BOOLEAN NOT NULL DEFAULT true,
                    is_verified BOOLEAN NOT NULL DEFAULT false,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#
                .to_string(),
                r#"
                CREATE INDEX idx_users_email ON users(email);
                CREATE INDEX idx_users_active ON users(is_active);
                CREATE INDEX idx_users_created_at ON users(created_at);
                "#
                .to_string(),
            ],
            down_sql: vec!["DROP TABLE users".to_string()],
        },
        Migration {
            version: 2,
            name: "create_assets_table".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE assets (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    name VARCHAR(255) NOT NULL,
                    description TEXT,
                    asset_type VARCHAR(50) NOT NULL,
                    total_value DECIMAL(20, 2) NOT NULL,
                    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
                    owner_id UUID NOT NULL REFERENCES users(id),
                    is_tokenized BOOLEAN NOT NULL DEFAULT false,
                    token_address VARCHAR(255),
                    blockchain_network VARCHAR(50),
                    metadata JSONB,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#
                .to_string(),
                r#"
                CREATE INDEX idx_assets_owner_id ON assets(owner_id);
                CREATE INDEX idx_assets_type ON assets(asset_type);
                CREATE INDEX idx_assets_tokenized ON assets(is_tokenized);
                CREATE INDEX idx_assets_blockchain ON assets(blockchain_network);
                CREATE INDEX idx_assets_created_at ON assets(created_at);
                "#
                .to_string(),
            ],
            down_sql: vec!["DROP TABLE assets".to_string()],
        },
        Migration {
            version: 3,
            name: "create_transactions_table".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE transactions (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    transaction_hash VARCHAR(255) UNIQUE NOT NULL,
                    blockchain_network VARCHAR(50) NOT NULL,
                    from_address VARCHAR(255) NOT NULL,
                    to_address VARCHAR(255) NOT NULL,
                    amount DECIMAL(30, 18) NOT NULL,
                    fee DECIMAL(30, 18),
                    status VARCHAR(20) NOT NULL DEFAULT 'pending',
                    block_number BIGINT,
                    confirmations INTEGER DEFAULT 0,
                    asset_id UUID REFERENCES assets(id),
                    user_id UUID REFERENCES users(id),
                    metadata JSONB,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#
                .to_string(),
                r#"
                CREATE INDEX idx_transactions_hash ON transactions(transaction_hash);
                CREATE INDEX idx_transactions_network ON transactions(blockchain_network);
                CREATE INDEX idx_transactions_status ON transactions(status);
                CREATE INDEX idx_transactions_user_id ON transactions(user_id);
                CREATE INDEX idx_transactions_asset_id ON transactions(asset_id);
                CREATE INDEX idx_transactions_created_at ON transactions(created_at);
                "#
                .to_string(),
            ],
            down_sql: vec!["DROP TABLE transactions".to_string()],
        },
        Migration {
            version: 4,
            name: "create_user_sessions_table".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE user_sessions (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    user_id UUID NOT NULL REFERENCES users(id),
                    session_token VARCHAR(255) UNIQUE NOT NULL,
                    ip_address INET,
                    user_agent TEXT,
                    is_active BOOLEAN NOT NULL DEFAULT true,
                    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#
                .to_string(),
                r#"
                CREATE INDEX idx_sessions_user_id ON user_sessions(user_id);
                CREATE INDEX idx_sessions_token ON user_sessions(session_token);
                CREATE INDEX idx_sessions_active ON user_sessions(is_active);
                CREATE INDEX idx_sessions_expires_at ON user_sessions(expires_at);
                "#
                .to_string(),
            ],
            down_sql: vec!["DROP TABLE user_sessions".to_string()],
        },
        Migration {
            version: 5,
            name: "create_audit_logs_table".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE audit_logs (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    user_id UUID REFERENCES users(id),
                    action VARCHAR(100) NOT NULL,
                    resource_type VARCHAR(50) NOT NULL,
                    resource_id UUID,
                    old_values JSONB,
                    new_values JSONB,
                    ip_address INET,
                    user_agent TEXT,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#
                .to_string(),
                r#"
                CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
                CREATE INDEX idx_audit_logs_action ON audit_logs(action);
                CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
                CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
                "#
                .to_string(),
            ],
            down_sql: vec!["DROP TABLE audit_logs".to_string()],
        },
        Migration {
            version: 6,
            name: "create_blockchain_wallets_table".to_string(),
            up_sql: vec![
                r#"
                CREATE TABLE blockchain_wallets (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    user_id UUID NOT NULL REFERENCES users(id),
                    blockchain_network VARCHAR(50) NOT NULL,
                    address VARCHAR(255) NOT NULL,
                    is_primary BOOLEAN NOT NULL DEFAULT false,
                    balance DECIMAL(30, 18) DEFAULT 0,
                    last_balance_update TIMESTAMP WITH TIME ZONE,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    UNIQUE(user_id, blockchain_network, address)
                )
                "#
                .to_string(),
                r#"
                CREATE INDEX idx_wallets_user_id ON blockchain_wallets(user_id);
                CREATE INDEX idx_wallets_network ON blockchain_wallets(blockchain_network);
                CREATE INDEX idx_wallets_address ON blockchain_wallets(address);
                CREATE INDEX idx_wallets_primary ON blockchain_wallets(user_id, is_primary);
                "#
                .to_string(),
            ],
            down_sql: vec!["DROP TABLE blockchain_wallets".to_string()],
        },
    ]
}

/// Rollback migrations to a specific version
pub async fn rollback_to_version(
    pool: &Pool<Postgres>,
    target_version: i32,
) -> Result<(), DatabaseError> {
    info!("Rolling back migrations to version {}", target_version);

    let current_version = get_current_migration_version(pool).await?;

    if target_version >= current_version {
        warn!(
            "Target version {} is not less than current version {}",
            target_version, current_version
        );
        return Ok(());
    }

    let migrations = get_migrations();

    // Run rollbacks in reverse order
    for migration in migrations.iter().rev() {
        if migration.version > target_version && migration.version <= current_version {
            info!(
                "Rolling back migration {}: {}",
                migration.version, migration.name
            );

            // Start transaction
            let mut tx = pool.begin().await.map_err(DatabaseError::from)?;

            // Execute rollback
            for statement in &migration.down_sql {
                sqlx::query(statement)
                    .execute(&mut *tx)
                    .await
                    .map_err(DatabaseError::from)?;
            }

            // Remove migration record
            sqlx::query("DELETE FROM migrations WHERE version = $1")
                .bind(migration.version)
                .execute(&mut *tx)
                .await
                .map_err(DatabaseError::from)?;

            // Commit transaction
            tx.commit().await.map_err(DatabaseError::from)?;

            info!("Migration {} rolled back successfully", migration.version);
        }
    }

    info!("Rollback completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_order() {
        let migrations = get_migrations();

        // Check that migrations are in order
        for i in 1..migrations.len() {
            assert!(migrations[i].version > migrations[i - 1].version);
        }

        // Check that all migrations have required fields
        for migration in migrations {
            assert!(!migration.name.is_empty());
            assert!(!migration.up_sql.is_empty());
            assert!(!migration.down_sql.is_empty());
        }
    }
}
