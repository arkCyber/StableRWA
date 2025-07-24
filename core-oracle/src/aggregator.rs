// =====================================================================================
// File: core-oracle/src/aggregator.rs
// Description: Price data aggregation and consensus module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{OracleError, OracleResult},
    service::ProviderClient,
    types::{
        AggregationConfig, AggregationMethod, OracleProvider, PriceData, ValidationRule,
        ValidationRuleType, ValidationSeverity,
    },
};

/// Price aggregator trait
#[async_trait]
pub trait PriceAggregator: Send + Sync {
    /// Aggregate prices from multiple sources
    async fn aggregate_prices(
        &self,
        prices: Vec<PriceData>,
        method: AggregationMethod,
    ) -> OracleResult<PriceData>;

    /// Validate aggregated price
    async fn validate_price(
        &self,
        feed_id: &str,
        price_data: &PriceData,
        source_prices: &[PriceData],
    ) -> OracleResult<bool>;

    /// Calculate consensus score
    async fn calculate_consensus(&self, prices: &[PriceData]) -> OracleResult<f64>;

    /// Detect and filter outliers
    async fn filter_outliers(
        &self,
        prices: Vec<PriceData>,
        threshold_percent: f64,
    ) -> OracleResult<Vec<PriceData>>;
}

/// Multi-source price aggregator
pub struct MultiSourceAggregator {
    config: AggregationConfig,
    providers: HashMap<OracleProvider, Box<dyn ProviderClient>>,
    validation_rules: Vec<ValidationRule>,
}

/// Aggregation result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub aggregated_price: PriceData,
    pub source_prices: Vec<PriceData>,
    pub method_used: AggregationMethod,
    pub consensus_score: f64,
    pub outliers_removed: u32,
    pub validation_passed: bool,
    pub aggregation_metadata: AggregationMetadata,
}

/// Aggregation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationMetadata {
    pub total_sources: u32,
    pub successful_sources: u32,
    pub failed_sources: u32,
    pub price_variance: f64,
    pub confidence_range: (f64, f64),
    pub timestamp_range: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
    pub processing_time_ms: u64,
}

/// Outlier detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetectionResult {
    pub valid_prices: Vec<PriceData>,
    pub outliers: Vec<PriceData>,
    pub detection_method: OutlierDetectionMethod,
    pub threshold_used: f64,
}

/// Outlier detection methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutlierDetectionMethod {
    StandardDeviation,
    InterquartileRange,
    ZScore,
    ModifiedZScore,
}

/// Consensus calculation methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsensusMethod {
    PriceVariance,
    ConfidenceWeighted,
    SourceAgreement,
    Combined,
}

