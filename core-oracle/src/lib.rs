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

pub mod aggregator;
pub mod band;
pub mod chainlink;
pub mod error;
pub mod pyth;
pub mod service;
pub mod types;

// Re-export main types and traits
pub use aggregator::{
    AggregationResult, ConsensusMethod, MultiSourceAggregator, OutlierDetectionResult,
    PriceAggregator,
};
pub use band::{BandFeed, BandOracle, BandOracleScript, BandPriceResponse, BandRequest};
pub use chainlink::{
    ChainlinkAggregator, ChainlinkFeed, ChainlinkNetwork, ChainlinkOracle, ChainlinkPriceResponse,
    ChainlinkRoundData,
};
pub use error::{OracleError, OracleResult};
pub use pyth::{PythFeed, PythOracle, PythPrice, PythPriceData, PythPriceFeedResponse};
pub use service::{
    HistoricalQuery, MockProviderClient, OracleService, OracleServiceImpl, PriceQuery,
    ProviderClient,
};
pub use types::{
    AggregationMethod, AlertSeverity, AlertType, DataSource, FeedConfig, OracleConfig,
    OracleHealthStatus, OracleProvider, PriceData, PriceFeed, PricePoint, ProviderConfig,
    TimeSeriesData, ValidationRule,
};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
            default_staleness_threshold: 3600,          // 1 hour
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
