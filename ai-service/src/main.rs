// =====================================================================================
// File: ai-service/src/main.rs
// Description: Actix-web HTTP server for the AI microservice. Exposes REST API
//              endpoints for AI completion and model info.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, post, get, App, HttpServer, Responder, HttpResponse};
use chrono::Utc;
use log::{info, error};
use ai_service::{AiClient, CompletionRequest};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    client: Arc<AiClient>,
}

/// POST /ai/complete - AI completion endpoint
#[post("/ai/complete")]
async fn ai_complete(data: web::Data<AppState>, req: web::Json<CompletionRequest>) -> impl Responder {
    info!("[{}] POST /ai/complete called", Utc::now());
    let client = data.client.clone();
    // Use blocking for sync OpenAI API call
    let result = web::block(move || client.complete(&req)).await;
    match result {
        Ok(Ok(resp)) => HttpResponse::Ok().json(resp),
        Ok(Err(e)) => {
            error!("[{}] AI completion error: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body(format!("AI completion error: {}", e))
        }
        Err(e) => {
            error!("[{}] AI completion thread error: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body(format!("AI completion thread error: {}", e))
        }
    }
}

/// GET /ai/model - Get model info
#[get("/ai/model")]
async fn ai_model(data: web::Data<AppState>) -> impl Responder {
    info!("[{}] GET /ai/model called", Utc::now());
    let info = data.client.model_info();
    HttpResponse::Ok().json(serde_json::json!({ "model": info }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    info!("[{}] Starting AI microservice on 0.0.0.0:8090", Utc::now());
    let client = Arc::new(AiClient::from_env().expect("Failed to init AiClient"));
    let state = AppState { client };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(ai_complete)
            .service(ai_model)
    })
    .bind(("0.0.0.0", 8090))?
    .run()
    .await
}

// ======================
// Tests for the server
// ======================
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    // use ai_service::CompletionRequest;

    // #[actix_web::test]
    // async fn test_model_info() {
    //     let client = Arc::new(AiClient {
    //         api_key: "test".to_string(),
    //         api_url: "http://localhost".to_string(),
    //         client: reqwest::blocking::Client::new(),
    //     });
    //     let state = AppState { client };
    //     let app = test::init_service(
    //         App::new()
    //             .app_data(web::Data::new(state.clone()))
    //             .service(ai_model)
    //     ).await;
    //     let req = test::TestRequest::get().uri("/ai/model").to_request();
    //     let resp = test::call_service(&app, req).await;
    //     assert!(resp.status().is_success());
    // }
} 