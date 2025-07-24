// =====================================================================================
// RWA Tokenization Platform - Oracle Service Unit Tests Module
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod test_aggregator;
pub mod test_cache;
pub mod test_config;
pub mod test_error;
pub mod test_handlers;
pub mod test_health;
pub mod test_metrics;
pub mod test_models;
pub mod test_providers;
pub mod test_service;

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use service_oracle::models::*;
use std::collections::HashMap;
use uuid::Uuid;

/// Create a test asset price for testing purposes
pub fn create_test_asset_price(asset_id: &str, price: Decimal, currency: &str) -> AssetPrice {
    AssetPrice {
        asset_id: asset_id.to_string(),
        price,
        currency: currency.to_string(),
        timestamp: Utc::now(),
        confidence: dec!(0.95),
        source: "test".to_string(),
        metadata: Some(HashMap::new()),
    }
}

/// Create a test price feed for testing purposes
pub fn create_test_price_feed(asset_id: &str, currency: &str) -> PriceFeed {
    PriceFeed {
        id: Uuid::new_v4(),
        asset_id: asset_id.to_string(),
        name: format!("{} {} Feed", asset_id, currency),
        description: Some(format!("Test feed for {} in {}", asset_id, currency)),
        currency: currency.to_string(),
        update_interval: 60,
        providers: vec!["test_provider".to_string()],
        aggregation_method: AggregationMethod::Mean,
        deviation_threshold: dec!(10.0),
        is_active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test subscription for testing purposes
pub fn create_test_subscription(feed_id: Uuid, subscriber_id: &str) -> Subscription {
    Subscription {
        id: Uuid::new_v4(),
        feed_id,
        subscriber_id: subscriber_id.to_string(),
        subscriber_type: SubscriberType::External,
        webhook_url: Some("https://example.com/webhook".to_string()),
        notification_method: NotificationMethod::Webhook,
        filters: None,
        retry_config: None,
        is_active: true,
        last_notification: None,
        notification_count: 0,
        failure_count: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test provider health status
pub fn create_test_provider_health(provider_id: &str, is_healthy: bool) -> ProviderHealth {
    ProviderHealth {
        id: Uuid::new_v4(),
        provider_id: provider_id.to_string(),
        is_healthy,
        response_time_ms: Some(100),
        error_message: if is_healthy { None } else { Some("Test error".to_string()) },
        checked_at: Utc::now(),
    }
}

/// Create test aggregation result
pub fn create_test_aggregation_result(asset_id: &str, price: Decimal) -> AggregationResult {
    AggregationResult {
        asset_id: asset_id.to_string(),
        currency: "USD".to_string(),
        aggregated_price: price,
        confidence: dec!(0.95),
        source_count: 3,
        aggregation_method: AggregationMethod::Mean,
        deviation_percent: dec!(2.5),
        outliers_removed: 0,
        processing_time_ms: 50,
        metadata: HashMap::new(),
        timestamp: Utc::now(),
    }
}

/// Create test batch price request
pub fn create_test_batch_request(asset_ids: Vec<&str>) -> BatchPriceRequest {
    BatchPriceRequest {
        asset_ids: asset_ids.into_iter().map(|s| s.to_string()).collect(),
        currency: Some("USD".to_string()),
        include_metadata: Some(true),
    }
}

/// Create test price feed request
pub fn create_test_feed_request(asset_id: &str) -> CreatePriceFeedRequest {
    CreatePriceFeedRequest {
        asset_id: asset_id.to_string(),
        name: format!("{} Test Feed", asset_id),
        description: Some(format!("Test feed for {}", asset_id)),
        currency: "USD".to_string(),
        update_interval: 60,
        providers: vec!["test_provider".to_string()],
        aggregation_method: AggregationMethod::Mean,
        deviation_threshold: dec!(10.0),
    }
}

/// Create test subscription request
pub fn create_test_subscription_request(subscriber_id: &str) -> SubscriptionRequest {
    SubscriptionRequest {
        subscriber_id: subscriber_id.to_string(),
        webhook_url: Some("https://example.com/webhook".to_string()),
        filters: None,
    }
}

/// Mock provider for testing
pub struct MockProvider {
    pub name: String,
    pub weight: Decimal,
    pub should_fail: bool,
    pub response_delay_ms: u64,
}

impl MockProvider {
    pub fn new(name: &str, weight: Decimal) -> Self {
        Self {
            name: name.to_string(),
            weight,
            should_fail: false,
            response_delay_ms: 0,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.response_delay_ms = delay_ms;
        self
    }
}

/// Test utilities for async operations
pub mod async_utils {
    use std::time::Duration;
    use tokio::time::timeout;

    /// Run an async test with timeout
    pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, &'static str>
    where
        F: std::future::Future<Output = T>,
    {
        timeout(duration, future)
            .await
            .map_err(|_| "Test timed out")
    }

    /// Create a test runtime for synchronous tests
    pub fn create_test_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create test runtime")
    }
}

/// Database test utilities
pub mod db_utils {
    use sqlx::{PgPool, Row};
    use uuid::Uuid;

    /// Clean up test data from database
    pub async fn cleanup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
        let tables = vec![
            "notification_history",
            "notification_queue",
            "subscriptions",
            "feed_updates",
            "feed_schedules",
            "price_feeds",
            "asset_prices",
            "provider_health",
            "aggregation_logs",
            "system_metrics",
            "alert_history",
            "alerts",
            "cache_stats",
        ];

        for table in tables {
            let query = format!("TRUNCATE TABLE {} CASCADE", table);
            sqlx::query(&query).execute(pool).await?;
        }

        Ok(())
    }

    /// Insert test asset price
    pub async fn insert_test_price(
        pool: &PgPool,
        asset_id: &str,
        price: rust_decimal::Decimal,
        currency: &str,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        let query = r#"
            INSERT INTO asset_prices (id, asset_id, price, currency, confidence, source, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
        "#;

        sqlx::query(query)
            .bind(&id)
            .bind(asset_id)
            .bind(price)
            .bind(currency)
            .bind(rust_decimal_macros::dec!(0.95))
            .bind("test")
            .execute(pool)
            .await?;

        Ok(id)
    }

    /// Count rows in table
    pub async fn count_rows(pool: &PgPool, table: &str) -> Result<i64, sqlx::Error> {
        let query = format!("SELECT COUNT(*) FROM {}", table);
        let row = sqlx::query(&query).fetch_one(pool).await?;
        Ok(row.get::<i64, _>(0))
    }
}

/// Redis test utilities
pub mod redis_utils {
    use redis::{AsyncCommands, Client};

    /// Clean up test data from Redis
    pub async fn cleanup_test_cache(redis_url: &str) -> Result<(), redis::RedisError> {
        let client = Client::open(redis_url)?;
        let mut conn = client.get_async_connection().await?;
        
        // Delete all keys with test prefix
        let keys: Vec<String> = conn.keys("test:*").await?;
        if !keys.is_empty() {
            conn.del(&keys).await?;
        }

        Ok(())
    }

    /// Set test value in Redis
    pub async fn set_test_value(
        redis_url: &str,
        key: &str,
        value: &str,
        ttl_seconds: usize,
    ) -> Result<(), redis::RedisError> {
        let client = Client::open(redis_url)?;
        let mut conn = client.get_async_connection().await?;
        
        conn.set_ex(key, value, ttl_seconds).await?;
        Ok(())
    }

    /// Get test value from Redis
    pub async fn get_test_value(redis_url: &str, key: &str) -> Result<Option<String>, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let mut conn = client.get_async_connection().await?;
        
        let value: Option<String> = conn.get(key).await?;
        Ok(value)
    }
}

/// HTTP test utilities
pub mod http_utils {
    use actix_web::{test, web, App, HttpResponse};
    use serde_json::Value;

    /// Create test app with routes
    pub fn create_test_app() -> actix_web::test::TestServer {
        test::start(|| {
            App::new()
                .route("/test", web::get().to(|| async { HttpResponse::Ok().json("test") }))
        })
    }

    /// Parse JSON response
    pub async fn parse_json_response(resp: actix_web::dev::ServiceResponse) -> Value {
        let body = test::read_body(resp).await;
        serde_json::from_slice(&body).expect("Failed to parse JSON response")
    }

    /// Assert JSON field equals value
    pub fn assert_json_field(json: &Value, field: &str, expected: &Value) {
        let actual = json.get(field).expect(&format!("Field '{}' not found", field));
        assert_eq!(actual, expected, "Field '{}' mismatch", field);
    }
}

/// Performance test utilities
pub mod perf_utils {
    use std::time::{Duration, Instant};

    /// Measure execution time of a function
    pub async fn measure_async<F, T>(f: F) -> (T, Duration)
    where
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Assert execution time is within bounds
    pub fn assert_execution_time(duration: Duration, max_duration: Duration) {
        assert!(
            duration <= max_duration,
            "Execution took {:?}, expected <= {:?}",
            duration,
            max_duration
        );
    }

    /// Run performance test with multiple iterations
    pub async fn run_performance_test<F, T>(
        iterations: usize,
        test_fn: F,
    ) -> (Vec<Duration>, Duration, Duration)
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>,
    {
        let mut durations = Vec::with_capacity(iterations);
        
        for _ in 0..iterations {
            let (_, duration) = measure_async(test_fn()).await;
            durations.push(duration);
        }

        let total: Duration = durations.iter().sum();
        let avg = total / iterations as u32;
        let max = *durations.iter().max().unwrap();

        (durations, avg, max)
    }
}
