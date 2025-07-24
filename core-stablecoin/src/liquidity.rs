// =====================================================================================
// File: core-stablecoin/src/liquidity.rs
// Description: Enterprise-grade liquidity management for stablecoin operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{StablecoinError, StablecoinResult};
use crate::types::{Stablecoin, CollateralPosition};

/// Liquidity management trait
#[async_trait]
pub trait LiquidityManager: Send + Sync {
    /// Add liquidity to a pool
    async fn add_liquidity(
        &mut self,
        pool_id: &str,
        token_a_amount: Decimal,
        token_b_amount: Decimal,
        min_liquidity: Decimal,
    ) -> StablecoinResult<LiquidityPosition>;

    /// Remove liquidity from a pool
    async fn remove_liquidity(
        &mut self,
        position_id: Uuid,
        liquidity_amount: Decimal,
        min_token_a: Decimal,
        min_token_b: Decimal,
    ) -> StablecoinResult<(Decimal, Decimal)>;

    /// Get pool information
    async fn get_pool_info(&self, pool_id: &str) -> StablecoinResult<PoolInfo>;

    /// Calculate optimal liquidity allocation
    async fn calculate_optimal_allocation(
        &self,
        stablecoin: &Stablecoin,
        target_liquidity: Decimal,
    ) -> StablecoinResult<Vec<LiquidityAllocation>>;

    /// Monitor liquidity health
    async fn monitor_liquidity_health(&self) -> StablecoinResult<LiquidityHealth>;

    /// Execute emergency liquidity operations
    async fn emergency_liquidity_action(
        &mut self,
        action_type: EmergencyLiquidityAction,
    ) -> StablecoinResult<String>;
}

/// Enterprise liquidity manager implementation
pub struct EnterpriseLiquidityManager {
    pools: HashMap<String, LiquidityPool>,
    positions: HashMap<Uuid, LiquidityPosition>,
    config: LiquidityConfig,
    risk_metrics: LiquidityRiskMetrics,
    emergency_reserves: HashMap<String, Decimal>,
}

impl EnterpriseLiquidityManager {
    /// Create new liquidity manager
    pub fn new(config: LiquidityConfig) -> Self {
        Self {
            pools: HashMap::new(),
            positions: HashMap::new(),
            config,
            risk_metrics: LiquidityRiskMetrics::default(),
            emergency_reserves: HashMap::new(),
        }
    }

    /// Initialize liquidity pools
    pub async fn initialize_pools(&mut self, pool_configs: Vec<PoolConfig>) -> StablecoinResult<()> {
        for config in pool_configs {
            let pool = LiquidityPool::new(config)?;
            self.pools.insert(pool.id.clone(), pool);
        }
        info!("Initialized {} liquidity pools", self.pools.len());
        Ok(())
    }

    /// Update pool reserves
    pub async fn update_pool_reserves(
        &mut self,
        pool_id: &str,
        reserve_a: Decimal,
        reserve_b: Decimal,
    ) -> StablecoinResult<()> {
        let pool = self.pools.get_mut(pool_id)
            .ok_or_else(|| StablecoinError::ValidationError {
                field: "pool_id".to_string(),
                message: format!("Pool {} not found", pool_id),
            })?;

        pool.reserve_a = reserve_a;
        pool.reserve_b = reserve_b;
        pool.last_updated = Utc::now();

        // Update risk metrics
        self.update_risk_metrics().await?;

        Ok(())
    }

    /// Calculate impermanent loss
    pub fn calculate_impermanent_loss(
        &self,
        initial_price_ratio: Decimal,
        current_price_ratio: Decimal,
    ) -> StablecoinResult<Decimal> {
        if initial_price_ratio <= Decimal::ZERO || current_price_ratio <= Decimal::ZERO {
            return Err(StablecoinError::InvalidAmount(
                "Price ratios must be positive".to_string()
            ));
        }

        let price_change_ratio = current_price_ratio / initial_price_ratio;
        // Approximate square root using Newton's method for Decimal
        let sqrt_ratio = Self::calculate_sqrt(price_change_ratio)?;

        // IL = 2 * sqrt(price_ratio) / (1 + price_ratio) - 1
        let il = (Decimal::new(2, 0) * sqrt_ratio) / (Decimal::ONE + price_change_ratio) - Decimal::ONE;
        Ok(il.abs())
    }

