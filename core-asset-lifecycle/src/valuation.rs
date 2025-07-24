// =====================================================================================
// File: core-asset-lifecycle/src/valuation.rs
// Description: Asset valuation service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    error::{AssetError, AssetResult},
    types::{Asset, AssetType, AssetValuation},
};

/// Asset valuation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationConfig {
    /// Default valuation providers by asset type
    pub default_providers: HashMap<AssetType, String>,
    /// Valuation provider configurations
    pub providers: HashMap<String, ValuationProviderConfig>,
    /// Automatic revaluation intervals by asset type (in days)
    pub revaluation_intervals: HashMap<AssetType, u32>,
    /// Enable market data integration
    pub market_data_integration: bool,
    /// Valuation tolerance thresholds (percentage change that triggers alerts)
    pub tolerance_thresholds: HashMap<AssetType, Decimal>,
    /// Enable multiple valuation methods for cross-validation
    pub multi_method_validation: bool,
}

impl Default for ValuationConfig {
    fn default() -> Self {
        let mut default_providers = HashMap::new();
        default_providers.insert(AssetType::RealEstate, "appraisal_institute".to_string());
        default_providers.insert(AssetType::Art, "art_appraisers_guild".to_string());
        default_providers.insert(AssetType::Commodities, "market_data_provider".to_string());

        let mut providers = HashMap::new();
        providers.insert(
            "appraisal_institute".to_string(),
            ValuationProviderConfig {
                api_url: "https://api.appraisalinstitute.com".to_string(),
                api_key: "".to_string(),
                supported_methods: vec![
                    ValuationMethod::ComparablesSales,
                    ValuationMethod::IncomeApproach,
                    ValuationMethod::CostApproach,
                ],
                supported_asset_types: vec![AssetType::RealEstate],
                certification_level: "ASA Certified".to_string(),
            },
        );

        let mut revaluation_intervals = HashMap::new();
        revaluation_intervals.insert(AssetType::RealEstate, 365); // Annual
        revaluation_intervals.insert(AssetType::Art, 730); // Biennial
        revaluation_intervals.insert(AssetType::Commodities, 30); // Monthly

        let mut tolerance_thresholds = HashMap::new();
        tolerance_thresholds.insert(AssetType::RealEstate, Decimal::new(10, 0)); // 10%
        tolerance_thresholds.insert(AssetType::Art, Decimal::new(20, 0)); // 20%
        tolerance_thresholds.insert(AssetType::Commodities, Decimal::new(5, 0)); // 5%

        Self {
            default_providers,
            providers,
            revaluation_intervals,
            market_data_integration: true,
            tolerance_thresholds,
            multi_method_validation: true,
        }
    }
}

/// Valuation provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationProviderConfig {
    pub api_url: String,
    pub api_key: String,
    pub supported_methods: Vec<ValuationMethod>,
    pub supported_asset_types: Vec<AssetType>,
    pub certification_level: String,
}

/// Valuation method enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValuationMethod {
    /// Market-based valuation using comparable sales
    ComparablesSales,
    /// Income-based valuation approach
    IncomeApproach,
    /// Cost-based valuation approach
    CostApproach,
    /// Current market price for liquid assets
    MarketPrice,
    /// Expert professional appraisal
    ExpertAppraisal,
    /// Automated valuation model
    AutomatedValuation,
    /// Discounted cash flow analysis
    DiscountedCashFlow,
    /// Net asset value calculation
    NetAssetValue,
    /// Replacement cost method
    ReplacementCost,
    /// Liquidation value assessment
    LiquidationValue,
}

