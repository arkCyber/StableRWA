// =====================================================================================
// File: core-stablecoin/src/mechanisms/fiat_collateralized.rs
// Description: Fiat-collateralized stablecoin mechanism (e.g., USDC, USDT)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::{StablecoinError, StablecoinResult};
use crate::types::{Stablecoin, CollateralPosition, StabilityParameters};
use super::{
    StabilityMechanismTrait, StabilityAction, StabilityMetrics, RiskLevel,
    EmergencyType, EmergencyResponse, EmergencyAction,
};

/// Fiat-collateralized stablecoin mechanism
/// 
/// This mechanism maintains stability through 1:1 backing with fiat currency reserves.
/// Examples: USDC, USDT, BUSD
#[derive(Debug, Clone)]
pub struct FiatCollateralized {
    parameters: StabilityParameters,
    reserve_ratio: Decimal, // Should be 100% or higher
    supported_currencies: Vec<String>,
}

impl FiatCollateralized {
    pub fn new(parameters: StabilityParameters) -> Self {
        Self {
            parameters,
            reserve_ratio: Decimal::new(100, 0), // 100% reserve ratio
            supported_currencies: vec!["USD".to_string(), "EUR".to_string(), "GBP".to_string()],
        }
    }

    pub fn add_supported_currency(&mut self, currency: String) {
        if !self.supported_currencies.contains(&currency) {
            self.supported_currencies.push(currency);
        }
    }

    pub fn set_reserve_ratio(&mut self, ratio: Decimal) -> StablecoinResult<()> {
        if ratio < Decimal::new(100, 2) {
            return Err(StablecoinError::InvalidConfiguration(
                "Reserve ratio cannot be less than 100% for fiat-collateralized stablecoins".to_string()
            ));
        }
        self.reserve_ratio = ratio;
        Ok(())
    }
}

