// =====================================================================================
// File: service-asset/src/models.rs
// Description: Data models for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{AssetError, AssetResult};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// Asset model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Asset {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub total_value: Decimal,
    pub currency: String,
    pub location: Option<String>,
    pub is_tokenized: bool,
    pub token_address: Option<String>,
    pub blockchain_network: Option<String>,
    pub token_supply: Option<i64>,
    pub token_symbol: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset metadata model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetMetadata {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub metadata_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset valuation model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetValuation {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub value: Decimal,
    pub currency: String,
    pub valuation_method: String,
    pub valuation_date: DateTime<Utc>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Tokenization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizationResult {
    pub asset_id: Uuid,
    pub token_address: String,
    pub blockchain_network: String,
    pub token_supply: u64,
    pub token_symbol: String,
    pub transaction_hash: String,
    pub block_number: Option<u64>,
}

/// Asset repository for database operations
pub struct AssetRepository {
    pool: PgPool,
}

impl AssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new asset
    pub async fn create_asset(
        &self,
        owner_id: &Uuid,
        name: &str,
        description: &str,
        asset_type: &str,
        total_value: Decimal,
        currency: &str,
        location: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> AssetResult<Asset> {
        let asset_id = Uuid::new_v4();
        let now = Utc::now();

        let asset = sqlx::query_as!(
            Asset,
            r#"
            INSERT INTO assets (
                id, owner_id, name, description, asset_type, total_value,
                currency, location, is_tokenized, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
            asset_id,
            owner_id,
            name,
            description,
            asset_type,
            total_value,
            currency,
            location,
            false,
            metadata,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(asset)
    }

    /// Find asset by ID
    pub async fn find_by_id(&self, asset_id: &Uuid) -> AssetResult<Option<Asset>> {
        let asset = sqlx::query_as!(
            Asset,
            "SELECT * FROM assets WHERE id = $1",
            asset_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(asset)
    }

    /// Find asset by ID and owner
    pub async fn find_by_id_and_owner(
        &self,
        asset_id: &Uuid,
        owner_id: &Uuid,
    ) -> AssetResult<Option<Asset>> {
        let asset = sqlx::query_as!(
            Asset,
            "SELECT * FROM assets WHERE id = $1 AND owner_id = $2",
            asset_id,
            owner_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(asset)
    }

    /// Find assets by owner
    pub async fn find_by_owner(
        &self,
        owner_id: &Uuid,
        page: i64,
        per_page: i64,
    ) -> AssetResult<(Vec<Asset>, i64)> {
        let offset = (page - 1) * per_page;

        let assets = sqlx::query_as!(
            Asset,
            r#"
            SELECT * FROM assets 
            WHERE owner_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            owner_id,
            per_page,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM assets WHERE owner_id = $1",
            owner_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok((assets, total))
    }

    /// List all assets with filters
    pub async fn list_assets(
        &self,
        asset_type: Option<&str>,
        is_tokenized: Option<bool>,
        page: i64,
        per_page: i64,
    ) -> AssetResult<(Vec<Asset>, i64)> {
        let offset = (page - 1) * per_page;

        let mut query = "SELECT * FROM assets WHERE 1=1".to_string();
        let mut count_query = "SELECT COUNT(*) FROM assets WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(asset_type) = asset_type {
            param_count += 1;
            query.push_str(&format!(" AND asset_type = ${}", param_count));
            count_query.push_str(&format!(" AND asset_type = ${}", param_count));
            params.push(asset_type);
        }

        if let Some(is_tokenized) = is_tokenized {
            param_count += 1;
            query.push_str(&format!(" AND is_tokenized = ${}", param_count));
            count_query.push_str(&format!(" AND is_tokenized = ${}", param_count));
            params.push(if is_tokenized { "true" } else { "false" });
        }

        query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));

        // This is a simplified implementation - in practice, you'd use a query builder
        // or dynamic query construction for better type safety
        let assets = sqlx::query_as::<_, Asset>(&query)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let total = sqlx::query_scalar::<_, i64>(&count_query)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        Ok((assets, total))
    }

    /// Update asset
    pub async fn update_asset(&self, asset: &Asset) -> AssetResult<Asset> {
        let updated_asset = sqlx::query_as!(
            Asset,
            r#"
            UPDATE assets SET
                name = $2,
                description = $3,
                asset_type = $4,
                total_value = $5,
                currency = $6,
                location = $7,
                is_tokenized = $8,
                token_address = $9,
                blockchain_network = $10,
                token_supply = $11,
                token_symbol = $12,
                metadata = $13,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            asset.id,
            asset.name,
            asset.description,
            asset.asset_type,
            asset.total_value,
            asset.currency,
            asset.location,
            asset.is_tokenized,
            asset.token_address,
            asset.blockchain_network,
            asset.token_supply,
            asset.token_symbol,
            asset.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_asset)
    }

    /// Update asset tokenization info
    pub async fn update_tokenization_info(
        &self,
        asset_id: &Uuid,
        token_address: &str,
        blockchain_network: &str,
        token_supply: u64,
        token_symbol: &str,
    ) -> AssetResult<()> {
        sqlx::query!(
            r#"
            UPDATE assets SET
                is_tokenized = true,
                token_address = $2,
                blockchain_network = $3,
                token_supply = $4,
                token_symbol = $5,
                updated_at = NOW()
            WHERE id = $1
            "#,
            asset_id,
            token_address,
            blockchain_network,
            token_supply as i64,
            token_symbol
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete asset
    pub async fn delete_asset(&self, asset_id: &Uuid) -> AssetResult<()> {
        sqlx::query!("DELETE FROM assets WHERE id = $1", asset_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Search assets by name or description
    pub async fn search_assets(
        &self,
        search_term: &str,
        owner_id: Option<&Uuid>,
        page: i64,
        per_page: i64,
    ) -> AssetResult<(Vec<Asset>, i64)> {
        let offset = (page - 1) * per_page;
        let search_pattern = format!("%{}%", search_term);

        let (assets, total) = if let Some(owner_id) = owner_id {
            let assets = sqlx::query_as!(
                Asset,
                r#"
                SELECT * FROM assets 
                WHERE owner_id = $1 AND (name ILIKE $2 OR description ILIKE $3)
                ORDER BY created_at DESC
                LIMIT $4 OFFSET $5
                "#,
                owner_id,
                search_pattern,
                search_pattern,
                per_page,
                offset
            )
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar!(
                r#"
                SELECT COUNT(*) FROM assets 
                WHERE owner_id = $1 AND (name ILIKE $2 OR description ILIKE $3)
                "#,
                owner_id,
                search_pattern,
                search_pattern
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

            (assets, total)
        } else {
            let assets = sqlx::query_as!(
                Asset,
                r#"
                SELECT * FROM assets 
                WHERE name ILIKE $1 OR description ILIKE $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                search_pattern,
                search_pattern,
                per_page,
                offset
            )
            .fetch_all(&self.pool)
            .await?;

            let total = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM assets WHERE name ILIKE $1 OR description ILIKE $2",
                search_pattern,
                search_pattern
            )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

            (assets, total)
        };

        Ok((assets, total))
    }

    // Asset metadata operations

    /// Create asset metadata
    pub async fn create_metadata(
        &self,
        asset_id: &Uuid,
        key: &str,
        value: &serde_json::Value,
        metadata_type: &str,
    ) -> AssetResult<AssetMetadata> {
        let metadata_id = Uuid::new_v4();
        let now = Utc::now();

        let metadata = sqlx::query_as!(
            AssetMetadata,
            r#"
            INSERT INTO asset_metadata (
                id, asset_id, key, value, metadata_type, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            metadata_id,
            asset_id,
            key,
            value,
            metadata_type,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(metadata)
    }

    /// Get asset metadata
    pub async fn get_metadata(&self, asset_id: &Uuid) -> AssetResult<Vec<AssetMetadata>> {
        let metadata = sqlx::query_as!(
            AssetMetadata,
            "SELECT * FROM asset_metadata WHERE asset_id = $1 ORDER BY created_at DESC",
            asset_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(metadata)
    }

    /// Update asset metadata
    pub async fn update_metadata(
        &self,
        metadata_id: &Uuid,
        value: &serde_json::Value,
    ) -> AssetResult<AssetMetadata> {
        let metadata = sqlx::query_as!(
            AssetMetadata,
            r#"
            UPDATE asset_metadata SET
                value = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
            metadata_id,
            value
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(metadata)
    }

    /// Delete asset metadata
    pub async fn delete_metadata(&self, metadata_id: &Uuid) -> AssetResult<()> {
        sqlx::query!("DELETE FROM asset_metadata WHERE id = $1", metadata_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Asset valuation operations

    /// Create asset valuation
    pub async fn create_valuation(
        &self,
        asset_id: &Uuid,
        value: Decimal,
        currency: &str,
        valuation_method: &str,
        valuation_date: DateTime<Utc>,
        notes: Option<&str>,
        metadata: Option<&serde_json::Value>,
    ) -> AssetResult<AssetValuation> {
        let valuation_id = Uuid::new_v4();
        let now = Utc::now();

        let valuation = sqlx::query_as!(
            AssetValuation,
            r#"
            INSERT INTO asset_valuations (
                id, asset_id, value, currency, valuation_method, valuation_date,
                notes, metadata, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            valuation_id,
            asset_id,
            value,
            currency,
            valuation_method,
            valuation_date,
            notes,
            metadata,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(valuation)
    }

    /// Get asset valuations
    pub async fn get_valuations(
        &self,
        asset_id: &Uuid,
        page: i64,
        per_page: i64,
    ) -> AssetResult<(Vec<AssetValuation>, i64)> {
        let offset = (page - 1) * per_page;

        let valuations = sqlx::query_as!(
            AssetValuation,
            r#"
            SELECT * FROM asset_valuations 
            WHERE asset_id = $1
            ORDER BY valuation_date DESC
            LIMIT $2 OFFSET $3
            "#,
            asset_id,
            per_page,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM asset_valuations WHERE asset_id = $1",
            asset_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok((valuations, total))
    }

    /// Get latest asset valuation
    pub async fn get_latest_valuation(&self, asset_id: &Uuid) -> AssetResult<Option<AssetValuation>> {
        let valuation = sqlx::query_as!(
            AssetValuation,
            r#"
            SELECT * FROM asset_valuations 
            WHERE asset_id = $1
            ORDER BY valuation_date DESC
            LIMIT 1
            "#,
            asset_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(valuation)
    }

    /// Get asset statistics
    pub async fn get_asset_statistics(&self, owner_id: Option<&Uuid>) -> AssetResult<serde_json::Value> {
        let stats = if let Some(owner_id) = owner_id {
            sqlx::query!(
                r#"
                SELECT 
                    COUNT(*) as total_assets,
                    COUNT(CASE WHEN is_tokenized = true THEN 1 END) as tokenized_assets,
                    COUNT(DISTINCT asset_type) as asset_types,
                    COALESCE(SUM(total_value), 0) as total_value
                FROM assets 
                WHERE owner_id = $1
                "#,
                owner_id
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                SELECT 
                    COUNT(*) as total_assets,
                    COUNT(CASE WHEN is_tokenized = true THEN 1 END) as tokenized_assets,
                    COUNT(DISTINCT asset_type) as asset_types,
                    COALESCE(SUM(total_value), 0) as total_value
                FROM assets
                "#
            )
            .fetch_one(&self.pool)
            .await?
        };

        Ok(serde_json::json!({
            "total_assets": stats.total_assets,
            "tokenized_assets": stats.tokenized_assets,
            "asset_types": stats.asset_types,
            "total_value": stats.total_value
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use sqlx::PgPool;

    async fn create_test_asset(repo: &AssetRepository, owner_id: &Uuid) -> Asset {
        repo.create_asset(
            owner_id,
            "Test Property",
            "A test real estate property",
            "real_estate",
            dec!(500000.00),
            "USD",
            Some("New York, NY"),
            None,
        )
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_and_find_asset(pool: PgPool) {
        let repo = AssetRepository::new(pool);
        let owner_id = Uuid::new_v4();
        
        let asset = create_test_asset(&repo, &owner_id).await;
        assert_eq!(asset.name, "Test Property");
        assert_eq!(asset.asset_type, "real_estate");
        assert_eq!(asset.total_value, dec!(500000.00));
        assert!(!asset.is_tokenized);

        let found_asset = repo.find_by_id(&asset.id).await.unwrap();
        assert!(found_asset.is_some());
        assert_eq!(found_asset.unwrap().id, asset.id);
    }

    #[sqlx::test]
    async fn test_update_asset(pool: PgPool) {
        let repo = AssetRepository::new(pool);
        let owner_id = Uuid::new_v4();
        let mut asset = create_test_asset(&repo, &owner_id).await;

        asset.name = "Updated Property".to_string();
        asset.total_value = dec!(600000.00);

        let updated_asset = repo.update_asset(&asset).await.unwrap();
        assert_eq!(updated_asset.name, "Updated Property");
        assert_eq!(updated_asset.total_value, dec!(600000.00));
    }

    #[sqlx::test]
    async fn test_tokenization_update(pool: PgPool) {
        let repo = AssetRepository::new(pool);
        let owner_id = Uuid::new_v4();
        let asset = create_test_asset(&repo, &owner_id).await;

        repo.update_tokenization_info(
            &asset.id,
            "0x1234567890abcdef",
            "ethereum",
            1000000,
            "PROP",
        ).await.unwrap();

        let updated_asset = repo.find_by_id(&asset.id).await.unwrap().unwrap();
        assert!(updated_asset.is_tokenized);
        assert_eq!(updated_asset.token_address, Some("0x1234567890abcdef".to_string()));
        assert_eq!(updated_asset.blockchain_network, Some("ethereum".to_string()));
        assert_eq!(updated_asset.token_supply, Some(1000000));
        assert_eq!(updated_asset.token_symbol, Some("PROP".to_string()));
    }

    #[sqlx::test]
    async fn test_asset_valuations(pool: PgPool) {
        let repo = AssetRepository::new(pool);
        let owner_id = Uuid::new_v4();
        let asset = create_test_asset(&repo, &owner_id).await;

        let valuation = repo.create_valuation(
            &asset.id,
            dec!(550000.00),
            "USD",
            "market_analysis",
            Utc::now(),
            Some("Updated market valuation"),
            None,
        ).await.unwrap();

        assert_eq!(valuation.asset_id, asset.id);
        assert_eq!(valuation.value, dec!(550000.00));
        assert_eq!(valuation.valuation_method, "market_analysis");

        let latest_valuation = repo.get_latest_valuation(&asset.id).await.unwrap();
        assert!(latest_valuation.is_some());
        assert_eq!(latest_valuation.unwrap().id, valuation.id);

        let (valuations, total) = repo.get_valuations(&asset.id, 1, 10).await.unwrap();
        assert_eq!(valuations.len(), 1);
        assert_eq!(total, 1);
    }

    #[sqlx::test]
    async fn test_asset_search(pool: PgPool) {
        let repo = AssetRepository::new(pool);
        let owner_id = Uuid::new_v4();
        let _asset = create_test_asset(&repo, &owner_id).await;

        let (assets, total) = repo.search_assets("Test", Some(&owner_id), 1, 10).await.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(assets[0].name, "Test Property");

        let (assets, total) = repo.search_assets("NonExistent", Some(&owner_id), 1, 10).await.unwrap();
        assert_eq!(assets.len(), 0);
        assert_eq!(total, 0);
    }
}
