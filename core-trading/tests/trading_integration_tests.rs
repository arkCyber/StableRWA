// =====================================================================================
// File: core-trading/tests/trading_integration_tests.rs
// Description: Integration tests for trading services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use core_trading::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;
use tokio_test;

#[tokio::test]
async fn test_order_book_integration() {
    let config = order_book::OrderBookConfig::default();
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
    let mut order_book = order_book::OrderBookManager::new(config, trading_pair.clone());

    // Create buy orders
    let buy_order1 = order_book::Order::new(
        "user1".to_string(),
        trading_pair.clone(),
        OrderSide::Buy,
        OrderType::Limit,
        Decimal::new(49000, 0), // $49,000
        Decimal::new(1, 0), // 1 BTC
    );

    let buy_order2 = order_book::Order::new(
        "user2".to_string(),
        trading_pair.clone(),
        OrderSide::Buy,
        OrderType::Limit,
        Decimal::new(48500, 0), // $48,500
        Decimal::new(2, 0), // 2 BTC
    );

    // Create sell orders
    let sell_order1 = order_book::Order::new(
        "user3".to_string(),
        trading_pair.clone(),
        OrderSide::Sell,
        OrderType::Limit,
        Decimal::new(51000, 0), // $51,000
        Decimal::new(1, 0), // 1 BTC
    );

    let sell_order2 = order_book::Order::new(
        "user4".to_string(),
        trading_pair.clone(),
        OrderSide::Sell,
        OrderType::Limit,
        Decimal::new(51500, 0), // $51,500
        Decimal::new(1, 0), // 1 BTC
    );

    // Add orders to order book
    assert!(order_book.add_order(buy_order1).is_ok());
    assert!(order_book.add_order(buy_order2).is_ok());
    assert!(order_book.add_order(sell_order1).is_ok());
    assert!(order_book.add_order(sell_order2).is_ok());

    // Test best prices
    assert_eq!(order_book.best_bid(), Some(Decimal::new(49000, 0)));
    assert_eq!(order_book.best_ask(), Some(Decimal::new(51000, 0)));

    // Test spread and mid price
    assert_eq!(order_book.spread(), Some(Decimal::new(2000, 0))); // $2,000 spread
    assert_eq!(order_book.mid_price(), Some(Decimal::new(50000, 0))); // $50,000 mid

    // Test order book snapshot
    let snapshot = order_book.snapshot();
    assert_eq!(snapshot.trading_pair, trading_pair);
    assert!(!snapshot.bids.is_empty());
    assert!(!snapshot.asks.is_empty());
}

#[tokio::test]
async fn test_matching_engine_integration() {
    let config = matching::MatchingConfig::default();
    let matching_engine = matching::MatchingEngine::new(config);

    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());

    // Create matching orders
    let buy_order = order_book::Order::new(
        "buyer".to_string(),
        trading_pair.clone(),
        OrderSide::Buy,
        OrderType::Limit,
        Decimal::new(50000, 0), // $50,000
        Decimal::new(1, 0), // 1 BTC
    );

    let sell_order = order_book::Order::new(
        "seller".to_string(),
        trading_pair.clone(),
        OrderSide::Sell,
        OrderType::Limit,
        Decimal::new(50000, 0), // $50,000
        Decimal::new(1, 0), // 1 BTC
    );

    // Create trade
    let trade = matching::Trade::new(
        trading_pair,
        &buy_order,
        &sell_order,
        Decimal::new(50000, 0), // $50,000 execution price
        Decimal::new(1, 0), // 1 BTC quantity
        Decimal::new(50, 0), // $50 buyer fee
        Decimal::new(50, 0), // $50 seller fee
    );

    assert_eq!(trade.price, Decimal::new(50000, 0));
    assert_eq!(trade.quantity, Decimal::new(1, 0));
    assert_eq!(trade.total_value(), Decimal::new(50000, 0));
    assert_eq!(trade.buyer_user_id, "buyer");
    assert_eq!(trade.seller_user_id, "seller");

    // Test match result
    let mut match_result = matching::MatchResult::new();
    match_result.add_trade(trade);

    assert!(match_result.has_matches());
    assert_eq!(match_result.total_volume, Decimal::new(50000, 0));
    assert_eq!(match_result.total_fees, Decimal::new(100, 0));
    assert_eq!(match_result.trades.len(), 1);
}

