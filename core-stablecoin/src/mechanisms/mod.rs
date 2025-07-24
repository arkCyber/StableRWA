// =====================================================================================
// File: core-stablecoin/src/mechanisms/mod.rs
// Description: Stability mechanism implementations for different stablecoin types
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Stability Mechanisms
//! 
//! This module provides implementations for different stablecoin stability mechanisms:
//! 
//! - **Fiat Collateralized**: Backed by fiat currency reserves (e.g., USDC, USDT)
//! - **Crypto Collateralized**: Backed by cryptocurrency collateral (e.g., DAI)
//! - **Algorithmic**: Maintained through algorithmic supply adjustments
//! - **RWA Backed**: Backed by real-world assets (StableRWA's innovation)
//! - **Hybrid**: Combination of multiple mechanisms

use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{StablecoinError, StablecoinResult};
use crate::types::{Stablecoin, CollateralPosition, StabilityParameters};

pub mod fiat_collateralized;
pub mod crypto_collateralized;
pub mod algorithmic;
pub mod rwa_backed;
pub mod hybrid;

// Re-export implementations
pub use fiat_collateralized::FiatCollateralized;
pub use crypto_collateralized::CryptoCollateralized;
pub use algorithmic::Algorithmic;
pub use rwa_backed::RWABacked;
pub use hybrid::Hybrid;

/// Trait defining the interface for stability mechanisms
#[async_trait]
pub trait StabilityMechanismTrait: Send + Sync {
    /// Get the mechanism name
    fn name(&self) -> &'static str;

    /// Get the mechanism description
    fn description(&self) -> &'static str;

    /// Check if the mechanism requires collateral
    fn requires_collateral(&self) -> bool;

    /// Get minimum collateral ratio required
    fn min_collateral_ratio(&self) -> Decimal;

    /// Calculate required collateral for issuance
    async fn calculate_required_collateral(
        &self,
        stablecoin: &Stablecoin,
        issuance_amount: Decimal,
    ) -> StablecoinResult<Decimal>;

    /// Validate collateral for issuance
    async fn validate_collateral(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
        issuance_amount: Decimal,
    ) -> StablecoinResult<bool>;

    /// Calculate collateral ratio
    async fn calculate_collateral_ratio(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
    ) -> StablecoinResult<Decimal>;

    /// Check if position needs liquidation
    async fn needs_liquidation(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
    ) -> StablecoinResult<bool>;

    /// Calculate liquidation penalty
    async fn calculate_liquidation_penalty(
        &self,
        stablecoin: &Stablecoin,
        collateral_value: Decimal,
    ) -> StablecoinResult<Decimal>;

    /// Perform stability action (rebalancing, arbitrage, etc.)
    async fn perform_stability_action(
        &self,
        stablecoin: &mut Stablecoin,
        current_price: Decimal,
    ) -> StablecoinResult<StabilityAction>;

    /// Update stability parameters
    async fn update_parameters(
        &mut self,
        parameters: StabilityParameters,
    ) -> StablecoinResult<()>;

    /// Get current stability metrics
    async fn get_stability_metrics(
        &self,
        stablecoin: &Stablecoin,
    ) -> StablecoinResult<StabilityMetrics>;

    /// Handle emergency situations
    async fn handle_emergency(
        &self,
        stablecoin: &mut Stablecoin,
        emergency_type: EmergencyType,
    ) -> StablecoinResult<EmergencyResponse>;
}

/// Stability action types
#[derive(Debug, Clone, PartialEq)]
pub enum StabilityAction {
    /// No action needed
    None,
    /// Increase supply to lower price
    IncreaseSupply { amount: Decimal },
    /// Decrease supply to raise price
    DecreaseSupply { amount: Decimal },
    /// Adjust interest rates
    AdjustInterestRate { new_rate: Decimal },
    /// Trigger arbitrage incentives
    TriggerArbitrage { incentive_rate: Decimal },
    /// Rebalance collateral
    RebalanceCollateral { target_allocation: Vec<(String, Decimal)> },
    /// Emergency pause
    EmergencyPause { reason: String },
}

