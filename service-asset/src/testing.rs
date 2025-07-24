// =====================================================================================
// File: service-asset/src/testing.rs
// Description: Enterprise-grade testing utilities for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use uuid::Uuid;
#[cfg(test)]
use chrono::{DateTime, Utc};
#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use actix_web::{test, web, App, HttpResponse, Result};

#[cfg(test)]
use crate::models::{Asset, AssetType, AssetStatus, AssetMetadata, AssetValuation};
#[cfg(test)]
use crate::cache::{Cache, CacheResult, MemoryCache};
#[cfg(test)]
use crate::metrics::AssetMetrics;
#[cfg(test)]
use crate::health::{HealthService, HealthConfig};

/// Test utilities for asset service
#[cfg(test)]
pub struct TestUtils;

#[cfg(test)]
impl TestUtils {
    /// Create a test asset
    pub fn create_test_asset() -> Asset {
        Asset {
            id: Uuid::new_v4(),
            name: "Test Real Estate Property".to_string(),
            description: Some("A beautiful test property for unit testing".to_string()),
            asset_type: AssetType::RealEstate,
            status: AssetStatus::Active,
            owner_id: Uuid::new_v4(),
            total_value: rust_decimal::Decimal::new(1000000, 2), // $10,000.00
            tokenized_value: Some(rust_decimal::Decimal::new(500000, 2)), // $5,000.00
            token_symbol: Some("PROP001".to_string()),
            token_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            metadata: AssetMetadata {
                location: Some("123 Test Street, Test City, TC 12345".to_string()),
                size: Some("2500 sq ft".to_string()),
                year_built: Some(2020),
                condition: Some("Excellent".to_string()),
                legal_description: Some("Lot 1, Block 1, Test Subdivision".to_string()),
                zoning: Some("Residential".to_string()),
                tax_id: Some("TAX123456".to_string()),
                insurance_policy: Some("INS789012".to_string()),
                custom_fields: {
                    let mut fields = HashMap::new();
                    fields.insert("bedrooms".to_string(), Value::Number(serde_json::Number::from(4)));
                    fields.insert("bathrooms".to_string(), Value::Number(serde_json::Number::from(3)));
                    fields.insert("garage".to_string(), Value::Bool(true));
                    fields
                },
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Create a test asset with specific values
    pub fn create_test_asset_with_values(
        name: &str,
        asset_type: AssetType,
        total_value: rust_decimal::Decimal,
        owner_id: Uuid,
    ) -> Asset {
        let mut asset = Self::create_test_asset();
        asset.name = name.to_string();
        asset.asset_type = asset_type;
        asset.total_value = total_value;
        asset.owner_id = owner_id;
        asset
    }
    
    /// Create a test asset valuation
    pub fn create_test_valuation(asset_id: Uuid) -> AssetValuation {
        AssetValuation {
            id: Uuid::new_v4(),
            asset_id,
            valuation_date: Utc::now(),
            appraised_value: rust_decimal::Decimal::new(1050000, 2), // $10,500.00
            market_value: rust_decimal::Decimal::new(1000000, 2), // $10,000.00
            replacement_value: rust_decimal::Decimal::new(1200000, 2), // $12,000.00
            valuation_method: "Comparative Market Analysis".to_string(),
            appraiser_name: "John Doe, Certified Appraiser".to_string(),
            appraiser_license: "APP123456".to_string(),
            confidence_level: rust_decimal::Decimal::new(95, 0), // 95%
            notes: Some("Property in excellent condition with recent renovations".to_string()),
            supporting_documents: vec![
                "appraisal_report.pdf".to_string(),
                "comparable_sales.xlsx".to_string(),
                "property_photos.zip".to_string(),
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Create multiple test assets
    pub fn create_test_assets(count: usize) -> Vec<Asset> {
        (0..count)
            .map(|i| {
                let mut asset = Self::create_test_asset();
                asset.name = format!("Test Asset {}", i + 1);
                asset
            })
            .collect()
    }
    
    /// Create test cache
    pub fn create_test_cache() -> Arc<dyn Cache> {
        Arc::new(MemoryCache::new(1000))
    }
    
    /// Create test metrics
    pub fn create_test_metrics() -> AssetMetrics {
        AssetMetrics::new().expect("Failed to create test metrics")
    }
    
    /// Create test health service
    pub fn create_test_health_service() -> HealthService {
        let config = HealthConfig::default();
        HealthService::new(config)
    }
    
    /// Create test database pool (mock)
    pub async fn create_test_db_pool() -> sqlx::PgPool {
        // In a real test environment, this would create a test database
        // For now, we'll create a mock pool
        sqlx::PgPool::connect("postgresql://test:test@localhost/test_db")
            .await
            .expect("Failed to create test database pool")
    }
    
    /// Setup test environment
    pub async fn setup_test_env() -> TestEnvironment {
        TestEnvironment {
            cache: Self::create_test_cache(),
            metrics: Self::create_test_metrics(),
            health_service: Self::create_test_health_service(),
        }
    }
    
    /// Generate test JWT token
    pub fn generate_test_jwt(user_id: Uuid) -> String {
        // In a real implementation, this would generate a proper JWT
        format!("Bearer test_token_for_user_{}", user_id)
    }
    
    /// Create test HTTP request with authentication
    pub fn create_authenticated_request(method: &str, uri: &str, user_id: Uuid) -> test::TestRequest {
        let token = Self::generate_test_jwt(user_id);
        test::TestRequest::with_uri(uri)
            .method(actix_web::http::Method::from_bytes(method.as_bytes()).unwrap())
            .insert_header(("Authorization", token))
    }
    
    /// Assert asset equality (ignoring timestamps)
    pub fn assert_assets_equal_ignore_timestamps(actual: &Asset, expected: &Asset) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.description, expected.description);
        assert_eq!(actual.asset_type, expected.asset_type);
        assert_eq!(actual.status, expected.status);
        assert_eq!(actual.owner_id, expected.owner_id);
        assert_eq!(actual.total_value, expected.total_value);
        assert_eq!(actual.tokenized_value, expected.tokenized_value);
        assert_eq!(actual.token_symbol, expected.token_symbol);
        assert_eq!(actual.token_address, expected.token_address);
        // Note: We skip created_at and updated_at comparison
    }
    
    /// Create test asset request JSON
    pub fn create_asset_request_json() -> Value {
        serde_json::json!({
            "name": "Test Property",
            "description": "A test property for API testing",
            "asset_type": "RealEstate",
            "total_value": "1000000.00",
            "metadata": {
                "location": "123 Test St, Test City, TC 12345",
                "size": "2000 sq ft",
                "year_built": 2020,
                "condition": "Good",
                "custom_fields": {
                    "bedrooms": 3,
                    "bathrooms": 2
                }
            }
        })
    }
    
    /// Create test valuation request JSON
    pub fn create_valuation_request_json(asset_id: Uuid) -> Value {
        serde_json::json!({
            "asset_id": asset_id,
            "appraised_value": "1050000.00",
            "market_value": "1000000.00",
            "replacement_value": "1200000.00",
            "valuation_method": "Comparative Market Analysis",
            "appraiser_name": "Jane Smith",
            "appraiser_license": "APP789012",
            "confidence_level": "95",
            "notes": "Property in excellent condition"
        })
    }
    
    /// Validate asset response structure
    pub fn validate_asset_response(response: &Value) -> bool {
        response.get("id").is_some()
            && response.get("name").is_some()
            && response.get("asset_type").is_some()
            && response.get("status").is_some()
            && response.get("owner_id").is_some()
            && response.get("total_value").is_some()
            && response.get("created_at").is_some()
            && response.get("updated_at").is_some()
    }
    
    /// Validate valuation response structure
    pub fn validate_valuation_response(response: &Value) -> bool {
        response.get("id").is_some()
            && response.get("asset_id").is_some()
            && response.get("appraised_value").is_some()
            && response.get("market_value").is_some()
            && response.get("valuation_method").is_some()
            && response.get("appraiser_name").is_some()
            && response.get("created_at").is_some()
    }
    
    /// Create test error response
    pub fn create_error_response(code: &str, message: &str) -> Value {
        serde_json::json!({
            "error": {
                "code": code,
                "message": message,
                "timestamp": Utc::now().to_rfc3339()
            }
        })
    }
    
    /// Wait for async operations to complete
    pub async fn wait_for_async_operations() {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    /// Clean up test data
    pub async fn cleanup_test_data(cache: &Arc<dyn Cache>) {
        let _ = cache.clear().await;
    }
}

/// Test environment setup
#[cfg(test)]
pub struct TestEnvironment {
    pub cache: Arc<dyn Cache>,
    pub metrics: AssetMetrics,
    pub health_service: HealthService,
}

/// Mock HTTP client for testing external API calls
#[cfg(test)]
pub struct MockHttpClient {
    responses: HashMap<String, (u16, Value)>,
}

#[cfg(test)]
impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }
    
    pub fn add_response(&mut self, url: &str, status: u16, body: Value) {
        self.responses.insert(url.to_string(), (status, body));
    }
    
    pub async fn get(&self, url: &str) -> Result<(u16, Value), String> {
        if let Some((status, body)) = self.responses.get(url) {
            Ok((*status, body.clone()))
        } else {
            Err(format!("No mock response configured for URL: {}", url))
        }
    }
    
    pub async fn post(&self, url: &str, _body: Value) -> Result<(u16, Value), String> {
        self.get(url).await
    }
}

/// Test database utilities
#[cfg(test)]
pub struct TestDatabase;

#[cfg(test)]
impl TestDatabase {
    /// Setup test database schema
    pub async fn setup_schema(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        // Create test tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS test_assets (
                id UUID PRIMARY KEY,
                name VARCHAR NOT NULL,
                description TEXT,
                asset_type VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                owner_id UUID NOT NULL,
                total_value DECIMAL(20,2) NOT NULL,
                tokenized_value DECIMAL(20,2),
                token_symbol VARCHAR(10),
                token_address VARCHAR(42),
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS test_asset_valuations (
                id UUID PRIMARY KEY,
                asset_id UUID NOT NULL REFERENCES test_assets(id),
                valuation_date TIMESTAMPTZ NOT NULL,
                appraised_value DECIMAL(20,2) NOT NULL,
                market_value DECIMAL(20,2) NOT NULL,
                replacement_value DECIMAL(20,2),
                valuation_method VARCHAR NOT NULL,
                appraiser_name VARCHAR NOT NULL,
                appraiser_license VARCHAR,
                confidence_level DECIMAL(5,2),
                notes TEXT,
                supporting_documents TEXT[],
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    /// Clean up test data
    pub async fn cleanup(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM test_asset_valuations").execute(pool).await?;
        sqlx::query("DELETE FROM test_assets").execute(pool).await?;
        Ok(())
    }
    
    /// Insert test asset
    pub async fn insert_test_asset(pool: &sqlx::PgPool, asset: &Asset) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO test_assets (
                id, name, description, asset_type, status, owner_id,
                total_value, tokenized_value, token_symbol, token_address,
                metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(asset.id)
        .bind(&asset.name)
        .bind(&asset.description)
        .bind(&asset.asset_type.to_string())
        .bind(&asset.status.to_string())
        .bind(asset.owner_id)
        .bind(asset.total_value)
        .bind(asset.tokenized_value)
        .bind(&asset.token_symbol)
        .bind(&asset.token_address)
        .bind(serde_json::to_value(&asset.metadata).unwrap())
        .bind(asset.created_at)
        .bind(asset.updated_at)
        .execute(pool)
        .await?;
        
        Ok(())
    }
}

/// Performance testing utilities
#[cfg(test)]
pub struct PerformanceTest;

#[cfg(test)]
impl PerformanceTest {
    /// Measure function execution time
    pub async fn measure_async<F, Fut, T>(f: F) -> (T, std::time::Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        (result, duration)
    }
    
    /// Run load test
    pub async fn load_test<F, Fut, T>(
        f: F,
        concurrent_requests: usize,
        total_requests: usize,
    ) -> LoadTestResult<T>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = T> + Send,
        T: Send + 'static,
    {
        let start = std::time::Instant::now();
        let mut handles = Vec::new();
        let f = Arc::new(f);
        
        for _ in 0..concurrent_requests {
            let f_clone = f.clone();
            let requests_per_task = total_requests / concurrent_requests;
            
            let handle = tokio::spawn(async move {
                let mut results = Vec::new();
                for _ in 0..requests_per_task {
                    let result = f_clone().await;
                    results.push(result);
                }
                results
            });
            
            handles.push(handle);
        }
        
        let mut all_results = Vec::new();
        for handle in handles {
            let results = handle.await.unwrap();
            all_results.extend(results);
        }
        
        let total_duration = start.elapsed();
        
        LoadTestResult {
            total_requests: all_results.len(),
            total_duration,
            requests_per_second: all_results.len() as f64 / total_duration.as_secs_f64(),
            results: all_results,
        }
    }
}

#[cfg(test)]
pub struct LoadTestResult<T> {
    pub total_requests: usize,
    pub total_duration: std::time::Duration,
    pub requests_per_second: f64,
    pub results: Vec<T>,
}

// Integration test helpers
#[cfg(test)]
pub async fn create_test_app() -> actix_web::test::TestServer {
    use actix_web::{middleware, web, App, HttpServer};
    use crate::handlers;
    
    test::start(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/assets", web::get().to(handlers::list_assets))
                    .route("/assets", web::post().to(handlers::create_asset))
                    .route("/assets/{id}", web::get().to(handlers::get_asset))
                    .route("/assets/{id}", web::put().to(handlers::update_asset))
                    .route("/assets/{id}", web::delete().to(handlers::delete_asset))
                    .route("/assets/{id}/valuations", web::post().to(handlers::create_valuation))
                    .route("/assets/{id}/tokenize", web::post().to(handlers::tokenize_asset))
            )
            .route("/health", web::get().to(crate::health::health_check))
            .route("/ready", web::get().to(crate::health::readiness_probe))
            .route("/live", web::get().to(crate::health::liveness_probe))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_asset() {
        let asset = TestUtils::create_test_asset();
        assert!(!asset.name.is_empty());
        assert_eq!(asset.asset_type, AssetType::RealEstate);
        assert_eq!(asset.status, AssetStatus::Active);
        assert!(asset.total_value > rust_decimal::Decimal::ZERO);
    }

    #[test]
    fn test_create_multiple_assets() {
        let assets = TestUtils::create_test_assets(5);
        assert_eq!(assets.len(), 5);
        
        for (i, asset) in assets.iter().enumerate() {
            assert_eq!(asset.name, format!("Test Asset {}", i + 1));
        }
    }

    #[test]
    fn test_validate_asset_response() {
        let response = serde_json::json!({
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Test Asset",
            "asset_type": "RealEstate",
            "status": "Active",
            "owner_id": "550e8400-e29b-41d4-a716-446655440001",
            "total_value": "1000000.00",
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-01-01T00:00:00Z"
        });
        
        assert!(TestUtils::validate_asset_response(&response));
    }

    #[tokio::test]
    async fn test_setup_test_env() {
        let env = TestUtils::setup_test_env().await;
        
        // Test cache
        env.cache.set("test_key", &"test_value", std::time::Duration::from_secs(60)).await.unwrap();
        let value: Option<String> = env.cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
        
        // Test metrics
        env.metrics.record_asset_created();
        assert_eq!(env.metrics.assets_created_total.get(), 1);
    }
}