impl ValuationMethod {
    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            ValuationMethod::ComparablesSales => "Comparable Sales",
            ValuationMethod::IncomeApproach => "Income Approach",
            ValuationMethod::CostApproach => "Cost Approach",
            ValuationMethod::MarketPrice => "Market Price",
            ValuationMethod::ExpertAppraisal => "Expert Appraisal",
            ValuationMethod::AutomatedValuation => "Automated Valuation",
            ValuationMethod::DiscountedCashFlow => "Discounted Cash Flow",
            ValuationMethod::NetAssetValue => "Net Asset Value",
            ValuationMethod::ReplacementCost => "Replacement Cost",
            ValuationMethod::LiquidationValue => "Liquidation Value",
        }
    }

    /// Get typical confidence level for this method
    pub fn typical_confidence_level(&self) -> f64 {
        match self {
            ValuationMethod::MarketPrice => 0.95,
            ValuationMethod::ExpertAppraisal => 0.90,
            ValuationMethod::ComparablesSales => 0.85,
            ValuationMethod::IncomeApproach => 0.80,
            ValuationMethod::CostApproach => 0.75,
            ValuationMethod::AutomatedValuation => 0.70,
            ValuationMethod::DiscountedCashFlow => 0.75,
            ValuationMethod::NetAssetValue => 0.85,
            ValuationMethod::ReplacementCost => 0.70,
            ValuationMethod::LiquidationValue => 0.65,
        }
    }
}

/// Valuation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationRequest {
    pub asset_id: Uuid,
    pub valuation_methods: Vec<ValuationMethod>,
    pub currency: String,
    pub valuation_date: Option<DateTime<Utc>>,
    pub purpose: ValuationPurpose,
    pub appraiser_id: Option<String>,
    pub rush_order: bool,
    pub special_instructions: Option<String>,
    pub market_conditions: Option<String>,
    pub assumptions: Vec<String>,
    pub limitations: Vec<String>,
}

/// Valuation purpose enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValuationPurpose {
    /// Initial tokenization valuation
    Tokenization,
    /// Periodic revaluation
    Revaluation,
    /// Transaction-related valuation
    Transaction,
    /// Insurance valuation
    Insurance,
    /// Tax assessment
    TaxAssessment,
    /// Legal proceedings
    Legal,
    /// Financial reporting
    FinancialReporting,
    /// Loan collateral assessment
    Collateral,
}

/// Valuation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationResult {
    pub valuation_id: Uuid,
    pub asset_id: Uuid,
    pub primary_valuation: AssetValuation,
    pub alternative_valuations: Vec<AssetValuation>,
    pub valuation_summary: ValuationSummary,
    pub market_analysis: Option<MarketAnalysis>,
    pub risk_assessment: RiskAssessment,
    pub quality_indicators: QualityIndicators,
    pub compliance_notes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Valuation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationSummary {
    pub recommended_value: Decimal,
    pub value_range_low: Decimal,
    pub value_range_high: Decimal,
    pub confidence_level: f64,
    pub methodology_summary: String,
    pub key_value_drivers: Vec<String>,
    pub value_detractors: Vec<String>,
}

/// Market analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysis {
    pub market_conditions: String,
    pub comparable_transactions: Vec<ComparableTransaction>,
    pub market_trends: Vec<MarketTrend>,
    pub liquidity_assessment: LiquidityAssessment,
    pub volatility_metrics: VolatilityMetrics,
}

/// Comparable transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparableTransaction {
    pub transaction_id: String,
    pub asset_description: String,
    pub transaction_price: Decimal,
    pub transaction_date: DateTime<Utc>,
    pub similarity_score: f64,
    pub adjustments: Vec<ValuationAdjustment>,
}

/// Valuation adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationAdjustment {
    pub adjustment_type: String,
    pub adjustment_amount: Decimal,
    pub adjustment_percentage: Decimal,
    pub justification: String,
}

/// Market trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTrend {
    pub trend_type: String,
    pub direction: TrendDirection,
    pub magnitude: f64,
    pub time_period: String,
    pub impact_on_value: String,
}

/// Trend direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Liquidity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityAssessment {
    pub liquidity_score: f64,
    pub expected_time_to_sell: u32, // days
    pub market_depth: String,
    pub transaction_costs: Decimal,
}

