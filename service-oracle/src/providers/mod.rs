// =====================================================================================
// RWA Tokenization Platform - Oracle Price Providers
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod binance;
pub mod coingecko;
pub mod coinmarketcap;
// pub mod chainlink; // Disabled until ethers dependency is added

use crate::error::{OracleError, OracleResult};
use crate::models::{AssetPrice, ProviderType};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Price provider trait that all providers must implement
#[async_trait]
pub trait PriceProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;
    
    /// Get the provider type
    fn provider_type(&self) -> ProviderType;
    
    /// Get the provider weight for aggregation
    fn weight(&self) -> Decimal;
    
    /// Check if the provider is healthy
    async fn health_check(&self) -> OracleResult<bool>;
    
    /// Get price for a single asset
    async fn get_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice>;
    
    /// Get prices for multiple assets (batch operation)
    async fn get_batch_prices(&self, asset_ids: &[String], currency: &str) -> OracleResult<Vec<AssetPrice>>;
    
    /// Get supported assets
    async fn get_supported_assets(&self) -> OracleResult<Vec<String>>;
    
    /// Get rate limit information
    fn get_rate_limit(&self) -> RateLimit;
}

/// Rate limiting information
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub current_usage: u32,
    pub reset_time: DateTime<Utc>,
}

/// Provider manager for handling multiple price providers
pub struct ProviderManager {
    providers: HashMap<String, Arc<dyn PriceProvider>>,
    rate_limiters: HashMap<String, RateLimiter>,
}

/// Simple rate limiter implementation
pub struct RateLimiter {
    requests_per_minute: u32,
    requests: Vec<DateTime<Utc>>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            rate_limiters: HashMap::new(),
        }
    }
    
    /// Add a provider to the manager
    pub fn add_provider(&mut self, provider: Arc<dyn PriceProvider>) {
        let name = provider.name().to_string();
        let rate_limit = provider.get_rate_limit();
        
        self.providers.insert(name.clone(), provider);
        self.rate_limiters.insert(
            name,
            RateLimiter::new(rate_limit.requests_per_minute),
        );
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&Arc<dyn PriceProvider>> {
        self.providers.get(name)
    }
    
    /// Get all provider names
    pub fn get_provider_names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
    
    /// Get price from a specific provider with rate limiting
    pub async fn get_price_from_provider(
        &mut self,
        provider_name: &str,
        asset_id: &str,
        currency: &str,
        timeout_duration: Duration,
    ) -> OracleResult<AssetPrice> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| OracleError::Provider {
                provider: provider_name.to_string(),
                message: "Provider not found".to_string(),
            })?;
        
        // Check rate limit
        if let Some(rate_limiter) = self.rate_limiters.get_mut(provider_name) {
            if !rate_limiter.can_make_request() {
                return Err(OracleError::RateLimitExceeded {
                    provider: provider_name.to_string(),
                });
            }
            rate_limiter.record_request();
        }
        
        // Make request with timeout
        let result = timeout(
            timeout_duration,
            provider.get_price(asset_id, currency)
        ).await;
        
        match result {
            Ok(Ok(price)) => Ok(price),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(OracleError::Timeout {
                operation: format!("get_price from {}", provider_name),
            }),
        }
    }
    
    /// Get prices from multiple providers
    pub async fn get_prices_from_providers(
        &mut self,
        provider_names: &[String],
        asset_id: &str,
        currency: &str,
        timeout_duration: Duration,
    ) -> HashMap<String, OracleResult<AssetPrice>> {
        let mut results = HashMap::new();
        
        for provider_name in provider_names {
            let result = self.get_price_from_provider(
                provider_name,
                asset_id,
                currency,
                timeout_duration,
            ).await;
            results.insert(provider_name.clone(), result);
        }
        
        results
    }
    
    /// Check health of all providers
    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        
        for (name, provider) in &self.providers {
            let health = provider.health_check().await.unwrap_or(false);
            results.insert(name.clone(), health);
        }
        
        results
    }
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            requests: Vec::new(),
        }
    }
    
    /// Check if a request can be made
    pub fn can_make_request(&mut self) -> bool {
        self.cleanup_old_requests();
        self.requests.len() < self.requests_per_minute as usize
    }
    
    /// Record a new request
    pub fn record_request(&mut self) {
        self.requests.push(Utc::now());
    }
    
    /// Remove requests older than 1 minute
    fn cleanup_old_requests(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::minutes(1);
        self.requests.retain(|&time| time > cutoff);
    }
    
    /// Get current usage
    pub fn current_usage(&mut self) -> u32 {
        self.cleanup_old_requests();
        self.requests.len() as u32
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to validate asset ID format
pub fn validate_asset_id(asset_id: &str) -> OracleResult<()> {
    if asset_id.is_empty() {
        return Err(OracleError::Validation {
            field: "asset_id".to_string(),
            message: "cannot be empty".to_string(),
        });
    }
    
    if asset_id.len() > 20 {
        return Err(OracleError::Validation {
            field: "asset_id".to_string(),
            message: "cannot be longer than 20 characters".to_string(),
        });
    }
    
    if !asset_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(OracleError::Validation {
            field: "asset_id".to_string(),
            message: "can only contain alphanumeric characters, hyphens, and underscores".to_string(),
        });
    }
    
    Ok(())
}

