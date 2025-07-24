// =====================================================================================
// File: core-stablecoin/src/mechanisms/hybrid.rs
// Description: Hybrid stablecoin mechanism (combination of multiple approaches)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::{StablecoinResult};
use crate::types::{Stablecoin, CollateralPosition, StabilityParameters};
use super::{
    StabilityMechanismTrait, StabilityAction, StabilityMetrics, RiskLevel,
    EmergencyType, EmergencyResponse, EmergencyAction,
};

/// Hybrid stablecoin mechanism combining multiple approaches
#[derive(Debug, Clone)]
pub struct Hybrid {
    parameters: StabilityParameters,
    collateral_weight: Decimal, // Weight of collateral-based stability
    algorithmic_weight: Decimal, // Weight of algorithmic stability
    min_collateral_ratio: Decimal,
}

impl Hybrid {
    pub fn new(parameters: StabilityParameters) -> Self {
        Self {
            parameters,
            collateral_weight: Decimal::new(70, 2), // 70% collateral-based
            algorithmic_weight: Decimal::new(30, 2), // 30% algorithmic
            min_collateral_ratio: Decimal::new(120, 2), // 120%
        }
    }

    pub fn set_weights(&mut self, collateral_weight: Decimal, algorithmic_weight: Decimal) -> StablecoinResult<()> {
        if collateral_weight + algorithmic_weight != Decimal::new(100, 2) {
            return Err(crate::StablecoinError::InvalidConfiguration(
                "Weights must sum to 100%".to_string()
            ));
        }
        self.collateral_weight = collateral_weight;
        self.algorithmic_weight = algorithmic_weight;
        Ok(())
    }
}