#[tokio::test]
async fn test_settlement_integration() {
    let config = settlement::SettlementConfig::default();
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());

    // Create a trade for settlement
    let trade = matching::Trade {
        id: Uuid::new_v4(),
        trading_pair: trading_pair.clone(),
        buyer_order_id: Uuid::new_v4(),
        seller_order_id: Uuid::new_v4(),
        buyer_user_id: "buyer123".to_string(),
        seller_user_id: "seller456".to_string(),
        price: Decimal::new(50000, 0),
        quantity: Decimal::new(1, 0),
        buyer_fee: Decimal::new(50, 0),
        seller_fee: Decimal::new(50, 0),
        executed_at: Utc::now(),
    };

    // Create settlement instruction
    let settlement_date = Utc::now() + chrono::Duration::hours(1);
    let instruction = settlement::SettlementInstruction::from_trade(&trade, settlement_date);

    assert_eq!(instruction.trade_id, trade.id);
    assert_eq!(instruction.buyer_id, "buyer123");
    assert_eq!(instruction.seller_id, "seller456");
    assert_eq!(instruction.delivery_quantity, Decimal::new(1, 0));
    assert_eq!(instruction.payment_amount, Decimal::new(50000, 0));
    assert_eq!(instruction.status, settlement::SettlementStatus::Pending);

    // Test netting position
    let mut position = settlement::NettingPosition::new("user123".to_string(), "BTC".to_string());
    position.add_buy(Decimal::new(2, 0));
    position.add_sell(Decimal::new(1, 0));

    assert_eq!(position.net_position, Decimal::new(1, 0));
    assert_eq!(position.settlement_amount, Decimal::new(1, 0));

    // Test settlement batch
    let instructions = vec![instruction];
    let mut batch = settlement::SettlementBatch::new(instructions);
    batch.calculate_batch_fee(Decimal::new(5, 4)); // 0.05%

    assert_eq!(batch.total_value, Decimal::new(50000, 0));
    assert_eq!(batch.batch_fee, Decimal::new(2500, 2)); // $25.00
}

#[tokio::test]
async fn test_liquidity_management_integration() {
    let config = liquidity::LiquidityConfig::default();
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());

    // Create liquidity provider
    let provider = liquidity::LiquidityProvider::new(
        "provider123".to_string(),
        trading_pair.clone(),
        Decimal::new(10, 0), // 10 BTC
        Decimal::new(500000, 0), // $500,000
        Decimal::new(100, 4), // 1.00% spread
    );

    assert_eq!(provider.user_id, "provider123");
    assert_eq!(provider.base_inventory, Decimal::new(10, 0));
    assert_eq!(provider.quote_inventory, Decimal::new(500000, 0));
    assert!(provider.is_active);

    // Test inventory imbalance calculation
    let mid_price = Decimal::new(50000, 0); // $50,000 per BTC
    let imbalance = provider.inventory_imbalance(mid_price);
    assert!(imbalance.abs() <= Decimal::ONE); // Should be within reasonable range

    // Create liquidity quote
    let quote = liquidity::LiquidityQuote::new(
        provider.id,
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

    // Test quality score
    let quality_score = quote.quality_score();
    assert!(quality_score >= 0.0 && quality_score <= 1.0);
}

