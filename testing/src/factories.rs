// =====================================================================================
// File: testing/src/factories.rs
// Description: Test data factories for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use fake::{Fake, Faker};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

// Asset factories
pub mod asset {
    use super::*;
    use core_asset_lifecycle::types::*;

    pub struct AssetFactory;

    impl AssetFactory {
        pub fn create_real_estate() -> Asset {
            let mut asset = Asset::new(
                format!("Property {}", Faker.fake::<String>()),
                "Luxury residential property".to_string(),
                AssetType::RealEstate,
                format!("owner_{}", Uuid::new_v4()),
                AssetLocation {
                    address: "123 Main Street".to_string(),
                    city: "New York".to_string(),
                    state_province: Some("NY".to_string()),
                    country: "US".to_string(),
                    postal_code: Some("10001".to_string()),
                    coordinates: Some((40.7128, -74.0060)),
                    timezone: Some("America/New_York".to_string()),
                },
            );

            // Add some metadata
            asset.metadata.physical_attributes.insert(
                "square_feet".to_string(),
                "2500".to_string(),
            );
            asset.metadata.physical_attributes.insert(
                "bedrooms".to_string(),
                "3".to_string(),
            );
            asset.metadata.physical_attributes.insert(
                "bathrooms".to_string(),
                "2".to_string(),
            );

            asset.metadata.legal_attributes.insert(
                "deed_number".to_string(),
                format!("DEED{}", Faker.fake::<u32>()),
            );

            asset.metadata.financial_attributes.insert(
                "purchase_price".to_string(),
                "750000.00".to_string(),
            );

            asset
        }

        pub fn create_art() -> Asset {
            let mut asset = Asset::new(
                format!("Artwork {}", Faker.fake::<String>()),
                "Contemporary art piece".to_string(),
                AssetType::Art,
                format!("owner_{}", Uuid::new_v4()),
                AssetLocation {
                    address: "456 Gallery Ave".to_string(),
                    city: "Los Angeles".to_string(),
                    state_province: Some("CA".to_string()),
                    country: "US".to_string(),
                    postal_code: Some("90210".to_string()),
                    coordinates: Some((34.0522, -118.2437)),
                    timezone: Some("America/Los_Angeles".to_string()),
                },
            );

            asset.metadata.physical_attributes.insert(
                "medium".to_string(),
                "Oil on canvas".to_string(),
            );
            asset.metadata.physical_attributes.insert(
                "dimensions".to_string(),
                "24x36 inches".to_string(),
            );

            asset.metadata.legal_attributes.insert(
                "artist".to_string(),
                "Famous Artist".to_string(),
            );
            asset.metadata.legal_attributes.insert(
                "year_created".to_string(),
                "2020".to_string(),
            );

            asset
        }

        pub fn create_commodity() -> Asset {
            Asset::new(
                "Gold Bars".to_string(),
                "999.9 fine gold bars".to_string(),
                AssetType::Commodities,
                format!("owner_{}", Uuid::new_v4()),
                AssetLocation {
                    address: "Secure Vault 1".to_string(),
                    city: "Zurich".to_string(),
                    state_province: None,
                    country: "CH".to_string(),
                    postal_code: Some("8001".to_string()),
                    coordinates: Some((47.3769, 8.5417)),
                    timezone: Some("Europe/Zurich".to_string()),
                },
            )
        }

        pub fn create_with_valuation(mut asset: Asset, value: Decimal) -> Asset {
            let valuation = AssetValuation {
                id: Uuid::new_v4(),
                asset_id: asset.id,
                valuation_method: core_asset_lifecycle::types::ValuationMethod::ExpertAppraisal,
                valuation_amount: value,
                currency: "USD".to_string(),
                valuation_date: Utc::now(),
                valid_until: Some(Utc::now() + chrono::Duration::days(365)),
                appraiser_id: "appraiser_123".to_string(),
                appraiser_credentials: "Certified Real Estate Appraiser".to_string(),
                confidence_level: 0.95,
                methodology_notes: "Comparative market analysis".to_string(),
                supporting_documents: vec![],
                market_conditions: Some("Stable market conditions".to_string()),
                assumptions: vec!["Property in good condition".to_string()],
                limitations: vec!["Subject to market changes".to_string()],
            };

            asset.valuations.push(valuation);
            asset
        }
    }
}

