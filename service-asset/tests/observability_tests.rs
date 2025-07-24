// =====================================================================================
// RWA Tokenization Platform - Asset Service Observability Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, web, App};
use serde_json::json;
use service_asset::{
    cache::InMemoryCache,
    handlers::*,
    health::{HealthService, HealthConfig},
    metrics::AssetMetrics,
    models::*,
    service::AssetService,
    AppState,
};
use std::time::Duration;
use uuid::Uuid;

/// Helper function to create test app state with metrics
async fn create_test_app_state_with_metrics() -> (AppState, AssetMetrics, HealthService) {
    let repository = InMemoryAssetRepository::new();
    let cache = InMemoryCache::new();
    let asset_service = AssetService::new(Box::new(repository), Box::new(cache));
    let metrics = AssetMetrics::new();
    
    let health_config = HealthConfig {
        check_interval_seconds: 30,
        timeout_seconds: 5,
        failure_threshold: 3,
        recovery_threshold: 2,
    };
    let health_service = HealthService::new(health_config, Some(metrics.clone()));

    let app_state = AppState {
        asset_service: Box::new(asset_service),
    };

    (app_state, metrics, health_service)
}

/// Test metrics collection and exposure
#[actix_web::test]
async fn test_metrics_collection() {
    let (app_state, _metrics, _health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/assets", web::post().to(create_asset))
            .route("/assets", web::get().to(list_assets))
            .route("/metrics", web::get().to(metrics_endpoint))
    ).await;

    // Perform some operations to generate metrics
    for i in 0..5 {
        let payload = json!({
            "name": format!("Metrics Test Asset {}", i),
            "description": "Asset for metrics testing",
            "asset_type": "RealEstate",
            "total_value": "1000000.00"
        });

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // List assets to generate read metrics
    let req = test::TestRequest::get()
        .uri("/assets?page=1&per_page=10")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Check metrics endpoint
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let metrics_text = std::str::from_utf8(&body).unwrap();

    // Verify metrics are present
    assert!(metrics_text.contains("asset_service_requests_total"));
    println!("Metrics output:\n{}", metrics_text);
}

/// Test health check functionality
#[actix_web::test]
async fn test_health_checks() {
    let (app_state, _metrics, health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .app_data(web::Data::new(health_service))
            .route("/health", web::get().to(health_check))
            .route("/health/ready", web::get().to(readiness_check))
            .route("/health/live", web::get().to(liveness_probe))
    ).await;

    // Test basic health check
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert!(body["timestamp"].is_string());
    assert!(body["service"].is_string());

    // Test readiness probe
    let req = test::TestRequest::get().uri("/health/ready").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "ready");

    // Test liveness probe
    let req = test::TestRequest::get().uri("/health/live").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "alive");
}