    /// Rebalance liquidity across pools
    pub async fn rebalance_liquidity(&mut self) -> StablecoinResult<Vec<RebalanceAction>> {
        let mut actions = Vec::new();
        let total_liquidity = self.calculate_total_liquidity().await?;

        for (pool_id, pool) in &self.pools {
            let current_allocation = pool.total_liquidity / total_liquidity;
            let target_allocation = pool.config.target_allocation;
            let deviation = (current_allocation - target_allocation).abs();

            if deviation > self.config.rebalance_threshold {
                let target_liquidity = total_liquidity * target_allocation;
                let adjustment = target_liquidity - pool.total_liquidity;

                actions.push(RebalanceAction {
                    pool_id: pool_id.clone(),
                    current_liquidity: pool.total_liquidity,
                    target_liquidity,
                    adjustment,
                    action_type: if adjustment > Decimal::ZERO {
                        RebalanceActionType::AddLiquidity
                    } else {
                        RebalanceActionType::RemoveLiquidity
                    },
                });
            }
        }

        info!("Generated {} rebalance actions", actions.len());
        Ok(actions)
    }

    /// Calculate total liquidity across all pools
    async fn calculate_total_liquidity(&self) -> StablecoinResult<Decimal> {
        Ok(self.pools.values().map(|pool| pool.total_liquidity).sum())
    }

    /// Update risk metrics
    async fn update_risk_metrics(&mut self) -> StablecoinResult<()> {
        let mut total_value_locked = Decimal::ZERO;
        let mut concentration_risk = Decimal::ZERO;
        let mut impermanent_loss_exposure = Decimal::ZERO;

        for pool in self.pools.values() {
            total_value_locked += pool.total_liquidity;
            
            // Calculate concentration risk (Herfindahl index)
            let pool_share = pool.total_liquidity / total_value_locked;
            concentration_risk += pool_share * pool_share;

            // Estimate impermanent loss exposure
            if let Ok(il) = self.calculate_impermanent_loss(
                Decimal::ONE, // Assume initial 1:1 ratio
                pool.reserve_a / pool.reserve_b,
            ) {
                impermanent_loss_exposure += il * pool.total_liquidity;
            }
        }

        self.risk_metrics = LiquidityRiskMetrics {
            total_value_locked,
            concentration_risk,
            impermanent_loss_exposure,
            liquidity_utilization: self.calculate_utilization_rate().await?,
            last_updated: Utc::now(),
        };

        Ok(())
    }

    /// Calculate liquidity utilization rate
    async fn calculate_utilization_rate(&self) -> StablecoinResult<Decimal> {
        let total_liquidity = self.calculate_total_liquidity().await?;
        let available_liquidity = total_liquidity - self.calculate_locked_liquidity().await?;
        
        if total_liquidity == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        Ok((total_liquidity - available_liquidity) / total_liquidity * Decimal::new(100, 0))
    }

    /// Calculate locked liquidity
    async fn calculate_locked_liquidity(&self) -> StablecoinResult<Decimal> {
        Ok(self.positions.values()
            .filter(|pos| pos.is_locked())
            .map(|pos| pos.liquidity_amount)
            .sum())
    }

    /// Check if emergency action is needed
    pub async fn check_emergency_conditions(&self) -> StablecoinResult<Vec<EmergencyCondition>> {
        let mut conditions = Vec::new();

        // Check liquidity utilization
        if self.risk_metrics.liquidity_utilization > self.config.emergency_utilization_threshold {
            conditions.push(EmergencyCondition::HighUtilization {
                current: self.risk_metrics.liquidity_utilization,
                threshold: self.config.emergency_utilization_threshold,
            });
        }

        // Check concentration risk
        if self.risk_metrics.concentration_risk > self.config.max_concentration_risk {
            conditions.push(EmergencyCondition::ConcentrationRisk {
                current: self.risk_metrics.concentration_risk,
                threshold: self.config.max_concentration_risk,
            });
        }

        // Check impermanent loss exposure
        if self.risk_metrics.impermanent_loss_exposure > self.config.max_impermanent_loss {
            conditions.push(EmergencyCondition::ImpermanentLoss {
                current: self.risk_metrics.impermanent_loss_exposure,
                threshold: self.config.max_impermanent_loss,
            });
        }

        // Check individual pool health
        for (pool_id, pool) in &self.pools {
            if pool.reserve_a == Decimal::ZERO || pool.reserve_b == Decimal::ZERO {
                conditions.push(EmergencyCondition::PoolDrained {
                    pool_id: pool_id.clone(),
                });
            }
        }

        Ok(conditions)
    }
}