// Trading factories
pub mod trading {
    use super::*;
    use core_trading::types::*;

    pub struct TradingFactory;

    impl TradingFactory {
        pub fn create_trading_pair(base: &str, quote: &str) -> TradingPair {
            TradingPair::new(base.to_string(), quote.to_string())
        }

        pub fn create_buy_order(
            trading_pair: TradingPair,
            quantity: Decimal,
            price: Option<Decimal>,
        ) -> Order {
            Order::new(
                format!("user_{}", Uuid::new_v4()),
                trading_pair,
                OrderType::Limit,
                OrderSide::Buy,
                quantity,
                price,
            )
        }

        pub fn create_sell_order(
            trading_pair: TradingPair,
            quantity: Decimal,
            price: Option<Decimal>,
        ) -> Order {
            Order::new(
                format!("user_{}", Uuid::new_v4()),
                trading_pair,
                OrderType::Limit,
                OrderSide::Sell,
                quantity,
                price,
            )
        }

        pub fn create_market_order(
            trading_pair: TradingPair,
            side: OrderSide,
            quantity: Decimal,
        ) -> Order {
            Order::new(
                format!("user_{}", Uuid::new_v4()),
                trading_pair,
                OrderType::Market,
                side,
                quantity,
                None,
            )
        }

        pub fn create_trade(
            trading_pair: TradingPair,
            quantity: Decimal,
            price: Decimal,
        ) -> Trade {
            let buy_order = Self::create_buy_order(trading_pair.clone(), quantity, Some(price));
            let sell_order = Self::create_sell_order(trading_pair.clone(), quantity, Some(price));

            Trade::new(trading_pair, &buy_order, &sell_order, quantity, price)
        }

        pub fn create_order_book(trading_pair: TradingPair) -> OrderBook {
            let mut book = OrderBook::new(trading_pair.clone());

            // Add some sample bids and asks
            let bid_prices = vec![
                Decimal::new(99000, 2), // $990.00
                Decimal::new(98500, 2), // $985.00
                Decimal::new(98000, 2), // $980.00
            ];

            let ask_prices = vec![
                Decimal::new(100500, 2), // $1005.00
                Decimal::new(101000, 2), // $1010.00
                Decimal::new(101500, 2), // $1015.00
            ];

            for price in bid_prices {
                let order = Self::create_buy_order(
                    trading_pair.clone(),
                    Decimal::new(100, 0), // 1.00 quantity
                    Some(price),
                );
                book.add_order(&order);
            }

            for price in ask_prices {
                let order = Self::create_sell_order(
                    trading_pair.clone(),
                    Decimal::new(100, 0), // 1.00 quantity
                    Some(price),
                );
                book.add_order(&order);
            }

            book
        }

        pub fn create_liquidity_pool(trading_pair: TradingPair) -> LiquidityPool {
            LiquidityPool {
                id: Uuid::new_v4(),
                trading_pair,
                base_reserve: Decimal::new(1000, 0), // 1000 base tokens
                quote_reserve: Decimal::new(100000000, 2), // $1,000,000
                total_liquidity: Decimal::new(10000, 0),
                fee_rate: Decimal::new(30, 4), // 0.30%
                providers: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                active: true,
            }
        }
    }
}

// Risk management factories
pub mod risk {
    use super::*;
    use core_risk_management::types::*;

    pub struct RiskFactory;

    impl RiskFactory {
        pub fn create_risk_factor(risk_type: RiskType, impact_level: RiskLevel) -> RiskFactor {
            RiskFactor {
                id: Uuid::new_v4(),
                name: format!("{:?} Risk Factor", risk_type),
                risk_type,
                description: "Test risk factor".to_string(),
                impact_level,
                probability: 0.3,
                time_horizon: TimeHorizon::MediumTerm,
                quantitative_impact: Some(Decimal::new(10000000, 2)), // $100,000
                qualitative_impact: "Moderate impact expected".to_string(),
                mitigation_strategies: vec!["Diversification".to_string()],
                data_sources: vec!["Market data".to_string()],
                last_updated: Utc::now(),
            }
        }

