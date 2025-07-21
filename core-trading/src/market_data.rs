// =====================================================================================
// File: core-trading/src/market_data.rs
// Description: Market data service for RWA trading system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    error::{TradingError, TradingResult},
    types::TradingPair,
    matching::Trade,
};

/// Market data configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataConfig {
    /// Data retention period in days
    pub retention_days: u32,
    /// Update frequency in milliseconds
    pub update_frequency_ms: u64,
    /// Enable real-time streaming
    pub enable_streaming: bool,
    /// Maximum subscribers per feed
    pub max_subscribers: u32,
    /// Candlestick intervals to maintain
    pub candlestick_intervals: Vec<CandlestickInterval>,
    /// Enable market depth tracking
    pub enable_depth_tracking: bool,
    /// Depth levels to track
    pub depth_levels: u32,
}

impl Default for MarketDataConfig {
    fn default() -> Self {
        Self {
            retention_days: 365,
            update_frequency_ms: 100,
            enable_streaming: true,
            max_subscribers: 1000,
            candlestick_intervals: vec![
                CandlestickInterval::OneMinute,
                CandlestickInterval::FiveMinutes,
                CandlestickInterval::FifteenMinutes,
                CandlestickInterval::OneHour,
                CandlestickInterval::FourHours,
                CandlestickInterval::OneDay,
            ],
            enable_depth_tracking: true,
            depth_levels: 20,
        }
    }
}

/// Candlestick interval enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandlestickInterval {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
    SixHours,
    EightHours,
    TwelveHours,
    OneDay,
    ThreeDays,
    OneWeek,
    OneMonth,
}

impl CandlestickInterval {
    /// Get duration in seconds
    pub fn duration_seconds(&self) -> u64 {
        match self {
            CandlestickInterval::OneMinute => 60,
            CandlestickInterval::FiveMinutes => 300,
            CandlestickInterval::FifteenMinutes => 900,
            CandlestickInterval::ThirtyMinutes => 1800,
            CandlestickInterval::OneHour => 3600,
            CandlestickInterval::TwoHours => 7200,
            CandlestickInterval::FourHours => 14400,
            CandlestickInterval::SixHours => 21600,
            CandlestickInterval::EightHours => 28800,
            CandlestickInterval::TwelveHours => 43200,
            CandlestickInterval::OneDay => 86400,
            CandlestickInterval::ThreeDays => 259200,
            CandlestickInterval::OneWeek => 604800,
            CandlestickInterval::OneMonth => 2592000, // 30 days
        }
    }
}

/// Candlestick data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    pub trading_pair: TradingPair,
    pub interval: CandlestickInterval,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub close_price: Decimal,
    pub volume: Decimal,
    pub quote_volume: Decimal,
    pub trade_count: u64,
    pub taker_buy_volume: Decimal,
    pub taker_buy_quote_volume: Decimal,
}

impl Candlestick {
    /// Create a new candlestick from the first trade
    pub fn new(trade: &Trade, interval: CandlestickInterval) -> Self {
        let interval_duration = Duration::seconds(interval.duration_seconds() as i64);
        let open_time = Self::align_timestamp(trade.executed_at, interval);
        let close_time = open_time + interval_duration - Duration::milliseconds(1);

        Self {
            trading_pair: trade.trading_pair.clone(),
            interval,
            open_time,
            close_time,
            open_price: trade.price,
            high_price: trade.price,
            low_price: trade.price,
            close_price: trade.price,
            volume: trade.quantity,
            quote_volume: trade.price * trade.quantity,
            trade_count: 1,
            taker_buy_volume: trade.quantity, // Assuming buyer is taker
            taker_buy_quote_volume: trade.price * trade.quantity,
        }
    }

    /// Update candlestick with a new trade
    pub fn update_with_trade(&mut self, trade: &Trade) {
        self.high_price = self.high_price.max(trade.price);
        self.low_price = self.low_price.min(trade.price);
        self.close_price = trade.price;
        self.volume += trade.quantity;
        self.quote_volume += trade.price * trade.quantity;
        self.trade_count += 1;
        
        // Assuming buyer is always taker (simplified)
        self.taker_buy_volume += trade.quantity;
        self.taker_buy_quote_volume += trade.price * trade.quantity;
    }

    /// Align timestamp to interval boundary
    fn align_timestamp(timestamp: DateTime<Utc>, interval: CandlestickInterval) -> DateTime<Utc> {
        let duration_seconds = interval.duration_seconds() as i64;
        let timestamp_seconds = timestamp.timestamp();
        let aligned_seconds = (timestamp_seconds / duration_seconds) * duration_seconds;
        DateTime::from_timestamp(aligned_seconds, 0).unwrap_or(timestamp)
    }
}

/// Market ticker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub trading_pair: TradingPair,
    pub last_price: Decimal,
    pub price_change: Decimal,
    pub price_change_percent: Decimal,
    pub high_price_24h: Decimal,
    pub low_price_24h: Decimal,
    pub volume_24h: Decimal,
    pub quote_volume_24h: Decimal,
    pub open_price_24h: Decimal,
    pub trade_count_24h: u64,
    pub bid_price: Option<Decimal>,
    pub ask_price: Option<Decimal>,
    pub bid_quantity: Option<Decimal>,
    pub ask_quantity: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
}

/// Order book depth snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthSnapshot {
    pub trading_pair: TradingPair,
    pub bids: Vec<DepthLevel>,
    pub asks: Vec<DepthLevel>,
    pub last_update_id: u64,
    pub timestamp: DateTime<Utc>,
}

