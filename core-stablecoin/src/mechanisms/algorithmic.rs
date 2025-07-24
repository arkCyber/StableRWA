// =====================================================================================
// File: core-stablecoin/src/mechanisms/algorithmic.rs
// Description: Algorithmic stablecoin mechanism
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

/// Algorithmic stablecoin mechanism
#[derive(Debug, Clone)]
pub struct Algorithmic {
    parameters: StabilityParameters,
    expansion_rate: Decimal,
    contraction_rate: Decimal,
}

impl Algorithmic {
    pub fn new(parameters: StabilityParameters) -> Self {
        Self {
            parameters,
            expansion_rate: Decimal::new(5, 2), // 5% expansion rate
            contraction_rate: Decimal::new(3, 2), // 3% contraction rate
        }
    }
}

#[async_trait]
impl StabilityMechanismTrait for Algorithmic {
    fn name(&self) -> &'static str {
        "Algorithmic"
    }

    fn description(&self) -> &'static str {
        "Stablecoin maintained through algorithmic supply adjustments"
    }

    fn requires_collateral(&self) -> bool {
        false
    }

    fn min_collateral_ratio(&self) -> Decimal {
        Decimal::ZERO
    }

    async fn calculate_required_collateral(
        &self,
        _stablecoin: &Stablecoin,
        _issuance_amount: Decimal,
    ) -> StablecoinResult<Decimal> {
        Ok(Decimal::ZERO)
    }

    async fn validate_collateral(
        &self,
        _stablecoin: &Stablecoin,
        _collateral: &[CollateralPosition],
        _issuance_amount: Decimal,
    ) -> StablecoinResult<bool> {
        Ok(true) // No collateral required
    }

    async fn calculate_collateral_ratio(
        &self,
        _stablecoin: &Stablecoin,
        _collateral: &[CollateralPosition],
    ) -> StablecoinResult<Decimal> {
        Ok(Decimal::ZERO) // No collateral
    }

    async fn needs_liquidation(
        &self,
        _stablecoin: &Stablecoin,
        _collateral: &[CollateralPosition],
    ) -> StablecoinResult<bool> {
        Ok(false) // No liquidation for algorithmic
    }

    async fn calculate_liquidation_penalty(
        &self,
        _stablecoin: &Stablecoin,
        _collateral_value: Decimal,
    ) -> StablecoinResult<Decimal> {
        Ok(Decimal::ZERO)
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

        if current_price < target_price {
            // Price below target - decrease supply
            let contraction_amount = stablecoin.total_supply * price_deviation.abs() * self.contraction_rate / Decimal::new(100, 0);
            Ok(StabilityAction::DecreaseSupply { amount: contraction_amount })
        } else {
            // Price above target - increase supply
            let expansion_amount = stablecoin.total_supply * price_deviation * self.expansion_rate / Decimal::new(100, 0);
            Ok(StabilityAction::IncreaseSupply { amount: expansion_amount })
        }
    }

    async fn update_parameters(&mut self, parameters: StabilityParameters) -> StablecoinResult<()> {
        self.parameters = parameters;
        Ok(())
    }

    async fn get_stability_metrics(&self, stablecoin: &Stablecoin) -> StablecoinResult<StabilityMetrics> {
        let price_deviation = ((stablecoin.current_price - stablecoin.target_price) / stablecoin.target_price).abs();
        
        let mut stability_score = 60u8; // Lower base score due to higher volatility
        
        if price_deviation > Decimal::new(10, 2) { // > 10%
            stability_score = stability_score.saturating_sub(40);
        } else if price_deviation > Decimal::new(5, 2) { // > 5%
            stability_score = stability_score.saturating_sub(20);
        }

        let risk_level = match stability_score {
            70..=100 => RiskLevel::Medium, // Algorithmic is inherently riskier
            50..=69 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(StabilityMetrics {
            price_deviation,
            collateral_ratio: Decimal::ZERO,
            supply_utilization: Decimal::new(100, 2), // Full utilization
            stability_score,
            risk_level,
            time_since_rebalancing: 300, // Frequent rebalancing
            arbitrage_opportunities: if price_deviation > Decimal::new(2, 2) { 5 } else { 1 },
        })
    }

    async fn handle_emergency(
        &self,
        stablecoin: &mut Stablecoin,
        emergency_type: EmergencyType,
    ) -> StablecoinResult<EmergencyResponse> {
        match emergency_type {
            EmergencyType::PriceDeviation { deviation } => {
                if deviation > Decimal::new(20, 2) { // > 20%
                    stablecoin.status = crate::types::StablecoinStatus::EmergencyPause;
                    Ok(EmergencyResponse {
                        action: EmergencyAction::PauseAll,
                        reason: format!("Extreme price deviation {} for algorithmic stablecoin", deviation),
                        estimated_recovery_time: Some(7200), // 2 hours
                        requires_manual_intervention: true,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::CircuitBreaker { duration: 3600 },
                        reason: "High volatility circuit breaker".to_string(),
                        estimated_recovery_time: Some(3600),
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
    fn test_algorithmic_creation() {
        let parameters = StabilityParameters::default();
        let mechanism = Algorithmic::new(parameters);
        
        assert_eq!(mechanism.name(), "Algorithmic");
        assert!(!mechanism.requires_collateral());
        assert_eq!(mechanism.min_collateral_ratio(), Decimal::ZERO);
    }
}
