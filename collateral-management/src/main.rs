// =====================================================================================
// File: collateral-management/src/main.rs
// Description: Actix-web HTTP server for the collateral management microservice. Exposes REST API
//              endpoints for registering, updating, releasing, and querying collaterals.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, get, post, App, HttpServer, Responder, HttpResponse};
use chrono::Utc;
use log::{info, error};
use collateral_management::{CollateralStore, Collateral, CollateralError};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    store: CollateralStore,
}

/// Request body for registering collateral
#[derive(Debug, Deserialize)]
struct RegisterCollateralRequest {
    owner: String,
    asset_type: String,
    value: f64,
}

/// Request body for updating collateral value
#[derive(Debug, Deserialize)]
struct UpdateValueRequest {
    id: u64,
    value: f64,
}

/// Request body for releasing collateral
#[derive(Debug, Deserialize)]
struct ReleaseCollateralRequest {
    id: u64,
}

/// POST /collateral/register - Register a new collateral
#[post("/collateral/register")]
async fn register_collateral(data: web::Data<AppState>, req: web::Json<RegisterCollateralRequest>) -> impl Responder {
    info!("[{}] POST /collateral/register called", Utc::now());
    match data.store.register(req.owner.clone(), req.asset_type.clone(), req.value) {
        Ok(collateral) => HttpResponse::Ok().json(collateral),
        Err(e) => {
            error!("[{}] Register collateral error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Register collateral error: {}", e))
        }
    }
}

/// POST /collateral/update - Update collateral value
#[post("/collateral/update")]
async fn update_collateral_value(data: web::Data<AppState>, req: web::Json<UpdateValueRequest>) -> impl Responder {
    info!("[{}] POST /collateral/update called", Utc::now());
    match data.store.update_value(req.id, req.value) {
        Ok(collateral) => HttpResponse::Ok().json(collateral),
        Err(e) => {
            error!("[{}] Update collateral value error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Update collateral value error: {}", e))
        }
    }
}

/// POST /collateral/release - Release a collateral
#[post("/collateral/release")]
async fn release_collateral(data: web::Data<AppState>, req: web::Json<ReleaseCollateralRequest>) -> impl Responder {
    info!("[{}] POST /collateral/release called", Utc::now());
    match data.store.release(req.id) {
        Ok(collateral) => HttpResponse::Ok().json(collateral),
        Err(e) => {
            error!("[{}] Release collateral error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Release collateral error: {}", e))
        }
    }
}

/// GET /collateral/{id} - Get collateral by id
#[get("/collateral/{id}")]
async fn get_collateral(data: web::Data<AppState>, path: web::Path<u64>) -> impl Responder {
    let id = path.into_inner();
    info!("[{}] GET /collateral/{} called", Utc::now(), id);
    match data.store.get(id) {
        Ok(collateral) => HttpResponse::Ok().json(collateral),
        Err(e) => {
            error!("[{}] Get collateral error: {}", Utc::now(), e);
            HttpResponse::NotFound().body(format!("Get collateral error: {}", e))
        }
    }
}

/// GET /collaterals/owner/{owner} - List all collaterals for an owner
#[get("/collaterals/owner/{owner}")]
async fn list_collaterals_by_owner(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let owner = path.into_inner();
    info!("[{}] GET /collaterals/owner/{} called", Utc::now(), owner);
    let list = data.store.list_by_owner(&owner);
    HttpResponse::Ok().json(list)
}

/// GET /collaterals - List all collaterals
#[get("/collaterals")]
async fn list_all_collaterals(data: web::Data<AppState>) -> impl Responder {
    info!("[{}] GET /collaterals called", Utc::now());
    let list = data.store.list_all();
    HttpResponse::Ok().json(list)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("[{}] Starting collateral management microservice on 0.0.0.0:8094", Utc::now());
    let state = AppState { store: CollateralStore::new() };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(register_collateral)
            .service(update_collateral_value)
            .service(release_collateral)
            .service(get_collateral)
            .service(list_collaterals_by_owner)
            .service(list_all_collaterals)
    })
    .bind(("0.0.0.0", 8094))?
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
    async fn test_register_update_release_query() {
        let state = AppState { store: CollateralStore::new() };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(register_collateral)
                .service(update_collateral_value)
                .service(release_collateral)
                .service(get_collateral)
                .service(list_collaterals_by_owner)
                .service(list_all_collaterals)
        ).await;
        // Register
        let req = test::TestRequest::post().uri("/collateral/register").set_json(&RegisterCollateralRequest {
            owner: "0xabc".to_string(),
            asset_type: "RealEstate".to_string(),
            value: 1000.0,
        }).to_request();
        let collateral: Collateral = test::call_and_read_body_json(&app, req).await;
        assert_eq!(collateral.owner, "0xabc");
        // Update
        let req = test::TestRequest::post().uri("/collateral/update").set_json(&UpdateValueRequest {
            id: collateral.id,
            value: 1200.0,
        }).to_request();
        let updated: Collateral = test::call_and_read_body_json(&app, req).await;
        assert_eq!(updated.value, 1200.0);
        // Release
        let req = test::TestRequest::post().uri("/collateral/release").set_json(&ReleaseCollateralRequest {
            id: collateral.id,
        }).to_request();
        let released: Collateral = test::call_and_read_body_json(&app, req).await;
        assert!(released.released);
        // Query by id
        let req = test::TestRequest::get().uri(&format!("/collateral/{}", collateral.id)).to_request();
        let fetched: Collateral = test::call_and_read_body_json(&app, req).await;
        assert_eq!(fetched.id, collateral.id);
        // List by owner
        let req = test::TestRequest::get().uri("/collaterals/owner/0xabc").to_request();
        let list: Vec<Collateral> = test::call_and_read_body_json(&app, req).await;
        assert!(!list.is_empty());
        // List all
        let req = test::TestRequest::get().uri("/collaterals").to_request();
        let all: Vec<Collateral> = test::call_and_read_body_json(&app, req).await;
        assert!(!all.is_empty());
    }
} 