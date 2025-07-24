// =====================================================================================
// File: core-stablecoin/src/stability.rs
// Description: Price stability maintenance and arbitrage systems
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use crate::{StablecoinResult, StablecoinError};
use crate::types::{Stablecoin, CollateralPosition};

/// Stability action types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StabilityAction {
    /// Mint new tokens to increase supply
    MintTokens { amount: Decimal },
    /// Burn tokens to decrease supply
    BurnTokens { amount: Decimal },
    /// Adjust interest rates
    AdjustInterestRate { new_rate: Decimal },
    /// Rebalance collateral
    RebalanceCollateral { target_ratio: Decimal },
    /// Execute arbitrage trade
    ExecuteArbitrage { opportunity: ArbitrageOpportunity },
    /// Adjust stability fees
    AdjustStabilityFee { new_fee: Decimal },
    /// Emergency pause
    EmergencyPause,
}

/// Stability trigger conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StabilityTrigger {
    /// Price deviation exceeds threshold
    PriceDeviation { current: Decimal, threshold: Decimal },
    /// Collateral ratio below minimum
    LowCollateralRatio { current: Decimal, minimum: Decimal },
    /// High volatility detected
    HighVolatility { volatility: Decimal },
    /// Large redemption pressure
    RedemptionPressure { amount: Decimal },
    /// Market manipulation detected
    MarketManipulation,
}

/// Stability service trait
#[async_trait]
pub trait StabilityService: Send + Sync {
    /// Monitor price stability
    async fn monitor_stability(&self, stablecoin_id: Uuid) -> StablecoinResult<StabilityStatus>;

    /// Execute stability actions
    async fn execute_stability_action(&self, stablecoin_id: Uuid, action: StabilityAction) -> StablecoinResult<StabilityActionResult>;

    /// Get current stability metrics
    async fn get_stability_metrics(&self, stablecoin_id: Uuid) -> StablecoinResult<StabilityMetrics>;

    /// Get stability history
    async fn get_stability_history(&self, stablecoin_id: Uuid, limit: Option<usize>) -> StablecoinResult<Vec<StabilityEvent>>;

    /// Set stability parameters
    async fn set_stability_parameters(&self, stablecoin_id: Uuid, params: StabilityParameters) -> StablecoinResult<()>;

    /// Get recommended actions
    async fn get_recommended_actions(&self, stablecoin_id: Uuid) -> StablecoinResult<Vec<StabilityAction>>;
}

/// Stability parameters configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityParameters {
    /// Maximum allowed price deviation (percentage)
    pub max_price_deviation: Decimal,
    /// Target collateral ratio
    pub target_collateral_ratio: Decimal,
    /// Minimum collateral ratio before action
    pub min_collateral_ratio: Decimal,
    /// Stability fee rate
    pub stability_fee_rate: Decimal,
    /// Interest rate adjustment step
    pub interest_rate_step: Decimal,
    /// Maximum daily mint/burn amount
    pub max_daily_adjustment: Decimal,
    /// Volatility threshold
    pub volatility_threshold: Decimal,
    /// Action cooldown period (seconds)
    pub action_cooldown: u64,
}

impl Default for StabilityParameters {
    fn default() -> Self {
        Self {
            max_price_deviation: Decimal::new(2, 2), // 2%
            target_collateral_ratio: Decimal::new(150, 2), // 150%
            min_collateral_ratio: Decimal::new(120, 2), // 120%
            stability_fee_rate: Decimal::new(5, 3), // 0.5%
            interest_rate_step: Decimal::new(25, 4), // 0.25%
            max_daily_adjustment: Decimal::new(1_000_000, 0), // 1M tokens
            volatility_threshold: Decimal::new(5, 2), // 5%
            action_cooldown: 300, // 5 minutes
        }
    }
}

/// Enterprise price stabilizer implementation
pub struct EnterprisePriceStabilizer {
    parameters: Arc<Mutex<HashMap<Uuid, StabilityParameters>>>,
    stability_history: Arc<Mutex<HashMap<Uuid, Vec<StabilityEvent>>>>,
    last_actions: Arc<Mutex<HashMap<Uuid, DateTime<Utc>>>>,
    price_feeds: Arc<Mutex<HashMap<Uuid, PriceFeed>>>,
    collateral_monitors: Arc<Mutex<HashMap<Uuid, CollateralMonitor>>>,
}

