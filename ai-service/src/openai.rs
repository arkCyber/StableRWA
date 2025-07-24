// =====================================================================================
// File: ai-service/src/openai.rs
// Description: OpenAI API integration for AI service
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{error, info, instrument};

/// OpenAI API errors
#[derive(Error, Debug)]
pub enum OpenAIError {
    #[error("API key not provided")]
    ApiKeyMissing,
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid model: {0}")]
    InvalidModel(String),
}

/// OpenAI client for API interactions
pub struct OpenAIClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client
    pub fn new(api_key: &str) -> Result<Self, OpenAIError> {
        if api_key.is_empty() {
            return Err(OpenAIError::ApiKeyMissing);
        }

        let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

        Ok(Self {
            client,
            api_key: api_key.to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    /// Create client with custom base URL
    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self, OpenAIError> {
        let mut client = Self::new(api_key)?;
        client.base_url = base_url.to_string();
        Ok(client)
    }

    /// Health check - verify API connectivity
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<(), OpenAIError> {
        info!("Performing OpenAI health check");

        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if response.status().is_success() {
            info!("OpenAI health check successful");
            Ok(())
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenAI health check failed: {} - {}", status, error_text);
            Err(OpenAIError::ApiError {
                status,
                message: error_text,
            })
        }
    }

    /// Complete text using OpenAI API
    #[instrument(skip(self, request))]
    pub async fn complete(
        &self,
        request: OpenAICompletionRequest,
    ) -> Result<OpenAICompletionResponse, OpenAIError> {
        info!("Sending completion request to OpenAI");

        // Validate model
        self.validate_model(&request.model)?;

        let url = format!("{}/chat/completions", self.base_url);

        // Convert to OpenAI chat format
        let chat_request = OpenAIChatRequest {
            model: request.model.clone(),
            messages: vec![OpenAIChatMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: None,
            n: Some(1),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&chat_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            error!("OpenAI API error: {} - {}", status, response_text);

            // Handle specific error cases
            if status.as_u16() == 429 {
                return Err(OpenAIError::RateLimitExceeded);
            }

            return Err(OpenAIError::ApiError {
                status: status.as_u16(),
                message: response_text,
            });
        }

        let openai_response: OpenAIChatResponse = serde_json::from_str(&response_text)?;

        // Extract the completion content
        let content = openai_response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .unwrap_or(&"No response generated".to_string())
            .clone();

        info!("OpenAI completion successful");

        Ok(OpenAICompletionResponse {
            content,
            model: request.model,
            usage: openai_response.usage.map(|u| OpenAITokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    /// Validate that the model is supported
    fn validate_model(&self, model: &str) -> Result<(), OpenAIError> {
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
            Err(OpenAIError::InvalidModel(model.to_string()))
        }
    }
}

/// OpenAI completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICompletionRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// OpenAI completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAICompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<OpenAITokenUsage>,
}

/// OpenAI token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI chat request format
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

/// OpenAI chat response format
#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChatChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI chat choice
#[derive(Debug, Deserialize)]
struct OpenAIChatChoice {
    message: OpenAIChatResponseMessage,
    finish_reason: Option<String>,
}

/// OpenAI chat response message
#[derive(Debug, Deserialize)]
struct OpenAIChatResponseMessage {
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

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new("test-api-key");
        assert!(client.is_ok());
    }

    #[test]
    fn test_openai_client_empty_key() {
        let client = OpenAIClient::new("");
        assert!(matches!(client, Err(OpenAIError::ApiKeyMissing)));
    }

    #[test]
    fn test_model_validation() {
        let client = OpenAIClient::new("test-key").unwrap();

        assert!(client.validate_model("gpt-3.5-turbo").is_ok());
        assert!(client.validate_model("gpt-4").is_ok());
        assert!(client.validate_model("invalid-model").is_err());
    }

    #[test]
    fn test_custom_base_url() {
        let client = OpenAIClient::with_base_url("test-key", "https://custom.api.com/v1");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url, "https://custom.api.com/v1");
    }

    #[tokio::test]
    async fn test_completion_request_structure() {
        let request = OpenAICompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            prompt: "Test prompt".to_string(),
            max_tokens: Some(100),
            temperature: Some(0.7),
        };

        // Test serialization
        let json = serde_json::to_string(&request);
        assert!(json.is_ok());
    }

    #[test]
    fn test_openai_error_display() {
        let error = OpenAIError::ApiKeyMissing;
        assert_eq!(error.to_string(), "API key not provided");

        let error = OpenAIError::InvalidModel("test-model".to_string());
        assert_eq!(error.to_string(), "Invalid model: test-model");
    }
}
