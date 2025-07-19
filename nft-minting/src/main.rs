// =====================================================================================
// File: nft-minting/src/main.rs
// Description: Actix-web HTTP server for the NFT minting microservice. Exposes REST API
//              endpoints for minting and querying NFTs.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, get, post, App, HttpServer, Responder, HttpResponse};
use chrono::Utc;
use log::{info, error};
use nft_minting::{NftStore, Nft, NftError};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Clone)]
struct AppState {
    store: NftStore,
}

/// Request body for minting NFT
#[derive(Debug, Deserialize)]
struct MintNftRequest {
    owner: String,
    metadata_uri: String,
}

/// Response for minting NFT
#[derive(Debug, Serialize)]
struct MintNftResponse {
    nft: Nft,
}

/// POST /mint-nft - Mint a new NFT
#[post("/mint-nft")]
async fn mint_nft(data: web::Data<AppState>, req: web::Json<MintNftRequest>) -> impl Responder {
    info!("[{}] POST /mint-nft called", Utc::now());
    match data.store.mint_nft(req.owner.clone(), req.metadata_uri.clone()) {
        Ok(nft) => {
            info!("[{}] Minted NFT id={} owner={}", Utc::now(), nft.id, nft.owner);
            HttpResponse::Ok().json(MintNftResponse { nft })
        }
        Err(e) => {
            error!("[{}] Mint NFT error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Mint NFT error: {}", e))
        }
    }
}

/// GET /nft/{id} - Get NFT by id
#[get("/nft/{id}")]
async fn get_nft(data: web::Data<AppState>, path: web::Path<u64>) -> impl Responder {
    let id = path.into_inner();
    info!("[{}] GET /nft/{} called", Utc::now(), id);
    match data.store.get_nft(id) {
        Ok(nft) => HttpResponse::Ok().json(nft),
        Err(e) => {
            error!("[{}] Get NFT error: {}", Utc::now(), e);
            HttpResponse::NotFound().body(format!("Get NFT error: {}", e))
        }
    }
}

/// GET /nft/owner/{address} - Get all NFTs owned by address
#[get("/nft/owner/{address}")]
async fn get_nfts_by_owner(data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let address = path.into_inner();
    info!("[{}] GET /nft/owner/{} called", Utc::now(), address);
    let nfts = data.store.get_nfts_by_owner(&address);
    HttpResponse::Ok().json(nfts)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("[{}] Starting NFT minting microservice on 0.0.0.0:8092", Utc::now());
    let state = AppState { store: NftStore::new() };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(mint_nft)
            .service(get_nft)
            .service(get_nfts_by_owner)
    })
    .bind(("0.0.0.0", 8092))?
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
    async fn test_mint_and_get_nft() {
        let state = AppState { store: NftStore::new() };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(mint_nft)
                .service(get_nft)
                .service(get_nfts_by_owner)
        ).await;
        // Mint NFT
        let req = test::TestRequest::post()
            .uri("/mint-nft")
            .set_json(&MintNftRequest {
                owner: "0xabc".to_string(),
                metadata_uri: "ipfs://meta1".to_string(),
            })
            .to_request();
        let resp: MintNftResponse = test::call_and_read_body_json(&app, req).await;
        assert_eq!(resp.nft.owner, "0xabc");
        // Get NFT by id
        let req = test::TestRequest::get().uri(&format!("/nft/{}", resp.nft.id)).to_request();
        let nft: Nft = test::call_and_read_body_json(&app, req).await;
        assert_eq!(nft.id, resp.nft.id);
        // Get NFTs by owner
        let req = test::TestRequest::get().uri("/nft/owner/0xabc").to_request();
        let nfts: Vec<Nft> = test::call_and_read_body_json(&app, req).await;
        assert!(!nfts.is_empty());
    }
} 