// =====================================================================================
// RWA Tokenization Platform - Oracle HTTP Handlers
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{OracleError, OracleResult};
use crate::metrics::RequestTimer;
use crate::models::{
    BatchPriceRequest, CreatePriceFeedRequest, UpdatePriceFeedRequest, 
    SubscriptionRequest, PriceHistoryRequest
};
// Removed unused imports
use crate::{AppState};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError, Result as ActixResult};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

/// Get asset price
pub async fn get_asset_price(
    path: web::Path<String>,
    query: web::Query<serde_json::Value>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let asset_id = path.into_inner();
    let currency = query.get("currency")
        .and_then(|v| v.as_str())
        .unwrap_or("USD");

    debug!("Getting price for asset: {} in currency: {}", asset_id, currency);

    match data.oracle_service.get_asset_price(&asset_id, currency).await {
        Ok(price) => {
            timer.finish(200);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": price,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to get asset price: {}", e);
            let status_code = match &e {
                OracleError::PriceNotFound { .. } => 404,
                OracleError::Validation { .. } => 400,
                OracleError::RateLimitExceeded { .. } => 429,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// Get price history for an asset
pub async fn get_price_history(
    path: web::Path<String>,
    query: web::Query<PriceHistoryRequest>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let asset_id = path.into_inner();
    let request = query.into_inner();

    debug!("Getting price history for asset: {}", asset_id);

    match data.oracle_service.get_price_history(&asset_id, &request).await {
        Ok(history) => {
            timer.finish(200);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": history,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to get price history: {}", e);
            let status_code = match &e {
                OracleError::Validation { .. } => 400,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// Get batch prices for multiple assets
pub async fn get_batch_prices(
    request: web::Json<BatchPriceRequest>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let batch_request = request.into_inner();
    
    debug!("Getting batch prices for {} assets", batch_request.asset_ids.len());

    if batch_request.asset_ids.is_empty() {
        timer.finish(400);
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "asset_ids cannot be empty",
            "timestamp": chrono::Utc::now()
        })));
    }

    if batch_request.asset_ids.len() > 100 {
        timer.finish(400);
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Maximum 100 assets allowed per batch request",
            "timestamp": chrono::Utc::now()
        })));
    }

    match data.oracle_service.get_batch_prices(&batch_request).await {
        Ok(response) => {
            timer.finish(200);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": response,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to get batch prices: {}", e);
            timer.finish(500);
            Ok(e.error_response())
        }
    }
}

/// List all price feeds
pub async fn list_price_feeds(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    debug!("Listing all price feeds");

    match data.oracle_service.list_price_feeds().await {
        Ok(feeds) => {
            timer.finish(200);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": feeds,
                "count": feeds.len(),
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to list price feeds: {}", e);
            timer.finish(500);
            Ok(e.error_response())
        }
    }
}