/// Stability metrics
#[derive(Debug, Clone)]
pub struct StabilityMetrics {
    /// Current price deviation from target
    pub price_deviation: Decimal,
    /// Collateral ratio
    pub collateral_ratio: Decimal,
    /// Supply utilization
    pub supply_utilization: Decimal,
    /// Stability score (0-100)
    pub stability_score: u8,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Time since last rebalancing
    pub time_since_rebalancing: u64,
    /// Active arbitrage opportunities
    pub arbitrage_opportunities: u32,
}

/// Risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Emergency types
#[derive(Debug, Clone, PartialEq)]
pub enum EmergencyType {
    /// Price deviation beyond threshold
    PriceDeviation { deviation: Decimal },
    /// Collateral ratio below minimum
    LowCollateralRatio { ratio: Decimal },
    /// Oracle failure or manipulation
    OracleFailure { reason: String },
    /// Smart contract vulnerability
    ContractVulnerability { description: String },
    /// Regulatory action
    RegulatoryAction { jurisdiction: String, action: String },
    /// Market manipulation detected
    MarketManipulation { evidence: String },
}

/// Emergency response
#[derive(Debug, Clone)]
pub struct EmergencyResponse {
    /// Action taken
    pub action: EmergencyAction,
    /// Reason for action
    pub reason: String,
    /// Estimated recovery time
    pub estimated_recovery_time: Option<u64>,
    /// Required manual intervention
    pub requires_manual_intervention: bool,
}

/// Emergency actions
#[derive(Debug, Clone, PartialEq)]
pub enum EmergencyAction {
    /// Pause all operations
    PauseAll,
    /// Pause issuance only
    PauseIssuance,
    /// Pause redemption only
    PauseRedemption,
    /// Switch to backup oracle
    SwitchOracle { new_oracle: String },
    /// Trigger liquidation
    TriggerLiquidation { positions: Vec<Uuid> },
    /// Activate circuit breaker
    CircuitBreaker { duration: u64 },
    /// Notify authorities
    NotifyAuthorities { message: String },
}

/// Factory for creating stability mechanisms
pub struct StabilityMechanismFactory;

impl StabilityMechanismFactory {
    /// Create a stability mechanism instance
    pub fn create(
        mechanism_type: crate::types::StabilityMechanism,
        parameters: StabilityParameters,
    ) -> StablecoinResult<Box<dyn StabilityMechanismTrait>> {
        match mechanism_type {
            crate::types::StabilityMechanism::Fiat => {
                Ok(Box::new(FiatCollateralized::new(parameters)))
            }
            crate::types::StabilityMechanism::Crypto => {
                Ok(Box::new(CryptoCollateralized::new(parameters)))
            }
            crate::types::StabilityMechanism::Algorithmic => {
                Ok(Box::new(Algorithmic::new(parameters)))
            }
            crate::types::StabilityMechanism::RWABacked => {
                Ok(Box::new(RWABacked::new(parameters)))
            }
            crate::types::StabilityMechanism::Hybrid => {
                Ok(Box::new(Hybrid::new(parameters)))
            }
        }
    }

    /// Get supported mechanisms
    pub fn supported_mechanisms() -> Vec<crate::types::StabilityMechanism> {
        vec![
            crate::types::StabilityMechanism::Fiat,
            crate::types::StabilityMechanism::Crypto,
            crate::types::StabilityMechanism::Algorithmic,
            crate::types::StabilityMechanism::RWABacked,
            crate::types::StabilityMechanism::Hybrid,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_supported_mechanisms() {
        let mechanisms = StabilityMechanismFactory::supported_mechanisms();
        assert_eq!(mechanisms.len(), 5);
        assert!(mechanisms.contains(&crate::types::StabilityMechanism::RWABacked));
    }

    #[tokio::test]
    async fn test_factory_create_mechanism() {
        let parameters = StabilityParameters::default();
        let mechanism = StabilityMechanismFactory::create(
            crate::types::StabilityMechanism::RWABacked,
            parameters,
        ).unwrap();
        
        assert_eq!(mechanism.name(), "RWA Backed");
        assert!(mechanism.requires_collateral());
    }
}
