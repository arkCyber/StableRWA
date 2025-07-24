// =====================================================================================
// File: core-stablecoin/src/mechanisms/crypto_collateralized.rs
// Description: Crypto-collateralized stablecoin mechanism (e.g., DAI)
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

/// Crypto-collateralized stablecoin mechanism
#[derive(Debug, Clone)]
pub struct CryptoCollateralized {
    parameters: StabilityParameters,
    min_collateral_ratio: Decimal,
    liquidation_ratio: Decimal,
    supported_tokens: Vec<String>,
}

impl CryptoCollateralized {
    pub fn new(parameters: StabilityParameters) -> Self {
        Self {
            parameters,
            min_collateral_ratio: Decimal::new(150, 2), // 150%
            liquidation_ratio: Decimal::new(130, 2), // 130%
            supported_tokens: vec!["ETH".to_string(), "BTC".to_string(), "WBTC".to_string()],
        }
    }
}

#[async_trait]
impl StabilityMechanismTrait for CryptoCollateralized {
    fn name(&self) -> &'static str {
        "Crypto Collateralized"
    }

    fn description(&self) -> &'static str {
        "Stablecoin backed by cryptocurrency collateral with over-collateralization"
    }

    fn requires_collateral(&self) -> bool {
        true
    }

    fn min_collateral_ratio(&self) -> Decimal {
        self.min_collateral_ratio
    }

    async fn calculate_required_collateral(
        &self,
        _stablecoin: &Stablecoin,
        issuance_amount: Decimal,
    ) -> StablecoinResult<Decimal> {
        Ok(issuance_amount * self.min_collateral_ratio / Decimal::new(100, 0))
    }

    async fn validate_collateral(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
        issuance_amount: Decimal,
    ) -> StablecoinResult<bool> {
        let required_collateral = self.calculate_required_collateral(stablecoin, issuance_amount).await?;
        let total_collateral_value: Decimal = collateral.iter().map(|c| c.value_usd).sum();
        
        // Validate that collateral is crypto-based
        for position in collateral {
            match &position.collateral_type {
                crate::types::CollateralType::Crypto { symbol, .. } => {
                    if !self.supported_tokens.contains(symbol) {
                        return Err(StablecoinError::UnsupportedStabilityMechanism(
                            format!("Token {} not supported as collateral", symbol)
                        ));
                    }
                }
                _ => {
                    return Err(StablecoinError::UnsupportedStabilityMechanism(
                        "Only crypto collateral supported for crypto-collateralized stablecoins".to_string()
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
        let total_debt = total_collateral_value / self.min_collateral_ratio * Decimal::new(100, 0);
        
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
        Ok(collateral_ratio < self.liquidation_ratio)
    }

    async fn calculate_liquidation_penalty(
        &self,
        _stablecoin: &Stablecoin,
        collateral_value: Decimal,
    ) -> StablecoinResult<Decimal> {
        let penalty_rate = Decimal::new(13, 2); // 13% penalty (typical for MakerDAO)
        Ok(collateral_value * penalty_rate / Decimal::new(100, 0))
    }

    async fn perform_stability_action(
        &self,
        stablecoin: &mut Stablecoin,
        current_price: Decimal,
    ) -> StablecoinResult<StabilityAction> {
        let target_price = stablecoin.target_price;
        let price_deviation = ((current_price - target_price) / target_price).abs();
        
        if price_deviation <= self.parameters.price_band / Decimal::new(100, 0) {
            return Ok(StabilityAction::None);
        }

        if current_price < target_price {
            // Price below target - adjust stability fee to reduce supply
            let new_rate = self.parameters.stability_fee * (Decimal::ONE + price_deviation);
            Ok(StabilityAction::AdjustInterestRate { new_rate })
        } else {
            // Price above target - lower stability fee to increase supply
            let new_rate = self.parameters.stability_fee * (Decimal::ONE - price_deviation / Decimal::new(2, 0));
            Ok(StabilityAction::AdjustInterestRate { new_rate })
        }
    }

    async fn update_parameters(&mut self, parameters: StabilityParameters) -> StablecoinResult<()> {
        self.parameters = parameters;
        Ok(())
    }

    async fn get_stability_metrics(&self, stablecoin: &Stablecoin) -> StablecoinResult<StabilityMetrics> {
        let price_deviation = ((stablecoin.current_price - stablecoin.target_price) / stablecoin.target_price).abs();
        let collateral_ratio = stablecoin.collateral_ratio;
        
        let mut stability_score = 85u8; // Start with good score
        
        if price_deviation > Decimal::new(5, 2) { // > 5%
            stability_score = stability_score.saturating_sub(30);
        } else if price_deviation > Decimal::new(2, 2) { // > 2%
            stability_score = stability_score.saturating_sub(15);
        }
        
        if collateral_ratio < Decimal::new(140, 2) { // < 140%
            stability_score = stability_score.saturating_sub(25);
        }

        let risk_level = match stability_score {
            80..=100 => RiskLevel::Low,
            60..=79 => RiskLevel::Medium,
            40..=59 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(StabilityMetrics {
            price_deviation,
            collateral_ratio,
            supply_utilization: Decimal::new(70, 2),
            stability_score,
            risk_level,
            time_since_rebalancing: 3600, // Mock value
            arbitrage_opportunities: if price_deviation > Decimal::new(1, 2) { 2 } else { 0 },
        })
    }

    async fn handle_emergency(
        &self,
        stablecoin: &mut Stablecoin,
        emergency_type: EmergencyType,
    ) -> StablecoinResult<EmergencyResponse> {
        match emergency_type {
            EmergencyType::LowCollateralRatio { ratio } => {
                if ratio < self.liquidation_ratio {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::TriggerLiquidation { positions: vec![] },
                        reason: format!("Collateral ratio {} below liquidation threshold", ratio),
                        estimated_recovery_time: Some(1800),
                        requires_manual_intervention: false,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::CircuitBreaker { duration: 3600 },
                        reason: "Preventive circuit breaker activation".to_string(),
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
    fn test_crypto_collateralized_creation() {
        let parameters = StabilityParameters::default();
        let mechanism = CryptoCollateralized::new(parameters);
        
        assert_eq!(mechanism.name(), "Crypto Collateralized");
        assert!(mechanism.requires_collateral());
        assert_eq!(mechanism.min_collateral_ratio(), Decimal::new(150, 2));
    }
}
