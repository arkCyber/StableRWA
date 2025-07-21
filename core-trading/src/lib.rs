// =====================================================================================
// File: core-trading/src/lib.rs
// Description: Trading and liquidity management system for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Trading Module
//! 
//! This module provides comprehensive trading and liquidity management functionality for the
//! StableRWA platform, including order book management, liquidity pools, price discovery,
//! market making, and trade settlement.

pub mod order_book;
pub mod liquidity;
pub mod matching;
pub mod settlement;
pub mod market_data;
pub mod price_discovery;
pub mod market_making;
pub mod risk_controls;
pub mod error;
pub mod types;
pub mod service;

// Re-export main types and traits
pub use error::{TradingError, TradingResult};
pub use types::{
    Order, OrderType, OrderSide, OrderStatus, Trade, TradeStatus,
    LiquidityPool, MarketData, PriceLevel, OrderBook, TradingPair
};
pub use service::TradingService;
pub use order_book::{OrderBookManager, OrderBookSnapshot};
pub use liquidity::{LiquidityManager, LiquidityProvider};
pub use matching::{MatchingEngine, MatchResult};
pub use settlement::{SettlementService, SettlementResult};
pub use market_data::{MarketDataService, MarketDataFeed};
pub use price_discovery::{PriceDiscoveryEngine, PriceQuote};
pub use market_making::{MarketMaker, MarketMakingStrategy};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Order book configuration
    pub order_book_config: order_book::OrderBookConfig,
    /// Liquidity management configuration
    pub liquidity_config: liquidity::LiquidityConfig,
    /// Matching engine configuration
    pub matching_config: matching::MatchingConfig,
    /// Settlement configuration
    pub settlement_config: settlement::SettlementConfig,
    /// Market data configuration
    pub market_data_config: market_data::MarketDataConfig,
    /// Price discovery configuration
    pub price_discovery_config: price_discovery::PriceDiscoveryConfig,
    /// Market making configuration
    pub market_making_config: market_making::MarketMakingConfig,
    /// Risk controls configuration
    pub risk_controls_config: risk_controls::RiskControlsConfig,
    /// Global trading settings
    pub global_settings: GlobalTradingSettings,
}

impl Default for TradingConfig {
    fn default() -> Self {
        Self {
            order_book_config: order_book::OrderBookConfig::default(),
            liquidity_config: liquidity::LiquidityConfig::default(),
            matching_config: matching::MatchingConfig::default(),
            settlement_config: settlement::SettlementConfig::default(),
            market_data_config: market_data::MarketDataConfig::default(),
            price_discovery_config: price_discovery::PriceDiscoveryConfig::default(),
            market_making_config: market_making::MarketMakingConfig::default(),
            risk_controls_config: risk_controls::RiskControlsConfig::default(),
            global_settings: GlobalTradingSettings::default(),
        }
    }
}

/// Global trading settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalTradingSettings {
    /// Supported trading pairs
    pub supported_pairs: Vec<TradingPairConfig>,
    /// Minimum order size (in USD)
    pub min_order_size: Decimal,
    /// Maximum order size (in USD)
    pub max_order_size: Decimal,
    /// Trading fees configuration
    pub fee_structure: FeeStructure,
    /// Market hours configuration
    pub market_hours: MarketHours,
    /// Enable circuit breakers
    pub circuit_breakers_enabled: bool,
    /// Circuit breaker thresholds
    pub circuit_breaker_thresholds: CircuitBreakerThresholds,
    /// Rate limiting settings
    pub rate_limits: TradingRateLimits,
}

impl Default for GlobalTradingSettings {
    fn default() -> Self {
        Self {
            supported_pairs: vec![
                TradingPairConfig::new("BTC", "USD", true),
                TradingPairConfig::new("ETH", "USD", true),
                TradingPairConfig::new("RWA", "USD", true),
                TradingPairConfig::new("RWA", "BTC", true),
                TradingPairConfig::new("RWA", "ETH", true),
            ],
            min_order_size: Decimal::new(1000, 2), // $10.00
            max_order_size: Decimal::new(100000000, 2), // $1,000,000.00
            fee_structure: FeeStructure::default(),
            market_hours: MarketHours::default(),
            circuit_breakers_enabled: true,
            circuit_breaker_thresholds: CircuitBreakerThresholds::default(),
            rate_limits: TradingRateLimits::default(),
        }
    }
}

/// Trading pair configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPairConfig {
    pub base_asset: String,
    pub quote_asset: String,
    pub enabled: bool,
    pub min_price_increment: Decimal,
    pub min_quantity_increment: Decimal,
    pub max_price_deviation: Decimal,
    pub liquidity_requirements: LiquidityRequirements,
}

