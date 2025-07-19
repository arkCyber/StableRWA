// =====================================================================================
// File: auth-service/src/main.rs
// Description: Actix-web HTTP server for the authentication microservice. Exposes a REST API
//              endpoint to authenticate users and return a mock token.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use log::{info, error};

/// Request body for login.
#[derive(Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

/// Response body for login.
#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    token: Option<String>,
    error: Option<String>,
}

/// POST /login - Authenticates a user and returns a mock token on success.
#[post("/login")]
async fn login(req: web::Json<LoginRequest>) -> impl Responder {
    info!("[{}] POST /login called for user: {}", Utc::now(), req.username);
    if req.username == "admin" && req.password == "password" {
        HttpResponse::Ok().json(LoginResponse {
            success: true,
            token: Some("mock-token-123".to_string()),
            error: None,
        })
    } else {
        error!("[{}] Invalid credentials for user: {}", Utc::now(), req.username);
        HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            token: None,
            error: Some("Invalid credentials".to_string()),
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("[{}] Starting auth microservice on 127.0.0.1:8083", Utc::now());
    HttpServer::new(|| {
        App::new()
            .service(login)
    })
    .bind(("127.0.0.1", 8083))?
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

    #[actix_web::test]
    async fn test_login_success() {
        let app = test::init_service(App::new().service(login)).await;
        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(&LoginRequest {
                username: "admin".to_string(),
                password: "password".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
    #[actix_web::test]
    async fn test_login_failure() {
        let app = test::init_service(App::new().service(login)).await;
        let req = test::TestRequest::post()
            .uri("/login")
            .set_json(&LoginRequest {
                username: "user".to_string(),
                password: "wrong".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }
} 