// =====================================================================================
// File: core-trading/src/settlement.rs
// Description: Trade settlement service for RWA trading system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    error::{TradingError, TradingResult},
    types::TradingPair,
    matching::Trade,
};

/// Settlement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    /// Settlement cycle duration in seconds
    pub settlement_cycle_seconds: u64,
    /// Maximum settlement attempts
    pub max_settlement_attempts: u32,
    /// Settlement timeout in seconds
    pub settlement_timeout_seconds: u64,
    /// Enable atomic settlement
    pub atomic_settlement: bool,
    /// Settlement fee percentage
    pub settlement_fee_percentage: Decimal,
    /// Minimum settlement amount
    pub min_settlement_amount: Decimal,
    /// Enable netting
    pub enable_netting: bool,
    /// Netting threshold
    pub netting_threshold: Decimal,
    /// Enable DVP (Delivery vs Payment)
    pub enable_dvp: bool,
}

impl Default for SettlementConfig {
    fn default() -> Self {
        Self {
            settlement_cycle_seconds: 300, // 5 minutes
            max_settlement_attempts: 3,
            settlement_timeout_seconds: 1800, // 30 minutes
            atomic_settlement: true,
            settlement_fee_percentage: Decimal::new(5, 4), // 0.05%
            min_settlement_amount: Decimal::new(100, 2), // $1.00
            enable_netting: true,
            netting_threshold: Decimal::new(10000, 2), // $100.00
            enable_dvp: true,
        }
    }
}

/// Settlement instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementInstruction {
    pub id: Uuid,
    pub trade_id: Uuid,
    pub trading_pair: TradingPair,
    pub buyer_id: String,
    pub seller_id: String,
    pub asset_to_deliver: String,
    pub asset_to_receive: String,
    pub delivery_quantity: Decimal,
    pub payment_amount: Decimal,
    pub settlement_date: DateTime<Utc>,
    pub status: SettlementStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attempts: u32,
    pub error_message: Option<String>,
}

impl SettlementInstruction {
    /// Create a new settlement instruction from a trade
    pub fn from_trade(trade: &Trade, settlement_date: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            trade_id: trade.id,
            trading_pair: trade.trading_pair.clone(),
            buyer_id: trade.buyer_user_id.clone(),
            seller_id: trade.seller_user_id.clone(),
            asset_to_deliver: trade.trading_pair.base.clone(),
            asset_to_receive: trade.trading_pair.quote.clone(),
            delivery_quantity: trade.quantity,
            payment_amount: trade.price * trade.quantity,
            settlement_date,
            status: SettlementStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            attempts: 0,
            error_message: None,
        }
    }

    /// Check if settlement instruction has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.settlement_date + Duration::hours(24)
    }

    /// Update settlement status
    pub fn update_status(&mut self, status: SettlementStatus, error_message: Option<String>) {
        self.status = status;
        self.error_message = error_message;
        self.updated_at = Utc::now();
    }

    /// Increment attempt counter
    pub fn increment_attempts(&mut self) {
        self.attempts += 1;
        self.updated_at = Utc::now();
    }
}

/// Settlement status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementStatus {
    Pending,
    InProgress,
    Settled,
    Failed,
    Cancelled,
    Expired,
}

/// Settlement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementResult {
    pub instruction_id: Uuid,
    pub trade_id: Uuid,
    pub status: SettlementStatus,
    pub settlement_fee: Decimal,
    pub delivery_tx_hash: Option<String>,
    pub payment_tx_hash: Option<String>,
    pub settled_at: Option<DateTime<Utc>>,
    pub settlement_time_ms: u64,
    pub error_message: Option<String>,
}

/// Settlement batch for processing multiple instructions together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementBatch {
    pub id: Uuid,
    pub instructions: Vec<SettlementInstruction>,
    pub total_value: Decimal,
    pub batch_fee: Decimal,
    pub status: SettlementStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl SettlementBatch {
    /// Create a new settlement batch
    pub fn new(instructions: Vec<SettlementInstruction>) -> Self {
        let total_value = instructions.iter()
            .map(|i| i.payment_amount)
            .sum();

        Self {
            id: Uuid::new_v4(),
            instructions,
            total_value,
            batch_fee: Decimal::ZERO,
            status: SettlementStatus::Pending,
            created_at: Utc::now(),
            processed_at: None,
        }
    }

    /// Calculate batch processing fee
    pub fn calculate_batch_fee(&mut self, fee_percentage: Decimal) {
        self.batch_fee = self.total_value * fee_percentage;
    }
}

/// Netting position for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NettingPosition {
    pub user_id: String,
    pub asset: String,
    pub gross_buy: Decimal,
    pub gross_sell: Decimal,
    pub net_position: Decimal,
    pub settlement_amount: Decimal,
}

impl NettingPosition {
    /// Create a new netting position
    pub fn new(user_id: String, asset: String) -> Self {
        Self {
            user_id,
            asset,
            gross_buy: Decimal::ZERO,
            gross_sell: Decimal::ZERO,
            net_position: Decimal::ZERO,
            settlement_amount: Decimal::ZERO,
        }
    }

    /// Add a buy transaction
    pub fn add_buy(&mut self, amount: Decimal) {
        self.gross_buy += amount;
        self.calculate_net_position();
    }

    /// Add a sell transaction
    pub fn add_sell(&mut self, amount: Decimal) {
        self.gross_sell += amount;
        self.calculate_net_position();
    }

    /// Calculate net position
    fn calculate_net_position(&mut self) {
        self.net_position = self.gross_buy - self.gross_sell;
        self.settlement_amount = self.net_position.abs();
    }
}

/// Settlement service trait
#[async_trait]
pub trait SettlementService: Send + Sync {
    /// Create settlement instruction from trade
    async fn create_settlement_instruction(&self, trade: Trade) -> TradingResult<SettlementInstruction>;
    
