// =====================================================================================
// File: core-bridge/src/liquidity.rs
// Description: Cross-chain liquidity management service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    error::{BridgeError, BridgeResult},
    types::{ChainId, LiquidityPool},
};

/// Liquidity service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityConfig {
    /// Minimum liquidity pool size (in USD)
    pub min_pool_size: Decimal,
    /// Maximum liquidity pool size (in USD)
    pub max_pool_size: Decimal,
    /// Liquidity provider fee percentage
    pub lp_fee_percentage: Decimal,
    /// Protocol fee percentage
    pub protocol_fee_percentage: Decimal,
    /// Minimum liquidity provision amount
    pub min_provision_amount: Decimal,
    /// Maximum liquidity provision amount
    pub max_provision_amount: Decimal,
    /// Rebalancing threshold percentage
    pub rebalancing_threshold: Decimal,
    /// Auto-rebalancing enabled
    pub auto_rebalancing: bool,
    /// Impermanent loss protection enabled
    pub impermanent_loss_protection: bool,
    /// Yield farming enabled
    pub yield_farming_enabled: bool,
}

impl Default for LiquidityConfig {
    fn default() -> Self {
        Self {
            min_pool_size: Decimal::new(10000000, 2), // $100,000
            max_pool_size: Decimal::new(10000000000, 2), // $100,000,000
            lp_fee_percentage: Decimal::new(30, 4), // 0.30%
            protocol_fee_percentage: Decimal::new(5, 4), // 0.05%
            min_provision_amount: Decimal::new(100000, 2), // $1,000
            max_provision_amount: Decimal::new(100000000, 2), // $1,000,000
            rebalancing_threshold: Decimal::new(500, 4), // 5.00%
            auto_rebalancing: true,
            impermanent_loss_protection: true,
            yield_farming_enabled: true,
        }
    }
}

/// Liquidity provision request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityRequest {
    pub id: Uuid,
    pub provider_id: String,
    pub pool_id: Uuid,
    pub chain_id: ChainId,
    pub asset_a_symbol: String,
    pub asset_b_symbol: String,
    pub amount_a: Decimal,
    pub amount_b: Decimal,
    pub min_amount_a: Option<Decimal>,
    pub min_amount_b: Option<Decimal>,
    pub deadline: Option<DateTime<Utc>>,
    pub slippage_tolerance: Decimal,
    pub created_at: DateTime<Utc>,
}

impl LiquidityRequest {
    /// Create a new liquidity provision request
    pub fn new(
        provider_id: String,
        pool_id: Uuid,
        chain_id: ChainId,
        asset_a_symbol: String,
        asset_b_symbol: String,
        amount_a: Decimal,
        amount_b: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider_id,
            pool_id,
            chain_id,
            asset_a_symbol,
            asset_b_symbol,
            amount_a,
            amount_b,
            min_amount_a: None,
            min_amount_b: None,
            deadline: None,
            slippage_tolerance: Decimal::new(100, 4), // 1.00% default
            created_at: Utc::now(),
        }
    }

    /// Set minimum amounts
    pub fn with_min_amounts(mut self, min_a: Decimal, min_b: Decimal) -> Self {
        self.min_amount_a = Some(min_a);
        self.min_amount_b = Some(min_b);
        self
    }

    /// Set deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Set slippage tolerance
    pub fn with_slippage(mut self, slippage: Decimal) -> Self {
        self.slippage_tolerance = slippage;
        self
    }
}

/// Liquidity withdrawal request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    pub id: Uuid,
    pub provider_id: String,
    pub pool_id: Uuid,
    pub lp_token_amount: Decimal,
    pub min_amount_a: Option<Decimal>,
    pub min_amount_b: Option<Decimal>,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Liquidity position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub id: Uuid,
    pub provider_id: String,
    pub pool_id: Uuid,
    pub lp_token_balance: Decimal,
    pub share_percentage: Decimal,
    pub asset_a_amount: Decimal,
    pub asset_b_amount: Decimal,
    pub initial_value_usd: Decimal,
    pub current_value_usd: Decimal,
    pub impermanent_loss: Decimal,
    pub fees_earned: Decimal,
    pub yield_earned: Decimal,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

/// Liquidity pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub pool_id: Uuid,
    pub total_value_locked: Decimal,
    pub volume_24h: Decimal,
    pub volume_7d: Decimal,
    pub fees_24h: Decimal,
    pub fees_7d: Decimal,
    pub apy: Decimal,
    pub liquidity_providers_count: u64,
    pub price_impact: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Liquidity service trait
