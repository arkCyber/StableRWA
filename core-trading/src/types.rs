// =====================================================================================
// File: core-trading/src/types.rs
// Description: Core types for trading system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// Trading pair identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradingPair {
    pub base_asset: String,
    pub quote_asset: String,
}

impl TradingPair {
    pub fn new(base: String, quote: String) -> Self {
        Self { base_asset: base, quote_asset: quote }
    }

    pub fn symbol(&self) -> String {
        format!("{}/{}", self.base_asset, self.quote_asset)
    }

    pub fn from_symbol(symbol: &str) -> Option<Self> {
        let parts: Vec<&str> = symbol.split('/').collect();
        if parts.len() == 2 {
            Some(Self::new(parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }
}

/// Order structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: String,
    pub trading_pair: TradingPair,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub filled_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub client_order_id: Option<String>,
    pub stop_price: Option<Decimal>,
    pub iceberg_quantity: Option<Decimal>,
    pub fees_paid: Decimal,
    pub average_fill_price: Option<Decimal>,
}

impl Order {
    /// Create a new order
    pub fn new(
        user_id: String,
        trading_pair: TradingPair,
        order_type: OrderType,
        side: OrderSide,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            trading_pair,
            order_type,
            side,
            quantity,
            price,
            filled_quantity: Decimal::ZERO,
            remaining_quantity: quantity,
            status: OrderStatus::Pending,
            time_in_force: TimeInForce::GTC,
            created_at: now,
            updated_at: now,
            expires_at: None,
            client_order_id: None,
            stop_price: None,
            iceberg_quantity: None,
            fees_paid: Decimal::ZERO,
            average_fill_price: None,
        }
    }

    /// Check if order is fully filled
    pub fn is_filled(&self) -> bool {
        self.filled_quantity >= self.quantity
    }

    /// Check if order is partially filled
    pub fn is_partially_filled(&self) -> bool {
        self.filled_quantity > Decimal::ZERO && self.filled_quantity < self.quantity
    }

    /// Check if order can be matched
    pub fn can_be_matched(&self) -> bool {
        matches!(self.status, OrderStatus::Open | OrderStatus::PartiallyFilled)
    }

    /// Check if order has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Update order with fill information
    pub fn apply_fill(&mut self, fill_quantity: Decimal, fill_price: Decimal) {
        self.filled_quantity += fill_quantity;
        self.remaining_quantity = self.quantity - self.filled_quantity;
        self.updated_at = Utc::now();

        // Update average fill price
        if let Some(avg_price) = self.average_fill_price {
            let total_filled_value = avg_price * (self.filled_quantity - fill_quantity) + fill_price * fill_quantity;
            self.average_fill_price = Some(total_filled_value / self.filled_quantity);
        } else {
            self.average_fill_price = Some(fill_price);
        }

        // Update status
        if self.is_filled() {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
}

/// Order type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Market order - execute immediately at best available price
    Market,
    /// Limit order - execute only at specified price or better
    Limit,
    /// Stop order - becomes market order when stop price is reached
    Stop,
    /// Stop-limit order - becomes limit order when stop price is reached
    StopLimit,
    /// Iceberg order - large order split into smaller visible portions
    Iceberg,
    /// Fill or kill - execute completely or cancel
    FOK,
    /// Immediate or cancel - execute immediately, cancel remainder
    IOC,
}

/// Order side enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    /// Get the opposite side
    pub fn opposite(&self) -> Self {
        match self {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }
}

/// Order status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order submitted but not yet processed
    Pending,
    /// Order is active in the order book
    Open,
    /// Order has been partially filled
    PartiallyFilled,
    /// Order has been completely filled
    Filled,
    /// Order has been cancelled
    Cancelled,
    /// Order was rejected
    Rejected,
    /// Order has expired
    Expired,
}

impl OrderStatus {
    /// Check if order is in a final state
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected | OrderStatus::Expired
        )
    }

    /// Check if order is active
    pub fn is_active(&self) -> bool {
        matches!(self, OrderStatus::Open | OrderStatus::PartiallyFilled)
    }
}

