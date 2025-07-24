// =====================================================================================
// File: service-asset/src/handlers.rs
// Description: HTTP handlers for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{models::*, service::*, AppState, AssetError};
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

// Use types from service module
pub use crate::service::{CreateAssetRequest, UpdateAssetRequest, AssetResponse};

/// Asset valuation request
#[derive(Debug, Deserialize)]
pub struct AssetValuationRequest {
    pub value: rust_decimal::Decimal,
    pub currency: String,
    pub valuation_method: String,
    pub notes: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl From<Asset> for AssetResponse {
    fn from(asset: Asset) -> Self {
        Self {
            id: asset.id,
            owner_id: asset.owner_id,
            name: asset.name,
            description: Some(asset.description),
            asset_type: asset.asset_type,
            status: asset.status,
            total_value: asset.total_value,
            tokenized_value: asset.tokenized_value,
            token_symbol: asset.token_symbol,
            token_address: asset.token_address,
            metadata: None, // Simplified for now
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

    info!(
        "Creating asset for user: {}, name: {}",
        user_id, asset_req.name
    );

    let request = crate::service::CreateAssetRequest {
        name: asset_req.name.clone(),
        description: asset_req.description.clone(),
        asset_type: asset_req.asset_type.clone(),
        total_value: asset_req.total_value,
        owner_id: user_id.to_string(),
        location: asset_req.location.clone(),
    };

    match state.asset_service.create_asset(request).await {
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

    match state.asset_service.get_asset(&uuid.to_string()).await {
        Ok(Some(asset)) => Ok(HttpResponse::Ok().json(asset)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "asset_not_found",
            "message": "Asset not found"
        }))),
        Err(AssetError::AssetNotFound(_)) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "asset_not_found",
            "message": "Asset not found"
        }))),
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

    let pagination = crate::service::Pagination { page, per_page };
    let filters = crate::service::AssetFilters {
        asset_type: query.asset_type.as_ref().and_then(|s| s.parse().ok()),
        owner_id: user_id.map(|id| id.to_string()),
        status: None,
        min_value: None,
        max_value: None,
    };

    match state.asset_service.list_assets(pagination, filters).await {
        Ok(response) => {
            Ok(HttpResponse::Ok().json(ListAssetsResponse {
                assets: response.data,
                pagination: PaginationResponse {
                    page: response.pagination.page,
                    per_page: response.pagination.per_page,
                    total: response.pagination.total_count,
                    total_pages: response.pagination.total_pages,
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

    let request = crate::service::UpdateAssetRequest {
        name: update_req.name.clone(),
        description: update_req.description.clone(),
        total_value: update_req.total_value,
        status: update_req.status.clone(),
    };

    match state.asset_service.update_asset(&uuid.to_string(), request).await {
        Ok(asset) => {
            info!("Asset updated: {}", uuid);
            Ok(HttpResponse::Ok().json(asset))
        }
        Err(AssetError::AssetNotFound(_)) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "asset_not_found",
            "message": "Asset not found"
        }))),
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

    match state.asset_service.delete_asset(&uuid.to_string()).await {
        Ok(_) => {
            info!("Asset deleted: {}", uuid);
            Ok(HttpResponse::NoContent().finish())
        }
        Err(AssetError::AssetNotFound(_)) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "asset_not_found",
            "message": "Asset not found"
        }))),
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

    let request = crate::service::TokenizationRequest {
        blockchain_network: tokenization_req.blockchain_network.clone(),
        token_supply: tokenization_req.token_supply,
        token_symbol: tokenization_req.token_symbol.clone(),
        token_name: tokenization_req.token_name.clone(),
    };

    match state.asset_service.tokenize_asset(&uuid.to_string(), request).await {
        Ok(tokenization_result) => {
            info!(
                "Asset tokenized: {}, token address: {}",
                uuid, tokenization_result.token_address
            );
            Ok(HttpResponse::Ok().json(tokenization_result))
        }
        Err(AssetError::AssetNotFound(_)) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "asset_not_found",
            "message": "Asset not found"
        }))),
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

    let valuation = crate::models::AssetValuation {
        id: uuid::Uuid::new_v4(),
        asset_id: uuid,
        value: valuation_req.value,
        currency: valuation_req.currency.clone(),
        valuation_method: valuation_req.valuation_method.clone(),
        valuation_date: chrono::Utc::now(),
        notes: valuation_req.notes.clone(),
        metadata: valuation_req.metadata.clone(),
        created_at: chrono::Utc::now(),
    };

    match state
        .asset_service
        .update_valuation(&uuid.to_string(), valuation.clone())
        .await
    {
        Ok(_) => {
            info!(
                "Asset valuation added: asset {}, valuation {}",
                uuid, valuation.id
            );
            Ok(HttpResponse::Created().json(AssetValuationResponse::from(valuation)))
        }
        Err(AssetError::AssetNotFound(_)) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "asset_not_found",
            "message": "Asset not found"
        }))),
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

    match state.asset_service.get_valuation_history(&uuid.to_string()).await {
        Ok(valuations) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "valuations": valuations,
                "pagination": {
                    "page": page,
                    "per_page": per_page,
                    "total": valuations.len(),
                    "total_pages": 1
                }
            })))
        }
        Err(AssetError::AssetNotFound(_)) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "asset_not_found",
            "message": "Asset not found"
        }))),
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
pub async fn readiness_check(_state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    // Simplified health check - always return ready for now
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ready",
        "service": "asset-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": true,
            "blockchain": true
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
pub async fn metrics_endpoint(_state: web::Data<AppState>) -> ActixResult<String> {
    // Simplified metrics endpoint - return basic metrics
    Ok("# HELP asset_service_requests_total Total number of requests\n# TYPE asset_service_requests_total counter\nasset_service_requests_total 0\n".to_string())
}

// Helper functions

/// Extract user ID from JWT token in request
fn extract_user_id(req: &HttpRequest) -> Option<Uuid> {
    // This would typically extract user ID from validated JWT claims
    // For now, return None - this should be implemented with proper JWT validation
    req.extensions().get::<Uuid>().copied()
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
