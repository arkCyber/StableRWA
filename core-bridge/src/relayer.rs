// =====================================================================================
// File: core-bridge/src/relayer.rs
// Description: Bridge relayer service for cross-chain message and transaction relay
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{BridgeError, BridgeResult},
    types::{BridgeStatus, ChainId},
};

/// Relayer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerConfig {
    /// Relayer node ID
    pub node_id: String,
    /// Supported chains for relaying
    pub supported_chains: Vec<ChainId>,
    /// Maximum concurrent relays
    pub max_concurrent_relays: u32,
    /// Relay timeout in seconds
    pub relay_timeout_seconds: u64,
    /// Gas price multiplier for priority
    pub gas_price_multiplier: Decimal,
    /// Minimum relay fee (in USD)
    pub min_relay_fee: Decimal,
    /// Maximum relay fee (in USD)
    pub max_relay_fee: Decimal,
    /// Enable automatic retry
    pub auto_retry: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Health check interval in seconds
    pub health_check_interval_seconds: u64,
}

impl Default for RelayerConfig {
    fn default() -> Self {
        Self {
            node_id: format!("relayer-{}", Uuid::new_v4()),
            supported_chains: vec![
                ChainId::Ethereum,
                ChainId::Polygon,
                ChainId::BSC,
                ChainId::Bitcoin,
            ],
            max_concurrent_relays: 100,
            relay_timeout_seconds: 1800,                // 30 minutes
            gas_price_multiplier: Decimal::new(110, 2), // 1.10x
            min_relay_fee: Decimal::new(100, 2),        // $1.00
            max_relay_fee: Decimal::new(10000, 2),      // $100.00
            auto_retry: true,
            max_retry_attempts: 3,
            retry_delay_seconds: 60,
            health_check_interval_seconds: 30,
        }
    }
}

/// Relay request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayRequest {
    pub id: Uuid,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub source_tx_hash: String,
    pub message_data: Vec<u8>,
    pub recipient_address: String,
    pub gas_limit: u64,
    pub gas_price: Option<Decimal>,
    pub priority: RelayPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub callback_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl RelayRequest {
    /// Create a new relay request
    pub fn new(
        source_chain: ChainId,
        destination_chain: ChainId,
        source_tx_hash: String,
        message_data: Vec<u8>,
        recipient_address: String,
        gas_limit: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_chain,
            destination_chain,
            source_tx_hash,
            message_data,
            recipient_address,
            gas_limit,
            gas_price: None,
            priority: RelayPriority::Normal,
            deadline: None,
            callback_url: None,
            created_at: Utc::now(),
        }
    }

    /// Set relay priority
    pub fn with_priority(mut self, priority: RelayPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Set callback URL
    pub fn with_callback<S: Into<String>>(mut self, callback_url: S) -> Self {
        self.callback_url = Some(callback_url.into());
        self
    }

    /// Validate the relay request
    pub fn validate(&self) -> BridgeResult<()> {
        if self.source_chain == self.destination_chain {
            return Err(BridgeError::validation_error(
                "chains",
                "Source and destination chains cannot be the same",
            ));
        }

        if self.source_tx_hash.is_empty() {
            return Err(BridgeError::validation_error(
                "source_tx_hash",
                "Source transaction hash cannot be empty",
            ));
        }

        if self.recipient_address.is_empty() {
            return Err(BridgeError::validation_error(
                "recipient_address",
                "Recipient address cannot be empty",
            ));
        }

        if self.gas_limit == 0 {
            return Err(BridgeError::validation_error(
                "gas_limit",
                "Gas limit must be greater than zero",
            ));
        }

        if let Some(deadline) = self.deadline {
            if deadline <= Utc::now() {
                return Err(BridgeError::validation_error(
                    "deadline",
                    "Deadline must be in the future",
                ));
            }
        }

        Ok(())
    }
}

/// Relay priority enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl RelayPriority {
    /// Get priority score for ordering
    pub fn score(&self) -> u8 {
        match self {
            RelayPriority::Low => 1,
            RelayPriority::Normal => 2,
            RelayPriority::High => 3,
            RelayPriority::Critical => 4,
        }
    }
}

/// Relay result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayResult {
    pub request_id: Uuid,
    pub status: BridgeStatus,
    pub destination_tx_hash: Option<String>,
    pub gas_used: Option<u64>,
    pub gas_price: Option<Decimal>,
    pub relay_fee: Decimal,
    pub execution_time_ms: u64,
    pub confirmations: u32,
    pub error_message: Option<String>,
    pub relayed_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

