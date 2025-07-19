// =====================================================================================
// File: core-events/src/handlers.rs
// Description: Event handlers for domain events
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{Event, EventEnvelope, EventHandler, EventError, EventResult, domain_events::*};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Email service trait for sending notifications
#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_welcome_email(&self, email: &str, name: &str) -> Result<(), String>;
    async fn send_verification_email(&self, email: &str, verification_code: &str) -> Result<(), String>;
    async fn send_payment_confirmation(&self, email: &str, amount: f64, currency: &str) -> Result<(), String>;
    async fn send_asset_notification(&self, email: &str, asset_name: &str, action: &str) -> Result<(), String>;
}

/// Notification service trait for various notification channels
#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_push_notification(&self, user_id: &str, title: &str, message: &str) -> Result<(), String>;
    async fn send_sms_notification(&self, phone: &str, message: &str) -> Result<(), String>;
    async fn create_in_app_notification(&self, user_id: &str, title: &str, message: &str) -> Result<(), String>;
}

/// Analytics service trait for tracking events
#[async_trait]
pub trait AnalyticsService: Send + Sync {
    async fn track_user_event(&self, user_id: &str, event_name: &str, properties: serde_json::Value) -> Result<(), String>;
    async fn track_asset_event(&self, asset_id: &str, event_name: &str, properties: serde_json::Value) -> Result<(), String>;
    async fn track_payment_event(&self, payment_id: &str, event_name: &str, properties: serde_json::Value) -> Result<(), String>;
}

/// User registration event handler
pub struct UserRegistrationHandler {
    email_service: Arc<dyn EmailService>,
    notification_service: Arc<dyn NotificationService>,
    analytics_service: Arc<dyn AnalyticsService>,
}

impl UserRegistrationHandler {
    pub fn new(
        email_service: Arc<dyn EmailService>,
        notification_service: Arc<dyn NotificationService>,
        analytics_service: Arc<dyn AnalyticsService>,
    ) -> Self {
        Self {
            email_service,
            notification_service,
            analytics_service,
        }
    }
}

#[async_trait]
impl EventHandler<dyn Event> for UserRegistrationHandler {
    async fn handle(&self, event: &dyn Event, envelope: &EventEnvelope) -> EventResult<()> {
        if let Ok(user_registered) = serde_json::from_value::<UserRegistered>(envelope.payload.clone()) {
            info!(
                user_id = %user_registered.user_id,
                email = %user_registered.email,
                "Handling user registration event"
            );

            // Send welcome email
            if let Err(e) = self.email_service.send_welcome_email(
                &user_registered.email,
                &format!("{} {}", user_registered.first_name, user_registered.last_name),
            ).await {
                error!(
                    user_id = %user_registered.user_id,
                    error = %e,
                    "Failed to send welcome email"
                );
            }

            // Create in-app notification
            if let Err(e) = self.notification_service.create_in_app_notification(
                &user_registered.user_id,
                "Welcome to RWA Platform",
                "Your account has been created successfully. Please verify your email address.",
            ).await {
                error!(
                    user_id = %user_registered.user_id,
                    error = %e,
                    "Failed to create in-app notification"
                );
            }

            // Track analytics event
            let properties = serde_json::json!({
                "email": user_registered.email,
                "first_name": user_registered.first_name,
                "last_name": user_registered.last_name,
                "registration_timestamp": user_registered.timestamp
            });

            if let Err(e) = self.analytics_service.track_user_event(
                &user_registered.user_id,
                "user_registered",
                properties,
            ).await {
                error!(
                    user_id = %user_registered.user_id,
                    error = %e,
                    "Failed to track analytics event"
                );
            }

            info!(
                user_id = %user_registered.user_id,
                "User registration event handled successfully"
            );

            Ok(())
        } else {
            Err(EventError::DeserializationError("Failed to deserialize UserRegistered event".to_string()))
        }
    }

    fn handler_name(&self) -> &str {
        "UserRegistrationHandler"
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "UserRegistered"
    }
}

/// User email verification handler
pub struct UserEmailVerificationHandler {
    email_service: Arc<dyn EmailService>,
    analytics_service: Arc<dyn AnalyticsService>,
}

impl UserEmailVerificationHandler {
    pub fn new(
        email_service: Arc<dyn EmailService>,
        analytics_service: Arc<dyn AnalyticsService>,
    ) -> Self {
        Self {
            email_service,
            analytics_service,
        }
    }
}

