// =====================================================================================
// File: core-defi/src/governance.rs
// Description: Governance protocol implementation (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub voting_period_days: u32,
    pub quorum_threshold: Decimal,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            voting_period_days: 7,
            quorum_threshold: Decimal::new(10, 2),
        }
    }
}
