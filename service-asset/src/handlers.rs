// =====================================================================================
// File: service-asset/src/handlers.rs
// Description: HTTP handlers for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{models::*, service::AssetService, AppState, AssetError, AssetResult};
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Create asset request
#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub total_value: rust_decimal::Decimal,
    pub currency: String,
    pub location: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Update asset request
#[derive(Debug, Deserialize)]
pub struct UpdateAssetRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub total_value: Option<rust_decimal::Decimal>,
    pub location: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Tokenization request
#[derive(Debug, Deserialize)]
pub struct TokenizationRequest {
    pub blockchain_network: String,
    pub token_supply: u64,
    pub token_symbol: String,
    pub token_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Asset valuation request
#[derive(Debug, Deserialize)]
pub struct AssetValuationRequest {
    pub value: rust_decimal::Decimal,
    pub currency: String,
    pub valuation_method: String,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Asset response
#[derive(Debug, Serialize)]
pub struct AssetResponse {
    pub id: String,
    pub owner_id: String,
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub total_value: rust_decimal::Decimal,
    pub currency: String,
    pub location: Option<String>,
    pub is_tokenized: bool,
    pub token_address: Option<String>,
    pub blockchain_network: Option<String>,
    pub token_supply: Option<u64>,
    pub token_symbol: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Asset> for AssetResponse {
    fn from(asset: Asset) -> Self {
        Self {
            id: asset.id.to_string(),
            owner_id: asset.owner_id.to_string(),
            name: asset.name,
            description: asset.description,
            asset_type: asset.asset_type,
            total_value: asset.total_value,
            currency: asset.currency,
            location: asset.location,
            is_tokenized: asset.is_tokenized,
            token_address: asset.token_address,
            blockchain_network: asset.blockchain_network,
            token_supply: asset.token_supply.map(|s| s as u64),
            token_symbol: asset.token_symbol,
            metadata: asset.metadata,
            created_at: asset.created_at,
            updated_at: asset.updated_at,
        }
    }
}

/// Asset valuation response
#[derive(Debug, Serialize)]
pub struct AssetValuationResponse {
    pub id: String,
    pub asset_id: String,
    pub value: rust_decimal::Decimal,
    pub currency: String,
    pub valuation_method: String,
    pub valuation_date: chrono::DateTime<chrono::Utc>,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AssetValuation> for AssetValuationResponse {
    fn from(valuation: AssetValuation) -> Self {
        Self {
            id: valuation.id.to_string(),
            asset_id: valuation.asset_id.to_string(),
            value: valuation.value,
            currency: valuation.currency,
            valuation_method: valuation.valuation_method,
            valuation_date: valuation.valuation_date,
            notes: valuation.notes,
            metadata: valuation.metadata,
            created_at: valuation.created_at,
        }
    }
}

/// List assets response
#[derive(Debug, Serialize)]
pub struct ListAssetsResponse {
    pub assets: Vec<AssetResponse>,
    pub pagination: PaginationResponse,
}

/// Pagination response
#[derive(Debug, Serialize)]
pub struct PaginationResponse {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

/// Create a new asset
pub async fn create_asset(
    state: web::Data<AppState>,
    req: HttpRequest,
    asset_req: web::Json<CreateAssetRequest>,
) -> ActixResult<HttpResponse> {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Authentication required"
            })));
        }
    };

    info!("Creating asset for user: {}, name: {}", user_id, asset_req.name);

    match state.asset_service.create_asset(
        &user_id,
        &asset_req.name,
        &asset_req.description,
        &asset_req.asset_type,
        asset_req.total_value,
        &asset_req.currency,
        asset_req.location.as_deref(),
        asset_req.metadata.as_ref(),
    ).await {
        Ok(asset) => {
            info!("Asset created successfully: {}", asset.id);
            Ok(HttpResponse::Created().json(AssetResponse::from(asset)))
        }
        Err(AssetError::ValidationError(msg)) => {
            warn!("Asset creation failed - validation error: {}", msg);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })))
        }
        Err(AssetError::InvalidAssetType(asset_type)) => {
            warn!("Asset creation failed - invalid asset type: {}", asset_type);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_type",
                "message": format!("Invalid asset type: {}", asset_type)
            })))
        }
        Err(e) => {
            error!("Asset creation failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "creation_failed",
                "message": "Failed to create asset"
            })))
        }
    }
}