impl EnterprisePriceStabilizer {
    pub fn new() -> Self {
        Self {
            parameters: Arc::new(Mutex::new(HashMap::new())),
            stability_history: Arc::new(Mutex::new(HashMap::new())),
            last_actions: Arc::new(Mutex::new(HashMap::new())),
            price_feeds: Arc::new(Mutex::new(HashMap::new())),
            collateral_monitors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize monitoring for a stablecoin
    pub async fn initialize_monitoring(&self, stablecoin_id: Uuid, params: StabilityParameters) -> StablecoinResult<()> {
        {
            let mut parameters = self.parameters.lock().await;
            parameters.insert(stablecoin_id, params);
        }

        {
            let mut history = self.stability_history.lock().await;
            history.insert(stablecoin_id, Vec::new());
        }

        {
            let mut feeds = self.price_feeds.lock().await;
            feeds.insert(stablecoin_id, PriceFeed::new(stablecoin_id));
        }

        {
            let mut monitors = self.collateral_monitors.lock().await;
            monitors.insert(stablecoin_id, CollateralMonitor::new(stablecoin_id));
        }

        Ok(())
    }

    /// Check if action is allowed (cooldown period)
    async fn is_action_allowed(&self, stablecoin_id: Uuid) -> StablecoinResult<bool> {
        let last_actions = self.last_actions.lock().await;
        let parameters = self.parameters.lock().await;

        if let (Some(last_action), Some(params)) = (last_actions.get(&stablecoin_id), parameters.get(&stablecoin_id)) {
            let cooldown_duration = Duration::seconds(params.action_cooldown as i64);
            let now = Utc::now();
            Ok(now - *last_action > cooldown_duration)
        } else {
            Ok(true)
        }
    }

    /// Record stability action
    async fn record_action(&self, stablecoin_id: Uuid, action: StabilityAction, result: &StabilityActionResult) -> StablecoinResult<()> {
        let event = StabilityEvent {
            id: Uuid::new_v4(),
            stablecoin_id,
            action: action.clone(),
            trigger: result.trigger.clone(),
            result: result.clone(),
            timestamp: Utc::now(),
        };

        {
            let mut history = self.stability_history.lock().await;
            history.entry(stablecoin_id)
                .or_insert_with(Vec::new)
                .push(event);
        }

        {
            let mut last_actions = self.last_actions.lock().await;
            last_actions.insert(stablecoin_id, Utc::now());
        }

        Ok(())
    }
}

impl Default for EnterprisePriceStabilizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Price feed for monitoring market prices
#[derive(Debug, Clone)]
pub struct PriceFeed {
    stablecoin_id: Uuid,
    current_price: Decimal,
    target_price: Decimal,
    price_history: Vec<PricePoint>,
    last_update: DateTime<Utc>,
}

impl PriceFeed {
    pub fn new(stablecoin_id: Uuid) -> Self {
        Self {
            stablecoin_id,
            current_price: Decimal::new(100, 2), // $1.00
            target_price: Decimal::new(100, 2), // $1.00
            price_history: Vec::new(),
            last_update: Utc::now(),
        }
    }

    pub fn update_price(&mut self, price: Decimal) {
        self.price_history.push(PricePoint {
            price: self.current_price,
            timestamp: self.last_update,
        });

        // Keep only last 100 price points
        if self.price_history.len() > 100 {
            self.price_history.remove(0);
        }

        self.current_price = price;
        self.last_update = Utc::now();
    }

    pub fn get_price_deviation(&self) -> Decimal {
        if self.target_price.is_zero() {
            return Decimal::ZERO;
        }

        ((self.current_price - self.target_price) / self.target_price).abs() * Decimal::new(100, 0)
    }

    pub fn calculate_volatility(&self) -> Decimal {
        if self.price_history.len() < 2 {
            return Decimal::ZERO;
        }

        let prices: Vec<Decimal> = self.price_history.iter().map(|p| p.price).collect();
        let mean = prices.iter().sum::<Decimal>() / Decimal::new(prices.len() as i64, 0);

        let variance = prices.iter()
            .map(|price| (*price - mean) * (*price - mean))
            .sum::<Decimal>() / Decimal::new(prices.len() as i64, 0);

        // Simplified volatility calculation (would use proper statistical methods in production)
        // Using a simple approximation since Decimal doesn't have sqrt
        if variance > Decimal::ZERO {
            // Approximate square root using Newton's method (simplified)
            let mut x = variance / Decimal::new(2, 0);
            for _ in 0..5 { // 5 iterations should be enough for approximation
                if x.is_zero() { break; }
                x = (x + variance / x) / Decimal::new(2, 0);
            }
            x * Decimal::new(100, 0)
        } else {
            Decimal::ZERO
        }
    }
}

/// Collateral monitor for tracking collateral health
#[derive(Debug, Clone)]
pub struct CollateralMonitor {
    stablecoin_id: Uuid,
    total_collateral_value: Decimal,
    total_issued_tokens: Decimal,
    collateral_positions: Vec<CollateralPosition>,
    last_update: DateTime<Utc>,
}

impl CollateralMonitor {
    pub fn new(stablecoin_id: Uuid) -> Self {
        Self {
            stablecoin_id,
            total_collateral_value: Decimal::ZERO,
            total_issued_tokens: Decimal::ZERO,
            collateral_positions: Vec::new(),
            last_update: Utc::now(),
        }
    }

    pub fn update_collateral(&mut self, positions: Vec<CollateralPosition>, issued_tokens: Decimal) {
        self.collateral_positions = positions;
        self.total_collateral_value = self.collateral_positions.iter()
            .map(|p| p.value_usd)
            .sum();
        self.total_issued_tokens = issued_tokens;
        self.last_update = Utc::now();
    }

    pub fn get_collateral_ratio(&self) -> Decimal {
        if self.total_issued_tokens.is_zero() {
            return Decimal::new(1000, 0); // 1000% if no tokens issued
        }

        (self.total_collateral_value / self.total_issued_tokens) * Decimal::new(100, 0)
    }

    pub fn is_undercollateralized(&self, min_ratio: Decimal) -> bool {
        self.get_collateral_ratio() < min_ratio
    }
}

#[async_trait]
impl StabilityService for EnterprisePriceStabilizer {
    async fn monitor_stability(&self, stablecoin_id: Uuid) -> StablecoinResult<StabilityStatus> {
        let parameters = self.parameters.lock().await;
        let price_feeds = self.price_feeds.lock().await;
        let collateral_monitors = self.collateral_monitors.lock().await;

        let params = parameters.get(&stablecoin_id)
            .ok_or(StablecoinError::InvalidRequest("Stablecoin not initialized".to_string()))?;

        let price_feed = price_feeds.get(&stablecoin_id)
            .ok_or(StablecoinError::InvalidRequest("Price feed not found".to_string()))?;

        let collateral_monitor = collateral_monitors.get(&stablecoin_id)
            .ok_or(StablecoinError::InvalidRequest("Collateral monitor not found".to_string()))?;

        let price_deviation = price_feed.get_price_deviation();
        let volatility = price_feed.calculate_volatility();
        let collateral_ratio = collateral_monitor.get_collateral_ratio();

        let mut triggers = Vec::new();
        let mut is_stable = true;

        // Check price deviation
        if price_deviation > params.max_price_deviation {
            triggers.push(StabilityTrigger::PriceDeviation {
                current: price_deviation,
                threshold: params.max_price_deviation,
            });
            is_stable = false;
        }

        // Check collateral ratio
        if collateral_ratio < params.min_collateral_ratio {
            triggers.push(StabilityTrigger::LowCollateralRatio {
                current: collateral_ratio,
                minimum: params.min_collateral_ratio,
            });
            is_stable = false;
        }

        // Check volatility
        if volatility > params.volatility_threshold {
            triggers.push(StabilityTrigger::HighVolatility { volatility });
            is_stable = false;
        }

        Ok(StabilityStatus {
            is_stable,
            triggers,
            current_price: price_feed.current_price,
            price_deviation,
            collateral_ratio,
            volatility,
            last_check: Utc::now(),
        })
    }

    async fn execute_stability_action(&self, stablecoin_id: Uuid, action: StabilityAction) -> StablecoinResult<StabilityActionResult> {
        // Check if action is allowed (cooldown)
        if !self.is_action_allowed(stablecoin_id).await? {
            return Err(StablecoinError::InvalidRequest("Action cooldown period not elapsed".to_string()));
        }

        let status = self.monitor_stability(stablecoin_id).await?;
        let trigger = if !status.triggers.is_empty() {
            Some(status.triggers[0].clone())
        } else {
            None
        };

        let result = match action.clone() {
            StabilityAction::MintTokens { amount } => {
                // Simulate minting tokens
                StabilityActionResult {
                    action: action.clone(),
                    success: true,
                    trigger,
                    details: format!("Minted {} tokens to increase supply", amount),
                    gas_used: Some(150_000),
                    transaction_hash: Some(format!("mint_tx_{}", Uuid::new_v4().simple())),
                    timestamp: Utc::now(),
                }
            },
            StabilityAction::BurnTokens { amount } => {
                // Simulate burning tokens
                StabilityActionResult {
                    action: action.clone(),
                    success: true,
                    trigger,
                    details: format!("Burned {} tokens to decrease supply", amount),
                    gas_used: Some(120_000),
                    transaction_hash: Some(format!("burn_tx_{}", Uuid::new_v4().simple())),
                    timestamp: Utc::now(),
                }
            },
            StabilityAction::AdjustInterestRate { new_rate } => {
                // Simulate interest rate adjustment
                StabilityActionResult {
                    action: action.clone(),
                    success: true,
                    trigger,
                    details: format!("Adjusted interest rate to {}%", new_rate),
                    gas_used: Some(80_000),
                    transaction_hash: Some(format!("rate_tx_{}", Uuid::new_v4().simple())),
                    timestamp: Utc::now(),
                }
            },
            StabilityAction::RebalanceCollateral { target_ratio } => {
                // Simulate collateral rebalancing
                StabilityActionResult {
                    action: action.clone(),
                    success: true,
                    trigger,
                    details: format!("Rebalanced collateral to {}% ratio", target_ratio),
                    gas_used: Some(200_000),
                    transaction_hash: Some(format!("rebalance_tx_{}", Uuid::new_v4().simple())),
                    timestamp: Utc::now(),
                }
            },
            StabilityAction::ExecuteArbitrage { opportunity } => {
                // Simulate arbitrage execution
                StabilityActionResult {
                    action: action.clone(),
                    success: true,
                    trigger,
                    details: format!("Executed arbitrage between {} and {}", opportunity.exchange_a, opportunity.exchange_b),
                    gas_used: Some(300_000),
                    transaction_hash: Some(format!("arb_tx_{}", Uuid::new_v4().simple())),
                    timestamp: Utc::now(),
                }
            },
            StabilityAction::AdjustStabilityFee { new_fee } => {
                // Simulate stability fee adjustment
                StabilityActionResult {
                    action: action.clone(),
                    success: true,
                    trigger,
                    details: format!("Adjusted stability fee to {}%", new_fee),
                    gas_used: Some(90_000),
                    transaction_hash: Some(format!("fee_tx_{}", Uuid::new_v4().simple())),
                    timestamp: Utc::now(),
                }
            },
            StabilityAction::EmergencyPause => {
                // Simulate emergency pause
                StabilityActionResult {
                    action: action.clone(),
                    success: true,
                    trigger,
                    details: "Emergency pause activated".to_string(),
                    gas_used: Some(100_000),
                    transaction_hash: Some(format!("pause_tx_{}", Uuid::new_v4().simple())),
                    timestamp: Utc::now(),
                }
            },
        };

        // Record the action
        self.record_action(stablecoin_id, action, &result).await?;

        Ok(result)
    }

    async fn get_stability_metrics(&self, stablecoin_id: Uuid) -> StablecoinResult<StabilityMetrics> {
        let status = self.monitor_stability(stablecoin_id).await?;
        let history = self.stability_history.lock().await;

        let empty_vec = Vec::new();
        let events = history.get(&stablecoin_id).unwrap_or(&empty_vec);
        let last_action = events.last().map(|e| e.timestamp).unwrap_or_else(Utc::now);

        // Calculate stability score based on various factors
        let mut stability_score = 100u8;

        // Deduct points for price deviation
        if status.price_deviation > Decimal::new(1, 2) { // > 1%
            stability_score = stability_score.saturating_sub(20);
        } else if status.price_deviation > Decimal::new(5, 3) { // > 0.5%
            stability_score = stability_score.saturating_sub(10);
        }

        // Deduct points for low collateral ratio
        if status.collateral_ratio < Decimal::new(130, 2) { // < 130%
            stability_score = stability_score.saturating_sub(30);
        } else if status.collateral_ratio < Decimal::new(150, 2) { // < 150%
            stability_score = stability_score.saturating_sub(15);
        }

        // Deduct points for high volatility
        if status.volatility > Decimal::new(5, 2) { // > 5%
            stability_score = stability_score.saturating_sub(25);
        } else if status.volatility > Decimal::new(2, 2) { // > 2%
            stability_score = stability_score.saturating_sub(10);
        }

        Ok(StabilityMetrics {
            price_deviation: status.price_deviation,
            stability_score,
            last_action,
            collateral_ratio: status.collateral_ratio,
            volatility: status.volatility,
            total_actions: events.len(),
            successful_actions: events.iter().filter(|e| e.result.success).count(),
        })
    }

    async fn get_stability_history(&self, stablecoin_id: Uuid, limit: Option<usize>) -> StablecoinResult<Vec<StabilityEvent>> {
        let history = self.stability_history.lock().await;

        if let Some(events) = history.get(&stablecoin_id) {
            let mut sorted_events = events.clone();
            sorted_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Newest first

            if let Some(limit) = limit {
                sorted_events.truncate(limit);
            }

            Ok(sorted_events)
        } else {
            Ok(Vec::new())
        }
    }

    async fn set_stability_parameters(&self, stablecoin_id: Uuid, params: StabilityParameters) -> StablecoinResult<()> {
        let mut parameters = self.parameters.lock().await;
        parameters.insert(stablecoin_id, params);
        Ok(())
    }

    async fn get_recommended_actions(&self, stablecoin_id: Uuid) -> StablecoinResult<Vec<StabilityAction>> {
        let status = self.monitor_stability(stablecoin_id).await?;
        let parameters = self.parameters.lock().await;

        let params = parameters.get(&stablecoin_id)
            .ok_or(StablecoinError::InvalidRequest("Stablecoin not initialized".to_string()))?;

        let mut recommendations = Vec::new();

        for trigger in &status.triggers {
            match trigger {
                StabilityTrigger::PriceDeviation { current, .. } => {
                    if *current > Decimal::ZERO {
                        // Price is above target, recommend burning tokens or increasing supply
                        if status.current_price > Decimal::new(101, 2) { // > $1.01
                            recommendations.push(StabilityAction::MintTokens {
                                amount: params.max_daily_adjustment / Decimal::new(10, 0), // 10% of max
                            });
                        } else if status.current_price < Decimal::new(99, 2) { // < $0.99
                            recommendations.push(StabilityAction::BurnTokens {
                                amount: params.max_daily_adjustment / Decimal::new(20, 0), // 5% of max
                            });
                        }
                    }
                },
                StabilityTrigger::LowCollateralRatio { current, .. } => {
                    if *current < params.min_collateral_ratio {
                        recommendations.push(StabilityAction::RebalanceCollateral {
                            target_ratio: params.target_collateral_ratio,
                        });

                        // Also recommend increasing stability fees to reduce demand
                        recommendations.push(StabilityAction::AdjustStabilityFee {
                            new_fee: params.stability_fee_rate + Decimal::new(1, 3), // +0.1%
                        });
                    }
                },
                StabilityTrigger::HighVolatility { volatility } => {
                    if *volatility > params.volatility_threshold * Decimal::new(2, 0) {
                        // Very high volatility - emergency measures
                        recommendations.push(StabilityAction::EmergencyPause);
                    } else {
                        // Moderate volatility - adjust interest rates
                        recommendations.push(StabilityAction::AdjustInterestRate {
                            new_rate: params.interest_rate_step,
                        });
                    }
                },
                StabilityTrigger::RedemptionPressure { amount } => {
                    // High redemption pressure - increase incentives to hold
                    recommendations.push(StabilityAction::AdjustInterestRate {
                        new_rate: params.interest_rate_step * Decimal::new(2, 0),
                    });

                    if *amount > params.max_daily_adjustment {
                        recommendations.push(StabilityAction::EmergencyPause);
                    }
                },
                StabilityTrigger::MarketManipulation => {
                    // Suspected manipulation - pause and investigate
                    recommendations.push(StabilityAction::EmergencyPause);
                },
            }
        }

        // Remove duplicates
        recommendations.dedup();

        Ok(recommendations)
    }
}

/// Enhanced arbitrage bot for maintaining price stability
pub struct EnterpriseArbitrageBot {
    min_profit_threshold: Decimal,
    max_trade_size: Decimal,
    supported_exchanges: Vec<String>,
    active_opportunities: Arc<Mutex<HashMap<Uuid, ArbitrageOpportunity>>>,
    execution_history: Arc<Mutex<Vec<ArbitrageExecution>>>,
}

impl EnterpriseArbitrageBot {
    pub fn new() -> Self {
        Self {
            min_profit_threshold: Decimal::new(1, 3), // 0.1%
            max_trade_size: Decimal::new(100_000, 0), // $100k max trade
            supported_exchanges: vec![
                "Uniswap".to_string(),
                "Curve".to_string(),
                "Balancer".to_string(),
                "SushiSwap".to_string(),
            ],
            active_opportunities: Arc::new(Mutex::new(HashMap::new())),
            execution_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_config(min_profit: Decimal, max_size: Decimal, exchanges: Vec<String>) -> Self {
        Self {
            min_profit_threshold: min_profit,
            max_trade_size: max_size,
            supported_exchanges: exchanges,
            active_opportunities: Arc::new(Mutex::new(HashMap::new())),
            execution_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Detect arbitrage opportunities across exchanges
    pub async fn detect_opportunities(&self, stablecoin_id: Uuid) -> StablecoinResult<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();

        // Simulate price discovery across exchanges
        let exchange_prices = self.fetch_exchange_prices(stablecoin_id).await?;

        // Find arbitrage opportunities between exchange pairs
        for (i, (exchange_a, price_a)) in exchange_prices.iter().enumerate() {
            for (exchange_b, price_b) in exchange_prices.iter().skip(i + 1) {
                let price_diff = (*price_a - *price_b).abs();
                let avg_price = (*price_a + *price_b) / Decimal::new(2, 0);
                let profit_percentage = (price_diff / avg_price) * Decimal::new(100, 0);

                if profit_percentage >= self.min_profit_threshold {
                    let opportunity = ArbitrageOpportunity {
                        id: Uuid::new_v4(),
                        stablecoin_id,
                        exchange_a: exchange_a.clone(),
                        exchange_b: exchange_b.clone(),
                        price_a: *price_a,
                        price_b: *price_b,
                        potential_profit: profit_percentage,
                        max_trade_size: self.max_trade_size,
                        expires_at: Utc::now() + Duration::minutes(5),
                        created_at: Utc::now(),
                    };

                    opportunities.push(opportunity);
                }
            }
        }

        // Store active opportunities
        {
            let mut active = self.active_opportunities.lock().await;
            for opp in &opportunities {
                active.insert(opp.id, opp.clone());
            }
        }

        Ok(opportunities)
    }

    /// Execute arbitrage trade
    pub async fn execute_arbitrage(&self, opportunity: ArbitrageOpportunity) -> StablecoinResult<ArbitrageExecution> {
        // Validate opportunity is still active
        {
            let active = self.active_opportunities.lock().await;
            if !active.contains_key(&opportunity.id) {
                return Err(StablecoinError::InvalidRequest("Opportunity no longer active".to_string()));
            }
        }

        // Check if opportunity hasn't expired
        if Utc::now() > opportunity.expires_at {
            return Err(StablecoinError::InvalidRequest("Opportunity expired".to_string()));
        }

        // Determine trade direction and size
        let (buy_exchange, sell_exchange, buy_price, sell_price) = if opportunity.price_a < opportunity.price_b {
            (opportunity.exchange_a.clone(), opportunity.exchange_b.clone(), opportunity.price_a, opportunity.price_b)
        } else {
            (opportunity.exchange_b.clone(), opportunity.exchange_a.clone(), opportunity.price_b, opportunity.price_a)
        };

        let trade_size = self.calculate_optimal_trade_size(&opportunity).await?;
        let expected_profit = (sell_price - buy_price) * trade_size;

        // Simulate trade execution
        let execution = ArbitrageExecution {
            id: Uuid::new_v4(),
            opportunity_id: opportunity.id,
            stablecoin_id: opportunity.stablecoin_id,
            buy_exchange,
            sell_exchange,
            buy_price,
            sell_price,
            trade_size,
            actual_profit: expected_profit * Decimal::new(95, 2), // 95% of expected (slippage)
            gas_cost: Decimal::new(50, 0), // $50 gas cost
            success: true,
            buy_tx_hash: format!("buy_tx_{}", Uuid::new_v4().simple()),
            sell_tx_hash: format!("sell_tx_{}", Uuid::new_v4().simple()),
            executed_at: Utc::now(),
        };

        // Record execution
        {
            let mut history = self.execution_history.lock().await;
            history.push(execution.clone());
        }

        // Remove from active opportunities
        {
            let mut active = self.active_opportunities.lock().await;
            active.remove(&opportunity.id);
        }

        Ok(execution)
    }

    /// Get arbitrage execution history
    pub async fn get_execution_history(&self, limit: Option<usize>) -> StablecoinResult<Vec<ArbitrageExecution>> {
        let history = self.execution_history.lock().await;
        let mut executions = history.clone();

        executions.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));

        if let Some(limit) = limit {
            executions.truncate(limit);
        }

        Ok(executions)
    }

    /// Calculate optimal trade size for arbitrage
    async fn calculate_optimal_trade_size(&self, opportunity: &ArbitrageOpportunity) -> StablecoinResult<Decimal> {
        // Simple calculation - in production would consider liquidity, slippage, etc.
        let max_profitable_size = self.max_trade_size.min(opportunity.max_trade_size);

        // Use 50% of max size to account for slippage
        Ok(max_profitable_size * Decimal::new(5, 1))
    }

    /// Simulate fetching prices from exchanges
    async fn fetch_exchange_prices(&self, _stablecoin_id: Uuid) -> StablecoinResult<Vec<(String, Decimal)>> {
        // Mock implementation - in production would call actual exchange APIs
        Ok(vec![
            ("Uniswap".to_string(), Decimal::new(10015, 4)), // $1.0015
            ("Curve".to_string(), Decimal::new(9995, 4)),    // $0.9995
            ("Balancer".to_string(), Decimal::new(10005, 4)), // $1.0005
            ("SushiSwap".to_string(), Decimal::new(9985, 4)), // $0.9985
        ])
    }
}

impl Default for EnterpriseArbitrageBot {
    fn default() -> Self {
        Self::new()
    }
}

/// Stability status result
#[derive(Debug, Clone)]
pub struct StabilityStatus {
    pub is_stable: bool,
    pub triggers: Vec<StabilityTrigger>,
    pub current_price: Decimal,
    pub price_deviation: Decimal,
    pub collateral_ratio: Decimal,
    pub volatility: Decimal,
    pub last_check: DateTime<Utc>,
}

/// Stability action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityActionResult {
    pub action: StabilityAction,
    pub success: bool,
    pub trigger: Option<StabilityTrigger>,
    pub details: String,
    pub gas_used: Option<u64>,
    pub transaction_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Stability event record
#[derive(Debug, Clone)]
pub struct StabilityEvent {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub action: StabilityAction,
    pub trigger: Option<StabilityTrigger>,
    pub result: StabilityActionResult,
    pub timestamp: DateTime<Utc>,
}

/// Enhanced stability metrics
#[derive(Debug, Clone)]
pub struct StabilityMetrics {
    pub price_deviation: Decimal,
    pub stability_score: u8,
    pub last_action: DateTime<Utc>,
    pub collateral_ratio: Decimal,
    pub volatility: Decimal,
    pub total_actions: usize,
    pub successful_actions: usize,
}

/// Price point for historical tracking
#[derive(Debug, Clone)]
pub struct PricePoint {
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// Enhanced arbitrage opportunity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: Uuid,
    pub stablecoin_id: Uuid,
    pub exchange_a: String,
    pub exchange_b: String,
    pub price_a: Decimal,
    pub price_b: Decimal,
    pub potential_profit: Decimal,
    pub max_trade_size: Decimal,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Arbitrage execution record
#[derive(Debug, Clone)]
pub struct ArbitrageExecution {
    pub id: Uuid,
    pub opportunity_id: Uuid,
    pub stablecoin_id: Uuid,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: Decimal,
    pub sell_price: Decimal,
    pub trade_size: Decimal,
    pub actual_profit: Decimal,
    pub gas_cost: Decimal,
    pub success: bool,
    pub buy_tx_hash: String,
    pub sell_tx_hash: String,
    pub executed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enterprise_price_stabilizer_creation() {
        let stabilizer = EnterprisePriceStabilizer::new();
        let stablecoin_id = Uuid::new_v4();

        // Initialize monitoring
        let params = StabilityParameters::default();
        stabilizer.initialize_monitoring(stablecoin_id, params).await.unwrap();

        // Test monitoring
        let status = stabilizer.monitor_stability(stablecoin_id).await.unwrap();
        assert!(status.is_stable); // Should be stable initially
        assert_eq!(status.current_price, Decimal::new(100, 2)); // $1.00
    }

    #[tokio::test]
    async fn test_stability_parameters() {
        let params = StabilityParameters::default();

        assert_eq!(params.max_price_deviation, Decimal::new(2, 2)); // 2%
        assert_eq!(params.target_collateral_ratio, Decimal::new(150, 2)); // 150%
        assert_eq!(params.min_collateral_ratio, Decimal::new(120, 2)); // 120%
        assert_eq!(params.stability_fee_rate, Decimal::new(5, 3)); // 0.5%
    }

    #[tokio::test]
    async fn test_price_feed() {
        let mut feed = PriceFeed::new(Uuid::new_v4());

        // Test initial state
        assert_eq!(feed.current_price, Decimal::new(100, 2)); // $1.00
        assert_eq!(feed.get_price_deviation(), Decimal::ZERO);

        // Update price and test deviation
        feed.update_price(Decimal::new(102, 2)); // $1.02
        let deviation = feed.get_price_deviation();
        assert_eq!(deviation, Decimal::new(2, 0)); // 2%

        // Test volatility calculation
        feed.update_price(Decimal::new(98, 2)); // $0.98
        feed.update_price(Decimal::new(101, 2)); // $1.01
        let volatility = feed.calculate_volatility();
        assert!(volatility > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_collateral_monitor() {
        let mut monitor = CollateralMonitor::new(Uuid::new_v4());

        let collateral = vec![
            CollateralPosition {
                id: Uuid::new_v4(),
                collateral_type: crate::types::CollateralType::Fiat { currency: "USD".to_string() },
                amount: Decimal::new(1000, 0),
                value_usd: Decimal::new(1000, 0),
                locked_until: None,
                status: crate::types::CollateralStatus::Active,
            }
        ];

        monitor.update_collateral(collateral, Decimal::new(500, 0)); // $500 issued

        let ratio = monitor.get_collateral_ratio();
        assert_eq!(ratio, Decimal::new(200, 0)); // 200%

        assert!(!monitor.is_undercollateralized(Decimal::new(150, 0))); // Not under 150%
        assert!(monitor.is_undercollateralized(Decimal::new(250, 0))); // Under 250%
    }

    #[tokio::test]
    async fn test_stability_actions() {
        let stabilizer = EnterprisePriceStabilizer::new();
        let stablecoin_id = Uuid::new_v4();

        // Initialize monitoring with short cooldown for testing
        let mut params = StabilityParameters::default();
        params.action_cooldown = 1; // 1 second cooldown
        stabilizer.initialize_monitoring(stablecoin_id, params).await.unwrap();

        // Test mint action
        let mint_action = StabilityAction::MintTokens { amount: Decimal::new(1000, 0) };
        let result = stabilizer.execute_stability_action(stablecoin_id, mint_action).await.unwrap();

        assert!(result.success);
        assert!(result.transaction_hash.is_some());
        assert!(result.gas_used.is_some());

        // Wait for cooldown period
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Test burn action
        let burn_action = StabilityAction::BurnTokens { amount: Decimal::new(500, 0) };
        let result = stabilizer.execute_stability_action(stablecoin_id, burn_action).await.unwrap();

        assert!(result.success);
        assert!(result.details.contains("Burned 500 tokens"));
    }

    #[tokio::test]
    async fn test_stability_metrics() {
        let stabilizer = EnterprisePriceStabilizer::new();
        let stablecoin_id = Uuid::new_v4();

        // Initialize monitoring
        let params = StabilityParameters::default();
        stabilizer.initialize_monitoring(stablecoin_id, params).await.unwrap();

        // Execute some actions to create history
        let action = StabilityAction::AdjustInterestRate { new_rate: Decimal::new(1, 2) };
        stabilizer.execute_stability_action(stablecoin_id, action).await.unwrap();

        let metrics = stabilizer.get_stability_metrics(stablecoin_id).await.unwrap();

        assert_eq!(metrics.total_actions, 1);
        assert_eq!(metrics.successful_actions, 1);
        assert!(metrics.stability_score <= 100);
    }

    #[tokio::test]
    async fn test_stability_history() {
        let stabilizer = EnterprisePriceStabilizer::new();
        let stablecoin_id = Uuid::new_v4();

        // Initialize monitoring with short cooldown for testing
        let mut params = StabilityParameters::default();
        params.action_cooldown = 1; // 1 second cooldown
        stabilizer.initialize_monitoring(stablecoin_id, params).await.unwrap();

        // Execute multiple actions with delays
        let actions = vec![
            StabilityAction::MintTokens { amount: Decimal::new(1000, 0) },
            StabilityAction::BurnTokens { amount: Decimal::new(500, 0) },
            StabilityAction::AdjustInterestRate { new_rate: Decimal::new(2, 2) },
        ];

        for (i, action) in actions.into_iter().enumerate() {
            if i > 0 {
                // Wait for cooldown period between actions
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            stabilizer.execute_stability_action(stablecoin_id, action).await.unwrap();
        }

        let history = stabilizer.get_stability_history(stablecoin_id, Some(2)).await.unwrap();
        assert_eq!(history.len(), 2); // Limited to 2

        let full_history = stabilizer.get_stability_history(stablecoin_id, None).await.unwrap();
        assert_eq!(full_history.len(), 3); // All actions
    }

    #[tokio::test]
    async fn test_recommended_actions() {
        let stabilizer = EnterprisePriceStabilizer::new();
        let stablecoin_id = Uuid::new_v4();

        // Initialize monitoring
        let params = StabilityParameters::default();
        stabilizer.initialize_monitoring(stablecoin_id, params).await.unwrap();

        // Get recommendations (should be empty for stable state)
        let recommendations = stabilizer.get_recommended_actions(stablecoin_id).await.unwrap();
        assert!(recommendations.is_empty()); // Should be stable initially
    }

    #[tokio::test]
    async fn test_enterprise_arbitrage_bot() {
        let bot = EnterpriseArbitrageBot::new();
        let stablecoin_id = Uuid::new_v4();

        // Test opportunity detection
        let opportunities = bot.detect_opportunities(stablecoin_id).await.unwrap();
        assert!(!opportunities.is_empty()); // Should find some opportunities

        // Test execution
        if let Some(opportunity) = opportunities.first() {
            let execution = bot.execute_arbitrage(opportunity.clone()).await.unwrap();
            assert!(execution.success);
            assert!(!execution.buy_tx_hash.is_empty());
            assert!(!execution.sell_tx_hash.is_empty());
        }
    }

    #[tokio::test]
    async fn test_arbitrage_bot_config() {
        let bot = EnterpriseArbitrageBot::with_config(
            Decimal::new(5, 4), // 0.05% min profit
            Decimal::new(50_000, 0), // $50k max trade
            vec!["Uniswap".to_string(), "Curve".to_string()],
        );

        assert_eq!(bot.min_profit_threshold, Decimal::new(5, 4));
        assert_eq!(bot.max_trade_size, Decimal::new(50_000, 0));
        assert_eq!(bot.supported_exchanges.len(), 2);
    }

    #[tokio::test]
    async fn test_arbitrage_execution_history() {
        let bot = EnterpriseArbitrageBot::new();
        let stablecoin_id = Uuid::new_v4();

        // Execute some arbitrage
        let opportunities = bot.detect_opportunities(stablecoin_id).await.unwrap();
        for opportunity in opportunities.iter().take(2) {
            bot.execute_arbitrage(opportunity.clone()).await.unwrap();
        }

        let history = bot.get_execution_history(Some(1)).await.unwrap();
        assert_eq!(history.len(), 1); // Limited to 1

        let full_history = bot.get_execution_history(None).await.unwrap();
        assert_eq!(full_history.len(), 2); // All executions
    }

    #[tokio::test]
    async fn test_action_cooldown() {
        let stabilizer = EnterprisePriceStabilizer::new();
        let stablecoin_id = Uuid::new_v4();

        // Initialize with short cooldown for testing
        let mut params = StabilityParameters::default();
        params.action_cooldown = 1; // 1 second
        stabilizer.initialize_monitoring(stablecoin_id, params).await.unwrap();

        // Execute first action
        let action = StabilityAction::MintTokens { amount: Decimal::new(100, 0) };
        let result = stabilizer.execute_stability_action(stablecoin_id, action.clone()).await.unwrap();
        assert!(result.success);

        // Try immediate second action (should fail due to cooldown)
        let result = stabilizer.execute_stability_action(stablecoin_id, action.clone()).await;
        assert!(result.is_err());

        // Wait for cooldown and try again
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let result = stabilizer.execute_stability_action(stablecoin_id, action).await.unwrap();
        assert!(result.success);
    }
}
