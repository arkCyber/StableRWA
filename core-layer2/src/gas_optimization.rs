// =====================================================================================
// File: core-layer2/src/gas_optimization.rs
// Description: Gas optimization strategies for Layer 2 (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Gas optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    pub enable_gas_optimization: bool,
    pub target_gas_price: u64,
    pub gas_price_buffer: Decimal,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            enable_gas_optimization: true,
            target_gas_price: 20,
            gas_price_buffer: Decimal::new(110, 2), // 10% buffer
        }
    }
}

/// Gas optimization service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasOptimizationService {
    pub enabled: bool,
    pub batch_transactions: bool,
    pub dynamic_pricing: bool,
}

/// Gas price oracle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPriceOracle {
    pub current_price: Decimal,
    pub fast_price: Decimal,
    pub safe_price: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Transaction batcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBatcher {
    pub max_batch_size: u32,
    pub batch_timeout_seconds: u64,
    pub gas_savings_percentage: Decimal,
}

/// Gas estimator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimator {
    pub base_gas: u64,
    pub per_byte_gas: u64,
    pub contract_call_gas: u64,
}

impl Default for GasOptimizationService {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_transactions: true,
            dynamic_pricing: true,
        }
    }
}
