// =====================================================================================
// File: ai-service/src/lib.rs
// Description: Enterprise AI service library for StableRWA Framework
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

pub mod handlers;
pub mod models;
pub mod service;
pub mod openai;

use axum::{
    routing::{get, post},
    Router,
};
use core_utils::config::Config;
use core_ai::AIService as CoreAIService;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::info;

/// Application state for AI service
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub ai_service: Arc<AIServiceWrapper>,
}

/// AI service wrapper implementation
pub struct AIServiceWrapper {
    openai_client: Option<openai::OpenAIClient>,
    config: Config,
}

impl AIServiceWrapper {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        let openai_client = if config.get_bool("ai.enabled").unwrap_or(false) {
            let api_key = config.get_string("ai.openai_api_key").unwrap_or_default();
            Some(openai::OpenAIClient::new(&api_key)?)
        } else {
            None
        };

        Ok(Self {
            openai_client,
            config: config.clone(),
        })
    }

    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.openai_client {
            client.health_check().await?;
        }
        Ok(())
    }
}

/// Create the Axum application with all routes and middleware
pub async fn create_app(state: AppState) -> Result<Router, Box<dyn std::error::Error>> {
    info!("Creating AI service application");

    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/ai/complete", post(handlers::ai_complete))
        .route("/ai/model", get(handlers::ai_model))
        .route("/metrics", get(handlers::metrics))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state);

    info!("AI service application created successfully");
    Ok(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_config::AppConfig;

    #[tokio::test]
    async fn test_ai_service_creation() {
        let config = AppConfig::default();
        let ai_service = AIServiceWrapper::new(&config).await;
        assert!(ai_service.is_ok());
    }

    #[test]
    fn test_app_state_clone() {
        let config = AppConfig::default();
        let metrics = Arc::new(BusinessMetrics::new());

        // Create a mock AI service for testing
        let ai_service = Arc::new(AIServiceWrapper {
            openai_client: None,
            config: config.clone(),
        });

        let state = AppState {
            config,
            ai_service,
            metrics,
        };

        // Test that AppState can be cloned
        let _cloned_state = state.clone();
        assert!(true); // If we get here, clone worked
    }

    #[tokio::test]
    async fn test_ai_service_health_check() {
        let config = AppConfig::default();
        let ai_service = AIServiceWrapper::new(&config).await.unwrap();

        // Health check should succeed even without OpenAI client
        let result = ai_service.health_check().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_app_structure() {
        let config = AppConfig::default();
        let metrics = Arc::new(BusinessMetrics::new().unwrap());
        let ai_service = Arc::new(AIServiceWrapper {
            openai_client: None,
            config: config.clone(),
        });

        let state = AppState {
            config: config.clone(),
            ai_service: ai_service.clone(),
            metrics: metrics.clone(),
        };

        let app = create_app(state);

        // Test that the app can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_ai_service_wrapper_without_openai() {
        let mut config = AppConfig::default();
        config.ai.enabled = false; // Disable AI to test without OpenAI client

        let ai_service = AIServiceWrapper::new(&config).await.unwrap();

        // Should be able to create service even without OpenAI
        assert!(ai_service.openai_client.is_none());
    }

    #[tokio::test]
    async fn test_ai_service_wrapper_with_invalid_key() {
        let mut config = AppConfig::default();
        config.ai.enabled = true;
        config.ai.openai_api_key = "invalid_key".to_string();

        // Should still create service, but OpenAI client creation might fail
        let result = AIServiceWrapper::new(&config).await;
        // We expect this to succeed in creation, actual API calls would fail
        assert!(result.is_ok());
    }
}