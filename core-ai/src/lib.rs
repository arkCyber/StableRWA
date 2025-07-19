// =====================================================================================
// File: core-ai/src/lib.rs
// Description: AI intelligence core framework for StableRWA Platform
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

pub mod openai;
pub mod models;
pub mod plugins;
pub mod analytics;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;
use tracing::{info, error};

/// AI service errors
#[derive(Error, Debug)]
pub enum AIError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Plugin error: {0}")]
    PluginError(String),
}

impl From<reqwest::Error> for AIError {
    fn from(err: reqwest::Error) -> Self {
        AIError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for AIError {
    fn from(err: serde_json::Error) -> Self {
        AIError::ApiError(format!("JSON parsing error: {}", err))
    }
}

/// AI service result type
pub type AIResult<T> = Result<T, AIError>;

/// AI request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub id: Uuid,
    pub prompt: String,
    pub model: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: Option<String>,
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
}

impl AIRequest {
    pub fn new(prompt: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            prompt,
            model: None,
            parameters: HashMap::new(),
            context: None,
            user_id: None,
            session_id: None,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters.insert(key, value);
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// AI response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub id: Uuid,
    pub request_id: Uuid,
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processing_time_ms: u64,
}

impl AIResponse {
    pub fn new(request_id: Uuid, content: String, model: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_id,
            content,
            model,
            usage: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            processing_time_ms: 0,
        }
    }

    pub fn with_usage(mut self, usage: TokenUsage) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
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
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// AI provider trait for different AI services
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Process AI request and return response
    async fn process(&self, request: AIRequest) -> AIResult<AIResponse>;

    /// Check if the provider is healthy
    async fn health_check(&self) -> AIResult<()>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Get supported models
    fn supported_models(&self) -> Vec<String>;

    /// Validate request before processing
    fn validate_request(&self, request: &AIRequest) -> AIResult<()> {
        if request.prompt.trim().is_empty() {
            return Err(AIError::InvalidInput("Prompt cannot be empty".to_string()));
        }
        Ok(())
    }
}

/// AI service manager
pub struct AIService {
    providers: HashMap<String, Box<dyn AIProvider>>,
    default_provider: Option<String>,
}

impl AIService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    pub fn add_provider(&mut self, name: String, provider: Box<dyn AIProvider>) {
        if self.default_provider.is_none() {
            self.default_provider = Some(name.clone());
        }
        self.providers.insert(name, provider);
    }

    pub async fn process_request(&self, request: AIRequest) -> AIResult<AIResponse> {
        let provider_name = request.model.as_ref()
            .or(self.default_provider.as_ref())
            .ok_or_else(|| AIError::ConfigError("No provider specified".to_string()))?;

        let provider = self.providers.get(provider_name)
            .ok_or_else(|| AIError::ModelNotFound(format!("Provider '{}' not found", provider_name)))?;

        provider.validate_request(&request)?;
        provider.process(request).await
    }

    pub async fn health_check(&self) -> AIResult<HashMap<String, bool>> {
        let mut results = HashMap::new();

        for (name, provider) in &self.providers {
            let is_healthy = provider.health_check().await.is_ok();
            results.insert(name.clone(), is_healthy);
        }

        Ok(results)
    }
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        name: String,
    }

    #[async_trait]
    impl AIProvider for MockProvider {
        async fn process(&self, request: AIRequest) -> AIResult<AIResponse> {
            Ok(AIResponse::new(
                request.id,
                format!("Mock response for: {}", request.prompt),
                self.name.clone(),
            ))
        }

        async fn health_check(&self) -> AIResult<()> {
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn supported_models(&self) -> Vec<String> {
            vec!["mock-model".to_string()]
        }
    }

    #[tokio::test]
    async fn test_ai_service_basic_functionality() {
        let mut service = AIService::new();
        service.add_provider(
            "mock".to_string(),
            Box::new(MockProvider {
                name: "mock".to_string(),
            }),
        );

        let request = AIRequest::new("Hello, AI!".to_string());
        let response = service.process_request(request).await.unwrap();

        assert!(response.content.contains("Hello, AI!"));
        assert_eq!(response.model, "mock");
    }

    #[tokio::test]
    async fn test_ai_service_health_check() {
        let mut service = AIService::new();
        service.add_provider(
            "mock".to_string(),
            Box::new(MockProvider {
                name: "mock".to_string(),
            }),
        );

        let health = service.health_check().await.unwrap();
        assert_eq!(health.get("mock"), Some(&true));
    }

    #[test]
    fn test_ai_request_builder() {
        let request = AIRequest::new("Test prompt".to_string())
            .with_model("gpt-4".to_string())
            .with_parameter("temperature".to_string(), serde_json::json!(0.7))
            .with_context("Test context".to_string());

        assert_eq!(request.prompt, "Test prompt");
        assert_eq!(request.model, Some("gpt-4".to_string()));
        assert_eq!(request.context, Some("Test context".to_string()));
        assert!(request.parameters.contains_key("temperature"));
    }
}