        pub fn create_risk_assessment(asset_id: Option<Uuid>) -> RiskAssessment {
            RiskAssessment {
                id: Uuid::new_v4(),
                asset_id,
                portfolio_id: None,
                assessment_type: AssessmentType::Periodic,
                overall_risk_level: RiskLevel::Medium,
                risk_score: 3.5,
                risk_factors: vec![
                    Self::create_risk_factor(RiskType::Market, RiskLevel::Medium),
                    Self::create_risk_factor(RiskType::Liquidity, RiskLevel::Low),
                ],
                risk_metrics: RiskMetrics {
                    value_at_risk: HashMap::new(),
                    expected_shortfall: HashMap::new(),
                    maximum_drawdown: Decimal::new(15, 0), // 15%
                    volatility: Decimal::new(20, 0), // 20%
                    beta: Some(1.2),
                    correlation_matrix: None,
                    sharpe_ratio: Some(1.5),
                    sortino_ratio: Some(1.8),
                    information_ratio: Some(0.8),
                    tracking_error: Some(Decimal::new(5, 0)), // 5%
                },
                scenario_analysis: vec![],
                recommendations: vec![],
                assessor_id: "risk_analyst_123".to_string(),
                assessment_date: Utc::now(),
                valid_until: Utc::now() + chrono::Duration::days(90),
                confidence_level: 0.95,
                methodology: "Monte Carlo simulation".to_string(),
                assumptions: vec!["Normal market conditions".to_string()],
                limitations: vec!["Based on historical data".to_string()],
            }
        }

        pub fn create_insurance_policy(policy_type: PolicyType) -> InsurancePolicy {
            InsurancePolicy {
                id: Uuid::new_v4(),
                policy_number: format!("POL-{}", Faker.fake::<u32>()),
                policy_type,
                insurer: "Test Insurance Co.".to_string(),
                insured_entity: "StableRWA Platform".to_string(),
                coverage_amount: Decimal::new(100000000, 2), // $1,000,000
                deductible: Decimal::new(1000000, 2), // $10,000
                premium: Decimal::new(500000, 2), // $5,000
                policy_start: Utc::now(),
                policy_end: Utc::now() + chrono::Duration::days(365),
                covered_risks: vec![RiskType::Operational, RiskType::Technology],
                exclusions: vec!["War and terrorism".to_string()],
                claims_history: vec![],
                status: PolicyStatus::Active,
            }
        }
    }
}

// Bridge factories
pub mod bridge {
    use super::*;
    use core_bridge::types::*;

    pub struct BridgeFactory;

    impl BridgeFactory {
        pub fn create_bridge_transaction(
            source_chain: ChainId,
            destination_chain: ChainId,
            amount: Decimal,
        ) -> BridgeTransaction {
            BridgeTransaction::new(
                format!("user_{}", Uuid::new_v4()),
                source_chain,
                destination_chain,
                AssetTransfer {
                    token_symbol: "USDC".to_string(),
                    token_address: Some("0xA0b86a33E6441b8C4505B8C4505B8C4505B8C450".to_string()),
                    amount,
                    decimals: 6,
                    source_address: "0x1234567890123456789012345678901234567890".to_string(),
                    destination_address: "0x0987654321098765432109876543210987654321".to_string(),
                    memo: None,
                },
                12, // required confirmations
                3600, // timeout seconds
            )
        }

        pub fn create_atomic_swap(
            source_chain: ChainId,
            destination_chain: ChainId,
        ) -> AtomicSwap {
            AtomicSwap {
                id: Uuid::new_v4(),
                initiator: format!("user_{}", Uuid::new_v4()),
                participant: format!("user_{}", Uuid::new_v4()),
                source_chain,
                destination_chain,
                source_asset: AssetInfo {
                    token_symbol: "ETH".to_string(),
                    amount: Decimal::new(1000000000000000000u64, 0), // 1 ETH
                    address: "0x1234567890123456789012345678901234567890".to_string(),
                    contract_address: None,
                },
                destination_asset: AssetInfo {
                    token_symbol: "BTC".to_string(),
                    amount: Decimal::new(100000000, 0), // 1 BTC
                    address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
                    contract_address: None,
                },
                hash_lock: "0xabcdef1234567890".to_string(),
                time_lock: Utc::now() + chrono::Duration::hours(24),
                secret: None,
                status: SwapStatus::Initiated,
                source_tx_hash: None,
                destination_tx_hash: None,
                refund_tx_hash: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        pub fn create_security_alert(alert_type: AlertType, severity: AlertSeverity) -> SecurityAlert {
            SecurityAlert {
                id: Uuid::new_v4(),
                alert_type,
                severity,
                chain_id: Some(ChainId::Ethereum),
                transaction_id: Some(Uuid::new_v4()),
                description: format!("{:?} detected", alert_type),
                details: serde_json::json!({"test": "data"}),
                triggered_at: Utc::now(),
                resolved_at: None,
                resolved: false,
                actions_taken: vec![],
            }
        }
    }
}

// Analytics factories
pub mod analytics {
    use super::*;
    use core_analytics::types::*;

