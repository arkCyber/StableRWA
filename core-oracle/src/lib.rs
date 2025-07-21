// =====================================================================================
// File: core-oracle/src/lib.rs
// Description: Oracle and price data services for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Oracle Module
//! 
//! This module provides comprehensive oracle and price data services for the StableRWA
//! platform, integrating with major oracle providers like Chainlink, Band Protocol,
//! Pyth Network, and UMA.

pub mod error;
pub mod types;
pub mod chainlink;
pub mod band;
pub mod pyth;
pub mod uma;
pub mod tellor;
pub mod api3;
pub mod dia;
pub mod aggregator;
pub mod validator;
pub mod feed_manager;
pub mod price_calculator;
pub mod service;

// Re-export main types and traits
pub use error::{OracleError, OracleResult};
pub use types::{
    PriceFeed, PriceData, OracleProvider, DataSource, FeedConfig,
    PricePoint, TimeSeriesData, AggregationMethod, ValidationRule
};
pub use chainlink::{ChainlinkOracle, ChainlinkFeed, ChainlinkAggregator};
pub use band::{BandOracle, BandFeed, BandRequest};
pub use pyth::{PythOracle, PythFeed, PythPrice};
pub use uma::{UMAOracle, UMARequest, UMAProposal};
pub use aggregator::{
    PriceAggregator, WeightedAggregator, MedianAggregator,
    TWAPAggregator, VWAPAggregator
};
pub use validator::{
    PriceValidator, DeviationValidator, FreshnessValidator,
    CircuitBreakerValidator, SanityCheckValidator
};
pub use feed_manager::{
    FeedManager, FeedSubscription, FeedUpdate,
    RealTimeFeed, HistoricalFeed
};
pub use price_calculator::{
    PriceCalculator, VolatilityCalculator, CorrelationCalculator,
    TechnicalIndicators, RiskMetrics
};
pub use service::OracleService;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main Oracle service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleServiceConfig {
    /// Chainlink configuration
    pub chainlink_config: chainlink::ChainlinkConfig,
    /// Band Protocol configuration
    pub band_config: band::BandConfig,
    /// Pyth Network configuration
    pub pyth_config: pyth::PythConfig,
    /// UMA configuration
    pub uma_config: uma::UMAConfig,
    /// Price aggregation configuration
    pub aggregation_config: aggregator::AggregationConfig,
    /// Validation configuration
    pub validation_config: validator::ValidationConfig,
    /// Feed management configuration
    pub feed_config: feed_manager::FeedConfig,
    /// Global oracle settings
    pub global_settings: GlobalOracleSettings,
}

impl Default for OracleServiceConfig {
    fn default() -> Self {
        Self {
            chainlink_config: chainlink::ChainlinkConfig::default(),
            band_config: band::BandConfig::default(),
            pyth_config: pyth::PythConfig::default(),
            uma_config: uma::UMAConfig::default(),
            aggregation_config: aggregator::AggregationConfig::default(),
            validation_config: validator::ValidationConfig::default(),
            feed_config: feed_manager::FeedConfig::default(),
            global_settings: GlobalOracleSettings::default(),
        }
    }
}

/// Global oracle settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalOracleSettings {
    /// Default price staleness threshold in seconds
    pub default_staleness_threshold: u64,
    /// Maximum price deviation percentage
    pub max_price_deviation: Decimal,
    /// Minimum number of sources for aggregation
    pub min_sources_for_aggregation: u32,
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
    /// Circuit breaker threshold
    pub circuit_breaker_threshold: Decimal,
    /// Enable price validation
    pub enable_price_validation: bool,
    /// Enable historical data storage
    pub enable_historical_storage: bool,
    /// Historical data retention days
    pub historical_retention_days: u32,
    /// Update frequency in seconds
    pub update_frequency_seconds: u64,
    /// Enable real-time updates
    pub enable_real_time_updates: bool,
}

impl Default for GlobalOracleSettings {
    fn default() -> Self {
        Self {
            default_staleness_threshold: 3600, // 1 hour
            max_price_deviation: Decimal::new(1000, 4), // 10%
            min_sources_for_aggregation: 3,
            enable_circuit_breaker: true,
            circuit_breaker_threshold: Decimal::new(2000, 4), // 20%
            enable_price_validation: true,
            enable_historical_storage: true,
            historical_retention_days: 365,
            update_frequency_seconds: 60,
            enable_real_time_updates: true,
        }
    }
}

/// Oracle metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleMetrics {
    pub total_price_feeds: u64,
    pub active_price_feeds: u64,
    pub total_price_updates_24h: u64,
    pub successful_updates_24h: u64,
    pub failed_updates_24h: u64,
    pub average_update_latency_ms: f64,
    pub price_deviation_events_24h: u64,
    pub circuit_breaker_triggers_24h: u64,
    pub provider_metrics: HashMap<String, ProviderMetrics>,
    pub last_updated: DateTime<Utc>,
}

/// Provider-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub provider_name: String,
    pub active_feeds: u64,
    pub updates_24h: u64,
    pub success_rate: Decimal,
    pub average_latency_ms: f64,
    pub uptime_percentage: Decimal,
    pub last_update: DateTime<Utc>,
}

/// Oracle health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleHealthStatus {
    pub overall_status: String,
    pub chainlink_status: String,
    pub band_status: String,
    pub pyth_status: String,
    pub uma_status: String,
    pub aggregation_status: String,
    pub validation_status: String,
    pub feed_manager_status: String,
    pub provider_statuses: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
}