/// Volatility metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    pub historical_volatility: f64,
    pub implied_volatility: Option<f64>,
    pub value_at_risk: Decimal,
    pub confidence_interval: (Decimal, Decimal),
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_level: RiskLevel,
    pub risk_factors: Vec<RiskFactor>,
    pub mitigation_strategies: Vec<String>,
    pub risk_adjusted_value: Option<Decimal>,
}

/// Risk level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    VeryHigh = 4,
}

/// Risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub impact_level: RiskLevel,
    pub probability: f64,
    pub description: String,
    pub potential_impact: Decimal,
}

/// Quality indicators for valuation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIndicators {
    pub data_quality_score: f64,
    pub methodology_appropriateness: f64,
    pub market_evidence_strength: f64,
    pub appraiser_expertise_level: f64,
    pub overall_quality_score: f64,
    pub quality_notes: Vec<String>,
}

/// Asset valuation service
pub struct AssetValuationService {
    config: ValuationConfig,
}

impl AssetValuationService {
    /// Create new valuation service
    pub fn new(config: ValuationConfig) -> Self {
        Self { config }
    }

    /// Request asset valuation
    pub async fn request_valuation(
        &self,
        request: ValuationRequest,
    ) -> AssetResult<ValuationResult> {
        // Validate the valuation request
        self.validate_valuation_request(&request)?;

        // Get asset information
        let asset = self.get_asset(request.asset_id).await?
            .ok_or_else(|| AssetError::asset_not_found(request.asset_id.to_string()))?;

        // Determine appropriate valuation methods if not specified
        let methods = if request.valuation_methods.is_empty() {
            asset.asset_type.typical_valuation_methods()
        } else {
            request.valuation_methods.clone()
        };

        // Perform valuations using different methods
        let mut valuations = Vec::new();
        for method in &methods {
            let valuation = self.perform_valuation(&asset, *method, &request).await?;
            valuations.push(valuation);
        }

        // Select primary valuation (highest confidence)
        let primary_valuation = valuations
            .iter()
            .max_by(|a, b| a.confidence_level.partial_cmp(&b.confidence_level).unwrap())
            .cloned()
            .ok_or_else(|| AssetError::valuation_error("No valid valuations produced"))?;

        let alternative_valuations: Vec<AssetValuation> = valuations
            .into_iter()
            .filter(|v| v.id != primary_valuation.id)
            .collect();

        // Generate valuation summary
        let valuation_summary = self.generate_valuation_summary(&primary_valuation, &alternative_valuations);

        // Perform market analysis if enabled
        let market_analysis = if self.config.market_data_integration {
            Some(self.perform_market_analysis(&asset, &primary_valuation).await?)
        } else {
            None
        };

        // Assess risks
        let risk_assessment = self.assess_valuation_risks(&asset, &primary_valuation).await?;

        // Calculate quality indicators
        let quality_indicators = self.calculate_quality_indicators(&primary_valuation, &alternative_valuations);

        Ok(ValuationResult {
            valuation_id: Uuid::new_v4(),
            asset_id: request.asset_id,
            primary_valuation,
            alternative_valuations,
            valuation_summary,
            market_analysis,
            risk_assessment,
            quality_indicators,
            compliance_notes: vec!["Valuation performed in accordance with industry standards".to_string()],
            created_at: Utc::now(),
        })
    }

