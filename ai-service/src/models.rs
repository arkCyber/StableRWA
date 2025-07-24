// =====================================================================================
// File: ai-service/src/models.rs
// Description: Data models for AI service
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AI completion request model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The prompt to complete
    pub prompt: String,
    /// The model to use (optional, defaults to gpt-3.5-turbo)
    pub model: Option<String>,
    /// Maximum number of tokens to generate
    pub max_tokens: Option<u32>,
    /// Sampling temperature (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Number of completions to generate
    pub n: Option<u32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty (-2.0 to 2.0)
    pub presence_penalty: Option<f32>,
    /// Frequency penalty (-2.0 to 2.0)
    pub frequency_penalty: Option<f32>,
    /// User identifier for tracking
    pub user: Option<String>,
}

impl CompletionRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            model: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.prompt.trim().is_empty() {
            return Err("Prompt cannot be empty".to_string());
        }

        if let Some(temp) = self.temperature {
            if temp < 0.0 || temp > 2.0 {
                return Err("Temperature must be between 0.0 and 2.0".to_string());
            }
        }

        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 || max_tokens > 8192 {
                return Err("Max tokens must be between 1 and 8192".to_string());
            }
        }

        Ok(())
    }
}

/// AI completion response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique identifier for this completion
    pub id: Uuid,
    /// The generated completion text
    pub content: String,
    /// The model used for completion
    pub model: String,
    /// Token usage information
    pub usage: Option<TokenUsage>,
    /// When the completion was created
    pub created_at: DateTime<Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

impl CompletionResponse {
    pub fn new(content: String, model: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            model,
            usage: None,
            created_at: Utc::now(),
            processing_time_ms: 0,
        }
    }

    pub fn with_usage(mut self, usage: TokenUsage) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn with_processing_time(mut self, processing_time_ms: u64) -> Self {
        self.processing_time_ms = processing_time_ms;
        self
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Number of tokens in the completion
    pub completion_tokens: u32,
    /// Total number of tokens used
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// Asset valuation request model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValuationRequest {
    /// Type of asset (real_estate, commodity, artwork, etc.)
    pub asset_type: String,
    /// Asset location or description
    pub location: Option<String>,
    /// Asset area or size
    pub area: Option<f64>,
    /// Year built or created
    pub year_built: Option<u32>,
    /// Additional amenities or features
    pub amenities: Option<Vec<String>>,
    /// Current market conditions
    pub market_conditions: Option<String>,
    /// Additional context for valuation
    pub context: Option<String>,
}

/// Asset valuation response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValuationResponse {
    /// Unique identifier for this valuation
    pub id: Uuid,
    /// Estimated asset value
    pub estimated_value: f64,
    /// Currency of the valuation
    pub currency: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f32,
    /// Value range (min, max)
    pub value_range: Option<ValueRange>,
    /// Factors that influenced the valuation
    pub factors: Vec<ValuationFactor>,
    /// When the valuation was performed
    pub created_at: DateTime<Utc>,
}

/// Value range for asset valuation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueRange {
    pub min: f64,
    pub max: f64,
}

/// Factor that influenced asset valuation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationFactor {
    /// Factor name (location, market_trend, etc.)
    pub factor: String,
    /// Impact weight (0.0 to 1.0)
    pub impact: f32,
    /// Description of the factor's influence
    pub description: String,
}

/// Risk assessment request model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentRequest {
    /// Asset identifier
    pub asset_id: String,
    /// Investment amount
    pub investment_amount: f64,
    /// Investment time horizon
    pub time_horizon: String,
    /// Investor profile information
    pub investor_profile: Option<InvestorProfile>,
}

/// Investor profile for risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestorProfile {
    /// Risk tolerance (conservative, moderate, aggressive)
    pub risk_tolerance: String,
    /// Investment experience level
    pub investment_experience: String,
    /// Investor age
    pub age: Option<u32>,
}

/// Risk assessment response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentResponse {
    /// Unique identifier for this assessment
    pub id: Uuid,
    /// Overall risk score (0.0 to 10.0)
    pub risk_score: f32,
    /// Risk level (low, medium, high)
    pub risk_level: String,
    /// Individual risk factors
    pub risk_factors: Vec<RiskFactor>,
    /// Risk mitigation recommendations
    pub recommendations: Vec<String>,
    /// When the assessment was performed
    pub created_at: DateTime<Utc>,
}

/// Individual risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Risk factor type
    pub factor_type: String,
    /// Risk score for this factor (0.0 to 10.0)
    pub score: f32,
    /// Description of the risk
    pub description: String,
}

/// Market analysis request model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysisRequest {
    /// Market type to analyze
    pub market_type: String,
    /// Geographic region
    pub region: Option<String>,
    /// Analysis time horizon
    pub time_horizon: Option<String>,
    /// Depth of analysis (basic, comprehensive)
    pub analysis_depth: Option<String>,
}

/// Market analysis response model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketAnalysisResponse {
    /// Unique identifier for this analysis
    pub id: Uuid,
    /// Market trend (bullish, bearish, neutral)
    pub trend: String,
    /// Trend confidence (0.0 to 1.0)
    pub confidence: f32,
    /// Key market insights
    pub insights: Vec<String>,
    /// Market indicators
    pub indicators: Vec<MarketIndicator>,
    /// When the analysis was performed
    pub created_at: DateTime<Utc>,
}

/// Market indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIndicator {
    /// Indicator name
    pub name: String,
    /// Current value
    pub value: f64,
    /// Change from previous period
    pub change: f64,
    /// Change percentage
    pub change_percent: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request_validation() {
        let valid_request = CompletionRequest::new("Test prompt".to_string());
        assert!(valid_request.validate().is_ok());

        let empty_request = CompletionRequest::new("".to_string());
        assert!(empty_request.validate().is_err());

        let invalid_temp_request = CompletionRequest::new("Test".to_string()).with_temperature(3.0);
        assert!(invalid_temp_request.validate().is_err());
    }

    #[test]
    fn test_completion_response_creation() {
        let response =
            CompletionResponse::new("Test completion".to_string(), "gpt-3.5-turbo".to_string());

        assert_eq!(response.content, "Test completion");
        assert_eq!(response.model, "gpt-3.5-turbo");
        assert!(response.usage.is_none());
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(10, 20);
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_completion_request_builder() {
        let request = CompletionRequest::new("Test prompt".to_string())
            .with_model("gpt-4".to_string())
            .with_max_tokens(100)
            .with_temperature(0.7);

        assert_eq!(request.prompt, "Test prompt");
        assert_eq!(request.model, Some("gpt-4".to_string()));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
    }
}
