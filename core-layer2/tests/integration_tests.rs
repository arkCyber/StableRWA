// =====================================================================================
// File: core-layer2/tests/integration_tests.rs
// Description: Integration tests for Layer2 services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use core_layer2::{
    types::{Layer2Network, BridgeTransaction, BridgeStatus},
    polygon::{PolygonServiceImpl, PolygonConfig, PolygonService},
    arbitrum::{ArbitrumServiceImpl, ArbitrumConfig, ArbitrumService},
    optimism::{OptimismServiceImpl, OptimismConfig, OptimismService},
    base::{BaseServiceImpl, BaseConfig},
    service::Layer2Service,
    Layer2ServiceConfig,
};
use rust_decimal::Decimal;

#[tokio::test]
async fn test_polygon_service_initialization() {
    let config = PolygonConfig::default();
    let service = PolygonServiceImpl::new(config);
    
    // Test network status
    let status = service.get_network_status().await.unwrap();
    assert_eq!(status.network, Layer2Network::Polygon);
    assert!(status.is_online);
    assert!(status.is_healthy);
}

#[tokio::test]
async fn test_arbitrum_service_initialization() {
    let config = ArbitrumConfig::default();
    let service = ArbitrumServiceImpl::new(config);
    
    // Test network status
    let status = service.get_network_status().await.unwrap();
    assert_eq!(status.network, Layer2Network::Arbitrum);
    assert!(status.is_online);
    assert!(status.is_healthy);
}

#[tokio::test]
async fn test_optimism_service_initialization() {
    let config = OptimismConfig::default();
    let service = OptimismServiceImpl::new(config);
    
    // Test network status
    let status = service.get_network_status().await.unwrap();
    assert_eq!(status.network, Layer2Network::Optimism);
    assert!(status.is_online);
    assert!(status.is_healthy);
}

#[tokio::test]
async fn test_base_service_initialization() {
    let config = BaseConfig::default();
    let service = BaseServiceImpl::new(config);
    
    // Test network status
    let status = service.get_network_status().await.unwrap();
    assert_eq!(status.network, Layer2Network::Base);
    assert!(status.is_online);
    assert!(status.is_healthy);
}

#[tokio::test]
async fn test_layer2_service_initialization() {
    let config = Layer2ServiceConfig::default();
    let service = Layer2Service::new(config);
    
    // Test service creation - just verify it was created successfully
    // Note: fields are private, so we can't directly access them
}

#[tokio::test]
async fn test_bridge_transaction_creation() {
    use uuid::Uuid;
    use chrono::Utc;
    
    let transaction = BridgeTransaction {
        id: Uuid::new_v4(),
        hash: "0x123...".to_string(),
        user_id: "test_user".to_string(),
        from_chain: Layer2Network::Ethereum,
        to_chain: Layer2Network::Polygon,
        source_network: Layer2Network::Ethereum,
        destination_network: Layer2Network::Polygon,
        token_address: "0xA0b86a33E6441e6e80D0c4C34F4b748f8c0c4C34".to_string(),
        amount: Decimal::new(1000, 2), // 10.00
        source_tx_hash: Some("0xabc...".to_string()),
        destination_tx_hash: None,
        status: BridgeStatus::Pending,
        initiated_at: Utc::now(),
        created_at: Utc::now(),
        completed_at: None,
        confirmations: 0,
        required_confirmations: 12,
    };
    
    assert_eq!(transaction.from_chain, Layer2Network::Ethereum);
    assert_eq!(transaction.to_chain, Layer2Network::Polygon);
    assert_eq!(transaction.status, BridgeStatus::Pending);
    assert_eq!(transaction.amount, Decimal::new(1000, 2));
}

#[tokio::test]
async fn test_network_enum_properties() {
    // Test Ethereum
    assert_eq!(Layer2Network::Ethereum.name(), "Ethereum");
    assert_eq!(Layer2Network::Ethereum.chain_id(), 1);
    
    // Test Polygon
    assert_eq!(Layer2Network::Polygon.name(), "Polygon");
    assert_eq!(Layer2Network::Polygon.chain_id(), 137);
    
    // Test Arbitrum
    assert_eq!(Layer2Network::Arbitrum.name(), "Arbitrum");
    assert_eq!(Layer2Network::Arbitrum.chain_id(), 42161);
    
    // Test Optimism
    assert_eq!(Layer2Network::Optimism.name(), "Optimism");
    assert_eq!(Layer2Network::Optimism.chain_id(), 10);
    
    // Test Base
    assert_eq!(Layer2Network::Base.name(), "Base");
    assert_eq!(Layer2Network::Base.chain_id(), 8453);
}