#[async_trait]
pub trait LiquidityService: Send + Sync {
    /// Add liquidity to a pool
    async fn add_liquidity(&self, request: LiquidityRequest) -> BridgeResult<LiquidityPosition>;
    
    /// Remove liquidity from a pool
    async fn remove_liquidity(&self, request: WithdrawalRequest) -> BridgeResult<WithdrawalResult>;
    
    /// Get liquidity position
    async fn get_position(&self, provider_id: &str, pool_id: Uuid) -> BridgeResult<Option<LiquidityPosition>>;
    
    /// Get all positions for a provider
    async fn get_provider_positions(&self, provider_id: &str) -> BridgeResult<Vec<LiquidityPosition>>;
    
    /// Get pool information
    async fn get_pool(&self, pool_id: Uuid) -> BridgeResult<Option<LiquidityPool>>;
    
    /// Get all available pools
    async fn get_pools(&self, chain_id: Option<ChainId>) -> BridgeResult<Vec<LiquidityPool>>;
    
    /// Get pool statistics
    async fn get_pool_statistics(&self, pool_id: Uuid) -> BridgeResult<PoolStatistics>;
    
    /// Calculate optimal liquidity amounts
    async fn calculate_optimal_amounts(
        &self,
        pool_id: Uuid,
        desired_amount_a: Decimal,
        desired_amount_b: Decimal,
    ) -> BridgeResult<OptimalAmounts>;
    
    /// Estimate liquidity provision returns
    async fn estimate_returns(
        &self,
        pool_id: Uuid,
        amount_a: Decimal,
        amount_b: Decimal,
        duration_days: u32,
    ) -> BridgeResult<ReturnEstimate>;
    
    /// Rebalance pool if needed
    async fn rebalance_pool(&self, pool_id: Uuid) -> BridgeResult<RebalanceResult>;
    
    /// Health check
    async fn health_check(&self) -> BridgeResult<LiquidityHealthStatus>;
}

/// Withdrawal result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalResult {
    pub request_id: Uuid,
    pub amount_a_received: Decimal,
    pub amount_b_received: Decimal,
    pub lp_tokens_burned: Decimal,
    pub fees_collected: Decimal,
    pub tx_hash: Option<String>,
    pub completed_at: DateTime<Utc>,
}

/// Optimal liquidity amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalAmounts {
    pub amount_a: Decimal,
    pub amount_b: Decimal,
    pub lp_tokens_expected: Decimal,
    pub share_percentage: Decimal,
    pub price_impact: Decimal,
}

/// Return estimate for liquidity provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnEstimate {
    pub estimated_fees: Decimal,
    pub estimated_yield: Decimal,
    pub estimated_impermanent_loss: Decimal,
    pub net_return: Decimal,
    pub apy: Decimal,
    pub confidence_level: f64,
}

/// Pool rebalance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceResult {
    pub pool_id: Uuid,
    pub rebalanced: bool,
    pub amount_a_adjusted: Decimal,
    pub amount_b_adjusted: Decimal,
    pub new_ratio: Decimal,
    pub gas_used: Option<u64>,
    pub tx_hash: Option<String>,
    pub completed_at: DateTime<Utc>,
}

/// Liquidity health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityHealthStatus {
    pub status: String,
    pub total_pools: u64,
    pub active_pools: u64,
    pub total_tvl: Decimal,
    pub total_volume_24h: Decimal,
    pub average_apy: Decimal,
    pub rebalancing_needed: u64,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquidity_request_creation() {
        let request = LiquidityRequest::new(
            "provider123".to_string(),
            Uuid::new_v4(),
            ChainId::Ethereum,
            "USDC".to_string(),
            "ETH".to_string(),
            Decimal::new(100000, 6), // 100 USDC
            Decimal::new(1, 18), // 1 ETH
        )
        .with_min_amounts(Decimal::new(99000, 6), Decimal::new(99, 20))
        .with_slippage(Decimal::new(50, 4)); // 0.5%

        assert_eq!(request.provider_id, "provider123");
        assert_eq!(request.asset_a_symbol, "USDC");
        assert_eq!(request.asset_b_symbol, "ETH");
        assert_eq!(request.slippage_tolerance, Decimal::new(50, 4));
    }

    #[test]
    fn test_liquidity_config_default() {
        let config = LiquidityConfig::default();
        assert_eq!(config.min_pool_size, Decimal::new(10000000, 2));
        assert_eq!(config.lp_fee_percentage, Decimal::new(30, 4));
        assert!(config.auto_rebalancing);
        assert!(config.yield_farming_enabled);
    }
}
