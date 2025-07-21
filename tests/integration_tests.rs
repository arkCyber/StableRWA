// =====================================================================================
// File: tests/integration_tests.rs
// Description: Integration tests for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

/// Test configuration
struct TestConfig {
    gateway_url: String,
    asset_service_url: String,
    ai_service_url: String,
    user_service_url: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            gateway_url: "http://localhost:8080".to_string(),
            asset_service_url: "http://localhost:8081".to_string(),
            ai_service_url: "http://localhost:8082".to_string(),
            user_service_url: "http://localhost:8083".to_string(),
        }
    }
}

/// Test client for making HTTP requests
struct TestClient {
    client: Client,
    config: TestConfig,
    auth_token: Option<String>,
}

impl TestClient {
    fn new() -> Self {
        Self {
            client: Client::new(),
            config: TestConfig::default(),
            auth_token: None,
        }
    }

    async fn authenticate(&mut self, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/auth/login", self.config.gateway_url))
            .json(&json!({
                "username": username,
                "password": password
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: Value = response.json().await?;
            self.auth_token = Some(auth_response["access_token"].as_str().unwrap().to_string());
        }

        Ok(())
    }

    async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.get(url);
        
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request.send().await
    }

    async fn post(&self, url: &str, body: Value) -> Result<reqwest::Response, reqwest::Error> {
        let mut request = self.client.post(url).json(&body);
        
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request.send().await
    }
}

#[tokio::test]
async fn test_service_health_checks() {
    let client = TestClient::new();
    
    // Test gateway health
    let response = client.get(&format!("{}/health", client.config.gateway_url)).await.unwrap();
    assert!(response.status().is_success());
    
    // Test asset service health
    let response = client.get(&format!("{}/health", client.config.asset_service_url)).await.unwrap();
    assert!(response.status().is_success());
    
    // Test AI service health
    let response = client.get(&format!("{}/health", client.config.ai_service_url)).await.unwrap();
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_user_registration_and_authentication() {
    let mut client = TestClient::new();
    let test_user = format!("test_user_{}", Uuid::new_v4());
    let test_email = format!("{}@test.com", test_user);
    
    // Register new user
    let registration_response = client.post(
        &format!("{}/api/v1/auth/register", client.config.gateway_url),
        json!({
            "username": test_user,
            "email": test_email,
            "password": "test_password_123"
        })
    ).await.unwrap();
    
    assert!(registration_response.status().is_success());
    
    // Authenticate user
    client.authenticate(&test_user, "test_password_123").await.unwrap();
    assert!(client.auth_token.is_some());
    
    // Test authenticated endpoint
    let profile_response = client.get(&format!("{}/api/v1/user/profile", client.config.gateway_url)).await.unwrap();
    assert!(profile_response.status().is_success());
}

#[tokio::test]
async fn test_asset_tokenization_workflow() {
    let mut client = TestClient::new();
    
    // Authenticate as test user
    client.authenticate("admin", "admin_password").await.unwrap();
    
    // Create asset
    let asset_response = client.post(
        &format!("{}/api/v1/assets", client.config.gateway_url),
        json!({
            "name": "Test Real Estate Asset",
            "description": "A test property for tokenization",
            "asset_type": "RealEstate",
            "location": "Test City, Test Country",
            "valuation": {
                "amount": 1000000,
                "currency": "USD",
                "valuation_date": "2024-01-01T00:00:00Z"
            },
            "metadata": {
                "property_type": "residential",
                "square_feet": 2500,
                "bedrooms": 4,
                "bathrooms": 3
            }
        })
    ).await.unwrap();
    
    assert!(asset_response.status().is_success());
    let asset: Value = asset_response.json().await.unwrap();
    let asset_id = asset["id"].as_str().unwrap();
    
    // Request tokenization
    let tokenization_response = client.post(
        &format!("{}/api/v1/assets/{}/tokenize", client.config.gateway_url, asset_id),
        json!({
            "token_supply": 1000000,
            "token_symbol": "TREA",
            "blockchain_network": "Ethereum"
        })
    ).await.unwrap();
    
    assert!(tokenization_response.status().is_success());
    
    // Check tokenization status
    sleep(Duration::from_secs(2)).await;
    
    let status_response = client.get(&format!("{}/api/v1/assets/{}/tokenization", client.config.gateway_url, asset_id)).await.unwrap();
    assert!(status_response.status().is_success());
}

#[tokio::test]
async fn test_ai_valuation_service() {
    let mut client = TestClient::new();
    
    // Authenticate
    client.authenticate("admin", "admin_password").await.unwrap();
    
    // Request AI valuation
    let valuation_response = client.post(
        &format!("{}/api/v1/ai/valuate", client.config.gateway_url),
        json!({
            "asset_type": "RealEstate",
            "location": "New York, NY",
            "property_details": {
                "square_feet": 1200,
                "bedrooms": 2,
                "bathrooms": 2,
                "year_built": 2010
            },
            "market_data": {
                "comparable_sales": [
                    {"price": 800000, "square_feet": 1100},
                    {"price": 900000, "square_feet": 1300},
                    {"price": 850000, "square_feet": 1250}
                ]
            }
        })
    ).await.unwrap();
    
    assert!(valuation_response.status().is_success());
    let valuation: Value = valuation_response.json().await.unwrap();
    
    assert!(valuation["estimated_value"].is_number());
    assert!(valuation["confidence_score"].is_number());
    assert!(valuation["analysis_report"].is_string());
}

#[tokio::test]
async fn test_ipfs_integration() {
    let mut client = TestClient::new();
    
    // Authenticate
    client.authenticate("admin", "admin_password").await.unwrap();
    
    // Upload document to IPFS
    let upload_response = client.post(
        &format!("{}/api/v1/ipfs/upload", client.config.gateway_url),
        json!({
            "content": "This is a test document for IPFS storage",
            "filename": "test_document.txt",
            "content_type": "text/plain"
        })
    ).await.unwrap();
    
    assert!(upload_response.status().is_success());
    let upload_result: Value = upload_response.json().await.unwrap();
    let ipfs_hash = upload_result["hash"].as_str().unwrap();
    
    // Retrieve document from IPFS
    let retrieve_response = client.get(&format!("{}/api/v1/ipfs/{}", client.config.gateway_url, ipfs_hash)).await.unwrap();
    assert!(retrieve_response.status().is_success());
    
    let content = retrieve_response.text().await.unwrap();
    assert_eq!(content, "This is a test document for IPFS storage");
}

#[tokio::test]
async fn test_rate_limiting() {
    let client = TestClient::new();
    
    // Make multiple rapid requests to test rate limiting
    let mut responses = Vec::new();
    for _ in 0..20 {
        let response = client.get(&format!("{}/api/v1/health", client.config.gateway_url)).await.unwrap();
        responses.push(response.status());
    }
    
    // Should have some rate limited responses (429)
    let rate_limited_count = responses.iter().filter(|status| status.as_u16() == 429).count();
    assert!(rate_limited_count > 0, "Rate limiting should be triggered");
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let client = TestClient::new();
    
    // Test Prometheus metrics endpoint
    let response = client.get(&format!("{}/metrics", client.config.gateway_url)).await.unwrap();
    assert!(response.status().is_success());
    
    let metrics = response.text().await.unwrap();
    assert!(metrics.contains("http_requests_total"));
    assert!(metrics.contains("http_request_duration_seconds"));
}

#[tokio::test]
async fn test_error_handling() {
    let client = TestClient::new();
    
    // Test 404 error
    let response = client.get(&format!("{}/api/v1/nonexistent", client.config.gateway_url)).await.unwrap();
    assert_eq!(response.status(), 404);
    
    // Test invalid JSON
    let response = client.client
        .post(&format!("{}/api/v1/assets", client.config.gateway_url))
        .body("invalid json")
        .header("content-type", "application/json")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 400);
}

/// Helper function to wait for services to be ready
async fn wait_for_services() {
    let client = TestClient::new();
    let max_retries = 30;
    let mut retries = 0;
    
    while retries < max_retries {
        if let Ok(response) = client.get(&format!("{}/health", client.config.gateway_url)).await {
            if response.status().is_success() {
                return;
            }
        }
        
        sleep(Duration::from_secs(1)).await;
        retries += 1;
    }
    
    panic!("Services did not become ready within timeout");
}

/// Setup function to run before all tests
#[tokio::test]
async fn setup_integration_tests() {
    wait_for_services().await;
}