/// Get asset by ID
pub async fn get_asset(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let user_id = extract_user_id(&req);
    let asset_id = path.into_inner();
    let uuid = match Uuid::parse_str(&asset_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_id",
                "message": "Invalid asset ID format"
            })));
        }
    };

    match state.asset_service.get_asset(&uuid, user_id.as_ref()).await {
        Ok(asset) => {
            Ok(HttpResponse::Ok().json(AssetResponse::from(asset)))
        }
        Err(AssetError::AssetNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "asset_not_found",
                "message": "Asset not found"
            })))
        }
        Err(AssetError::PermissionDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "permission_denied",
                "message": "Access denied to this asset"
            })))
        }
        Err(e) => {
            error!("Failed to get asset: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "fetch_failed",
                "message": "Failed to fetch asset"
            })))
        }
    }
}

/// List assets
pub async fn list_assets(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<AssetListQuery>,
) -> ActixResult<HttpResponse> {
    let user_id = extract_user_id(&req);
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match state.asset_service.list_assets(
        user_id.as_ref(),
        query.asset_type.as_deref(),
        query.is_tokenized,
        page,
        per_page,
    ).await {
        Ok((assets, total)) => {
            let asset_responses: Vec<AssetResponse> = assets
                .into_iter()
                .map(AssetResponse::from)
                .collect();
            
            let total_pages = (total as f64 / per_page as f64).ceil() as i64;

            Ok(HttpResponse::Ok().json(ListAssetsResponse {
                assets: asset_responses,
                pagination: PaginationResponse {
                    page,
                    per_page,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            error!("Failed to list assets: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "list_failed",
                "message": "Failed to list assets"
            })))
        }
    }
}

/// Update asset
pub async fn update_asset(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    update_req: web::Json<UpdateAssetRequest>,
) -> ActixResult<HttpResponse> {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Authentication required"
            })));
        }
    };

    let asset_id = path.into_inner();
    let uuid = match Uuid::parse_str(&asset_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_id",
                "message": "Invalid asset ID format"
            })));
        }
    };

    match state.asset_service.update_asset(
        &uuid,
        &user_id,
        update_req.name.as_deref(),
        update_req.description.as_deref(),
        update_req.total_value,
        update_req.location.as_deref(),
        update_req.metadata.as_ref(),
    ).await {
        Ok(asset) => {
            info!("Asset updated: {}", uuid);
            Ok(HttpResponse::Ok().json(AssetResponse::from(asset)))
        }
        Err(AssetError::AssetNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "asset_not_found",
                "message": "Asset not found"
            })))
        }
        Err(AssetError::PermissionDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "permission_denied",
                "message": "Access denied to this asset"
            })))
        }
        Err(AssetError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Failed to update asset: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "update_failed",
                "message": "Failed to update asset"
            })))
        }
    }
}

/// Delete asset
pub async fn delete_asset(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Authentication required"
            })));
        }
    };

    let asset_id = path.into_inner();
    let uuid = match Uuid::parse_str(&asset_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_id",
                "message": "Invalid asset ID format"
            })));
        }
    };

    match state.asset_service.delete_asset(&uuid, &user_id).await {
        Ok(_) => {
            info!("Asset deleted: {}", uuid);
            Ok(HttpResponse::NoContent().finish())
        }
        Err(AssetError::AssetNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "asset_not_found",
                "message": "Asset not found"
            })))
        }
        Err(AssetError::PermissionDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "permission_denied",
                "message": "Access denied to this asset"
            })))
        }
        Err(AssetError::InvalidAssetState(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_state",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Failed to delete asset: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "deletion_failed",
                "message": "Failed to delete asset"
            })))
        }
    }
}

/// Tokenize asset
pub async fn tokenize_asset(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    tokenization_req: web::Json<TokenizationRequest>,
) -> ActixResult<HttpResponse> {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Authentication required"
            })));
        }
    };

    let asset_id = path.into_inner();
    let uuid = match Uuid::parse_str(&asset_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_id",
                "message": "Invalid asset ID format"
            })));
        }
    };

    match state.asset_service.tokenize_asset(
        &uuid,
        &user_id,
        &tokenization_req.blockchain_network,
        tokenization_req.token_supply,
        &tokenization_req.token_symbol,
        tokenization_req.token_name.as_deref(),
        tokenization_req.metadata.as_ref(),
    ).await {
        Ok(tokenization_result) => {
            info!("Asset tokenized: {}, token address: {:?}", uuid, tokenization_result.token_address);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "asset_id": asset_id,
                "token_address": tokenization_result.token_address,
                "blockchain_network": tokenization_result.blockchain_network,
                "token_supply": tokenization_result.token_supply,
                "token_symbol": tokenization_result.token_symbol,
                "transaction_hash": tokenization_result.transaction_hash,
                "block_number": tokenization_result.block_number
            })))
        }
        Err(AssetError::AssetNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "asset_not_found",
                "message": "Asset not found"
            })))
        }
        Err(AssetError::PermissionDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "permission_denied",
                "message": "Access denied to this asset"
            })))
        }
        Err(AssetError::AssetAlreadyTokenized) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "asset_already_tokenized",
                "message": "Asset is already tokenized"
            })))
        }
        Err(AssetError::BlockchainError(msg)) => {
            error!("Tokenization failed - blockchain error: {}", msg);
            Ok(HttpResponse::BadGateway().json(serde_json::json!({
                "error": "blockchain_error",
                "message": "Blockchain operation failed"
            })))
        }
        Err(e) => {
            error!("Failed to tokenize asset: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "tokenization_failed",
                "message": "Failed to tokenize asset"
            })))
        }
    }
}

