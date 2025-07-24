// =====================================================================================
// RWA Tokenization Platform - Asset Service API Contract Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, web, App, http::header};
use serde_json::{json, Value};
use service_asset::{
    cache::InMemoryCache,
    handlers::*,
    models::*,
    service::AssetService,
    AppState,
};
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

/// Test API versioning and backward compatibility
#[actix_web::test]
async fn test_api_versioning() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(
                web::scope("/api/v1")
                    .route("/assets", web::post().to(create_asset))
                    .route("/assets", web::get().to(list_assets))
                    .route("/assets/{id}", web::get().to(get_asset))
            )
    ).await;

    // Test v1 API endpoints
    let payload = json!({
        "name": "API Version Test Asset",
        "description": "Testing API versioning",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/assets")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert!(body["id"].is_string());
    assert_eq!(body["name"], "API Version Test Asset");
    
    // Verify API version in response headers
    if let Some(api_version) = resp.headers().get("API-Version") {
        assert_eq!(api_version.to_str().unwrap(), "v1");
    }
}

/// Test request/response schema validation
#[actix_web::test]
async fn test_schema_validation() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    // Test valid schema
    let valid_payload = json!({
        "name": "Valid Asset",
        "description": "Valid description",
        "asset_type": "RealEstate",
        "total_value": "1000000.00",
        "location": "Valid Location"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&valid_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test invalid schemas
    let invalid_schemas = vec![
        // Missing required fields
        json!({
            "description": "Missing name and asset_type"
        }),
        // Invalid data types
        json!({
            "name": "Test Asset",
            "description": "Test",
            "asset_type": "RealEstate",
            "total_value": 1000000 // Should be string
        }),
        // Invalid enum values
        json!({
            "name": "Test Asset",
            "description": "Test",
            "asset_type": "InvalidType",
            "total_value": "1000000.00"
        }),
        // Invalid field types
        json!({
            "name": 123, // Should be string
            "description": "Test",
            "asset_type": "RealEstate",
            "total_value": "1000000.00"
        }),
    ];

    for (i, invalid_payload) in invalid_schemas.iter().enumerate() {
        let req = test::TestRequest::post()
            .uri("/assets")
            .set_json(invalid_payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error(), "Invalid schema {} should be rejected", i);
    }
}

/// Test HTTP status codes compliance
#[actix_web::test]
async fn test_http_status_codes() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/assets", web::get().to(list_assets))
            .route("/assets/{id}", web::get().to(get_asset))
            .route("/assets/{id}", web::put().to(update_asset))
            .route("/assets/{id}", web::delete().to(delete_asset))
    ).await;

    // Test 201 Created for successful creation
    let payload = json!({
        "name": "Status Code Test Asset",
        "description": "Testing status codes",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201); // Created

    let body: Value = test::read_body_json(resp).await;
    let asset_id = body["id"].as_str().unwrap();

    // Test 200 OK for successful retrieval
    let req = test::TestRequest::get()
        .uri(&format!("/assets/{}", asset_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200); // OK

    // Test 200 OK for successful list
    let req = test::TestRequest::get()
        .uri("/assets")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200); // OK

    // Test 200 OK for successful update
    let update_payload = json!({
        "name": "Updated Asset Name"
    });

    let req = test::TestRequest::put()
        .uri(&format!("/assets/{}", asset_id))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200); // OK

    // Test 204 No Content for successful deletion
    let req = test::TestRequest::delete()
        .uri(&format!("/assets/{}", asset_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204); // No Content

    // Test 404 Not Found for non-existent resource
    let req = test::TestRequest::get()
        .uri(&format!("/assets/{}", Uuid::new_v4()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404); // Not Found

    // Test 400 Bad Request for invalid UUID
    let req = test::TestRequest::get()
        .uri("/assets/invalid-uuid")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request
}

/// Test content type handling
#[actix_web::test]
async fn test_content_type_handling() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    let payload = json!({
        "name": "Content Type Test Asset",
        "description": "Testing content types",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    // Test correct content type
    let req = test::TestRequest::post()
        .uri("/assets")
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test incorrect content type
    let req = test::TestRequest::post()
        .uri("/assets")
        .insert_header((header::CONTENT_TYPE, "text/plain"))
        .set_payload(payload.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Test missing content type
    let req = test::TestRequest::post()
        .uri("/assets")
        .set_payload(payload.to_string())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

/// Test response format consistency
#[actix_web::test]
async fn test_response_format_consistency() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/assets", web::get().to(list_assets))
            .route("/assets/{id}", web::get().to(get_asset))
    ).await;

    // Create an asset
    let payload = json!({
        "name": "Response Format Test Asset",
        "description": "Testing response formats",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let create_body: Value = test::read_body_json(resp).await;
    let asset_id = create_body["id"].as_str().unwrap();

    // Verify create response format
    assert!(create_body["id"].is_string());
    assert!(create_body["name"].is_string());
    assert!(create_body["description"].is_string());
    assert!(create_body["asset_type"].is_string());
    assert!(create_body["total_value"].is_string());
    assert!(create_body["created_at"].is_string());
    assert!(create_body["updated_at"].is_string());

    // Get single asset and verify format consistency
    let req = test::TestRequest::get()
        .uri(&format!("/assets/{}", asset_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let get_body: Value = test::read_body_json(resp).await;
    
    // Verify same fields are present with same types
    assert_eq!(create_body["id"], get_body["id"]);
    assert_eq!(create_body["name"], get_body["name"]);
    assert_eq!(create_body["asset_type"], get_body["asset_type"]);

    // List assets and verify format consistency
    let req = test::TestRequest::get()
        .uri("/assets")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let list_body: Value = test::read_body_json(resp).await;
    
    // Verify list response structure
    assert!(list_body["assets"].is_array());
    assert!(list_body["pagination"].is_object());
    
    let pagination = &list_body["pagination"];
    assert!(pagination["page"].is_number());
    assert!(pagination["per_page"].is_number());
    assert!(pagination["total"].is_number());
    assert!(pagination["total_pages"].is_number());

    // Verify asset in list has same format
    let assets = list_body["assets"].as_array().unwrap();
    if !assets.is_empty() {
        let list_asset = &assets[0];
        assert!(list_asset["id"].is_string());
        assert!(list_asset["name"].is_string());
        assert!(list_asset["asset_type"].is_string());
    }
}

/// Test error response format consistency
#[actix_web::test]
async fn test_error_response_format() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/assets/{id}", web::get().to(get_asset))
    ).await;

    // Test validation error format
    let invalid_payload = json!({
        "name": "",
        "asset_type": "InvalidType"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&invalid_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    let error_body: Value = test::read_body_json(resp).await;
    
    // Verify error response structure
    assert!(error_body["error"].is_string());
    assert!(error_body["message"].is_string());
    
    // Test not found error format
    let req = test::TestRequest::get()
        .uri(&format!("/assets/{}", Uuid::new_v4()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let not_found_body: Value = test::read_body_json(resp).await;
    
    // Verify consistent error structure
    assert!(not_found_body["error"].is_string());
    assert!(not_found_body["message"].is_string());
}

/// Test pagination contract
#[actix_web::test]
async fn test_pagination_contract() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
            .route("/assets", web::get().to(list_assets))
    ).await;

    // Create multiple assets
    for i in 0..15 {
        let payload = json!({
            "name": format!("Pagination Test Asset {}", i),
            "description": "Testing pagination",
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

    // Test default pagination
    let req = test::TestRequest::get()
        .uri("/assets")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let pagination = &body["pagination"];
    
    assert_eq!(pagination["page"], 1);
    assert_eq!(pagination["per_page"], 20); // Default per_page
    assert!(pagination["total"].as_u64().unwrap() >= 15);

    // Test custom pagination
    let req = test::TestRequest::get()
        .uri("/assets?page=2&per_page=5")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let pagination = &body["pagination"];
    
    assert_eq!(pagination["page"], 2);
    assert_eq!(pagination["per_page"], 5);
    
    let assets = body["assets"].as_array().unwrap();
    assert!(assets.len() <= 5);

    // Test pagination bounds
    let req = test::TestRequest::get()
        .uri("/assets?page=999&per_page=10")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let assets = body["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 0); // No assets on page 999
}
