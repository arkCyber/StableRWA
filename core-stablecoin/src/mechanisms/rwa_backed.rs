// =====================================================================================
// File: core-stablecoin/src/mechanisms/rwa_backed.rs
// Description: RWA-backed stablecoin mechanism - StableRWA's core innovation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{StablecoinError, StablecoinResult};
use crate::types::{Stablecoin, CollateralPosition, StabilityParameters, CollateralType};
use super::{
    StabilityMechanismTrait, StabilityAction, StabilityMetrics, RiskLevel,
    EmergencyType, EmergencyResponse, EmergencyAction,
};

/// RWA-backed stablecoin mechanism
/// 
/// This mechanism uses real-world assets as collateral to maintain price stability.
/// It's StableRWA's core innovation, providing stability through diversified RWA portfolios.
#[derive(Debug, Clone)]
pub struct RWABacked {
    /// Stability parameters
    parameters: StabilityParameters,
    /// RWA portfolio composition
    portfolio: RWAPortfolio,
    /// Valuation model
    valuation_model: ValuationModel,
    /// Risk management settings
    risk_settings: RiskSettings,
    /// Last rebalancing timestamp
    last_rebalancing: Option<DateTime<Utc>>,
}

impl RWABacked {
    /// Create new RWA-backed mechanism
    pub fn new(parameters: StabilityParameters) -> Self {
        Self {
            parameters,
            portfolio: RWAPortfolio::default(),
            valuation_model: ValuationModel::default(),
            risk_settings: RiskSettings::default(),
            last_rebalancing: None,
        }
    }

    /// Add RWA asset to portfolio
    pub async fn add_rwa_asset(
        &mut self,
        asset: RWAAsset,
        target_allocation: Decimal,
    ) -> StablecoinResult<()> {
        // Validate asset
        self.validate_rwa_asset(&asset).await?;
        
        // Add to portfolio
        let asset_id = asset.id;
        self.portfolio.assets.insert(asset_id, asset);
        self.portfolio.target_allocations.insert(asset_id, target_allocation);
        
        // Update portfolio metrics
        self.update_portfolio_metrics().await?;
        
        Ok(())
    }

    /// Remove RWA asset from portfolio
    pub async fn remove_rwa_asset(&mut self, asset_id: Uuid) -> StablecoinResult<()> {
        self.portfolio.assets.remove(&asset_id);
        self.portfolio.target_allocations.remove(&asset_id);
        self.update_portfolio_metrics().await?;
        Ok(())
    }

    /// Validate RWA asset for inclusion
    async fn validate_rwa_asset(&self, asset: &RWAAsset) -> StablecoinResult<()> {
        // Check asset type is supported
        if !self.risk_settings.supported_asset_types.contains(&asset.asset_type) {
            return Err(StablecoinError::UnsupportedStabilityMechanism(
                format!("Asset type {} not supported", asset.asset_type)
            ));
        }

        // Check minimum value
        if asset.current_value < self.risk_settings.min_asset_value {
            return Err(StablecoinError::InvalidAmount(
                format!("Asset value {} below minimum {}", asset.current_value, self.risk_settings.min_asset_value)
            ));
        }

        // Check liquidity requirements
        if asset.liquidity_score < self.risk_settings.min_liquidity_score {
            return Err(StablecoinError::RiskLimitExceeded(
                format!("Asset liquidity score {} below minimum {}", asset.liquidity_score, self.risk_settings.min_liquidity_score)
            ));
        }

        // Check geographic diversification
        self.check_geographic_diversification(&asset.location).await?;

        Ok(())
    }

    /// Check geographic diversification requirements
    async fn check_geographic_diversification(&self, location: &str) -> StablecoinResult<()> {
        let current_exposure = self.portfolio.geographic_exposure.get(location).unwrap_or(&Decimal::ZERO);
        let max_exposure = self.risk_settings.max_geographic_exposure;
        
        if *current_exposure >= max_exposure {
            return Err(StablecoinError::RiskLimitExceeded(
                format!("Geographic exposure to {} ({}) exceeds maximum ({})", location, current_exposure, max_exposure)
            ));
        }
        
        Ok(())
    }

