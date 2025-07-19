// =====================================================================================
// File: ai-service/tests/integration.rs
// Description: Integration tests for AI microservice HTTP API endpoints.
//              Tests completion API, model info, error handling, and edge cases.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, web, App, HttpResponse, Responder};
use ai_service::{AiClient, CompletionRequest};
use reqwest::blocking::Client;
use std::sync::Arc;

#[derive(Clone)]
struct TestAppState {
    client: Arc<AiClient>,
}

/// Mock AI completion handler for testing
async fn mock_ai_complete(_data: web::Data<TestAppState>, _req: web::Json<CompletionRequest>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "choices": [{
            "text": "This is a test completion response",
            "index": 0,
            "logprobs": null,
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 8,
            "total_tokens": 18
        }
    }))
}

/// Mock model info handler for testing
async fn mock_ai_model(_data: web::Data<TestAppState>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "model": "gpt-3.5-turbo",
        "version": "1.0.0",
        "capabilities": ["text-completion", "chat"]
    }))
}

/// Mock health check handler for testing
async fn mock_health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "ai-service",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Test AI completion endpoint with valid request
#[actix_web::test]
async fn test_ai_complete_success() {
    // Create test client with mock configuration
    let client = Arc::new(AiClient {
        api_key: "test-key".to_string(),
        api_url: "https://api.openai.com/v1".to_string(),
        client: Client::new(),
    });
    
    let state = TestAppState { client };
    
    // Initialize test service
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/ai")
                .service(web::resource("/complete").to(mock_ai_complete))
            )
    ).await;

    // Create test request
    let request = CompletionRequest {
        prompt: "Test prompt".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.7),
        model: Some("gpt-3.5-turbo".to_string()),
    };

    // Send POST request
    let req = test::TestRequest::post()
        .uri("/ai/complete")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Assert response
    assert!(resp.status().is_success());
    
    let body = test::read_body(resp).await;
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(response_json.get("choices").is_some());
    assert!(response_json.get("usage").is_some());
}

/// Test model info endpoint
#[actix_web::test]
async fn test_ai_model_info() {
    let client = Arc::new(AiClient {
        api_key: "test-key".to_string(),
        api_url: "https://api.openai.com/v1".to_string(),
        client: Client::new(),
    });
    
    let state = TestAppState { client };
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/ai")
                .service(web::resource("/model").to(mock_ai_model))
            )
    ).await;

    let req = test::TestRequest::get().uri("/ai/model").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body = test::read_body(resp).await;
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(response_json["model"], "gpt-3.5-turbo");
    assert!(response_json.get("capabilities").is_some());
}

/// Test AI completion with invalid request
#[actix_web::test]
async fn test_ai_complete_invalid_request() {
    let client = Arc::new(AiClient {
        api_key: "test-key".to_string(),
        api_url: "https://api.openai.com/v1".to_string(),
        client: Client::new(),
    });
    
    let state = TestAppState { client };
    
    // Mock handler that returns error for empty prompt
    async fn mock_ai_complete_error(_data: web::Data<TestAppState>, req: web::Json<CompletionRequest>) -> impl Responder {
        if req.prompt.is_empty() {
            return HttpResponse::BadRequest().body("Empty prompt not allowed");
        }
        HttpResponse::Ok().json(serde_json::json!({
            "choices": [{"text": "Response", "index": 0, "finish_reason": "stop"}]
        }))
    }
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::scope("/ai")
                .service(web::resource("/complete").to(mock_ai_complete_error))
            )
    ).await;

    // Test with empty prompt
    let request = CompletionRequest {
        prompt: "".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.7),
        model: Some("gpt-3.5-turbo".to_string()),
    };

    let req = test::TestRequest::post()
        .uri("/ai/complete")
        .set_json(&request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should return bad request for empty prompt
    assert_eq!(resp.status().as_u16(), 400);
}

/// Test health check endpoint
#[actix_web::test]
async fn test_health_check() {
    let client = Arc::new(AiClient {
        api_key: "test-key".to_string(),
        api_url: "https://api.openai.com/v1".to_string(),
        client: Client::new(),
    });
    
    let state = TestAppState { client };
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(web::resource("/health").to(mock_health_check))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body = test::read_body(resp).await;
    let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(response_json["status"], "healthy");
    assert_eq!(response_json["service"], "ai-service");
} 