impl TradingPairConfig {
    pub fn new(base: &str, quote: &str, enabled: bool) -> Self {
        Self {
            base_asset: base.to_string(),
            quote_asset: quote.to_string(),
            enabled,
            min_price_increment: Decimal::new(1, 2), // $0.01
            min_quantity_increment: Decimal::new(1, 8), // 0.00000001
            max_price_deviation: Decimal::new(10, 0), // 10%
            liquidity_requirements: LiquidityRequirements::default(),
        }
    }

    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base_asset, self.quote_asset)
    }
}

/// Liquidity requirements for trading pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityRequirements {
    pub min_liquidity_usd: Decimal,
    pub min_spread_bps: u32, // basis points
    pub max_spread_bps: u32,
    pub min_depth_levels: u32,
}

impl Default for LiquidityRequirements {
    fn default() -> Self {
        Self {
            min_liquidity_usd: Decimal::new(10000000, 2), // $100,000
            min_spread_bps: 10,  // 0.10%
            max_spread_bps: 500, // 5.00%
            min_depth_levels: 5,
        }
    }
}

/// Fee structure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    pub maker_fee_bps: u32,
    pub taker_fee_bps: u32,
    pub withdrawal_fee_bps: u32,
    pub deposit_fee_bps: u32,
    pub volume_tiers: Vec<VolumeTier>,
}

impl Default for FeeStructure {
    fn default() -> Self {
        Self {
            maker_fee_bps: 10,  // 0.10%
            taker_fee_bps: 20,  // 0.20%
            withdrawal_fee_bps: 50, // 0.50%
            deposit_fee_bps: 0,  // 0.00%
            volume_tiers: vec![
                VolumeTier {
                    min_volume_usd: Decimal::ZERO,
                    maker_fee_bps: 10,
                    taker_fee_bps: 20,
                },
                VolumeTier {
                    min_volume_usd: Decimal::new(100000000, 2), // $1M
                    maker_fee_bps: 8,
                    taker_fee_bps: 15,
                },
                VolumeTier {
                    min_volume_usd: Decimal::new(1000000000, 2), // $10M
                    maker_fee_bps: 5,
                    taker_fee_bps: 10,
                },
            ],
        }
    }
}

/// Volume tier for fee calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeTier {
    pub min_volume_usd: Decimal,
    pub maker_fee_bps: u32,
    pub taker_fee_bps: u32,
}

/// Market hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHours {
    pub always_open: bool,
    pub timezone: String,
    pub trading_sessions: Vec<TradingSession>,
    pub holidays: Vec<chrono::NaiveDate>,
}

impl Default for MarketHours {
    fn default() -> Self {
        Self {
            always_open: true, // Crypto markets are 24/7
            timezone: "UTC".to_string(),
            trading_sessions: vec![],
            holidays: vec![],
        }
    }
}

/// Trading session definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSession {
    pub name: String,
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
    pub days_of_week: Vec<chrono::Weekday>,
}

/// Circuit breaker thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerThresholds {
    pub price_change_threshold_bps: u32,
    pub volume_spike_threshold: Decimal,
    pub order_rate_threshold: u32,
    pub pause_duration_seconds: u64,
}

impl Default for CircuitBreakerThresholds {
    fn default() -> Self {
        Self {
            price_change_threshold_bps: 1000, // 10%
            volume_spike_threshold: Decimal::new(500, 2), // 5x normal volume
            order_rate_threshold: 1000, // orders per second
            pause_duration_seconds: 300, // 5 minutes
        }
    }
}

/// Trading rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingRateLimits {
    pub orders_per_second: u32,
    pub orders_per_minute: u32,
    pub orders_per_hour: u32,
    pub max_open_orders: u32,
    pub max_order_value_usd: Decimal,
}

impl Default for TradingRateLimits {
    fn default() -> Self {
        Self {
            orders_per_second: 10,
            orders_per_minute: 100,
            orders_per_hour: 1000,
            max_open_orders: 100,
            max_order_value_usd: Decimal::new(100000000, 2), // $1M
        }
    }
}

/// Trading statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingStatistics {
    pub trading_pair: String,
    pub volume_24h: Decimal,
    pub price_change_24h: Decimal,
    pub price_change_24h_percent: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub last_price: Decimal,
    pub bid_price: Option<Decimal>,
    pub ask_price: Option<Decimal>,
    pub spread: Option<Decimal>,
    pub spread_bps: Option<u32>,
    pub total_trades: u64,
    pub active_orders: u64,
    pub last_updated: DateTime<Utc>,
}

/// Market status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketStatus {
    /// Market is open for trading
    Open,
    /// Market is closed
    Closed,
    /// Market is paused due to circuit breaker
    Paused,
    /// Market is in pre-opening phase
    PreOpen,
    /// Market is in post-closing phase
    PostClose,
    /// Market is under maintenance
    Maintenance,
    /// Market is halted due to emergency
    Halted,
}

impl MarketStatus {
    /// Check if trading is allowed
    pub fn allows_trading(&self) -> bool {
        matches!(self, MarketStatus::Open)
    }

