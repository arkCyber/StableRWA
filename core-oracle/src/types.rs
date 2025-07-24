// =====================================================================================
// File: core-oracle/src/types.rs
// Description: Core types for oracle and price data operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Oracle service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    pub providers: Vec<ProviderConfig>,
    pub feeds: Vec<FeedConfig>,
    pub aggregation: AggregationConfig,
    pub validation: ValidationConfig,
    pub update_intervals: HashMap<String, u64>,
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: OracleProvider,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub rate_limit_per_second: u32,
    pub weight: f64,
    pub priority: u32,
}

/// Feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    pub id: String,
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub providers: Vec<OracleProvider>,
    pub update_interval_seconds: u64,
    pub deviation_threshold_percent: f64,
    pub staleness_threshold_seconds: u64,
    pub min_sources: u32,
    pub enabled: bool,
}

/// Aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    pub default_method: AggregationMethod,
    pub outlier_detection: bool,
    pub outlier_threshold_percent: f64,
    pub min_sources_for_consensus: u32,
    pub confidence_threshold: f64,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub enable_deviation_check: bool,
    pub enable_freshness_check: bool,
    pub enable_sanity_check: bool,
    pub max_price_change_percent: f64,
    pub min_price: Decimal,
    pub max_price: Decimal,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub recovery_timeout_seconds: u64,
    pub half_open_max_calls: u32,
}

/// Oracle providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OracleProvider {
    Chainlink,
    BandProtocol,
    PythNetwork,
    UMA,
    Tellor,
    API3,
    DIA,
    Custom(u32),
}

/// Data source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub provider: OracleProvider,
    pub feed_id: String,
    pub contract_address: Option<String>,
    pub api_endpoint: Option<String>,
    pub update_mechanism: UpdateMechanism,
    pub reliability_score: f64,
}

/// Update mechanism
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpdateMechanism {
    Push,
    Pull,
    Hybrid,
}

/// Price feed data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeed {
    pub id: String,
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub current_price: PriceData,
    pub historical_prices: Vec<PriceData>,
    pub sources: Vec<DataSource>,
    pub metadata: FeedMetadata,
    pub status: FeedStatus,
}

/// Price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub source: OracleProvider,
    pub confidence: f64,
    pub volume: Option<Decimal>,
    pub market_cap: Option<Decimal>,
    pub deviation: Option<f64>,
    pub round_id: Option<u64>,
}

/// Price point for time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub source_count: u32,
}

/// Time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub symbol: String,
    pub interval: TimeInterval,
    pub points: Vec<PricePoint>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

/// Time intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInterval {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
    FourHours,
    OneDay,
    OneWeek,
}

/// Feed metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedMetadata {
    pub description: String,
    pub decimals: u8,
    pub heartbeat: u64,
    pub deviation_threshold: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

/// Feed status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeedStatus {
    Active,
    Inactive,
    Deprecated,
    Maintenance,
    Error,
}

/// Aggregation methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggregationMethod {
    Mean,
    Median,
    WeightedMean,
    TWAP, // Time-Weighted Average Price
    VWAP, // Volume-Weighted Average Price
    Mode,
    Custom,
}

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub severity: ValidationSeverity,
}

/// Validation rule types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationRuleType {
    DeviationCheck,
    FreshnessCheck,
    SanityCheck,
    CircuitBreaker,
    OutlierDetection,
    ConsensusCheck,
}

/// Validation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Oracle request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleRequest {
    pub id: Uuid,
    pub feed_id: String,
    pub requested_at: DateTime<Utc>,
    pub requester: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub callback_url: Option<String>,
    pub timeout_seconds: u64,
}

/// Oracle response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    pub request_id: Uuid,
    pub feed_id: String,
    pub price_data: PriceData,
    pub sources_used: Vec<OracleProvider>,
    pub aggregation_method: AggregationMethod,
    pub confidence_score: f64,
    pub response_time_ms: u64,
    pub responded_at: DateTime<Utc>,
}

/// Feed subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSubscription {
    pub id: Uuid,
    pub feed_id: String,
    pub subscriber: String,
    pub callback_url: String,
    pub filters: Vec<SubscriptionFilter>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

/// Subscription filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    pub filter_type: FilterType,
    pub condition: FilterCondition,
    pub value: serde_json::Value,
}

/// Filter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterType {
    PriceChange,
    PriceThreshold,
    TimeInterval,
    Deviation,
    Volume,
}

