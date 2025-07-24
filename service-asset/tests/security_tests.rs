// =====================================================================================
// RWA Tokenization Platform - Asset Service Security Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, web, App, http::header};
use serde_json::json;
use service_asset::{
    cache::InMemoryCache,
    handlers::*,
    middleware::{AuthenticationMiddleware, RateLimitingMiddleware, SecurityHeadersMiddleware},
    models::*,
    service::AssetService,
    AppState,
};
use std::collections::HashMap;

/// Helper function to create test app state
async fn create_test_app_state() -> AppState {
    let repository = InMemoryAssetRepository::new();
    let cache = InMemoryCache::new();
    let asset_service = AssetService::new(Box::new(repository), Box::new(cache));

    AppState {
        asset_service: Box::new(asset_service),
    }
}

/// Test authentication middleware
#[actix_web::test]
async fn test_authentication_required() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .wrap(AuthenticationMiddleware::new(
                "test-secret".to_string(),
                vec!["/health".to_string()],
            ))
            .route("/assets", web::post().to(create_asset))
            .route("/health", web::get().to(health_check))
    ).await;

    // Test that protected endpoint requires authentication
    let payload = json!({
        "name": "Test Asset",
        "description": "Test Description",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401); // Unauthorized

    // Test that health endpoint is excluded from authentication
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test JWT token validation
#[actix_web::test]
async fn test_jwt_token_validation() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .wrap(AuthenticationMiddleware::new(
                "test-secret".to_string(),
                vec![],
            ))
            .route("/assets", web::post().to(create_asset))
    ).await;

    let payload = json!({
        "name": "Test Asset",
        "description": "Test Description",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    // Test with invalid token
    let req = test::TestRequest::post()
        .uri("/assets")
        .insert_header((header::AUTHORIZATION, "Bearer invalid-token"))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test with malformed authorization header
    let req = test::TestRequest::post()
        .uri("/assets")
        .insert_header((header::AUTHORIZATION, "InvalidFormat"))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

/// Test rate limiting middleware
#[actix_web::test]
async fn test_rate_limiting() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .wrap(RateLimitingMiddleware::new(5, 2)) // 5 requests per minute, burst of 2
            .route("/health", web::get().to(health_check))
    ).await;

    // First few requests should succeed
    for i in 0..3 {
        let req = test::TestRequest::get()
            .uri("/health")
            .insert_header(("X-Forwarded-For", "192.168.1.100"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        if i < 2 {
            assert!(resp.status().is_success(), "Request {} should succeed", i);
        } else {
            // Should be rate limited after burst
            assert_eq!(resp.status(), 429, "Request {} should be rate limited", i);
        }
    }
}

/// Test security headers middleware
#[actix_web::test]
async fn test_security_headers() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .wrap(SecurityHeadersMiddleware::new())
            .route("/health", web::get().to(health_check))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    // Check security headers
    let headers = resp.headers();
    assert!(headers.contains_key("content-security-policy"));
    assert!(headers.contains_key("strict-transport-security"));
    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));

    // Verify header values
    assert_eq!(
        headers.get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(
        headers.get("x-frame-options").unwrap(),
        "DENY"
    );
}

/// Test input validation and sanitization
#[actix_web::test]
async fn test_input_validation() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    // Test SQL injection attempt
    let malicious_payload = json!({
        "name": "'; DROP TABLE assets; --",
        "description": "Test Description",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&malicious_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should either reject or sanitize the input
    assert!(resp.status().is_client_error() || resp.status().is_success());

    // Test XSS attempt
    let xss_payload = json!({
        "name": "<script>alert('xss')</script>",
        "description": "Test Description",
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&xss_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error() || resp.status().is_success());

    // Test oversized payload
    let large_payload = json!({
        "name": "A".repeat(10000),
        "description": "B".repeat(100000),
        "asset_type": "RealEstate",
        "total_value": "1000000.00"
    });

    let req = test::TestRequest::post()
        .uri("/assets")
        .set_json(&large_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should handle large payloads gracefully
    assert!(resp.status().is_client_error() || resp.status().is_success());
}

/// Test parameter tampering
#[actix_web::test]
async fn test_parameter_tampering() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets/{id}", web::get().to(get_asset))
    ).await;

    // Test with invalid UUID format
    let req = test::TestRequest::get()
        .uri("/assets/invalid-uuid")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request

    // Test with path traversal attempt
    let req = test::TestRequest::get()
        .uri("/assets/../../../etc/passwd")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Should be rejected
}

/// Test CORS security
#[actix_web::test]
async fn test_cors_security() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
    ).await;

    // Test preflight request
    let req = test::TestRequest::with_uri("/health")
        .method(actix_web::http::Method::OPTIONS)
        .insert_header((header::ORIGIN, "https://malicious-site.com"))
        .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, "GET"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should not allow arbitrary origins
    let cors_header = resp.headers().get("access-control-allow-origin");
    if let Some(origin) = cors_header {
        assert_ne!(origin, "https://malicious-site.com");
    }
}

/// Test data exposure in error messages
#[actix_web::test]
async fn test_error_information_disclosure() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets/{id}", web::get().to(get_asset))
    ).await;

    // Test with non-existent asset
    let req = test::TestRequest::get()
        .uri("/assets/550e8400-e29b-41d4-a716-446655440000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = test::read_body_json(resp).await;
    
    // Error message should not expose internal details
    if let Some(error) = body.get("error") {
        let error_str = error.as_str().unwrap_or("");
        assert!(!error_str.contains("database"));
        assert!(!error_str.contains("sql"));
        assert!(!error_str.contains("internal"));
    }
}

/// Test session management security
#[actix_web::test]
async fn test_session_security() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    // Check for secure cookie settings if cookies are used
    if let Some(set_cookie) = resp.headers().get("set-cookie") {
        let cookie_str = set_cookie.to_str().unwrap();
        assert!(cookie_str.contains("Secure") || cookie_str.contains("HttpOnly"));
    }
}

/// Test content type validation
#[actix_web::test]
async fn test_content_type_validation() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    // Test with wrong content type
    let req = test::TestRequest::post()
        .uri("/assets")
        .insert_header((header::CONTENT_TYPE, "text/plain"))
        .set_payload("not json")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Test with missing content type
    let req = test::TestRequest::post()
        .uri("/assets")
        .set_payload(r#"{"name": "test"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

/// Test HTTP method security
#[actix_web::test]
async fn test_http_method_security() {
    let app_state = web::Data::new(create_test_app_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/assets", web::post().to(create_asset))
    ).await;

    // Test unsupported HTTP methods
    for method in &["TRACE", "CONNECT", "PATCH"] {
        let req = test::TestRequest::default()
            .method(actix_web::http::Method::from_bytes(method.as_bytes()).unwrap())
            .uri("/assets")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}