/// Time in force enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good Till Cancelled
    GTC,
    /// Good Till Date
    GTD,
    /// Immediate Or Cancel
    IOC,
    /// Fill Or Kill
    FOK,
    /// At The Opening
    ATO,
    /// At The Close
    ATC,
}

/// Trade structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub trading_pair: TradingPair,
    pub buyer_order_id: Uuid,
    pub seller_order_id: Uuid,
    pub buyer_user_id: String,
    pub seller_user_id: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub value: Decimal,
    pub buyer_fee: Decimal,
    pub seller_fee: Decimal,
    pub status: TradeStatus,
    pub executed_at: DateTime<Utc>,
    pub settled_at: Option<DateTime<Utc>>,
    pub settlement_id: Option<Uuid>,
}

impl Trade {
    /// Create a new trade
    pub fn new(
        trading_pair: TradingPair,
        buyer_order: &Order,
        seller_order: &Order,
        quantity: Decimal,
        price: Decimal,
    ) -> Self {
        let value = quantity * price;
        Self {
            id: Uuid::new_v4(),
            trading_pair,
            buyer_order_id: buyer_order.id,
            seller_order_id: seller_order.id,
            buyer_user_id: buyer_order.user_id.clone(),
            seller_user_id: seller_order.user_id.clone(),
            quantity,
            price,
            value,
            buyer_fee: Decimal::ZERO,
            seller_fee: Decimal::ZERO,
            status: TradeStatus::Executed,
            executed_at: Utc::now(),
            settled_at: None,
            settlement_id: None,
        }
    }

    /// Check if trade is settled
    pub fn is_settled(&self) -> bool {
        matches!(self.status, TradeStatus::Settled)
    }
}

/// Trade status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeStatus {
    /// Trade has been executed
    Executed,
    /// Trade is being settled
    Settling,
    /// Trade has been settled
    Settled,
    /// Trade settlement failed
    SettlementFailed,
    /// Trade was cancelled
    Cancelled,
}

/// Price level in order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: Decimal,
    pub order_count: u32,
    pub last_updated: DateTime<Utc>,
}

impl PriceLevel {
    pub fn new(price: Decimal, quantity: Decimal) -> Self {
        Self {
            price,
            quantity,
            order_count: 1,
            last_updated: Utc::now(),
        }
    }

    pub fn add_quantity(&mut self, quantity: Decimal) {
        self.quantity += quantity;
        self.order_count += 1;
        self.last_updated = Utc::now();
    }

    pub fn remove_quantity(&mut self, quantity: Decimal) {
        self.quantity -= quantity;
        if self.order_count > 0 {
            self.order_count -= 1;
        }
        self.last_updated = Utc::now();
    }
}

/// Order book structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub trading_pair: TradingPair,
    pub bids: BTreeMap<Decimal, PriceLevel>, // Sorted descending by price
    pub asks: BTreeMap<Decimal, PriceLevel>, // Sorted ascending by price
    pub last_updated: DateTime<Utc>,
    pub sequence_number: u64,
}

impl OrderBook {
    pub fn new(trading_pair: TradingPair) -> Self {
        Self {
            trading_pair,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_updated: Utc::now(),
            sequence_number: 0,
        }
    }

    /// Get best bid price
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    /// Get best ask price
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

    /// Get spread
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Get mid price
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some((ask + bid) / Decimal::TWO),
            _ => None,
        }
    }

    /// Add order to book
    pub fn add_order(&mut self, order: &Order) {
        if let Some(price) = order.price {
            let levels = match order.side {
                OrderSide::Buy => &mut self.bids,
                OrderSide::Sell => &mut self.asks,
            };

            levels
                .entry(price)
                .and_modify(|level| level.add_quantity(order.remaining_quantity))
                .or_insert_with(|| PriceLevel::new(price, order.remaining_quantity));

            self.last_updated = Utc::now();
            self.sequence_number += 1;
        }
    }

    /// Remove order from book
    pub fn remove_order(&mut self, order: &Order) {
        if let Some(price) = order.price {
            let levels = match order.side {
                OrderSide::Buy => &mut self.bids,
                OrderSide::Sell => &mut self.asks,
            };

            if let Some(level) = levels.get_mut(&price) {
                level.remove_quantity(order.remaining_quantity);
                if level.quantity <= Decimal::ZERO {
                    levels.remove(&price);
                }
            }

            self.last_updated = Utc::now();
            self.sequence_number += 1;
        }
    }
}