    /// Get valuation history for an asset
    pub async fn get_valuation_history(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<ValuationResult>> {
        // In a real implementation, this would fetch valuation history
        Ok(vec![])
    }

    /// Check if asset needs revaluation
    pub async fn needs_revaluation(&self, asset: &Asset) -> bool {
        if let Some(latest_valuation) = asset.current_valuation() {
            if let Some(interval) = self.config.revaluation_intervals.get(&asset.asset_type) {
                let days_since_valuation = (Utc::now() - latest_valuation.valuation_date).num_days();
                return days_since_valuation >= *interval as i64;
            }
        }
        true // No valuation exists, so revaluation is needed
    }

    // Private helper methods

    fn validate_valuation_request(&self, request: &ValuationRequest) -> AssetResult<()> {
        if request.currency.is_empty() {
            return Err(AssetError::validation_error(
                "currency",
                "Currency is required",
            ));
        }

        if let Some(valuation_date) = request.valuation_date {
            if valuation_date > Utc::now() {
                return Err(AssetError::validation_error(
                    "valuation_date",
                    "Valuation date cannot be in the future",
                ));
            }
        }

        Ok(())
    }

    async fn get_asset(&self, asset_id: Uuid) -> AssetResult<Option<Asset>> {
        // In a real implementation, this would fetch the asset from database
        Ok(None)
    }

    async fn perform_valuation(
        &self,
        asset: &Asset,
        method: ValuationMethod,
        request: &ValuationRequest,
    ) -> AssetResult<AssetValuation> {
        // In a real implementation, this would perform actual valuation
        // using the specified method and external data sources

        let valuation_amount = match method {
            ValuationMethod::MarketPrice => Decimal::new(100000000, 2), // $1,000,000
            ValuationMethod::ExpertAppraisal => Decimal::new(105000000, 2), // $1,050,000
            ValuationMethod::ComparablesSales => Decimal::new(98000000, 2), // $980,000
            _ => Decimal::new(100000000, 2), // Default $1,000,000
        };

        Ok(AssetValuation {
            id: Uuid::new_v4(),
            asset_id: asset.id,
            valuation_method: method,
            valuation_amount,
            currency: request.currency.clone(),
            valuation_date: request.valuation_date.unwrap_or_else(Utc::now),
            valid_until: Some(Utc::now() + chrono::Duration::days(365)),
            appraiser_id: request.appraiser_id.clone().unwrap_or_else(|| "system".to_string()),
            appraiser_credentials: "Certified Asset Appraiser".to_string(),
            confidence_level: method.typical_confidence_level(),
            methodology_notes: format!("Valuation performed using {}", method.display_name()),
            supporting_documents: vec![],
            market_conditions: request.market_conditions.clone(),
            assumptions: request.assumptions.clone(),
            limitations: request.limitations.clone(),
        })
    }

    fn generate_valuation_summary(
        &self,
        primary: &AssetValuation,
        alternatives: &[AssetValuation],
    ) -> ValuationSummary {
        let all_values: Vec<Decimal> = std::iter::once(primary.valuation_amount)
            .chain(alternatives.iter().map(|v| v.valuation_amount))
            .collect();

        let min_value = all_values.iter().min().cloned().unwrap_or(primary.valuation_amount);
        let max_value = all_values.iter().max().cloned().unwrap_or(primary.valuation_amount);

        ValuationSummary {
            recommended_value: primary.valuation_amount,
            value_range_low: min_value,
            value_range_high: max_value,
            confidence_level: primary.confidence_level,
            methodology_summary: primary.methodology_notes.clone(),
            key_value_drivers: vec!["Location".to_string(), "Condition".to_string()],
            value_detractors: vec!["Market volatility".to_string()],
        }
    }

    async fn perform_market_analysis(
        &self,
        _asset: &Asset,
        _valuation: &AssetValuation,
    ) -> AssetResult<MarketAnalysis> {
        // In a real implementation, this would analyze market data
        Ok(MarketAnalysis {
            market_conditions: "Stable market conditions".to_string(),
            comparable_transactions: vec![],
            market_trends: vec![],
            liquidity_assessment: LiquidityAssessment {
                liquidity_score: 0.75,
                expected_time_to_sell: 90,
                market_depth: "Moderate".to_string(),
                transaction_costs: Decimal::new(500, 2), // 5%
            },
            volatility_metrics: VolatilityMetrics {
                historical_volatility: 0.15,
                implied_volatility: None,
                value_at_risk: Decimal::new(5000000, 2), // $50,000
                confidence_interval: (Decimal::new(95000000, 2), Decimal::new(105000000, 2)),
            },
        })
    }

    async fn assess_valuation_risks(
        &self,
        _asset: &Asset,
        _valuation: &AssetValuation,
    ) -> AssetResult<RiskAssessment> {
        // In a real implementation, this would assess various risks
        Ok(RiskAssessment {
            overall_risk_level: RiskLevel::Medium,
            risk_factors: vec![
                RiskFactor {
                    factor_type: "Market Risk".to_string(),
                    impact_level: RiskLevel::Medium,
                    probability: 0.3,
                    description: "Market volatility may affect asset value".to_string(),
                    potential_impact: Decimal::new(10000000, 2), // $100,000
                },
            ],
            mitigation_strategies: vec!["Regular revaluation".to_string()],
            risk_adjusted_value: Some(Decimal::new(95000000, 2)), // $950,000
        })
    }

    fn calculate_quality_indicators(
        &self,
        primary: &AssetValuation,
        alternatives: &[AssetValuation],
    ) -> QualityIndicators {
        let data_quality = if alternatives.is_empty() { 0.7 } else { 0.9 };
        let methodology_score = primary.confidence_level;
        let market_evidence = if alternatives.len() >= 2 { 0.9 } else { 0.6 };
        let expertise_level = 0.85;

        let overall_quality = (data_quality + methodology_score + market_evidence + expertise_level) / 4.0;

        QualityIndicators {
            data_quality_score: data_quality,
            methodology_appropriateness: methodology_score,
            market_evidence_strength: market_evidence,
            appraiser_expertise_level: expertise_level,
            overall_quality_score: overall_quality,
            quality_notes: vec!["High-quality valuation with multiple methods".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valuation_config_default() {
        let config = ValuationConfig::default();
        assert!(!config.default_providers.is_empty());
        assert!(config.market_data_integration);
        assert!(config.multi_method_validation);
    }

    #[test]
    fn test_valuation_method_display_names() {
        assert_eq!(ValuationMethod::ComparablesSales.display_name(), "Comparable Sales");
        assert_eq!(ValuationMethod::ExpertAppraisal.display_name(), "Expert Appraisal");
        assert_eq!(ValuationMethod::MarketPrice.display_name(), "Market Price");
    }

    #[test]
    fn test_valuation_method_confidence_levels() {
        assert!(ValuationMethod::MarketPrice.typical_confidence_level() > 0.9);
        assert!(ValuationMethod::ExpertAppraisal.typical_confidence_level() > 0.8);
        assert!(ValuationMethod::AutomatedValuation.typical_confidence_level() < 0.8);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::VeryHigh);
    }

    #[test]
    fn test_valuation_service_creation() {
        let config = ValuationConfig::default();
        let _service = AssetValuationService::new(config);
    }

    #[tokio::test]
    async fn test_valuation_request_validation() {
        let config = ValuationConfig::default();
        let service = AssetValuationService::new(config);

        let request = ValuationRequest {
            asset_id: Uuid::new_v4(),
            valuation_methods: vec![ValuationMethod::ExpertAppraisal],
            currency: "USD".to_string(),
            valuation_date: Some(Utc::now() - chrono::Duration::days(1)),
            purpose: ValuationPurpose::Tokenization,
            appraiser_id: None,
            rush_order: false,
            special_instructions: None,
            market_conditions: None,
            assumptions: vec![],
            limitations: vec![],
        };

        let result = service.validate_valuation_request(&request);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_currency_validation() {
        let config = ValuationConfig::default();
        let service = AssetValuationService::new(config);

        let request = ValuationRequest {
            asset_id: Uuid::new_v4(),
            valuation_methods: vec![ValuationMethod::ExpertAppraisal],
            currency: "".to_string(), // Empty currency
            valuation_date: None,
            purpose: ValuationPurpose::Tokenization,
            appraiser_id: None,
            rush_order: false,
            special_instructions: None,
            market_conditions: None,
            assumptions: vec![],
            limitations: vec![],
        };

        let result = service.validate_valuation_request(&request);
        assert!(result.is_err());
    }
}