    /// Check if order placement is allowed
    pub fn allows_orders(&self) -> bool {
        matches!(self, MarketStatus::Open | MarketStatus::PreOpen)
    }

    /// Check if order cancellation is allowed
    pub fn allows_cancellation(&self) -> bool {
        !matches!(self, MarketStatus::Maintenance | MarketStatus::Halted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_config_default() {
        let config = TradingConfig::default();
        assert!(!config.global_settings.supported_pairs.is_empty());
        assert!(config.global_settings.circuit_breakers_enabled);
        assert_eq!(config.global_settings.min_order_size, Decimal::new(1000, 2));
    }

    #[test]
    fn test_trading_pair_config() {
        let pair = TradingPairConfig::new("BTC", "USD", true);
        assert_eq!(pair.base_asset, "BTC");
        assert_eq!(pair.quote_asset, "USD");
        assert_eq!(pair.symbol(), "BTC/USD");
        assert!(pair.enabled);
    }

    #[test]
    fn test_fee_structure_default() {
        let fees = FeeStructure::default();
        assert_eq!(fees.maker_fee_bps, 10);
        assert_eq!(fees.taker_fee_bps, 20);
        assert!(!fees.volume_tiers.is_empty());
    }

    #[test]
    fn test_market_hours_default() {
        let market_hours = MarketHours::default();
        assert!(market_hours.always_open);
        assert_eq!(market_hours.timezone, "UTC");
    }

    #[test]
    fn test_circuit_breaker_thresholds() {
        let thresholds = CircuitBreakerThresholds::default();
        assert_eq!(thresholds.price_change_threshold_bps, 1000);
        assert_eq!(thresholds.pause_duration_seconds, 300);
    }

    #[test]
    fn test_trading_rate_limits() {
        let limits = TradingRateLimits::default();
        assert_eq!(limits.orders_per_second, 10);
        assert_eq!(limits.orders_per_minute, 100);
        assert_eq!(limits.max_open_orders, 100);
    }

    #[test]
    fn test_market_status_permissions() {
        assert!(MarketStatus::Open.allows_trading());
        assert!(MarketStatus::Open.allows_orders());
        assert!(MarketStatus::Open.allows_cancellation());

        assert!(!MarketStatus::Closed.allows_trading());
        assert!(!MarketStatus::Closed.allows_orders());
        assert!(MarketStatus::Closed.allows_cancellation());

        assert!(MarketStatus::PreOpen.allows_orders());
        assert!(!MarketStatus::PreOpen.allows_trading());

        assert!(!MarketStatus::Halted.allows_cancellation());
        assert!(!MarketStatus::Maintenance.allows_cancellation());
    }

    #[test]
    fn test_liquidity_requirements_default() {
        let requirements = LiquidityRequirements::default();
        assert_eq!(requirements.min_liquidity_usd, Decimal::new(10000000, 2));
        assert_eq!(requirements.min_spread_bps, 10);
        assert_eq!(requirements.max_spread_bps, 500);
        assert_eq!(requirements.min_depth_levels, 5);
    }

    #[test]
    fn test_volume_tier_structure() {
        let fees = FeeStructure::default();
        assert_eq!(fees.volume_tiers.len(), 3);
        
        // Check that tiers are ordered by volume
        for i in 1..fees.volume_tiers.len() {
            assert!(fees.volume_tiers[i].min_volume_usd > fees.volume_tiers[i-1].min_volume_usd);
        }
        
        // Check that higher volume tiers have lower fees
        for i in 1..fees.volume_tiers.len() {
            assert!(fees.volume_tiers[i].maker_fee_bps <= fees.volume_tiers[i-1].maker_fee_bps);
            assert!(fees.volume_tiers[i].taker_fee_bps <= fees.volume_tiers[i-1].taker_fee_bps);
        }
    }

    #[test]
    fn test_trading_statistics_creation() {
        let stats = TradingStatistics {
            trading_pair: "BTC/USD".to_string(),
            volume_24h: Decimal::new(100000000, 2), // $1,000,000
            price_change_24h: Decimal::new(500000, 2), // $5,000
            price_change_24h_percent: Decimal::new(500, 2), // 5%
            high_24h: Decimal::new(10500000, 2), // $105,000
            low_24h: Decimal::new(9500000, 2), // $95,000
            last_price: Decimal::new(10000000, 2), // $100,000
            bid_price: Some(Decimal::new(9999000, 2)), // $99,990
            ask_price: Some(Decimal::new(10001000, 2)), // $100,010
            spread: Some(Decimal::new(2000, 2)), // $20
            spread_bps: Some(20), // 0.20%
            total_trades: 1000,
            active_orders: 50,
            last_updated: Utc::now(),
        };

        assert_eq!(stats.trading_pair, "BTC/USD");
        assert_eq!(stats.volume_24h, Decimal::new(100000000, 2));
        assert_eq!(stats.total_trades, 1000);
        assert!(stats.bid_price.is_some());
        assert!(stats.ask_price.is_some());
    }
}
