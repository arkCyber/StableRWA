// =====================================================================================
// File: smart-contract/tests/integration.rs
// Description: Integration tests for the smart contract microservice HTTP API.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, App};
use smart_contract::ContractManager;
use ethers::abi::Token;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct DeployRequest {
    abi_path: String,
    bytecode_hex: String,
    constructor_args: Vec<Token>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CallRequest {
    abi_path: String,
    address: String,
    function: String,
    args: Vec<Token>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SendRequest {
    abi_path: String,
    address: String,
    function: String,
    args: Vec<Token>,
}

#[derive(Debug, Deserialize, Serialize)]
struct EventQueryRequest {
    abi_path: String,
    address: String,
    event: String,
}

#[actix_web::test]
async fn test_deploy_call_send_event_fail() {
    use smart_contract::{ContractManager};
    use actix_web::{App, web, post, HttpResponse, Responder};
    use std::sync::Arc;

    #[post("/contract/deploy")]
    async fn deploy_contract(data: web::Data<Arc<ContractManager>>, req: web::Json<DeployRequest>) -> impl Responder {
        let abi = match ContractManager::load_abi(&req.abi_path) {
            Ok(abi) => abi,
            Err(e) => return HttpResponse::BadRequest().body(format!("Load ABI error: {}", e)),
        };
        let bytecode = match hex::decode(&req.bytecode_hex.trim_start_matches("0x")) {
            Ok(b) => ethers::types::Bytes::from(b),
            Err(e) => return HttpResponse::BadRequest().body(format!("Bytecode decode error: {}", e)),
        };
        let res = data.deploy_contract(abi, bytecode, req.constructor_args.clone()).await;
        match res {
            Ok(addr) => HttpResponse::Ok().json(addr),
            Err(e) => HttpResponse::InternalServerError().body(format!("Deploy contract error: {}", e)),
        }
    }

    let manager = Arc::new(ContractManager::from_env().unwrap());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(manager.clone()))
            .service(deploy_contract)
    ).await;

    // Deploy with invalid ABI path and bytecode
    let req = test::TestRequest::post().uri("/contract/deploy").set_json(&DeployRequest {
        abi_path: "nonexistent.json".to_string(),
        bytecode_hex: "deadbeef".to_string(),
        constructor_args: vec![],
    }).to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
} 