// Stub modules for compilation
pub mod chainlink {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChainlinkConfig {
        pub node_url: String,
        pub api_key: String,
        pub supported_feeds: Vec<String>,
    }
    
    impl Default for ChainlinkConfig {
        fn default() -> Self {
            Self {
                node_url: "https://api.chain.link".to_string(),
                api_key: "".to_string(),
                supported_feeds: vec![
                    "ETH/USD".to_string(),
                    "BTC/USD".to_string(),
                    "USDC/USD".to_string(),
                ],
            }
        }
    }
    
    pub struct ChainlinkOracle;
    pub struct ChainlinkFeed;
    pub struct ChainlinkAggregator;
}

pub mod band {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BandConfig {
        pub api_url: String,
        pub chain_id: String,
        pub supported_symbols: Vec<String>,
    }
    
    impl Default for BandConfig {
        fn default() -> Self {
            Self {
                api_url: "https://api.bandprotocol.com".to_string(),
                chain_id: "band-laozi-mainnet".to_string(),
                supported_symbols: vec![
                    "BTC".to_string(),
                    "ETH".to_string(),
                    "USDT".to_string(),
                ],
            }
        }
    }
    
    pub struct BandOracle;
    pub struct BandFeed;
    pub struct BandRequest;
}

pub mod pyth {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PythConfig {
        pub network_url: String,
        pub program_id: String,
        pub supported_products: Vec<String>,
    }
    
    impl Default for PythConfig {
        fn default() -> Self {
            Self {
                network_url: "https://api.pythnetwork.com".to_string(),
                program_id: "FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH".to_string(),
                supported_products: vec![
                    "Crypto.BTC/USD".to_string(),
                    "Crypto.ETH/USD".to_string(),
                    "Crypto.SOL/USD".to_string(),
                ],
            }
        }
    }
    
    pub struct PythOracle;
    pub struct PythFeed;
    pub struct PythPrice;
}

pub mod uma {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UMAConfig {
        pub optimistic_oracle_address: String,
        pub voting_token_address: String,
        pub minimum_bond: Decimal,
    }
    
    impl Default for UMAConfig {
        fn default() -> Self {
            Self {
                optimistic_oracle_address: "0x...".to_string(),
                voting_token_address: "0x...".to_string(),
                minimum_bond: Decimal::new(1000, 2), // $10
            }
        }
    }
    
    pub struct UMAOracle;
    pub struct UMARequest;
    pub struct UMAProposal;
}

pub mod aggregator {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AggregationConfig {
        pub default_method: AggregationMethod,
        pub outlier_detection: bool,
        pub weight_by_reliability: bool,
    }
    
    impl Default for AggregationConfig {
        fn default() -> Self {
            Self {
                default_method: AggregationMethod::WeightedAverage,
                outlier_detection: true,
                weight_by_reliability: true,
            }
        }
    }
    
    pub struct PriceAggregator;
    pub struct WeightedAggregator;
    pub struct MedianAggregator;
    pub struct TWAPAggregator;
    pub struct VWAPAggregator;
}

pub mod validator {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ValidationConfig {
        pub enable_deviation_check: bool,
        pub enable_freshness_check: bool,
        pub enable_sanity_check: bool,
        pub max_deviation_percentage: Decimal,
    }
    
    impl Default for ValidationConfig {
        fn default() -> Self {
            Self {
                enable_deviation_check: true,
                enable_freshness_check: true,
                enable_sanity_check: true,
                max_deviation_percentage: Decimal::new(500, 4), // 5%
            }
        }
    }
    
    pub struct PriceValidator;
    pub struct DeviationValidator;
    pub struct FreshnessValidator;
    pub struct CircuitBreakerValidator;
    pub struct SanityCheckValidator;
}

pub mod feed_manager {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FeedConfig {
        pub max_concurrent_feeds: u32,
        pub update_batch_size: u32,
        pub retry_attempts: u32,
    }
    
    impl Default for FeedConfig {
        fn default() -> Self {
            Self {
                max_concurrent_feeds: 100,
                update_batch_size: 10,
                retry_attempts: 3,
            }
        }
    }
    
    pub struct FeedManager;
    pub struct FeedSubscription;
    pub struct FeedUpdate;
    pub struct RealTimeFeed;
    pub struct HistoricalFeed;
}

pub mod price_calculator {
    use super::*;
    
    pub struct PriceCalculator;
    pub struct VolatilityCalculator;
    pub struct CorrelationCalculator;
    pub struct TechnicalIndicators;
    pub struct RiskMetrics;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_config_default() {
        let config = OracleServiceConfig::default();
        assert_eq!(config.global_settings.default_staleness_threshold, 3600);
        assert_eq!(config.global_settings.min_sources_for_aggregation, 3);
        assert!(config.global_settings.enable_circuit_breaker);
        assert!(config.global_settings.enable_price_validation);
        assert!(config.global_settings.enable_real_time_updates);
    }

    #[test]
    fn test_chainlink_config() {
        let config = chainlink::ChainlinkConfig::default();
        assert_eq!(config.node_url, "https://api.chain.link");
        assert_eq!(config.supported_feeds.len(), 3);
        assert!(config.supported_feeds.contains(&"ETH/USD".to_string()));
    }

    #[test]
    fn test_global_oracle_settings() {
        let settings = GlobalOracleSettings::default();
        assert_eq!(settings.max_price_deviation, Decimal::new(1000, 4));
        assert_eq!(settings.circuit_breaker_threshold, Decimal::new(2000, 4));
        assert_eq!(settings.historical_retention_days, 365);
        assert_eq!(settings.update_frequency_seconds, 60);
    }
}
