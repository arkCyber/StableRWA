// =====================================================================================
// File: tests/integration/api_tests.rs
// Description: API integration tests for RWA Platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use super::*;
use serde_json::{json, Value};
use reqwest::StatusCode;

#[tokio::test]
async fn test_health_check() {
    let config = init_test_env().await;
    let client = TestClient::new(config.api_base_url);

    let response = client.get("/health").await.unwrap();
    TestAssertions::assert_response_status(&response, StatusCode::OK);

    let json = TestAssertions::assert_response_json(response, &["status", "timestamp"]).await;
    assert_eq!(json["status"], "healthy");
}

#[tokio::test]
async fn test_user_registration_and_login() {
    let config = init_test_env().await;
    cleanup_test_data().await;
    
    let client = TestClient::new(config.api_base_url);
    let user_data = TestDataFactory::create_test_user();

    // Test user registration
    let register_response = client
        .post("/api/v1/auth/register", serde_json::to_value(user_data).unwrap())
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&register_response, StatusCode::CREATED);
    
    let register_json = TestAssertions::assert_response_json(
        register_response,
        &["user_id", "email", "first_name", "last_name", "created_at"]
    ).await;
    
    TestAssertions::assert_uuid_format(register_json["user_id"].as_str().unwrap());
    TestAssertions::assert_timestamp_format(register_json["created_at"].as_str().unwrap());

    // Test user login
    let login_data = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let login_response = client
        .post("/api/v1/auth/login", login_data)
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&login_response, StatusCode::OK);
    
    let login_json = TestAssertions::assert_response_json(
        login_response,
        &["access_token", "refresh_token", "expires_in", "user"]
    ).await;

    assert!(login_json["access_token"].as_str().unwrap().len() > 0);
    assert!(login_json["refresh_token"].as_str().unwrap().len() > 0);
    assert!(login_json["expires_in"].as_u64().unwrap() > 0);

    cleanup_test_data().await;
}

#[tokio::test]
async fn test_duplicate_user_registration() {
    let config = init_test_env().await;
    cleanup_test_data().await;
    
    let client = TestClient::new(config.api_base_url);
    let user_data = TestDataFactory::create_test_user();

    // First registration should succeed
    let first_response = client
        .post("/api/v1/auth/register", serde_json::to_value(&user_data).unwrap())
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&first_response, StatusCode::CREATED);

    // Second registration with same email should fail
    let second_response = client
        .post("/api/v1/auth/register", serde_json::to_value(&user_data).unwrap())
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&second_response, StatusCode::CONFLICT);

    cleanup_test_data().await;
}

#[tokio::test]
async fn test_invalid_login_credentials() {
    let config = init_test_env().await;
    cleanup_test_data().await;
    
    let client = TestClient::new(config.api_base_url);

    // Test with non-existent user
    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "password123"
    });

    let response = client
        .post("/api/v1/auth/login", login_data)
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&response, StatusCode::UNAUTHORIZED);

    // Register a user first
    let user_data = TestDataFactory::create_test_user();
    client
        .post("/api/v1/auth/register", serde_json::to_value(user_data).unwrap())
        .await
        .unwrap();

    // Test with wrong password
    let wrong_password_data = json!({
        "email": "test@example.com",
        "password": "wrongpassword"
    });

    let response = client
        .post("/api/v1/auth/login", wrong_password_data)
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&response, StatusCode::UNAUTHORIZED);

    cleanup_test_data().await;
}

authenticated_api_test!(test_user_profile_operations, |client: TestClient| async move {
    // Test get user profile
    let profile_response = client.get("/api/v1/users/profile").await.unwrap();
    TestAssertions::assert_response_status(&profile_response, StatusCode::OK);
    
    let profile_json = TestAssertions::assert_response_json(
        profile_response,
        &["user_id", "email", "first_name", "last_name", "created_at"]
    ).await;

    // Test update user profile
    let update_data = json!({
        "first_name": "Jane",
        "last_name": "Smith",
        "phone": "+1987654321"
    });

    let update_response = client
        .put("/api/v1/users/profile", update_data)
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&update_response, StatusCode::OK);
    
    let updated_json = TestAssertions::assert_response_json(
        update_response,
        &["user_id", "email", "first_name", "last_name"]
    ).await;

    assert_eq!(updated_json["first_name"], "Jane");
    assert_eq!(updated_json["last_name"], "Smith");
});

authenticated_api_test!(test_asset_crud_operations, |client: TestClient| async move {
    // Test create asset
    let asset_data = TestDataFactory::create_test_asset();
    
    let create_response = client
        .post("/api/v1/assets", serde_json::to_value(asset_data).unwrap())
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&create_response, StatusCode::CREATED);
    
    let created_asset = TestAssertions::assert_response_json(
        create_response,
        &["id", "name", "description", "asset_type", "total_value", "currency", "owner_id", "created_at"]
    ).await;

    let asset_id = created_asset["id"].as_str().unwrap();
    TestAssertions::assert_uuid_format(asset_id);

    // Test get asset
    let get_response = client
        .get(&format!("/api/v1/assets/{}", asset_id))
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&get_response, StatusCode::OK);
    
    let retrieved_asset = TestAssertions::assert_response_json(
        get_response,
        &["id", "name", "description", "asset_type", "total_value"]
    ).await;

    assert_eq!(retrieved_asset["id"], created_asset["id"]);
    assert_eq!(retrieved_asset["name"], "Test Real Estate");

    // Test update asset
    let update_data = json!({
        "name": "Updated Real Estate",
        "total_value": 1200000
    });

    let update_response = client
        .put(&format!("/api/v1/assets/{}", asset_id), update_data)
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&update_response, StatusCode::OK);
    
    let updated_asset = TestAssertions::assert_response_json(
        update_response,
        &["id", "name", "total_value"]
    ).await;

    assert_eq!(updated_asset["name"], "Updated Real Estate");
    assert_eq!(updated_asset["total_value"], 1200000);

    // Test list assets
    let list_response = client
        .get("/api/v1/assets?page=1&per_page=10")
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&list_response, StatusCode::OK);
    
    let list_json = TestAssertions::assert_response_json(
        list_response,
        &["data", "pagination"]
    ).await;

    let assets = list_json["data"].as_array().unwrap();
    assert!(assets.len() > 0);
    assert_eq!(assets[0]["id"], created_asset["id"]);

    // Test delete asset
    let delete_response = client
        .delete(&format!("/api/v1/assets/{}", asset_id))
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&delete_response, StatusCode::NO_CONTENT);

    // Verify asset is deleted
    let get_deleted_response = client
        .get(&format!("/api/v1/assets/{}", asset_id))
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&get_deleted_response, StatusCode::NOT_FOUND);
});