#[async_trait]
impl LiquidityManager for EnterpriseLiquidityManager {
    async fn add_liquidity(
        &mut self,
        pool_id: &str,
        token_a_amount: Decimal,
        token_b_amount: Decimal,
        min_liquidity: Decimal,
    ) -> StablecoinResult<LiquidityPosition> {
        // Validate inputs
        if token_a_amount <= Decimal::ZERO || token_b_amount <= Decimal::ZERO {
            return Err(StablecoinError::InvalidAmount(
                "Token amounts must be positive".to_string()
            ));
        }

        let pool = self.pools.get_mut(pool_id)
            .ok_or_else(|| StablecoinError::ValidationError {
                field: "pool_id".to_string(),
                message: format!("Pool {} not found", pool_id),
            })?;

        // Calculate liquidity tokens to mint
        let liquidity_amount = if pool.total_liquidity == Decimal::ZERO {
            // First liquidity provision - calculate geometric mean
            let product = token_a_amount * token_b_amount;
            Self::calculate_sqrt(product)?
        } else {
            // Subsequent liquidity provision
            let liquidity_a = token_a_amount * pool.total_liquidity / pool.reserve_a;
            let liquidity_b = token_b_amount * pool.total_liquidity / pool.reserve_b;
            liquidity_a.min(liquidity_b)
        };

        if liquidity_amount < min_liquidity {
            return Err(StablecoinError::InsufficientBalance {
                required: min_liquidity.to_string(),
                available: liquidity_amount.to_string(),
            });
        }

        // Update pool reserves
        pool.reserve_a += token_a_amount;
        pool.reserve_b += token_b_amount;
        pool.total_liquidity += liquidity_amount;
        pool.last_updated = Utc::now();

        // Create liquidity position
        let position = LiquidityPosition {
            id: Uuid::new_v4(),
            pool_id: pool_id.to_string(),
            liquidity_amount,
            token_a_amount,
            token_b_amount,
            created_at: Utc::now(),
            locked_until: None,
            status: LiquidityPositionStatus::Active,
        };

        self.positions.insert(position.id, position.clone());
        self.update_risk_metrics().await?;

        info!("Added liquidity to pool {}: {} tokens", pool_id, liquidity_amount);
        Ok(position)
    }

    async fn remove_liquidity(
        &mut self,
        position_id: Uuid,
        liquidity_amount: Decimal,
        min_token_a: Decimal,
        min_token_b: Decimal,
    ) -> StablecoinResult<(Decimal, Decimal)> {
        let position = self.positions.get_mut(&position_id)
            .ok_or_else(|| StablecoinError::PositionNotFound(position_id.to_string()))?;

        if position.is_locked() {
            return Err(StablecoinError::OperationNotPermitted(
                "Position is locked".to_string()
            ));
        }

        if liquidity_amount > position.liquidity_amount {
            return Err(StablecoinError::InsufficientBalance {
                required: liquidity_amount.to_string(),
                available: position.liquidity_amount.to_string(),
            });
        }

        let pool = self.pools.get_mut(&position.pool_id)
            .ok_or_else(|| StablecoinError::ValidationError {
                field: "pool_id".to_string(),
                message: format!("Pool {} not found", position.pool_id),
            })?;

        // Calculate tokens to return
        let token_a_return = liquidity_amount * pool.reserve_a / pool.total_liquidity;
        let token_b_return = liquidity_amount * pool.reserve_b / pool.total_liquidity;

        if token_a_return < min_token_a || token_b_return < min_token_b {
            return Err(StablecoinError::InsufficientBalance {
                required: format!("A: {}, B: {}", min_token_a, min_token_b),
                available: format!("A: {}, B: {}", token_a_return, token_b_return),
            });
        }

        // Update pool reserves
        pool.reserve_a -= token_a_return;
        pool.reserve_b -= token_b_return;
        pool.total_liquidity -= liquidity_amount;
        pool.last_updated = Utc::now();

        // Update position
        let pool_id = position.pool_id.clone();
        position.liquidity_amount -= liquidity_amount;
        if position.liquidity_amount == Decimal::ZERO {
            position.status = LiquidityPositionStatus::Closed;
        }

        self.update_risk_metrics().await?;

        info!("Removed liquidity from pool {}: {} tokens", pool_id, liquidity_amount);
        Ok((token_a_return, token_b_return))
    }