#[async_trait]
impl StabilityMechanismTrait for FiatCollateralized {
    fn name(&self) -> &'static str {
        "Fiat Collateralized"
    }

    fn description(&self) -> &'static str {
        "Stablecoin backed 1:1 by fiat currency reserves"
    }

    fn requires_collateral(&self) -> bool {
        true
    }

    fn min_collateral_ratio(&self) -> Decimal {
        self.reserve_ratio
    }

    async fn calculate_required_collateral(
        &self,
        _stablecoin: &Stablecoin,
        issuance_amount: Decimal,
    ) -> StablecoinResult<Decimal> {
        // For fiat-collateralized, collateral requirement is 1:1 (or higher based on reserve ratio)
        Ok(issuance_amount * self.reserve_ratio / Decimal::new(100, 0))
    }

    async fn validate_collateral(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
        issuance_amount: Decimal,
    ) -> StablecoinResult<bool> {
        let required_collateral = self.calculate_required_collateral(stablecoin, issuance_amount).await?;
        let total_collateral_value: Decimal = collateral.iter().map(|c| c.value_usd).sum();
        
        // Validate that collateral is fiat-based
        for position in collateral {
            match &position.collateral_type {
                crate::types::CollateralType::Fiat { currency } => {
                    if !self.supported_currencies.contains(currency) {
                        return Err(StablecoinError::UnsupportedStabilityMechanism(
                            format!("Currency {} not supported", currency)
                        ));
                    }
                }
                _ => {
                    return Err(StablecoinError::UnsupportedStabilityMechanism(
                        "Only fiat collateral supported for fiat-collateralized stablecoins".to_string()
                    ));
                }
            }
        }
        
        Ok(total_collateral_value >= required_collateral)
    }

    async fn calculate_collateral_ratio(
        &self,
        _stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
    ) -> StablecoinResult<Decimal> {
        let total_collateral_value: Decimal = collateral.iter().map(|c| c.value_usd).sum();
        let total_debt = total_collateral_value; // Simplified: assume debt equals collateral for 1:1 backing
        
        if total_debt == Decimal::ZERO {
            return Ok(Decimal::MAX);
        }
        
        Ok((total_collateral_value / total_debt) * Decimal::new(100, 0))
    }

    async fn needs_liquidation(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
    ) -> StablecoinResult<bool> {
        let collateral_ratio = self.calculate_collateral_ratio(stablecoin, collateral).await?;
        // For fiat-collateralized, liquidation threshold is typically very close to 100%
        Ok(collateral_ratio < Decimal::new(100, 2))
    }

    async fn calculate_liquidation_penalty(
        &self,
        _stablecoin: &Stablecoin,
        collateral_value: Decimal,
    ) -> StablecoinResult<Decimal> {
        // Minimal penalty for fiat-collateralized as risk is low
        let penalty_rate = Decimal::new(1, 2); // 1% penalty
        Ok(collateral_value * penalty_rate / Decimal::new(100, 0))
    }

    async fn perform_stability_action(
        &self,
        stablecoin: &mut Stablecoin,
        current_price: Decimal,
    ) -> StablecoinResult<StabilityAction> {
        let target_price = stablecoin.target_price;
        let price_deviation = ((current_price - target_price) / target_price).abs();
        
        // Fiat-collateralized stablecoins typically have very tight price bands
        let tight_band = Decimal::new(5, 3); // 0.5%
        
        if price_deviation <= tight_band {
            return Ok(StabilityAction::None);
        }

        // For fiat-collateralized, stability is maintained through:
        // 1. Arbitrage opportunities
        // 2. Direct market operations by the issuer
        
        if price_deviation > Decimal::new(2, 2) { // > 2%
            // Significant deviation - trigger emergency measures
            Ok(StabilityAction::TriggerArbitrage { 
                incentive_rate: Decimal::new(50, 4) // 0.5% incentive
            })
        } else if current_price < target_price {
            // Price below peg - incentivize buying
            Ok(StabilityAction::TriggerArbitrage { 
                incentive_rate: Decimal::new(25, 4) // 0.25% incentive
            })
        } else {
            // Price above peg - increase supply slightly
            let supply_increase = stablecoin.total_supply * price_deviation * Decimal::new(1, 2); // 1% of deviation
            Ok(StabilityAction::IncreaseSupply { amount: supply_increase })
        }
    }

    async fn update_parameters(&mut self, parameters: StabilityParameters) -> StablecoinResult<()> {
        self.parameters = parameters;
        Ok(())
    }

    async fn get_stability_metrics(&self, stablecoin: &Stablecoin) -> StablecoinResult<StabilityMetrics> {
        let price_deviation = ((stablecoin.current_price - stablecoin.target_price) / stablecoin.target_price).abs();
        let collateral_ratio = stablecoin.collateral_ratio;
        
        // Fiat-collateralized stablecoins should have high stability scores
        let mut stability_score = 95u8; // Start with high score
        
        // Penalize for price deviation (stricter than other mechanisms)
        if price_deviation > Decimal::new(2, 2) { // > 2%
            stability_score = stability_score.saturating_sub(40);
        } else if price_deviation > Decimal::new(1, 2) { // > 1%
            stability_score = stability_score.saturating_sub(20);
        } else if price_deviation > Decimal::new(5, 3) { // > 0.5%
            stability_score = stability_score.saturating_sub(10);
        }
        
        // Penalize for insufficient reserves
        if collateral_ratio < Decimal::new(100, 2) { // < 100%
            stability_score = stability_score.saturating_sub(50);
        }

        let risk_level = match stability_score {
            90..=100 => RiskLevel::Low,
            75..=89 => RiskLevel::Medium,
            60..=74 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(StabilityMetrics {
            price_deviation,
            collateral_ratio,
            supply_utilization: Decimal::new(95, 2), // High utilization expected
            stability_score,
            risk_level,
            time_since_rebalancing: 0, // Continuous rebalancing
            arbitrage_opportunities: if price_deviation > Decimal::new(5, 3) { 1 } else { 0 },
        })
    }

    async fn handle_emergency(
        &self,
        stablecoin: &mut Stablecoin,
        emergency_type: EmergencyType,
    ) -> StablecoinResult<EmergencyResponse> {
        match emergency_type {
            EmergencyType::PriceDeviation { deviation } => {
                if deviation > Decimal::new(5, 2) { // > 5%
                    stablecoin.status = crate::types::StablecoinStatus::EmergencyPause;
                    Ok(EmergencyResponse {
                        action: EmergencyAction::PauseAll,
                        reason: format!("Severe price deviation {} for fiat-collateralized stablecoin", deviation),
                        estimated_recovery_time: Some(1800), // 30 minutes
                        requires_manual_intervention: true,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::TriggerLiquidation { positions: vec![] },
                        reason: "Triggering market operations to restore peg".to_string(),
                        estimated_recovery_time: Some(600), // 10 minutes
                        requires_manual_intervention: false,
                    })
                }
            }
            EmergencyType::LowCollateralRatio { ratio } => {
                if ratio < Decimal::new(95, 2) { // < 95%
                    Ok(EmergencyResponse {
                        action: EmergencyAction::PauseIssuance,
                        reason: format!("Reserve ratio {} below safe threshold", ratio),
                        estimated_recovery_time: Some(3600), // 1 hour
                        requires_manual_intervention: true,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::NotifyAuthorities { 
                            message: "Reserve ratio approaching minimum threshold".to_string() 
                        },
                        reason: "Preventive notification".to_string(),
                        estimated_recovery_time: Some(1800),
                        requires_manual_intervention: false,
                    })
                }
            }
            _ => {
                Ok(EmergencyResponse {
                    action: EmergencyAction::NotifyAuthorities { 
                        message: format!("Emergency situation detected: {:?}", emergency_type) 
                    },
                    reason: "Standard emergency protocol".to_string(),
                    estimated_recovery_time: None,
                    requires_manual_intervention: true,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_fiat_collateralized_creation() {
        let parameters = StabilityParameters::default();
        let mechanism = FiatCollateralized::new(parameters);
        
        assert_eq!(mechanism.name(), "Fiat Collateralized");
        assert!(mechanism.requires_collateral());
        assert_eq!(mechanism.min_collateral_ratio(), Decimal::new(100, 0));
    }

    #[test]
    fn test_reserve_ratio_validation() {
        let parameters = StabilityParameters::default();
        let mut mechanism = FiatCollateralized::new(parameters);
        
        // Should fail for ratio < 100%
        assert!(mechanism.set_reserve_ratio(Decimal::new(95, 2)).is_err());
        
        // Should succeed for ratio >= 100%
        assert!(mechanism.set_reserve_ratio(Decimal::new(105, 2)).is_ok());
        assert_eq!(mechanism.min_collateral_ratio(), Decimal::new(105, 2));
    }

    #[tokio::test]
    async fn test_calculate_required_collateral() {
        let parameters = StabilityParameters::default();
        let mechanism = FiatCollateralized::new(parameters);
        let stablecoin = create_test_stablecoin();

        let required = mechanism.calculate_required_collateral(&stablecoin, Decimal::new(1000, 0)).await.unwrap();
        assert_eq!(required, Decimal::new(1000, 0)); // 100% of 1000 = 1000
    }

    fn create_test_stablecoin() -> Stablecoin {
        Stablecoin {
            id: Uuid::new_v4(),
            symbol: "FUSD".to_string(),
            name: "Fiat USD".to_string(),
            decimals: 18,
            stability_mechanism: crate::types::StabilityMechanism::Fiat,
            target_price: Decimal::ONE,
            current_price: Decimal::ONE,
            total_supply: Decimal::ZERO,
            total_collateral_value: Decimal::ZERO,
            collateral_ratio: Decimal::ZERO,
            supported_collateral: vec![],
            contract_addresses: HashMap::new(),
            stability_parameters: StabilityParameters::default(),
            status: crate::types::StablecoinStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
