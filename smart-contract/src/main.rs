// =====================================================================================
// File: smart-contract/src/main.rs
// Description: Actix-web HTTP server for the smart contract microservice. Exposes REST API
//              endpoints for contract deployment, function call, transaction, and event query.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, post, get, App, HttpServer, Responder, HttpResponse};
use chrono::Utc;
use log::{info, error};
use smart_contract::{ContractManager, ContractError};
use ethers::abi::{Abi, Token};
use ethers::types::{Address, Bytes, H256};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    manager: Arc<ContractManager>,
}

/// Request for deploying a contract
#[derive(Debug, Deserialize)]
struct DeployRequest {
    abi_path: String,
    bytecode_hex: String,
    constructor_args: Vec<Token>,
}

/// Request for calling a contract function (read-only)
#[derive(Debug, Deserialize)]
struct CallRequest {
    abi_path: String,
    address: String,
    function: String,
    args: Vec<Token>,
}

/// Request for sending a contract transaction (write)
#[derive(Debug, Deserialize)]
struct SendRequest {
    abi_path: String,
    address: String,
    function: String,
    args: Vec<Token>,
}

/// Request for querying contract events
#[derive(Debug, Deserialize)]
struct EventQueryRequest {
    abi_path: String,
    address: String,
    event: String,
}

/// POST /contract/deploy - Deploy a new contract
#[post("/contract/deploy")]
async fn deploy_contract(data: web::Data<AppState>, req: web::Json<DeployRequest>) -> impl Responder {
    info!("[{}] POST /contract/deploy called", Utc::now());
    let abi = match ContractManager::load_abi(&req.abi_path) {
        Ok(abi) => abi,
        Err(e) => {
            error!("[{}] Load ABI error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Load ABI error: {}", e));
        }
    };
    let bytecode = match hex::decode(&req.bytecode_hex.trim_start_matches("0x")) {
        Ok(b) => Bytes::from(b),
        Err(e) => {
            error!("[{}] Bytecode decode error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Bytecode decode error: {}", e));
        }
    };
    match data.manager.deploy_contract(abi, bytecode, req.constructor_args.clone()).await {
        Ok(addr) => HttpResponse::Ok().json(addr),
        Err(e) => {
            error!("[{}] Deploy contract error: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body(format!("Deploy contract error: {}", e))
        }
    }
}

/// POST /contract/call - Call a contract function (read-only)
#[post("/contract/call")]
async fn call_function(data: web::Data<AppState>, req: web::Json<CallRequest>) -> impl Responder {
    info!("[{}] POST /contract/call called", Utc::now());
    let abi = match ContractManager::load_abi(&req.abi_path) {
        Ok(abi) => abi,
        Err(e) => {
            error!("[{}] Load ABI error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Load ABI error: {}", e));
        }
    };
    let address = match req.address.parse::<Address>() {
        Ok(a) => a,
        Err(e) => {
            error!("[{}] Address parse error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Address parse error: {}", e));
        }
    };
    match data.manager.call_function(abi, address, &req.function, req.args.clone()).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            error!("[{}] Call function error: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body(format!("Call function error: {}", e))
        }
    }
}

/// POST /contract/send - Send a contract transaction (write)
#[post("/contract/send")]
async fn send_transaction(data: web::Data<AppState>, req: web::Json<SendRequest>) -> impl Responder {
    info!("[{}] POST /contract/send called", Utc::now());
    let abi = match ContractManager::load_abi(&req.abi_path) {
        Ok(abi) => abi,
        Err(e) => {
            error!("[{}] Load ABI error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Load ABI error: {}", e));
        }
    };
    let address = match req.address.parse::<Address>() {
        Ok(a) => a,
        Err(e) => {
            error!("[{}] Address parse error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Address parse error: {}", e));
        }
    };
    match data.manager.send_transaction(abi, address, &req.function, req.args.clone()).await {
        Ok(tx_hash) => HttpResponse::Ok().json(tx_hash),
        Err(e) => {
            error!("[{}] Send transaction error: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body(format!("Send transaction error: {}", e))
        }
    }
}

/// POST /contract/event - Query contract events
#[post("/contract/event")]
async fn query_event(data: web::Data<AppState>, req: web::Json<EventQueryRequest>) -> impl Responder {
    info!("[{}] POST /contract/event called", Utc::now());
    let abi = match ContractManager::load_abi(&req.abi_path) {
        Ok(abi) => abi,
        Err(e) => {
            error!("[{}] Load ABI error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Load ABI error: {}", e));
        }
    };
    let address = match req.address.parse::<Address>() {
        Ok(a) => a,
        Err(e) => {
            error!("[{}] Address parse error: {}", Utc::now(), e);
            return HttpResponse::BadRequest().body(format!("Address parse error: {}", e));
        }
    };
    match data.manager.query_events(abi, address, &req.event).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            error!("[{}] Query event error: {}", Utc::now(), e);
            HttpResponse::InternalServerError().body(format!("Query event error: {}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();
    info!("[{}] Starting smart contract microservice on 0.0.0.0:8096", Utc::now());
    let manager = Arc::new(ContractManager::from_env().expect("Failed to init ContractManager"));
    let state = AppState { manager };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(deploy_contract)
            .service(call_function)
            .service(send_transaction)
            .service(query_event)
    })
    .bind(("0.0.0.0", 8096))?
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
    async fn test_deploy_fail() {
        let manager = Arc::new(ContractManager::from_env().unwrap());
        let state = AppState { manager };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(deploy_contract)
        ).await;
        // Invalid ABI path and bytecode
        let req = test::TestRequest::post().uri("/contract/deploy").set_json(&DeployRequest {
            abi_path: "nonexistent.json".to_string(),
            bytecode_hex: "deadbeef".to_string(),
            constructor_args: vec![],
        }).to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }
} 