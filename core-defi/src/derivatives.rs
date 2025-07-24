// =====================================================================================
// File: core-defi/src/derivatives.rs
// Description: Derivatives trading implementation (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Derivatives configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivativesConfig {
    pub enable_options: bool,
    pub enable_futures: bool,
}

impl Default for DerivativesConfig {
    fn default() -> Self {
        Self {
            enable_options: true,
            enable_futures: true,
        }
    }
}