#[async_trait]
impl EventHandler<dyn Event> for UserEmailVerificationHandler {
    async fn handle(&self, event: &dyn Event, envelope: &EventEnvelope) -> EventResult<()> {
        if let Ok(email_verified) = serde_json::from_value::<UserEmailVerified>(envelope.payload.clone()) {
            info!(
                user_id = %email_verified.user_id,
                email = %email_verified.email,
                "Handling user email verification event"
            );

            // Track analytics event
            let properties = serde_json::json!({
                "email": email_verified.email,
                "verified_at": email_verified.verified_at
            });

            if let Err(e) = self.analytics_service.track_user_event(
                &email_verified.user_id,
                "email_verified",
                properties,
            ).await {
                error!(
                    user_id = %email_verified.user_id,
                    error = %e,
                    "Failed to track analytics event"
                );
            }

            Ok(())
        } else {
            Err(EventError::DeserializationError("Failed to deserialize UserEmailVerified event".to_string()))
        }
    }

    fn handler_name(&self) -> &str {
        "UserEmailVerificationHandler"
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "UserEmailVerified"
    }
}

/// Asset creation handler
pub struct AssetCreationHandler {
    notification_service: Arc<dyn NotificationService>,
    analytics_service: Arc<dyn AnalyticsService>,
}

impl AssetCreationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationService>,
        analytics_service: Arc<dyn AnalyticsService>,
    ) -> Self {
        Self {
            notification_service,
            analytics_service,
        }
    }
}

#[async_trait]
impl EventHandler<dyn Event> for AssetCreationHandler {
    async fn handle(&self, event: &dyn Event, envelope: &EventEnvelope) -> EventResult<()> {
        if let Ok(asset_created) = serde_json::from_value::<AssetCreated>(envelope.payload.clone()) {
            info!(
                asset_id = %asset_created.asset_id,
                owner_id = %asset_created.owner_id,
                "Handling asset creation event"
            );

            // Create notification for asset owner
            if let Err(e) = self.notification_service.create_in_app_notification(
                &asset_created.owner_id,
                "Asset Created",
                &format!("Your asset '{}' has been created successfully.", asset_created.name),
            ).await {
                error!(
                    asset_id = %asset_created.asset_id,
                    error = %e,
                    "Failed to create notification"
                );
            }

            // Track analytics event
            let properties = serde_json::json!({
                "asset_name": asset_created.name,
                "asset_type": asset_created.asset_type,
                "total_value": asset_created.total_value,
                "currency": asset_created.currency,
                "owner_id": asset_created.owner_id
            });

            if let Err(e) = self.analytics_service.track_asset_event(
                &asset_created.asset_id,
                "asset_created",
                properties,
            ).await {
                error!(
                    asset_id = %asset_created.asset_id,
                    error = %e,
                    "Failed to track analytics event"
                );
            }

            Ok(())
        } else {
            Err(EventError::DeserializationError("Failed to deserialize AssetCreated event".to_string()))
        }
    }

    fn handler_name(&self) -> &str {
        "AssetCreationHandler"
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "AssetCreated"
    }
}

/// Asset tokenization handler
pub struct AssetTokenizationHandler {
    notification_service: Arc<dyn NotificationService>,
    analytics_service: Arc<dyn AnalyticsService>,
}

impl AssetTokenizationHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationService>,
        analytics_service: Arc<dyn AnalyticsService>,
    ) -> Self {
        Self {
            notification_service,
            analytics_service,
        }
    }
}

#[async_trait]
impl EventHandler<dyn Event> for AssetTokenizationHandler {
    async fn handle(&self, event: &dyn Event, envelope: &EventEnvelope) -> EventResult<()> {
        if let Ok(asset_tokenized) = serde_json::from_value::<AssetTokenized>(envelope.payload.clone()) {
            info!(
                asset_id = %asset_tokenized.asset_id,
                token_address = %asset_tokenized.token_address,
                "Handling asset tokenization event"
            );

            // Track analytics event
            let properties = serde_json::json!({
                "token_address": asset_tokenized.token_address,
                "blockchain_network": asset_tokenized.blockchain_network,
                "token_supply": asset_tokenized.token_supply,
                "token_symbol": asset_tokenized.token_symbol
            });

            if let Err(e) = self.analytics_service.track_asset_event(
                &asset_tokenized.asset_id,
                "asset_tokenized",
                properties,
            ).await {
                error!(
                    asset_id = %asset_tokenized.asset_id,
                    error = %e,
                    "Failed to track analytics event"
                );
            }

            Ok(())
        } else {
            Err(EventError::DeserializationError("Failed to deserialize AssetTokenized event".to_string()))
        }
    }

