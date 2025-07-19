// =====================================================================================
// File: tests/integration/mod.rs
// Description: Integration test module for RWA Platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Integration Tests
//! 
//! This module contains integration tests that test the entire system
//! end-to-end, including database interactions, API endpoints, and
//! service integrations.

pub mod api_tests;
pub mod database_tests;
pub mod event_tests;
pub mod blockchain_tests;
pub mod payment_tests;

use std::sync::Once;
use tokio::sync::Mutex;
use std::collections::HashMap;
use serde_json::Value;

static INIT: Once = Once::new();
static TEST_DB: Mutex<Option<TestDatabase>> = Mutex::const_new(None);

/// Test database configuration
pub struct TestDatabase {
    pub url: String,
    pub pool: sqlx::PgPool,
}

/// Test configuration
pub struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub encryption_key: String,
    pub api_base_url: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/rwa_test".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379/1".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "test_jwt_secret_key_32_characters_long".to_string()),
            encryption_key: std::env::var("ENCRYPTION_KEY")
                .unwrap_or_else(|_| "test_encryption_key_32_characters_long".to_string()),
            api_base_url: std::env::var("API_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        }
    }
}

/// Initialize test environment
pub async fn init_test_env() -> TestConfig {
    INIT.call_once(|| {
        // Initialize logging for tests
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
    });

    let config = TestConfig::default();
    
    // Initialize test database
    let mut test_db = TEST_DB.lock().await;
    if test_db.is_none() {
        let pool = sqlx::PgPool::connect(&config.database_url)
            .await
            .expect("Failed to connect to test database");
        
        *test_db = Some(TestDatabase {
            url: config.database_url.clone(),
            pool,
        });
    }

    config
}

/// Get test database pool
pub async fn get_test_db() -> sqlx::PgPool {
    let test_db = TEST_DB.lock().await;
    test_db.as_ref()
        .expect("Test database not initialized")
        .pool.clone()
}

/// Clean up test data
pub async fn cleanup_test_data() {
    let pool = get_test_db().await;
    
    // Clean up in reverse dependency order
    let _ = sqlx::query("TRUNCATE TABLE payment_methods CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE TABLE payments CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE TABLE asset_valuations CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE TABLE asset_metadata CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE TABLE assets CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE TABLE user_sessions CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE TABLE users CASCADE").execute(&pool).await;
    let _ = sqlx::query("TRUNCATE TABLE audit_log CASCADE").execute(&pool).await;
}

/// Test HTTP client
pub struct TestClient {
    client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
}

impl TestClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            auth_token: None,
        }
    }

    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    pub async fn get(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        let mut request = self.client.get(&format!("{}{}", self.base_url, path));
        
        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request.send().await
    }

    pub async fn post(&self, path: &str, body: Value) -> reqwest::Result<reqwest::Response> {
        let mut request = self.client
            .post(&format!("{}{}", self.base_url, path))
            .json(&body);
        
        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request.send().await
    }

    pub async fn put(&self, path: &str, body: Value) -> reqwest::Result<reqwest::Response> {
        let mut request = self.client
            .put(&format!("{}{}", self.base_url, path))
            .json(&body);
        
        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request.send().await
    }

    pub async fn delete(&self, path: &str) -> reqwest::Result<reqwest::Response> {
        let mut request = self.client.delete(&format!("{}{}", self.base_url, path));
        
        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request.send().await
    }
}

/// Test data factory
pub struct TestDataFactory;

impl TestDataFactory {
    pub fn create_test_user() -> HashMap<String, Value> {
        let mut user = HashMap::new();
        user.insert("email".to_string(), Value::String("test@example.com".to_string()));
        user.insert("password".to_string(), Value::String("password123".to_string()));
        user.insert("first_name".to_string(), Value::String("John".to_string()));
        user.insert("last_name".to_string(), Value::String("Doe".to_string()));
        user.insert("phone".to_string(), Value::String("+1234567890".to_string()));
        user
    }

    pub fn create_test_asset() -> HashMap<String, Value> {
        let mut asset = HashMap::new();
        asset.insert("name".to_string(), Value::String("Test Real Estate".to_string()));
        asset.insert("description".to_string(), Value::String("A test property for integration testing".to_string()));
        asset.insert("asset_type".to_string(), Value::String("real_estate".to_string()));
        asset.insert("total_value".to_string(), Value::Number(serde_json::Number::from(1000000)));
        asset.insert("currency".to_string(), Value::String("USD".to_string()));
        asset.insert("location".to_string(), Value::String("New York, NY".to_string()));
        asset
    }