authenticated_api_test!(test_asset_tokenization, |client: TestClient| async move {
    // Create an asset first
    let asset_data = TestDataFactory::create_test_asset();
    
    let create_response = client
        .post("/api/v1/assets", serde_json::to_value(asset_data).unwrap())
        .await
        .unwrap();
    
    let created_asset = create_response.json::<Value>().await.unwrap();
    let asset_id = created_asset["id"].as_str().unwrap();

    // Test tokenization
    let tokenization_data = TestDataFactory::create_tokenization_request();
    
    let tokenize_response = client
        .post(
            &format!("/api/v1/assets/{}/tokenize", asset_id),
            serde_json::to_value(tokenization_data).unwrap()
        )
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&tokenize_response, StatusCode::OK);
    
    let tokenization_result = TestAssertions::assert_response_json(
        tokenize_response,
        &["asset_id", "token_address", "blockchain_network", "token_supply", "token_symbol", "transaction_hash"]
    ).await;

    assert_eq!(tokenization_result["asset_id"], asset_id);
    assert_eq!(tokenization_result["blockchain_network"], "ethereum_testnet");
    assert_eq!(tokenization_result["token_supply"], 1000000);
    assert_eq!(tokenization_result["token_symbol"], "TEST");

    // Verify asset is now tokenized
    let get_response = client
        .get(&format!("/api/v1/assets/{}", asset_id))
        .await
        .unwrap();
    
    let updated_asset = get_response.json::<Value>().await.unwrap();
    assert_eq!(updated_asset["is_tokenized"], true);
    assert!(updated_asset["token_address"].as_str().unwrap().len() > 0);
});

authenticated_api_test!(test_payment_processing, |client: TestClient| async move {
    // Test process payment
    let payment_data = TestDataFactory::create_test_payment();
    
    let payment_response = client
        .post("/api/v1/payments", serde_json::to_value(payment_data).unwrap())
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&payment_response, StatusCode::CREATED);
    
    let payment_result = TestAssertions::assert_response_json(
        payment_response,
        &["id", "user_id", "amount", "currency", "payment_method_type", "provider", "status", "created_at"]
    ).await;

    let payment_id = payment_result["id"].as_str().unwrap();
    TestAssertions::assert_uuid_format(payment_id);

    assert_eq!(payment_result["amount"], 100);
    assert_eq!(payment_result["currency"], "USD");
    assert_eq!(payment_result["payment_method_type"], "credit_card");
    assert_eq!(payment_result["provider"], "stripe");

    // Test get payment
    let get_payment_response = client
        .get(&format!("/api/v1/payments/{}", payment_id))
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&get_payment_response, StatusCode::OK);
    
    let retrieved_payment = TestAssertions::assert_response_json(
        get_payment_response,
        &["id", "amount", "status"]
    ).await;

    assert_eq!(retrieved_payment["id"], payment_id);

    // Test list payments
    let list_payments_response = client
        .get("/api/v1/payments?page=1&per_page=10")
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&list_payments_response, StatusCode::OK);
    
    let payments_list = TestAssertions::assert_response_json(
        list_payments_response,
        &["data", "pagination"]
    ).await;

    let payments = payments_list["data"].as_array().unwrap();
    assert!(payments.len() > 0);
});

#[tokio::test]
async fn test_unauthorized_access() {
    let config = init_test_env().await;
    let client = TestClient::new(config.api_base_url);

    // Test accessing protected endpoints without authentication
    let protected_endpoints = vec![
        "/api/v1/users/profile",
        "/api/v1/assets",
        "/api/v1/payments",
    ];

    for endpoint in protected_endpoints {
        let response = client.get(endpoint).await.unwrap();
        TestAssertions::assert_response_status(&response, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn test_invalid_request_data() {
    let config = init_test_env().await;
    let client = TestClient::new(config.api_base_url);

    // Test registration with invalid email
    let invalid_user_data = json!({
        "email": "invalid-email",
        "password": "password123",
        "first_name": "John",
        "last_name": "Doe"
    });

    let response = client
        .post("/api/v1/auth/register", invalid_user_data)
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&response, StatusCode::BAD_REQUEST);

    // Test registration with missing required fields
    let incomplete_user_data = json!({
        "email": "test@example.com"
        // Missing password, first_name, last_name
    });

    let response = client
        .post("/api/v1/auth/register", incomplete_user_data)
        .await
        .unwrap();
    
    TestAssertions::assert_response_status(&response, StatusCode::BAD_REQUEST);
}