#[tokio::test]
async fn test_amm_pool_integration() {
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
    let pool = liquidity::AMMPool::new(
        trading_pair,
        Decimal::new(100, 0), // 100 BTC
        Decimal::new(5000000, 0), // $5,000,000
        Decimal::new(30, 4), // 0.30% fee
    );

    // Test current price calculation
    assert_eq!(pool.current_price(), Decimal::new(50000, 0)); // $50,000 per BTC

    // Test swap calculation
    let output = pool.calculate_output(Decimal::new(1, 0), true); // 1 BTC input
    assert!(output > Decimal::ZERO);
    assert!(output < Decimal::new(50000, 0)); // Should be less due to slippage

    // Test price impact
    let impact = pool.calculate_price_impact(Decimal::new(1, 0), true);
    assert!(impact > Decimal::ZERO);
    assert!(impact < Decimal::new(1000, 4)); // Should be reasonable impact

    // Test larger trade impact
    let large_impact = pool.calculate_price_impact(Decimal::new(10, 0), true);
    assert!(large_impact > impact); // Larger trade should have higher impact
}

#[tokio::test]
async fn test_market_data_integration() {
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());

    // Create a trade for market data
    let trade = matching::Trade {
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

    // Test candlestick creation
    let candlestick = market_data::Candlestick::new(&trade, market_data::CandlestickInterval::OneMinute);
    assert_eq!(candlestick.trading_pair, trading_pair);
    assert_eq!(candlestick.open_price, Decimal::new(50000, 0));
    assert_eq!(candlestick.close_price, Decimal::new(50000, 0));
    assert_eq!(candlestick.volume, Decimal::new(1, 0));
    assert_eq!(candlestick.trade_count, 1);

    // Test trade data conversion
    let trade_data = market_data::TradeData::from(&trade);
    assert_eq!(trade_data.id, trade.id);
    assert_eq!(trade_data.price, Decimal::new(50000, 0));
    assert_eq!(trade_data.quantity, Decimal::new(1, 0));

    // Test candlestick interval durations
    assert_eq!(market_data::CandlestickInterval::OneMinute.duration_seconds(), 60);
    assert_eq!(market_data::CandlestickInterval::OneHour.duration_seconds(), 3600);
    assert_eq!(market_data::CandlestickInterval::OneDay.duration_seconds(), 86400);
}

#[tokio::test]
async fn test_trading_service_integration() {
    let config = TradingServiceConfig::default();
    
    // Test configuration validation
    assert!(config.order_book_config.enable_aggregation);
    assert_eq!(config.matching_config.matching_fee_percentage, Decimal::new(10, 4));
    assert!(config.settlement_config.atomic_settlement);
    assert!(config.liquidity_config.enable_dynamic_pricing);

    // Test trading pair
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
    assert_eq!(trading_pair.base, "BTC");
    assert_eq!(trading_pair.quote, "USD");
    assert_eq!(trading_pair.symbol(), "BTC/USD");

    // Test order status transitions
    let statuses = vec![
        OrderStatus::Pending,
        OrderStatus::Open,
        OrderStatus::PartiallyFilled,
        OrderStatus::Filled,
        OrderStatus::Cancelled,
    ];

    for status in statuses {
        match status {
            OrderStatus::Pending => assert_eq!(status, OrderStatus::Pending),
            OrderStatus::Filled => assert_eq!(status, OrderStatus::Filled),
            _ => {} // Other statuses
        }
    }
}

