// =====================================================================================
// RWA Tokenization Platform - Price Aggregator
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{OracleError, OracleResult};
use crate::models::{AggregationMethod, AssetPrice};
use crate::providers::ProviderManager;
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
// Removed unused import: std::sync::Arc
use tokio::time::Duration;
use tracing::{debug, warn};

/// Price aggregator for combining prices from multiple providers
pub struct PriceAggregator {
    provider_manager: ProviderManager,
    min_sources: usize,
    max_deviation_percent: Decimal,
    confidence_threshold: Decimal,
    outlier_detection: bool,
    timeout_duration: Duration,
}

/// Aggregation result with detailed information
#[derive(Debug, Clone)]
pub struct AggregationResult {
    pub final_price: AssetPrice,
    pub source_prices: Vec<AssetPrice>,
    pub failed_providers: Vec<String>,
    pub outliers_removed: Vec<AssetPrice>,
    pub aggregation_method: AggregationMethod,
    pub quality_score: Decimal,
}

/// Statistical information about price data
#[derive(Debug, Clone)]
pub struct PriceStatistics {
    pub mean: Decimal,
    pub median: Decimal,
    pub std_deviation: Decimal,
    pub min_price: Decimal,
    pub max_price: Decimal,
    pub price_range_percent: Decimal,
}

impl PriceAggregator {
    /// Create a new price aggregator
    pub fn new(
        provider_manager: ProviderManager,
        min_sources: usize,
        max_deviation_percent: Decimal,
        confidence_threshold: Decimal,
        outlier_detection: bool,
        timeout_duration: Duration,
    ) -> Self {
        Self {
            provider_manager,
            min_sources,
            max_deviation_percent,
            confidence_threshold,
            outlier_detection,
            timeout_duration,
        }
    }

    /// Aggregate price from multiple providers
    pub async fn aggregate_price(
        &mut self,
        asset_id: &str,
        currency: &str,
        method: &AggregationMethod,
        provider_names: Option<&[String]>,
    ) -> OracleResult<AggregationResult> {
        debug!("Starting price aggregation for {} in {}", asset_id, currency);

        // Get provider names to use
        let providers_to_use = match provider_names {
            Some(names) => names.to_vec(),
            None => self.provider_manager.get_provider_names(),
        };

        if providers_to_use.is_empty() {
            return Err(OracleError::InsufficientDataSources {
                required: self.min_sources,
                available: 0,
            });
        }

        // Fetch prices from all providers
        let price_results = self.provider_manager.get_prices_from_providers(
            &providers_to_use,
            asset_id,
            currency,
            self.timeout_duration,
        ).await;

        // Separate successful and failed results
        let mut source_prices = Vec::new();
        let mut failed_providers = Vec::new();

        for (provider_name, result) in price_results {
            match result {
                Ok(price) => {
                    if price.confidence >= self.confidence_threshold {
                        source_prices.push(price);
                    } else {
                        warn!("Price from {} has low confidence: {}", provider_name, price.confidence);
                        failed_providers.push(format!("{} (low confidence)", provider_name));
                    }
                }
                Err(e) => {
                    warn!("Failed to get price from {}: {}", provider_name, e);
                    failed_providers.push(provider_name);
                }
            }
        }

        // Check if we have enough sources
        if source_prices.len() < self.min_sources {
            return Err(OracleError::InsufficientDataSources {
                required: self.min_sources,
                available: source_prices.len(),
            });
        }

        // Remove outliers if enabled
        let mut outliers_removed = Vec::new();
        if self.outlier_detection && source_prices.len() > 2 {
            let (filtered_prices, outliers) = self.remove_outliers(&source_prices)?;
            source_prices = filtered_prices;
            outliers_removed = outliers;

            // Check again after outlier removal
            if source_prices.len() < self.min_sources {
                return Err(OracleError::InsufficientDataSources {
                    required: self.min_sources,
                    available: source_prices.len(),
                });
            }
        }

        // Check price deviation
        let stats = self.calculate_statistics(&source_prices)?;
        if stats.price_range_percent > self.max_deviation_percent {
            return Err(OracleError::PriceDeviationTooHigh {
                deviation: stats.price_range_percent.to_string().parse().unwrap_or(0.0),
                threshold: self.max_deviation_percent.to_string().parse().unwrap_or(0.0),
            });
        }

        // Aggregate the prices
        let aggregated_price = self.apply_aggregation_method(&source_prices, method)?;
        let quality_score = self.calculate_quality_score(&source_prices, &stats);

        let final_price = AssetPrice {
            asset_id: asset_id.to_string(),
            price: aggregated_price,
            currency: currency.to_string(),
            timestamp: Utc::now(),
            confidence: quality_score,
            source: "Aggregated".to_string(),
            metadata: Some(self.create_aggregation_metadata(&source_prices, &stats)),
        };

        debug!("Price aggregation completed successfully for {}", asset_id);

        Ok(AggregationResult {
            final_price,
            source_prices,
            failed_providers,
            outliers_removed,
            aggregation_method: method.clone(),
            quality_score,
        })
    }

