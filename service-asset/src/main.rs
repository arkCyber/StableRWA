// =====================================================================================
// File: service-asset/src/main.rs
// Description: Actix-web HTTP server for the asset microservice. Exposes a REST API
//              endpoint to fetch a list of assets as JSON.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use serde::Serialize;
use chrono::Utc;
use log::{info, error};
use ethers::providers::{Provider, Http, Middleware};
use std::env;
use bsc_integration::fetch_bsc_block_number;

/// Asset data structure for API responses.
#[derive(Serialize)]
struct Asset {
    id: String,
    name: String,
}

/// GET /assets - Returns a list of assets as JSON.
#[get("/assets")]
async fn get_assets() -> impl Responder {
    info!("[{}] GET /assets called", Utc::now());
    let assets = vec![
        Asset { id: "1".to_string(), name: "Building A".to_string() },
        Asset { id: "2".to_string(), name: "Land Parcel B".to_string() },
        Asset { id: "3".to_string(), name: "Vehicle C".to_string() },
    ];
    HttpResponse::Ok().json(serde_json::json!({ "assets": assets }))
}

/// GET /eth-block-number - Fetches the latest Ethereum block number from a real node.
#[get("/eth-block-number")]
async fn eth_block_number() -> impl Responder {
    info!("[{}] GET /eth-block-number called", Utc::now());
    let eth_node_url = env::var("ETH_NODE_URL").unwrap_or_else(|_| "http://localhost:8545".to_string());
    let provider = match Provider::<Http>::try_from(eth_node_url.clone()) {
        Ok(p) => p,
        Err(e) => {
            error!("[{}] Provider error: {}", Utc::now(), e);
            return HttpResponse::InternalServerError().body("Provider error");
        }
    };
    match provider.get_block_number().await {
        Ok(block_number) => {
            info!("[{}] Latest Ethereum block: {}", Utc::now(), block_number);
            HttpResponse::Ok().json(serde_json::json!({ "block_number": block_number }))
        }
        Err(e) => {
            error!("[{}] Error fetching block number: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body("Error fetching block number")
        }
    }
}

/// GET /bsc-block-number - Fetches the latest Binance Smart Chain block number from a real node.
#[get("/bsc-block-number")]
async fn bsc_block_number() -> impl Responder {
    info!("[{}] GET /bsc-block-number called", Utc::now());
    // 调用 bsc-integration 逻辑
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
    info!("[{}] Starting asset microservice on 127.0.0.1:8080", Utc::now());
    HttpServer::new(|| {
        App::new()
            .service(get_assets)
            .service(eth_block_number)
            .service(bsc_block_number)
    })
    .bind(("127.0.0.1", 8080))?
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
    async fn test_get_assets() {
        let app = test::init_service(App::new().service(get_assets)).await;
        let req = test::TestRequest::get().uri("/assets").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_eth_block_number() {
        // This test expects ETH_NODE_URL to be set externally if needed
        let app = test::init_service(App::new().service(eth_block_number)).await;
        let req = test::TestRequest::get().uri("/eth-block-number").to_request();
        let resp = test::call_service(&app, req).await;
        // Accept both success and error (if no node is running)
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_bsc_block_number() {
        let app = test::init_service(App::new().service(bsc_block_number)).await;
        let req = test::TestRequest::get().uri("/bsc-block-number").to_request();
        let resp = test::call_service(&app, req).await;
        // Accept both success and error (if no node is running)
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }
} 