// =====================================================================================
// File: core-trading/src/matching.rs
// Description: Order matching engine for RWA trading system
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::cmp::Ordering;

use crate::{
    error::{TradingError, TradingResult},
    types::{OrderSide, OrderType, TradingPair},
    order_book::{Order, OrderBookManager},
};

/// Matching engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingConfig {
    /// Enable price-time priority
    pub price_time_priority: bool,
    /// Enable pro-rata matching
    pub pro_rata_matching: bool,
    /// Minimum match size
    pub min_match_size: Decimal,
    /// Maximum matches per order
    pub max_matches_per_order: u32,
    /// Enable self-trade prevention
    pub self_trade_prevention: bool,
    /// Matching fee percentage
    pub matching_fee_percentage: Decimal,
    /// Enable partial fills
    pub allow_partial_fills: bool,
}

impl Default for MatchingConfig {
    fn default() -> Self {
        Self {
            price_time_priority: true,
            pro_rata_matching: false,
            min_match_size: Decimal::new(1, 8), // 0.00000001
            max_matches_per_order: 100,
            self_trade_prevention: true,
            matching_fee_percentage: Decimal::new(10, 4), // 0.10%
            allow_partial_fills: true,
        }
    }
}

/// Trade execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub trading_pair: TradingPair,
    pub buyer_order_id: Uuid,
    pub seller_order_id: Uuid,
    pub buyer_user_id: String,
    pub seller_user_id: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub buyer_fee: Decimal,
    pub seller_fee: Decimal,
    pub executed_at: DateTime<Utc>,
}

impl Trade {
    /// Create a new trade
    pub fn new(
        trading_pair: TradingPair,
        buyer_order: &Order,
        seller_order: &Order,
        price: Decimal,
        quantity: Decimal,
        buyer_fee: Decimal,
        seller_fee: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            trading_pair,
            buyer_order_id: buyer_order.id,
            seller_order_id: seller_order.id,
            buyer_user_id: buyer_order.user_id.clone(),
            seller_user_id: seller_order.user_id.clone(),
            price,
            quantity,
            buyer_fee,
            seller_fee,
            executed_at: Utc::now(),
        }
    }

    /// Calculate total value of the trade
    pub fn total_value(&self) -> Decimal {
        self.price * self.quantity
    }
}

/// Match result containing all trades from a matching operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub trades: Vec<Trade>,
    pub updated_orders: Vec<Order>,
    pub total_volume: Decimal,
    pub total_fees: Decimal,
    pub matched_at: DateTime<Utc>,
}

impl MatchResult {
    /// Create a new empty match result
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            updated_orders: Vec::new(),
            total_volume: Decimal::ZERO,
            total_fees: Decimal::ZERO,
            matched_at: Utc::now(),
        }
    }

    /// Add a trade to the match result
    pub fn add_trade(&mut self, trade: Trade) {
        self.total_volume += trade.total_value();
        self.total_fees += trade.buyer_fee + trade.seller_fee;
        self.trades.push(trade);
    }

    /// Add an updated order
    pub fn add_updated_order(&mut self, order: Order) {
        self.updated_orders.push(order);
    }

    /// Check if any matches occurred
    pub fn has_matches(&self) -> bool {
        !self.trades.is_empty()
    }
}

/// Matching engine implementation
pub struct MatchingEngine {
    config: MatchingConfig,
}

impl MatchingEngine {
    /// Create a new matching engine
    pub fn new(config: MatchingConfig) -> Self {
        Self { config }
    }

    /// Match an incoming order against the order book
    pub fn match_order(
        &self,
        incoming_order: &mut Order,
        order_book: &mut OrderBookManager,
    ) -> TradingResult<MatchResult> {
        let mut result = MatchResult::new();

        match incoming_order.side {
            OrderSide::Buy => {
                self.match_buy_order(incoming_order, order_book, &mut result)?;
            }
            OrderSide::Sell => {
                self.match_sell_order(incoming_order, order_book, &mut result)?;
            }
        }

        Ok(result)
    }

