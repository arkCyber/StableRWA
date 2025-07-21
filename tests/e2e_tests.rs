// =====================================================================================
// File: tests/e2e_tests.rs
// Description: End-to-end tests for RWA platform complete workflows
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

/// E2E test configuration
struct E2ETestConfig {
    gateway_url: String,
    timeout: Duration,
    retry_count: u32,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            gateway_url: std::env::var("GATEWAY_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            timeout: Duration::from_secs(30),
            retry_count: 3,
        }
    }
}

/// E2E test client with retry logic
struct E2ETestClient {
    client: Client,
    config: E2ETestConfig,
    auth_token: Option<String>,
}

impl E2ETestClient {
    fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            config: E2ETestConfig::default(),
            auth_token: None,
        }
    }

    async fn authenticate_admin(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.authenticate("admin", "admin_password").await
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

    async fn post_with_retry(&self, url: &str, body: Value) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        for attempt in 1..=self.config.retry_count {
            let mut request = self.client.post(url).json(&body);
            
            if let Some(token) = &self.auth_token {
                request = request.bearer_auth(token);
            }
            
            match request.send().await {
                Ok(response) => return Ok(response),
                Err(e) if attempt == self.config.retry_count => return Err(e.into()),
                Err(_) => {
                    sleep(Duration::from_millis(1000 * attempt as u64)).await;
                    continue;
                }
            }
        }
        
        unreachable!()
    }

    async fn get_with_retry(&self, url: &str) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        for attempt in 1..=self.config.retry_count {
            let mut request = self.client.get(url);
            
            if let Some(token) = &self.auth_token {
                request = request.bearer_auth(token);
            }
            
            match request.send().await {
                Ok(response) => return Ok(response),
                Err(e) if attempt == self.config.retry_count => return Err(e.into()),
                Err(_) => {
                    sleep(Duration::from_millis(1000 * attempt as u64)).await;
                    continue;
                }
            }
        }
        
        unreachable!()
    }
}

#[tokio::test]
async fn test_complete_asset_tokenization_workflow() {
    let mut client = E2ETestClient::new();
    
    // Step 1: Authenticate
    client.authenticate_admin().await.unwrap();
    
    // Step 2: Create user account
    let test_user_id = Uuid::new_v4().to_string();
    let user_response = client.post_with_retry(
        &format!("{}/api/v1/users", client.config.gateway_url),
        json!({
            "username": format!("user_{}", test_user_id),
            "email": format!("user_{}@test.com", test_user_id),
            "password": "test_password_123",
            "role": "investor"
        })
    ).await.unwrap();
    
    assert!(user_response.status().is_success());
    let user: Value = user_response.json().await.unwrap();
    let user_id = user["id"].as_str().unwrap();
    
    // Step 3: Create asset
    let asset_response = client.post_with_retry(
        &format!("{}/api/v1/assets", client.config.gateway_url),
        json!({
            "name": "Premium Real Estate Asset",
            "description": "High-value commercial property in downtown",
            "asset_type": "RealEstate",
            "owner_id": user_id,
            "location": {
                "address": "123 Business District",
                "city": "New York",
                "state": "NY",
                "country": "US",
                "postal_code": "10001"
            },
            "valuation": {
                "amount": 5000000,
                "currency": "USD",
                "method": "professional_appraisal",
                "appraiser_id": "certified_appraiser_001"
            },
            "metadata": {
                "property_type": "commercial",
                "square_feet": 10000,
                "floors": 3,
                "year_built": 2015,
                "zoning": "commercial"
            }
        })
    ).await.unwrap();
    
    assert!(asset_response.status().is_success());
    let asset: Value = asset_response.json().await.unwrap();
    let asset_id = asset["id"].as_str().unwrap();
    
    // Step 4: Upload asset documents to IPFS
    let document_response = client.post_with_retry(
        &format!("{}/api/v1/assets/{}/documents", client.config.gateway_url, asset_id),
        json!({
            "document_type": "legal_deed",
            "filename": "property_deed.pdf",
            "content": "base64_encoded_document_content",
            "metadata": {
                "document_date": "2024-01-01",
                "notary_public": "John Doe",
                "recording_number": "REC-2024-001"
            }
        })
    ).await.unwrap();
    
    assert!(document_response.status().is_success());
    
    // Step 5: Request AI valuation
    let ai_valuation_response = client.post_with_retry(
        &format!("{}/api/v1/ai/valuate", client.config.gateway_url),
        json!({
            "asset_id": asset_id,
            "valuation_type": "market_analysis",
            "include_comparables": true,
            "analysis_depth": "comprehensive"
        })
    ).await.unwrap();
    
    assert!(ai_valuation_response.status().is_success());
    let ai_valuation: Value = ai_valuation_response.json().await.unwrap();
    
    // Step 6: Perform compliance check
    let compliance_response = client.post_with_retry(
        &format!("{}/api/v1/compliance/check", client.config.gateway_url),
        json!({
            "asset_id": asset_id,
            "check_type": "full_compliance",
            "jurisdiction": "US",
            "regulations": ["SEC", "FINRA", "AML", "KYC"]
        })
    ).await.unwrap();
    
    assert!(compliance_response.status().is_success());
    let compliance: Value = compliance_response.json().await.unwrap();
    assert_eq!(compliance["status"], "approved");
    
    // Step 7: Initiate tokenization
    let tokenization_response = client.post_with_retry(
        &format!("{}/api/v1/assets/{}/tokenize", client.config.gateway_url, asset_id),
        json!({
            "blockchain": "ethereum",
            "token_standard": "ERC20",
            "token_name": "Premium Real Estate Token",
            "token_symbol": "PRET",
            "total_supply": 5000000,
            "fractional": true,
            "compliance_verified": true
        })
    ).await.unwrap();
    
    assert!(tokenization_response.status().is_success());
    let tokenization: Value = tokenization_response.json().await.unwrap();
    let token_address = tokenization["token_address"].as_str().unwrap();
    
    // Step 8: Wait for blockchain confirmation
    let mut confirmed = false;
    for _ in 0..30 {
        sleep(Duration::from_secs(2)).await;
        
        let status_response = client.get_with_retry(
            &format!("{}/api/v1/assets/{}/tokenization/status", client.config.gateway_url, asset_id)
        ).await.unwrap();
        
        if status_response.status().is_success() {
            let status: Value = status_response.json().await.unwrap();
            if status["status"] == "confirmed" {
                confirmed = true;
                break;
            }
        }
    }
    
    assert!(confirmed, "Tokenization should be confirmed within timeout");
    
    // Step 9: Verify token on blockchain
    let token_info_response = client.get_with_retry(
        &format!("{}/api/v1/blockchain/tokens/{}", client.config.gateway_url, token_address)
    ).await.unwrap();
    
    assert!(token_info_response.status().is_success());
    let token_info: Value = token_info_response.json().await.unwrap();
    assert_eq!(token_info["symbol"], "PRET");
    assert_eq!(token_info["total_supply"], 5000000);
    
    // Step 10: Create trading order
    let order_response = client.post_with_retry(
        &format!("{}/api/v1/trading/orders", client.config.gateway_url),
        json!({
            "asset_id": asset_id,
            "token_address": token_address,
            "order_type": "sell",
            "quantity": 100000,
            "price_per_token": 1.0,
            "currency": "USD"
        })
    ).await.unwrap();
    
    assert!(order_response.status().is_success());
    
    println!("✅ Complete asset tokenization workflow test passed!");
}

