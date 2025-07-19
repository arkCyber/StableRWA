// =====================================================================================
// File: collateral-management/tests/integration.rs
// Description: Integration tests for the collateral management microservice HTTP API.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, App};
use collateral_management::{CollateralStore, Collateral};
use serde_json::json;
use actix_web::{web, get, post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct RegisterCollateralRequest {
    owner: String,
    asset_type: String,
    value: f64,
}

#[derive(Debug, Deserialize)]
struct UpdateValueRequest {
    id: u64,
    value: f64,
}

#[derive(Debug, Deserialize)]
struct ReleaseCollateralRequest {
    id: u64,
}

#[actix_web::test]
async fn test_register_update_release_query() {
    use collateral_management::CollateralStore;
    use actix_web::{App, web, get, post, HttpResponse, Responder};

    #[post("/collateral/register")]
    async fn register_collateral(data: web::Data<CollateralStore>, req: web::Json<RegisterCollateralRequest>) -> impl Responder {
        match data.register(req.owner.clone(), req.asset_type.clone(), req.value) {
            Ok(collateral) => HttpResponse::Ok().json(collateral),
            Err(e) => HttpResponse::BadRequest().body(format!("Register collateral error: {}", e)),
        }
    }

    #[post("/collateral/update")]
    async fn update_collateral_value(data: web::Data<CollateralStore>, req: web::Json<UpdateValueRequest>) -> impl Responder {
        match data.update_value(req.id, req.value) {
            Ok(collateral) => HttpResponse::Ok().json(collateral),
            Err(e) => HttpResponse::BadRequest().body(format!("Update collateral value error: {}", e)),
        }
    }

    #[post("/collateral/release")]
    async fn release_collateral(data: web::Data<CollateralStore>, req: web::Json<ReleaseCollateralRequest>) -> impl Responder {
        match data.release(req.id) {
            Ok(collateral) => HttpResponse::Ok().json(collateral),
            Err(e) => HttpResponse::BadRequest().body(format!("Release collateral error: {}", e)),
        }
    }

    #[get("/collateral/{id}")]
    async fn get_collateral(data: web::Data<CollateralStore>, path: web::Path<u64>) -> impl Responder {
        match data.get(path.into_inner()) {
            Ok(collateral) => HttpResponse::Ok().json(collateral),
            Err(e) => HttpResponse::NotFound().body(format!("Get collateral error: {}", e)),
        }
    }

    #[get("/collaterals/owner/{owner}")]
    async fn list_collaterals_by_owner(data: web::Data<CollateralStore>, path: web::Path<String>) -> impl Responder {
        let list = data.list_by_owner(&path.into_inner());
        HttpResponse::Ok().json(list)
    }

    #[get("/collaterals")]
    async fn list_all_collaterals(data: web::Data<CollateralStore>) -> impl Responder {
        let list = data.list_all();
        HttpResponse::Ok().json(list)
    }

    let store = CollateralStore::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
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