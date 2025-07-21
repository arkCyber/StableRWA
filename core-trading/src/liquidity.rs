// =====================================================================================
// File: core-trading/src/liquidity.rs
// Description: Liquidity management for RWA trading system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    error::{TradingError, TradingResult},
    types::TradingPair,
};

/// Liquidity manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityConfig {
    /// Minimum liquidity threshold
    pub min_liquidity_threshold: Decimal,
    /// Maximum spread percentage
    pub max_spread_percentage: Decimal,
    /// Liquidity provider fee percentage
    pub lp_fee_percentage: Decimal,
    /// Minimum order size for liquidity provision
    pub min_order_size: Decimal,
    /// Maximum order size for liquidity provision
    pub max_order_size: Decimal,
    /// Price update frequency in seconds
    pub price_update_frequency_seconds: u64,
    /// Enable dynamic pricing
    pub enable_dynamic_pricing: bool,
    /// Inventory management enabled
    pub inventory_management_enabled: bool,
    /// Maximum inventory imbalance percentage
    pub max_inventory_imbalance: Decimal,
}

impl Default for LiquidityConfig {
    fn default() -> Self {
        Self {
            min_liquidity_threshold: Decimal::new(10000, 0), // $10,000
            max_spread_percentage: Decimal::new(500, 4), // 5.00%
            lp_fee_percentage: Decimal::new(30, 4), // 0.30%
            min_order_size: Decimal::new(100, 0), // $100
            max_order_size: Decimal::new(1000000, 0), // $1,000,000
            price_update_frequency_seconds: 5,
            enable_dynamic_pricing: true,
            inventory_management_enabled: true,
            max_inventory_imbalance: Decimal::new(2000, 4), // 20.00%
        }
    }
}

/// Liquidity provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityProvider {
    pub id: Uuid,
    pub user_id: String,
    pub trading_pair: TradingPair,
    pub base_inventory: Decimal,
    pub quote_inventory: Decimal,
    pub target_spread: Decimal,
    pub min_order_size: Decimal,
    pub max_order_size: Decimal,
    pub is_active: bool,
    pub total_volume: Decimal,
    pub total_fees_earned: Decimal,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

impl LiquidityProvider {
    /// Create a new liquidity provider
    pub fn new(
        user_id: String,
        trading_pair: TradingPair,
        base_inventory: Decimal,
        quote_inventory: Decimal,
        target_spread: Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            trading_pair,
            base_inventory,
            quote_inventory,
            target_spread,
            min_order_size: Decimal::new(100, 0),
            max_order_size: Decimal::new(10000, 0),
            is_active: true,
            total_volume: Decimal::ZERO,
            total_fees_earned: Decimal::ZERO,
            created_at: now,
            last_active: now,
        }
    }

    /// Calculate inventory imbalance
    pub fn inventory_imbalance(&self, mid_price: Decimal) -> Decimal {
        let total_value = self.base_inventory * mid_price + self.quote_inventory;
        if total_value == Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        let base_value = self.base_inventory * mid_price;
        let target_value = total_value / Decimal::TWO;
        
        (base_value - target_value) / total_value
    }

    /// Update inventory after trade
    pub fn update_inventory(&mut self, base_change: Decimal, quote_change: Decimal, fee_earned: Decimal) {
        self.base_inventory += base_change;
        self.quote_inventory += quote_change;
        self.total_fees_earned += fee_earned;
        self.last_active = Utc::now();
    }
}

/// Liquidity quote for market making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityQuote {
    pub provider_id: Uuid,
    pub trading_pair: TradingPair,
    pub bid_price: Decimal,
    pub ask_price: Decimal,
    pub bid_size: Decimal,
    pub ask_size: Decimal,
    pub spread: Decimal,
    pub mid_price: Decimal,
    pub valid_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl LiquidityQuote {
    /// Create a new liquidity quote
    pub fn new(
        provider_id: Uuid,
        trading_pair: TradingPair,
        mid_price: Decimal,
        spread: Decimal,
        bid_size: Decimal,
        ask_size: Decimal,
        validity_seconds: u64,
    ) -> Self {
        let half_spread = spread / Decimal::TWO;
        let bid_price = mid_price - half_spread;
        let ask_price = mid_price + half_spread;
        let now = Utc::now();
        
        Self {
            provider_id,
            trading_pair,
            bid_price,
            ask_price,
            bid_size,
            ask_size,
            spread,
            mid_price,
            valid_until: now + chrono::Duration::seconds(validity_seconds as i64),
            created_at: now,
        }
    }

    /// Check if quote is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() <= self.valid_until
    }

    /// Calculate quote quality score
    pub fn quality_score(&self) -> f64 {
        let spread_score = 1.0 / (1.0 + self.spread.to_f64().unwrap_or(1.0));
        let size_score = (self.bid_size + self.ask_size).to_f64().unwrap_or(0.0) / 10000.0;
        
        (spread_score + size_score.min(1.0)) / 2.0
    }
}