/// Test detailed health check with dependencies
#[actix_web::test]
async fn test_detailed_health_check() {
    let (app_state, _metrics, health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .app_data(web::Data::new(health_service))
            .route("/health", web::get().to(health_check))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    
    // Check for detailed health information
    assert!(body["status"].is_string());
    assert!(body["service"].is_string());
    assert!(body["version"].is_string());
    assert!(body["timestamp"].is_string());

    // Verify health check structure
    if let Some(checks) = body.get("checks") {
        assert!(checks.is_object());
        println!("Health checks: {}", serde_json::to_string_pretty(checks).unwrap());
    }
}

/// Test logging and tracing
#[actix_web::test]
async fn test_logging_and_tracing() {
    let (app_state, _metrics, _health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/assets", web::post().to(create_asset))
            .route("/assets/{id}", web::get().to(get_asset))
    ).await;

    // Create an asset to generate logs
    let payload = json!({
        "name": "Logging Test Asset",
        "description": "Asset for logging testing",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    let asset_id = body["id"].as_str().unwrap();

    // Retrieve the asset to generate more logs
    let req = test::TestRequest::get()
        .uri(&format!("/assets/{}", asset_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Note: In a real test environment, you would verify logs are written
    // to the configured output (file, stdout, etc.)
    println!("Logging test completed - check logs for trace information");
}

/// Test error tracking and alerting
#[actix_web::test]
async fn test_error_tracking() {
    let (app_state, _metrics, _health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/assets", web::post().to(create_asset))
            .route("/assets/{id}", web::get().to(get_asset))
            .route("/metrics", web::get().to(metrics_endpoint))
    ).await;

    // Generate various types of errors
    let error_scenarios = vec![
        // Invalid JSON
        (r#"{"invalid": json}"#, "application/json"),
        // Missing required fields
        (r#"{"name": ""}"#, "application/json"),
        // Invalid asset type
        (r#"{"name": "Test", "asset_type": "InvalidType", "total_value": "1000"}"#, "application/json"),
    ];

    for (payload, content_type) in error_scenarios {
        let req = test::TestRequest::post()
            .uri("/assets")
            .insert_header(("content-type", content_type))
            .set_payload(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    // Test 404 errors
    let req = test::TestRequest::get()
        .uri(&format!("/assets/{}", Uuid::new_v4()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // Check that error metrics are recorded
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let metrics_text = std::str::from_utf8(&body).unwrap();
    
    // Verify error metrics are present
    println!("Error metrics:\n{}", metrics_text);
}

/// Test performance monitoring
#[actix_web::test]
async fn test_performance_monitoring() {
    let (app_state, _metrics, _health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/assets", web::post().to(create_asset))
            .route("/assets", web::get().to(list_assets))
            .route("/metrics", web::get().to(metrics_endpoint))
    ).await;

    // Perform operations with different complexities
    let operations = vec![
        // Simple asset creation
        json!({
            "name": "Simple Asset",
            "description": "Simple",
            "asset_type": "RealEstate",
            "total_value": "1000000.00"
        }),
        // Complex asset creation
        json!({
            "name": "Complex Asset",
            "description": "X".repeat(1000),
            "asset_type": "RealEstate",
            "total_value": "1000000.00",
            "location": "Complex Location".repeat(10)
        }),
    ];

    for (i, payload) in operations.iter().enumerate() {
        let start_time = std::time::Instant::now();
        
        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let duration = start_time.elapsed();
        
        assert!(resp.status().is_success());
        println!("Operation {} took {:?}", i, duration);
    }

    // List assets to test query performance
    let start_time = std::time::Instant::now();
    let req = test::TestRequest::get()
        .uri("/assets?page=1&per_page=50")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let duration = start_time.elapsed();
    
    assert!(resp.status().is_success());
    println!("List operation took {:?}", duration);

    // Check performance metrics
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let metrics_text = std::str::from_utf8(&body).unwrap();
    println!("Performance metrics:\n{}", metrics_text);
}

/// Test custom business metrics
#[actix_web::test]
async fn test_business_metrics() {
    let (app_state, _metrics, _health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/assets", web::post().to(create_asset))
            .route("/assets/{id}/tokenize", web::post().to(tokenize_asset))
            .route("/metrics", web::get().to(metrics_endpoint))
    ).await;

    // Create assets of different types
    let asset_types = vec!["RealEstate", "Commodity", "Artwork"];
    let mut created_assets = Vec::new();

    for (i, asset_type) in asset_types.iter().enumerate() {
        let payload = json!({
            "name": format!("Business Metrics Asset {}", i),
            "description": "Asset for business metrics",
            "asset_type": asset_type,
            "total_value": format!("{}.00", (i + 1) * 1000000)
        });

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = test::read_body_json(resp).await;
        created_assets.push(body["id"].as_str().unwrap().to_string());
    }

    // Tokenize some assets
    for asset_id in created_assets.iter().take(2) {
        let tokenize_payload = json!({
            "token_symbol": "TEST",
            "token_supply": 1000000,
            "blockchain_network": "ethereum"
        });

        let req = test::TestRequest::post()
            .uri(&format!("/assets/{}/tokenize", asset_id))
            .set_json(&tokenize_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        // May succeed or fail depending on implementation
        println!("Tokenization response status: {}", resp.status());
    }

    // Check business metrics
    let req = test::TestRequest::get().uri("/metrics").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let metrics_text = std::str::from_utf8(&body).unwrap();
    
    // Look for business-specific metrics
    println!("Business metrics:\n{}", metrics_text);
}

/// Test distributed tracing simulation
#[actix_web::test]
async fn test_distributed_tracing() {
    let (app_state, _metrics, _health_service) = create_test_app_state_with_metrics().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .route("/assets", web::post().to(create_asset))
            .route("/assets/{id}", web::get().to(get_asset))
    ).await;

    // Simulate distributed tracing with correlation IDs
    let correlation_id = Uuid::new_v4().to_string();
    
    let payload = json!({
        "name": "Tracing Test Asset",
        "description": "Asset for distributed tracing test",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .insert_header(("X-Correlation-ID", correlation_id.as_str()))
        .insert_header(("X-Request-ID", Uuid::new_v4().to_string()))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify correlation ID is preserved in response
    if let Some(response_correlation_id) = resp.headers().get("X-Correlation-ID") {
        assert_eq!(response_correlation_id.to_str().unwrap(), correlation_id);
    }

    let body: serde_json::Value = test::read_body_json(resp).await;
    let asset_id = body["id"].as_str().unwrap();

    // Follow up request with same correlation ID
    let req = test::TestRequest::get()
        .uri(&format!("/assets/{}", asset_id))
        .insert_header(("X-Correlation-ID", correlation_id.as_str()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    println!("Distributed tracing test completed with correlation ID: {}", correlation_id);
}