/// Liquidity pool structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub id: Uuid,
    pub trading_pair: TradingPair,
    pub base_reserve: Decimal,
    pub quote_reserve: Decimal,
    pub total_liquidity: Decimal,
    pub fee_rate: Decimal,
    pub providers: Vec<LiquidityProvider>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active: bool,
}

impl LiquidityPool {
    /// Calculate current price based on reserves
    pub fn current_price(&self) -> Option<Decimal> {
        if self.base_reserve > Decimal::ZERO {
            Some(self.quote_reserve / self.base_reserve)
        } else {
            None
        }
    }

    /// Calculate output amount for a given input
    pub fn calculate_output(&self, input_amount: Decimal, input_is_base: bool) -> Option<Decimal> {
        let (input_reserve, output_reserve) = if input_is_base {
            (self.base_reserve, self.quote_reserve)
        } else {
            (self.quote_reserve, self.base_reserve)
        };

        if input_reserve <= Decimal::ZERO || output_reserve <= Decimal::ZERO {
            return None;
        }

        // Apply fee
        let input_with_fee = input_amount * (Decimal::ONE - self.fee_rate);
        
        // Constant product formula: x * y = k
        let output_amount = (output_reserve * input_with_fee) / (input_reserve + input_with_fee);
        
        Some(output_amount)
    }
}

/// Liquidity provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityProvider {
    pub user_id: String,
    pub liquidity_tokens: Decimal,
    pub share_percentage: Decimal,
    pub provided_at: DateTime<Utc>,
    pub rewards_earned: Decimal,
}

/// Market data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub trading_pair: TradingPair,
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub volume_quote: Decimal,
    pub trade_count: u64,
    pub vwap: Option<Decimal>, // Volume Weighted Average Price
}

impl MarketData {
    /// Calculate price change
    pub fn price_change(&self) -> Decimal {
        self.close - self.open
    }

    /// Calculate price change percentage
    pub fn price_change_percent(&self) -> Option<Decimal> {
        if self.open > Decimal::ZERO {
            Some((self.price_change() / self.open) * Decimal::ONE_HUNDRED)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_pair() {
        let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        assert_eq!(pair.symbol(), "BTC/USD");

        let parsed = TradingPair::from_symbol("ETH/BTC").unwrap();
        assert_eq!(parsed.base_asset, "ETH");
        assert_eq!(parsed.quote_asset, "BTC");

        assert!(TradingPair::from_symbol("invalid").is_none());
    }

    #[test]
    fn test_order_creation() {
        let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let order = Order::new(
            "user123".to_string(),
            pair,
            OrderType::Limit,
            OrderSide::Buy,
            Decimal::new(1, 0), // 1 BTC
            Some(Decimal::new(50000, 0)), // $50,000
        );

        assert_eq!(order.quantity, Decimal::new(1, 0));
        assert_eq!(order.remaining_quantity, Decimal::new(1, 0));
        assert_eq!(order.filled_quantity, Decimal::ZERO);
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(!order.is_filled());
        assert!(!order.is_partially_filled());
    }

    #[test]
    fn test_order_fill() {
        let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let mut order = Order::new(
            "user123".to_string(),
            pair,
            OrderType::Limit,
            OrderSide::Buy,
            Decimal::new(2, 0), // 2 BTC
            Some(Decimal::new(50000, 0)), // $50,000
        );

        // Partial fill
        order.apply_fill(Decimal::new(1, 0), Decimal::new(50000, 0));
        assert!(order.is_partially_filled());
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.filled_quantity, Decimal::new(1, 0));
        assert_eq!(order.remaining_quantity, Decimal::new(1, 0));

        // Complete fill
        order.apply_fill(Decimal::new(1, 0), Decimal::new(51000, 0));
        assert!(order.is_filled());
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.filled_quantity, Decimal::new(2, 0));
        assert_eq!(order.remaining_quantity, Decimal::ZERO);
        assert_eq!(order.average_fill_price, Some(Decimal::new(50500, 0))); // Average of 50000 and 51000
    }