    pub struct AnalyticsFactory;

    impl AnalyticsFactory {
        pub fn create_metric(name: &str, metric_type: MetricType, value: MetricValue) -> Metric {
            Metric::new(name, metric_type, value, "test_source")
                .with_unit("count")
                .with_description("Test metric")
                .with_tag("environment", "test")
        }

        pub fn create_time_series_data(metric_name: &str, points: Vec<(DateTime<Utc>, f64)>) -> TimeSeriesData {
            let time_series_points = points
                .into_iter()
                .map(|(timestamp, value)| TimeSeriesPoint {
                    timestamp,
                    value,
                    tags: HashMap::new(),
                })
                .collect();

            TimeSeriesData {
                metric_name: metric_name.to_string(),
                points: time_series_points,
                metadata: TimeSeriesMetadata {
                    unit: "count".to_string(),
                    description: "Test time series".to_string(),
                    aggregation_method: Some("sum".to_string()),
                    resolution: Some("1m".to_string()),
                    data_source: "test_source".to_string(),
                },
            }
        }

        pub fn create_dashboard(name: &str) -> Dashboard {
            Dashboard {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: "Test dashboard".to_string(),
                widgets: vec![],
                layout: DashboardLayout {
                    grid_size: GridSize {
                        columns: 12,
                        rows: 8,
                        cell_width: 100,
                        cell_height: 100,
                    },
                    auto_arrange: false,
                    responsive: true,
                },
                refresh_interval: Some(60),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                created_by: "test_user".to_string(),
                shared: false,
                permissions: vec![],
            }
        }

        pub fn create_widget(widget_type: WidgetType, title: &str) -> Widget {
            Widget {
                id: Uuid::new_v4(),
                title: title.to_string(),
                widget_type,
                position: WidgetPosition { x: 0, y: 0 },
                size: WidgetSize { width: 4, height: 3 },
                configuration: WidgetConfiguration {
                    query: Some("SELECT COUNT(*) FROM transactions".to_string()),
                    chart_config: None,
                    display_options: HashMap::new(),
                    thresholds: None,
                },
                data_source: "test_db".to_string(),
                refresh_interval: Some(300),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_factory() {
        let real_estate = asset::AssetFactory::create_real_estate();
        assert_eq!(real_estate.asset_type, core_asset_lifecycle::types::AssetType::RealEstate);
        assert!(!real_estate.metadata.physical_attributes.is_empty());

        let art = asset::AssetFactory::create_art();
        assert_eq!(art.asset_type, core_asset_lifecycle::types::AssetType::Art);

        let commodity = asset::AssetFactory::create_commodity();
        assert_eq!(commodity.asset_type, core_asset_lifecycle::types::AssetType::Commodities);
    }

    #[test]
    fn test_trading_factory() {
        let pair = trading::TradingFactory::create_trading_pair("BTC", "USD");
        assert_eq!(pair.symbol(), "BTC/USD");

        let order = trading::TradingFactory::create_buy_order(
            pair.clone(),
            Decimal::new(1, 0),
            Some(Decimal::new(50000, 0)),
        );
        assert_eq!(order.side, core_trading::types::OrderSide::Buy);

        let book = trading::TradingFactory::create_order_book(pair);
        assert!(book.best_bid().is_some());
        assert!(book.best_ask().is_some());
    }

    #[test]
    fn test_risk_factory() {
        let risk_factor = risk::RiskFactory::create_risk_factor(
            core_risk_management::types::RiskType::Market,
            core_risk_management::types::RiskLevel::High,
        );
        assert_eq!(risk_factor.risk_type, core_risk_management::types::RiskType::Market);
        assert_eq!(risk_factor.impact_level, core_risk_management::types::RiskLevel::High);

        let assessment = risk::RiskFactory::create_risk_assessment(Some(Uuid::new_v4()));
        assert!(assessment.asset_id.is_some());
        assert!(!assessment.risk_factors.is_empty());
    }
}