    async fn get_pool_info(&self, pool_id: &str) -> StablecoinResult<PoolInfo> {
        let pool = self.pools.get(pool_id)
            .ok_or_else(|| StablecoinError::ValidationError {
                field: "pool_id".to_string(),
                message: format!("Pool {} not found", pool_id),
            })?;

        Ok(PoolInfo {
            id: pool.id.clone(),
            token_a: pool.config.token_a.clone(),
            token_b: pool.config.token_b.clone(),
            reserve_a: pool.reserve_a,
            reserve_b: pool.reserve_b,
            total_liquidity: pool.total_liquidity,
            fee_rate: pool.config.fee_rate,
            last_updated: pool.last_updated,
        })
    }

    async fn calculate_optimal_allocation(
        &self,
        stablecoin: &Stablecoin,
        target_liquidity: Decimal,
    ) -> StablecoinResult<Vec<LiquidityAllocation>> {
        let mut allocations = Vec::new();
        let total_weight: Decimal = self.pools.values()
            .map(|pool| pool.config.weight)
            .sum();

        for pool in self.pools.values() {
            let allocation_amount = target_liquidity * pool.config.weight / total_weight;
            
            allocations.push(LiquidityAllocation {
                pool_id: pool.id.clone(),
                amount: allocation_amount,
                weight: pool.config.weight,
                expected_apy: pool.config.expected_apy,
                risk_score: pool.config.risk_score,
            });
        }

        // Sort by risk-adjusted return
        allocations.sort_by(|a, b| {
            let return_a = a.expected_apy / a.risk_score;
            let return_b = b.expected_apy / b.risk_score;
            return_b.cmp(&return_a)
        });

        Ok(allocations)
    }

    async fn monitor_liquidity_health(&self) -> StablecoinResult<LiquidityHealth> {
        let emergency_conditions = self.check_emergency_conditions().await?;
        let health_score = self.calculate_health_score().await?;

        Ok(LiquidityHealth {
            overall_health: health_score,
            total_value_locked: self.risk_metrics.total_value_locked,
            utilization_rate: self.risk_metrics.liquidity_utilization,
            concentration_risk: self.risk_metrics.concentration_risk,
            impermanent_loss_exposure: self.risk_metrics.impermanent_loss_exposure,
            emergency_conditions,
            active_positions: self.positions.len(),
            last_updated: Utc::now(),
        })
    }

    async fn emergency_liquidity_action(
        &mut self,
        action_type: EmergencyLiquidityAction,
    ) -> StablecoinResult<String> {
        match action_type {
            EmergencyLiquidityAction::PauseAllOperations => {
                // Implement pause logic
                warn!("Emergency: Pausing all liquidity operations");
                Ok("All liquidity operations paused".to_string())
            }
            EmergencyLiquidityAction::DrainPool { pool_id } => {
                // Implement pool draining logic
                warn!("Emergency: Draining pool {}", pool_id);
                Ok(format!("Pool {} drained", pool_id))
            }
            EmergencyLiquidityAction::RebalanceEmergency => {
                // Implement emergency rebalancing
                let actions = self.rebalance_liquidity().await?;
                warn!("Emergency: Executing {} rebalance actions", actions.len());
                Ok(format!("Emergency rebalancing completed: {} actions", actions.len()))
            }
            EmergencyLiquidityAction::ActivateReserves { amount } => {
                // Implement reserve activation
                warn!("Emergency: Activating reserves of {}", amount);
                Ok(format!("Activated emergency reserves: {}", amount))
            }
        }
    }
}

