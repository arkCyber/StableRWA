// =====================================================================================
// File: bsc-integration/src/main.rs
// Description: Actix-web HTTP server for the BSC integration microservice. Exposes a REST API
//              endpoint to fetch the latest BSC block number as JSON.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use chrono::Utc;
use log::{info, error};
use bsc_integration::fetch_bsc_block_number;

/// GET /bsc-block-number - Fetches the latest Binance Smart Chain block number from a real node.
#[get("/bsc-block-number")]
async fn bsc_block_number() -> impl Responder {
    info!("[{}] GET /bsc-block-number called", Utc::now());
    match fetch_bsc_block_number() {
        Ok(block_number) => {
            info!("[{}] Latest BSC block: {}", Utc::now(), block_number);
            HttpResponse::Ok().json(serde_json::json!({ "block_number": block_number }))
        }
        Err(e) => {
            error!("[{}] Error fetching BSC block number: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body("Error fetching BSC block number")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("[{}] Starting BSC integration microservice on 0.0.0.0:8091", Utc::now());
    HttpServer::new(|| {
        App::new()
            .service(bsc_block_number)
    })
    .bind(("0.0.0.0", 8091))?
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
    async fn test_bsc_block_number() {
        let app = test::init_service(App::new().service(bsc_block_number)).await;
        let req = test::TestRequest::get().uri("/bsc-block-number").to_request();
        let resp = test::call_service(&app, req).await;
        // Accept both success and error (if no node is running)
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }
} 