    /// Update portfolio metrics
    async fn update_portfolio_metrics(&mut self) -> StablecoinResult<()> {
        let mut total_value = Decimal::ZERO;
        let mut geographic_exposure = HashMap::new();
        let mut sector_exposure = HashMap::new();

        for asset in self.portfolio.assets.values() {
            total_value += asset.current_value;
            
            // Update geographic exposure
            *geographic_exposure.entry(asset.location.clone()).or_insert(Decimal::ZERO) += asset.current_value;
            
            // Update sector exposure
            *sector_exposure.entry(asset.asset_type.clone()).or_insert(Decimal::ZERO) += asset.current_value;
        }

        // Convert to percentages
        if total_value > Decimal::ZERO {
            for (_, exposure) in geographic_exposure.iter_mut() {
                *exposure = (*exposure / total_value) * Decimal::new(100, 0);
            }
            for (_, exposure) in sector_exposure.iter_mut() {
                *exposure = (*exposure / total_value) * Decimal::new(100, 0);
            }
        }

        self.portfolio.total_value = total_value;
        self.portfolio.geographic_exposure = geographic_exposure;
        self.portfolio.sector_exposure = sector_exposure;
        self.portfolio.last_updated = Utc::now();

        Ok(())
    }

    /// Calculate portfolio yield
    async fn calculate_portfolio_yield(&self) -> StablecoinResult<Decimal> {
        let mut weighted_yield = Decimal::ZERO;
        let total_value = self.portfolio.total_value;

        if total_value == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        for asset in self.portfolio.assets.values() {
            let weight = asset.current_value / total_value;
            weighted_yield += weight * asset.annual_yield;
        }

        Ok(weighted_yield)
    }

    /// Rebalance portfolio to target allocations
    async fn rebalance_portfolio(&mut self) -> StablecoinResult<Vec<RebalanceAction>> {
        let mut actions = Vec::new();
        let total_value = self.portfolio.total_value;

        if total_value == Decimal::ZERO {
            return Ok(actions);
        }

        for (asset_id, target_allocation) in &self.portfolio.target_allocations {
            if let Some(asset) = self.portfolio.assets.get(asset_id) {
                let current_allocation = (asset.current_value / total_value) * Decimal::new(100, 0);
                let deviation = (current_allocation - target_allocation).abs();

                // Rebalance if deviation exceeds threshold
                if deviation > self.risk_settings.rebalancing_threshold {
                    let target_value = (target_allocation / Decimal::new(100, 0)) * total_value;
                    let adjustment = target_value - asset.current_value;

                    actions.push(RebalanceAction {
                        asset_id: *asset_id,
                        current_value: asset.current_value,
                        target_value,
                        adjustment,
                        action_type: if adjustment > Decimal::ZERO {
                            RebalanceActionType::Increase
                        } else {
                            RebalanceActionType::Decrease
                        },
                    });
                }
            }
        }

        self.last_rebalancing = Some(Utc::now());
        Ok(actions)
    }
}

