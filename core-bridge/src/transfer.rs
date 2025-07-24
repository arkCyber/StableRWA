// =====================================================================================
// File: core-bridge/src/transfer.rs
// Description: Cross-chain asset transfer service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{BridgeError, BridgeResult},
    types::{AssetTransfer, BridgeStatus, BridgeTransaction, ChainId},
};

/// Transfer service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfig {
    /// Minimum transfer amount (in USD)
    pub min_transfer_amount: Decimal,
    /// Maximum transfer amount (in USD)
    pub max_transfer_amount: Decimal,
    /// Transfer fee percentage
    pub transfer_fee_percentage: Decimal,
    /// Maximum slippage tolerance
    pub max_slippage: Decimal,
    /// Transfer timeout in seconds
    pub timeout_seconds: u64,
    /// Enable automatic retry
    pub auto_retry: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
    /// Enable transfer batching
    pub enable_batching: bool,
    /// Batch size for transfers
    pub batch_size: usize,
    /// Batch timeout in seconds
    pub batch_timeout_seconds: u64,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            min_transfer_amount: Decimal::new(1000, 2),      // $10.00
            max_transfer_amount: Decimal::new(100000000, 2), // $1,000,000.00
            transfer_fee_percentage: Decimal::new(30, 4),    // 0.30%
            max_slippage: Decimal::new(500, 4),              // 5.00%
            timeout_seconds: 3600,                           // 1 hour
            auto_retry: true,
            max_retry_attempts: 3,
            retry_delay_seconds: 60,
            enable_batching: true,
            batch_size: 10,
            batch_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Transfer request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub id: Uuid,
    pub user_id: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub asset_symbol: String,
    pub amount: Decimal,
    pub source_address: String,
    pub destination_address: String,
    pub memo: Option<String>,
    pub priority: TransferPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub slippage_tolerance: Option<Decimal>,
    pub gas_price: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

impl TransferRequest {
    /// Create a new transfer request
    pub fn new(
        user_id: String,
        source_chain: ChainId,
        destination_chain: ChainId,
        asset_symbol: String,
        amount: Decimal,
        source_address: String,
        destination_address: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            source_chain,
            destination_chain,
            asset_symbol,
            amount,
            source_address,
            destination_address,
            memo: None,
            priority: TransferPriority::Normal,
            deadline: None,
            slippage_tolerance: None,
            gas_price: None,
            created_at: Utc::now(),
        }
    }

    /// Set transfer priority
    pub fn with_priority(mut self, priority: TransferPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Set memo
    pub fn with_memo<S: Into<String>>(mut self, memo: S) -> Self {
        self.memo = Some(memo.into());
        self
    }

    /// Validate the transfer request
    pub fn validate(&self, config: &TransferConfig) -> BridgeResult<()> {
        // Check amount limits
        if self.amount < config.min_transfer_amount {
            return Err(BridgeError::validation_error(
                "amount",
                format!(
                    "Amount {} is below minimum {}",
                    self.amount, config.min_transfer_amount
                ),
            ));
        }

        if self.amount > config.max_transfer_amount {
            return Err(BridgeError::validation_error(
                "amount",
                format!(
                    "Amount {} exceeds maximum {}",
                    self.amount, config.max_transfer_amount
                ),
            ));
        }

        // Check deadline
        if let Some(deadline) = self.deadline {
            if deadline <= Utc::now() {
                return Err(BridgeError::validation_error(
                    "deadline",
                    "Deadline must be in the future",
                ));
            }
        }

        // Check addresses
        if self.source_address.is_empty() {
            return Err(BridgeError::validation_error(
                "source_address",
                "Source address cannot be empty",
            ));
        }

        if self.destination_address.is_empty() {
            return Err(BridgeError::validation_error(
                "destination_address",
                "Destination address cannot be empty",
            ));
        }

        // Check same chain transfer
        if self.source_chain == self.destination_chain {
            return Err(BridgeError::validation_error(
                "chains",
                "Source and destination chains cannot be the same",
            ));
        }

        Ok(())
    }
}