/// Helper function to validate currency format
pub fn validate_currency(currency: &str) -> OracleResult<()> {
    if currency.is_empty() {
        return Err(OracleError::Validation {
            field: "currency".to_string(),
            message: "cannot be empty".to_string(),
        });
    }
    
    if currency.len() != 3 {
        return Err(OracleError::Validation {
            field: "currency".to_string(),
            message: "must be exactly 3 characters (ISO 4217 format)".to_string(),
        });
    }
    
    if !currency.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(OracleError::Validation {
            field: "currency".to_string(),
            message: "must be uppercase letters only".to_string(),
        });
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetPrice, ProviderType};
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Mock provider for testing
    struct MockProvider {
        name: String,
        weight: Decimal,
        should_fail: bool,
        delay_ms: u64,
    }

    #[async_trait]
    impl PriceProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn provider_type(&self) -> ProviderType {
            ProviderType::Custom("mock".to_string())
        }

        fn weight(&self) -> Decimal {
            self.weight
        }

        async fn health_check(&self) -> OracleResult<bool> {
            Ok(!self.should_fail)
        }

        async fn get_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
            if self.should_fail {
                return Err(OracleError::Provider {
                    provider: self.name.clone(),
                    message: "Mock failure".to_string(),
                });
            }

            if self.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            }

            Ok(AssetPrice {
                asset_id: asset_id.to_string(),
                price: dec!(50000.0),
                currency: currency.to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.95),
                source: self.name.clone(),
                metadata: Some(HashMap::new()),
            })
        }

        async fn get_batch_prices(&self, asset_ids: &[String], currency: &str) -> OracleResult<Vec<AssetPrice>> {
            let mut prices = Vec::new();
            for asset_id in asset_ids {
                prices.push(self.get_price(asset_id, currency).await?);
            }
            Ok(prices)
        }

        async fn get_supported_assets(&self) -> OracleResult<Vec<String>> {
            Ok(vec!["BTC".to_string(), "ETH".to_string()])
        }

        fn get_rate_limit(&self) -> RateLimit {
            RateLimit {
                requests_per_minute: 60,
                current_usage: 0,
                reset_time: Utc::now() + chrono::Duration::minutes(1),
            }
        }
    }

    #[test]
    fn test_provider_manager_creation() {
        let manager = ProviderManager::new();
        assert!(manager.providers.is_empty());
        assert!(manager.rate_limiters.is_empty());
    }

    #[test]
    fn test_add_provider() {
        let mut manager = ProviderManager::new();
        let provider = Arc::new(MockProvider {
            name: "test_provider".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 0,
        });

        manager.add_provider(provider);

        assert_eq!(manager.providers.len(), 1);
        assert_eq!(manager.rate_limiters.len(), 1);
        assert!(manager.get_provider("test_provider").is_some());
    }

    #[tokio::test]
    async fn test_get_price_from_provider_success() {
        let mut manager = ProviderManager::new();
        let provider = Arc::new(MockProvider {
            name: "test_provider".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 0,
        });

        manager.add_provider(provider);

        let result = manager.get_price_from_provider(
            "test_provider",
            "BTC",
            "USD",
            Duration::from_secs(5),
        ).await;

        assert!(result.is_ok());
        let price = result.unwrap();
        assert_eq!(price.asset_id, "BTC");
        assert_eq!(price.currency, "USD");
        assert_eq!(price.price, dec!(50000.0));
    }

    #[tokio::test]
    async fn test_get_price_from_provider_failure() {
        let mut manager = ProviderManager::new();
        let provider = Arc::new(MockProvider {
            name: "failing_provider".to_string(),
            weight: dec!(1.0),
            should_fail: true,
            delay_ms: 0,
        });

        manager.add_provider(provider);

        let result = manager.get_price_from_provider(
            "failing_provider",
            "BTC",
            "USD",
            Duration::from_secs(5),
        ).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OracleError::Provider { .. }));
    }

    #[tokio::test]
    async fn test_get_price_timeout() {
        let mut manager = ProviderManager::new();
        let provider = Arc::new(MockProvider {
            name: "slow_provider".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 2000, // 2 second delay
        });

        manager.add_provider(provider);

        let result = manager.get_price_from_provider(
            "slow_provider",
            "BTC",
            "USD",
            Duration::from_millis(100), // 100ms timeout
        ).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OracleError::Timeout { .. }));
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let mut manager = ProviderManager::new();

        let healthy_provider = Arc::new(MockProvider {
            name: "healthy".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 0,
        });

        let unhealthy_provider = Arc::new(MockProvider {
            name: "unhealthy".to_string(),
            weight: dec!(1.0),
            should_fail: true,
            delay_ms: 0,
        });

        manager.add_provider(healthy_provider);
        manager.add_provider(unhealthy_provider);

        let health_results = manager.health_check_all().await;

        assert_eq!(health_results.len(), 2);
        assert_eq!(health_results.get("healthy"), Some(&true));
        assert_eq!(health_results.get("unhealthy"), Some(&false));
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);

        // Should allow first request
        assert!(limiter.can_make_request());
        limiter.record_request();

        // Should allow second request
        assert!(limiter.can_make_request());
        limiter.record_request();

        // Should not allow third request
        assert!(!limiter.can_make_request());

        // Current usage should be 2
        assert_eq!(limiter.current_usage(), 2);
    }

    #[test]
    fn test_validate_asset_id() {
        // Valid asset IDs
        assert!(validate_asset_id("BTC").is_ok());
        assert!(validate_asset_id("ETH-USD").is_ok());
        assert!(validate_asset_id("ASSET_123").is_ok());

        // Invalid asset IDs
        assert!(validate_asset_id("").is_err());
        assert!(validate_asset_id("VERY_LONG_ASSET_ID_NAME").is_err());
        assert!(validate_asset_id("BTC@USD").is_err());
        assert!(validate_asset_id("BTC USD").is_err());
    }

    #[test]
    fn test_validate_currency() {
        // Valid currencies
        assert!(validate_currency("USD").is_ok());
        assert!(validate_currency("EUR").is_ok());
        assert!(validate_currency("GBP").is_ok());

        // Invalid currencies
        assert!(validate_currency("").is_err());
        assert!(validate_currency("US").is_err());
        assert!(validate_currency("USDT").is_err());
        assert!(validate_currency("usd").is_err());
        assert!(validate_currency("US1").is_err());
    }

    #[tokio::test]
    async fn test_get_prices_from_multiple_providers() {
        let mut manager = ProviderManager::new();

        let provider1 = Arc::new(MockProvider {
            name: "provider1".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 0,
        });

        let provider2 = Arc::new(MockProvider {
            name: "provider2".to_string(),
            weight: dec!(1.0),
            should_fail: true,
            delay_ms: 0,
        });

        manager.add_provider(provider1);
        manager.add_provider(provider2);

        let provider_names = vec!["provider1".to_string(), "provider2".to_string()];
        let results = manager.get_prices_from_providers(
            &provider_names,
            "BTC",
            "USD",
            Duration::from_secs(5),
        ).await;

        assert_eq!(results.len(), 2);
        assert!(results.get("provider1").unwrap().is_ok());
        assert!(results.get("provider2").unwrap().is_err());
    }

    #[tokio::test]
    async fn test_rate_limit_exceeded() {
        let mut manager = ProviderManager::new();
        let provider = Arc::new(MockProvider {
            name: "limited_provider".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 0,
        });

        manager.add_provider(provider);

        // Make requests up to the limit
        for _ in 0..60 {
            let result = manager.get_price_from_provider(
                "limited_provider",
                "BTC",
                "USD",
                Duration::from_secs(1),
            ).await;
            assert!(result.is_ok());
        }

        // Next request should be rate limited
        let result = manager.get_price_from_provider(
            "limited_provider",
            "BTC",
            "USD",
            Duration::from_secs(1),
        ).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OracleError::RateLimitExceeded { .. }));
    }

    #[test]
    fn test_provider_not_found() {
        let manager = ProviderManager::new();
        assert!(manager.get_provider("nonexistent").is_none());
    }

    #[test]
    fn test_get_provider_names() {
        let mut manager = ProviderManager::new();

        let provider1 = Arc::new(MockProvider {
            name: "provider1".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 0,
        });

        let provider2 = Arc::new(MockProvider {
            name: "provider2".to_string(),
            weight: dec!(1.0),
            should_fail: false,
            delay_ms: 0,
        });

        manager.add_provider(provider1);
        manager.add_provider(provider2);

        let names = manager.get_provider_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"provider1".to_string()));
        assert!(names.contains(&"provider2".to_string()));
    }
}
