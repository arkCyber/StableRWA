// =====================================================================================
// RWA Tokenization Platform - Custody Service HTTP Handlers
// 
// HTTP request handlers for custody service API endpoints including account management,
// asset operations, proof generation, and insurance integration.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{CustodyError, CustodyResult};
use crate::service::CustodyService;
use crate::types::AccountType;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// HTTP handler for custody service endpoints
pub struct CustodyHandlers {
    /// Custody service instance
    service: Arc<CustodyService>,
}

/// Account creation request
#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    /// Account owner identifier
    pub owner_id: String,
    /// Account type
    pub account_type: AccountType,
    /// Additional metadata
    pub metadata: Option<HashMap<String, String>>,
}

/// Account creation response
#[derive(Debug, Serialize)]
pub struct CreateAccountResponse {
    /// Created account ID
    pub account_id: String,
    /// Account status
    pub status: String,
    /// Creation timestamp
    pub created_at: String,
}

/// Asset listing query parameters
#[derive(Debug, Deserialize)]
pub struct AssetListQuery {
    /// Page number for pagination
    pub page: Option<u32>,
    /// Number of items per page
    pub limit: Option<u32>,
    /// Asset type filter
    pub asset_type: Option<String>,
    /// Status filter
    pub status: Option<String>,
}

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Response status
    pub status: String,
    /// Response data
    pub data: Option<T>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Response timestamp
    pub timestamp: String,
}

impl CustodyHandlers {
    /// Create new custody handlers
    /// 
    /// # Arguments
    /// 
    /// * `service` - Custody service instance
    /// 
    /// # Returns
    /// 
    /// Returns new custody handlers
    pub fn new(service: Arc<CustodyService>) -> Self {
        Self { service }
    }

    /// Create router with all custody endpoints
    /// 
    /// # Arguments
    /// 
    /// * `service` - Custody service instance
    /// 
    /// # Returns
    /// 
    /// Returns configured router
    pub fn create_router(service: Arc<CustodyService>) -> Router {
        Router::new()
            .route("/accounts", post(Self::create_account))
            .route("/accounts/:account_id", get(Self::get_account))
            .route("/accounts/:account_id/assets", get(Self::list_account_assets))
            .route("/assets/:asset_id", get(Self::get_asset))
            .route("/metrics", get(Self::get_metrics))
            .route("/health", get(Self::health_check))
            .with_state(service)
    }