/// Market depth information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDepth {
    pub trading_pair: TradingPair,
    pub bid_depth: Vec<DepthLevel>,
    pub ask_depth: Vec<DepthLevel>,
    pub total_bid_volume: Decimal,
    pub total_ask_volume: Decimal,
    pub spread: Decimal,
    pub mid_price: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// Depth level in market depth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthLevel {
    pub price: Decimal,
    pub size: Decimal,
    pub provider_count: u32,
}

/// Liquidity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityMetrics {
    pub trading_pair: TradingPair,
    pub total_liquidity: Decimal,
    pub average_spread: Decimal,
    pub bid_ask_imbalance: Decimal,
    pub price_impact_1pct: Decimal,
    pub price_impact_5pct: Decimal,
    pub active_providers: u32,
    pub volume_24h: Decimal,
    pub trades_24h: u64,
    pub last_updated: DateTime<Utc>,
}

/// Liquidity manager trait
#[async_trait]
pub trait LiquidityManager: Send + Sync {
    /// Register a new liquidity provider
    async fn register_provider(&self, provider: LiquidityProvider) -> TradingResult<()>;
    
    /// Update liquidity provider status
    async fn update_provider_status(&self, provider_id: Uuid, is_active: bool) -> TradingResult<()>;
    
    /// Get liquidity provider
    async fn get_provider(&self, provider_id: Uuid) -> TradingResult<Option<LiquidityProvider>>;
    
    /// Get all active providers for a trading pair
    async fn get_active_providers(&self, trading_pair: TradingPair) -> TradingResult<Vec<LiquidityProvider>>;
    
    /// Submit liquidity quote
    async fn submit_quote(&self, quote: LiquidityQuote) -> TradingResult<()>;
    
    /// Get best quotes for a trading pair
    async fn get_best_quotes(&self, trading_pair: TradingPair) -> TradingResult<Vec<LiquidityQuote>>;
    
    /// Calculate market depth
    async fn calculate_market_depth(&self, trading_pair: TradingPair, depth_levels: usize) -> TradingResult<MarketDepth>;
    
    /// Get liquidity metrics
    async fn get_liquidity_metrics(&self, trading_pair: TradingPair) -> TradingResult<LiquidityMetrics>;
    
    /// Calculate price impact
    async fn calculate_price_impact(
        &self,
        trading_pair: TradingPair,
        side: crate::types::OrderSide,
        size: Decimal,
    ) -> TradingResult<Decimal>;
    
    /// Update provider inventory after trade
    async fn update_provider_inventory(
        &self,
        provider_id: Uuid,
        base_change: Decimal,
        quote_change: Decimal,
        fee_earned: Decimal,
    ) -> TradingResult<()>;
    
    /// Health check
    async fn health_check(&self) -> TradingResult<LiquidityHealthStatus>;
}

/// Liquidity health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityHealthStatus {
    pub status: String,
    pub active_providers: u64,
    pub total_providers: u64,
    pub total_liquidity: Decimal,
    pub average_spread: Decimal,
    pub market_pairs_covered: u64,
    pub quotes_per_second: f64,
    pub last_quote_update: DateTime<Utc>,
    pub last_check: DateTime<Utc>,
}

/// Automated Market Maker (AMM) pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMPool {
    pub id: Uuid,
    pub trading_pair: TradingPair,
    pub base_reserve: Decimal,
    pub quote_reserve: Decimal,
    pub total_shares: Decimal,
    pub fee_percentage: Decimal,
    pub k_constant: Decimal, // x * y = k
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