    /// Apply the specified aggregation method
    fn apply_aggregation_method(
        &self,
        prices: &[AssetPrice],
        method: &AggregationMethod,
    ) -> OracleResult<Decimal> {
        if prices.is_empty() {
            return Err(OracleError::AggregationFailed {
                reason: "No prices to aggregate".to_string(),
            });
        }

        match method {
            AggregationMethod::Mean => {
                let sum: Decimal = prices.iter().map(|p| p.price).sum();
                Ok(sum / Decimal::from(prices.len()))
            }
            AggregationMethod::Median => {
                let mut sorted_prices: Vec<Decimal> = prices.iter().map(|p| p.price).collect();
                sorted_prices.sort();
                
                let len = sorted_prices.len();
                if len % 2 == 0 {
                    Ok((sorted_prices[len / 2 - 1] + sorted_prices[len / 2]) / Decimal::from(2))
                } else {
                    Ok(sorted_prices[len / 2])
                }
            }
            AggregationMethod::WeightedAverage => {
                let mut weighted_sum = Decimal::ZERO;
                let mut total_weight = Decimal::ZERO;

                for price in prices {
                    // Use confidence as weight if available, otherwise use provider weight
                    let weight = price.confidence;
                    weighted_sum += price.price * weight;
                    total_weight += weight;
                }

                if total_weight == Decimal::ZERO {
                    return Err(OracleError::AggregationFailed {
                        reason: "Total weight is zero".to_string(),
                    });
                }

                Ok(weighted_sum / total_weight)
            }
            AggregationMethod::VolumeWeighted => {
                // For volume-weighted, we'd need volume data from metadata
                // For now, fall back to weighted average
                self.apply_aggregation_method(prices, &AggregationMethod::WeightedAverage)
            }
            AggregationMethod::Custom(method_name) => {
                Err(OracleError::AggregationFailed {
                    reason: format!("Custom aggregation method '{}' not implemented", method_name),
                })
            }
        }
    }

    /// Remove outliers using IQR method
    fn remove_outliers(&self, prices: &[AssetPrice]) -> OracleResult<(Vec<AssetPrice>, Vec<AssetPrice>)> {
        if prices.len() < 3 {
            return Ok((prices.to_vec(), Vec::new()));
        }

        let mut sorted_prices: Vec<(Decimal, &AssetPrice)> = prices.iter()
            .map(|p| (p.price, p))
            .collect();
        sorted_prices.sort_by(|a, b| a.0.cmp(&b.0));

        let len = sorted_prices.len();
        let q1_index = len / 4;
        let q3_index = (3 * len) / 4;

        let q1 = sorted_prices[q1_index].0;
        let q3 = sorted_prices[q3_index].0;
        let iqr = q3 - q1;
        let lower_bound = q1 - (iqr * Decimal::from_str("1.5").unwrap());
        let upper_bound = q3 + (iqr * Decimal::from_str("1.5").unwrap());

        let mut filtered = Vec::new();
        let mut outliers = Vec::new();

        for price in prices {
            if price.price >= lower_bound && price.price <= upper_bound {
                filtered.push(price.clone());
            } else {
                outliers.push(price.clone());
            }
        }

        Ok((filtered, outliers))
    }

