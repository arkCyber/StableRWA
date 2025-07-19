// =====================================================================================
// File: ai-service/src/lib.rs
// Description: Core AI logic for enterprise-grade AI microservice (OpenAI compatible).
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use reqwest::blocking::Client;
use std::env;

/// AI error type
#[derive(Debug, Error)]
pub enum AiError {
    #[error("API key not set")]
    ApiKeyMissing,
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// AI completion request
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// AI completion response
#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub completion: String,
}

/// AI client for OpenAI-compatible APIs
pub struct AiClient {
    pub api_key: String,
    pub api_url: String,
    pub client: Client,
}

impl AiClient {
    /// Create a new AI client from environment variables
    pub fn from_env() -> Result<Self, AiError> {
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| AiError::ApiKeyMissing)?;
        let api_url = env::var("OPENAI_API_URL").unwrap_or_else(|_| "https://api.openai.com/v1/completions".to_string());
        let client = Client::new();
        info!("{} - [AiClient] Initialized with endpoint {}", Utc::now(), api_url);
        Ok(Self { api_key, api_url, client })
    }

    /// Call the AI completion API
    pub fn complete(&self, req: &CompletionRequest) -> Result<CompletionResponse, AiError> {
        let now = Utc::now();
        let mut body = serde_json::json!({
            "prompt": req.prompt,
            "max_tokens": req.max_tokens.unwrap_or(64),
            "temperature": req.temperature.unwrap_or(0.7),
        });
        if let Some(model) = &req.model {
            body["model"] = serde_json::Value::String(model.clone());
        }
        let resp = self.client.post(&self.api_url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .map_err(|e| {
                error!("{} - [AiClient] HTTP error: {}", now, e);
                AiError::Http(e.to_string())
            })?;
        let status = resp.status();
        let json: serde_json::Value = resp.json().map_err(|e| AiError::Api(e.to_string()))?;
        if !status.is_success() {
            error!("{} - [AiClient] API error: {:?}", now, json);
            return Err(AiError::Api(format!("API error: {:?}", json)));
        }
        let completion = json["choices"][0]["text"].as_str().unwrap_or("").to_string();
        info!("{} - [AiClient] Completion success", now);
        Ok(CompletionResponse { completion })
    }

    /// Get model info (stub for demo)
    pub fn model_info(&self) -> String {
        "OpenAI GPT-compatible endpoint".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Info).try_init();
    }

    #[test]
    fn test_model_info() {
        init_logger();
        let client = AiClient {
            api_key: "test".to_string(),
            api_url: "http://localhost".to_string(),
            client: Client::new(),
        };
        assert!(client.model_info().contains("OpenAI"));
    }
} 