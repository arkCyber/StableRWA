// =====================================================================================
// File: core-bridge/tests/integration_tests.rs
// Description: Integration tests for bridge services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use core_bridge::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio_test;
use uuid::Uuid;

#[tokio::test]
async fn test_transfer_service_integration() {
    // Test transfer request creation and validation
    let request = TransferRequest::new(
        "user123".to_string(),
        ChainId::Ethereum,
        ChainId::Polygon,
        "USDC".to_string(),
        Decimal::new(100000000, 6), // 100 USDC
        "0x1234567890123456789012345678901234567890".to_string(),
        "0x0987654321098765432109876543210987654321".to_string(),
    )
    .with_priority(transfer::TransferPriority::High)
    .with_memo("Integration test transfer");

    let config = TransferConfig::default();
    assert!(request.validate(&config).is_ok());

    // Test priority ordering
    assert!(
        transfer::TransferPriority::Critical.score() > transfer::TransferPriority::High.score()
    );
    assert!(transfer::TransferPriority::High.score() > transfer::TransferPriority::Normal.score());
}

#[tokio::test]
async fn test_liquidity_service_integration() {
    // Test liquidity request creation
    let pool_id = Uuid::new_v4();
    let request = liquidity::LiquidityRequest::new(
        "provider123".to_string(),
        pool_id,
        ChainId::Ethereum,
        "USDC".to_string(),
        "ETH".to_string(),
        Decimal::new(100000000, 6),            // 100 USDC
        Decimal::new(1000000000000000000, 18), // 1 ETH
    )
    .with_min_amounts(
        Decimal::new(99000000, 6),
        Decimal::new(990000000000000000, 18),
    )
    .with_slippage(Decimal::new(50, 4)); // 0.5%

    assert_eq!(request.provider_id, "provider123");
    assert_eq!(request.pool_id, pool_id);
    assert_eq!(request.slippage_tolerance, Decimal::new(50, 4));

    // Test withdrawal request
    let withdrawal = liquidity::WithdrawalRequest {
        id: Uuid::new_v4(),
        provider_id: "provider123".to_string(),
        pool_id,
        lp_token_amount: Decimal::new(1000000000000000000, 18), // 1 LP token
        min_amount_a: Some(Decimal::new(99000000, 6)),
        min_amount_b: Some(Decimal::new(990000000000000000, 18)),
        deadline: Some(Utc::now() + chrono::Duration::hours(1)),
        created_at: Utc::now(),
    };

    assert_eq!(withdrawal.provider_id, "provider123");
    assert!(withdrawal.deadline.is_some());
}

#[tokio::test]
async fn test_atomic_swap_integration() {
    // Test atomic swap creation
    let swap_request = atomic_swap::SwapRequest::new(
        "user1".to_string(),
        "user2".to_string(),
        ChainId::Ethereum,
        ChainId::Bitcoin,
        "ETH".to_string(),
        "BTC".to_string(),
        Decimal::new(1000000000000000000, 18), // 1 ETH
        Decimal::new(5000000, 8),              // 0.05 BTC
        "0x1234567890123456789012345678901234567890".to_string(),
        "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
        24, // 24 hours timelock
    )
    .unwrap();

    let config = atomic_swap::AtomicSwapConfig::default();
    assert!(swap_request.validate(&config).is_ok());

    // Test hash lock verification
    let secret = "test_secret_123456789012345678901234567890";
    let hash_lock = atomic_swap::generate_hash_lock(secret);
    assert!(atomic_swap::verify_hash_lock(&hash_lock, secret));
    assert!(!atomic_swap::verify_hash_lock(&hash_lock, "wrong_secret"));

    // Test expiration check
    assert!(!swap_request.is_expired());
    assert!(swap_request.time_remaining().is_some());
}