    /// Calculate statistical information about prices
    fn calculate_statistics(&self, prices: &[AssetPrice]) -> OracleResult<PriceStatistics> {
        if prices.is_empty() {
            return Err(OracleError::AggregationFailed {
                reason: "Cannot calculate statistics for empty price list".to_string(),
            });
        }

        let price_values: Vec<Decimal> = prices.iter().map(|p| p.price).collect();
        
        let min_price = price_values.iter().min().unwrap().clone();
        let max_price = price_values.iter().max().unwrap().clone();
        
        let sum: Decimal = price_values.iter().sum();
        let mean = sum / Decimal::from(price_values.len());

        // Calculate median
        let mut sorted_prices = price_values.clone();
        sorted_prices.sort();
        let len = sorted_prices.len();
        let median = if len % 2 == 0 {
            (sorted_prices[len / 2 - 1] + sorted_prices[len / 2]) / Decimal::from(2)
        } else {
            sorted_prices[len / 2]
        };

        // Calculate standard deviation
        let variance: Decimal = price_values.iter()
            .map(|price| {
                let diff = *price - mean;
                diff * diff
            })
            .sum::<Decimal>() / Decimal::from(price_values.len());
        
        // Simple approximation for square root since Decimal doesn't have sqrt
        let std_deviation = if variance > Decimal::ZERO {
            // Convert to f64, take sqrt, then back to Decimal
            let variance_f64 = variance.to_string().parse::<f64>().unwrap_or(0.0);
            Decimal::from_str(&variance_f64.sqrt().to_string()).unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        // Calculate price range percentage
        let price_range_percent = if mean > Decimal::ZERO {
            ((max_price - min_price) / mean) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };

        Ok(PriceStatistics {
            mean,
            median,
            std_deviation,
            min_price,
            max_price,
            price_range_percent,
        })
    }

    /// Calculate quality score based on various factors
    fn calculate_quality_score(&self, prices: &[AssetPrice], stats: &PriceStatistics) -> Decimal {
        let mut score = Decimal::from_str("0.5").unwrap(); // Base score

        // Factor 1: Number of sources (more sources = higher quality)
        let source_factor = Decimal::from(prices.len().min(10)) / Decimal::from(10);
        score += source_factor * Decimal::from_str("0.2").unwrap();

        // Factor 2: Average confidence of sources
        let avg_confidence: Decimal = prices.iter().map(|p| p.confidence).sum::<Decimal>() 
            / Decimal::from(prices.len());
        score += avg_confidence * Decimal::from_str("0.2").unwrap();

        // Factor 3: Price consistency (lower deviation = higher quality)
        let consistency_factor = if stats.price_range_percent > Decimal::ZERO {
            (Decimal::from(10) - stats.price_range_percent.min(Decimal::from(10))) / Decimal::from(10)
        } else {
            Decimal::ONE
        };
        score += consistency_factor * Decimal::from_str("0.1").unwrap();

        // Ensure score is between 0 and 1
        score.max(Decimal::ZERO).min(Decimal::ONE)
    }

    /// Create metadata for aggregated price
    fn create_aggregation_metadata(&self, prices: &[AssetPrice], stats: &PriceStatistics) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        
        metadata.insert("source_count".to_string(), serde_json::Value::Number(
            serde_json::Number::from(prices.len())
        ));
        
        metadata.insert("mean_price".to_string(), serde_json::Value::String(
            stats.mean.to_string()
        ));
        
        metadata.insert("median_price".to_string(), serde_json::Value::String(
            stats.median.to_string()
        ));
        
        metadata.insert("std_deviation".to_string(), serde_json::Value::String(
            stats.std_deviation.to_string()
        ));
        
        metadata.insert("price_range_percent".to_string(), serde_json::Value::String(
            stats.price_range_percent.to_string()
        ));
        
        let sources: Vec<String> = prices.iter().map(|p| p.source.clone()).collect();
        metadata.insert("sources".to_string(), serde_json::Value::Array(
            sources.into_iter().map(serde_json::Value::String).collect()
        ));

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetPrice, ProviderType};
    use crate::providers::{PriceProvider, ProviderManager};
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Mock provider for testing
    struct MockProvider {
        name: String,
        prices: HashMap<String, Decimal>,
        confidence: Decimal,
        should_fail: bool,
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
            dec!(1.0)
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