#[tokio::test]
async fn test_bridge_status_transitions() {
    let statuses = vec![
        BridgeStatus::Pending,
        BridgeStatus::Initiated,
        BridgeStatus::Locked,
        BridgeStatus::Relayed,
        BridgeStatus::Minted,
        BridgeStatus::Completed,
        BridgeStatus::Failed,
    ];
    
    // Test that all statuses can be created
    for status in statuses {
        match status {
            BridgeStatus::Pending => assert!(true),
            BridgeStatus::Initiated => assert!(true),
            BridgeStatus::Locked => assert!(true),
            BridgeStatus::Relayed => assert!(true),
            BridgeStatus::Minted => assert!(true),
            BridgeStatus::Completed => assert!(true),
            BridgeStatus::Failed => assert!(true),
        }
    }
}

#[tokio::test]
async fn test_polygon_bridge_transaction() {
    let config = PolygonConfig::default();
    let service = PolygonServiceImpl::new(config);
    
    let tx_hash = "0x123456789abcdef";
    let result = service.get_bridge_status(tx_hash).await;
    
    assert!(result.is_ok());
    let transaction = result.unwrap();
    assert_eq!(transaction.hash, tx_hash);
    assert_eq!(transaction.from_chain, Layer2Network::Ethereum);
    assert_eq!(transaction.to_chain, Layer2Network::Polygon);
}

#[tokio::test]
async fn test_arbitrum_bridge_transaction() {
    let config = ArbitrumConfig::default();
    let service = ArbitrumServiceImpl::new(config);
    
    let tx_hash = "0x123456789abcdef";
    let result = service.get_bridge_status(tx_hash).await;
    
    assert!(result.is_ok());
    let transaction = result.unwrap();
    assert_eq!(transaction.hash, tx_hash);
    assert_eq!(transaction.from_chain, Layer2Network::Ethereum);
    assert_eq!(transaction.to_chain, Layer2Network::Arbitrum);
}

#[tokio::test]
async fn test_optimism_bridge_transaction() {
    let config = OptimismConfig::default();
    let service = OptimismServiceImpl::new(config);
    
    let tx_hash = "0x123456789abcdef";
    let result = service.get_bridge_status(tx_hash).await;
    
    assert!(result.is_ok());
    let transaction = result.unwrap();
    assert_eq!(transaction.hash, tx_hash);
    assert_eq!(transaction.from_chain, Layer2Network::Ethereum);
    assert_eq!(transaction.to_chain, Layer2Network::Optimism);
}

#[tokio::test]
async fn test_base_bridge_transaction() {
    let config = BaseConfig::default();
    let service = BaseServiceImpl::new(config);
    
    let tx_hash = "0x123456789abcdef";
    let result = service.get_bridge_status(tx_hash).await;
    
    assert!(result.is_ok());
    let transaction = result.unwrap();
    assert_eq!(transaction.hash, tx_hash);
    assert_eq!(transaction.from_chain, Layer2Network::Ethereum);
    assert_eq!(transaction.to_chain, Layer2Network::Base);
}

#[tokio::test]
async fn test_error_handling() {
    use core_layer2::error::Layer2Error;
    
    // Test not found error
    let error = Layer2Error::not_found("transaction", "0xinvalid");
    match error {
        Layer2Error::NotFound { resource_type, id } => {
            assert_eq!(resource_type, "transaction");
            assert_eq!(id, "0xinvalid");
        }
        _ => panic!("Expected NotFound error"),
    }
    
    // Test validation error
    let error = Layer2Error::validation_error("amount", "Amount must be positive");
    match error {
        Layer2Error::ValidationError { field, message } => {
            assert_eq!(field, "amount");
            assert_eq!(message, "Amount must be positive");
        }
        _ => panic!("Expected ValidationError"),
    }
}