impl AMMPool {
    /// Create a new AMM pool
    pub fn new(
        trading_pair: TradingPair,
        base_reserve: Decimal,
        quote_reserve: Decimal,
        fee_percentage: Decimal,
    ) -> Self {
        let k_constant = base_reserve * quote_reserve;
        let total_shares = (base_reserve * quote_reserve).sqrt().unwrap_or(Decimal::ZERO);
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4(),
            trading_pair,
            base_reserve,
            quote_reserve,
            total_shares,
            fee_percentage,
            k_constant,
            created_at: now,
            last_updated: now,
        }
    }

    /// Calculate current price
    pub fn current_price(&self) -> Decimal {
        if self.base_reserve == Decimal::ZERO {
            return Decimal::ZERO;
        }
        self.quote_reserve / self.base_reserve
    }

    /// Calculate output amount for a given input
    pub fn calculate_output(&self, input_amount: Decimal, input_is_base: bool) -> Decimal {
        let (input_reserve, output_reserve) = if input_is_base {
            (self.base_reserve, self.quote_reserve)
        } else {
            (self.quote_reserve, self.base_reserve)
        };

        let input_with_fee = input_amount * (Decimal::ONE - self.fee_percentage);
        let numerator = input_with_fee * output_reserve;
        let denominator = input_reserve + input_with_fee;
        
        if denominator == Decimal::ZERO {
            Decimal::ZERO
        } else {
            numerator / denominator
        }
    }

    /// Calculate price impact for a trade
    pub fn calculate_price_impact(&self, input_amount: Decimal, input_is_base: bool) -> Decimal {
        let current_price = self.current_price();
        let output_amount = self.calculate_output(input_amount, input_is_base);
        
        if output_amount == Decimal::ZERO || input_amount == Decimal::ZERO {
            return Decimal::ZERO;
        }

        let effective_price = if input_is_base {
            output_amount / input_amount
        } else {
            input_amount / output_amount
        };

        ((effective_price - current_price) / current_price).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquidity_provider_creation() {
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let provider = LiquidityProvider::new(
            "user123".to_string(),
            trading_pair.clone(),
            Decimal::new(10, 0), // 10 BTC
            Decimal::new(500000, 0), // $500,000
            Decimal::new(100, 4), // 1.00% spread
        );

        assert_eq!(provider.user_id, "user123");
        assert_eq!(provider.trading_pair, trading_pair);
        assert_eq!(provider.base_inventory, Decimal::new(10, 0));
        assert_eq!(provider.quote_inventory, Decimal::new(500000, 0));
        assert!(provider.is_active);
    }

    #[test]
    fn test_liquidity_quote_creation() {
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let quote = LiquidityQuote::new(
            Uuid::new_v4(),
            trading_pair,
            Decimal::new(50000, 0), // $50,000 mid price
            Decimal::new(100, 0), // $100 spread
            Decimal::new(1, 0), // 1 BTC bid size
            Decimal::new(1, 0), // 1 BTC ask size
            30, // 30 seconds validity
        );

        assert_eq!(quote.mid_price, Decimal::new(50000, 0));
        assert_eq!(quote.spread, Decimal::new(100, 0));
        assert_eq!(quote.bid_price, Decimal::new(49950, 0));
        assert_eq!(quote.ask_price, Decimal::new(50050, 0));
        assert!(quote.is_valid());
    }

    #[test]
    fn test_amm_pool() {
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let pool = AMMPool::new(
            trading_pair,
            Decimal::new(100, 0), // 100 BTC
            Decimal::new(5000000, 0), // $5,000,000
            Decimal::new(30, 4), // 0.30% fee
        );

        assert_eq!(pool.current_price(), Decimal::new(50000, 0)); // $50,000 per BTC
        
        // Test swap calculation
        let output = pool.calculate_output(Decimal::new(1, 0), true); // 1 BTC input
        assert!(output > Decimal::ZERO);
        assert!(output < Decimal::new(50000, 0)); // Should be less than current price due to slippage
        
        // Test price impact
        let impact = pool.calculate_price_impact(Decimal::new(1, 0), true);
        assert!(impact > Decimal::ZERO);
    }
}