            let key = format!("{}_{}", asset_id, currency);
            let price = self.prices.get(&key).copied().unwrap_or(dec!(100.0));

            Ok(AssetPrice {
                asset_id: asset_id.to_string(),
                price,
                currency: currency.to_string(),
                timestamp: Utc::now(),
                confidence: self.confidence,
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

        fn get_rate_limit(&self) -> crate::providers::RateLimit {
            crate::providers::RateLimit {
                requests_per_minute: 60,
                current_usage: 0,
                reset_time: Utc::now() + chrono::Duration::minutes(1),
            }
        }
    }

    fn create_test_aggregator() -> PriceAggregator {
        let mut provider_manager = ProviderManager::new();

        // Add mock providers with different prices
        let mut prices1 = HashMap::new();
        prices1.insert("BTC_USD".to_string(), dec!(50000.0));
        let provider1 = Arc::new(MockProvider {
            name: "provider1".to_string(),
            prices: prices1,
            confidence: dec!(0.9),
            should_fail: false,
        });

        let mut prices2 = HashMap::new();
        prices2.insert("BTC_USD".to_string(), dec!(50100.0));
        let provider2 = Arc::new(MockProvider {
            name: "provider2".to_string(),
            prices: prices2,
            confidence: dec!(0.85),
            should_fail: false,
        });

        let mut prices3 = HashMap::new();
        prices3.insert("BTC_USD".to_string(), dec!(49900.0));
        let provider3 = Arc::new(MockProvider {
            name: "provider3".to_string(),
            prices: prices3,
            confidence: dec!(0.95),
            should_fail: false,
        });

        provider_manager.add_provider(provider1);
        provider_manager.add_provider(provider2);
        provider_manager.add_provider(provider3);

        PriceAggregator::new(
            provider_manager,
            2, // min_sources
            dec!(5.0), // max_deviation_percent
            dec!(0.8), // confidence_threshold
            true, // outlier_detection
            Duration::from_secs(5),
        )
    }

    #[tokio::test]
    async fn test_aggregate_price_mean() {
        let mut aggregator = create_test_aggregator();

        let result = aggregator.aggregate_price(
            "BTC",
            "USD",
            &AggregationMethod::Mean,
            None,
        ).await;

        assert!(result.is_ok());
        let aggregation_result = result.unwrap();

        // Mean of 50000, 50100, 49900 = 50000
        assert_eq!(aggregation_result.final_price.price, dec!(50000.0));
        assert_eq!(aggregation_result.source_prices.len(), 3);
        assert!(aggregation_result.failed_providers.is_empty());
        assert_eq!(aggregation_result.final_price.asset_id, "BTC");
        assert_eq!(aggregation_result.final_price.currency, "USD");
    }

    #[tokio::test]
    async fn test_aggregate_price_median() {
        let mut aggregator = create_test_aggregator();

        let result = aggregator.aggregate_price(
            "BTC",
            "USD",
            &AggregationMethod::Median,
            None,
        ).await;

        assert!(result.is_ok());
        let aggregation_result = result.unwrap();

        // Median of 49900, 50000, 50100 = 50000
        assert_eq!(aggregation_result.final_price.price, dec!(50000.0));
    }

    #[tokio::test]
    async fn test_aggregate_price_weighted_average() {
        let mut aggregator = create_test_aggregator();

        let result = aggregator.aggregate_price(
            "BTC",
            "USD",
            &AggregationMethod::WeightedAverage,
            None,
        ).await;

        assert!(result.is_ok());
        let aggregation_result = result.unwrap();

        // Weighted by confidence: (50000*0.9 + 50100*0.85 + 49900*0.95) / (0.9+0.85+0.95)
        let expected = (dec!(50000.0) * dec!(0.9) + dec!(50100.0) * dec!(0.85) + dec!(49900.0) * dec!(0.95))
                      / (dec!(0.9) + dec!(0.85) + dec!(0.95));
        assert!((aggregation_result.final_price.price - expected).abs() < dec!(0.01));
    }

    #[tokio::test]
    async fn test_insufficient_data_sources() {
        let mut provider_manager = ProviderManager::new();

        // Add only one provider but require 2 minimum
        let provider = Arc::new(MockProvider {
            name: "single_provider".to_string(),
            prices: HashMap::new(),
            confidence: dec!(0.9),
            should_fail: false,
        });
        provider_manager.add_provider(provider);

        let mut aggregator = PriceAggregator::new(
            provider_manager,
            2, // min_sources (more than available)
            dec!(5.0),
            dec!(0.8),
            true,
            Duration::from_secs(5),
        );

        let result = aggregator.aggregate_price(
            "BTC",
            "USD",
            &AggregationMethod::Mean,
            None,
        ).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OracleError::InsufficientDataSources { .. }));
    }

