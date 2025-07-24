// =====================================================================================
// RWA Tokenization Platform - Asset Service Resilience Tests
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
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
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

/// Test service behavior under memory pressure
#[tokio::test]
async fn test_memory_pressure_resilience() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/health", web::get().to(health_check))
    ).await;

    // Create many large assets to simulate memory pressure
    let large_asset_count = 100;
    let mut success_count = 0;
    let mut error_count = 0;

    for i in 0..large_asset_count {
        let payload = json!({
            "name": format!("Large Asset {}", i),
            "description": "X".repeat(50000), // 50KB description
            "asset_type": "RealEstate",
            "total_value": "1000000.00",
            "location": "Location".repeat(1000) // Large location
        });

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        if resp.status().is_success() {
            success_count += 1;
        } else {
            error_count += 1;
        }

        // Check health every 10 requests
        if i % 10 == 0 {
            let health_req = test::TestRequest::get().uri("/health").to_request();
            let health_resp = test::call_service(&app, health_req).await;
            assert!(health_resp.status().is_success(), "Health check failed under memory pressure");
        }
    }

    println!("Memory pressure test: {} success, {} errors", success_count, error_count);
    
    // Service should handle at least 80% of requests successfully
    let success_rate = success_count as f64 / large_asset_count as f64;
    assert!(success_rate > 0.8, "Success rate too low under memory pressure: {:.2}%", success_rate * 100.0);
}

/// Test graceful degradation under high load
#[tokio::test]
async fn test_graceful_degradation() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/assets", web::get().to(list_assets))
            .route("/health", web::get().to(health_check))
    ).await;

    let concurrent_requests = 200;
    let start_time = Instant::now();
    
    // Generate high concurrent load
    let tasks: Vec<_> = (0..concurrent_requests).map(|i| {
        let app = &app;
        async move {
            let operation = i % 3;
            match operation {
                0 => {
                    // Create operation
                    let payload = json!({
                        "name": format!("Load Asset {}", i),
                        "description": "Asset under load",
                        "asset_type": "RealEstate",
                        "total_value": "1000000.00"
                    });

                    let req = test::TestRequest::post()
                        .uri("/assets")
                        .set_json(&payload)
                        .to_request();

                    test::call_service(app, req).await
                },
                1 => {
                    // List operation
                    let req = test::TestRequest::get()
                        .uri("/assets?page=1&per_page=10")
                        .to_request();

                    test::call_service(app, req).await
                },
                _ => {
                    // Health check
                    let req = test::TestRequest::get().uri("/health").to_request();
                    test::call_service(app, req).await
                }
            }
        }
    }).collect();

    let results = join_all(tasks).await;
    let duration = start_time.elapsed();

    // Analyze results
    let mut success_count = 0;
    let mut client_errors = 0;
    let mut server_errors = 0;

    for result in results {
        if result.status().is_success() {
            success_count += 1;
        } else if result.status().is_client_error() {
            client_errors += 1;
        } else if result.status().is_server_error() {
            server_errors += 1;
        }
    }

    println!("Graceful degradation: {} success, {} client errors, {} server errors in {:?}", 
             success_count, client_errors, server_errors, duration);

    // Should maintain reasonable success rate and minimize server errors
    let success_rate = success_count as f64 / concurrent_requests as f64;
    let server_error_rate = server_errors as f64 / concurrent_requests as f64;
    
    assert!(success_rate > 0.7, "Success rate too low: {:.2}%", success_rate * 100.0);
    assert!(server_error_rate < 0.1, "Server error rate too high: {:.2}%", server_error_rate * 100.0);
}

/// Test error recovery and retry mechanisms
#[tokio::test]
async fn test_error_recovery() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    // Simulate transient errors by sending malformed requests followed by valid ones
    let test_cycles = 10;
    let mut recovery_success = 0;

    for i in 0..test_cycles {
        // Send malformed request (should fail)
        let malformed_payload = json!({
            "name": "", // Invalid empty name
            "asset_type": "InvalidType",
            "total_value": "not_a_number"
        });

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&malformed_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error(), "Malformed request should fail");

        // Immediately follow with valid request (should succeed)
        let valid_payload = json!({
            "name": format!("Recovery Asset {}", i),
            "description": "Asset after error recovery",
            "asset_type": "RealEstate",
            "total_value": "1000000.00"
        });

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&valid_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        if resp.status().is_success() {
            recovery_success += 1;
        }
    }

    // Service should recover from errors quickly
    let recovery_rate = recovery_success as f64 / test_cycles as f64;
    assert!(recovery_rate > 0.9, "Error recovery rate too low: {:.2}%", recovery_rate * 100.0);
}

/// Test timeout handling
#[tokio::test]
async fn test_timeout_handling() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/health", web::get().to(health_check))
    ).await;

    // Create requests with varying complexity
    let requests = vec![
        // Simple request
        json!({
            "name": "Simple Asset",
            "description": "Simple",
            "asset_type": "RealEstate",
            "total_value": "1000000.00"
        }),
        // Complex request with large data
        json!({
            "name": "Complex Asset",
            "description": "X".repeat(10000),
            "asset_type": "RealEstate",
            "total_value": "1000000.00",
            "location": "Location".repeat(500)
        }),
    ];

    for (i, payload) in requests.iter().enumerate() {
        let start_time = Instant::now();
        
        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        let duration = start_time.elapsed();

        // Requests should complete within reasonable time
        assert!(duration < Duration::from_secs(5), "Request {} took too long: {:?}", i, duration);
        
        // Should either succeed or fail gracefully
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }
}

