// =====================================================================================
// File: core-layer2/src/lib.rs
// Description: Layer 2 and cross-chain interoperability for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Layer 2 Module
//! 
//! This module provides comprehensive Layer 2 scaling solutions and cross-chain
//! interoperability for the StableRWA platform, supporting major L2 networks
//! like Polygon, Arbitrum, Optimism, and Base.

pub mod error;
pub mod types;
pub mod polygon;
pub mod arbitrum;
pub mod optimism;
pub mod base;
pub mod zksync;
pub mod starknet;
pub mod avalanche;
pub mod cross_chain;
pub mod bridge;
pub mod state_sync;
pub mod gas_optimization;
pub mod service;

// Re-export main types and traits
pub use error::{Layer2Error, Layer2Result};
pub use types::{
    Layer2Network, CrossChainMessage, BridgeTransaction, StateUpdate,
    GasEstimate, NetworkStatus, ChainConfig
};
pub use polygon::{PolygonService, PolygonConfig, PolygonBridge};
pub use arbitrum::{ArbitrumService, ArbitrumConfig, ArbitrumBridge};
pub use optimism::{OptimismService, OptimismConfig, OptimismBridge};
pub use base::{BaseService, BaseConfig, BaseBridge, BaseTransaction, PaymasterData, SmartWalletConfig};
pub use zksync::{ZkSyncConfig, ZkSyncNetwork};
pub use starknet::{StarkNetConfig, StarkNetNetwork};
pub use avalanche::{AvalancheConfig, AvalancheNetwork};
pub use cross_chain::{
    CrossChainService, CrossChainRouter, MessageRelay,
    CrossChainSwap, CrossChainLiquidity, CrossChainProtocol, LayerZeroService, AxelarService
};
pub use bridge::{
    BridgeService, BridgeValidator, BridgeRelay, BridgeConfig, BridgeType,
    LockAndMint, BurnAndRelease, NativeBridge
};
pub use state_sync::{
    StateSyncService, StateRoot, StateMerkleTree,
    CheckpointManager, StateProof
};
pub use gas_optimization::{
    GasOptimizationService, GasPriceOracle, TransactionBatcher,
    GasEstimator
};
pub use service::Layer2Service;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main Layer 2 service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2ServiceConfig {
    /// Polygon configuration
    pub polygon_config: polygon::PolygonConfig,
    /// Arbitrum configuration
    pub arbitrum_config: arbitrum::ArbitrumConfig,
    /// Optimism configuration
    pub optimism_config: optimism::OptimismConfig,
    /// Base configuration
    pub base_config: base::BaseConfig,
    /// Cross-chain configuration
    pub cross_chain_config: cross_chain::CrossChainConfig,
    /// Bridge configuration
    pub bridge_config: bridge::BridgeConfig,
    /// Gas optimization configuration
    pub gas_config: gas_optimization::GasConfig,
    /// Global Layer 2 settings
    pub global_settings: GlobalLayer2Settings,
}

impl Default for Layer2ServiceConfig {
    fn default() -> Self {
        Self {
            polygon_config: polygon::PolygonConfig::default(),
            arbitrum_config: arbitrum::ArbitrumConfig::default(),
            optimism_config: optimism::OptimismConfig::default(),
            base_config: base::BaseConfig::default(),
            cross_chain_config: cross_chain::CrossChainConfig::default(),
            bridge_config: bridge::BridgeConfig::default(),
            gas_config: gas_optimization::GasConfig::default(),
            global_settings: GlobalLayer2Settings::default(),
        }
    }
}

/// Global Layer 2 settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLayer2Settings {
    /// Enable automatic network selection
    pub enable_auto_network_selection: bool,
    /// Default gas price strategy
    pub default_gas_strategy: GasStrategy,
    /// Maximum gas price in gwei
    pub max_gas_price_gwei: u64,
    /// Enable transaction batching
    pub enable_batching: bool,
    /// Batch size for transactions
    pub batch_size: u32,
    /// Enable meta transactions
    pub enable_meta_transactions: bool,
    /// Enable gasless transactions
    pub enable_gasless_transactions: bool,
    /// Confirmation blocks required
    pub confirmation_blocks: u32,
    /// Transaction timeout in minutes
    pub transaction_timeout_minutes: u32,
}

impl Default for GlobalLayer2Settings {
    fn default() -> Self {
        Self {
            enable_auto_network_selection: true,
            default_gas_strategy: GasStrategy::Fast,
            max_gas_price_gwei: 100,
            enable_batching: true,
            batch_size: 10,
            enable_meta_transactions: true,
            enable_gasless_transactions: true,
            confirmation_blocks: 12,
            transaction_timeout_minutes: 30,
        }
    }
}

/// Gas price strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GasStrategy {
    Slow,
    Standard,
    Fast,
    Fastest,
    Custom(u64),
}

/// Layer 2 metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2Metrics {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub total_gas_saved: u64,
    pub average_confirmation_time: f64,
    pub cross_chain_volume_24h: Decimal,
    pub bridge_transactions_24h: u64,
    pub network_metrics: HashMap<String, NetworkMetrics>,
    pub last_updated: DateTime<Utc>,
}

/// Network-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub network_name: String,
    pub transactions_24h: u64,
    pub volume_24h: Decimal,
    pub average_gas_price: u64,
    pub average_block_time: f64,
    pub tps: f64,
    pub uptime_percentage: Decimal,
}

/// Layer 2 health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer2HealthStatus {
    pub overall_status: String,
    pub polygon_status: String,
    pub arbitrum_status: String,
    pub optimism_status: String,
    pub base_status: String,
    pub bridge_status: String,
    pub cross_chain_status: String,
    pub network_statuses: HashMap<String, NetworkStatus>,
    pub last_check: DateTime<Utc>,
}







#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer2_config_default() {
        let config = Layer2ServiceConfig::default();
        assert_eq!(config.polygon_config.chain_id, 137);
        assert_eq!(config.arbitrum_config.chain_id, 42161);
        assert_eq!(config.optimism_config.chain_id, 10);
        assert_eq!(config.base_config.chain_id, 8453);
        assert!(config.global_settings.enable_auto_network_selection);
        assert!(config.global_settings.enable_batching);
    }

    #[test]
    fn test_gas_strategy() {
        let strategies = vec![
            GasStrategy::Slow,
            GasStrategy::Standard,
            GasStrategy::Fast,
            GasStrategy::Fastest,
            GasStrategy::Custom(100),
        ];

        for strategy in strategies {
            match strategy {
                GasStrategy::Custom(price) => assert_eq!(price, 100),
                _ => {} // Other strategies
            }
        }
    }

    #[test]
    fn test_global_settings() {
        let settings = GlobalLayer2Settings::default();
        assert_eq!(settings.default_gas_strategy, GasStrategy::Fast);
        assert_eq!(settings.max_gas_price_gwei, 100);
        assert_eq!(settings.batch_size, 10);
        assert_eq!(settings.confirmation_blocks, 12);
        assert!(settings.enable_meta_transactions);
        assert!(settings.enable_gasless_transactions);
    }
}
