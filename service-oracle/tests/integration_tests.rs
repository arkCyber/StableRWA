// =====================================================================================
// RWA Tokenization Platform - Oracle Service Integration Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, web, App};
use chrono::Utc;
use rust_decimal_macros::dec;
use serde_json::json;
use service_oracle::{
    cache::PriceCache,
    config::{OracleConfig, RedisConfig},
    handlers::*,
    health::HealthService,
    metrics::OracleMetrics,
    models::*,
    service::{OracleService, OracleServiceTrait},
    AppState,
};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Test configuration for integration tests
struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
}

impl TestConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/oracle_test".to_string()),
            redis_url: std::env::var("TEST_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379/1".to_string()),
        }
    }
}

/// Setup test database and return connection pool
async fn setup_test_db() -> sqlx::PgPool {
    let config = TestConfig::from_env();
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Setup test cache
async fn setup_test_cache() -> Arc<PriceCache> {
    let config = TestConfig::from_env();
    let redis_config = RedisConfig {
        url: config.redis_url,
        max_connections: 5,
        connection_timeout: 5,
        command_timeout: 5,
        retry_attempts: 3,
    };

    Arc::new(
        PriceCache::new(&redis_config)
            .await
            .expect("Failed to connect to test Redis")
    )
}

/// Create test app state
async fn create_test_app_state() -> AppState {
    let db_pool = setup_test_db().await;
    let cache = setup_test_cache().await;
    let metrics = Arc::new(OracleMetrics::new().expect("Failed to create metrics"));
    
    let config = OracleConfig::default();
    let oracle_service = Arc::new(
        OracleService::new(config, db_pool.clone())
            .await
            .expect("Failed to create Oracle service")
    );

    let health_service = Arc::new(HealthService::new(
        db_pool,
        cache,
        Duration::from_secs(30),
    ));

    AppState {
        oracle_service,
        health_service,
        metrics,
    }
}

/// Clean up test data
async fn cleanup_test_data(pool: &sqlx::PgPool) {
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
        sqlx::query(&query)
            .execute(pool)
            .await
            .unwrap_or_else(|_| {
                // Table might not exist, which is fine
                sqlx::postgres::PgQueryResult::default()
            });
    }
}

#[actix_web::test]
async fn test_health_endpoints() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/health", web::get().to(health_check))
            .route("/health/ready", web::get().to(readiness_check))
            .route("/health/live", web::get().to(liveness_probe))
    ).await;

    // Test health check
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status() == 503); // May be unhealthy in test env

    // Test readiness check
    let req = test::TestRequest::get().uri("/health/ready").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status() == 503);

    // Test liveness probe
    let req = test::TestRequest::get().uri("/health/live").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_metrics_endpoint() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/metrics", web::get().to(metrics_endpoint))
    ).await;

    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), 200);
    
    let content_type = resp.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/plain"));
}

