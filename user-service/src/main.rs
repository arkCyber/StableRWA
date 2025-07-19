// =====================================================================================
// File: user-service/src/main.rs
// Description: Actix-web HTTP server for the user microservice. Exposes a REST API
//              endpoint to fetch a list of users as JSON.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use serde::Serialize;
use chrono::Utc;
use log::info;

/// User data structure for API responses.
#[derive(Serialize)]
struct User {
    id: String,
    name: String,
}

/// GET /users - Returns a list of users as JSON.
#[get("/users")]
async fn get_users() -> impl Responder {
    info!("[{}] GET /users called", Utc::now());
    let users = vec![
        User { id: "u1".to_string(), name: "Alice".to_string() },
        User { id: "u2".to_string(), name: "Bob".to_string() },
        User { id: "u3".to_string(), name: "Charlie".to_string() },
    ];
    HttpResponse::Ok().json(serde_json::json!({ "users": users }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("[{}] Starting user microservice on 127.0.0.1:8081", Utc::now());
    HttpServer::new(|| {
        App::new()
            .service(get_users)
    })
    .bind(("127.0.0.1", 8081))?
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
    async fn test_get_users() {
        let app = test::init_service(App::new().service(get_users)).await;
        let req = test::TestRequest::get().uri("/users").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
} 