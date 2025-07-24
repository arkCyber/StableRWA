// =====================================================================================
// File: core-defi/src/price_oracle.rs
// Description: Price oracle integration implementation (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Price oracle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceOracleConfig {
    pub update_interval_seconds: u64,
    pub deviation_threshold: Decimal,
}

impl Default for PriceOracleConfig {
    fn default() -> Self {
        Self {
            update_interval_seconds: 60,
            deviation_threshold: Decimal::new(5, 2),
        }
    }
}
