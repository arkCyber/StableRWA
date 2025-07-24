// =====================================================================================
// RWA Tokenization Platform - Asset Service Load Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, web, App};
use futures::future::join_all;
use rust_decimal_macros::dec;
use serde_json::json;
use service_asset::{
    cache::InMemoryCache,
    handlers::*,
    models::*,
    service::{AssetService, CreateAssetRequest},
    AppState,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use uuid::Uuid;

/// Helper function to create test app state
async fn create_test_app_state() -> AppState {
    let repository = InMemoryAssetRepository::new();
    let cache = InMemoryCache::new();
    let asset_service = AssetService::new(Box::new(repository), Box::new(cache));

    AppState {
        asset_service: Box::new(asset_service),
    }
}

/// Load test for concurrent asset creation
#[tokio::test]
async fn test_concurrent_asset_creation_load() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    let concurrent_requests = 100;
    let start_time = Instant::now();

    // Create concurrent requests
    let tasks: Vec<_> = (0..concurrent_requests).map(|i| {
        let app = &app;
        async move {
            let payload = json!({
                "name": format!("Load Test Asset {}", i),
                "description": "Asset created during load testing",
                "asset_type": "RealEstate",
                "total_value": "1000000.00",
                "location": "Load Test Location"
            });

            let req = test::TestRequest::post()
                .uri("/assets")
                .set_json(&payload)
                .to_request();

            let resp = test::call_service(app, req).await;
            assert!(resp.status().is_success());
            resp
        }
    }).collect();

    // Execute all requests concurrently
    let results = join_all(tasks).await;
    let duration = start_time.elapsed();

    // Verify all requests succeeded
    assert_eq!(results.len(), concurrent_requests);
    
    // Performance assertions
    assert!(duration < Duration::from_secs(10), "Load test took too long: {:?}", duration);
    
    let requests_per_second = concurrent_requests as f64 / duration.as_secs_f64();
    println!("Asset creation RPS: {:.2}", requests_per_second);
    
    // Should handle at least 10 RPS
    assert!(requests_per_second > 10.0, "RPS too low: {:.2}", requests_per_second);
}

/// Load test for asset retrieval
#[tokio::test]
async fn test_asset_retrieval_load() {
    let app_state = web::Data::new(create_test_app_state().await);
    
    // Pre-create assets for retrieval
    let mut asset_ids = Vec::new();
    for i in 0..50 {
        let request = CreateAssetRequest {
            name: format!("Retrieval Test Asset {}", i),
            description: "Asset for retrieval testing".to_string(),
            asset_type: AssetType::RealEstate,
            total_value: dec!(1000000.00),
            owner_id: Uuid::new_v4().to_string(),
            location: None,
        };
        let asset = app_state.asset_service.create_asset(request).await.unwrap();
        asset_ids.push(asset.id);
    }

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets/{id}", web::get().to(get_asset))
    ).await;

    let concurrent_requests = 200;
    let start_time = Instant::now();

    // Create concurrent retrieval requests
    let tasks: Vec<_> = (0..concurrent_requests).map(|i| {
        let app = &app;
        let asset_id = asset_ids[i % asset_ids.len()];
        async move {
            let req = test::TestRequest::get()
                .uri(&format!("/assets/{}", asset_id))
                .to_request();

            let resp = test::call_service(app, req).await;
            assert!(resp.status().is_success());
            resp
        }
    }).collect();

    // Execute all requests concurrently
    let results = join_all(tasks).await;
    let duration = start_time.elapsed();

    // Verify all requests succeeded
    assert_eq!(results.len(), concurrent_requests);
    
    let requests_per_second = concurrent_requests as f64 / duration.as_secs_f64();
    println!("Asset retrieval RPS: {:.2}", requests_per_second);
    
    // Should handle at least 50 RPS for reads
    assert!(requests_per_second > 50.0, "Read RPS too low: {:.2}", requests_per_second);
}

/// Load test for asset listing with pagination
#[tokio::test]
async fn test_asset_listing_load() {
    let app_state = web::Data::new(create_test_app_state().await);
    
    // Pre-create many assets
    for i in 0..500 {
        let request = CreateAssetRequest {
            name: format!("List Test Asset {}", i),
            description: "Asset for listing testing".to_string(),
            asset_type: if i % 2 == 0 { AssetType::RealEstate } else { AssetType::Commodity },
            total_value: dec!(1000000.00),
            owner_id: Uuid::new_v4().to_string(),
            location: None,
        };
        app_state.asset_service.create_asset(request).await.unwrap();
    }

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::get().to(list_assets))
    ).await;

    let concurrent_requests = 50;
    let start_time = Instant::now();

    // Create concurrent listing requests with different pagination
    let tasks: Vec<_> = (0..concurrent_requests).map(|i| {
        let app = &app;
        async move {
            let page = (i % 10) + 1;
            let per_page = 20;
            
            let req = test::TestRequest::get()
                .uri(&format!("/assets?page={}&per_page={}", page, per_page))
                .to_request();

            let resp = test::call_service(app, req).await;
            assert!(resp.status().is_success());
            
            let body: serde_json::Value = test::read_body_json(resp).await;
            assert!(body["assets"].is_array());
            resp
        }
    }).collect();

    // Execute all requests concurrently
    let results = join_all(tasks).await;
    let duration = start_time.elapsed();

    assert_eq!(results.len(), concurrent_requests);
    
    let requests_per_second = concurrent_requests as f64 / duration.as_secs_f64();
    println!("Asset listing RPS: {:.2}", requests_per_second);
    
    // Should handle at least 20 RPS for complex queries
    assert!(requests_per_second > 20.0, "Listing RPS too low: {:.2}", requests_per_second);
}