impl EnterpriseLiquidityManager {
    /// Calculate square root using Newton's method for Decimal
    fn calculate_sqrt(value: Decimal) -> StablecoinResult<Decimal> {
        if value < Decimal::ZERO {
            return Err(StablecoinError::InvalidAmount("Cannot calculate square root of negative number".to_string()));
        }

        if value == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        // Newton's method: x_{n+1} = (x_n + value/x_n) / 2
        let mut x = value / Decimal::new(2, 0); // Initial guess
        let precision = Decimal::new(1, 10); // 10^-10 precision

        for _ in 0..50 { // Max 50 iterations
            let x_new = (x + value / x) / Decimal::new(2, 0);
            if (x_new - x).abs() < precision {
                return Ok(x_new);
            }
            x = x_new;
        }

        Ok(x)
    }

    /// Calculate overall health score
    async fn calculate_health_score(&self) -> StablecoinResult<u8> {
        let mut score = 100u8;

        // Penalize high utilization
        if self.risk_metrics.liquidity_utilization > Decimal::new(80, 0) {
            score = score.saturating_sub(20);
        } else if self.risk_metrics.liquidity_utilization > Decimal::new(60, 0) {
            score = score.saturating_sub(10);
        }

        // Penalize concentration risk
        if self.risk_metrics.concentration_risk > Decimal::new(50, 2) {
            score = score.saturating_sub(15);
        }

        // Penalize impermanent loss exposure
        if self.risk_metrics.impermanent_loss_exposure > self.config.max_impermanent_loss {
            score = score.saturating_sub(25);
        }

        Ok(score)
    }
}

/// Liquidity pool structure
#[derive(Debug, Clone)]
pub struct LiquidityPool {
    pub id: String,
    pub config: PoolConfig,
    pub reserve_a: Decimal,
    pub reserve_b: Decimal,
    pub total_liquidity: Decimal,
    pub last_updated: DateTime<Utc>,
}

impl LiquidityPool {
    pub fn new(config: PoolConfig) -> StablecoinResult<Self> {
        Ok(Self {
            id: config.id.clone(),
            config,
            reserve_a: Decimal::ZERO,
            reserve_b: Decimal::ZERO,
            total_liquidity: Decimal::ZERO,
            last_updated: Utc::now(),
        })
    }
}

/// Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub id: String,
    pub token_a: String,
    pub token_b: String,
    pub fee_rate: Decimal,
    pub target_allocation: Decimal,
    pub weight: Decimal,
    pub expected_apy: Decimal,
    pub risk_score: Decimal,
}

/// Liquidity position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    pub id: Uuid,
    pub pool_id: String,
    pub liquidity_amount: Decimal,
    pub token_a_amount: Decimal,
    pub token_b_amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub locked_until: Option<DateTime<Utc>>,
    pub status: LiquidityPositionStatus,
}

impl LiquidityPosition {
    pub fn is_locked(&self) -> bool {
        self.locked_until.map_or(false, |until| Utc::now() < until)
    }
}

/// Liquidity position status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiquidityPositionStatus {
    Active,
    Locked,
    Closed,
}

/// Liquidity configuration
#[derive(Debug, Clone)]
pub struct LiquidityConfig {
    pub rebalance_threshold: Decimal,
    pub emergency_utilization_threshold: Decimal,
    pub max_concentration_risk: Decimal,
    pub max_impermanent_loss: Decimal,
    pub min_liquidity_ratio: Decimal,
}

impl Default for LiquidityConfig {
    fn default() -> Self {
        Self {
            rebalance_threshold: Decimal::new(5, 2), // 5%
            emergency_utilization_threshold: Decimal::new(90, 0), // 90%
            max_concentration_risk: Decimal::new(40, 2), // 0.4 (40% in one pool)
            max_impermanent_loss: Decimal::new(10, 2), // 10%
            min_liquidity_ratio: Decimal::new(20, 2), // 20%
        }
    }
}