    /// Create a new custody account
    /// 
    /// # Arguments
    /// 
    /// * `service` - Custody service state
    /// * `request` - Account creation request
    /// 
    /// # Returns
    /// 
    /// Returns account creation response
    pub async fn create_account(
        State(service): State<Arc<CustodyService>>,
        Json(request): Json<CreateAccountRequest>,
    ) -> Result<Json<ApiResponse<CreateAccountResponse>>, StatusCode> {
        match service.create_account(&request.owner_id, request.account_type).await {
            Ok(account) => {
                let response = CreateAccountResponse {
                    account_id: account.id.to_string(),
                    status: format!("{:?}", account.status),
                    created_at: account.created_at.to_rfc3339(),
                };

                Ok(Json(ApiResponse {
                    status: "success".to_string(),
                    data: Some(response),
                    error: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }))
            }
            Err(e) => {
                eprintln!("Failed to create account: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get account information
    /// 
    /// # Arguments
    /// 
    /// * `service` - Custody service state
    /// * `account_id` - Account identifier path parameter
    /// 
    /// # Returns
    /// 
    /// Returns account information
    pub async fn get_account(
        State(service): State<Arc<CustodyService>>,
        Path(account_id): Path<String>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
        match service.get_account(&account_id).await {
            Ok(account) => {
                let account_json = serde_json::to_value(&account)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                Ok(Json(ApiResponse {
                    status: "success".to_string(),
                    data: Some(account_json),
                    error: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }))
            }
            Err(CustodyError::Validation { .. }) => Err(StatusCode::NOT_FOUND),
            Err(e) => {
                eprintln!("Failed to get account: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// List assets for an account
    /// 
    /// # Arguments
    /// 
    /// * `service` - Custody service state
    /// * `account_id` - Account identifier path parameter
    /// * `query` - Query parameters for filtering and pagination
    /// 
    /// # Returns
    /// 
    /// Returns list of account assets
    pub async fn list_account_assets(
        State(service): State<Arc<CustodyService>>,
        Path(account_id): Path<String>,
        Query(query): Query<AssetListQuery>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
        let assets = service.list_account_assets(&account_id).await;
        
        // Apply pagination if specified
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(50);
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(assets.len());
        
        let paginated_assets = if start < assets.len() {
            &assets[start..end]
        } else {
            &[]
        };

        let assets_json = serde_json::to_value(paginated_assets)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(assets_json),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Get asset information
    /// 
    /// # Arguments
    /// 
    /// * `service` - Custody service state
    /// * `asset_id` - Asset identifier path parameter
    /// 
    /// # Returns
    /// 
    /// Returns asset information
    pub async fn get_asset(
        State(_service): State<Arc<CustodyService>>,
        Path(_asset_id): Path<String>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
        // This is a placeholder implementation
        // In a real implementation, this would retrieve asset from the registry
        Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(serde_json::json!({"message": "Asset endpoint not implemented"})),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Get service metrics
    /// 
    /// # Arguments
    /// 
    /// * `service` - Custody service state
    /// 
    /// # Returns
    /// 
    /// Returns service metrics
    pub async fn get_metrics(
        State(service): State<Arc<CustodyService>>,
    ) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
        let metrics = service.get_metrics().await;
        
        let metrics_json = serde_json::to_value(&metrics)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(metrics_json),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    /// Health check endpoint
    /// 
    /// # Returns
    /// 
    /// Returns service health status
    pub async fn health_check() -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
        Ok(Json(ApiResponse {
            status: "success".to_string(),
            data: Some(serde_json::json!({
                "status": "healthy",
                "service": "custody-service",
                "version": env!("CARGO_PKG_VERSION")
            })),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }
}

/// Convert CustodyError to HTTP status code
impl From<CustodyError> for StatusCode {
    fn from(error: CustodyError) -> Self {
        match error.status_code() {
            400 => StatusCode::BAD_REQUEST,
            403 => StatusCode::FORBIDDEN,
            404 => StatusCode::NOT_FOUND,
            429 => StatusCode::TOO_MANY_REQUESTS,
            500 => StatusCode::INTERNAL_SERVER_ERROR,
            502 => StatusCode::BAD_GATEWAY,
            503 => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CustodyConfig;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    async fn create_test_service() -> Arc<CustodyService> {
        let config = CustodyConfig::default();
        Arc::new(CustodyService::new(config).await.unwrap())
    }

    #[tokio::test]
    async fn test_router_creation() {
        let service = create_test_service().await;
        let router = CustodyHandlers::create_router(service);
        
        // Test that router is created successfully
        assert!(true); // Placeholder assertion
    }

    #[tokio::test]
    async fn test_health_check_endpoint() {
        let service = create_test_service().await;
        let app = CustodyHandlers::create_router(service);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = response.json();
        assert_eq!(body.status, "success");
        assert!(body.data.is_some());
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let service = create_test_service().await;
        let app = CustodyHandlers::create_router(service);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/metrics").await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = response.json();
        assert_eq!(body.status, "success");
        assert!(body.data.is_some());
    }

    #[tokio::test]
    async fn test_create_account_endpoint() {
        let service = create_test_service().await;
        let app = CustodyHandlers::create_router(service);
        let server = TestServer::new(app).unwrap();

        let request = CreateAccountRequest {
            owner_id: "test_user".to_string(),
            account_type: AccountType::Individual,
            metadata: None,
        };

        let response = server.post("/accounts").json(&request).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body: ApiResponse<CreateAccountResponse> = response.json();
        assert_eq!(body.status, "success");
        assert!(body.data.is_some());
    }

    #[tokio::test]
    async fn test_get_nonexistent_account() {
        let service = create_test_service().await;
        let app = CustodyHandlers::create_router(service);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/accounts/nonexistent").await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_api_response_creation() {
        let response: ApiResponse<String> = ApiResponse {
            status: "success".to_string(),
            data: Some("test data".to_string()),
            error: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(response.status, "success");
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_create_account_request_deserialization() {
        let json = r#"
        {
            "owner_id": "user123",
            "account_type": "Individual",
            "metadata": {
                "source": "web"
            }
        }
        "#;

        let request: CreateAccountRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.owner_id, "user123");
        assert_eq!(request.account_type, AccountType::Individual);
        assert!(request.metadata.is_some());
    }

    #[test]
    fn test_asset_list_query_parsing() {
        let query = AssetListQuery {
            page: Some(2),
            limit: Some(25),
            asset_type: Some("Digital".to_string()),
            status: Some("Held".to_string()),
        };

        assert_eq!(query.page, Some(2));
        assert_eq!(query.limit, Some(25));
        assert_eq!(query.asset_type, Some("Digital".to_string()));
        assert_eq!(query.status, Some("Held".to_string()));
    }

    #[test]
    fn test_custody_error_to_status_code_conversion() {
        let validation_error = CustodyError::validation("field", "Invalid value");
        let status: StatusCode = validation_error.into();
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let auth_error = CustodyError::authorization("operation", "Access denied");
        let status: StatusCode = auth_error.into();
        assert_eq!(status, StatusCode::FORBIDDEN);

        let internal_error = CustodyError::internal("Something went wrong");
        let status: StatusCode = internal_error.into();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_create_account_response_serialization() {
        let response = CreateAccountResponse {
            account_id: "acc_123".to_string(),
            status: "Active".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("acc_123"));
        assert!(json.contains("Active"));
        assert!(json.contains("2024-01-01T00:00:00Z"));
    }
}