impl MultiSourceAggregator {
    pub fn new(
        config: AggregationConfig,
        providers: HashMap<OracleProvider, Box<dyn ProviderClient>>,
    ) -> Self {
        let validation_rules = vec![
            ValidationRule {
                rule_type: ValidationRuleType::DeviationCheck,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "max_deviation_percent".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(10.0)),
                    );
                    params
                },
                enabled: true,
                severity: ValidationSeverity::Warning,
            },
            ValidationRule {
                rule_type: ValidationRuleType::ConsensusCheck,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "min_consensus_score".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(0.8)),
                    );
                    params
                },
                enabled: true,
                severity: ValidationSeverity::Error,
            },
        ];

        Self {
            config,
            providers,
            validation_rules,
        }
    }

    /// Get prices from all available providers
    pub async fn get_all_prices(&self, feed_id: &str) -> OracleResult<Vec<PriceData>> {
        let mut prices = Vec::new();
        let mut failed_providers = Vec::new();

        for (provider, client) in &self.providers {
            match client.get_price(feed_id).await {
                Ok(price_data) => {
                    prices.push(price_data);
                }
                Err(e) => {
                    failed_providers.push(*provider);
                    eprintln!("Failed to get price from {:?}: {}", provider, e);
                }
            }
        }

        if prices.len() < self.config.min_sources_for_consensus as usize {
            return Err(OracleError::insufficient_data(
                feed_id,
                self.config.min_sources_for_consensus,
                prices.len() as u32,
            ));
        }

        Ok(prices)
    }

    /// Aggregate prices with full pipeline
    pub async fn aggregate_with_validation(
        &self,
        feed_id: &str,
    ) -> OracleResult<AggregationResult> {
        let start_time = std::time::Instant::now();

        // Get prices from all sources
        let source_prices = self.get_all_prices(feed_id).await?;
        let total_sources = self.providers.len() as u32;
        let successful_sources = source_prices.len() as u32;
        let failed_sources = total_sources - successful_sources;

        // Filter outliers if enabled
        let filtered_prices = if self.config.outlier_detection {
            let outlier_result = self
                .filter_outliers(source_prices.clone(), self.config.outlier_threshold_percent)
                .await?;
            outlier_result
        } else {
            source_prices.clone()
        };

        let outliers_removed = (source_prices.len() - filtered_prices.len()) as u32;

        // Aggregate prices
        let aggregated_price = self
            .aggregate_prices(filtered_prices.clone(), self.config.default_method)
            .await?;

        // Calculate consensus score
        let consensus_score = self.calculate_consensus(&filtered_prices).await?;

        // Validate aggregated price
        let validation_passed = self
            .validate_price(feed_id, &aggregated_price, &filtered_prices)
            .await
            .unwrap_or(false);

        // Calculate metadata
        let price_variance = self.calculate_price_variance(&filtered_prices);
        let confidence_range = self.calculate_confidence_range(&filtered_prices);
        let timestamp_range = self.calculate_timestamp_range(&filtered_prices);
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(AggregationResult {
            aggregated_price,
            source_prices,
            method_used: self.config.default_method,
            consensus_score,
            outliers_removed,
            validation_passed,
            aggregation_metadata: AggregationMetadata {
                total_sources,
                successful_sources,
                failed_sources,
                price_variance,
                confidence_range,
                timestamp_range,
                processing_time_ms,
            },
        })
    }

    /// Calculate price variance
    fn calculate_price_variance(&self, prices: &[PriceData]) -> f64 {
        if prices.len() < 2 {
            return 0.0;
        }

        let mean = prices
            .iter()
            .map(|p| p.price.to_f64().unwrap_or(0.0))
            .sum::<f64>()
            / prices.len() as f64;

        let variance = prices
            .iter()
            .map(|p| {
                let price = p.price.to_f64().unwrap_or(0.0);
                (price - mean).powi(2)
            })
            .sum::<f64>()
            / prices.len() as f64;

        variance.sqrt() / mean * 100.0 // Return as percentage
    }

    /// Calculate confidence range
    fn calculate_confidence_range(&self, prices: &[PriceData]) -> (f64, f64) {
        if prices.is_empty() {
            return (0.0, 0.0);
        }

        let confidences: Vec<f64> = prices.iter().map(|p| p.confidence).collect();
        let min_confidence = confidences.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_confidence = confidences.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        (min_confidence, max_confidence)
    }

    /// Calculate timestamp range
    fn calculate_timestamp_range(
        &self,
        prices: &[PriceData],
    ) -> (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) {
        if prices.is_empty() {
            let now = Utc::now();
            return (now, now);
        }

        let timestamps: Vec<chrono::DateTime<chrono::Utc>> =
            prices.iter().map(|p| p.timestamp).collect();
        let min_timestamp = timestamps.iter().min().copied().unwrap_or(Utc::now());
        let max_timestamp = timestamps.iter().max().copied().unwrap_or(Utc::now());

        (min_timestamp, max_timestamp)
    }

    /// Detect outliers using standard deviation method
    fn detect_outliers_std_dev(
        &self,
        prices: &[PriceData],
        threshold: f64,
    ) -> OutlierDetectionResult {
        if prices.len() < 3 {
            return OutlierDetectionResult {
                valid_prices: prices.to_vec(),
                outliers: Vec::new(),
                detection_method: OutlierDetectionMethod::StandardDeviation,
                threshold_used: threshold,
            };
        }

        let price_values: Vec<f64> = prices
            .iter()
            .map(|p| p.price.to_f64().unwrap_or(0.0))
            .collect();

        let mean = price_values.iter().sum::<f64>() / price_values.len() as f64;
        let variance = price_values
            .iter()
            .map(|&price| (price - mean).powi(2))
            .sum::<f64>()
            / price_values.len() as f64;
        let std_dev = variance.sqrt();

        let threshold_value = std_dev * threshold / 100.0;

        let mut valid_prices = Vec::new();
        let mut outliers = Vec::new();

        for (i, price_data) in prices.iter().enumerate() {
            let price_value = price_values[i];
            if (price_value - mean).abs() <= threshold_value {
                valid_prices.push(price_data.clone());
            } else {
                outliers.push(price_data.clone());
            }
        }

        OutlierDetectionResult {
            valid_prices,
            outliers,
            detection_method: OutlierDetectionMethod::StandardDeviation,
            threshold_used: threshold,
        }
    }

    /// Apply validation rules
    fn apply_validation_rules(
        &self,
        feed_id: &str,
        price_data: &PriceData,
        source_prices: &[PriceData],
    ) -> Vec<(ValidationRule, bool, String)> {
        let mut results = Vec::new();

        for rule in &self.validation_rules {
            if !rule.enabled {
                continue;
            }

            let (passed, message) = match rule.rule_type {
                ValidationRuleType::DeviationCheck => {
                    self.validate_deviation(price_data, source_prices, &rule.parameters)
                }
                ValidationRuleType::ConsensusCheck => {
                    self.validate_consensus(source_prices, &rule.parameters)
                }
                ValidationRuleType::FreshnessCheck => {
                    self.validate_freshness(price_data, &rule.parameters)
                }
                _ => (true, "Rule not implemented".to_string()),
            };

            results.push((rule.clone(), passed, message));
        }

        results
    }

    /// Validate price deviation
    fn validate_deviation(
        &self,
        price_data: &PriceData,
        source_prices: &[PriceData],
        parameters: &HashMap<String, serde_json::Value>,
    ) -> (bool, String) {
        let max_deviation = parameters
            .get("max_deviation_percent")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0);

        if source_prices.is_empty() {
            return (true, "No source prices to compare".to_string());
        }

        let source_price_values: Vec<f64> = source_prices
            .iter()
            .map(|p| p.price.to_f64().unwrap_or(0.0))
            .collect();

        let mean_source_price =
            source_price_values.iter().sum::<f64>() / source_price_values.len() as f64;
        let aggregated_price = price_data.price.to_f64().unwrap_or(0.0);

        if mean_source_price == 0.0 {
            return (false, "Mean source price is zero".to_string());
        }

        let deviation_percent =
            ((aggregated_price - mean_source_price).abs() / mean_source_price) * 100.0;

        if deviation_percent <= max_deviation {
            (
                true,
                format!(
                    "Deviation {:.2}% is within threshold {:.2}%",
                    deviation_percent, max_deviation
                ),
            )
        } else {
            (
                false,
                format!(
                    "Deviation {:.2}% exceeds threshold {:.2}%",
                    deviation_percent, max_deviation
                ),
            )
        }
    }

    /// Validate consensus
    fn validate_consensus(
        &self,
        source_prices: &[PriceData],
        parameters: &HashMap<String, serde_json::Value>,
    ) -> (bool, String) {
        let min_consensus = parameters
            .get("min_consensus_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);

        if source_prices.len() < 2 {
            return (true, "Insufficient sources for consensus check".to_string());
        }

        // Simple consensus calculation based on price agreement
        let price_values: Vec<f64> = source_prices
            .iter()
            .map(|p| p.price.to_f64().unwrap_or(0.0))
            .collect();

        let mean_price = price_values.iter().sum::<f64>() / price_values.len() as f64;
        let agreement_threshold = mean_price * 0.05; // 5% agreement threshold

        let agreeing_sources = price_values
            .iter()
            .filter(|&&price| (price - mean_price).abs() <= agreement_threshold)
            .count();

        let consensus_score = agreeing_sources as f64 / price_values.len() as f64;

        if consensus_score >= min_consensus {
            (
                true,
                format!(
                    "Consensus score {:.2} meets threshold {:.2}",
                    consensus_score, min_consensus
                ),
            )
        } else {
            (
                false,
                format!(
                    "Consensus score {:.2} below threshold {:.2}",
                    consensus_score, min_consensus
                ),
            )
        }
    }

    /// Validate data freshness
    fn validate_freshness(
        &self,
        price_data: &PriceData,
        parameters: &HashMap<String, serde_json::Value>,
    ) -> (bool, String) {
        let max_age_seconds = parameters
            .get("max_age_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(300); // 5 minutes default

        let age_seconds = (Utc::now() - price_data.timestamp).num_seconds() as u64;

        if age_seconds <= max_age_seconds {
            (
                true,
                format!(
                    "Data age {}s is within threshold {}s",
                    age_seconds, max_age_seconds
                ),
            )
        } else {
            (
                false,
                format!(
                    "Data age {}s exceeds threshold {}s",
                    age_seconds, max_age_seconds
                ),
            )
        }
    }
}

#[async_trait]
impl PriceAggregator for MultiSourceAggregator {
    async fn aggregate_prices(
        &self,
        prices: Vec<PriceData>,
        method: AggregationMethod,
    ) -> OracleResult<PriceData> {
        if prices.is_empty() {
            return Err(OracleError::insufficient_data("aggregation", 1, 0));
        }

        let aggregated_price = match method {
            AggregationMethod::Mean => {
                let sum: Decimal = prices.iter().map(|p| p.price).sum();
                sum / Decimal::new(prices.len() as i64, 0)
            }
            AggregationMethod::Median => {
                let mut sorted_prices: Vec<Decimal> = prices.iter().map(|p| p.price).collect();
                sorted_prices.sort();
                let mid = sorted_prices.len() / 2;
                if sorted_prices.len() % 2 == 0 {
                    (sorted_prices[mid - 1] + sorted_prices[mid]) / Decimal::new(2, 0)
                } else {
                    sorted_prices[mid]
                }
            }
            AggregationMethod::WeightedMean => {
                let total_weight: f64 = prices.iter().map(|p| p.confidence).sum();
                if total_weight == 0.0 {
                    return Err(OracleError::aggregation_error("Total weight is zero"));
                }
                let weighted_sum: Decimal = prices
                    .iter()
                    .map(|p| {
                        p.price * Decimal::from_f64_retain(p.confidence).unwrap_or(Decimal::ONE)
                    })
                    .sum();
                weighted_sum / Decimal::from_f64_retain(total_weight).unwrap_or(Decimal::ONE)
            }
            _ => {
                // Default to median for other methods
                let mut sorted_prices: Vec<Decimal> = prices.iter().map(|p| p.price).collect();
                sorted_prices.sort();
                let mid = sorted_prices.len() / 2;
                sorted_prices[mid]
            }
        };

        let avg_confidence = prices.iter().map(|p| p.confidence).sum::<f64>() / prices.len() as f64;
        let latest_timestamp = prices
            .iter()
            .map(|p| p.timestamp)
            .max()
            .unwrap_or(Utc::now());

        Ok(PriceData {
            price: aggregated_price,
            timestamp: latest_timestamp,
            source: OracleProvider::Custom(0), // Aggregated source
            confidence: avg_confidence,
            volume: None,
            market_cap: None,
            deviation: None,
            round_id: None,
        })
    }

    async fn validate_price(
        &self,
        feed_id: &str,
        price_data: &PriceData,
        source_prices: &[PriceData],
    ) -> OracleResult<bool> {
        let validation_results = self.apply_validation_rules(feed_id, price_data, source_prices);

        // Check if any critical validations failed
        for (rule, passed, message) in validation_results {
            if !passed && rule.severity >= ValidationSeverity::Error {
                return Err(OracleError::validation_error(
                    format!("{:?}", rule.rule_type),
                    message,
                ));
            }
        }

        Ok(true)
    }

    async fn calculate_consensus(&self, prices: &[PriceData]) -> OracleResult<f64> {
        if prices.len() < 2 {
            return Ok(1.0); // Perfect consensus with single source
        }

        let price_values: Vec<f64> = prices
            .iter()
            .map(|p| p.price.to_f64().unwrap_or(0.0))
            .collect();

        let mean_price = price_values.iter().sum::<f64>() / price_values.len() as f64;
        let variance = price_values
            .iter()
            .map(|&price| (price - mean_price).powi(2))
            .sum::<f64>()
            / price_values.len() as f64;

        let coefficient_of_variation = if mean_price != 0.0 {
            variance.sqrt() / mean_price
        } else {
            1.0
        };

        // Convert coefficient of variation to consensus score (lower variation = higher consensus)
        let consensus_score = (1.0 - coefficient_of_variation.min(1.0)).max(0.0);

        Ok(consensus_score)
    }

    async fn filter_outliers(
        &self,
        prices: Vec<PriceData>,
        threshold_percent: f64,
    ) -> OracleResult<Vec<PriceData>> {
        let outlier_result = self.detect_outliers_std_dev(&prices, threshold_percent);
        Ok(outlier_result.valid_prices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_price_aggregation_mean() {
        let config = AggregationConfig {
            default_method: AggregationMethod::Mean,
            outlier_detection: false,
            outlier_threshold_percent: 10.0,
            min_sources_for_consensus: 2,
            confidence_threshold: 0.8,
        };

        let aggregator = MultiSourceAggregator::new(config, HashMap::new());

        let prices = vec![
            PriceData {
                price: Decimal::new(100, 0),
                timestamp: Utc::now(),
                source: OracleProvider::Chainlink,
                confidence: 0.9,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
            PriceData {
                price: Decimal::new(102, 0),
                timestamp: Utc::now(),
                source: OracleProvider::BandProtocol,
                confidence: 0.8,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
        ];

        let result = aggregator
            .aggregate_prices(prices, AggregationMethod::Mean)
            .await
            .unwrap();
        assert_eq!(result.price, Decimal::new(101, 0));
        assert_eq!(result.confidence, 0.85);
    }

    #[tokio::test]
    async fn test_consensus_calculation() {
        let config = AggregationConfig {
            default_method: AggregationMethod::Median,
            outlier_detection: false,
            outlier_threshold_percent: 10.0,
            min_sources_for_consensus: 2,
            confidence_threshold: 0.8,
        };

        let aggregator = MultiSourceAggregator::new(config, HashMap::new());

        let prices = vec![
            PriceData {
                price: Decimal::new(100, 0),
                timestamp: Utc::now(),
                source: OracleProvider::Chainlink,
                confidence: 0.9,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
            PriceData {
                price: Decimal::new(101, 0),
                timestamp: Utc::now(),
                source: OracleProvider::BandProtocol,
                confidence: 0.8,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
        ];

        let consensus = aggregator.calculate_consensus(&prices).await.unwrap();
        assert!(consensus > 0.9); // Should have high consensus for similar prices
    }

    #[test]
    fn test_outlier_detection() {
        let config = AggregationConfig {
            default_method: AggregationMethod::Median,
            outlier_detection: true,
            outlier_threshold_percent: 10.0,
            min_sources_for_consensus: 2,
            confidence_threshold: 0.8,
        };

        let aggregator = MultiSourceAggregator::new(config, HashMap::new());

        let prices = vec![
            PriceData {
                price: Decimal::new(100, 0),
                timestamp: Utc::now(),
                source: OracleProvider::Chainlink,
                confidence: 0.9,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
            PriceData {
                price: Decimal::new(101, 0),
                timestamp: Utc::now(),
                source: OracleProvider::BandProtocol,
                confidence: 0.8,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
            PriceData {
                price: Decimal::new(200, 0), // Outlier
                timestamp: Utc::now(),
                source: OracleProvider::PythNetwork,
                confidence: 0.7,
                volume: None,
                market_cap: None,
                deviation: None,
                round_id: None,
            },
        ];

        let result = aggregator.detect_outliers_std_dev(&prices, 50.0);
        assert_eq!(result.valid_prices.len(), 2);
        assert_eq!(result.outliers.len(), 1);
        assert_eq!(result.outliers[0].price, Decimal::new(200, 0));
    }
}
