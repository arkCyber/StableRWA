// =====================================================================================
// File: core-trading/src/order_book.rs
// Description: Order book management system for RWA trading
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};

use crate::{
    error::{TradingError, TradingResult},
    types::{OrderSide, OrderType, OrderStatus, TradingPair},
};

/// Order book configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookConfig {
    /// Maximum number of price levels to maintain
    pub max_price_levels: usize,
    /// Minimum order size
    pub min_order_size: Decimal,
    /// Maximum order size
    pub max_order_size: Decimal,
    /// Price tick size
    pub tick_size: Decimal,
    /// Quantity step size
    pub step_size: Decimal,
    /// Enable order aggregation
    pub enable_aggregation: bool,
    /// Maximum orders per price level
    pub max_orders_per_level: usize,
    /// Order book depth for snapshots
    pub snapshot_depth: usize,
}

impl Default for OrderBookConfig {
    fn default() -> Self {
        Self {
            max_price_levels: 1000,
            min_order_size: Decimal::new(1, 6), // 0.000001
            max_order_size: Decimal::new(1000000, 0), // 1,000,000
            tick_size: Decimal::new(1, 8), // 0.00000001
            step_size: Decimal::new(1, 8), // 0.00000001
            enable_aggregation: true,
            max_orders_per_level: 100,
            snapshot_depth: 20,
        }
    }
}

/// Order structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: String,
    pub trading_pair: TradingPair,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl Order {
    /// Create a new order
    pub fn new(
        user_id: String,
        trading_pair: TradingPair,
        side: OrderSide,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            trading_pair,
            side,
            order_type,
            price,
            quantity,
            filled_quantity: Decimal::ZERO,
            remaining_quantity: quantity,
            status: OrderStatus::Pending,
            time_in_force: TimeInForce::GTC,
            created_at: now,
            updated_at: now,
            expires_at: None,
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

    /// Update filled quantity
    pub fn update_filled(&mut self, filled_qty: Decimal) {
        self.filled_quantity += filled_qty;
        self.remaining_quantity = self.quantity - self.filled_quantity;
        self.updated_at = Utc::now();
        
        if self.is_filled() {
            self.status = OrderStatus::Filled;
        } else if self.is_partially_filled() {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
}

/// Time in force enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good Till Cancelled
    GTC,
    /// Immediate Or Cancel
    IOC,
    /// Fill Or Kill
    FOK,
    /// Good Till Time
    GTT,
}

/// Price level in order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: Decimal,
    pub order_count: u32,
    pub orders: Vec<Uuid>,
}

impl PriceLevel {
    /// Create a new price level
    pub fn new(price: Decimal) -> Self {
        Self {
            price,
            quantity: Decimal::ZERO,
            order_count: 0,
            orders: Vec::new(),
        }
    }

    /// Add order to price level
    pub fn add_order(&mut self, order_id: Uuid, quantity: Decimal) {
        self.orders.push(order_id);
        self.quantity += quantity;
        self.order_count += 1;
    }

    /// Remove order from price level
    pub fn remove_order(&mut self, order_id: Uuid, quantity: Decimal) {
        if let Some(pos) = self.orders.iter().position(|&id| id == order_id) {
            self.orders.remove(pos);
            self.quantity -= quantity;
            self.order_count -= 1;
        }
    }

    /// Check if price level is empty
    pub fn is_empty(&self) -> bool {
        self.order_count == 0 || self.quantity <= Decimal::ZERO
    }
}

/// Order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub trading_pair: TradingPair,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub last_update_id: u64,
    pub timestamp: DateTime<Utc>,
}

/// Order book manager
pub struct OrderBookManager {
    config: OrderBookConfig,
    trading_pair: TradingPair,
    bids: BTreeMap<Decimal, PriceLevel>, // Price -> PriceLevel (descending)
    asks: BTreeMap<Decimal, PriceLevel>, // Price -> PriceLevel (ascending)
    orders: HashMap<Uuid, Order>,
    last_update_id: u64,
}

impl OrderBookManager {
    /// Create a new order book manager
    pub fn new(config: OrderBookConfig, trading_pair: TradingPair) -> Self {
        Self {
            config,
            trading_pair,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            last_update_id: 0,
        }
    }

    /// Add order to order book
    pub fn add_order(&mut self, mut order: Order) -> TradingResult<()> {
        // Validate order
        self.validate_order(&order)?;

        // Update order status
        order.status = OrderStatus::Open;
        order.updated_at = Utc::now();

        let order_id = order.id;
        let price = order.price;
        let quantity = order.remaining_quantity;
        let side = order.side;

        // Add to orders map
        self.orders.insert(order_id, order);

        // Add to appropriate side of order book
        match side {
            OrderSide::Buy => {
                let price_level = self.bids.entry(price).or_insert_with(|| PriceLevel::new(price));
                price_level.add_order(order_id, quantity);
            }
            OrderSide::Sell => {
                let price_level = self.asks.entry(price).or_insert_with(|| PriceLevel::new(price));
                price_level.add_order(order_id, quantity);
            }
        }

        self.last_update_id += 1;
        Ok(())
    }