    #[tokio::test]
    async fn test_price_deviation_too_high() {
        let mut provider_manager = ProviderManager::new();

        // Add providers with very different prices
        let mut prices1 = HashMap::new();
        prices1.insert("BTC_USD".to_string(), dec!(50000.0));
        let provider1 = Arc::new(MockProvider {
            name: "provider1".to_string(),
            prices: prices1,
            confidence: dec!(0.9),
            should_fail: false,
        });

        let mut prices2 = HashMap::new();
        prices2.insert("BTC_USD".to_string(), dec!(60000.0)); // 20% higher
        let provider2 = Arc::new(MockProvider {
            name: "provider2".to_string(),
            prices: prices2,
            confidence: dec!(0.9),
            should_fail: false,
        });

        provider_manager.add_provider(provider1);
        provider_manager.add_provider(provider2);

        let mut aggregator = PriceAggregator::new(
            provider_manager,
            2,
            dec!(5.0), // max_deviation_percent (5%, but actual is ~18%)
            dec!(0.8),
            false, // disable outlier detection
            Duration::from_secs(5),
        );

        let result = aggregator.aggregate_price(
            "BTC",
            "USD",
            &AggregationMethod::Mean,
            None,
        ).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OracleError::PriceDeviationTooHigh { .. }));
    }

    #[test]
    fn test_calculate_statistics() {
        let aggregator = create_test_aggregator();

        let prices = vec![
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(100.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.9),
                source: "test".to_string(),
                metadata: None,
            },
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(110.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.9),
                source: "test".to_string(),
                metadata: None,
            },
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(90.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.9),
                source: "test".to_string(),
                metadata: None,
            },
        ];

        let stats = aggregator.calculate_statistics(&prices).unwrap();

        assert_eq!(stats.mean, dec!(100.0));
        assert_eq!(stats.median, dec!(100.0));
        assert_eq!(stats.min_price, dec!(90.0));
        assert_eq!(stats.max_price, dec!(110.0));
        assert_eq!(stats.price_range_percent, dec!(20.0)); // (110-90)/100 * 100
    }

    #[test]
    fn test_remove_outliers() {
        let aggregator = create_test_aggregator();

        let prices = vec![
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(100.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.9),
                source: "normal1".to_string(),
                metadata: None,
            },
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(105.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.9),
                source: "normal2".to_string(),
                metadata: None,
            },
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(95.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.9),
                source: "normal3".to_string(),
                metadata: None,
            },
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(200.0), // Outlier
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.9),
                source: "outlier".to_string(),
                metadata: None,
            },
        ];

        let (filtered, outliers) = aggregator.remove_outliers(&prices).unwrap();

        assert_eq!(filtered.len(), 3);
        assert_eq!(outliers.len(), 1);
        assert_eq!(outliers[0].source, "outlier");
    }

    #[test]
    fn test_quality_score_calculation() {
        let aggregator = create_test_aggregator();

        let prices = vec![
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(100.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.95),
                source: "high_conf".to_string(),
                metadata: None,
            },
            AssetPrice {
                asset_id: "BTC".to_string(),
                price: dec!(101.0),
                currency: "USD".to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.90),
                source: "med_conf".to_string(),
                metadata: None,
            },
        ];

        let stats = aggregator.calculate_statistics(&prices).unwrap();
        let quality_score = aggregator.calculate_quality_score(&prices, &stats);

        // Should be high quality due to high confidence and low deviation
        assert!(quality_score > dec!(0.8));
        assert!(quality_score <= dec!(1.0));
    }
}