#[async_trait]
impl StabilityMechanismTrait for RWABacked {
    fn name(&self) -> &'static str {
        "RWA Backed"
    }

    fn description(&self) -> &'static str {
        "Stablecoin backed by a diversified portfolio of real-world assets"
    }

    fn requires_collateral(&self) -> bool {
        true
    }

    fn min_collateral_ratio(&self) -> Decimal {
        Decimal::new(110, 2) // 110%
    }

    async fn calculate_required_collateral(
        &self,
        _stablecoin: &Stablecoin,
        issuance_amount: Decimal,
    ) -> StablecoinResult<Decimal> {
        let collateral_ratio = self.min_collateral_ratio();
        Ok(issuance_amount * collateral_ratio)
    }

    async fn validate_collateral(
        &self,
        stablecoin: &Stablecoin,
        collateral: &[CollateralPosition],
        issuance_amount: Decimal,
    ) -> StablecoinResult<bool> {
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
        let total_debt = self.portfolio.total_value; // Simplified assumption
        
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
        Ok(collateral_ratio < self.parameters.liquidation_threshold)
    }

    async fn calculate_liquidation_penalty(
        &self,
        _stablecoin: &Stablecoin,
        collateral_value: Decimal,
    ) -> StablecoinResult<Decimal> {
        let penalty_rate = Decimal::new(5, 2); // 5% penalty
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

        // For RWA-backed stablecoins, stability is primarily maintained through:
        // 1. Portfolio rebalancing
        // 2. Yield distribution
        // 3. Collateral adjustments

        if current_price < target_price {
            // Price below target - consider buying back tokens or increasing yield
            let buyback_amount = stablecoin.total_supply * price_deviation * Decimal::new(10, 2); // 10% of deviation
            Ok(StabilityAction::DecreaseSupply { amount: buyback_amount })
        } else {
            // Price above target - consider issuing more tokens or reducing yield
            let issuance_amount = stablecoin.total_supply * price_deviation * Decimal::new(5, 2); // 5% of deviation
            Ok(StabilityAction::IncreaseSupply { amount: issuance_amount })
        }
    }

    async fn update_parameters(&mut self, parameters: StabilityParameters) -> StablecoinResult<()> {
        self.parameters = parameters;
        Ok(())
    }

    async fn get_stability_metrics(&self, stablecoin: &Stablecoin) -> StablecoinResult<StabilityMetrics> {
        let price_deviation = ((stablecoin.current_price - stablecoin.target_price) / stablecoin.target_price).abs();
        let collateral_ratio = stablecoin.collateral_ratio;
        
        // Calculate stability score based on multiple factors
        let mut stability_score = 100u8;
        
        // Penalize for price deviation
        if price_deviation > Decimal::new(5, 2) { // > 5%
            stability_score = stability_score.saturating_sub(30);
        } else if price_deviation > Decimal::new(2, 2) { // > 2%
            stability_score = stability_score.saturating_sub(15);
        }
        
        // Penalize for low collateral ratio
        if collateral_ratio < Decimal::new(120, 2) { // < 120%
            stability_score = stability_score.saturating_sub(25);
        }

        let risk_level = match stability_score {
            90..=100 => RiskLevel::Low,
            70..=89 => RiskLevel::Medium,
            50..=69 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        let time_since_rebalancing = self.last_rebalancing
            .map(|t| (Utc::now() - t).num_seconds() as u64)
            .unwrap_or(0);

        Ok(StabilityMetrics {
            price_deviation,
            collateral_ratio,
            supply_utilization: Decimal::new(75, 2), // Mock value
            stability_score,
            risk_level,
            time_since_rebalancing,
            arbitrage_opportunities: 0, // RWA-backed typically has fewer arbitrage opportunities
        })
    }

    async fn handle_emergency(
        &self,
        stablecoin: &mut Stablecoin,
        emergency_type: EmergencyType,
    ) -> StablecoinResult<EmergencyResponse> {
        match emergency_type {
            EmergencyType::PriceDeviation { deviation } => {
                if deviation > Decimal::new(10, 2) { // > 10%
                    stablecoin.status = crate::types::StablecoinStatus::EmergencyPause;
                    Ok(EmergencyResponse {
                        action: EmergencyAction::PauseAll,
                        reason: format!("Price deviation {} exceeds emergency threshold", deviation),
                        estimated_recovery_time: Some(3600), // 1 hour
                        requires_manual_intervention: true,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::CircuitBreaker { duration: 1800 }, // 30 minutes
                        reason: "Temporary circuit breaker activated".to_string(),
                        estimated_recovery_time: Some(1800),
                        requires_manual_intervention: false,
                    })
                }
            }
            EmergencyType::LowCollateralRatio { ratio } => {
                if ratio < Decimal::new(105, 2) { // < 105%
                    Ok(EmergencyResponse {
                        action: EmergencyAction::PauseIssuance,
                        reason: format!("Collateral ratio {} below emergency threshold", ratio),
                        estimated_recovery_time: Some(7200), // 2 hours
                        requires_manual_intervention: true,
                    })
                } else {
                    Ok(EmergencyResponse {
                        action: EmergencyAction::TriggerLiquidation { positions: vec![] }, // Would be populated with actual positions
                        reason: "Triggering liquidations to restore collateral ratio".to_string(),
                        estimated_recovery_time: Some(3600),
                        requires_manual_intervention: false,
                    })
                }
            }
            _ => {
                Ok(EmergencyResponse {
                    action: EmergencyAction::NotifyAuthorities { 
                        message: format!("Emergency situation detected: {:?}", emergency_type) 
                    },
                    reason: "Unknown emergency type".to_string(),
                    estimated_recovery_time: None,
                    requires_manual_intervention: true,
                })
            }
        }
    }
}

