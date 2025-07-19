// =====================================================================================
// File: nft-minting/tests/integration.rs
// Description: Integration tests for the NFT minting microservice HTTP API.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, App};
use nft_minting::{NftStore, Nft};
use serde_json::json;
use nft_minting::NftError;
use actix_web::{web, get, post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct MintNftRequest {
    owner: String,
    metadata_uri: String,
}

#[derive(Debug, Serialize)]
struct MintNftResponse {
    nft: Nft,
}

#[actix_web::test]
async fn test_mint_and_query_nft() {
    use nft_minting::{NftStore};
    use actix_web::{App, web, get, post, HttpResponse, Responder};
    use chrono::Utc;

    #[post("/mint-nft")]
    async fn mint_nft(data: web::Data<NftStore>, req: web::Json<MintNftRequest>) -> impl Responder {
        match data.mint_nft(req.owner.clone(), req.metadata_uri.clone()) {
            Ok(nft) => HttpResponse::Ok().json(MintNftResponse { nft }),
            Err(e) => HttpResponse::BadRequest().body(format!("Mint NFT error: {}", e)),
        }
    }

    #[get("/nft/{id}")]
    async fn get_nft(data: web::Data<NftStore>, path: web::Path<u64>) -> impl Responder {
        match data.get_nft(path.into_inner()) {
            Ok(nft) => HttpResponse::Ok().json(nft),
            Err(e) => HttpResponse::NotFound().body(format!("Get NFT error: {}", e)),
        }
    }

    #[get("/nft/owner/{address}")]
    async fn get_nfts_by_owner(data: web::Data<NftStore>, path: web::Path<String>) -> impl Responder {
        let nfts = data.get_nfts_by_owner(&path.into_inner());
        HttpResponse::Ok().json(nfts)
    }

    let store = NftStore::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
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