    fn handler_name(&self) -> &str {
        "AssetTokenizationHandler"
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "AssetTokenized"
    }
}

/// Payment processing handler
pub struct PaymentProcessingHandler {
    email_service: Arc<dyn EmailService>,
    notification_service: Arc<dyn NotificationService>,
    analytics_service: Arc<dyn AnalyticsService>,
}

impl PaymentProcessingHandler {
    pub fn new(
        email_service: Arc<dyn EmailService>,
        notification_service: Arc<dyn NotificationService>,
        analytics_service: Arc<dyn AnalyticsService>,
    ) -> Self {
        Self {
            email_service,
            notification_service,
            analytics_service,
        }
    }
}

#[async_trait]
impl EventHandler<dyn Event> for PaymentProcessingHandler {
    async fn handle(&self, event: &dyn Event, envelope: &EventEnvelope) -> EventResult<()> {
        if let Ok(payment_processed) = serde_json::from_value::<PaymentProcessed>(envelope.payload.clone()) {
            info!(
                payment_id = %payment_processed.payment_id,
                user_id = %payment_processed.user_id,
                amount = %payment_processed.amount,
                "Handling payment processing event"
            );

            // Send payment confirmation email (simplified - would need user email)
            // In a real implementation, you would fetch user details first

            // Create notification
            if let Err(e) = self.notification_service.create_in_app_notification(
                &payment_processed.user_id,
                "Payment Processed",
                &format!("Your payment of {} {} has been processed successfully.", 
                    payment_processed.amount, payment_processed.currency),
            ).await {
                error!(
                    payment_id = %payment_processed.payment_id,
                    error = %e,
                    "Failed to create notification"
                );
            }

            // Track analytics event
            let properties = serde_json::json!({
                "amount": payment_processed.amount,
                "currency": payment_processed.currency,
                "payment_method": payment_processed.payment_method,
                "provider": payment_processed.provider,
                "status": payment_processed.status
            });

            if let Err(e) = self.analytics_service.track_payment_event(
                &payment_processed.payment_id,
                "payment_processed",
                properties,
            ).await {
                error!(
                    payment_id = %payment_processed.payment_id,
                    error = %e,
                    "Failed to track analytics event"
                );
            }

            Ok(())
        } else {
            Err(EventError::DeserializationError("Failed to deserialize PaymentProcessed event".to_string()))
        }
    }

    fn handler_name(&self) -> &str {
        "PaymentProcessingHandler"
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "PaymentProcessed"
    }
}

/// Payment failure handler
pub struct PaymentFailureHandler {
    notification_service: Arc<dyn NotificationService>,
    analytics_service: Arc<dyn AnalyticsService>,
}

impl PaymentFailureHandler {
    pub fn new(
        notification_service: Arc<dyn NotificationService>,
        analytics_service: Arc<dyn AnalyticsService>,
    ) -> Self {
        Self {
            notification_service,
            analytics_service,
        }
    }
}

#[async_trait]
impl EventHandler<dyn Event> for PaymentFailureHandler {
    async fn handle(&self, event: &dyn Event, envelope: &EventEnvelope) -> EventResult<()> {
        if let Ok(payment_failed) = serde_json::from_value::<PaymentFailed>(envelope.payload.clone()) {
            warn!(
                payment_id = %payment_failed.payment_id,
                user_id = %payment_failed.user_id,
                failure_reason = %payment_failed.failure_reason,
                "Handling payment failure event"
            );

            // Create notification
            if let Err(e) = self.notification_service.create_in_app_notification(
                &payment_failed.user_id,
                "Payment Failed",
                &format!("Your payment of {} {} failed: {}", 
                    payment_failed.amount, payment_failed.currency, payment_failed.failure_reason),
            ).await {
                error!(
                    payment_id = %payment_failed.payment_id,
                    error = %e,
                    "Failed to create notification"
                );
            }

            // Track analytics event
            let properties = serde_json::json!({
                "amount": payment_failed.amount,
                "currency": payment_failed.currency,
                "failure_reason": payment_failed.failure_reason
            });

            if let Err(e) = self.analytics_service.track_payment_event(
                &payment_failed.payment_id,
                "payment_failed",
                properties,
            ).await {
                error!(
                    payment_id = %payment_failed.payment_id,
                    error = %e,
                    "Failed to track analytics event"
                );
            }

            Ok(())
        } else {
            Err(EventError::DeserializationError("Failed to deserialize PaymentFailed event".to_string()))
        }
    }

    fn handler_name(&self) -> &str {
        "PaymentFailureHandler"
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "PaymentFailed"
    }
}