    /// Process settlement instruction
    async fn process_settlement(&self, instruction: SettlementInstruction) -> TradingResult<SettlementResult>;
    
    /// Process settlement batch
    async fn process_batch(&self, batch: SettlementBatch) -> TradingResult<Vec<SettlementResult>>;
    
    /// Get settlement status
    async fn get_settlement_status(&self, instruction_id: Uuid) -> TradingResult<Option<SettlementStatus>>;
    
    /// Get pending settlements for a user
    async fn get_pending_settlements(&self, user_id: &str) -> TradingResult<Vec<SettlementInstruction>>;
    
    /// Calculate netting positions
    async fn calculate_netting_positions(&self, user_id: &str, asset: &str) -> TradingResult<NettingPosition>;
    
    /// Cancel settlement instruction
    async fn cancel_settlement(&self, instruction_id: Uuid) -> TradingResult<()>;
    
    /// Get settlement history
    async fn get_settlement_history(
        &self,
        user_id: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> TradingResult<Vec<SettlementResult>>;
    
    /// Health check
    async fn health_check(&self) -> TradingResult<SettlementHealthStatus>;
}

/// Settlement health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementHealthStatus {
    pub status: String,
    pub pending_settlements: u64,
    pub failed_settlements_24h: u64,
    pub average_settlement_time_minutes: f64,
    pub settlement_success_rate: f64,
    pub total_settlement_value_24h: Decimal,
    pub last_settlement_cycle: DateTime<Utc>,
    pub next_settlement_cycle: DateTime<Utc>,
    pub last_check: DateTime<Utc>,
}

/// Settlement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementStats {
    pub total_settlements: u64,
    pub successful_settlements: u64,
    pub failed_settlements: u64,
    pub total_settlement_value: Decimal,
    pub total_settlement_fees: Decimal,
    pub average_settlement_time: Duration,
    pub settlement_success_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// DVP (Delivery vs Payment) instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DVPInstruction {
    pub id: Uuid,
    pub settlement_instruction_id: Uuid,
    pub delivery_leg: DeliveryLeg,
    pub payment_leg: PaymentLeg,
    pub status: DVPStatus,
    pub created_at: DateTime<Utc>,
}

/// Delivery leg of DVP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryLeg {
    pub asset: String,
    pub quantity: Decimal,
    pub from_account: String,
    pub to_account: String,
    pub delivery_status: DeliveryStatus,
    pub tx_hash: Option<String>,
}

/// Payment leg of DVP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentLeg {
    pub currency: String,
    pub amount: Decimal,
    pub from_account: String,
    pub to_account: String,
    pub payment_status: PaymentStatus,
    pub tx_hash: Option<String>,
}

/// DVP status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DVPStatus {
    Pending,
    DeliveryInitiated,
    PaymentInitiated,
    BothInitiated,
    Completed,
    Failed,
    Cancelled,
}

/// Delivery status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Initiated,
    Confirmed,
    Failed,
}

/// Payment status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Initiated,
    Confirmed,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OrderSide;

    #[test]
    fn test_settlement_instruction_creation() {
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let trade = Trade {
            id: Uuid::new_v4(),
            trading_pair: trading_pair.clone(),
            buyer_order_id: Uuid::new_v4(),
            seller_order_id: Uuid::new_v4(),
            buyer_user_id: "buyer123".to_string(),
            seller_user_id: "seller456".to_string(),
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
            buyer_fee: Decimal::new(50, 0),
            seller_fee: Decimal::new(50, 0),
            executed_at: Utc::now(),
        };

        let settlement_date = Utc::now() + Duration::hours(1);
        let instruction = SettlementInstruction::from_trade(&trade, settlement_date);

        assert_eq!(instruction.trade_id, trade.id);
        assert_eq!(instruction.buyer_id, "buyer123");
        assert_eq!(instruction.seller_id, "seller456");
        assert_eq!(instruction.delivery_quantity, Decimal::new(1, 0));
        assert_eq!(instruction.payment_amount, Decimal::new(50000, 0));
        assert_eq!(instruction.status, SettlementStatus::Pending);
    }

    #[test]
    fn test_netting_position() {
        let mut position = NettingPosition::new("user123".to_string(), "BTC".to_string());
        
        position.add_buy(Decimal::new(2, 0));
        position.add_sell(Decimal::new(1, 0));
        
        assert_eq!(position.gross_buy, Decimal::new(2, 0));
        assert_eq!(position.gross_sell, Decimal::new(1, 0));
        assert_eq!(position.net_position, Decimal::new(1, 0));
        assert_eq!(position.settlement_amount, Decimal::new(1, 0));
    }

    #[test]
    fn test_settlement_batch() {
        let instructions = vec![
            SettlementInstruction {
                id: Uuid::new_v4(),
                trade_id: Uuid::new_v4(),
                trading_pair: TradingPair::new("BTC".to_string(), "USD".to_string()),
                buyer_id: "buyer1".to_string(),
                seller_id: "seller1".to_string(),
                asset_to_deliver: "BTC".to_string(),
                asset_to_receive: "USD".to_string(),
                delivery_quantity: Decimal::new(1, 0),
                payment_amount: Decimal::new(50000, 0),
                settlement_date: Utc::now(),
                status: SettlementStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                attempts: 0,
                error_message: None,
            },
        ];

        let mut batch = SettlementBatch::new(instructions);
        batch.calculate_batch_fee(Decimal::new(5, 4)); // 0.05%

        assert_eq!(batch.total_value, Decimal::new(50000, 0));
        assert_eq!(batch.batch_fee, Decimal::new(2500, 2)); // $25.00
        assert_eq!(batch.status, SettlementStatus::Pending);
    }
}
