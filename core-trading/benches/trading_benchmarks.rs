// =====================================================================================
// File: core-trading/benches/trading_benchmarks.rs
// Description: Performance benchmarks for trading operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use core_trading::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::Decimal;
use uuid::Uuid;

fn create_test_order() -> Order {
    Order {
        id: Uuid::new_v4(),
        user_id: "test_user".to_string(),
        asset_id: "test_asset".to_string(),
        order_type: OrderType::Market,
        side: OrderSide::Buy,
        quantity: Decimal::from(100),
        price: Some(Decimal::from(1000)),
        status: OrderStatus::Pending,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        filled_quantity: Decimal::ZERO,
        remaining_quantity: Decimal::from(100),
        average_fill_price: None,
        fees: Decimal::ZERO,
        time_in_force: TimeInForce::GTC,
        stop_price: None,
        metadata: std::collections::HashMap::new(),
    }
}

fn bench_order_creation(c: &mut Criterion) {
    c.bench_function("order_creation", |b| {
        b.iter(|| black_box(create_test_order()))
    });
}

fn bench_order_validation(c: &mut Criterion) {
    let order = create_test_order();

    c.bench_function("order_validation", |b| {
        b.iter(|| black_box(order.validate()))
    });
}

fn bench_price_calculation(c: &mut Criterion) {
    let quantity = Decimal::from(100);
    let price = Decimal::from(1000);

    c.bench_function("price_calculation", |b| {
        b.iter(|| black_box(quantity * price))
    });
}

criterion_group!(
    benches,
    bench_order_creation,
    bench_order_validation,
    bench_price_calculation
);
criterion_main!(benches);