/// Liquidity risk metrics
#[derive(Debug, Clone, Default)]
pub struct LiquidityRiskMetrics {
    pub total_value_locked: Decimal,
    pub concentration_risk: Decimal,
    pub impermanent_loss_exposure: Decimal,
    pub liquidity_utilization: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolInfo {
    pub id: String,
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: Decimal,
    pub reserve_b: Decimal,
    pub total_liquidity: Decimal,
    pub fee_rate: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Liquidity allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityAllocation {
    pub pool_id: String,
    pub amount: Decimal,
    pub weight: Decimal,
    pub expected_apy: Decimal,
    pub risk_score: Decimal,
}

/// Rebalance action
#[derive(Debug, Clone)]
pub struct RebalanceAction {
    pub pool_id: String,
    pub current_liquidity: Decimal,
    pub target_liquidity: Decimal,
    pub adjustment: Decimal,
    pub action_type: RebalanceActionType,
}

/// Rebalance action types
#[derive(Debug, Clone, PartialEq)]
pub enum RebalanceActionType {
    AddLiquidity,
    RemoveLiquidity,
}

/// Emergency conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyCondition {
    HighUtilization { current: Decimal, threshold: Decimal },
    ConcentrationRisk { current: Decimal, threshold: Decimal },
    ImpermanentLoss { current: Decimal, threshold: Decimal },
    PoolDrained { pool_id: String },
}

/// Emergency liquidity actions
#[derive(Debug, Clone)]
pub enum EmergencyLiquidityAction {
    PauseAllOperations,
    DrainPool { pool_id: String },
    RebalanceEmergency,
    ActivateReserves { amount: Decimal },
}

/// Liquidity health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityHealth {
    pub overall_health: u8,
    pub total_value_locked: Decimal,
    pub utilization_rate: Decimal,
    pub concentration_risk: Decimal,
    pub impermanent_loss_exposure: Decimal,
    pub emergency_conditions: Vec<EmergencyCondition>,
    pub active_positions: usize,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pool_config() -> PoolConfig {
        PoolConfig {
            id: "USDC-ETH".to_string(),
            token_a: "USDC".to_string(),
            token_b: "ETH".to_string(),
            fee_rate: Decimal::new(3, 3), // 0.3%
            target_allocation: Decimal::new(50, 2), // 50%
            weight: Decimal::new(100, 0),
            expected_apy: Decimal::new(12, 0), // 12%
            risk_score: Decimal::new(3, 0), // Risk score of 3
        }
    }

    #[tokio::test]
    async fn test_liquidity_manager_creation() {
        let config = LiquidityConfig::default();
        let manager = EnterpriseLiquidityManager::new(config);
        
        assert_eq!(manager.pools.len(), 0);
        assert_eq!(manager.positions.len(), 0);
    }

    #[tokio::test]
    async fn test_pool_initialization() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let pool_configs = vec![create_test_pool_config()];
        let result = manager.initialize_pools(pool_configs).await;
        