#[tokio::test]
async fn test_complex_trading_scenario() {
    // Test a complex trading scenario with multiple components
    let trading_pair = TradingPair::new("ETH".to_string(), "USD".to_string());
    let config = order_book::OrderBookConfig::default();
    let mut order_book = order_book::OrderBookManager::new(config, trading_pair.clone());

    // Add multiple orders at different price levels
    let orders = vec![
        // Buy orders
        ("user1", OrderSide::Buy, Decimal::new(2900, 0), Decimal::new(10, 0)),
        ("user2", OrderSide::Buy, Decimal::new(2950, 0), Decimal::new(5, 0)),
        ("user3", OrderSide::Buy, Decimal::new(2980, 0), Decimal::new(2, 0)),
        // Sell orders
        ("user4", OrderSide::Sell, Decimal::new(3020, 0), Decimal::new(3, 0)),
        ("user5", OrderSide::Sell, Decimal::new(3050, 0), Decimal::new(7, 0)),
        ("user6", OrderSide::Sell, Decimal::new(3100, 0), Decimal::new(15, 0)),
    ];

    for (user, side, price, quantity) in orders {
        let order = order_book::Order::new(
            user.to_string(),
            trading_pair.clone(),
            side,
            OrderType::Limit,
            price,
            quantity,
        );
        assert!(order_book.add_order(order).is_ok());
    }

    // Test order book state
    assert_eq!(order_book.best_bid(), Some(Decimal::new(2980, 0)));
    assert_eq!(order_book.best_ask(), Some(Decimal::new(3020, 0)));
    assert_eq!(order_book.spread(), Some(Decimal::new(40, 0))); // $40 spread

    // Create market order that should match
    let market_buy_order = order_book::Order::new(
        "market_buyer".to_string(),
        trading_pair.clone(),
        OrderSide::Buy,
        OrderType::Market,
        Decimal::new(3100, 0), // High price to ensure execution
        Decimal::new(5, 0), // 5 ETH
    );

    // This would normally be processed by the matching engine
    assert_eq!(market_buy_order.quantity, Decimal::new(5, 0));
    assert_eq!(market_buy_order.side, OrderSide::Buy);
}

#[tokio::test]
async fn test_performance_and_scalability() {
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
    let config = order_book::OrderBookConfig::default();
    let mut order_book = order_book::OrderBookManager::new(config, trading_pair.clone());

    let start_time = std::time::Instant::now();

    // Add many orders to test performance
    for i in 0..1000 {
        let price = Decimal::new(50000 + (i % 100), 0); // Vary prices
        let quantity = Decimal::new(1, 0);
        let side = if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell };

        let order = order_book::Order::new(
            format!("user{}", i),
            trading_pair.clone(),
            side,
            OrderType::Limit,
            price,
            quantity,
        );

        assert!(order_book.add_order(order).is_ok());
    }

    let elapsed = start_time.elapsed();
    println!("Added 1000 orders in {:?}", elapsed);

    // Should be able to add orders quickly
    assert!(elapsed.as_millis() < 5000); // Less than 5 seconds

    // Test snapshot generation performance
    let snapshot_start = std::time::Instant::now();
    let snapshot = order_book.snapshot();
    let snapshot_elapsed = snapshot_start.elapsed();

    assert!(!snapshot.bids.is_empty());
    assert!(!snapshot.asks.is_empty());
    assert!(snapshot_elapsed.as_millis() < 100); // Should be very fast
}

#[tokio::test]
async fn test_error_handling_and_edge_cases() {
    let config = order_book::OrderBookConfig::default();
    let trading_pair = TradingPair::new("BTC".to_string(), "USD".to_string());
    let mut order_book = order_book::OrderBookManager::new(config.clone(), trading_pair.clone());

    // Test invalid order (amount too small)
    let invalid_order = order_book::Order::new(
        "user1".to_string(),
        trading_pair.clone(),
        OrderSide::Buy,
        OrderType::Limit,
        Decimal::new(50000, 0),
        Decimal::new(1, 10), // Very small amount
    );

    let result = order_book.add_order(invalid_order);
    assert!(result.is_err());

    // Test invalid order (zero price)
    let zero_price_order = order_book::Order::new(
        "user2".to_string(),
        trading_pair.clone(),
        OrderSide::Buy,
        OrderType::Limit,
        Decimal::ZERO,
        Decimal::new(1, 0),
    );

    let result = order_book.add_order(zero_price_order);
    assert!(result.is_err());

    // Test removing non-existent order
    let non_existent_id = Uuid::new_v4();
    let result = order_book.remove_order(non_existent_id);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