/// Relayer node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerNode {
    pub node_id: String,
    pub operator_address: String,
    pub supported_chains: Vec<ChainId>,
    pub stake_amount: Decimal,
    pub reputation_score: f64,
    pub success_rate: f64,
    pub average_relay_time: Duration,
    pub total_relays: u64,
    pub failed_relays: u64,
    pub last_active: DateTime<Utc>,
    pub status: RelayerStatus,
}

/// Relayer status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayerStatus {
    Active,
    Inactive,
    Suspended,
    Slashed,
}

/// Relayer service trait
#[async_trait]
pub trait RelayerService: Send + Sync {
    /// Submit a relay request
    async fn submit_relay(&self, request: RelayRequest) -> BridgeResult<RelayResult>;

    /// Get relay status
    async fn get_relay_status(&self, request_id: Uuid) -> BridgeResult<Option<RelayResult>>;

    /// Cancel a pending relay
    async fn cancel_relay(&self, request_id: Uuid) -> BridgeResult<()>;

    /// Get relay history
    async fn get_relay_history(
        &self,
        chain_id: Option<ChainId>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> BridgeResult<Vec<RelayResult>>;

    /// Estimate relay fee
    async fn estimate_relay_fee(&self, request: &RelayRequest) -> BridgeResult<Decimal>;

    /// Get available relayers
    async fn get_available_relayers(&self, chain_id: ChainId) -> BridgeResult<Vec<RelayerNode>>;

    /// Register as a relayer
    async fn register_relayer(&self, node: RelayerNode) -> BridgeResult<()>;

    /// Update relayer status
    async fn update_relayer_status(&self, node_id: &str, status: RelayerStatus)
        -> BridgeResult<()>;

    /// Health check
    async fn health_check(&self) -> BridgeResult<RelayerHealthStatus>;
}

/// Relayer health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerHealthStatus {
    pub status: String,
    pub active_relayers: u64,
    pub total_relayers: u64,
    pub pending_relays: u64,
    pub completed_relays_24h: u64,
    pub failed_relays_24h: u64,
    pub average_relay_time_minutes: f64,
    pub network_congestion: HashMap<ChainId, f64>,
    pub last_check: DateTime<Utc>,
}

/// Relayer network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerNetworkStats {
    pub total_relayers: u64,
    pub active_relayers: u64,
    pub total_stake: Decimal,
    pub average_reputation: f64,
    pub total_relays_processed: u64,
    pub success_rate: f64,
    pub average_relay_time: Duration,
    pub chain_coverage: HashMap<ChainId, u64>,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_request_creation() {
        let request = RelayRequest::new(
            ChainId::Ethereum,
            ChainId::Polygon,
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            vec![1, 2, 3, 4, 5],
            "0x0987654321098765432109876543210987654321".to_string(),
            200000,
        )
        .with_priority(RelayPriority::High)
        .with_callback("https://api.example.com/callback");

        assert_eq!(request.source_chain, ChainId::Ethereum);
        assert_eq!(request.destination_chain, ChainId::Polygon);
        assert_eq!(request.priority, RelayPriority::High);
        assert!(request.callback_url.is_some());
    }

    #[test]
    fn test_relay_request_validation() {
        let valid_request = RelayRequest::new(
            ChainId::Ethereum,
            ChainId::Polygon,
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            vec![1, 2, 3, 4, 5],
            "0x0987654321098765432109876543210987654321".to_string(),
            200000,
        );

        assert!(valid_request.validate().is_ok());

        // Test same chain error
        let invalid_request = RelayRequest::new(
            ChainId::Ethereum,
            ChainId::Ethereum, // Same as source
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            vec![1, 2, 3, 4, 5],
            "0x0987654321098765432109876543210987654321".to_string(),
            200000,
        );

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_relay_priority_ordering() {
        assert!(RelayPriority::Critical.score() > RelayPriority::High.score());
        assert!(RelayPriority::High.score() > RelayPriority::Normal.score());
        assert!(RelayPriority::Normal.score() > RelayPriority::Low.score());
    }

    #[test]
    fn test_relayer_config_default() {
        let config = RelayerConfig::default();
        assert!(!config.node_id.is_empty());
        assert!(!config.supported_chains.is_empty());
        assert_eq!(config.max_concurrent_relays, 100);
        assert!(config.auto_retry);
    }
}
