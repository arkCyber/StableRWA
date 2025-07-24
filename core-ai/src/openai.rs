// =====================================================================================
// File: core-ai/src/openai.rs
// Description: OpenAI integration module for core-ai library
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

use crate::{AIError, AIProvider, AIRequest, AIResponse, AIResult, TokenUsage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info, instrument};

/// OpenAI provider implementation
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String) -> AIResult<Self> {
        if api_key.is_empty() {
            return Err(AIError::AuthenticationFailed(
                "API key is empty".to_string(),
            ));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-3.5-turbo".to_string(),
        })
    }

    /// Create provider with custom configuration
    pub fn with_config(api_key: String, base_url: String, default_model: String) -> AIResult<Self> {
        let mut provider = Self::new(api_key)?;
        provider.base_url = base_url;
        provider.default_model = default_model;
        Ok(provider)
    }

    /// Validate model name
    fn validate_model(&self, model: &str) -> AIResult<()> {
        let supported_models = [
            "gpt-3.5-turbo",
            "gpt-3.5-turbo-16k",
            "gpt-4",
            "gpt-4-32k",
            "gpt-4-turbo-preview",
        ];

        if supported_models.contains(&model) {
            Ok(())
        } else {
            Err(AIError::ModelNotFound(format!(
                "Unsupported model: {}",
                model
            )))
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    #[instrument(skip(self, request))]
    async fn process(&self, request: AIRequest) -> AIResult<AIResponse> {
        let model = request.model.as_deref().unwrap_or(&self.default_model);
        self.validate_model(model)?;

        info!("Processing OpenAI request with model: {}", model);

        let chat_request = OpenAIChatRequest {
            model: model.to_string(),
            messages: vec![OpenAIChatMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            max_tokens: request
                .parameters
                .get("max_tokens")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            temperature: request
                .parameters
                .get("temperature")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            top_p: request
                .parameters
                .get("top_p")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            n: Some(1),
            stop: None,
            presence_penalty: request
                .parameters
                .get("presence_penalty")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            frequency_penalty: request
                .parameters
                .get("frequency_penalty")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            user: request.user_id.map(|id| id.to_string()),
        };

        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenAI API error: {} - {}", status, error_text);

            return match status.as_u16() {
                401 => Err(AIError::AuthenticationFailed("Invalid API key".to_string())),
                429 => Err(AIError::RateLimitExceeded),
                _ => Err(AIError::ApiError(format!(
                    "HTTP {}: {}",
                    status, error_text
                ))),
            };
        }

        let openai_response: OpenAIChatResponse = response
            .json()
            .await
            .map_err(|e| AIError::ApiError(format!("JSON parsing error: {}", e)))?;

        let content = openai_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| AIError::ApiError("No content in response".to_string()))?
            .clone();

        let usage = openai_response.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        info!("OpenAI request completed successfully");

        Ok(
            AIResponse::new(request.id, content, model.to_string()).with_usage(
                usage.unwrap_or_else(|| TokenUsage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                }),
            ),
        )
    }

    async fn health_check(&self) -> AIResult<()> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| AIError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AIError::ApiError("Health check failed".to_string()))
        }
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gpt-3.5-turbo".to_string(),
            "gpt-3.5-turbo-16k".to_string(),
            "gpt-4".to_string(),
            "gpt-4-32k".to_string(),
            "gpt-4-turbo-preview".to_string(),
        ]
    }
}

/// OpenAI chat request structure
#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIChatMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    n: Option<u32>,
    stop: Option<Vec<String>>,
    presence_penalty: Option<f32>,
    frequency_penalty: Option<f32>,
    user: Option<String>,
}

/// OpenAI chat message
#[derive(Debug, Serialize)]
struct OpenAIChatMessage {
    role: String,
    content: String,
}

/// OpenAI chat response structure
#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChatChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI chat choice
#[derive(Debug, Deserialize)]
struct OpenAIChatChoice {
    message: OpenAIChatResponseMessage,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

/// OpenAI chat response message
#[derive(Debug, Deserialize)]
struct OpenAIChatResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: Option<String>,
}

/// OpenAI usage information
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_openai_provider_creation() {
        let provider = OpenAIProvider::new("test-key".to_string());
        assert!(provider.is_ok());
    }

    #[test]
    fn test_openai_provider_empty_key() {
        let provider = OpenAIProvider::new("".to_string());
        assert!(matches!(provider, Err(AIError::AuthenticationFailed(_))));
    }

    #[test]
    fn test_model_validation() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();

        assert!(provider.validate_model("gpt-3.5-turbo").is_ok());
        assert!(provider.validate_model("gpt-4").is_ok());
        assert!(provider.validate_model("invalid-model").is_err());
    }

    #[test]
    fn test_supported_models() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        let models = provider.supported_models();

        assert!(models.contains(&"gpt-3.5-turbo".to_string()));
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(!models.is_empty());
    }

    #[test]
    fn test_provider_name() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();
        assert_eq!(provider.name(), "openai");
    }

    #[tokio::test]
    async fn test_request_validation() {
        let provider = OpenAIProvider::new("test-key".to_string()).unwrap();

        let request = AIRequest::new("Test prompt".to_string());
        let validation_result = provider.validate_request(&request);

        assert!(validation_result.is_ok());
    }
}