    /// Match a buy order against sell orders in the book
    fn match_buy_order(
        &self,
        buy_order: &mut Order,
        order_book: &mut OrderBookManager,
        result: &mut MatchResult,
    ) -> TradingResult<()> {
        while buy_order.remaining_quantity > Decimal::ZERO {
            // Get best ask price
            let best_ask = match order_book.best_ask() {
                Some(price) => price,
                None => break, // No more sell orders
            };

            // Check if buy order price is high enough
            if buy_order.price < best_ask {
                break; // No match possible
            }

            // Find matching sell orders at the best ask price
            let matching_orders = self.find_matching_orders(order_book, OrderSide::Sell, best_ask)?;
            
            if matching_orders.is_empty() {
                break;
            }

            // Execute matches
            for sell_order_id in matching_orders {
                if buy_order.remaining_quantity <= Decimal::ZERO {
                    break;
                }

                let sell_order = match order_book.get_order(sell_order_id) {
                    Some(order) => order.clone(),
                    None => continue,
                };

                // Check self-trade prevention
                if self.config.self_trade_prevention && buy_order.user_id == sell_order.user_id {
                    continue;
                }

                // Calculate match quantity
                let match_quantity = buy_order.remaining_quantity.min(sell_order.remaining_quantity);
                
                if match_quantity < self.config.min_match_size {
                    continue;
                }

                // Execute the trade
                let trade = self.execute_trade(buy_order, &sell_order, best_ask, match_quantity)?;
                result.add_trade(trade);

                // Update orders
                buy_order.update_filled(match_quantity);
                let mut updated_sell_order = sell_order;
                updated_sell_order.update_filled(match_quantity);

                result.add_updated_order(buy_order.clone());
                result.add_updated_order(updated_sell_order.clone());

                // Remove filled sell order from book
                if updated_sell_order.is_filled() {
                    order_book.remove_order(sell_order_id)?;
                }

                // Check if we've reached max matches
                if result.trades.len() >= self.config.max_matches_per_order as usize {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Match a sell order against buy orders in the book
    fn match_sell_order(
        &self,
        sell_order: &mut Order,
        order_book: &mut OrderBookManager,
        result: &mut MatchResult,
    ) -> TradingResult<()> {
        while sell_order.remaining_quantity > Decimal::ZERO {
            // Get best bid price
            let best_bid = match order_book.best_bid() {
                Some(price) => price,
                None => break, // No more buy orders
            };

            // Check if sell order price is low enough
            if sell_order.price > best_bid {
                break; // No match possible
            }

            // Find matching buy orders at the best bid price
            let matching_orders = self.find_matching_orders(order_book, OrderSide::Buy, best_bid)?;
            
            if matching_orders.is_empty() {
                break;
            }

            // Execute matches
            for buy_order_id in matching_orders {
                if sell_order.remaining_quantity <= Decimal::ZERO {
                    break;
                }

                let buy_order = match order_book.get_order(buy_order_id) {
                    Some(order) => order.clone(),
                    None => continue,
                };

                // Check self-trade prevention
                if self.config.self_trade_prevention && sell_order.user_id == buy_order.user_id {
                    continue;
                }

                // Calculate match quantity
                let match_quantity = sell_order.remaining_quantity.min(buy_order.remaining_quantity);
                
                if match_quantity < self.config.min_match_size {
                    continue;
                }

                // Execute the trade
                let trade = self.execute_trade(&buy_order, sell_order, best_bid, match_quantity)?;
                result.add_trade(trade);

                // Update orders
                sell_order.update_filled(match_quantity);
                let mut updated_buy_order = buy_order;
                updated_buy_order.update_filled(match_quantity);

                result.add_updated_order(sell_order.clone());
                result.add_updated_order(updated_buy_order.clone());

                // Remove filled buy order from book
                if updated_buy_order.is_filled() {
                    order_book.remove_order(buy_order_id)?;
                }

                // Check if we've reached max matches
                if result.trades.len() >= self.config.max_matches_per_order as usize {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Find matching orders at a specific price level
    fn find_matching_orders(
        &self,
        order_book: &OrderBookManager,
        side: OrderSide,
        price: Decimal,
    ) -> TradingResult<Vec<Uuid>> {
        // This is a simplified implementation
        // In a real system, this would access the order book's internal structure
        // to get orders at the specific price level
        Ok(Vec::new())
    }

    /// Execute a trade between two orders
    fn execute_trade(
        &self,
        buy_order: &Order,
        sell_order: &Order,
        price: Decimal,
        quantity: Decimal,
    ) -> TradingResult<Trade> {
        // Calculate fees
        let trade_value = price * quantity;
        let buyer_fee = trade_value * self.config.matching_fee_percentage;
        let seller_fee = trade_value * self.config.matching_fee_percentage;

        let trade = Trade::new(
            buy_order.trading_pair.clone(),
            buy_order,
            sell_order,
            price,
            quantity,
            buyer_fee,
            seller_fee,
        );

        Ok(trade)
    }
}

/// Matching engine service trait
#[async_trait]
pub trait MatchingEngineService: Send + Sync {
    /// Match an order against the order book
    async fn match_order(&self, order: Order) -> TradingResult<MatchResult>;
    
    /// Get recent trades for a trading pair
    async fn get_recent_trades(&self, trading_pair: TradingPair, limit: usize) -> TradingResult<Vec<Trade>>;
    
    /// Get trade history for a user
    async fn get_user_trades(&self, user_id: &str, limit: usize) -> TradingResult<Vec<Trade>>;
    
    /// Get matching statistics
    async fn get_matching_stats(&self) -> TradingResult<MatchingStats>;
}

/// Matching engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingStats {
    pub total_trades: u64,
    pub total_volume: Decimal,
    pub total_fees: Decimal,
    pub average_trade_size: Decimal,
    pub trades_per_second: f64,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OrderStatus;

    #[test]
    fn test_trade_creation() {
        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        
        let buy_order = Order::new(
            "buyer".to_string(),
            trading_pair.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
        );

        let sell_order = Order::new(
            "seller".to_string(),
            trading_pair.clone(),
            OrderSide::Sell,
            OrderType::Limit,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
        );

        let trade = Trade::new(
            trading_pair,
            &buy_order,
            &sell_order,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
            Decimal::new(50, 0), // $50 fee
            Decimal::new(50, 0), // $50 fee
        );

        assert_eq!(trade.price, Decimal::new(50000, 0));
        assert_eq!(trade.quantity, Decimal::new(1, 0));
        assert_eq!(trade.total_value(), Decimal::new(50000, 0));
        assert_eq!(trade.buyer_user_id, "buyer");
        assert_eq!(trade.seller_user_id, "seller");
    }

    #[test]
    fn test_match_result() {
        let mut result = MatchResult::new();
        assert!(!result.has_matches());
        assert_eq!(result.total_volume, Decimal::ZERO);
        assert_eq!(result.total_fees, Decimal::ZERO);

        let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
        let buy_order = Order::new(
            "buyer".to_string(),
            trading_pair.clone(),
            OrderSide::Buy,
            OrderType::Limit,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
        );

        let sell_order = Order::new(
            "seller".to_string(),
            trading_pair.clone(),
            OrderSide::Sell,
            OrderType::Limit,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
        );

        let trade = Trade::new(
            trading_pair,
            &buy_order,
            &sell_order,
            Decimal::new(50000, 0),
            Decimal::new(1, 0),
            Decimal::new(50, 0),
            Decimal::new(50, 0),
        );

        result.add_trade(trade);
        assert!(result.has_matches());
        assert_eq!(result.total_volume, Decimal::new(50000, 0));
        assert_eq!(result.total_fees, Decimal::new(100, 0));
    }

    #[test]
    fn test_matching_config_default() {
        let config = MatchingConfig::default();
        assert!(config.price_time_priority);
        assert!(!config.pro_rata_matching);
        assert!(config.self_trade_prevention);
        assert!(config.allow_partial_fills);
        assert_eq!(config.matching_fee_percentage, Decimal::new(10, 4));
    }
}
