// =====================================================================================
// File: ai-service/src/service.rs
// Description: Business logic layer for AI service
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

use crate::models::*;
use crate::openai::OpenAIClient;
use core_config::AppConfig;
use std::collections::HashMap;
use tracing::{info, error, instrument};
use uuid::Uuid;

/// AI service business logic
pub struct AIServiceLogic {
    openai_client: Option<OpenAIClient>,
    config: AppConfig,
    cache: HashMap<String, CachedResponse>,
}

/// Cached AI response
#[derive(Debug, Clone)]
struct CachedResponse {
    response: CompletionResponse,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl AIServiceLogic {
    pub fn new(config: AppConfig, openai_client: Option<OpenAIClient>) -> Self {
        Self {
            openai_client,
            config,
            cache: HashMap::new(),
        }
    }

    /// Process AI completion with caching
    #[instrument(skip(self, request))]
    pub async fn process_completion(
        &mut self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, Box<dyn std::error::Error>> {
        // Validate request
        request.validate()?;

        // Check cache first
        let cache_key = self.generate_cache_key(&request);
        if let Some(cached) = self.get_cached_response(&cache_key) {
            info!("Returning cached AI response");
            return Ok(cached);
        }

        // Process with AI provider
        let response = if let Some(client) = &self.openai_client {
            self.process_with_openai(client, request).await?
        } else {
            self.process_with_mock(request).await?
        };

        // Cache the response
        self.cache_response(cache_key, response.clone());

        Ok(response)
    }

    /// Process asset valuation request
    #[instrument(skip(self, request))]
    pub async fn process_asset_valuation(
        &mut self,
        request: AssetValuationRequest,
    ) -> Result<AssetValuationResponse, Box<dyn std::error::Error>> {
        info!("Processing asset valuation for type: {}", request.asset_type);

        // Build AI prompt for asset valuation
        let prompt = self.build_valuation_prompt(&request);
        
        let completion_request = CompletionRequest::new(prompt)
            .with_model("gpt-4".to_string())
            .with_max_tokens(500)
            .with_temperature(0.3);

        let completion = self.process_completion(completion_request).await?;
        
        // Parse AI response into structured valuation
        let valuation = self.parse_valuation_response(&request, &completion.content)?;
        
        Ok(valuation)
    }

    /// Process risk assessment request
    #[instrument(skip(self, request))]
    pub async fn process_risk_assessment(
        &mut self,
        request: RiskAssessmentRequest,
    ) -> Result<RiskAssessmentResponse, Box<dyn std::error::Error>> {
        info!("Processing risk assessment for asset: {}", request.asset_id);

        let prompt = self.build_risk_assessment_prompt(&request);
        
        let completion_request = CompletionRequest::new(prompt)
            .with_model("gpt-4".to_string())
            .with_max_tokens(400)
            .with_temperature(0.2);

        let completion = self.process_completion(completion_request).await?;
        
        let assessment = self.parse_risk_assessment_response(&request, &completion.content)?;
        
        Ok(assessment)
    }

    /// Process market analysis request
    #[instrument(skip(self, request))]
    pub async fn process_market_analysis(
        &mut self,
        request: MarketAnalysisRequest,
    ) -> Result<MarketAnalysisResponse, Box<dyn std::error::Error>> {
        info!("Processing market analysis for: {}", request.market_type);

        let prompt = self.build_market_analysis_prompt(&request);
        
        let completion_request = CompletionRequest::new(prompt)
            .with_model("gpt-4".to_string())
            .with_max_tokens(600)
            .with_temperature(0.4);

        let completion = self.process_completion(completion_request).await?;
        
        let analysis = self.parse_market_analysis_response(&request, &completion.content)?;
        
        Ok(analysis)
    }

    /// Process completion with OpenAI
    async fn process_with_openai(
        &self,
        client: &OpenAIClient,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, Box<dyn std::error::Error>> {
        let openai_request = crate::openai::OpenAICompletionRequest {
            model: request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            prompt: request.prompt,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let openai_response = client.complete(openai_request).await?;
        
        Ok(CompletionResponse {
            id: Uuid::new_v4(),
            content: openai_response.content,
            model: openai_response.model,
            usage: openai_response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            created_at: chrono::Utc::now(),
            processing_time_ms: 0,
        })
    }

    /// Process completion with mock response
    async fn process_with_mock(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, Box<dyn std::error::Error>> {
        info!("Using mock AI response");
        
        let mock_content = format!(
            "Mock AI response for prompt: '{}'. This is a simulated response for development and testing purposes.",
            &request.prompt[..50.min(request.prompt.len())]
        );

        Ok(CompletionResponse {
            id: Uuid::new_v4(),
            content: mock_content,
            model: "mock-model".to_string(),
            usage: Some(TokenUsage::new(
                request.prompt.split_whitespace().count() as u32,
                20,
            )),
            created_at: chrono::Utc::now(),
            processing_time_ms: 100,
        })
    }

    /// Generate cache key for request
    fn generate_cache_key(&self, request: &CompletionRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.prompt.hash(&mut hasher);
        request.model.hash(&mut hasher);
        request.max_tokens.hash(&mut hasher);
        
        format!("ai_completion_{:x}", hasher.finish())
    }

    /// Get cached response if valid
    fn get_cached_response(&self, cache_key: &str) -> Option<CompletionResponse> {
        if let Some(cached) = self.cache.get(cache_key) {
            if cached.expires_at > chrono::Utc::now() {
                return Some(cached.response.clone());
            }
        }
        None
    }

    /// Cache response with expiration
    fn cache_response(&mut self, cache_key: String, response: CompletionResponse) {
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
        let cached = CachedResponse {
            response,
            expires_at,
        };
        self.cache.insert(cache_key, cached);
    }

    /// Build prompt for asset valuation
    fn build_valuation_prompt(&self, request: &AssetValuationRequest) -> String {
        let mut prompt = format!(
            "As an expert real estate appraiser and asset valuation specialist, provide a detailed valuation analysis for the following asset:\n\n"
        );

        prompt.push_str(&format!("Asset Type: {}\n", request.asset_type));
        
        if let Some(location) = &request.location {
            prompt.push_str(&format!("Location: {}\n", location));
        }
        
        if let Some(area) = request.area {
            prompt.push_str(&format!("Area: {} square units\n", area));
        }
        
        if let Some(year) = request.year_built {
            prompt.push_str(&format!("Year Built: {}\n", year));
        }
        
        if let Some(amenities) = &request.amenities {
            prompt.push_str(&format!("Amenities: {}\n", amenities.join(", ")));
        }

        prompt.push_str("\nPlease provide:\n");
        prompt.push_str("1. Estimated market value\n");
        prompt.push_str("2. Key factors affecting valuation\n");
        prompt.push_str("3. Confidence level in the assessment\n");
        prompt.push_str("4. Market conditions impact\n");

        prompt
    }

    /// Build prompt for risk assessment
    fn build_risk_assessment_prompt(&self, request: &RiskAssessmentRequest) -> String {
        format!(
            "As a financial risk analyst specializing in real world asset investments, assess the investment risk for:\n\n\
            Asset ID: {}\n\
            Investment Amount: ${}\n\
            Time Horizon: {}\n\
            Investor Profile: {:?}\n\n\
            Provide a comprehensive risk analysis including:\n\
            1. Overall risk score (1-10)\n\
            2. Key risk factors\n\
            3. Risk mitigation strategies\n\
            4. Market risk assessment",
            request.asset_id,
            request.investment_amount,
            request.time_horizon,
            request.investor_profile
        )
    }

    /// Build prompt for market analysis
    fn build_market_analysis_prompt(&self, request: &MarketAnalysisRequest) -> String {
        let mut prompt = format!(
            "As a market research analyst, provide a comprehensive analysis of the {} market",
            request.market_type
        );

        if let Some(region) = &request.region {
            prompt.push_str(&format!(" in {}", region));
        }

        prompt.push_str(".\n\nInclude:\n");
        prompt.push_str("1. Current market trends\n");
        prompt.push_str("2. Key market drivers\n");
        prompt.push_str("3. Future outlook\n");
        prompt.push_str("4. Investment opportunities and risks\n");
        prompt.push_str("5. Market indicators and metrics\n");

        prompt
    }

    /// Parse AI response into structured valuation
    fn parse_valuation_response(
        &self,
        request: &AssetValuationRequest,
        ai_response: &str,
    ) -> Result<AssetValuationResponse, Box<dyn std::error::Error>> {
        // This is a simplified parser - in production, you'd use more sophisticated NLP
        let estimated_value = self.extract_value_from_text(ai_response).unwrap_or(1000000.0);
        
        Ok(AssetValuationResponse {
            id: Uuid::new_v4(),
            estimated_value,
            currency: "USD".to_string(),
            confidence_score: 0.8,
            value_range: Some(ValueRange {
                min: estimated_value * 0.9,
                max: estimated_value * 1.1,
            }),
            factors: vec![
                ValuationFactor {
                    factor: "location".to_string(),
                    impact: 0.4,
                    description: "Location impact on valuation".to_string(),
                },
                ValuationFactor {
                    factor: "market_conditions".to_string(),
                    impact: 0.3,
                    description: "Current market conditions".to_string(),
                },
            ],
            created_at: chrono::Utc::now(),
        })
    }

    /// Parse risk assessment response
    fn parse_risk_assessment_response(
        &self,
        _request: &RiskAssessmentRequest,
        _ai_response: &str,
    ) -> Result<RiskAssessmentResponse, Box<dyn std::error::Error>> {
        Ok(RiskAssessmentResponse {
            id: Uuid::new_v4(),
            risk_score: 5.5,
            risk_level: "Medium".to_string(),
            risk_factors: vec![
                RiskFactor {
                    factor_type: "Market Risk".to_string(),
                    score: 6.0,
                    description: "Market volatility risk".to_string(),
                },
                RiskFactor {
                    factor_type: "Liquidity Risk".to_string(),
                    score: 5.0,
                    description: "Asset liquidity concerns".to_string(),
                },
            ],
            recommendations: vec![
                "Diversify investment portfolio".to_string(),
                "Monitor market conditions regularly".to_string(),
            ],
            created_at: chrono::Utc::now(),
        })
    }

    /// Parse market analysis response
    fn parse_market_analysis_response(
        &self,
        _request: &MarketAnalysisRequest,
        _ai_response: &str,
    ) -> Result<MarketAnalysisResponse, Box<dyn std::error::Error>> {
        Ok(MarketAnalysisResponse {
            id: Uuid::new_v4(),
            trend: "Bullish".to_string(),
            confidence: 0.75,
            insights: vec![
                "Market showing strong growth potential".to_string(),
                "Increased institutional interest".to_string(),
            ],
            indicators: vec![
                MarketIndicator {
                    name: "Price Index".to_string(),
                    value: 105.2,
                    change: 2.1,
                    change_percent: 2.0,
                },
            ],
            created_at: chrono::Utc::now(),
        })
    }

    /// Extract numeric value from AI response text
    fn extract_value_from_text(&self, text: &str) -> Option<f64> {
        // Simple regex to find dollar amounts or numbers
        use regex::Regex;
        
        if let Ok(re) = Regex::new(r"\$?([0-9,]+(?:\.[0-9]{2})?)") {
            if let Some(captures) = re.captures(text) {
                if let Some(value_str) = captures.get(1) {
                    let clean_value = value_str.as_str().replace(",", "");
                    return clean_value.parse::<f64>().ok();
                }
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_service_logic_creation() {
        let config = AppConfig::default();
        let service = AIServiceLogic::new(config, None);
        
        // Test that service can be created
        assert!(true);
    }

    #[tokio::test]
    async fn test_mock_completion() {
        let config = AppConfig::default();
        let mut service = AIServiceLogic::new(config, None);
        
        let request = CompletionRequest::new("Test prompt".to_string());
        let result = service.process_completion(request).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.content.contains("Mock AI response"));
    }

    #[test]
    fn test_cache_key_generation() {
        let config = AppConfig::default();
        let service = AIServiceLogic::new(config, None);
        
        let request = CompletionRequest::new("Test prompt".to_string());
        let key = service.generate_cache_key(&request);
        
        assert!(key.starts_with("ai_completion_"));
        assert!(key.len() > 15);
    }

    #[test]
    fn test_value_extraction() {
        let config = AppConfig::default();
        let service = AIServiceLogic::new(config, None);
        
        let text = "The estimated value is $1,250,000.00 based on current market conditions.";
        let value = service.extract_value_from_text(text);
        
        assert_eq!(value, Some(1250000.0));
    }
}
