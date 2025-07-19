// =====================================================================================
// File: service-payment/src/service.rs
// Description: Payment service business logic implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{PaymentError, PaymentRepository, Payment, PaymentMethod, PaymentProvider};
use async_trait::async_trait;
use core_utils::{validation::RwaValidate, helpers::{Pagination, PaginatedResponse}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Payment service trait
#[async_trait]
pub trait PaymentService: Send + Sync {
    /// Process a payment
    async fn process_payment(&self, request: ProcessPaymentRequest) -> Result<PaymentResponse, PaymentError>;
    
    /// Get payment by ID
    async fn get_payment(&self, payment_id: &str) -> Result<Option<PaymentResponse>, PaymentError>;
    
    /// List payments with pagination
    async fn list_payments(&self, pagination: Pagination, filters: PaymentFilters) -> Result<PaginatedResponse<PaymentResponse>, PaymentError>;
    
    /// Get payments by user
    async fn get_payments_by_user(&self, user_id: &str, pagination: Pagination) -> Result<PaginatedResponse<PaymentResponse>, PaymentError>;
    
    /// Refund a payment
    async fn refund_payment(&self, payment_id: &str, amount: Option<f64>, reason: &str) -> Result<RefundResponse, PaymentError>;
    
    /// Cancel a pending payment
    async fn cancel_payment(&self, payment_id: &str, reason: &str) -> Result<(), PaymentError>;
    
    /// Update payment status (for webhook callbacks)
    async fn update_payment_status(&self, payment_id: &str, status: PaymentStatus, provider_reference: Option<String>) -> Result<(), PaymentError>;
    
    /// Get payment methods for user
    async fn get_payment_methods(&self, user_id: &str) -> Result<Vec<PaymentMethodResponse>, PaymentError>;
    
    /// Add payment method for user
    async fn add_payment_method(&self, user_id: &str, request: AddPaymentMethodRequest) -> Result<PaymentMethodResponse, PaymentError>;
    
    /// Remove payment method
    async fn remove_payment_method(&self, user_id: &str, method_id: &str) -> Result<(), PaymentError>;
    
    /// Get payment analytics
    async fn get_payment_analytics(&self, filters: AnalyticsFilters) -> Result<PaymentAnalytics, PaymentError>;
}

/// Payment service implementation
pub struct PaymentServiceImpl {
    repository: Arc<dyn PaymentRepository>,
    providers: HashMap<String, Arc<dyn PaymentProvider>>,
}

impl PaymentServiceImpl {
    pub fn new(repository: Arc<dyn PaymentRepository>) -> Self {
        let mut providers = HashMap::new();
        
        // Add payment providers
        providers.insert("stripe".to_string(), Arc::new(StripeProvider::new()) as Arc<dyn PaymentProvider>);
        providers.insert("paypal".to_string(), Arc::new(PayPalProvider::new()) as Arc<dyn PaymentProvider>);
        providers.insert("crypto".to_string(), Arc::new(CryptoProvider::new()) as Arc<dyn PaymentProvider>);

        Self {
            repository,
            providers,
        }
    }

    /// Validate payment request
    fn validate_payment_request(&self, request: &ProcessPaymentRequest) -> Result<(), PaymentError> {
        RwaValidate::payment_data(
            request.amount,
            &request.currency,
            &request.payment_method_type,
        ).map_err(|e| PaymentError::ValidationError(e.to_string()))
    }

    /// Calculate fees
    fn calculate_fees(&self, amount: f64, payment_method: &str, provider: &str) -> f64 {
        match (provider, payment_method) {
            ("stripe", "credit_card") => amount * 0.029 + 0.30, // 2.9% + $0.30
            ("stripe", "bank_transfer") => amount * 0.008,       // 0.8%
            ("paypal", "paypal") => amount * 0.034 + 0.30,      // 3.4% + $0.30
            ("crypto", _) => amount * 0.01,                     // 1%
            _ => amount * 0.03,                                 // Default 3%
        }
    }
}

#[async_trait]
impl PaymentService for PaymentServiceImpl {
    async fn process_payment(&self, request: ProcessPaymentRequest) -> Result<PaymentResponse, PaymentError> {
        info!(
            user_id = %request.user_id,
            amount = %request.amount,
            currency = %request.currency,
            payment_method = %request.payment_method_type,
            "Processing payment"
        );

        // Validate request
        self.validate_payment_request(&request)?;

        // Get payment provider
        let provider = self.providers.get(&request.provider)
            .ok_or_else(|| PaymentError::UnsupportedProvider(request.provider.clone()))?;

        // Calculate fees
        let fees = self.calculate_fees(request.amount, &request.payment_method_type, &request.provider);
        let net_amount = request.amount - fees;

        // Create payment record
        let payment = Payment {
            id: Uuid::new_v4().to_string(),
            user_id: request.user_id.clone(),
            amount: request.amount,
            currency: request.currency.clone(),
            fees,
            net_amount,
            payment_method_type: request.payment_method_type.clone(),
            provider: request.provider.clone(),
            provider_payment_id: None,
            status: PaymentStatus::Pending,
            description: request.description.clone(),
            metadata: request.metadata.clone(),
            failure_reason: None,
            refunded_amount: 0.0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Save payment to database
        let created_payment = self.repository.create(&payment).await?;

        // Process payment with provider
        match provider.process_payment(&request, &created_payment.id).await {
            Ok(provider_response) => {
                // Update payment with provider response
                let mut updated_payment = created_payment;
                updated_payment.provider_payment_id = Some(provider_response.provider_payment_id);
                updated_payment.status = provider_response.status;
                updated_payment.updated_at = chrono::Utc::now();

                let final_payment = self.repository.update(&updated_payment).await?;

                info!(
                    payment_id = %final_payment.id,
                    provider_payment_id = ?final_payment.provider_payment_id,
                    status = ?final_payment.status,
                    "Payment processed successfully"
                );

                Ok(PaymentResponse::from_payment(final_payment))
            }
            Err(provider_error) => {
                // Update payment status to failed
                let mut failed_payment = created_payment;
                failed_payment.status = PaymentStatus::Failed;
                failed_payment.failure_reason = Some(provider_error.to_string());
                failed_payment.updated_at = chrono::Utc::now();

                self.repository.update(&failed_payment).await?;

                error!(
                    payment_id = %failed_payment.id,
                    error = %provider_error,
                    "Payment processing failed"
                );

                Err(PaymentError::ProcessingFailed(provider_error.to_string()))
            }
        }
    }

    async fn get_payment(&self, payment_id: &str) -> Result<Option<PaymentResponse>, PaymentError> {
        debug!(payment_id = %payment_id, "Getting payment");

        let payment = self.repository.find_by_id(payment_id).await?;
        Ok(payment.map(PaymentResponse::from_payment))
    }

    async fn list_payments(&self, pagination: Pagination, filters: PaymentFilters) -> Result<PaginatedResponse<PaymentResponse>, PaymentError> {
        debug!(
            page = pagination.page,
            per_page = pagination.per_page,
            "Listing payments with filters"
        );

        let (payments, total_count) = self.repository.list_with_filters(pagination.clone(), filters).await?;
        
        let payment_responses: Vec<PaymentResponse> = payments
            .into_iter()
            .map(PaymentResponse::from_payment)
            .collect();

        let pagination_with_total = pagination.with_total(total_count);

        Ok(PaginatedResponse::new(payment_responses, pagination_with_total))
    }

    async fn get_payments_by_user(&self, user_id: &str, pagination: Pagination) -> Result<PaginatedResponse<PaymentResponse>, PaymentError> {
        debug!(
            user_id = %user_id,
            page = pagination.page,
            per_page = pagination.per_page,
            "Getting payments by user"
        );

        let (payments, total_count) = self.repository.find_by_user(user_id, pagination.clone()).await?;
        
        let payment_responses: Vec<PaymentResponse> = payments
            .into_iter()
            .map(PaymentResponse::from_payment)
            .collect();

        let pagination_with_total = pagination.with_total(total_count);

        Ok(PaginatedResponse::new(payment_responses, pagination_with_total))
    }

    async fn refund_payment(&self, payment_id: &str, amount: Option<f64>, reason: &str) -> Result<RefundResponse, PaymentError> {
        info!(
            payment_id = %payment_id,
            refund_amount = ?amount,
            reason = %reason,
            "Processing refund"
        );

        // Get payment
        let mut payment = self.repository.find_by_id(payment_id).await?
            .ok_or_else(|| PaymentError::PaymentNotFound(payment_id.to_string()))?;

        // Check if payment can be refunded
        if payment.status != PaymentStatus::Completed {
            return Err(PaymentError::CannotRefundPayment("Payment not completed".to_string()));
        }

        let refund_amount = amount.unwrap_or(payment.amount - payment.refunded_amount);
        
        if refund_amount <= 0.0 || refund_amount > (payment.amount - payment.refunded_amount) {
            return Err(PaymentError::InvalidRefundAmount);
        }

        // Get payment provider
        let provider = self.providers.get(&payment.provider)
            .ok_or_else(|| PaymentError::UnsupportedProvider(payment.provider.clone()))?;

        // Process refund with provider
        let refund_request = RefundRequest {
            payment_id: payment_id.to_string(),
            provider_payment_id: payment.provider_payment_id.clone()
                .ok_or_else(|| PaymentError::MissingProviderPaymentId)?,
            amount: refund_amount,
            reason: reason.to_string(),
        };

        match provider.process_refund(&refund_request).await {
            Ok(provider_refund) => {
                // Update payment with refund info
                payment.refunded_amount += refund_amount;
                if payment.refunded_amount >= payment.amount {
                    payment.status = PaymentStatus::Refunded;
                } else {
                    payment.status = PaymentStatus::PartiallyRefunded;
                }
                payment.updated_at = chrono::Utc::now();

                self.repository.update(&payment).await?;

                // Record refund
                self.repository.record_refund(payment_id, refund_amount, reason).await?;

                info!(
                    payment_id = %payment_id,
                    refund_amount = %refund_amount,
                    "Refund processed successfully"
                );

                Ok(RefundResponse {
                    refund_id: provider_refund.refund_id,
                    payment_id: payment_id.to_string(),
                    amount: refund_amount,
                    status: RefundStatus::Completed,
                    reason: reason.to_string(),
                    processed_at: chrono::Utc::now(),
                })
            }
            Err(provider_error) => {
                error!(
                    payment_id = %payment_id,
                    error = %provider_error,
                    "Refund processing failed"
                );

                Err(PaymentError::RefundFailed(provider_error.to_string()))
            }
        }
    }

    async fn cancel_payment(&self, payment_id: &str, reason: &str) -> Result<(), PaymentError> {
        info!(
            payment_id = %payment_id,
            reason = %reason,
            "Cancelling payment"
        );

        // Get payment
        let mut payment = self.repository.find_by_id(payment_id).await?
            .ok_or_else(|| PaymentError::PaymentNotFound(payment_id.to_string()))?;

        // Check if payment can be cancelled
        if payment.status != PaymentStatus::Pending {
            return Err(PaymentError::CannotCancelPayment("Payment not pending".to_string()));
        }

        // Update payment status
        payment.status = PaymentStatus::Cancelled;
        payment.failure_reason = Some(reason.to_string());
        payment.updated_at = chrono::Utc::now();

        self.repository.update(&payment).await?;

        info!(payment_id = %payment_id, "Payment cancelled successfully");
        Ok(())
    }

    async fn update_payment_status(&self, payment_id: &str, status: PaymentStatus, provider_reference: Option<String>) -> Result<(), PaymentError> {
        debug!(
            payment_id = %payment_id,
            status = ?status,
            "Updating payment status"
        );

        // Get payment
        let mut payment = self.repository.find_by_id(payment_id).await?
            .ok_or_else(|| PaymentError::PaymentNotFound(payment_id.to_string()))?;

        // Update status
        payment.status = status;
        if let Some(reference) = provider_reference {
            payment.provider_payment_id = Some(reference);
        }
        payment.updated_at = chrono::Utc::now();

        self.repository.update(&payment).await?;

        info!(
            payment_id = %payment_id,
            status = ?payment.status,
            "Payment status updated successfully"
        );

        Ok(())
    }

    async fn get_payment_methods(&self, user_id: &str) -> Result<Vec<PaymentMethodResponse>, PaymentError> {
        debug!(user_id = %user_id, "Getting payment methods");

        let methods = self.repository.get_payment_methods(user_id).await?;
        Ok(methods.into_iter().map(PaymentMethodResponse::from_payment_method).collect())
    }

    async fn add_payment_method(&self, user_id: &str, request: AddPaymentMethodRequest) -> Result<PaymentMethodResponse, PaymentError> {
        info!(
            user_id = %user_id,
            method_type = %request.method_type,
            "Adding payment method"
        );

        let payment_method = PaymentMethod {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            method_type: request.method_type,
            provider: request.provider,
            provider_method_id: request.provider_method_id,
            is_default: request.is_default,
            metadata: request.metadata,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let created_method = self.repository.create_payment_method(&payment_method).await?;

        info!(
            user_id = %user_id,
            method_id = %created_method.id,
            "Payment method added successfully"
        );

        Ok(PaymentMethodResponse::from_payment_method(created_method))
    }

    async fn remove_payment_method(&self, user_id: &str, method_id: &str) -> Result<(), PaymentError> {
        info!(
            user_id = %user_id,
            method_id = %method_id,
            "Removing payment method"
        );

        self.repository.delete_payment_method(user_id, method_id).await?;

        info!(
            user_id = %user_id,
            method_id = %method_id,
            "Payment method removed successfully"
        );

        Ok(())
    }

    async fn get_payment_analytics(&self, filters: AnalyticsFilters) -> Result<PaymentAnalytics, PaymentError> {
        debug!("Getting payment analytics");

        let analytics = self.repository.get_analytics(filters).await?;
        Ok(analytics)
    }
}

/// Mock payment providers for demonstration
struct StripeProvider;
struct PayPalProvider;
struct CryptoProvider;

impl StripeProvider {
    fn new() -> Self {
        Self
    }
}

impl PayPalProvider {
    fn new() -> Self {
        Self
    }
}

impl CryptoProvider {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PaymentProvider for StripeProvider {
    async fn process_payment(&self, request: &ProcessPaymentRequest, payment_id: &str) -> Result<ProviderPaymentResponse, PaymentError> {
        // Simulate Stripe API call
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(ProviderPaymentResponse {
            provider_payment_id: format!("stripe_{}", Uuid::new_v4()),
            status: PaymentStatus::Completed,
        })
    }

    async fn process_refund(&self, request: &RefundRequest) -> Result<ProviderRefundResponse, PaymentError> {
        // Simulate Stripe refund API call
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(ProviderRefundResponse {
            refund_id: format!("stripe_refund_{}", Uuid::new_v4()),
        })
    }
}

#[async_trait]
impl PaymentProvider for PayPalProvider {
    async fn process_payment(&self, request: &ProcessPaymentRequest, payment_id: &str) -> Result<ProviderPaymentResponse, PaymentError> {
        // Simulate PayPal API call
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        Ok(ProviderPaymentResponse {
            provider_payment_id: format!("paypal_{}", Uuid::new_v4()),
            status: PaymentStatus::Completed,
        })
    }

    async fn process_refund(&self, request: &RefundRequest) -> Result<ProviderRefundResponse, PaymentError> {
        // Simulate PayPal refund API call
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        Ok(ProviderRefundResponse {
            refund_id: format!("paypal_refund_{}", Uuid::new_v4()),
        })
    }
}

#[async_trait]
impl PaymentProvider for CryptoProvider {
    async fn process_payment(&self, request: &ProcessPaymentRequest, payment_id: &str) -> Result<ProviderPaymentResponse, PaymentError> {
        // Simulate blockchain transaction
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        Ok(ProviderPaymentResponse {
            provider_payment_id: format!("crypto_{}", Uuid::new_v4()),
            status: PaymentStatus::Pending, // Crypto payments often start as pending
        })
    }

    async fn process_refund(&self, request: &RefundRequest) -> Result<ProviderRefundResponse, PaymentError> {
        // Crypto refunds are typically not supported
        Err(PaymentError::RefundNotSupported)
    }
}

/// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPaymentRequest {
    pub user_id: String,
    pub amount: f64,
    pub currency: String,
    pub payment_method_type: String,
    pub provider: String,
    pub description: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFilters {
    pub user_id: Option<String>,
    pub status: Option<PaymentStatus>,
    pub payment_method_type: Option<String>,
    pub provider: Option<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub currency: Option<String>,
    pub date_from: Option<chrono::DateTime<chrono::Utc>>,
    pub date_to: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPaymentMethodRequest {
    pub method_type: String,
    pub provider: String,
    pub provider_method_id: String,
    pub is_default: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsFilters {
    pub date_from: chrono::DateTime<chrono::Utc>,
    pub date_to: chrono::DateTime<chrono::Utc>,
    pub currency: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub id: String,
    pub user_id: String,
    pub amount: f64,
    pub currency: String,
    pub fees: f64,
    pub net_amount: f64,
    pub payment_method_type: String,
    pub provider: String,
    pub provider_payment_id: Option<String>,
    pub status: PaymentStatus,
    pub description: Option<String>,
    pub failure_reason: Option<String>,
    pub refunded_amount: f64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PaymentResponse {
    pub fn from_payment(payment: Payment) -> Self {
        Self {
            id: payment.id,
            user_id: payment.user_id,
            amount: payment.amount,
            currency: payment.currency,
            fees: payment.fees,
            net_amount: payment.net_amount,
            payment_method_type: payment.payment_method_type,
            provider: payment.provider,
            provider_payment_id: payment.provider_payment_id,
            status: payment.status,
            description: payment.description,
            failure_reason: payment.failure_reason,
            refunded_amount: payment.refunded_amount,
            created_at: payment.created_at,
            updated_at: payment.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodResponse {
    pub id: String,
    pub user_id: String,
    pub method_type: String,
    pub provider: String,
    pub is_default: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PaymentMethodResponse {
    pub fn from_payment_method(method: PaymentMethod) -> Self {
        Self {
            id: method.id,
            user_id: method.user_id,
            method_type: method.method_type,
            provider: method.provider,
            is_default: method.is_default,
            created_at: method.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    pub refund_id: String,
    pub payment_id: String,
    pub amount: f64,
    pub status: RefundStatus,
    pub reason: String,
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub payment_id: String,
    pub provider_payment_id: String,
    pub amount: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPaymentResponse {
    pub provider_payment_id: String,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRefundResponse {
    pub refund_id: String,
}

/// Payment status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
    Refunded,
    PartiallyRefunded,
}

/// Refund status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RefundStatus {
    Pending,
    Completed,
    Failed,
}

/// Payment analytics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAnalytics {
    pub total_volume: f64,
    pub total_count: u64,
    pub average_amount: f64,
    pub total_fees: f64,
    pub success_rate: f64,
    pub by_currency: HashMap<String, CurrencyAnalytics>,
    pub by_provider: HashMap<String, ProviderAnalytics>,
    pub by_method: HashMap<String, MethodAnalytics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyAnalytics {
    pub volume: f64,
    pub count: u64,
    pub average_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAnalytics {
    pub volume: f64,
    pub count: u64,
    pub success_rate: f64,
    pub average_processing_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodAnalytics {
    pub volume: f64,
    pub count: u64,
    pub average_amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_utils::fixtures::PaymentFixture;

    // Mock repository for testing
    struct MockPaymentRepository;

    #[async_trait]
    impl PaymentRepository for MockPaymentRepository {
        async fn create(&self, payment: &Payment) -> Result<Payment, PaymentError> {
            Ok(payment.clone())
        }

        async fn find_by_id(&self, _id: &str) -> Result<Option<Payment>, PaymentError> {
            let fixture = PaymentFixture::generate();
            Ok(Some(Payment {
                id: fixture.id,
                user_id: fixture.user_id,
                amount: fixture.amount,
                currency: fixture.currency,
                fees: 0.0,
                net_amount: fixture.amount,
                payment_method_type: "credit_card".to_string(),
                provider: "stripe".to_string(),
                provider_payment_id: None,
                status: PaymentStatus::Completed,
                description: None,
                metadata: HashMap::new(),
                failure_reason: None,
                refunded_amount: 0.0,
                created_at: fixture.created_at,
                updated_at: fixture.updated_at,
            }))
        }

        async fn update(&self, payment: &Payment) -> Result<Payment, PaymentError> {
            Ok(payment.clone())
        }

        async fn list_with_filters(&self, _pagination: Pagination, _filters: PaymentFilters) -> Result<(Vec<Payment>, u64), PaymentError> {
            Ok((vec![], 0))
        }

        async fn find_by_user(&self, _user_id: &str, _pagination: Pagination) -> Result<(Vec<Payment>, u64), PaymentError> {
            Ok((vec![], 0))
        }

        async fn record_refund(&self, _payment_id: &str, _amount: f64, _reason: &str) -> Result<(), PaymentError> {
            Ok(())
        }

        async fn get_payment_methods(&self, _user_id: &str) -> Result<Vec<PaymentMethod>, PaymentError> {
            Ok(vec![])
        }

        async fn create_payment_method(&self, method: &PaymentMethod) -> Result<PaymentMethod, PaymentError> {
            Ok(method.clone())
        }

        async fn delete_payment_method(&self, _user_id: &str, _method_id: &str) -> Result<(), PaymentError> {
            Ok(())
        }

        async fn get_analytics(&self, _filters: AnalyticsFilters) -> Result<PaymentAnalytics, PaymentError> {
            Ok(PaymentAnalytics {
                total_volume: 0.0,
                total_count: 0,
                average_amount: 0.0,
                total_fees: 0.0,
                success_rate: 0.0,
                by_currency: HashMap::new(),
                by_provider: HashMap::new(),
                by_method: HashMap::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_process_payment() {
        let repository = Arc::new(MockPaymentRepository);
        let service = PaymentServiceImpl::new(repository);

        let request = ProcessPaymentRequest {
            user_id: "user123".to_string(),
            amount: 100.0,
            currency: "USD".to_string(),
            payment_method_type: "credit_card".to_string(),
            provider: "stripe".to_string(),
            description: Some("Test payment".to_string()),
            metadata: HashMap::new(),
        };

        let result = service.process_payment(request).await;
        assert!(result.is_ok());

        let payment_response = result.unwrap();
        assert_eq!(payment_response.amount, 100.0);
        assert_eq!(payment_response.currency, "USD");
    }

    #[tokio::test]
    async fn test_get_payment() {
        let repository = Arc::new(MockPaymentRepository);
        let service = PaymentServiceImpl::new(repository);

        let result = service.get_payment("test_id").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