    /// Remove order from order book
    pub fn remove_order(&mut self, order_id: Uuid) -> TradingResult<Option<Order>> {
        if let Some(order) = self.orders.remove(&order_id) {
            let price = order.price;
            let quantity = order.remaining_quantity;
            let side = order.side;

            // Remove from appropriate side
            match side {
                OrderSide::Buy => {
                    if let Some(price_level) = self.bids.get_mut(&price) {
                        price_level.remove_order(order_id, quantity);
                        if price_level.is_empty() {
                            self.bids.remove(&price);
                        }
                    }
                }
                OrderSide::Sell => {
                    if let Some(price_level) = self.asks.get_mut(&price) {
                        price_level.remove_order(order_id, quantity);
                        if price_level.is_empty() {
                            self.asks.remove(&price);
                        }
                    }
                }
            }

            self.last_update_id += 1;
            Ok(Some(order))
        } else {
            Ok(None)
        }
    }

    /// Get order by ID
    pub fn get_order(&self, order_id: Uuid) -> Option<&Order> {
        self.orders.get(&order_id)
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

    /// Create order book snapshot
    pub fn snapshot(&self) -> OrderBookSnapshot {
        let bids: Vec<PriceLevel> = self.bids
            .values()
            .rev()
            .take(self.config.snapshot_depth)
            .cloned()
            .collect();

        let asks: Vec<PriceLevel> = self.asks
            .values()
            .take(self.config.snapshot_depth)
            .cloned()
            .collect();

        OrderBookSnapshot {
            trading_pair: self.trading_pair.clone(),
            bids,
            asks,
            last_update_id: self.last_update_id,
            timestamp: Utc::now(),
        }
    }

    /// Validate order
    fn validate_order(&self, order: &Order) -> TradingResult<()> {
        if order.quantity < self.config.min_order_size {
            return Err(TradingError::validation_error(
                "quantity",
                format!("Order quantity {} is below minimum {}", order.quantity, self.config.min_order_size),
            ));
        }

        if order.quantity > self.config.max_order_size {
            return Err(TradingError::validation_error(
                "quantity",
                format!("Order quantity {} exceeds maximum {}", order.quantity, self.config.max_order_size),
            ));
        }

        if order.price <= Decimal::ZERO {
            return Err(TradingError::validation_error(
                "price",
                "Order price must be positive",
            ));
        }

        Ok(())
    }
}

/// Order book service trait
#[async_trait]
pub trait OrderBookService: Send + Sync {
    /// Add order to order book
    async fn add_order(&self, order: Order) -> TradingResult<()>;
    
    /// Remove order from order book
    async fn remove_order(&self, order_id: Uuid) -> TradingResult<Option<Order>>;
    
    /// Get order by ID
    async fn get_order(&self, order_id: Uuid) -> TradingResult<Option<Order>>;
    
    /// Get order book snapshot
    async fn get_snapshot(&self, trading_pair: TradingPair) -> TradingResult<OrderBookSnapshot>;
    
    /// Get best bid and ask
    async fn get_best_prices(&self, trading_pair: TradingPair) -> TradingResult<(Option<Decimal>, Option<Decimal>)>;
    
    /// Get market depth
    async fn get_market_depth(&self, trading_pair: TradingPair, depth: usize) -> TradingResult<OrderBookSnapshot>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let order = Order::new(
            "user123".to_string(),
            TradingPair::new("BTC".to_string(), "USD".to_string()),
            OrderSide::Buy,
            OrderType::Limit,
            Decimal::new(50000, 0), // $50,000
            Decimal::new(1, 0), // 1 BTC
        );

        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.price, Decimal::new(50000, 0));
        assert_eq!(order.quantity, Decimal::new(1, 0));
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(!order.is_filled());
    }

    #[test]
    fn test_order_book_manager() {
        let config = OrderBookConfig::default();
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let mut order_book = OrderBookManager::new(config, trading_pair.clone());

        let buy_order = Order::new(
            "user1".to_string(),
            trading_pair.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            Decimal::new(49000, 0),
            Decimal::new(1, 0),
        );

        let sell_order = Order::new(
            "user2".to_string(),
            trading_pair,
            OrderSide::Sell,
            OrderType::Limit,
            Decimal::new(51000, 0),
            Decimal::new(1, 0),
        );

        assert!(order_book.add_order(buy_order).is_ok());
        assert!(order_book.add_order(sell_order).is_ok());

        assert_eq!(order_book.best_bid(), Some(Decimal::new(49000, 0)));
        assert_eq!(order_book.best_ask(), Some(Decimal::new(51000, 0)));
        assert_eq!(order_book.spread(), Some(Decimal::new(2000, 0)));
        assert_eq!(order_book.mid_price(), Some(Decimal::new(50000, 0)));
    }
}