    #[test]
    fn test_order_side() {
        assert_eq!(OrderSide::Buy.opposite(), OrderSide::Sell);
        assert_eq!(OrderSide::Sell.opposite(), OrderSide::Buy);
    }

    #[test]
    fn test_order_status() {
        assert!(OrderStatus::Filled.is_final());
        assert!(OrderStatus::Cancelled.is_final());
        assert!(!OrderStatus::Open.is_final());

        assert!(OrderStatus::Open.is_active());
        assert!(OrderStatus::PartiallyFilled.is_active());
        assert!(!OrderStatus::Filled.is_active());
    }

    #[test]
    fn test_trade_creation() {
        let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let buy_order = Order::new(
            "buyer".to_string(),
            pair.clone(),
            OrderType::Limit,
            OrderSide::Buy,
            Decimal::new(1, 0),
            Some(Decimal::new(50000, 0)),
        );
        let sell_order = Order::new(
            "seller".to_string(),
            pair.clone(),
            OrderType::Limit,
            OrderSide::Sell,
            Decimal::new(1, 0),
            Some(Decimal::new(50000, 0)),
        );

        let trade = Trade::new(
            pair,
            &buy_order,
            &sell_order,
            Decimal::new(1, 0),
            Decimal::new(50000, 0),
        );

        assert_eq!(trade.quantity, Decimal::new(1, 0));
        assert_eq!(trade.price, Decimal::new(50000, 0));
        assert_eq!(trade.value, Decimal::new(50000, 0));
        assert_eq!(trade.status, TradeStatus::Executed);
        assert!(!trade.is_settled());
    }

    #[test]
    fn test_order_book() {
        let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let mut book = OrderBook::new(pair.clone());

        let buy_order = Order::new(
            "user1".to_string(),
            pair.clone(),
            OrderType::Limit,
            OrderSide::Buy,
            Decimal::new(1, 0),
            Some(Decimal::new(49000, 0)),
        );

        let sell_order = Order::new(
            "user2".to_string(),
            pair,
            OrderType::Limit,
            OrderSide::Sell,
            Decimal::new(1, 0),
            Some(Decimal::new(51000, 0)),
        );

        book.add_order(&buy_order);
        book.add_order(&sell_order);

        assert_eq!(book.best_bid(), Some(Decimal::new(49000, 0)));
        assert_eq!(book.best_ask(), Some(Decimal::new(51000, 0)));
        assert_eq!(book.spread(), Some(Decimal::new(2000, 0)));
        assert_eq!(book.mid_price(), Some(Decimal::new(50000, 0)));
    }

    #[test]
    fn test_liquidity_pool() {
        let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let pool = LiquidityPool {
            id: Uuid::new_v4(),
            trading_pair: pair,
            base_reserve: Decimal::new(10, 0), // 10 BTC
            quote_reserve: Decimal::new(500000, 0), // $500,000
            total_liquidity: Decimal::new(1000, 0),
            fee_rate: Decimal::new(3, 3), // 0.3%
            providers: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            active: true,
        };

        assert_eq!(pool.current_price(), Some(Decimal::new(50000, 0))); // $50,000 per BTC

        // Test swap calculation
        let output = pool.calculate_output(Decimal::new(1, 0), true); // 1 BTC input
        assert!(output.is_some());
        assert!(output.unwrap() < Decimal::new(50000, 0)); // Should be less due to slippage and fees
    }

    #[test]
    fn test_market_data() {
        let pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let market_data = MarketData {
            trading_pair: pair,
            timestamp: Utc::now(),
            open: Decimal::new(50000, 0),
            high: Decimal::new(52000, 0),
            low: Decimal::new(48000, 0),
            close: Decimal::new(51000, 0),
            volume: Decimal::new(100, 0),
            volume_quote: Decimal::new(5100000, 0),
            trade_count: 1000,
            vwap: Some(Decimal::new(51000, 0)),
        };

        assert_eq!(market_data.price_change(), Decimal::new(1000, 0));
        assert_eq!(market_data.price_change_percent(), Some(Decimal::new(2, 0))); // 2%
    }
}
