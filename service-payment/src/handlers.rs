// =====================================================================================
// File: service-payment/src/handlers.rs
// Description: HTTP handlers for Payment Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{models::*, service::PaymentService, AppState, PaymentError, PaymentResult};
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Create payment request
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub payment_method_id: Option<String>,
    pub payment_method_type: String,
    pub provider: String,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Payment method request
#[derive(Debug, Deserialize)]
pub struct CreatePaymentMethodRequest {
    pub method_type: String,
    pub provider: String,
    pub provider_payment_method_id: String,
    pub is_default: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Update payment method request
#[derive(Debug, Deserialize)]
pub struct UpdatePaymentMethodRequest {
    pub is_default: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Refund request
#[derive(Debug, Deserialize)]
pub struct RefundRequest {
    pub amount: Option<rust_decimal::Decimal>,
    pub reason: Option<String>,
}

/// Payment response
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: String,
    pub user_id: String,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub status: String,
    pub payment_method_type: String,
    pub provider: String,
    pub provider_payment_id: Option<String>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Payment> for PaymentResponse {
    fn from(payment: Payment) -> Self {
        Self {
            id: payment.id.to_string(),
            user_id: payment.user_id.to_string(),
            amount: payment.amount,
            currency: payment.currency,
            status: payment.status,
            payment_method_type: payment.payment_method_type,
            provider: payment.provider,
            provider_payment_id: payment.provider_payment_id,
            description: payment.description,
            metadata: payment.metadata,
            created_at: payment.created_at,
            updated_at: payment.updated_at,
        }
    }
}

/// Payment method response
#[derive(Debug, Serialize)]
pub struct PaymentMethodResponse {
    pub id: String,
    pub user_id: String,
    pub method_type: String,
    pub provider: String,
    pub provider_payment_method_id: String,
    pub is_default: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<PaymentMethod> for PaymentMethodResponse {
    fn from(method: PaymentMethod) -> Self {
        Self {
            id: method.id.to_string(),
            user_id: method.user_id.to_string(),
            method_type: method.method_type,
            provider: method.provider,
            provider_payment_method_id: method.provider_payment_method_id,
            is_default: method.is_default,
            metadata: method.metadata,
            created_at: method.created_at,
            updated_at: method.updated_at,
        }
    }
}

/// List payments response
#[derive(Debug, Serialize)]
pub struct ListPaymentsResponse {
    pub payments: Vec<PaymentResponse>,
    pub pagination: PaginationResponse,
}

/// List payment methods response
#[derive(Debug, Serialize)]
pub struct ListPaymentMethodsResponse {
    pub payment_methods: Vec<PaymentMethodResponse>,
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

/// Create a new payment
pub async fn create_payment(
    state: web::Data<AppState>,
    req: HttpRequest,
    payment_req: web::Json<CreatePaymentRequest>,
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

    info!("Creating payment for user: {}, amount: {} {}", 
          user_id, payment_req.amount, payment_req.currency);

    match state.payment_service.create_payment(
        &user_id,
        payment_req.amount,
        &payment_req.currency,
        payment_req.payment_method_id.as_deref(),
        &payment_req.payment_method_type,
        &payment_req.provider,
        payment_req.description.as_deref(),
        payment_req.metadata.as_ref(),
    ).await {
        Ok(payment) => {
            info!("Payment created successfully: {}", payment.id);
            Ok(HttpResponse::Created().json(PaymentResponse::from(payment)))
        }
        Err(PaymentError::InsufficientFunds) => {
            warn!("Payment failed - insufficient funds: {}", user_id);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "insufficient_funds",
                "message": "Insufficient funds for this payment"
            })))
        }
        Err(PaymentError::PaymentMethodNotFound(_)) => {
            warn!("Payment failed - payment method not found: {}", user_id);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "payment_method_not_found",
                "message": "Payment method not found"
            })))
        }
        Err(PaymentError::InvalidAmount(msg)) => {
            warn!("Payment failed - invalid amount: {}", msg);
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_amount",
                "message": msg
            })))
        }
        Err(PaymentError::ProviderError(msg)) => {
            error!("Payment failed - provider error: {}", msg);
            Ok(HttpResponse::BadGateway().json(serde_json::json!({
                "error": "provider_error",
                "message": "Payment provider error"
            })))
        }
        Err(e) => {
            error!("Payment creation failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "payment_failed",
                "message": "Failed to create payment"
            })))
        }
    }
}