/// Depth level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// Trade data for market data feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    pub id: Uuid,
    pub trading_pair: TradingPair,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
    pub is_buyer_maker: bool,
}

impl From<&Trade> for TradeData {
    fn from(trade: &Trade) -> Self {
        Self {
            id: trade.id,
            trading_pair: trade.trading_pair.clone(),
            price: trade.price,
            quantity: trade.quantity,
            timestamp: trade.executed_at,
            is_buyer_maker: false, // Simplified assumption
        }
    }
}

/// Market statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    pub trading_pair: TradingPair,
    pub volume_24h: Decimal,
    pub quote_volume_24h: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub price_change_24h: Decimal,
    pub price_change_percent_24h: Decimal,
    pub trade_count_24h: u64,
    pub average_price_24h: Decimal,
    pub vwap_24h: Decimal, // Volume Weighted Average Price
    pub last_updated: DateTime<Utc>,
}

/// Market data service trait
#[async_trait]
pub trait MarketDataService: Send + Sync {
    /// Get current ticker for a trading pair
    async fn get_ticker(&self, trading_pair: TradingPair) -> TradingResult<Option<Ticker>>;
    
    /// Get all tickers
    async fn get_all_tickers(&self) -> TradingResult<Vec<Ticker>>;
    
    /// Get candlestick data
    async fn get_candlesticks(
        &self,
        trading_pair: TradingPair,
        interval: CandlestickInterval,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> TradingResult<Vec<Candlestick>>;
    
    /// Get order book depth
    async fn get_depth(&self, trading_pair: TradingPair, limit: Option<usize>) -> TradingResult<DepthSnapshot>;
    
    /// Get recent trades
    async fn get_recent_trades(
        &self,
        trading_pair: TradingPair,
        limit: Option<usize>,
    ) -> TradingResult<Vec<TradeData>>;
    
    /// Get market statistics
    async fn get_market_stats(&self, trading_pair: TradingPair) -> TradingResult<MarketStats>;
    
    /// Subscribe to real-time market data
    async fn subscribe_to_feed(&self, trading_pair: TradingPair, subscriber_id: String) -> TradingResult<()>;
    
    /// Unsubscribe from market data feed
    async fn unsubscribe_from_feed(&self, trading_pair: TradingPair, subscriber_id: String) -> TradingResult<()>;
    
    /// Update market data with new trade
    async fn update_with_trade(&self, trade: Trade) -> TradingResult<()>;
    
    /// Health check
    async fn health_check(&self) -> TradingResult<MarketDataHealthStatus>;
}

/// Market data feed for real-time streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataFeed {
    pub feed_type: FeedType,
    pub trading_pair: TradingPair,
    pub data: FeedData,
    pub timestamp: DateTime<Utc>,
}

/// Feed type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedType {
    Trade,
    Ticker,
    Depth,
    Candlestick,
}

/// Feed data enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedData {
    Trade(TradeData),
    Ticker(Ticker),
    Depth(DepthSnapshot),
    Candlestick(Candlestick),
}

/// Market data health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataHealthStatus {
    pub status: String,
    pub active_feeds: u64,
    pub total_subscribers: u64,
    pub updates_per_second: f64,
    pub data_latency_ms: u64,
    pub storage_usage_mb: u64,
    pub last_update: DateTime<Utc>,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candlestick_interval_duration() {
        assert_eq!(CandlestickInterval::OneMinute.duration_seconds(), 60);
        assert_eq!(CandlestickInterval::OneHour.duration_seconds(), 3600);
        assert_eq!(CandlestickInterval::OneDay.duration_seconds(), 86400);
    }

    #[test]
    fn test_candlestick_creation() {
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let trade = Trade {
            id: Uuid::new_v4(),
            trading_pair: trading_pair.clone(),
            buyer_order_id: Uuid::new_v4(),
            seller_order_id: Uuid::new_v4(),
            buyer_user_id: "buyer".to_string(),
            seller_user_id: "seller".to_string(),
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
            buyer_fee: Decimal::new(50, 0),
            seller_fee: Decimal::new(50, 0),
            executed_at: Utc::now(),
        };

        let candlestick = Candlestick::new(&trade, CandlestickInterval::OneMinute);
        
        assert_eq!(candlestick.trading_pair, trading_pair);
        assert_eq!(candlestick.open_price, Decimal::new(50000, 0));
        assert_eq!(candlestick.close_price, Decimal::new(50000, 0));
        assert_eq!(candlestick.volume, Decimal::new(1, 0));
        assert_eq!(candlestick.trade_count, 1);
    }

    #[test]
    fn test_trade_data_conversion() {
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let trade = Trade {
            id: Uuid::new_v4(),
            trading_pair: trading_pair.clone(),
            buyer_order_id: Uuid::new_v4(),
            seller_order_id: Uuid::new_v4(),
            buyer_user_id: "buyer".to_string(),
            seller_user_id: "seller".to_string(),
            price: Decimal::new(50000, 0),
            quantity: Decimal::new(1, 0),
            buyer_fee: Decimal::new(50, 0),
            seller_fee: Decimal::new(50, 0),
            executed_at: Utc::now(),
        };

        let trade_data = TradeData::from(&trade);
        
        assert_eq!(trade_data.id, trade.id);
        assert_eq!(trade_data.trading_pair, trading_pair);
        assert_eq!(trade_data.price, Decimal::new(50000, 0));
        assert_eq!(trade_data.quantity, Decimal::new(1, 0));
    }
}