#[tokio::test]
async fn test_security_service_integration() {
    // Test security check request
    let security_request = security::SecurityCheckRequest {
        transaction_id: Uuid::new_v4(),
        user_id: "user123".to_string(),
        source_chain: ChainId::Ethereum,
        destination_chain: ChainId::Polygon,
        amount: Decimal::new(50000000, 2), // $500,000
        asset_symbol: "USDC".to_string(),
        source_address: "0x1234567890123456789012345678901234567890".to_string(),
        destination_address: "0x0987654321098765432109876543210987654321".to_string(),
        timestamp: Utc::now(),
        user_agent: Some("Mozilla/5.0".to_string()),
        ip_address: Some("192.168.1.1".to_string()),
        session_id: Some("session123".to_string()),
    };

    // Test security monitor
    let config = security::SecurityConfig::default();
    let monitor = security::SecurityMonitor::new(config);
    let risk_score = monitor.calculate_risk_score(&security_request);

    assert!(risk_score >= 0.0 && risk_score <= 1.0);

    // Test user risk profile
    let risk_profile = security::UserRiskProfile {
        user_id: "user123".to_string(),
        risk_score: 0.3,
        trust_level: security::TrustLevel::Medium,
        transaction_count: 100,
        total_volume: Decimal::new(1000000000, 2), // $10M
        first_transaction: Utc::now() - chrono::Duration::days(365),
        last_transaction: Utc::now(),
        suspicious_activity_count: 0,
        manual_reviews_count: 1,
        kyc_verified: true,
        enhanced_verification: false,
        notes: vec!["Regular customer".to_string()],
        last_updated: Utc::now(),
    };

    assert_eq!(risk_profile.trust_level, security::TrustLevel::Medium);
    assert!(risk_profile.kyc_verified);
}

#[tokio::test]
async fn test_relayer_service_integration() {
    // Test relayer request creation
    let relayer_request = relayer::RelayRequest::new(
        ChainId::Ethereum,
        ChainId::Polygon,
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
        vec![1, 2, 3, 4, 5],
        "0x0987654321098765432109876543210987654321".to_string(),
        200000, // gas limit
    )
    .with_priority(relayer::RelayPriority::High)
    .with_callback("https://api.example.com/callback");

    assert!(relayer_request.validate().is_ok());
    assert_eq!(relayer_request.priority, relayer::RelayPriority::High);
    assert!(relayer_request.callback_url.is_some());

    // Test relayer node
    let relayer_node = relayer::RelayerNode {
        node_id: "relayer-001".to_string(),
        operator_address: "0x1111111111111111111111111111111111111111".to_string(),
        supported_chains: vec![ChainId::Ethereum, ChainId::Polygon],
        stake_amount: Decimal::new(10000000, 2), // $100,000
        reputation_score: 0.95,
        success_rate: 0.99,
        average_relay_time: chrono::Duration::minutes(5),
        total_relays: 10000,
        failed_relays: 100,
        last_active: Utc::now(),
        status: relayer::RelayerStatus::Active,
    };

    assert_eq!(relayer_node.status, relayer::RelayerStatus::Active);
    assert_eq!(relayer_node.success_rate, 0.99);
}

#[tokio::test]
async fn test_validator_service_integration() {
    // Test validation request creation
    let validation_request = validator::ValidationRequest::new(
        Uuid::new_v4(),
        ChainId::Ethereum,
        ChainId::Polygon,
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
        12345,
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        vec![1, 2, 3, 4, 5],
        vec![6, 7, 8, 9, 10],
        "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        vec!["0x2222222222222222222222222222222222222222222222222222222222222222".to_string()],
    );

    assert!(validation_request.validate().is_ok());
    assert_eq!(validation_request.source_block_number, 12345);
    assert!(!validation_request.merkle_proof.is_empty());

    // Test bridge validator
    let bridge_validator = validator::BridgeValidator {
        validator_id: "validator-001".to_string(),
        operator_address: "0x3333333333333333333333333333333333333333".to_string(),
        stake_amount: Decimal::new(10000000, 2), // $100,000
        reputation_score: 0.98,
        total_validations: 5000,
        successful_validations: 4900,
        failed_validations: 100,
        slashed_amount: Decimal::ZERO,
        rewards_earned: Decimal::new(50000, 2), // $500
        last_active: Utc::now(),
        status: validator::ValidatorStatus::Active,
        supported_chains: vec![ChainId::Ethereum, ChainId::Polygon, ChainId::BSC],
    };

    assert_eq!(bridge_validator.status, validator::ValidatorStatus::Active);
    assert_eq!(bridge_validator.reputation_score, 0.98);
}

