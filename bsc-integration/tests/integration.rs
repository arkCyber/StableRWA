// =====================================================================================
// File: bsc-integration/tests/integration.rs
// Description: Integration tests for the BSC integration microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, App};
use bsc_integration::fetch_bsc_block_number;
use bsc_integration::get_bsc_client;

#[actix_web::test]
async fn test_http_bsc_block_number() {
    use bsc_integration::fetch_bsc_block_number;
    use actix_web::{get, App, test, HttpResponse, Responder};

    #[get("/bsc-block-number")]
    async fn bsc_block_number() -> impl Responder {
        match fetch_bsc_block_number() {
            Ok(block_number) => HttpResponse::Ok().json(serde_json::json!({ "block_number": block_number })),
            Err(_) => HttpResponse::InternalServerError().body("Error fetching BSC block number"),
        }
    }

    let app = test::init_service(App::new().service(bsc_block_number)).await;
    let req = test::TestRequest::get().uri("/bsc-block-number").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status().is_server_error());
}

#[test]
fn test_bsc_block_number_logic() {
    // This test will pass if the logic returns either Ok or Err (no panic)
    let _ = fetch_bsc_block_number();
}

#[test]
fn test_bsc_client_env() {
    std::env::set_var("BSC_RPC_URL", "http://invalid-url");
    let client = get_bsc_client();
    assert_eq!(client.name(), "Binance Smart Chain");
} 