#[async_trait]
impl StabilityMechanismTrait for Hybrid {
    fn name(&self) -> &'static str {
        "Hybrid"
    }

    fn description(&self) -> &'static str {
        "Stablecoin using a combination of collateral backing and algorithmic mechanisms"
    }

    fn requires_collateral(&self) -> bool {
        self.collateral_weight > Decimal::ZERO
    }

    fn min_collateral_ratio(&self) -> Decimal {
        self.min_collateral_ratio
    }

    async fn calculate_required_collateral(
        &self,
        _stablecoin: &Stablecoin,
        issuance_amount: Decimal,
    ) -> StablecoinResult<Decimal> {
        // Only require collateral for the collateral-weighted portion
        let collateral_portion = issuance_amount * self.collateral_weight / Decimal::new(100, 0);
        Ok(collateral_portion * self.min_collateral_ratio / Decimal::new(100, 0))
    }

    async fn validate_collateral(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
        issuance_amount: Decimal,
    ) -> StablecoinResult<bool> {
        if self.collateral_weight == Decimal::ZERO {
            return Ok(true);
        }

        let required_collateral = self.calculate_required_collateral(stablecoin, issuance_amount).await?;
        let total_collateral_value: Decimal = collateral.iter().map(|c| c.value_usd).sum();
        
        Ok(total_collateral_value >= required_collateral)
    }

    async fn calculate_collateral_ratio(
        &self,
        _stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
    ) -> StablecoinResult<Decimal> {
        let total_collateral_value: Decimal = collateral.iter().map(|c| c.value_usd).sum();
        
        if self.collateral_weight == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        // Calculate debt based on collateral portion
        let collateral_backed_debt = total_collateral_value / self.min_collateral_ratio * Decimal::new(100, 0);
        
        if collateral_backed_debt == Decimal::ZERO {
            return Ok(Decimal::MAX);
        }
        
        Ok((total_collateral_value / collateral_backed_debt) * Decimal::new(100, 0))
    }

    async fn needs_liquidation(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
    ) -> StablecoinResult<bool> {
        if self.collateral_weight == Decimal::ZERO {
            return Ok(false);
        }

        let collateral_ratio = self.calculate_collateral_ratio(stablecoin, collateral).await?;
        Ok(collateral_ratio < self.parameters.liquidation_threshold)
    }

    async fn calculate_liquidation_penalty(
        &self,
        _stablecoin: &Stablecoin,
        collateral_value: Decimal,
    ) -> StablecoinResult<Decimal> {
        // Moderate penalty for hybrid mechanism
        let penalty_rate = Decimal::new(8, 2); // 8% penalty
        Ok(collateral_value * penalty_rate / Decimal::new(100, 0))
    }

    async fn perform_stability_action(
        &self,
        stablecoin: &mut Stablecoin,
        current_price: Decimal,
    ) -> StablecoinResult<StabilityAction> {
        let target_price = stablecoin.target_price;
        let price_deviation = (current_price - target_price) / target_price;
        
        if price_deviation.abs() <= self.parameters.price_band / Decimal::new(100, 0) {
            return Ok(StabilityAction::None);
        }

        // Hybrid approach: combine collateral and algorithmic responses
        if current_price < target_price {
            // Price below target
            if self.collateral_weight > self.algorithmic_weight {
                // Collateral-heavy: adjust interest rates
                let new_rate = self.parameters.stability_fee * (Decimal::ONE + price_deviation.abs());
                Ok(StabilityAction::AdjustInterestRate { new_rate })
            } else {
                // Algorithm-heavy: decrease supply
                let contraction_amount = stablecoin.total_supply * price_deviation.abs() * Decimal::new(2, 2) / Decimal::new(100, 0);
                Ok(StabilityAction::DecreaseSupply { amount: contraction_amount })
            }
        } else {
            // Price above target
            if self.collateral_weight > self.algorithmic_weight {
                // Collateral-heavy: lower interest rates
                let new_rate = self.parameters.stability_fee * (Decimal::ONE - price_deviation / Decimal::new(2, 0));
                Ok(StabilityAction::AdjustInterestRate { new_rate })
            } else {
                // Algorithm-heavy: increase supply
                let expansion_amount = stablecoin.total_supply * price_deviation * Decimal::new(3, 2) / Decimal::new(100, 0);
                Ok(StabilityAction::IncreaseSupply { amount: expansion_amount })
            }
        }
    }

    async fn update_parameters(&mut self, parameters: StabilityParameters) -> StablecoinResult<()> {
        self.parameters = parameters;
        Ok(())
    }

    async fn get_stability_metrics(&self, stablecoin: &Stablecoin) -> StablecoinResult<StabilityMetrics> {
        let price_deviation = ((stablecoin.current_price - stablecoin.target_price) / stablecoin.target_price).abs();
        let collateral_ratio = stablecoin.collateral_ratio;
        
        // Hybrid mechanisms have moderate stability scores
        let mut stability_score = 75u8;
        
        if price_deviation > Decimal::new(5, 2) { // > 5%
            stability_score = stability_score.saturating_sub(25);
        } else if price_deviation > Decimal::new(2, 2) { // > 2%
            stability_score = stability_score.saturating_sub(12);
        }
        
        if self.collateral_weight > Decimal::ZERO && collateral_ratio < Decimal::new(130, 2) { // < 130%
            stability_score = stability_score.saturating_sub(20);
        }

        let risk_level = match stability_score {
            80..=100 => RiskLevel::Low,
            65..=79 => RiskLevel::Medium,
            45..=64 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(StabilityMetrics {
            price_deviation,
            collateral_ratio,
            supply_utilization: Decimal::new(80, 2),
            stability_score,
            risk_level,
            time_since_rebalancing: 1800, // 30 minutes
            arbitrage_opportunities: if price_deviation > Decimal::new(15, 3) { 3 } else { 1 },
        })
    }

    async fn handle_emergency(
        &self,
        stablecoin: &mut Stablecoin,
        emergency_type: EmergencyType,
    ) -> StablecoinResult<EmergencyResponse> {
        match emergency_type {
            EmergencyType::PriceDeviation { deviation } => {
                if deviation > Decimal::new(15, 2) { // > 15%
                    stablecoin.status = crate::types::StablecoinStatus::EmergencyPause;
                    Ok(EmergencyResponse {
                        action: EmergencyAction::PauseAll,
                        reason: format!("Severe price deviation {} for hybrid stablecoin", deviation),
                        estimated_recovery_time: Some(3600), // 1 hour
                        requires_manual_intervention: true,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::CircuitBreaker { duration: 1800 },
                        reason: "Moderate volatility circuit breaker".to_string(),
                        estimated_recovery_time: Some(1800),
                        requires_manual_intervention: false,
                    })
                }
            }
            EmergencyType::LowCollateralRatio { ratio } => {
                if self.collateral_weight > Decimal::ZERO && ratio < self.parameters.liquidation_threshold {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::TriggerLiquidation { positions: vec![] },
                        reason: format!("Collateral ratio {} below threshold for hybrid mechanism", ratio),
                        estimated_recovery_time: Some(2400), // 40 minutes
                        requires_manual_intervention: false,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::NotifyAuthorities { 
                            message: "Collateral ratio warning for hybrid stablecoin".to_string() 
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
                        message: format!("Emergency: {:?}", emergency_type) 
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

    #[test]
    fn test_hybrid_creation() {
        let parameters = StabilityParameters::default();
        let mechanism = Hybrid::new(parameters);
        
        assert_eq!(mechanism.name(), "Hybrid");
        assert!(mechanism.requires_collateral());
        assert_eq!(mechanism.min_collateral_ratio(), Decimal::new(120, 2));
    }

    #[test]
    fn test_weight_setting() {
        let parameters = StabilityParameters::default();
        let mut mechanism = Hybrid::new(parameters);
        
        // Valid weights
        assert!(mechanism.set_weights(Decimal::new(60, 2), Decimal::new(40, 2)).is_ok());
        
        // Invalid weights (don't sum to 100%)
        assert!(mechanism.set_weights(Decimal::new(60, 2), Decimal::new(30, 2)).is_err());
    }
}