/// Get a specific price feed
pub async fn get_price_feed(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let feed_id = path.into_inner();
    
    debug!("Getting price feed: {}", feed_id);

    match data.oracle_service.get_price_feed(&feed_id).await {
        Ok(feed) => {
            timer.finish(200);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": feed,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to get price feed: {}", e);
            let status_code = match &e {
                OracleError::FeedNotFound { .. } => 404,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// Create a new price feed
pub async fn create_price_feed(
    request: web::Json<CreatePriceFeedRequest>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let create_request = request.into_inner();
    
    debug!("Creating price feed for asset: {}", create_request.asset_id);

    // Validate request
    if create_request.name.trim().is_empty() {
        timer.finish(400);
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Feed name cannot be empty",
            "timestamp": chrono::Utc::now()
        })));
    }

    if create_request.update_interval < 1 {
        timer.finish(400);
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Update interval must be at least 1 second",
            "timestamp": chrono::Utc::now()
        })));
    }

    match data.oracle_service.create_price_feed(&create_request).await {
        Ok(feed) => {
            timer.finish(201);
            Ok(HttpResponse::Created().json(json!({
                "success": true,
                "data": feed,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to create price feed: {}", e);
            let status_code = match &e {
                OracleError::Validation { .. } => 400,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// Update an existing price feed
pub async fn update_price_feed(
    path: web::Path<Uuid>,
    request: web::Json<UpdatePriceFeedRequest>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let feed_id = path.into_inner();
    let update_request = request.into_inner();
    
    debug!("Updating price feed: {}", feed_id);

    match data.oracle_service.update_price_feed(&feed_id, &update_request).await {
        Ok(feed) => {
            timer.finish(200);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": feed,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to update price feed: {}", e);
            let status_code = match &e {
                OracleError::FeedNotFound { .. } => 404,
                OracleError::Validation { .. } => 400,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// Delete a price feed
pub async fn delete_price_feed(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let feed_id = path.into_inner();
    
    debug!("Deleting price feed: {}", feed_id);

    match data.oracle_service.delete_price_feed(&feed_id).await {
        Ok(()) => {
            timer.finish(204);
            Ok(HttpResponse::NoContent().finish())
        }
        Err(e) => {
            error!("Failed to delete price feed: {}", e);
            let status_code = match &e {
                OracleError::FeedNotFound { .. } => 404,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// Subscribe to a price feed
pub async fn subscribe_to_feed(
    path: web::Path<Uuid>,
    request: web::Json<SubscriptionRequest>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let feed_id = path.into_inner();
    let subscription_request = request.into_inner();
    
    debug!("Creating subscription for feed: {}", feed_id);

    // Validate request
    if subscription_request.subscriber_id.trim().is_empty() {
        timer.finish(400);
        return Ok(HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "Subscriber ID cannot be empty",
            "timestamp": chrono::Utc::now()
        })));
    }

    match data.oracle_service.subscribe_to_feed(&feed_id, &subscription_request).await {
        Ok(subscription) => {
            timer.finish(201);
            Ok(HttpResponse::Created().json(json!({
                "success": true,
                "data": subscription,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to create subscription: {}", e);
            let status_code = match &e {
                OracleError::FeedNotFound { .. } => 404,
                OracleError::Validation { .. } => 400,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// Unsubscribe from a price feed
pub async fn unsubscribe_from_feed(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let subscription_id = path.into_inner();
    
    debug!("Deleting subscription: {}", subscription_id);

    match data.oracle_service.unsubscribe_from_feed(&subscription_id).await {
        Ok(()) => {
            timer.finish(204);
            Ok(HttpResponse::NoContent().finish())
        }
        Err(e) => {
            error!("Failed to delete subscription: {}", e);
            let status_code = match &e {
                OracleError::SubscriptionNotFound { .. } => 404,
                _ => 500,
            };
            timer.finish(status_code);
            Ok(e.error_response())
        }
    }
}

/// List available providers
pub async fn list_providers(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    debug!("Listing available providers");

    match data.oracle_service.get_provider_status().await {
        Ok(provider_status) => {
            timer.finish(200);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": provider_status,
                "timestamp": chrono::Utc::now()
            })))
        }
        Err(e) => {
            error!("Failed to get provider status: {}", e);
            timer.finish(500);
            Ok(e.error_response())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::PriceCache;
    use crate::config::{OracleConfig, RedisConfig};
    use crate::health::HealthService;
    use crate::metrics::OracleMetrics;
    use crate::models::{AssetPrice, AggregationMethod};
    use crate::service::OracleServiceTrait;
    use actix_web::{test, web, App};
    use async_trait::async_trait;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use uuid::Uuid;

    // Mock Oracle Service for testing
    struct MockOracleService {
        should_fail: bool,
    }

    #[async_trait]
    impl OracleServiceTrait for MockOracleService {
        async fn get_asset_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
            if self.should_fail {
                return Err(OracleError::PriceNotFound {
                    asset_id: asset_id.to_string(),
                });
            }

            Ok(AssetPrice {
                asset_id: asset_id.to_string(),
                price: dec!(50000.0),
                currency: currency.to_string(),
                timestamp: Utc::now(),
                confidence: dec!(0.95),
                source: "mock".to_string(),
                metadata: Some(HashMap::new()),
            })
        }

        async fn get_batch_prices(&self, request: &BatchPriceRequest) -> OracleResult<crate::models::BatchPriceResponse> {
            if self.should_fail {
                return Err(OracleError::Internal("Mock failure".to_string()));
            }

            let mut prices = HashMap::new();
            let currency = request.currency.as_deref().unwrap_or("USD");

            for asset_id in &request.asset_ids {
                prices.insert(asset_id.clone(), AssetPrice {
                    asset_id: asset_id.clone(),
                    price: dec!(1000.0),
                    currency: currency.to_string(),
                    timestamp: Utc::now(),
                    confidence: dec!(0.9),
                    source: "mock".to_string(),
                    metadata: Some(HashMap::new()),
                });
            }

            Ok(crate::models::BatchPriceResponse {
                prices,
                errors: HashMap::new(),
                timestamp: Utc::now(),
            })
        }

        async fn get_price_history(&self, _asset_id: &str, _request: &PriceHistoryRequest) -> OracleResult<crate::models::PriceHistoryResponse> {
            if self.should_fail {
                return Err(OracleError::Internal("Mock failure".to_string()));
            }

            Ok(crate::models::PriceHistoryResponse {
                asset_id: "BTC".to_string(),
                currency: "USD".to_string(),
                data: vec![],
                interval: "1h".to_string(),
                total_count: 0,
            })
        }

        async fn create_price_feed(&self, request: &CreatePriceFeedRequest) -> OracleResult<crate::models::PriceFeed> {
            if self.should_fail {
                return Err(OracleError::Internal("Mock failure".to_string()));
            }

            let now = Utc::now();
            Ok(crate::models::PriceFeed {
                id: Uuid::new_v4(),
                asset_id: request.asset_id.clone(),
                name: request.name.clone(),
                description: request.description.clone(),
                currency: request.currency.clone(),
                update_interval: request.update_interval,
                providers: request.providers.clone(),
                aggregation_method: request.aggregation_method.clone(),
                deviation_threshold: request.deviation_threshold,
                is_active: true,
                created_at: now,
                updated_at: now,
            })
        }

        async fn update_price_feed(&self, _feed_id: &Uuid, _request: &UpdatePriceFeedRequest) -> OracleResult<crate::models::PriceFeed> {
            if self.should_fail {
                return Err(OracleError::FeedNotFound {
                    feed_id: "test-id".to_string(),
                });
            }

            let now = Utc::now();
            Ok(crate::models::PriceFeed {
                id: Uuid::new_v4(),
                asset_id: "BTC".to_string(),
                name: "Updated Feed".to_string(),
                description: Some("Updated description".to_string()),
                currency: "USD".to_string(),
                update_interval: 60,
                providers: vec!["CoinGecko".to_string()],
                aggregation_method: AggregationMethod::Mean,
                deviation_threshold: dec!(5.0),
                is_active: true,
                created_at: now,
                updated_at: now,
            })
        }

        async fn delete_price_feed(&self, _feed_id: &Uuid) -> OracleResult<()> {
            if self.should_fail {
                return Err(OracleError::FeedNotFound {
                    feed_id: "test-id".to_string(),
                });
            }
            Ok(())
        }

        async fn get_price_feed(&self, _feed_id: &Uuid) -> OracleResult<crate::models::PriceFeed> {
            if self.should_fail {
                return Err(OracleError::FeedNotFound {
                    feed_id: "test-id".to_string(),
                });
            }

            let now = Utc::now();
            Ok(crate::models::PriceFeed {
                id: Uuid::new_v4(),
                asset_id: "BTC".to_string(),
                name: "Test Feed".to_string(),
                description: Some("Test description".to_string()),
                currency: "USD".to_string(),
                update_interval: 60,
                providers: vec!["CoinGecko".to_string()],
                aggregation_method: AggregationMethod::Mean,
                deviation_threshold: dec!(5.0),
                is_active: true,
                created_at: now,
                updated_at: now,
            })
        }

        async fn list_price_feeds(&self) -> OracleResult<Vec<crate::models::PriceFeed>> {
            if self.should_fail {
                return Err(OracleError::Internal("Mock failure".to_string()));
            }
            Ok(vec![])
        }

        async fn subscribe_to_feed(&self, feed_id: &Uuid, request: &SubscriptionRequest) -> OracleResult<crate::models::PriceSubscription> {
            if self.should_fail {
                return Err(OracleError::FeedNotFound {
                    feed_id: feed_id.to_string(),
                });
            }

            Ok(crate::models::PriceSubscription {
                id: Uuid::new_v4(),
                feed_id: *feed_id,
                subscriber_id: request.subscriber_id.clone(),
                webhook_url: request.webhook_url.clone(),
                filters: request.filters.clone(),
                is_active: true,
                created_at: Utc::now(),
            })
        }

        async fn unsubscribe_from_feed(&self, _subscription_id: &Uuid) -> OracleResult<()> {
            if self.should_fail {
                return Err(OracleError::SubscriptionNotFound {
                    subscription_id: "test-id".to_string(),
                });
            }
            Ok(())
        }

        async fn get_provider_status(&self) -> OracleResult<HashMap<String, bool>> {
            if self.should_fail {
                return Err(OracleError::Internal("Mock failure".to_string()));
            }

            let mut status = HashMap::new();
            status.insert("CoinGecko".to_string(), true);
            status.insert("Binance".to_string(), false);
            Ok(status)
        }
    }

    fn create_test_app_state(should_fail: bool) -> AppState {
        let oracle_service = Arc::new(MockOracleService { should_fail });
        let metrics = Arc::new(OracleMetrics::new().unwrap());

        // Create a mock health service (simplified)
        let redis_config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            command_timeout: 5,
            retry_attempts: 3,
        };

        // Note: In a real test, you'd use a test database and Redis instance
        // For now, we'll create a minimal health service
        let health_service = Arc::new(crate::health::HealthService::new(
            sqlx::PgPool::connect("postgresql://test").await.unwrap_or_else(|_| {
                // Create a dummy pool for testing
                panic!("Test database not available")
            }),
            Arc::new(PriceCache::new(&redis_config).await.unwrap_or_else(|_| {
                panic!("Test Redis not available")
            })),
            Duration::from_secs(30),
        ));

        AppState {
            oracle_service,
            health_service,
            metrics,
        }
    }

    #[actix_web::test]
    async fn test_get_asset_price_success() {
        let app_state = create_test_app_state(false);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/prices/{asset_id}", web::get().to(get_asset_price))
        ).await;

        let req = test::TestRequest::get()
            .uri("/prices/BTC?currency=USD")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert!(body["data"].is_object());
        assert_eq!(body["data"]["asset_id"], "BTC");
        assert_eq!(body["data"]["currency"], "USD");
    }

    #[actix_web::test]
    async fn test_get_asset_price_not_found() {
        let app_state = create_test_app_state(true); // should_fail = true
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/prices/{asset_id}", web::get().to(get_asset_price))
        ).await;

        let req = test::TestRequest::get()
            .uri("/prices/UNKNOWN")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "price_not_found");
    }

    #[actix_web::test]
    async fn test_get_batch_prices_success() {
        let app_state = create_test_app_state(false);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/prices/batch", web::post().to(get_batch_prices))
        ).await;

        let batch_request = BatchPriceRequest {
            asset_ids: vec!["BTC".to_string(), "ETH".to_string()],
            currency: Some("USD".to_string()),
            include_metadata: Some(true),
        };

        let req = test::TestRequest::post()
            .uri("/prices/batch")
            .set_json(&batch_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert!(body["data"].is_object());
    }

    #[actix_web::test]
    async fn test_get_batch_prices_empty_request() {
        let app_state = create_test_app_state(false);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/prices/batch", web::post().to(get_batch_prices))
        ).await;

        let batch_request = BatchPriceRequest {
            asset_ids: vec![], // Empty array
            currency: Some("USD".to_string()),
            include_metadata: Some(true),
        };

        let req = test::TestRequest::post()
            .uri("/prices/batch")
            .set_json(&batch_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], false);
        assert!(body["error"].as_str().unwrap().contains("cannot be empty"));
    }

    #[actix_web::test]
    async fn test_create_price_feed_success() {
        let app_state = create_test_app_state(false);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/feeds", web::post().to(create_price_feed))
        ).await;

        let create_request = CreatePriceFeedRequest {
            asset_id: "BTC".to_string(),
            name: "Bitcoin Feed".to_string(),
            description: Some("Bitcoin price feed".to_string()),
            currency: "USD".to_string(),
            update_interval: 60,
            providers: vec!["CoinGecko".to_string()],
            aggregation_method: AggregationMethod::Mean,
            deviation_threshold: dec!(5.0),
        };

        let req = test::TestRequest::post()
            .uri("/feeds")
            .set_json(&create_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], true);
        assert!(body["data"].is_object());
        assert_eq!(body["data"]["asset_id"], "BTC");
        assert_eq!(body["data"]["name"], "Bitcoin Feed");
    }

    #[actix_web::test]
    async fn test_create_price_feed_invalid_name() {
        let app_state = create_test_app_state(false);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/feeds", web::post().to(create_price_feed))
        ).await;

        let create_request = CreatePriceFeedRequest {
            asset_id: "BTC".to_string(),
            name: "".to_string(), // Empty name
            description: Some("Bitcoin price feed".to_string()),
            currency: "USD".to_string(),
            update_interval: 60,
            providers: vec!["CoinGecko".to_string()],
            aggregation_method: AggregationMethod::Mean,
            deviation_threshold: dec!(5.0),
        };

        let req = test::TestRequest::post()
            .uri("/feeds")
            .set_json(&create_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["success"], false);
        assert!(body["error"].as_str().unwrap().contains("name cannot be empty"));
    }

    #[actix_web::test]
    async fn test_health_check() {
        let app_state = create_test_app_state(false);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/health", web::get().to(health_check))
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Note: This might return 503 if health service dependencies are not available
        assert!(resp.status() == 200 || resp.status() == 503);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["data"].is_object());
        assert!(body["timestamp"].is_string());
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let app_state = create_test_app_state(false);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .route("/metrics", web::get().to(metrics_endpoint))
        ).await;

        let req = test::TestRequest::get()
            .uri("/metrics")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let content_type = resp.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("text/plain"));

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("rwa_oracle"));
    }
}

/// Get provider status
pub async fn get_provider_status(
    path: web::Path<String>,
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    let provider_id = path.into_inner();
    
    debug!("Getting status for provider: {}", provider_id);

    match data.oracle_service.get_provider_status().await {
        Ok(all_status) => {
            if let Some(status) = all_status.get(&provider_id) {
                timer.finish(200);
                Ok(HttpResponse::Ok().json(json!({
                    "success": true,
                    "data": {
                        "provider_id": provider_id,
                        "healthy": status,
                        "last_check": chrono::Utc::now()
                    },
                    "timestamp": chrono::Utc::now()
                })))
            } else {
                timer.finish(404);
                Ok(HttpResponse::NotFound().json(json!({
                    "success": false,
                    "error": format!("Provider '{}' not found", provider_id),
                    "timestamp": chrono::Utc::now()
                })))
            }
        }
        Err(e) => {
            error!("Failed to get provider status: {}", e);
            timer.finish(500);
            Ok(e.error_response())
        }
    }
}

/// Health check endpoint
pub async fn health_check(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    match data.health_service.get_health_status().await {
        health_status => {
            let status_code = match health_status.status {
                crate::health::ServiceStatus::Healthy => 200,
                crate::health::ServiceStatus::Degraded => 200,
                crate::health::ServiceStatus::Unhealthy => 503,
                crate::health::ServiceStatus::Unknown => 503,
            };
            
            timer.finish(status_code);
            Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
                .json(json!({
                    "success": status_code == 200,
                    "data": health_status,
                    "timestamp": chrono::Utc::now()
                })))
        }
    }
}

/// Readiness probe endpoint
pub async fn readiness_check(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    match data.health_service.readiness_check().await {
        Ok(ready) => {
            let status_code = if ready { 200 } else { 503 };
            timer.finish(status_code);
            Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
                .json(json!({
                    "ready": ready,
                    "timestamp": chrono::Utc::now()
                })))
        }
        Err(e) => {
            error!("Readiness check failed: {}", e);
            timer.finish(503);
            Ok(HttpResponse::ServiceUnavailable().json(json!({
                "ready": false,
                "error": e.to_string(),
                "timestamp": chrono::Utc::now()
            })))
        }
    }
}

/// Liveness probe endpoint
pub async fn liveness_probe(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    match data.health_service.liveness_check().await {
        Ok(alive) => {
            let status_code = if alive { 200 } else { 503 };
            timer.finish(status_code);
            Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
                .json(json!({
                    "alive": alive,
                    "timestamp": chrono::Utc::now()
                })))
        }
        Err(e) => {
            error!("Liveness check failed: {}", e);
            timer.finish(503);
            Ok(HttpResponse::ServiceUnavailable().json(json!({
                "alive": false,
                "error": e.to_string(),
                "timestamp": chrono::Utc::now()
            })))
        }
    }
}

/// Metrics endpoint
pub async fn metrics_endpoint(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> ActixResult<HttpResponse> {
    let timer = RequestTimer::new(
        data.metrics.clone(),
        req.method().to_string(),
        req.path().to_string(),
    );

    match data.metrics.export_metrics() {
        Ok(metrics_text) => {
            timer.finish(200);
            Ok(HttpResponse::Ok()
                .content_type("text/plain; version=0.0.4; charset=utf-8")
                .body(metrics_text))
        }
        Err(e) => {
            error!("Failed to export metrics: {}", e);
            timer.finish(500);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to export metrics",
                "timestamp": chrono::Utc::now()
            })))
        }
    }
}