/// Transfer priority enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl TransferPriority {
    /// Get priority score for ordering
    pub fn score(&self) -> u8 {
        match self {
            TransferPriority::Low => 1,
            TransferPriority::Normal => 2,
            TransferPriority::High => 3,
            TransferPriority::Critical => 4,
        }
    }
}

/// Transfer result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResult {
    pub request_id: Uuid,
    pub status: BridgeStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub actual_amount: Decimal,
    pub fee_amount: Decimal,
    pub gas_used: Option<u64>,
    pub gas_price: Option<Decimal>,
    pub execution_time_ms: u64,
    pub confirmations: u32,
    pub error_message: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Transfer service trait
#[async_trait]
pub trait TransferService: Send + Sync {
    /// Submit a transfer request
    async fn submit_transfer(&self, request: TransferRequest) -> BridgeResult<TransferResult>;

    /// Get transfer status
    async fn get_transfer_status(&self, request_id: Uuid) -> BridgeResult<Option<TransferResult>>;

    /// Cancel a pending transfer
    async fn cancel_transfer(&self, request_id: Uuid) -> BridgeResult<()>;

    /// Get transfer history for a user
    async fn get_transfer_history(
        &self,
        user_id: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> BridgeResult<Vec<TransferResult>>;

    /// Estimate transfer fee
    async fn estimate_fee(&self, request: &TransferRequest) -> BridgeResult<Decimal>;

    /// Get supported transfer routes
    async fn get_supported_routes(&self) -> BridgeResult<Vec<TransferRoute>>;

    /// Health check
    async fn health_check(&self) -> BridgeResult<TransferHealthStatus>;
}

/// Transfer route information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRoute {
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub supported_assets: Vec<String>,
    pub min_amount: Decimal,
    pub max_amount: Decimal,
    pub estimated_time_minutes: u32,
    pub fee_percentage: Decimal,
    pub enabled: bool,
}

/// Transfer health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHealthStatus {
    pub status: String,
    pub active_transfers: u64,
    pub pending_transfers: u64,
    pub failed_transfers_24h: u64,
    pub average_completion_time_minutes: f64,
    pub supported_chains: Vec<ChainId>,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_request_creation() {
        let request = TransferRequest::new(
            "user123".to_string(),
            ChainId::Ethereum,
            ChainId::Polygon,
            "USDC".to_string(),
            Decimal::new(100000, 6), // 100 USDC
            "0x1234567890123456789012345678901234567890".to_string(),
            "0x0987654321098765432109876543210987654321".to_string(),
        )
        .with_priority(TransferPriority::High)
        .with_memo("Test transfer");

        assert_eq!(request.user_id, "user123");
        assert_eq!(request.source_chain, ChainId::Ethereum);
        assert_eq!(request.destination_chain, ChainId::Polygon);
        assert_eq!(request.priority, TransferPriority::High);
        assert_eq!(request.memo, Some("Test transfer".to_string()));
    }

    #[test]
    fn test_transfer_request_validation() {
        let config = TransferConfig::default();

        let valid_request = TransferRequest::new(
            "user123".to_string(),
            ChainId::Ethereum,
            ChainId::Polygon,
            "USDC".to_string(),
            Decimal::new(100000, 6), // 100 USDC
            "0x1234567890123456789012345678901234567890".to_string(),
            "0x0987654321098765432109876543210987654321".to_string(),
        );

        assert!(valid_request.validate(&config).is_ok());

        // Test amount too small
        let invalid_request = TransferRequest::new(
            "user123".to_string(),
            ChainId::Ethereum,
            ChainId::Polygon,
            "USDC".to_string(),
            Decimal::new(100, 6), // 0.1 USDC (too small)
            "0x1234567890123456789012345678901234567890".to_string(),
            "0x0987654321098765432109876543210987654321".to_string(),
        );

        assert!(invalid_request.validate(&config).is_err());
    }

    #[test]
    fn test_transfer_priority_ordering() {
        assert!(TransferPriority::Critical.score() > TransferPriority::High.score());
        assert!(TransferPriority::High.score() > TransferPriority::Normal.score());
        assert!(TransferPriority::Normal.score() > TransferPriority::Low.score());
    }
}
