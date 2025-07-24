// =====================================================================================
// File: core-defi/src/liquidity_pools.rs
// Description: Liquidity pools management implementation (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Liquidity pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPoolConfig {
    pub min_liquidity: Decimal,
}

impl Default for LiquidityPoolConfig {
    fn default() -> Self {
        Self {
            min_liquidity: Decimal::new(1000, 2),
        }
    }
}