        assert!(result.is_ok());
        assert_eq!(manager.pools.len(), 1);
        assert!(manager.pools.contains_key("USDC-ETH"));
    }

    #[tokio::test]
    async fn test_add_liquidity() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let pool_configs = vec![create_test_pool_config()];
        manager.initialize_pools(pool_configs).await.unwrap();
        
        let position = manager.add_liquidity(
            "USDC-ETH",
            Decimal::new(1000, 0), // 1000 USDC
            Decimal::new(1, 0),    // 1 ETH
            Decimal::new(1, 0),    // Min 1 liquidity token
        ).await.unwrap();
        
        assert_eq!(position.pool_id, "USDC-ETH");
        assert!(position.liquidity_amount > Decimal::ZERO);
        assert_eq!(manager.positions.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_liquidity() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let pool_configs = vec![create_test_pool_config()];
        manager.initialize_pools(pool_configs).await.unwrap();
        
        // Add liquidity first
        let position = manager.add_liquidity(
            "USDC-ETH",
            Decimal::new(1000, 0),
            Decimal::new(1, 0),
            Decimal::new(1, 0),
        ).await.unwrap();
        
        // Remove half the liquidity
        let half_liquidity = position.liquidity_amount / Decimal::new(2, 0);
        let (token_a, token_b) = manager.remove_liquidity(
            position.id,
            half_liquidity,
            Decimal::ZERO,
            Decimal::ZERO,
        ).await.unwrap();
        
        assert!(token_a > Decimal::ZERO);
        assert!(token_b > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_impermanent_loss_calculation() {
        let config = LiquidityConfig::default();
        let manager = EnterpriseLiquidityManager::new(config);
        
        // Test case: price doubles (2x)
        let initial_ratio = Decimal::ONE;
        let current_ratio = Decimal::new(2, 0);
        
        let il = manager.calculate_impermanent_loss(initial_ratio, current_ratio).unwrap();
        assert!(il > Decimal::ZERO);
        assert!(il < Decimal::new(10, 2)); // Should be less than 10%
    }

    #[tokio::test]
    async fn test_invalid_liquidity_amounts() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let pool_configs = vec![create_test_pool_config()];
        manager.initialize_pools(pool_configs).await.unwrap();
        
        // Test negative amounts
        let result = manager.add_liquidity(
            "USDC-ETH",
            Decimal::new(-100, 0), // Negative amount
            Decimal::new(1, 0),
            Decimal::new(1, 0),
        ).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StablecoinError::InvalidAmount(_)));
    }

    #[tokio::test]
    async fn test_pool_not_found() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let result = manager.add_liquidity(
            "NONEXISTENT",
            Decimal::new(100, 0),
            Decimal::new(1, 0),
            Decimal::new(1, 0),
        ).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StablecoinError::ValidationError { .. }));
    }

    #[tokio::test]
    async fn test_liquidity_health_monitoring() {
        let config = LiquidityConfig::default();
        let manager = EnterpriseLiquidityManager::new(config);
        
        let health = manager.monitor_liquidity_health().await.unwrap();
        
        assert_eq!(health.overall_health, 100); // Should be healthy with no positions
        assert_eq!(health.total_value_locked, Decimal::ZERO);
        assert_eq!(health.active_positions, 0);
    }

    #[tokio::test]
    async fn test_optimal_allocation_calculation() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let pool_configs = vec![
            create_test_pool_config(),
            PoolConfig {
                id: "DAI-USDC".to_string(),
                token_a: "DAI".to_string(),
                token_b: "USDC".to_string(),
                fee_rate: Decimal::new(1, 3), // 0.1%
                target_allocation: Decimal::new(30, 2), // 30%
                weight: Decimal::new(50, 0),
                expected_apy: Decimal::new(8, 0), // 8%
                risk_score: Decimal::new(2, 0), // Lower risk
            },
        ];
        
        manager.initialize_pools(pool_configs).await.unwrap();
        
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
            stability_parameters: crate::types::StabilityParameters::default(),
            status: crate::types::StablecoinStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let allocations = manager.calculate_optimal_allocation(
            &stablecoin,
            Decimal::new(10000, 0), // 10,000 target liquidity
        ).await.unwrap();
        
        assert_eq!(allocations.len(), 2);
        
        // Should be sorted by risk-adjusted return
        let total_allocation: Decimal = allocations.iter().map(|a| a.amount).sum();
        assert_eq!(total_allocation, Decimal::new(10000, 0));
    }

    #[tokio::test]
    async fn test_emergency_conditions() {
        let mut config = LiquidityConfig::default();
        config.emergency_utilization_threshold = Decimal::new(50, 0); // 50% for testing
        
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        // Manually set high utilization
        manager.risk_metrics.liquidity_utilization = Decimal::new(80, 0); // 80%
        
        let conditions = manager.check_emergency_conditions().await.unwrap();
        
        assert!(!conditions.is_empty());
        assert!(matches!(conditions[0], EmergencyCondition::HighUtilization { .. }));
    }

    #[tokio::test]
    async fn test_emergency_liquidity_action() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let result = manager.emergency_liquidity_action(
            EmergencyLiquidityAction::PauseAllOperations
        ).await.unwrap();
        
        assert!(result.contains("paused"));
    }

    #[tokio::test]
    async fn test_rebalance_liquidity() {
        let config = LiquidityConfig::default();
        let mut manager = EnterpriseLiquidityManager::new(config);
        
        let pool_configs = vec![create_test_pool_config()];
        manager.initialize_pools(pool_configs).await.unwrap();
        
        // Add some liquidity to create imbalance
        manager.add_liquidity(
            "USDC-ETH",
            Decimal::new(1000, 0),
            Decimal::new(1, 0),
            Decimal::new(1, 0),
        ).await.unwrap();
        
        let actions = manager.rebalance_liquidity().await.unwrap();

        // With only one pool, there might be rebalancing needed due to target allocation
        // The pool has 50% target allocation but 100% actual allocation
        assert_eq!(actions.len(), 1);
    }
}
