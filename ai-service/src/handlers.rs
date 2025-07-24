// =====================================================================================
// File: ai-service/src/handlers.rs
// Description: HTTP request handlers for AI service endpoints
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - AI-Powered Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

use crate::{models::*, AppState};
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use tracing::{error, info, instrument};
use uuid::Uuid;

/// Health check endpoint
#[instrument(skip(state))]
pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    info!("Health check requested");

    match state.ai_service.health_check().await {
        Ok(_) => {
            let response = json!({
                "status": "healthy",
                "service": "ai-service",
                "version": env!("CARGO_PKG_VERSION"),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            Ok(Json(response))
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            let response = json!({
                "status": "unhealthy",
                "service": "ai-service",
                "error": e.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}

/// AI completion endpoint
#[instrument(skip(state, request))]
pub async fn ai_complete(
    State(state): State<AppState>,
    Json(request): Json<CompletionRequest>,
) -> Result<Json<CompletionResponse>, StatusCode> {
    info!(
        "AI completion requested for prompt: {}",
        &request.prompt[..50.min(request.prompt.len())]
    );

    // Validate request
    if request.prompt.trim().is_empty() {
        error!("Empty prompt provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Record metrics
    state
        .metrics
        .increment_counter("ai_requests_total", &[("endpoint", "complete")]);
    let start_time = std::time::Instant::now();

    // Process the AI request
    match process_ai_completion(&state, request).await {
        Ok(response) => {
            let duration = start_time.elapsed();
            state.metrics.record_histogram(
                "ai_request_duration_seconds",
                duration.as_secs_f64(),
            );
            state
                .metrics
                .increment_counter("ai_requests_success_total", &[("endpoint", "complete")]);

            info!("AI completion successful in {:?}", duration);
            Ok(Json(response))
        }
        Err(e) => {
            let duration = start_time.elapsed();
            state.metrics.record_histogram(
                "ai_request_duration_seconds",
                duration.as_secs_f64(),
            );
            state
                .metrics
                .increment_counter("ai_requests_error_total", &[("endpoint", "complete")]);

            error!("AI completion failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// AI model information endpoint
#[instrument(skip(state))]
pub async fn ai_model(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    info!("AI model info requested");

    let model_info = json!({
        "service": "StableRWA AI Service",
        "version": env!("CARGO_PKG_VERSION"),
        "models": [
            {
                "name": "gpt-3.5-turbo",
                "description": "Fast and efficient model for general tasks",
                "max_tokens": 4096
            },
            {
                "name": "gpt-4",
                "description": "Most capable model for complex reasoning",
                "max_tokens": 8192
            }
        ],
        "capabilities": [
            "text_completion",
            "asset_valuation",
            "risk_assessment",
            "market_analysis"
        ],
        "status": "available"
    });

    Ok(Json(model_info))
}

/// Metrics endpoint for Prometheus
#[instrument(skip(state))]
pub async fn metrics(State(state): State<AppState>) -> Result<String, StatusCode> {
    info!("Metrics requested");

    match state.metrics.export_metrics() {
        Ok(metrics_text) => Ok(metrics_text),
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Process AI completion request
async fn process_ai_completion(
    state: &AppState,
    request: CompletionRequest,
) -> Result<CompletionResponse, Box<dyn std::error::Error>> {
    let request_id = Uuid::new_v4();
    info!("Processing AI completion request {}", request_id);

    // Check if AI service is available
    if let Some(openai_client) = &state.ai_service.openai_client {
        // Use OpenAI for completion
        let openai_request = crate::openai::OpenAICompletionRequest {
            model: request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            prompt: request.prompt.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let openai_response = openai_client.complete(openai_request).await?;

        Ok(CompletionResponse {
            id: request_id,
            content: openai_response.content,
            model: openai_response.model,
            usage: openai_response.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            created_at: chrono::Utc::now(),
            processing_time_ms: 0, // Will be calculated by caller
        })
    } else {
        // Fallback to mock response
        info!("OpenAI not available, using mock response");
        Ok(CompletionResponse {
            id: request_id,
            content: format!("Mock AI response for: {}", request.prompt),
            model: "mock-model".to_string(),
            usage: Some(TokenUsage {
                prompt_tokens: request.prompt.split_whitespace().count() as u32,
                completion_tokens: 10,
                total_tokens: request.prompt.split_whitespace().count() as u32 + 10,
            }),
            created_at: chrono::Utc::now(),
            processing_time_ms: 100,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AIServiceWrapper;
    use core_config::AppConfig;
    use core_observability::BusinessMetrics;
    use std::sync::Arc;

    fn create_test_state() -> AppState {
        let config = AppConfig::default();
        let metrics = Arc::new(BusinessMetrics::new());
        let ai_service = Arc::new(AIServiceWrapper {
            openai_client: None,
            config: config.clone(),
        });

        AppState {
            config,
            ai_service,
            metrics,
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_state();
        let result = health_check(State(state)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ai_complete_empty_prompt() {
        let state = create_test_state();
        let request = CompletionRequest {
            prompt: "".to_string(),
            model: None,
            max_tokens: None,
            temperature: None,
        };

        let result = ai_complete(State(state), Json(request)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ai_complete_valid_request() {
        let state = create_test_state();
        let request = CompletionRequest {
            prompt: "Test prompt".to_string(),
            model: Some("gpt-3.5-turbo".to_string()),
            max_tokens: Some(100),
            temperature: Some(0.7),
        };

        let result = ai_complete(State(state), Json(request)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ai_model() {
        let state = create_test_state();
        let result = ai_model(State(state)).await;
        assert!(result.is_ok());

        if let Ok(Json(model_info)) = result {
            assert!(model_info["service"]
                .as_str()
                .unwrap()
                .contains("StableRWA"));
            assert!(model_info["models"].is_array());
        }
    }
}