/// Add asset valuation
pub async fn add_asset_valuation(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    valuation_req: web::Json<AssetValuationRequest>,
) -> ActixResult<HttpResponse> {
    let user_id = match extract_user_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "Authentication required"
            })));
        }
    };

    let asset_id = path.into_inner();
    let uuid = match Uuid::parse_str(&asset_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_id",
                "message": "Invalid asset ID format"
            })));
        }
    };

    match state.asset_service.add_asset_valuation(
        &uuid,
        &user_id,
        valuation_req.value,
        &valuation_req.currency,
        &valuation_req.valuation_method,
        valuation_req.notes.as_deref(),
        valuation_req.metadata.as_ref(),
    ).await {
        Ok(valuation) => {
            info!("Asset valuation added: asset {}, valuation {}", uuid, valuation.id);
            Ok(HttpResponse::Created().json(AssetValuationResponse::from(valuation)))
        }
        Err(AssetError::AssetNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "asset_not_found",
                "message": "Asset not found"
            })))
        }
        Err(AssetError::PermissionDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "permission_denied",
                "message": "Access denied to this asset"
            })))
        }
        Err(AssetError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Failed to add asset valuation: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "valuation_failed",
                "message": "Failed to add asset valuation"
            })))
        }
    }
}

/// Get asset valuations
pub async fn get_asset_valuations(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<PaginationQuery>,
) -> ActixResult<HttpResponse> {
    let user_id = extract_user_id(&req);
    let asset_id = path.into_inner();
    let uuid = match Uuid::parse_str(&asset_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_asset_id",
                "message": "Invalid asset ID format"
            })));
        }
    };

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match state.asset_service.get_asset_valuations(&uuid, user_id.as_ref(), page, per_page).await {
        Ok((valuations, total)) => {
            let valuation_responses: Vec<AssetValuationResponse> = valuations
                .into_iter()
                .map(AssetValuationResponse::from)
                .collect();
            
            let total_pages = (total as f64 / per_page as f64).ceil() as i64;

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "valuations": valuation_responses,
                "pagination": {
                    "page": page,
                    "per_page": per_page,
                    "total": total,
                    "total_pages": total_pages
                }
            })))
        }
        Err(AssetError::AssetNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "asset_not_found",
                "message": "Asset not found"
            })))
        }
        Err(e) => {
            error!("Failed to get asset valuations: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "fetch_failed",
                "message": "Failed to fetch asset valuations"
            })))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "asset-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Readiness check endpoint
pub async fn readiness_check(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    // Check database connectivity
    let db_healthy = state.database.health_check().await.is_ok();
    
    // Check blockchain connectivity
    let blockchain_healthy = state.asset_service.check_blockchain_health().await;
    
    let ready = db_healthy && blockchain_healthy;
    let status = if ready { "ready" } else { "not_ready" };
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "service": "asset-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": db_healthy,
            "blockchain": blockchain_healthy
        }
    })))
}

/// Liveness check endpoint
pub async fn liveness_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "service": "asset-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Metrics endpoint
pub async fn metrics_endpoint(state: web::Data<AppState>) -> ActixResult<String> {
    let metrics = state.metrics.export_prometheus_metrics().await;
    Ok(metrics)
}

// Helper functions

/// Extract user ID from JWT token in request
fn extract_user_id(req: &HttpRequest) -> Option<Uuid> {
    // This would typically extract user ID from validated JWT claims
    // For now, return None - this should be implemented with proper JWT validation
    req.extensions()
        .get::<Uuid>()
        .copied()
}

/// Asset list query parameters
#[derive(Debug, Deserialize)]
pub struct AssetListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub asset_type: Option<String>,
    pub is_tokenized: Option<bool>,
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