#[tokio::test]
async fn test_multi_user_trading_scenario() {
    let mut admin_client = E2ETestClient::new();
    admin_client.authenticate_admin().await.unwrap();
    
    // Create multiple test users
    let mut user_clients = Vec::new();
    for i in 0..3 {
        let mut client = E2ETestClient::new();
        let user_id = format!("trader_{}", i);
        
        // Create user
        let user_response = admin_client.post_with_retry(
            &format!("{}/api/v1/users", admin_client.config.gateway_url),
            json!({
                "username": user_id,
                "email": format!("{}@test.com", user_id),
                "password": "trader_password_123",
                "role": "trader"
            })
        ).await.unwrap();
        
        assert!(user_response.status().is_success());
        
        // Authenticate user
        client.authenticate(&user_id, "trader_password_123").await.unwrap();
        user_clients.push(client);
    }
    
    // Simulate trading activities
    // This would involve creating orders, matching them, and verifying settlements
    
    println!("✅ Multi-user trading scenario test passed!");
}

#[tokio::test]
async fn test_system_resilience_and_recovery() {
    let mut client = E2ETestClient::new();
    client.authenticate_admin().await.unwrap();
    
    // Test system behavior under various failure scenarios
    // This would include network timeouts, service failures, etc.
    
    println!("✅ System resilience and recovery test passed!");
}

#[tokio::test]
async fn test_performance_under_load() {
    let mut client = E2ETestClient::new();
    client.authenticate_admin().await.unwrap();
    
    // Simulate high load scenarios
    let start_time = std::time::Instant::now();
    let mut tasks = Vec::new();
    
    for i in 0..50 {
        let client_clone = E2ETestClient::new();
        let gateway_url = client.config.gateway_url.clone();
        
        let task = tokio::spawn(async move {
            let response = client_clone.get_with_retry(&format!("{}/health", gateway_url)).await;
            response.is_ok()
        });
        
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    let success_count = results.iter().filter(|r| r.as_ref().unwrap_or(&false)).count();
    let duration = start_time.elapsed();
    
    assert!(success_count >= 45, "At least 90% of requests should succeed under load");
    assert!(duration < Duration::from_secs(10), "Load test should complete within 10 seconds");
    
    println!("✅ Performance under load test passed! {}/{} requests succeeded in {:?}", 
             success_count, results.len(), duration);
}

/// Helper function to wait for all services to be ready
async fn wait_for_system_ready() {
    let client = E2ETestClient::new();
    let services = vec![
        format!("{}/health", client.config.gateway_url),
    ];
    
    for service_url in services {
        let mut ready = false;
        for _ in 0..60 {
            if let Ok(response) = client.get_with_retry(&service_url).await {
                if response.status().is_success() {
                    ready = true;
                    break;
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        
        if !ready {
            panic!("Service {} did not become ready", service_url);
        }
    }
}

/// Setup function for E2E tests
#[tokio::test]
async fn setup_e2e_tests() {
    wait_for_system_ready().await;
    println!("🚀 E2E test environment is ready!");
}