/// Get payment by ID
pub async fn get_payment(
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

    let payment_id = path.into_inner();
    let uuid = match Uuid::parse_str(&payment_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_payment_id",
                "message": "Invalid payment ID format"
            })));
        }
    };

    match state.payment_service.get_payment(&uuid, &user_id).await {
        Ok(payment) => {
            Ok(HttpResponse::Ok().json(PaymentResponse::from(payment)))
        }
        Err(PaymentError::PaymentNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "payment_not_found",
                "message": "Payment not found"
            })))
        }
        Err(PaymentError::PermissionDenied) => {
            Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "error": "permission_denied",
                "message": "Access denied to this payment"
            })))
        }
        Err(e) => {
            error!("Failed to get payment: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "fetch_failed",
                "message": "Failed to fetch payment"
            })))
        }
    }
}

/// Get payment status
pub async fn get_payment_status(
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

    let payment_id = path.into_inner();
    let uuid = match Uuid::parse_str(&payment_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_payment_id",
                "message": "Invalid payment ID format"
            })));
        }
    };

    match state.payment_service.get_payment_status(&uuid, &user_id).await {
        Ok(status) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "payment_id": payment_id,
                "status": status.status,
                "provider_status": status.provider_status,
                "last_updated": status.last_updated,
                "details": status.details
            })))
        }
        Err(PaymentError::PaymentNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "payment_not_found",
                "message": "Payment not found"
            })))
        }
        Err(e) => {
            error!("Failed to get payment status: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "status_fetch_failed",
                "message": "Failed to fetch payment status"
            })))
        }
    }
}

/// List payments for user
pub async fn list_payments(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaginationQuery>,
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

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match state.payment_service.list_user_payments(&user_id, page, per_page).await {
        Ok((payments, total)) => {
            let payment_responses: Vec<PaymentResponse> = payments
                .into_iter()
                .map(PaymentResponse::from)
                .collect();
            
            let total_pages = (total as f64 / per_page as f64).ceil() as i64;

            Ok(HttpResponse::Ok().json(ListPaymentsResponse {
                payments: payment_responses,
                pagination: PaginationResponse {
                    page,
                    per_page,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            error!("Failed to list payments: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "list_failed",
                "message": "Failed to list payments"
            })))
        }
    }
}

/// Cancel payment
pub async fn cancel_payment(
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

    let payment_id = path.into_inner();
    let uuid = match Uuid::parse_str(&payment_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_payment_id",
                "message": "Invalid payment ID format"
            })));
        }
    };

    match state.payment_service.cancel_payment(&uuid, &user_id).await {
        Ok(payment) => {
            info!("Payment cancelled: {}", uuid);
            Ok(HttpResponse::Ok().json(PaymentResponse::from(payment)))
        }
        Err(PaymentError::PaymentNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "payment_not_found",
                "message": "Payment not found"
            })))
        }
        Err(PaymentError::InvalidPaymentState(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_payment_state",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Failed to cancel payment: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "cancel_failed",
                "message": "Failed to cancel payment"
            })))
        }
    }
}

/// Refund payment
pub async fn refund_payment(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    refund_req: web::Json<RefundRequest>,
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

    let payment_id = path.into_inner();
    let uuid = match Uuid::parse_str(&payment_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_payment_id",
                "message": "Invalid payment ID format"
            })));
        }
    };

    match state.payment_service.refund_payment(
        &uuid,
        &user_id,
        refund_req.amount,
        refund_req.reason.as_deref(),
    ).await {
        Ok(refund) => {
            info!("Payment refunded: {}, refund: {}", uuid, refund.id);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "refund_id": refund.id.to_string(),
                "payment_id": payment_id,
                "amount": refund.amount,
                "status": refund.status,
                "created_at": refund.created_at
            })))
        }
        Err(PaymentError::PaymentNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "payment_not_found",
                "message": "Payment not found"
            })))
        }
        Err(PaymentError::InvalidPaymentState(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_payment_state",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Failed to refund payment: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "refund_failed",
                "message": "Failed to refund payment"
            })))
        }
    }
}

/// Create payment method
pub async fn create_payment_method(
    state: web::Data<AppState>,
    req: HttpRequest,
    method_req: web::Json<CreatePaymentMethodRequest>,
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

    match state.payment_service.create_payment_method(
        &user_id,
        &method_req.method_type,
        &method_req.provider,
        &method_req.provider_payment_method_id,
        method_req.is_default.unwrap_or(false),
        method_req.metadata.as_ref(),
    ).await {
        Ok(method) => {
            info!("Payment method created: {}", method.id);
            Ok(HttpResponse::Created().json(PaymentMethodResponse::from(method)))
        }
        Err(PaymentError::ValidationError(msg)) => {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })))
        }
        Err(e) => {
            error!("Failed to create payment method: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "creation_failed",
                "message": "Failed to create payment method"
            })))
        }
    }
}