/// Filter conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    Between,
    Outside,
}

/// Price alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: Uuid,
    pub feed_id: String,
    pub alert_type: AlertType,
    pub condition: AlertCondition,
    pub threshold: Decimal,
    pub current_value: Decimal,
    pub triggered_at: DateTime<Utc>,
    pub message: String,
    pub severity: AlertSeverity,
}

/// Alert types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertType {
    PriceThreshold,
    PriceDeviation,
    StaleData,
    SourceFailure,
    ConsensusFailure,
    CircuitBreaker,
}

/// Alert conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertCondition {
    Above,
    Below,
    DeviationExceeds,
    AgeExceeds,
    SourcesBelow,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Oracle health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleHealthStatus {
    pub overall_status: HealthStatus,
    pub provider_status: HashMap<OracleProvider, ProviderHealth>,
    pub feed_status: HashMap<String, FeedHealth>,
    pub last_updated: DateTime<Utc>,
    pub uptime_percentage: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Provider health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub status: HealthStatus,
    pub last_successful_update: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub response_time_ms: u64,
    pub success_rate: f64,
}

/// Feed health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedHealth {
    pub status: HealthStatus,
    pub last_update: DateTime<Utc>,
    pub data_age_seconds: u64,
    pub source_count: u32,
    pub consensus_score: f64,
    pub price_deviation: f64,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            feeds: Vec::new(),
            aggregation: AggregationConfig {
                default_method: AggregationMethod::Median,
                outlier_detection: true,
                outlier_threshold_percent: 10.0,
                min_sources_for_consensus: 3,
                confidence_threshold: 0.8,
            },
            validation: ValidationConfig {
                enable_deviation_check: true,
                enable_freshness_check: true,
                enable_sanity_check: true,
                max_price_change_percent: 50.0,
                min_price: Decimal::new(1, 8),             // 0.00000001
                max_price: Decimal::new(1_000_000_000, 0), // 1 billion
            },
            update_intervals: HashMap::new(),
            circuit_breaker: CircuitBreakerConfig {
                enabled: true,
                failure_threshold: 5,
                recovery_timeout_seconds: 300,
                half_open_max_calls: 3,
            },
        }
    }
}

impl TimeInterval {
    pub fn to_seconds(&self) -> u64 {
        match self {
            Self::OneMinute => 60,
            Self::FiveMinutes => 300,
            Self::FifteenMinutes => 900,
            Self::OneHour => 3600,
            Self::FourHours => 14400,
            Self::OneDay => 86400,
            Self::OneWeek => 604800,
        }
    }
}

impl OracleProvider {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Chainlink => "Chainlink",
            Self::BandProtocol => "Band Protocol",
            Self::PythNetwork => "Pyth Network",
            Self::UMA => "UMA",
            Self::Tellor => "Tellor",
            Self::API3 => "API3",
            Self::DIA => "DIA",
            Self::Custom(_) => "Custom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_config_default() {
        let config = OracleConfig::default();
        assert_eq!(config.aggregation.default_method, AggregationMethod::Median);
        assert!(config.validation.enable_deviation_check);
        assert!(config.circuit_breaker.enabled);
    }

    #[test]
    fn test_time_interval_conversion() {
        assert_eq!(TimeInterval::OneMinute.to_seconds(), 60);
        assert_eq!(TimeInterval::OneHour.to_seconds(), 3600);
        assert_eq!(TimeInterval::OneDay.to_seconds(), 86400);
    }

    #[test]
    fn test_oracle_provider_names() {
        assert_eq!(OracleProvider::Chainlink.name(), "Chainlink");
        assert_eq!(OracleProvider::BandProtocol.name(), "Band Protocol");
        assert_eq!(OracleProvider::PythNetwork.name(), "Pyth Network");
    }

    #[test]
    fn test_price_data_creation() {
        let price_data = PriceData {
            price: Decimal::new(50000, 2), // 500.00
            timestamp: Utc::now(),
            source: OracleProvider::Chainlink,
            confidence: 0.95,
            volume: Some(Decimal::new(1000000, 2)),
            market_cap: None,
            deviation: Some(1.5),
            round_id: Some(12345),
        };

        assert_eq!(price_data.source, OracleProvider::Chainlink);
        assert_eq!(price_data.confidence, 0.95);
        assert!(price_data.volume.is_some());
    }
}
