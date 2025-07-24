// =====================================================================================
// File: core-defi/src/lib.rs
// Description: DeFi protocol integration for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core DeFi Module
//! 
//! This module provides comprehensive DeFi protocol integration for the StableRWA platform,
//! including AMM, lending protocols, staking, yield farming, and other DeFi primitives.

pub mod error;
pub mod types;
pub mod amm;
pub mod lending;
pub mod staking;
pub mod yield_farming;
pub mod flash_loans;
pub mod derivatives;
pub mod liquidity_pools;
pub mod price_oracle;
pub mod governance;
pub mod service;

// Re-export main types and traits
pub use lending::{LendingService, LendingConfig, LendingPosition, LendingMarket, CompoundProtocol, AaveProtocol, MakerDAOProtocol};
pub use staking::{StakingService, StakingConfig, StakingPosition, StakingPool, StakingReward, UnstakingRequest, ValidatorStaking, LiquidStaking};
pub use yield_farming::{YieldFarmingService, YieldFarmConfig, FarmingPosition, YieldFarm, HarvestRequest};
pub use flash_loans::{FlashLoanService, FlashLoanConfig, FlashLoanRequest, FlashLoanExecution, ArbitrageOpportunity, LiquidationOpportunity};


// Additional re-exports
pub use error::{DeFiError, DeFiResult};
pub use types::{Token, TokenPair, LiquidityPool, Position, Strategy, AMMProtocol};
pub use amm::{AMMService, SwapRequest, SwapResult, LiquidityRequest, LiquidityResult};
pub use service::DeFiService;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main DeFi service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiServiceConfig {
    /// AMM configuration
    pub amm_config: amm::AMMConfig,
    /// Lending configuration
    pub lending_config: lending::LendingConfig,
    /// Staking configuration
    pub staking_config: staking::StakingConfig,
    /// Yield farming configuration
    pub yield_farming_config: yield_farming::YieldFarmConfig,
    /// Flash loan configuration
    pub flash_loan_config: flash_loans::FlashLoanConfig,
    /// Derivatives configuration
    pub derivatives_config: derivatives::DerivativesConfig,
    /// Price oracle configuration
    pub price_oracle_config: price_oracle::PriceOracleConfig,
    /// Governance configuration
    pub governance_config: governance::GovernanceConfig,
    /// Global DeFi settings
    pub global_settings: GlobalDeFiSettings,
}

impl Default for DeFiServiceConfig {
    fn default() -> Self {
        Self {
            amm_config: amm::AMMConfig::default(),
            lending_config: lending::LendingConfig::default(),
            staking_config: staking::StakingConfig::default(),
            yield_farming_config: yield_farming::YieldFarmConfig::default(),
            flash_loan_config: flash_loans::FlashLoanConfig::default(),
            derivatives_config: derivatives::DerivativesConfig::default(),
            price_oracle_config: price_oracle::PriceOracleConfig::default(),
            governance_config: governance::GovernanceConfig::default(),
            global_settings: GlobalDeFiSettings::default(),
        }
    }
}

/// Global DeFi settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDeFiSettings {
    /// Maximum slippage tolerance
    pub max_slippage_tolerance: Decimal,
    /// Default transaction deadline in minutes
    pub default_deadline_minutes: u32,
    /// Enable MEV protection
    pub enable_mev_protection: bool,
    /// Enable front-running protection
    pub enable_frontrun_protection: bool,
    /// Gas price strategy
    pub gas_price_strategy: GasPriceStrategy,
    /// Maximum gas price in gwei
    pub max_gas_price_gwei: u64,
    /// Enable transaction batching
    pub enable_transaction_batching: bool,
    /// Batch size for transactions
    pub transaction_batch_size: u32,
    /// Enable yield optimization
    pub enable_yield_optimization: bool,
    /// Risk tolerance level
    pub risk_tolerance: RiskTolerance,
}

impl Default for GlobalDeFiSettings {
    fn default() -> Self {
        Self {
            max_slippage_tolerance: Decimal::new(300, 4), // 3%
            default_deadline_minutes: 20,
            enable_mev_protection: true,
            enable_frontrun_protection: true,
            gas_price_strategy: GasPriceStrategy::Fast,
            max_gas_price_gwei: 200,
            enable_transaction_batching: true,
            transaction_batch_size: 10,
            enable_yield_optimization: true,
            risk_tolerance: RiskTolerance::Medium,
        }
    }
}

/// Gas price strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GasPriceStrategy {
    Slow,
    Standard,
    Fast,
    Fastest,
    Custom(u64),
}

/// Risk tolerance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTolerance {
    Conservative,
    Medium,
    Aggressive,
}