/// Test circuit breaker pattern simulation
#[tokio::test]
async fn test_circuit_breaker_simulation() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/health", web::get().to(health_check))
    ).await;

    let failure_threshold = 5;
    let mut consecutive_failures = 0;
    let mut circuit_open = false;

    // Simulate failures and recovery
    for i in 0..20 {
        let payload = if i % 7 == 0 && !circuit_open {
            // Simulate failure condition
            json!({
                "name": "",
                "asset_type": "InvalidType",
                "total_value": "invalid"
            })
        } else {
            // Normal request
            json!({
                "name": format!("Circuit Test Asset {}", i),
                "description": "Circuit breaker test",
                "asset_type": "RealEstate",
                "total_value": "1000000.00"
            })
        };

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;

        if resp.status().is_server_error() {
            consecutive_failures += 1;
            if consecutive_failures >= failure_threshold {
                circuit_open = true;
                println!("Circuit breaker would open at iteration {}", i);
            }
        } else {
            consecutive_failures = 0;
            if circuit_open {
                circuit_open = false;
                println!("Circuit breaker would close at iteration {}", i);
            }
        }

        // Health check should always work
        let health_req = test::TestRequest::get().uri("/health").to_request();
        let health_resp = test::call_service(&app, health_req).await;
        assert!(health_resp.status().is_success());
    }
}

/// Test resource cleanup under stress
#[tokio::test]
async fn test_resource_cleanup() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/assets/{id}", web::delete().to(delete_asset))
    ).await;

    let asset_count = 50;
    let mut created_assets = Vec::new();

    // Create many assets
    for i in 0..asset_count {
        let payload = json!({
            "name": format!("Cleanup Test Asset {}", i),
            "description": "Asset for cleanup testing",
            "asset_type": "RealEstate",
            "total_value": "1000000.00"
        });

        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        if resp.status().is_success() {
            let body: serde_json::Value = test::read_body_json(resp).await;
            if let Some(id) = body.get("id") {
                created_assets.push(id.as_str().unwrap().to_string());
            }
        }
    }

    // Delete assets concurrently
    let delete_tasks: Vec<_> = created_assets.iter().map(|asset_id| {
        let app = &app;
        let asset_id = asset_id.clone();
        async move {
            let req = test::TestRequest::delete()
                .uri(&format!("/assets/{}", asset_id))
                .to_request();

            test::call_service(app, req).await
        }
    }).collect();

    let delete_results = join_all(delete_tasks).await;
    
    // Most deletions should succeed
    let successful_deletions = delete_results.iter()
        .filter(|resp| resp.status().is_success())
        .count();

    let deletion_rate = successful_deletions as f64 / created_assets.len() as f64;
    assert!(deletion_rate > 0.8, "Resource cleanup rate too low: {:.2}%", deletion_rate * 100.0);
}

/// Test service behavior during rapid scaling
#[tokio::test]
async fn test_rapid_scaling() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/health", web::get().to(health_check))
    ).await;

    // Simulate rapid scaling by varying load intensity
    let phases = vec![
        (10, Duration::from_millis(100)),  // Low load
        (50, Duration::from_millis(50)),   // Medium load
        (100, Duration::from_millis(10)),  // High load
        (20, Duration::from_millis(100)),  // Scale down
    ];

    for (phase_idx, (requests_per_phase, interval)) in phases.iter().enumerate() {
        println!("Scaling phase {}: {} requests with {:?} interval", phase_idx, requests_per_phase, interval);
        
        let phase_start = Instant::now();
        let mut phase_success = 0;

        for i in 0..*requests_per_phase {
            let payload = json!({
                "name": format!("Scaling Asset P{}-{}", phase_idx, i),
                "description": "Asset during scaling test",
                "asset_type": "RealEstate",
                "total_value": "1000000.00"
            });

            let req = test::TestRequest::post()
                .uri("/assets")
                .set_json(&payload)
                .to_request();

            let resp = test::call_service(&app, req).await;
            if resp.status().is_success() {
                phase_success += 1;
            }

            sleep(*interval).await;
        }

        let phase_duration = phase_start.elapsed();
        let phase_success_rate = phase_success as f64 / *requests_per_phase as f64;
        
        println!("Phase {} completed in {:?} with {:.2}% success rate", 
                 phase_idx, phase_duration, phase_success_rate * 100.0);

        // Each phase should maintain reasonable success rate
        assert!(phase_success_rate > 0.8, "Phase {} success rate too low: {:.2}%", 
                phase_idx, phase_success_rate * 100.0);

        // Health check after each phase
        let health_req = test::TestRequest::get().uri("/health").to_request();
        let health_resp = test::call_service(&app, health_req).await;
        assert!(health_resp.status().is_success(), "Health check failed after phase {}", phase_idx);
    }
}
