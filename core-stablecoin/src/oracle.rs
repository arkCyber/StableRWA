// =====================================================================================
// File: core-stablecoin/src/oracle.rs
// Description: Enterprise-grade price oracle integration with multiple providers
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn};

use crate::{StablecoinError, StablecoinResult};

/// Price oracle trait for enterprise-grade price feeds
#[async_trait]
pub trait PriceOracle: Send + Sync {
    /// Get current price for an asset
    async fn get_price(&self, asset_symbol: &str) -> StablecoinResult<PriceData>;
    
    /// Get historical prices
    async fn get_historical_prices(
        &self,
        asset_symbol: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> StablecoinResult<Vec<PriceData>>;
    
    /// Check oracle health
    async fn health_check(&self) -> StablecoinResult<OracleHealth>;
    
    /// Get supported assets
    fn supported_assets(&self) -> Vec<String>;
    
    /// Get oracle metadata
    fn metadata(&self) -> OracleMetadata;
}

/// Multi-oracle aggregator with failover and validation
pub struct OracleAggregator {
    primary_oracle: Box<dyn PriceOracle>,
    backup_oracles: Vec<Box<dyn PriceOracle>>,
    config: OracleConfig,
    price_cache: HashMap<String, CachedPrice>,
    circuit_breaker: CircuitBreaker,
}

impl OracleAggregator {
    /// Create new oracle aggregator
    pub fn new(
        primary_oracle: Box<dyn PriceOracle>,
        backup_oracles: Vec<Box<dyn PriceOracle>>,
        config: OracleConfig,
    ) -> Self {
        let circuit_breaker_threshold = config.circuit_breaker_threshold;
        Self {
            primary_oracle,
            backup_oracles,
            config,
            price_cache: HashMap::new(),
            circuit_breaker: CircuitBreaker::new(circuit_breaker_threshold),
        }
    }

    /// Get price with failover and validation
    pub async fn get_validated_price(&mut self, asset_symbol: &str) -> StablecoinResult<PriceData> {
        // Check cache first
        if let Some(cached) = self.get_cached_price(asset_symbol) {
            if !self.is_price_stale(&cached) {
                return Ok(cached.price.clone());
            }
        }

        // Try primary oracle first
        match self.try_get_price_with_timeout(&*self.primary_oracle, asset_symbol).await {
            Ok(price) => {
                if self.validate_price(&price).await? {
                    self.cache_price(asset_symbol, price.clone());
                    self.circuit_breaker.record_success();
                    return Ok(price);
                } else {
                    warn!("Primary oracle price validation failed for {}", asset_symbol);
                }
            }
            Err(e) => {
                error!("Primary oracle failed for {}: {}", asset_symbol, e);
                self.circuit_breaker.record_failure();
            }
        }

        // Try backup oracles
        for (i, oracle) in self.backup_oracles.iter().enumerate() {
            match self.try_get_price_with_timeout(&**oracle, asset_symbol).await {
                Ok(price) => {
                    if self.validate_price(&price).await? {
                        info!("Using backup oracle {} for {}", i, asset_symbol);
                        self.cache_price(asset_symbol, price.clone());
                        return Ok(price);
                    }
                }
                Err(e) => {
                    warn!("Backup oracle {} failed for {}: {}", i, asset_symbol, e);
                }
            }
        }

        // All oracles failed - check if we have recent cached data
        if let Some(cached) = self.get_cached_price(asset_symbol) {
            if cached.price.timestamp > Utc::now() - chrono::Duration::minutes(self.config.emergency_cache_minutes as i64) {
                warn!("Using emergency cached price for {}", asset_symbol);
                return Ok(cached.price.clone());
            }
        }

        Err(StablecoinError::OracleUnavailable(format!(
            "All oracles failed for asset: {}", asset_symbol
        )))
    }

    /// Get aggregated price from multiple oracles
    pub async fn get_aggregated_price(&self, asset_symbol: &str) -> StablecoinResult<PriceData> {
        let mut prices = Vec::new();
        let mut successful_oracles = 0;

        // Collect prices from all available oracles
        if let Ok(price) = self.try_get_price_with_timeout(&*self.primary_oracle, asset_symbol).await {
            prices.push(price);
            successful_oracles += 1;
        }

        for oracle in &self.backup_oracles {
            if let Ok(price) = self.try_get_price_with_timeout(&**oracle, asset_symbol).await {
                prices.push(price);
                successful_oracles += 1;
            }
        }

        if successful_oracles < self.config.min_oracle_sources {
            return Err(StablecoinError::OracleUnavailable(format!(
                "Insufficient oracle sources: {} < {}", 
                successful_oracles, 
                self.config.min_oracle_sources
            )));
        }

        // Calculate aggregated price
        self.aggregate_prices(prices, asset_symbol).await
    }