/// Mock implementations for testing
pub mod mock {
    use super::*;

    pub struct MockEmailService;

    #[async_trait]
    impl EmailService for MockEmailService {
        async fn send_welcome_email(&self, email: &str, name: &str) -> Result<(), String> {
            debug!(email = %email, name = %name, "Mock: Sending welcome email");
            Ok(())
        }

        async fn send_verification_email(&self, email: &str, verification_code: &str) -> Result<(), String> {
            debug!(email = %email, code = %verification_code, "Mock: Sending verification email");
            Ok(())
        }

        async fn send_payment_confirmation(&self, email: &str, amount: f64, currency: &str) -> Result<(), String> {
            debug!(email = %email, amount = %amount, currency = %currency, "Mock: Sending payment confirmation");
            Ok(())
        }

        async fn send_asset_notification(&self, email: &str, asset_name: &str, action: &str) -> Result<(), String> {
            debug!(email = %email, asset_name = %asset_name, action = %action, "Mock: Sending asset notification");
            Ok(())
        }
    }

    pub struct MockNotificationService;

    #[async_trait]
    impl NotificationService for MockNotificationService {
        async fn send_push_notification(&self, user_id: &str, title: &str, message: &str) -> Result<(), String> {
            debug!(user_id = %user_id, title = %title, message = %message, "Mock: Sending push notification");
            Ok(())
        }

        async fn send_sms_notification(&self, phone: &str, message: &str) -> Result<(), String> {
            debug!(phone = %phone, message = %message, "Mock: Sending SMS notification");
            Ok(())
        }

        async fn create_in_app_notification(&self, user_id: &str, title: &str, message: &str) -> Result<(), String> {
            debug!(user_id = %user_id, title = %title, message = %message, "Mock: Creating in-app notification");
            Ok(())
        }
    }

    pub struct MockAnalyticsService;

    #[async_trait]
    impl AnalyticsService for MockAnalyticsService {
        async fn track_user_event(&self, user_id: &str, event_name: &str, properties: serde_json::Value) -> Result<(), String> {
            debug!(user_id = %user_id, event_name = %event_name, "Mock: Tracking user event");
            Ok(())
        }

        async fn track_asset_event(&self, asset_id: &str, event_name: &str, properties: serde_json::Value) -> Result<(), String> {
            debug!(asset_id = %asset_id, event_name = %event_name, "Mock: Tracking asset event");
            Ok(())
        }

        async fn track_payment_event(&self, payment_id: &str, event_name: &str, properties: serde_json::Value) -> Result<(), String> {
            debug!(payment_id = %payment_id, event_name = %event_name, "Mock: Tracking payment event");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::*;
    use crate::EventFactory;

    #[tokio::test]
    async fn test_user_registration_handler() {
        let email_service = Arc::new(MockEmailService);
        let notification_service = Arc::new(MockNotificationService);
        let analytics_service = Arc::new(MockAnalyticsService);

        let handler = UserRegistrationHandler::new(email_service, notification_service, analytics_service);

        let event = EventFactory::user_registered(
            "user123".to_string(),
            "test@example.com".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            1,
        );

        let envelope = EventEnvelope::new(&event, "User".to_string(), None, None, None).unwrap();

        let result = handler.handle(&event, &envelope).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_asset_creation_handler() {
        let notification_service = Arc::new(MockNotificationService);
        let analytics_service = Arc::new(MockAnalyticsService);

        let handler = AssetCreationHandler::new(notification_service, analytics_service);

        let event = EventFactory::asset_created(
            "asset123".to_string(),
            "Test Asset".to_string(),
            "real_estate".to_string(),
            1000000.0,
            "USD".to_string(),
            "user123".to_string(),
            1,
        );

        let envelope = EventEnvelope::new(&event, "Asset".to_string(), None, None, None).unwrap();

        let result = handler.handle(&event, &envelope).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_payment_processing_handler() {
        let email_service = Arc::new(MockEmailService);
        let notification_service = Arc::new(MockNotificationService);
        let analytics_service = Arc::new(MockAnalyticsService);

        let handler = PaymentProcessingHandler::new(email_service, notification_service, analytics_service);

        let event = EventFactory::payment_processed(
            "payment123".to_string(),
            "user123".to_string(),
            100.0,
            "USD".to_string(),
            "credit_card".to_string(),
            "stripe".to_string(),
            "completed".to_string(),
            1,
        );

        let envelope = EventEnvelope::new(&event, "Payment".to_string(), None, None, None).unwrap();

        let result = handler.handle(&event, &envelope).await;
        assert!(result.is_ok());
    }
}