/// List payment methods
pub async fn list_payment_methods(
    state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaginationQuery>,
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

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    match state.payment_service.list_user_payment_methods(&user_id, page, per_page).await {
        Ok((methods, total)) => {
            let method_responses: Vec<PaymentMethodResponse> = methods
                .into_iter()
                .map(PaymentMethodResponse::from)
                .collect();
            
            let total_pages = (total as f64 / per_page as f64).ceil() as i64;

            Ok(HttpResponse::Ok().json(ListPaymentMethodsResponse {
                payment_methods: method_responses,
                pagination: PaginationResponse {
                    page,
                    per_page,
                    total,
                    total_pages,
                },
            }))
        }
        Err(e) => {
            error!("Failed to list payment methods: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "list_failed",
                "message": "Failed to list payment methods"
            })))
        }
    }
}

/// Get payment method
pub async fn get_payment_method(
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

    let method_id = path.into_inner();
    let uuid = match Uuid::parse_str(&method_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_method_id",
                "message": "Invalid payment method ID format"
            })));
        }
    };

    match state.payment_service.get_payment_method(&uuid, &user_id).await {
        Ok(method) => {
            Ok(HttpResponse::Ok().json(PaymentMethodResponse::from(method)))
        }
        Err(PaymentError::PaymentMethodNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "payment_method_not_found",
                "message": "Payment method not found"
            })))
        }
        Err(e) => {
            error!("Failed to get payment method: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "fetch_failed",
                "message": "Failed to fetch payment method"
            })))
        }
    }
}

/// Update payment method
pub async fn update_payment_method(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
    update_req: web::Json<UpdatePaymentMethodRequest>,
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

    let method_id = path.into_inner();
    let uuid = match Uuid::parse_str(&method_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_method_id",
                "message": "Invalid payment method ID format"
            })));
        }
    };

    match state.payment_service.update_payment_method(
        &uuid,
        &user_id,
        update_req.is_default,
        update_req.metadata.as_ref(),
    ).await {
        Ok(method) => {
            info!("Payment method updated: {}", uuid);
            Ok(HttpResponse::Ok().json(PaymentMethodResponse::from(method)))
        }
        Err(PaymentError::PaymentMethodNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "payment_method_not_found",
                "message": "Payment method not found"
            })))
        }
        Err(e) => {
            error!("Failed to update payment method: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "update_failed",
                "message": "Failed to update payment method"
            })))
        }
    }
}

/// Delete payment method
pub async fn delete_payment_method(
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

    let method_id = path.into_inner();
    let uuid = match Uuid::parse_str(&method_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_method_id",
                "message": "Invalid payment method ID format"
            })));
        }
    };

    match state.payment_service.delete_payment_method(&uuid, &user_id).await {
        Ok(_) => {
            info!("Payment method deleted: {}", uuid);
            Ok(HttpResponse::NoContent().finish())
        }
        Err(PaymentError::PaymentMethodNotFound(_)) => {
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "payment_method_not_found",
                "message": "Payment method not found"
            })))
        }
        Err(e) => {
            error!("Failed to delete payment method: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "deletion_failed",
                "message": "Failed to delete payment method"
            })))
        }
    }
}

/// Stripe webhook handler
pub async fn stripe_webhook(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Bytes,
) -> ActixResult<HttpResponse> {
    let signature = req.headers()
        .get("stripe-signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    match state.payment_service.handle_stripe_webhook(&body, signature).await {
        Ok(_) => {
            info!("Stripe webhook processed successfully");
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "received": true
            })))
        }
        Err(PaymentError::InvalidWebhookSignature) => {
            warn!("Invalid Stripe webhook signature");
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_signature",
                "message": "Invalid webhook signature"
            })))
        }
        Err(e) => {
            error!("Failed to process Stripe webhook: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "webhook_failed",
                "message": "Failed to process webhook"
            })))
        }
    }
}

/// PayPal webhook handler
pub async fn paypal_webhook(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    match state.payment_service.handle_paypal_webhook(&body).await {
        Ok(_) => {
            info!("PayPal webhook processed successfully");
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "received": true
            })))
        }
        Err(e) => {
            error!("Failed to process PayPal webhook: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "webhook_failed",
                "message": "Failed to process webhook"
            })))
        }
    }
}

/// Health check endpoint
pub async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "payment-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Readiness check endpoint
pub async fn readiness_check(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    // Check database connectivity
    let db_healthy = state.database.health_check().await.is_ok();
    
    // Check payment provider connectivity
    let providers_healthy = state.payment_service.check_providers_health().await;
    
    let ready = db_healthy && providers_healthy;
    let status = if ready { "ready" } else { "not_ready" };
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "service": "payment-service",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": {
            "database": db_healthy,
            "payment_providers": providers_healthy
        }
    })))
}

/// Liveness check endpoint
pub async fn liveness_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "service": "payment-service",
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

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