/// Stress test with mixed operations
#[tokio::test]
async fn test_mixed_operations_stress() {
    let app_state = web::Data::new(create_test_app_state().await);
    
    // Pre-create some assets
    let mut asset_ids = Vec::new();
    for i in 0..20 {
        let request = CreateAssetRequest {
            name: format!("Stress Test Asset {}", i),
            description: "Asset for stress testing".to_string(),
            asset_type: AssetType::RealEstate,
            total_value: dec!(1000000.00),
            owner_id: Uuid::new_v4().to_string(),
            location: None,
        };
        let asset = app_state.asset_service.create_asset(request).await.unwrap();
        asset_ids.push(asset.id);
    }

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/assets", web::get().to(list_assets))
            .route("/assets/{id}", web::get().to(get_asset))
            .route("/assets/{id}", web::put().to(update_asset))
    ).await;

    let total_requests = 100;
    let start_time = Instant::now();

    // Mix of different operations
    let tasks: Vec<_> = (0..total_requests).map(|i| {
        let app = &app;
        let asset_ids = &asset_ids;
        async move {
            match i % 4 {
                // 25% create operations
                0 => {
                    let payload = json!({
                        "name": format!("Stress Asset {}", i),
                        "description": "Stress test asset",
                        "asset_type": "RealEstate",
                        "total_value": "1000000.00",
                        "location": "Stress Location"
                    });

                    let req = test::TestRequest::post()
                        .uri("/assets")
                        .set_json(&payload)
                        .to_request();

                    test::call_service(app, req).await
                },
                // 50% read operations
                1 | 2 => {
                    let asset_id = asset_ids[i % asset_ids.len()];
                    let req = test::TestRequest::get()
                        .uri(&format!("/assets/{}", asset_id))
                        .to_request();

                    test::call_service(app, req).await
                },
                // 25% list operations
                _ => {
                    let req = test::TestRequest::get()
                        .uri("/assets?page=1&per_page=10")
                        .to_request();

                    test::call_service(app, req).await
                }
            }
        }
    }).collect();

    // Execute all requests concurrently
    let results = join_all(tasks).await;
    let duration = start_time.elapsed();

    // Verify all requests succeeded
    for (i, result) in results.iter().enumerate() {
        assert!(result.status().is_success(), "Request {} failed with status: {}", i, result.status());
    }
    
    let requests_per_second = total_requests as f64 / duration.as_secs_f64();
    println!("Mixed operations RPS: {:.2}", requests_per_second);
    
    // Should handle mixed load efficiently
    assert!(requests_per_second > 15.0, "Mixed RPS too low: {:.2}", requests_per_second);
}

/// Memory pressure test
#[tokio::test]
async fn test_memory_pressure() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    // Create assets with large payloads
    let large_requests = 50;
    let start_time = Instant::now();

    let tasks: Vec<_> = (0..large_requests).map(|i| {
        let app = &app;
        async move {
            let payload = json!({
                "name": format!("Large Asset {}", i),
                "description": "A".repeat(10000), // Large description
                "asset_type": "RealEstate",
                "total_value": "1000000.00",
                "location": "Location".repeat(100) // Large location
            });

            let req = test::TestRequest::post()
                .uri("/assets")
                .set_json(&payload)
                .to_request();

            let resp = test::call_service(app, req).await;
            assert!(resp.status().is_success());
            resp
        }
    }).collect();

    let results = join_all(tasks).await;
    let duration = start_time.elapsed();

    assert_eq!(results.len(), large_requests);
    println!("Memory pressure test completed in {:?}", duration);
    
    // Should complete within reasonable time even with large payloads
    assert!(duration < Duration::from_secs(30), "Memory pressure test took too long");
}

/// Sustained load test
#[tokio::test]
async fn test_sustained_load() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/health", web::get().to(health_check))
    ).await;

    let duration = Duration::from_secs(10); // 10 second sustained test
    let requests_per_second = 5;
    let interval = Duration::from_millis(1000 / requests_per_second);
    
    let start_time = Instant::now();
    let mut request_count = 0;
    let mut success_count = 0;

    while start_time.elapsed() < duration {
        let payload = json!({
            "name": format!("Sustained Asset {}", request_count),
            "description": "Asset created during sustained load test",
            "asset_type": "RealEstate",
            "total_value": "1000000.00",
            "location": "Sustained Location"
        });

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        if resp.status().is_success() {
            success_count += 1;
        }
        request_count += 1;

        // Check health periodically
        if request_count % 10 == 0 {
            let health_req = test::TestRequest::get().uri("/health").to_request();
            let health_resp = test::call_service(&app, health_req).await;
            assert!(health_resp.status().is_success(), "Health check failed during sustained load");
        }

        sleep(interval).await;
    }

    let success_rate = success_count as f64 / request_count as f64;
    println!("Sustained load: {} requests, {:.2}% success rate", request_count, success_rate * 100.0);
    
    // Should maintain high success rate under sustained load
    assert!(success_rate > 0.95, "Success rate too low: {:.2}%", success_rate * 100.0);
    assert!(request_count > 40, "Not enough requests processed: {}", request_count);
}