    /// Try to get price with timeout
    async fn try_get_price_with_timeout(
        &self,
        oracle: &dyn PriceOracle,
        asset_symbol: &str,
    ) -> StablecoinResult<PriceData> {
        match timeout(
            Duration::from_secs(self.config.timeout_seconds),
            oracle.get_price(asset_symbol),
        ).await {
            Ok(result) => result,
            Err(_) => Err(StablecoinError::TimeoutError(format!(
                "Oracle timeout for asset: {}", asset_symbol
            ))),
        }
    }

    /// Validate price data
    async fn validate_price(&self, price: &PriceData) -> StablecoinResult<bool> {
        // Check if price is too old
        let age = Utc::now() - price.timestamp;
        if age.num_seconds() > self.config.max_price_age_seconds as i64 {
            return Ok(false);
        }

        // Check if price is reasonable (not zero or negative)
        if price.price <= Decimal::ZERO {
            return Ok(false);
        }

        // Check against cached price for deviation
        if let Some(cached) = self.price_cache.get(&price.asset_symbol) {
            let deviation = ((price.price - cached.price.price) / cached.price.price).abs();
            if deviation > self.config.max_price_deviation {
                warn!(
                    "Price deviation too high for {}: {}%", 
                    price.asset_symbol, 
                    deviation * Decimal::new(100, 0)
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Aggregate multiple prices using median
    async fn aggregate_prices(
        &self,
        mut prices: Vec<PriceData>,
        asset_symbol: &str,
    ) -> StablecoinResult<PriceData> {
        if prices.is_empty() {
            return Err(StablecoinError::OracleUnavailable(
                "No prices to aggregate".to_string()
            ));
        }

        // Sort prices
        prices.sort_by(|a, b| a.price.cmp(&b.price));

        // Calculate median price
        let median_price = if prices.len() % 2 == 0 {
            let mid = prices.len() / 2;
            (prices[mid - 1].price + prices[mid].price) / Decimal::new(2, 0)
        } else {
            prices[prices.len() / 2].price
        };

        // Use most recent timestamp
        let latest_timestamp = prices.iter().map(|p| p.timestamp).max().unwrap_or(Utc::now());

        Ok(PriceData {
            asset_symbol: asset_symbol.to_string(),
            price: median_price,
            timestamp: latest_timestamp,
            source: "aggregated".to_string(),
            confidence: self.calculate_confidence(&prices),
        })
    }

    /// Calculate confidence score based on price consistency
    fn calculate_confidence(&self, prices: &[PriceData]) -> Decimal {
        if prices.len() < 2 {
            return Decimal::new(50, 0); // 50% confidence for single source
        }

        let median_price = {
            let mut sorted_prices: Vec<_> = prices.iter().map(|p| p.price).collect();
            sorted_prices.sort();
            if sorted_prices.len() % 2 == 0 {
                let mid = sorted_prices.len() / 2;
                (sorted_prices[mid - 1] + sorted_prices[mid]) / Decimal::new(2, 0)
            } else {
                sorted_prices[sorted_prices.len() / 2]
            }
        };

        // Calculate average deviation from median
        let total_deviation: Decimal = prices
            .iter()
            .map(|p| ((p.price - median_price) / median_price).abs())
            .sum();
        
        let avg_deviation = total_deviation / Decimal::new(prices.len() as i64, 0);
        
        // Convert deviation to confidence (lower deviation = higher confidence)
        let confidence = Decimal::new(100, 0) - (avg_deviation * Decimal::new(1000, 0));
        confidence.max(Decimal::new(10, 0)).min(Decimal::new(100, 0))
    }

    /// Cache price data
    fn cache_price(&mut self, asset_symbol: &str, price: PriceData) {
        self.price_cache.insert(
            asset_symbol.to_string(),
            CachedPrice {
                price,
                cached_at: Utc::now(),
            },
        );
    }

    /// Get cached price if available
    fn get_cached_price(&self, asset_symbol: &str) -> Option<&CachedPrice> {
        self.price_cache.get(asset_symbol)
    }

    /// Check if cached price is stale
    fn is_price_stale(&self, cached: &CachedPrice) -> bool {
        let age = Utc::now() - cached.cached_at;
        age.num_seconds() > self.config.cache_ttl_seconds as i64
    }

    /// Get oracle health status
    pub async fn get_health_status(&self) -> OracleAggregatorHealth {
        let mut oracle_healths = Vec::new();

        // Check primary oracle
        match self.primary_oracle.health_check().await {
            Ok(health) => oracle_healths.push(("primary".to_string(), health)),
            Err(e) => oracle_healths.push((
                "primary".to_string(),
                OracleHealth {
                    is_healthy: false,
                    last_update: None,
                    error_message: Some(e.to_string()),
                    response_time_ms: None,
                },
            )),
        }

        // Check backup oracles
        for (i, oracle) in self.backup_oracles.iter().enumerate() {
            match oracle.health_check().await {
                Ok(health) => oracle_healths.push((format!("backup_{}", i), health)),
                Err(e) => oracle_healths.push((
                    format!("backup_{}", i),
                    OracleHealth {
                        is_healthy: false,
                        last_update: None,
                        error_message: Some(e.to_string()),
                        response_time_ms: None,
                    },
                )),
            }
        }

        let healthy_count = oracle_healths.iter().filter(|(_, h)| h.is_healthy).count();
        let total_count = oracle_healths.len();

        OracleAggregatorHealth {
            overall_healthy: healthy_count >= self.config.min_oracle_sources as usize,
            healthy_oracles: healthy_count,
            total_oracles: total_count,
            oracle_healths,
            circuit_breaker_open: self.circuit_breaker.is_open(),
            cache_size: self.price_cache.len(),
        }
    }
}

/// Price data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub asset_symbol: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub confidence: Decimal, // 0-100
}

/// Cached price with metadata
#[derive(Debug, Clone)]
struct CachedPrice {
    price: PriceData,
    cached_at: DateTime<Utc>,
}

/// Oracle configuration
#[derive(Debug, Clone)]
pub struct OracleConfig {
    pub timeout_seconds: u64,
    pub max_price_age_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub min_oracle_sources: u32,
    pub max_price_deviation: Decimal,
    pub emergency_cache_minutes: u32,
    pub circuit_breaker_threshold: u32,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            max_price_age_seconds: 300, // 5 minutes
            cache_ttl_seconds: 60,      // 1 minute
            min_oracle_sources: 2,
            max_price_deviation: Decimal::new(5, 2), // 5%
            emergency_cache_minutes: 30,
            circuit_breaker_threshold: 5,
        }
    }
}

/// Oracle health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleHealth {
    pub is_healthy: bool,
    pub last_update: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Oracle metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleMetadata {
    pub name: String,
    pub version: String,
    pub supported_assets: Vec<String>,
    pub update_frequency_seconds: u64,
    pub reliability_score: Decimal, // 0-100
}

/// Aggregator health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAggregatorHealth {
    pub overall_healthy: bool,
    pub healthy_oracles: usize,
    pub total_oracles: usize,
    pub oracle_healths: Vec<(String, OracleHealth)>,
    pub circuit_breaker_open: bool,
    pub cache_size: usize,
}

/// Circuit breaker for oracle failures
#[derive(Debug, Clone)]
struct CircuitBreaker {
    failure_count: u32,
    threshold: u32,
    last_failure: Option<DateTime<Utc>>,
    state: CircuitBreakerState,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    fn new(threshold: u32) -> Self {
        Self {
            failure_count: 0,
            threshold,
            last_failure: None,
            state: CircuitBreakerState::Closed,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Utc::now());
        
        if self.failure_count >= self.threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    fn is_open(&self) -> bool {
        matches!(self.state, CircuitBreakerState::Open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock oracle for testing
    struct MockOracle {
        name: String,
        prices: Arc<Mutex<HashMap<String, PriceData>>>,
        should_fail: bool,
    }

    impl MockOracle {
        fn new(name: &str, should_fail: bool) -> Self {
            let mut prices = HashMap::new();
            prices.insert("BTC".to_string(), PriceData {
                asset_symbol: "BTC".to_string(),
                price: Decimal::new(50000, 0),
                timestamp: Utc::now(),
                source: name.to_string(),
                confidence: Decimal::new(95, 0),
            });
            
            Self {
                name: name.to_string(),
                prices: Arc::new(Mutex::new(prices)),
                should_fail,
            }
        }
    }

    #[async_trait]
    impl PriceOracle for MockOracle {
        async fn get_price(&self, asset_symbol: &str) -> StablecoinResult<PriceData> {
            if self.should_fail {
                return Err(StablecoinError::OracleUnavailable("Mock failure".to_string()));
            }

            let prices = self.prices.lock().await;
            prices.get(asset_symbol)
                .cloned()
                .ok_or_else(|| StablecoinError::OracleUnavailable(
                    format!("Asset {} not found", asset_symbol)
                ))
        }

        async fn get_historical_prices(
            &self,
            _asset_symbol: &str,
            _from: DateTime<Utc>,
            _to: DateTime<Utc>,
        ) -> StablecoinResult<Vec<PriceData>> {
            Ok(vec![])
        }

        async fn health_check(&self) -> StablecoinResult<OracleHealth> {
            Ok(OracleHealth {
                is_healthy: !self.should_fail,
                last_update: Some(Utc::now()),
                error_message: None,
                response_time_ms: Some(10),
            })
        }

        fn supported_assets(&self) -> Vec<String> {
            vec!["BTC".to_string(), "ETH".to_string()]
        }

        fn metadata(&self) -> OracleMetadata {
            OracleMetadata {
                name: self.name.clone(),
                version: "1.0.0".to_string(),
                supported_assets: self.supported_assets(),
                update_frequency_seconds: 60,
                reliability_score: Decimal::new(95, 0),
            }
        }
    }

    #[tokio::test]
    async fn test_oracle_aggregator_primary_success() {
        let primary = Box::new(MockOracle::new("primary", false));
        let backup = vec![Box::new(MockOracle::new("backup", false)) as Box<dyn PriceOracle>];
        let config = OracleConfig::default();
        
        let mut aggregator = OracleAggregator::new(primary, backup, config);
        
        let price = aggregator.get_validated_price("BTC").await.unwrap();
        assert_eq!(price.asset_symbol, "BTC");
        assert_eq!(price.price, Decimal::new(50000, 0));
        assert_eq!(price.source, "primary");
    }

    #[tokio::test]
    async fn test_oracle_aggregator_failover() {
        let primary = Box::new(MockOracle::new("primary", true)); // Will fail
        let backup = vec![Box::new(MockOracle::new("backup", false)) as Box<dyn PriceOracle>];
        let config = OracleConfig::default();
        
        let mut aggregator = OracleAggregator::new(primary, backup, config);
        
        let price = aggregator.get_validated_price("BTC").await.unwrap();
        assert_eq!(price.asset_symbol, "BTC");
        assert_eq!(price.source, "backup");
    }

    #[tokio::test]
    async fn test_oracle_aggregator_all_fail() {
        let primary = Box::new(MockOracle::new("primary", true));
        let backup = vec![Box::new(MockOracle::new("backup", true)) as Box<dyn PriceOracle>];
        let config = OracleConfig::default();
        
        let mut aggregator = OracleAggregator::new(primary, backup, config);
        
        let result = aggregator.get_validated_price("BTC").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StablecoinError::OracleUnavailable(_)));
    }

    #[tokio::test]
    async fn test_price_aggregation() {
        let primary = Box::new(MockOracle::new("primary", false));
        let backup = vec![Box::new(MockOracle::new("backup", false)) as Box<dyn PriceOracle>];
        let config = OracleConfig::default();
        
        let aggregator = OracleAggregator::new(primary, backup, config);
        
        let price = aggregator.get_aggregated_price("BTC").await.unwrap();
        assert_eq!(price.asset_symbol, "BTC");
        assert_eq!(price.source, "aggregated");
        assert!(price.confidence > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut circuit_breaker = CircuitBreaker::new(3);
        
        // Record failures
        circuit_breaker.record_failure();
        circuit_breaker.record_failure();
        assert!(!circuit_breaker.is_open());
        
        circuit_breaker.record_failure();
        assert!(circuit_breaker.is_open());
        
        // Record success should reset
        circuit_breaker.record_success();
        assert!(!circuit_breaker.is_open());
    }

    #[tokio::test]
    async fn test_price_validation() {
        let primary = Box::new(MockOracle::new("primary", false));
        let backup = vec![];
        let config = OracleConfig::default();
        
        let aggregator = OracleAggregator::new(primary, backup, config);
        
        // Valid price
        let valid_price = PriceData {
            asset_symbol: "BTC".to_string(),
            price: Decimal::new(50000, 0),
            timestamp: Utc::now(),
            source: "test".to_string(),
            confidence: Decimal::new(95, 0),
        };
        assert!(aggregator.validate_price(&valid_price).await.unwrap());
        
        // Invalid price (zero)
        let invalid_price = PriceData {
            asset_symbol: "BTC".to_string(),
            price: Decimal::ZERO,
            timestamp: Utc::now(),
            source: "test".to_string(),
            confidence: Decimal::new(95, 0),
        };
        assert!(!aggregator.validate_price(&invalid_price).await.unwrap());
        
        // Stale price
        let stale_price = PriceData {
            asset_symbol: "BTC".to_string(),
            price: Decimal::new(50000, 0),
            timestamp: Utc::now() - chrono::Duration::hours(1),
            source: "test".to_string(),
            confidence: Decimal::new(95, 0),
        };
        assert!(!aggregator.validate_price(&stale_price).await.unwrap());
    }

    #[tokio::test]
    async fn test_health_status() {
        let primary = Box::new(MockOracle::new("primary", false));
        let backup = vec![Box::new(MockOracle::new("backup", true)) as Box<dyn PriceOracle>];
        let config = OracleConfig::default();
        
        let aggregator = OracleAggregator::new(primary, backup, config);
        
        let health = aggregator.get_health_status().await;
        assert_eq!(health.total_oracles, 2);
        assert_eq!(health.healthy_oracles, 1);
        assert!(!health.overall_healthy); // min_oracle_sources is 2, but we have only 1 healthy
    }
}