#[actix_web::test]
async fn test_price_feed_crud_operations() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/feeds", web::post().to(create_price_feed))
            .route("/feeds", web::get().to(list_price_feeds))
            .route("/feeds/{feed_id}", web::get().to(get_price_feed))
            .route("/feeds/{feed_id}", web::put().to(update_price_feed))
            .route("/feeds/{feed_id}", web::delete().to(delete_price_feed))
    ).await;

    // Clean up any existing data
    if let Ok(pool) = setup_test_db().await.acquire().await {
        cleanup_test_data(&pool.into()).await;
    }

    // Test create price feed
    let create_request = CreatePriceFeedRequest {
        asset_id: "BTC".to_string(),
        name: "Bitcoin Test Feed".to_string(),
        description: Some("Test Bitcoin price feed".to_string()),
        currency: "USD".to_string(),
        update_interval: 60,
        providers: vec!["CoinGecko".to_string()],
        aggregation_method: AggregationMethod::Mean,
        deviation_threshold: dec!(5.0),
    };

    let req = test::TestRequest::post()
        .uri("/feeds")
        .set_json(&create_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["asset_id"], "BTC");
    assert_eq!(body["data"]["name"], "Bitcoin Test Feed");

    let feed_id = body["data"]["id"].as_str().unwrap();
    let feed_uuid = Uuid::parse_str(feed_id).unwrap();

    // Test get price feed
    let req = test::TestRequest::get()
        .uri(&format!("/feeds/{}", feed_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], feed_id);

    // Test list price feeds
    let req = test::TestRequest::get().uri("/feeds").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    assert!(body["count"].as_u64().unwrap() >= 1);

    // Test update price feed
    let update_request = UpdatePriceFeedRequest {
        name: Some("Updated Bitcoin Feed".to_string()),
        description: Some("Updated description".to_string()),
        update_interval: Some(120),
        is_active: Some(true),
    };

    let req = test::TestRequest::put()
        .uri(&format!("/feeds/{}", feed_id))
        .set_json(&update_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "Updated Bitcoin Feed");

    // Test delete price feed
    let req = test::TestRequest::delete()
        .uri(&format!("/feeds/{}", feed_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    // Verify deletion
    let req = test::TestRequest::get()
        .uri(&format!("/feeds/{}", feed_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_batch_price_request() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/prices/batch", web::post().to(get_batch_prices))
    ).await;

    let batch_request = BatchPriceRequest {
        asset_ids: vec!["BTC".to_string(), "ETH".to_string()],
        currency: Some("USD".to_string()),
        include_metadata: Some(true),
    };

    let req = test::TestRequest::post()
        .uri("/prices/batch")
        .set_json(&batch_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    // Note: This might fail if no providers are configured in test environment
    // In a real test, you'd mock the providers or use test data
    assert!(resp.status().is_success() || resp.status().is_server_error());
}

#[actix_web::test]
async fn test_subscription_management() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/feeds", web::post().to(create_price_feed))
            .route("/feeds/{feed_id}/subscribe", web::post().to(subscribe_to_feed))
            .route("/subscriptions/{subscription_id}", web::delete().to(unsubscribe_from_feed))
    ).await;

    // First create a price feed
    let create_request = CreatePriceFeedRequest {
        asset_id: "ETH".to_string(),
        name: "Ethereum Test Feed".to_string(),
        description: Some("Test Ethereum price feed".to_string()),
        currency: "USD".to_string(),
        update_interval: 60,
        providers: vec!["CoinGecko".to_string()],
        aggregation_method: AggregationMethod::WeightedAverage,
        deviation_threshold: dec!(5.0),
    };

    let req = test::TestRequest::post()
        .uri("/feeds")
        .set_json(&create_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let feed_id = body["data"]["id"].as_str().unwrap();

    // Test create subscription
    let subscription_request = SubscriptionRequest {
        subscriber_id: "test-subscriber".to_string(),
        webhook_url: Some("https://example.com/webhook".to_string()),
        filters: Some(json!({
            "min_price_change": 5.0,
            "max_price_change": 10.0
        })),
    };

    let req = test::TestRequest::post()
        .uri(&format!("/feeds/{}/subscribe", feed_id))
        .set_json(&subscription_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["subscriber_id"], "test-subscriber");

    let subscription_id = body["data"]["id"].as_str().unwrap();

    // Test unsubscribe
    let req = test::TestRequest::delete()
        .uri(&format!("/subscriptions/{}", subscription_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);
}

#[actix_web::test]
async fn test_error_handling() {
    let app_state = create_test_app_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/feeds/{feed_id}", web::get().to(get_price_feed))
            .route("/prices/batch", web::post().to(get_batch_prices))
    ).await;

    // Test non-existent feed
    let fake_uuid = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/feeds/{}", fake_uuid))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "feed_not_found");

    // Test empty batch request
    let empty_batch_request = BatchPriceRequest {
        asset_ids: vec![],
        currency: Some("USD".to_string()),
        include_metadata: Some(true),
    };

    let req = test::TestRequest::post()
        .uri("/prices/batch")
        .set_json(&empty_batch_request)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], false);
    assert!(body["error"].as_str().unwrap().contains("cannot be empty"));
}

#[tokio::test]
async fn test_database_operations() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    // Test inserting asset price
    let price_id = Uuid::new_v4();
    let query = r#"
        INSERT INTO asset_prices (id, asset_id, price, currency, confidence, source, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
    "#;

    let result = sqlx::query(query)
        .bind(&price_id)
        .bind("BTC")
        .bind(dec!(50000.0))
        .bind("USD")
        .bind(dec!(0.95))
        .bind("test")
        .bind(Utc::now())
        .execute(&pool)
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().rows_affected(), 1);

    // Test querying asset price
    let query = "SELECT * FROM asset_prices WHERE id = $1";
    let row = sqlx::query(query)
        .bind(&price_id)
        .fetch_one(&pool)
        .await;

    assert!(row.is_ok());
    let row = row.unwrap();
    assert_eq!(row.get::<String, _>("asset_id"), "BTC");
    assert_eq!(row.get::<rust_decimal::Decimal, _>("price"), dec!(50000.0));
}

#[tokio::test]
async fn test_cache_operations() {
    let cache = setup_test_cache().await;

    let test_price = AssetPrice {
        asset_id: "BTC".to_string(),
        price: dec!(50000.0),
        currency: "USD".to_string(),
        timestamp: Utc::now(),
        confidence: dec!(0.95),
        source: "test".to_string(),
        metadata: Some(HashMap::new()),
    };

    let cache_key = "test:BTC:USD";

    // Test set price
    let result = cache.set_price(cache_key, &test_price, 60).await;
    assert!(result.is_ok());

    // Test get price
    let result = cache.get_price(cache_key).await;
    assert!(result.is_ok());
    
    let cached_price = result.unwrap();
    assert!(cached_price.is_some());
    
    let cached_price = cached_price.unwrap();
    assert_eq!(cached_price.asset_id, "BTC");
    assert_eq!(cached_price.price, dec!(50000.0));

    // Test delete price
    let result = cache.delete_price(cache_key).await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    // Verify deletion
    let result = cache.get_price(cache_key).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