/// RWA portfolio structure
#[derive(Debug, Clone, Default)]
pub struct RWAPortfolio {
    /// Portfolio assets
    pub assets: HashMap<Uuid, RWAAsset>,
    /// Target allocations (percentage)
    pub target_allocations: HashMap<Uuid, Decimal>,
    /// Total portfolio value
    pub total_value: Decimal,
    /// Geographic exposure breakdown
    pub geographic_exposure: HashMap<String, Decimal>,
    /// Sector exposure breakdown
    pub sector_exposure: HashMap<String, Decimal>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// RWA asset definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RWAAsset {
    /// Asset ID
    pub id: Uuid,
    /// Asset type (real_estate, commodities, bonds, etc.)
    pub asset_type: String,
    /// Asset description
    pub description: String,
    /// Current market value
    pub current_value: Decimal,
    /// Annual yield percentage
    pub annual_yield: Decimal,
    /// Liquidity score (0-100)
    pub liquidity_score: u8,
    /// Geographic location
    pub location: String,
    /// Last valuation date
    pub last_valuation: DateTime<Utc>,
    /// Valuation method
    pub valuation_method: String,
}

/// Valuation model for RWA assets
#[derive(Debug, Clone, Default)]
pub struct ValuationModel {
    /// Valuation frequency (in seconds)
    pub valuation_frequency: u64,
    /// Supported valuation methods
    pub supported_methods: Vec<String>,
    /// External valuation providers
    pub valuation_providers: Vec<String>,
}

/// Risk management settings
#[derive(Debug, Clone)]
pub struct RiskSettings {
    /// Supported asset types
    pub supported_asset_types: Vec<String>,
    /// Minimum asset value
    pub min_asset_value: Decimal,
    /// Minimum liquidity score
    pub min_liquidity_score: u8,
    /// Maximum geographic exposure (percentage)
    pub max_geographic_exposure: Decimal,
    /// Maximum sector exposure (percentage)
    pub max_sector_exposure: Decimal,
    /// Rebalancing threshold (percentage)
    pub rebalancing_threshold: Decimal,
}

impl Default for RiskSettings {
    fn default() -> Self {
        Self {
            supported_asset_types: vec![
                "real_estate".to_string(),
                "commodities".to_string(),
                "bonds".to_string(),
                "infrastructure".to_string(),
            ],
            min_asset_value: Decimal::new(100_000, 0), // $100k minimum
            min_liquidity_score: 30, // Minimum 30/100 liquidity score
            max_geographic_exposure: Decimal::new(40, 2), // 40% max per geography
            max_sector_exposure: Decimal::new(50, 2), // 50% max per sector
            rebalancing_threshold: Decimal::new(5, 2), // 5% deviation threshold
        }
    }
}

/// Rebalancing action
#[derive(Debug, Clone)]
pub struct RebalanceAction {
    pub asset_id: Uuid,
    pub current_value: Decimal,
    pub target_value: Decimal,
    pub adjustment: Decimal,
    pub action_type: RebalanceActionType,
}

/// Rebalancing action types
#[derive(Debug, Clone, PartialEq)]
pub enum RebalanceActionType {
    Increase,
    Decrease,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rwa_backed_creation() {
        let parameters = StabilityParameters::default();
        let mechanism = RWABacked::new(parameters);
        
        assert_eq!(mechanism.name(), "RWA Backed");
        assert!(mechanism.requires_collateral());
        assert_eq!(mechanism.min_collateral_ratio(), Decimal::new(110, 2));
    }

    #[tokio::test]
    async fn test_calculate_required_collateral() {
        let parameters = StabilityParameters::default();
        let mechanism = RWABacked::new(parameters);
        let stablecoin = Stablecoin {
            id: Uuid::new_v4(),
            symbol: "RWAUSD".to_string(),
            name: "RWA USD".to_string(),
            decimals: 18,
            stability_mechanism: crate::types::StabilityMechanism::RWABacked,
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
        };

        let required = mechanism.calculate_required_collateral(&stablecoin, Decimal::new(1000, 0)).await.unwrap();
        assert_eq!(required, Decimal::new(1100, 0)); // 1.10 * 1000 = 1100
    }
}