/// DeFi protocol metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiMetrics {
    pub total_value_locked: Decimal,
    pub total_trading_volume_24h: Decimal,
    pub total_fees_earned_24h: Decimal,
    pub active_positions: u64,
    pub active_strategies: u64,
    pub yield_generated_24h: Decimal,
    pub impermanent_loss_24h: Decimal,
    pub protocol_breakdown: HashMap<String, ProtocolMetrics>,
    pub last_updated: DateTime<Utc>,
}

/// Protocol-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetrics {
    pub protocol_name: String,
    pub tvl: Decimal,
    pub volume_24h: Decimal,
    pub fees_24h: Decimal,
    pub apy: Decimal,
    pub utilization_rate: Decimal,
    pub active_users: u64,
}

/// DeFi health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiHealthStatus {
    pub overall_status: String,
    pub amm_status: String,
    pub lending_status: String,
    pub staking_status: String,
    pub yield_farming_status: String,
    pub oracle_status: String,
    pub governance_status: String,
    pub protocol_statuses: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
}

/// DeFi transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiTransaction {
    pub id: Uuid,
    pub user_id: String,
    pub protocol: String,
    pub transaction_type: DeFiTransactionType,
    pub token_in: Option<Token>,
    pub token_out: Option<Token>,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub gas_used: u64,
    pub gas_price: u64,
    pub transaction_hash: String,
    pub block_number: u64,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// DeFi transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeFiTransactionType {
    Swap,
    AddLiquidity,
    RemoveLiquidity,
    Lend,
    Borrow,
    Repay,
    Liquidate,
    Stake,
    Unstake,
    Harvest,
    Compound,
    FlashLoan,
    OptionTrade,
    FutureTrade,
    Vote,
    Delegate,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

/// DeFi strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeFiStrategy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub strategy_type: StrategyType,
    pub protocols: Vec<String>,
    pub tokens: Vec<Token>,
    pub target_apy: Decimal,
    pub risk_level: RiskLevel,
    pub min_investment: Decimal,
    pub max_investment: Option<Decimal>,
    pub auto_compound: bool,
    pub rebalance_threshold: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    YieldFarming,
    LiquidityMining,
    Arbitrage,
    DeltaNeutral,
    LeveragedYield,
    StablecoinYield,
    LiquidityProvision,
    Staking,
}

/// Risk level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Portfolio position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub id: Uuid,
    pub user_id: String,
    pub strategy_id: Option<Uuid>,
    pub protocol: String,
    pub position_type: PositionType,
    pub tokens: Vec<TokenPosition>,
    pub entry_value: Decimal,
    pub current_value: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub yield_earned: Decimal,
    pub fees_paid: Decimal,
    pub opened_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub is_active: bool,
}

/// Position type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionType {
    LiquidityProvider,
    Lender,
    Borrower,
    Staker,
    YieldFarmer,
    Trader,
    Arbitrageur,
}

/// Token position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPosition {
    pub token: Token,
    pub amount: Decimal,
    pub value_usd: Decimal,
    pub weight: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defi_config_default() {
        let config = DeFiServiceConfig::default();
        assert_eq!(config.global_settings.max_slippage_tolerance, Decimal::new(300, 4));
        assert_eq!(config.global_settings.default_deadline_minutes, 20);
        assert!(config.global_settings.enable_mev_protection);
        assert!(config.global_settings.enable_yield_optimization);
    }

    #[test]
    fn test_gas_price_strategy() {
        let strategies = vec![
            GasPriceStrategy::Slow,
            GasPriceStrategy::Standard,
            GasPriceStrategy::Fast,
            GasPriceStrategy::Fastest,
            GasPriceStrategy::Custom(100),
        ];

        for strategy in strategies {
            match strategy {
                GasPriceStrategy::Custom(price) => assert_eq!(price, 100),
                _ => {} // Other strategies
            }
        }
    }

    #[test]
    fn test_defi_transaction() {
        let transaction = DeFiTransaction {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            protocol: "Uniswap V3".to_string(),
            transaction_type: DeFiTransactionType::Swap,
            token_in: None,
            token_out: None,
            amount_in: Decimal::new(1000, 6), // 1000 USDC
            amount_out: Decimal::new(1, 18), // 1 ETH
            gas_used: 150000,
            gas_price: 50,
            transaction_hash: "0x123...".to_string(),
            block_number: 18500000,
            status: TransactionStatus::Confirmed,
            created_at: Utc::now(),
            executed_at: Some(Utc::now()),
            metadata: serde_json::json!({}),
        };

        assert_eq!(transaction.transaction_type, DeFiTransactionType::Swap);
        assert_eq!(transaction.status, TransactionStatus::Confirmed);
        assert_eq!(transaction.protocol, "Uniswap V3");
    }
}