#[tokio::test]
async fn test_bridge_service_integration() {
    // Test bridge transaction creation
    let bridge_transaction = BridgeTransaction {
        id: Uuid::new_v4(),
        user_id: "user123".to_string(),
        source_chain: ChainId::Ethereum,
        destination_chain: ChainId::Polygon,
        asset_symbol: "USDC".to_string(),
        amount: Decimal::new(100000000, 6), // 100 USDC
        fee: Decimal::new(300000, 6),       // 0.3 USDC
        source_address: "0x1234567890123456789012345678901234567890".to_string(),
        destination_address: "0x0987654321098765432109876543210987654321".to_string(),
        source_tx_hash: Some("0xabc123".to_string()),
        destination_tx_hash: Some("0xdef456".to_string()),
        status: BridgeStatus::Completed,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        completed_at: Some(Utc::now()),
        metadata: serde_json::json!({"test": "data"}),
    };

    assert_eq!(bridge_transaction.status, BridgeStatus::Completed);
    assert!(bridge_transaction.completed_at.is_some());
    assert_eq!(bridge_transaction.amount, Decimal::new(100000000, 6));

    // Test bridge service configuration
    let config = BridgeServiceConfig::default();
    assert!(config.global_settings.enable_security_checks);
    assert!(config.global_settings.enable_compliance_checks);
    assert_eq!(
        config.global_settings.max_transaction_amount,
        Decimal::new(100000000, 2)
    );
}

#[tokio::test]
async fn test_cross_service_integration() {
    // Test a complete bridge flow integration
    let user_id = "integration_test_user".to_string();
    let amount = Decimal::new(50000000, 6); // 50 USDC

    // 1. Create transfer request
    let transfer_request = TransferRequest::new(
        user_id.clone(),
        ChainId::Ethereum,
        ChainId::Polygon,
        "USDC".to_string(),
        amount,
        "0x1234567890123456789012345678901234567890".to_string(),
        "0x0987654321098765432109876543210987654321".to_string(),
    );

    // 2. Create security check
    let security_request = security::SecurityCheckRequest {
        transaction_id: transfer_request.id,
        user_id: user_id.clone(),
        source_chain: transfer_request.source_chain,
        destination_chain: transfer_request.destination_chain,
        amount: transfer_request.amount,
        asset_symbol: transfer_request.asset_symbol.clone(),
        source_address: transfer_request.source_address.clone(),
        destination_address: transfer_request.destination_address.clone(),
        timestamp: transfer_request.created_at,
        user_agent: None,
        ip_address: None,
        session_id: None,
    };

    // 3. Create relay request
    let relay_request = relayer::RelayRequest::new(
        transfer_request.source_chain,
        transfer_request.destination_chain,
        "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
        vec![1, 2, 3, 4, 5],
        transfer_request.destination_address.clone(),
        200000,
    );

    // Verify all components work together
    assert_eq!(transfer_request.user_id, security_request.user_id);
    assert_eq!(transfer_request.source_chain, relay_request.source_chain);
    assert_eq!(
        transfer_request.destination_chain,
        relay_request.destination_chain
    );
    assert_eq!(
        transfer_request.destination_address,
        relay_request.recipient_address
    );
}

#[tokio::test]
async fn test_error_handling_integration() {
    // Test various error scenarios
    let config = TransferConfig::default();

    // Test invalid transfer request (amount too small)
    let invalid_request = TransferRequest::new(
        "user123".to_string(),
        ChainId::Ethereum,
        ChainId::Polygon,
        "USDC".to_string(),
        Decimal::new(100, 6), // 0.0001 USDC (too small)
        "0x1234567890123456789012345678901234567890".to_string(),
        "0x0987654321098765432109876543210987654321".to_string(),
    );

    let validation_result = invalid_request.validate(&config);
    assert!(validation_result.is_err());

    // Test same chain transfer (should fail)
    let same_chain_request = TransferRequest::new(
        "user123".to_string(),
        ChainId::Ethereum,
        ChainId::Ethereum, // Same as source
        "USDC".to_string(),
        Decimal::new(100000000, 6),
        "0x1234567890123456789012345678901234567890".to_string(),
        "0x0987654321098765432109876543210987654321".to_string(),
    );

    let validation_result = same_chain_request.validate(&config);
    assert!(validation_result.is_err());
}

#[tokio::test]
async fn test_performance_metrics() {
    // Test performance-related functionality
    let start_time = std::time::Instant::now();

    // Create multiple requests to test performance
    let mut requests = Vec::new();
    for i in 0..1000 {
        let request = TransferRequest::new(
            format!("user{}", i),
            ChainId::Ethereum,
            ChainId::Polygon,
            "USDC".to_string(),
            Decimal::new(100000000, 6),
            "0x1234567890123456789012345678901234567890".to_string(),
            "0x0987654321098765432109876543210987654321".to_string(),
        );
        requests.push(request);
    }

    let elapsed = start_time.elapsed();
    println!("Created 1000 transfer requests in {:?}", elapsed);

    // Should be able to create 1000 requests quickly
    assert!(elapsed.as_millis() < 1000); // Less than 1 second
    assert_eq!(requests.len(), 1000);
}