    pub fn create_test_payment() -> HashMap<String, Value> {
        let mut payment = HashMap::new();
        payment.insert("amount".to_string(), Value::Number(serde_json::Number::from(100)));
        payment.insert("currency".to_string(), Value::String("USD".to_string()));
        payment.insert("payment_method_type".to_string(), Value::String("credit_card".to_string()));
        payment.insert("provider".to_string(), Value::String("stripe".to_string()));
        payment.insert("description".to_string(), Value::String("Test payment".to_string()));
        payment
    }

    pub fn create_tokenization_request() -> HashMap<String, Value> {
        let mut request = HashMap::new();
        request.insert("blockchain_network".to_string(), Value::String("ethereum_testnet".to_string()));
        request.insert("token_supply".to_string(), Value::Number(serde_json::Number::from(1000000)));
        request.insert("token_symbol".to_string(), Value::String("TEST".to_string()));
        request
    }
}

/// Test assertions helper
pub struct TestAssertions;

impl TestAssertions {
    pub fn assert_response_status(response: &reqwest::Response, expected_status: reqwest::StatusCode) {
        assert_eq!(
            response.status(),
            expected_status,
            "Expected status {}, got {}",
            expected_status,
            response.status()
        );
    }

    pub async fn assert_response_json(response: reqwest::Response, expected_fields: &[&str]) -> Value {
        let json: Value = response.json().await.expect("Failed to parse JSON response");
        
        for field in expected_fields {
            assert!(
                json.get(field).is_some(),
                "Expected field '{}' not found in response: {}",
                field,
                json
            );
        }
        
        json
    }

    pub fn assert_uuid_format(value: &str) {
        assert!(
            uuid::Uuid::parse_str(value).is_ok(),
            "Value '{}' is not a valid UUID",
            value
        );
    }

    pub fn assert_timestamp_format(value: &str) {
        assert!(
            chrono::DateTime::parse_from_rfc3339(value).is_ok(),
            "Value '{}' is not a valid RFC3339 timestamp",
            value
        );
    }
}

/// Test database transaction helper
pub struct TestTransaction {
    tx: sqlx::Transaction<'static, sqlx::Postgres>,
}

impl TestTransaction {
    pub async fn begin() -> Result<Self, sqlx::Error> {
        let pool = get_test_db().await;
        let tx = pool.begin().await?;
        Ok(Self { tx })
    }

    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.tx.commit().await
    }

    pub async fn rollback(self) -> Result<(), sqlx::Error> {
        self.tx.rollback().await
    }

    pub fn as_executor(&mut self) -> &mut sqlx::Transaction<'static, sqlx::Postgres> {
        &mut self.tx
    }
}

/// Macro for creating integration tests with setup and cleanup
#[macro_export]
macro_rules! integration_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let _config = crate::integration::init_test_env().await;
            
            // Setup
            crate::integration::cleanup_test_data().await;
            
            // Run test
            let result = std::panic::AssertUnwindSafe($test_body).catch_unwind().await;
            
            // Cleanup
            crate::integration::cleanup_test_data().await;
            
            // Re-panic if test failed
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
}

/// Macro for creating authenticated API tests
#[macro_export]
macro_rules! authenticated_api_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() {
            let config = crate::integration::init_test_env().await;
            
            // Setup
            crate::integration::cleanup_test_data().await;
            
            // Create test user and get auth token
            let client = crate::integration::TestClient::new(config.api_base_url.clone());
            let user_data = crate::integration::TestDataFactory::create_test_user();
            
            let register_response = client.post("/api/v1/auth/register", serde_json::to_value(user_data).unwrap()).await.unwrap();
            assert_eq!(register_response.status(), reqwest::StatusCode::CREATED);
            
            let login_data = serde_json::json!({
                "email": "test@example.com",
                "password": "password123"
            });
            
            let login_response = client.post("/api/v1/auth/login", login_data).await.unwrap();
            assert_eq!(login_response.status(), reqwest::StatusCode::OK);
            
            let login_json: serde_json::Value = login_response.json().await.unwrap();
            let token = login_json["access_token"].as_str().unwrap().to_string();
            
            let authenticated_client = client.with_auth_token(token);
            
            // Run test with authenticated client
            let result = std::panic::AssertUnwindSafe($test_body(authenticated_client)).catch_unwind().await;
            
            // Cleanup
            crate::integration::cleanup_test_data().await;
            
            // Re-panic if test failed
            if let Err(panic) = result {
                std::panic::resume_unwind(panic);
            }
        }
    };
}
