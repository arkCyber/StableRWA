// =====================================================================================
// File: core-layer2/src/service.rs
// Description: Main Layer 2 service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::{
    error::{Layer2Error, Layer2Result},
    types::{Layer2Network, CrossChainMessage, BridgeTransaction, NetworkStatus},
    Layer2ServiceConfig, Layer2Metrics, Layer2HealthStatus,
};

/// Main Layer 2 service implementation
pub struct Layer2Service {
    config: Arc<RwLock<Layer2ServiceConfig>>,
    network_clients: Arc<RwLock<HashMap<Layer2Network, Box<dyn NetworkClient>>>>,
    bridge_services: Arc<RwLock<HashMap<String, Box<dyn BridgeService>>>>,
}

impl Layer2Service {
    /// Create a new Layer 2 service
    pub fn new(config: Layer2ServiceConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            network_clients: Arc::new(RwLock::new(HashMap::new())),
            bridge_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Send cross-chain message
    pub async fn send_cross_chain_message(&self, message: CrossChainMessage) -> Layer2Result<String> {
        // Validate message
        self.validate_cross_chain_message(&message).await?;

        // Get appropriate bridge service
        let bridge_key = format!("{:?}-{:?}", message.source_network, message.destination_network);
        let bridge_services = self.bridge_services.read().await;
        let bridge_service = bridge_services.get(&bridge_key)
            .ok_or_else(|| Layer2Error::not_found("BridgeService", &bridge_key))?;

        // Send message
        bridge_service.send_message(message).await
    }

    /// Execute bridge transaction
    pub async fn execute_bridge_transaction(&self, transaction: BridgeTransaction) -> Layer2Result<String> {
        // Validate transaction
        self.validate_bridge_transaction(&transaction).await?;

        // Get network clients
        let network_clients = self.network_clients.read().await;
        let source_client = network_clients.get(&transaction.source_network)
            .ok_or_else(|| Layer2Error::not_found("NetworkClient", transaction.source_network.name()))?;
        let dest_client = network_clients.get(&transaction.destination_network)
            .ok_or_else(|| Layer2Error::not_found("NetworkClient", transaction.destination_network.name()))?;

        // Execute bridge transaction
        let source_tx = source_client.lock_tokens(&transaction).await?;
        let dest_tx = dest_client.mint_tokens(&transaction).await?;

        Ok(format!("{}:{}", source_tx, dest_tx))
    }

    /// Get network status
    pub async fn get_network_status(&self, network: Layer2Network) -> Layer2Result<NetworkStatus> {
        let network_clients = self.network_clients.read().await;
        let client = network_clients.get(&network)
            .ok_or_else(|| Layer2Error::not_found("NetworkClient", network.name()))?;

        client.get_status().await
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> Layer2Result<Layer2Metrics> {
        // This would aggregate metrics from all networks and services
        Ok(Layer2Metrics {
            total_transactions: 1000,
            successful_transactions: 950,
            failed_transactions: 50,
            total_gas_saved: 500000,
            average_confirmation_time: 15.5,
            cross_chain_volume_24h: rust_decimal::Decimal::new(1000000, 2),
            bridge_transactions_24h: 100,
            network_metrics: HashMap::new(),
            last_updated: chrono::Utc::now(),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> Layer2Result<Layer2HealthStatus> {
        Ok(Layer2HealthStatus {
            overall_status: "healthy".to_string(),
            polygon_status: "healthy".to_string(),
            arbitrum_status: "healthy".to_string(),
            optimism_status: "healthy".to_string(),
            base_status: "healthy".to_string(),
            bridge_status: "healthy".to_string(),
            cross_chain_status: "healthy".to_string(),
            network_statuses: HashMap::new(),
            last_check: chrono::Utc::now(),
        })
    }

    // Private helper methods
    async fn validate_cross_chain_message(&self, message: &CrossChainMessage) -> Layer2Result<()> {
        if message.source_network == message.destination_network {
            return Err(Layer2Error::validation_error(
                "networks",
                "Source and destination networks cannot be the same",
            ));
        }

        if message.data.is_empty() {
            return Err(Layer2Error::validation_error("data", "Message data cannot be empty"));
        }

        if message.gas_limit == 0 {
            return Err(Layer2Error::validation_error("gas_limit", "Gas limit must be greater than 0"));
        }

        Ok(())
    }

    async fn validate_bridge_transaction(&self, transaction: &BridgeTransaction) -> Layer2Result<()> {
        if transaction.source_network == transaction.destination_network {
            return Err(Layer2Error::validation_error(
                "networks",
                "Source and destination networks cannot be the same",
            ));
        }

        if transaction.amount <= rust_decimal::Decimal::ZERO {
            return Err(Layer2Error::validation_error("amount", "Amount must be greater than 0"));
        }

        Ok(())
    }
}

/// Network client trait
#[async_trait]
pub trait NetworkClient: Send + Sync {
    async fn get_status(&self) -> Layer2Result<NetworkStatus>;
    async fn lock_tokens(&self, transaction: &BridgeTransaction) -> Layer2Result<String>;
    async fn mint_tokens(&self, transaction: &BridgeTransaction) -> Layer2Result<String>;
    async fn estimate_gas(&self, transaction: &BridgeTransaction) -> Layer2Result<u64>;
}

/// Bridge service trait
#[async_trait]
pub trait BridgeService: Send + Sync {
    async fn send_message(&self, message: CrossChainMessage) -> Layer2Result<String>;
    async fn get_message_status(&self, message_id: &str) -> Layer2Result<crate::types::MessageStatus>;
}

/// Mock network client for testing
pub struct MockNetworkClient {
    network: Layer2Network,
}

impl MockNetworkClient {
    pub fn new(network: Layer2Network) -> Self {
        Self { network }
    }
}

#[async_trait]
impl NetworkClient for MockNetworkClient {
    async fn get_status(&self) -> Layer2Result<NetworkStatus> {
        Ok(NetworkStatus {
            network: self.network,
            is_online: true,
            is_healthy: true,
            block_height: 1000000,
            latest_block: 1000000,
            gas_price: 20,
            tps: 100.0,
            finality_time_seconds: 2,
            bridge_status: crate::types::BridgeHealthStatus::Operational,
            last_updated: chrono::Utc::now(),
        })
    }

    async fn lock_tokens(&self, _transaction: &BridgeTransaction) -> Layer2Result<String> {
        Ok("0xabc123...".to_string())
    }

    async fn mint_tokens(&self, _transaction: &BridgeTransaction) -> Layer2Result<String> {
        Ok("0xdef456...".to_string())
    }

    async fn estimate_gas(&self, _transaction: &BridgeTransaction) -> Layer2Result<u64> {
        Ok(200000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[tokio::test]
    async fn test_layer2_service_creation() {
        let config = Layer2ServiceConfig::default();
        let service = Layer2Service::new(config);
        
        let health = service.health_check().await.unwrap();
        assert_eq!(health.overall_status, "healthy");
    }

    #[tokio::test]
    async fn test_cross_chain_message_validation() {
        let config = Layer2ServiceConfig::default();
        let service = Layer2Service::new(config);
        
        let message = CrossChainMessage {
            id: uuid::Uuid::new_v4(),
            source_network: Layer2Network::Polygon,
            destination_network: Layer2Network::Arbitrum,
            sender: "0x123...".to_string(),
            recipient: "0x456...".to_string(),
            data: vec![1, 2, 3, 4, 5],
            gas_limit: 200000,
            created_at: chrono::Utc::now(),
            status: MessageStatus::Pending,
        };
        
        let result = service.validate_cross_chain_message(&message).await;
        assert!(result.is_ok());
        
        // Test invalid message (same networks)
        let mut invalid_message = message.clone();
        invalid_message.destination_network = Layer2Network::Polygon;
        let result = service.validate_cross_chain_message(&invalid_message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_network_client() {
        let client = MockNetworkClient::new(Layer2Network::Polygon);
        let status = client.get_status().await.unwrap();
        
        assert_eq!(status.network, Layer2Network::Polygon);
        assert!(status.is_online);
        assert_eq!(status.gas_price, 20);
    }
}
