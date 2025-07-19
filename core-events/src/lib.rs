// =====================================================================================
// File: core-events/src/lib.rs
// Description: Event-driven architecture core library for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Events Library
//! 
//! This library provides event-driven architecture components including:
//! - Event publishing and subscription
//! - Message queuing
//! - Event sourcing
//! - Distributed transaction support

pub mod event_bus;
pub mod message_queue;
pub mod event_store;
pub mod saga;
pub mod handlers;

// Re-export main types and traits
pub use event_bus::*;
pub use message_queue::*;
pub use event_store::*;
pub use saga::*;
pub use handlers::*;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Event system errors
#[derive(Error, Debug, Clone)]
pub enum EventError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Event not found: {0}")]
    EventNotFound(String),
    
    #[error("Handler not found: {0}")]
    HandlerNotFound(String),
    
    #[error("Publishing failed: {0}")]
    PublishingFailed(String),
    
    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),
    
    #[error("Queue error: {0}")]
    QueueError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
}

/// Result type for event operations
pub type EventResult<T> = Result<T, EventError>;

/// Base event trait that all events must implement
pub trait Event: Send + Sync + fmt::Debug {
    /// Get event type identifier
    fn event_type(&self) -> &str;
    
    /// Get event ID
    fn event_id(&self) -> &str;
    
    /// Get aggregate ID that this event belongs to
    fn aggregate_id(&self) -> &str;
    
    /// Get event timestamp
    fn timestamp(&self) -> DateTime<Utc>;
    
    /// Get event version for optimistic concurrency control
    fn version(&self) -> u64;
    
    /// Serialize event to JSON
    fn to_json(&self) -> EventResult<String>;
    
    /// Get event metadata
    fn metadata(&self) -> &HashMap<String, serde_json::Value>;
}

/// Event envelope for wrapping events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub version: u64,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    pub fn new<T: Event + Serialize>(
        event: &T,
        aggregate_type: String,
        correlation_id: Option<String>,
        causation_id: Option<String>,
        user_id: Option<String>,
    ) -> EventResult<Self> {
        let payload = serde_json::to_value(event)
            .map_err(|e| EventError::SerializationError(e.to_string()))?;

        Ok(Self {
            event_id: event.event_id().to_string(),
            event_type: event.event_type().to_string(),
            aggregate_id: event.aggregate_id().to_string(),
            aggregate_type,
            version: event.version(),
            timestamp: event.timestamp(),
            correlation_id,
            causation_id,
            user_id,
            metadata: event.metadata().clone(),
            payload,
        })
    }

    pub fn deserialize_payload<T: for<'de> Deserialize<'de>>(&self) -> EventResult<T> {
        serde_json::from_value(self.payload.clone())
            .map_err(|e| EventError::DeserializationError(e.to_string()))
    }
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<T: Event>: Send + Sync {
    /// Handle the event
    async fn handle(&self, event: &T, envelope: &EventEnvelope) -> EventResult<()>;
    
    /// Get handler name for identification
    fn handler_name(&self) -> &str;
    
    /// Check if this handler can handle the event type
    fn can_handle(&self, event_type: &str) -> bool;
}

/// Event publisher trait
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single event
    async fn publish<T: Event + Serialize>(&self, event: &T, context: PublishContext) -> EventResult<()>;
    
    /// Publish multiple events in a batch
    async fn publish_batch<T: Event + Serialize>(&self, events: &[T], context: PublishContext) -> EventResult<()>;
    
    /// Publish event and wait for acknowledgment
    async fn publish_and_wait<T: Event + Serialize>(&self, event: &T, context: PublishContext) -> EventResult<()>;
}

/// Event subscriber trait
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Subscribe to events of a specific type
    async fn subscribe(&self, event_type: &str, handler: Box<dyn EventHandler<dyn Event>>) -> EventResult<String>;
    
    /// Unsubscribe from events
    async fn unsubscribe(&self, subscription_id: &str) -> EventResult<()>;
    
    /// Start processing events
    async fn start_processing(&self) -> EventResult<()>;
    
    /// Stop processing events
    async fn stop_processing(&self) -> EventResult<()>;
}

/// Publishing context for events
#[derive(Debug, Clone)]
pub struct PublishContext {
    pub aggregate_type: String,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub user_id: Option<String>,
    pub routing_key: Option<String>,
    pub delay: Option<std::time::Duration>,
    pub retry_policy: Option<RetryPolicy>,
}

impl PublishContext {
    pub fn new(aggregate_type: String) -> Self {
        Self {
            aggregate_type,
            correlation_id: None,
            causation_id: None,
            user_id: None,
            routing_key: None,
            delay: None,
            retry_policy: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_causation_id(mut self, causation_id: String) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_routing_key(mut self, routing_key: String) -> Self {
        self.routing_key = Some(routing_key);
        self
    }

    pub fn with_delay(mut self, delay: std::time::Duration) -> Self {
        self.delay = Some(delay);
        self
    }

    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }
}

/// Retry policy for failed event processing
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Event processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
    DeadLetter,
}

/// Event processing result
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub status: ProcessingStatus,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub processed_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
}

impl ProcessingResult {
    pub fn success() -> Self {
        Self {
            status: ProcessingStatus::Completed,
            attempts: 1,
            last_error: None,
            processed_at: Some(Utc::now()),
            next_retry_at: None,
        }
    }

    pub fn failure(error: String, attempts: u32) -> Self {
        Self {
            status: ProcessingStatus::Failed,
            attempts,
            last_error: Some(error),
            processed_at: Some(Utc::now()),
            next_retry_at: None,
        }
    }

    pub fn retry(error: String, attempts: u32, next_retry_at: DateTime<Utc>) -> Self {
        Self {
            status: ProcessingStatus::Retrying,
            attempts,
            last_error: Some(error),
            processed_at: Some(Utc::now()),
            next_retry_at: Some(next_retry_at),
        }
    }
}

/// Domain events for the RWA platform
pub mod domain_events {
    use super::*;

    /// User domain events
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserRegistered {
        pub event_id: String,
        pub user_id: String,
        pub email: String,
        pub first_name: String,
        pub last_name: String,
        pub timestamp: DateTime<Utc>,
        pub version: u64,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    impl Event for UserRegistered {
        fn event_type(&self) -> &str { "UserRegistered" }
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.user_id }
        fn timestamp(&self) -> DateTime<Utc> { self.timestamp }
        fn version(&self) -> u64 { self.version }
        fn metadata(&self) -> &HashMap<String, serde_json::Value> { &self.metadata }
        
        fn to_json(&self) -> EventResult<String> {
            serde_json::to_string(self)
                .map_err(|e| EventError::SerializationError(e.to_string()))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserEmailVerified {
        pub event_id: String,
        pub user_id: String,
        pub email: String,
        pub verified_at: DateTime<Utc>,
        pub timestamp: DateTime<Utc>,
        pub version: u64,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    impl Event for UserEmailVerified {
        fn event_type(&self) -> &str { "UserEmailVerified" }
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.user_id }
        fn timestamp(&self) -> DateTime<Utc> { self.timestamp }
        fn version(&self) -> u64 { self.version }
        fn metadata(&self) -> &HashMap<String, serde_json::Value> { &self.metadata }
        
        fn to_json(&self) -> EventResult<String> {
            serde_json::to_string(self)
                .map_err(|e| EventError::SerializationError(e.to_string()))
        }
    }

    /// Asset domain events
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AssetCreated {
        pub event_id: String,
        pub asset_id: String,
        pub name: String,
        pub asset_type: String,
        pub total_value: f64,
        pub currency: String,
        pub owner_id: String,
        pub timestamp: DateTime<Utc>,
        pub version: u64,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    impl Event for AssetCreated {
        fn event_type(&self) -> &str { "AssetCreated" }
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.asset_id }
        fn timestamp(&self) -> DateTime<Utc> { self.timestamp }
        fn version(&self) -> u64 { self.version }
        fn metadata(&self) -> &HashMap<String, serde_json::Value> { &self.metadata }
        
        fn to_json(&self) -> EventResult<String> {
            serde_json::to_string(self)
                .map_err(|e| EventError::SerializationError(e.to_string()))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AssetTokenized {
        pub event_id: String,
        pub asset_id: String,
        pub token_address: String,
        pub blockchain_network: String,
        pub token_supply: u64,
        pub token_symbol: String,
        pub timestamp: DateTime<Utc>,
        pub version: u64,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    impl Event for AssetTokenized {
        fn event_type(&self) -> &str { "AssetTokenized" }
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.asset_id }
        fn timestamp(&self) -> DateTime<Utc> { self.timestamp }
        fn version(&self) -> u64 { self.version }
        fn metadata(&self) -> &HashMap<String, serde_json::Value> { &self.metadata }
        
        fn to_json(&self) -> EventResult<String> {
            serde_json::to_string(self)
                .map_err(|e| EventError::SerializationError(e.to_string()))
        }
    }

    /// Payment domain events
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PaymentProcessed {
        pub event_id: String,
        pub payment_id: String,
        pub user_id: String,
        pub amount: f64,
        pub currency: String,
        pub payment_method: String,
        pub provider: String,
        pub status: String,
        pub timestamp: DateTime<Utc>,
        pub version: u64,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    impl Event for PaymentProcessed {
        fn event_type(&self) -> &str { "PaymentProcessed" }
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.payment_id }
        fn timestamp(&self) -> DateTime<Utc> { self.timestamp }
        fn version(&self) -> u64 { self.version }
        fn metadata(&self) -> &HashMap<String, serde_json::Value> { &self.metadata }
        
        fn to_json(&self) -> EventResult<String> {
            serde_json::to_string(self)
                .map_err(|e| EventError::SerializationError(e.to_string()))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PaymentFailed {
        pub event_id: String,
        pub payment_id: String,
        pub user_id: String,
        pub amount: f64,
        pub currency: String,
        pub failure_reason: String,
        pub timestamp: DateTime<Utc>,
        pub version: u64,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    impl Event for PaymentFailed {
        fn event_type(&self) -> &str { "PaymentFailed" }
        fn event_id(&self) -> &str { &self.event_id }
        fn aggregate_id(&self) -> &str { &self.payment_id }
        fn timestamp(&self) -> DateTime<Utc> { self.timestamp }
        fn version(&self) -> u64 { self.version }
        fn metadata(&self) -> &HashMap<String, serde_json::Value> { &self.metadata }
        
        fn to_json(&self) -> EventResult<String> {
            serde_json::to_string(self)
                .map_err(|e| EventError::SerializationError(e.to_string()))
        }
    }
}

/// Event factory for creating domain events
pub struct EventFactory;

impl EventFactory {
    pub fn user_registered(
        user_id: String,
        email: String,
        first_name: String,
        last_name: String,
        version: u64,
    ) -> domain_events::UserRegistered {
        domain_events::UserRegistered {
            event_id: Uuid::new_v4().to_string(),
            user_id,
            email,
            first_name,
            last_name,
            timestamp: Utc::now(),
            version,
            metadata: HashMap::new(),
        }
    }

    pub fn user_email_verified(
        user_id: String,
        email: String,
        version: u64,
    ) -> domain_events::UserEmailVerified {
        domain_events::UserEmailVerified {
            event_id: Uuid::new_v4().to_string(),
            user_id,
            email,
            verified_at: Utc::now(),
            timestamp: Utc::now(),
            version,
            metadata: HashMap::new(),
        }
    }

    pub fn asset_created(
        asset_id: String,
        name: String,
        asset_type: String,
        total_value: f64,
        currency: String,
        owner_id: String,
        version: u64,
    ) -> domain_events::AssetCreated {
        domain_events::AssetCreated {
            event_id: Uuid::new_v4().to_string(),
            asset_id,
            name,
            asset_type,
            total_value,
            currency,
            owner_id,
            timestamp: Utc::now(),
            version,
            metadata: HashMap::new(),
        }
    }

    pub fn asset_tokenized(
        asset_id: String,
        token_address: String,
        blockchain_network: String,
        token_supply: u64,
        token_symbol: String,
        version: u64,
    ) -> domain_events::AssetTokenized {
        domain_events::AssetTokenized {
            event_id: Uuid::new_v4().to_string(),
            asset_id,
            token_address,
            blockchain_network,
            token_supply,
            token_symbol,
            timestamp: Utc::now(),
            version,
            metadata: HashMap::new(),
        }
    }

    pub fn payment_processed(
        payment_id: String,
        user_id: String,
        amount: f64,
        currency: String,
        payment_method: String,
        provider: String,
        status: String,
        version: u64,
    ) -> domain_events::PaymentProcessed {
        domain_events::PaymentProcessed {
            event_id: Uuid::new_v4().to_string(),
            payment_id,
            user_id,
            amount,
            currency,
            payment_method,
            provider,
            status,
            timestamp: Utc::now(),
            version,
            metadata: HashMap::new(),
        }
    }

    pub fn payment_failed(
        payment_id: String,
        user_id: String,
        amount: f64,
        currency: String,
        failure_reason: String,
        version: u64,
    ) -> domain_events::PaymentFailed {
        domain_events::PaymentFailed {
            event_id: Uuid::new_v4().to_string(),
            payment_id,
            user_id,
            amount,
            currency,
            failure_reason,
            timestamp: Utc::now(),
            version,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_envelope_creation() {
        let event = EventFactory::user_registered(
            "user123".to_string(),
            "test@example.com".to_string(),
            "John".to_string(),
            "Doe".to_string(),
            1,
        );

        let envelope = EventEnvelope::new(
            &event,
            "User".to_string(),
            Some("correlation123".to_string()),
            None,
            Some("admin".to_string()),
        ).unwrap();

        assert_eq!(envelope.event_type, "UserRegistered");
        assert_eq!(envelope.aggregate_id, "user123");
        assert_eq!(envelope.aggregate_type, "User");
        assert_eq!(envelope.correlation_id, Some("correlation123".to_string()));
        assert_eq!(envelope.user_id, Some("admin".to_string()));
    }

    #[test]
    fn test_event_serialization() {
        let event = EventFactory::asset_created(
            "asset123".to_string(),
            "Test Asset".to_string(),
            "real_estate".to_string(),
            1000000.0,
            "USD".to_string(),
            "user123".to_string(),
            1,
        );

        let json = event.to_json().unwrap();
        assert!(json.contains("AssetCreated"));
        assert!(json.contains("asset123"));
        assert!(json.contains("Test Asset"));
    }

    #[test]
    fn test_publish_context_builder() {
        let context = PublishContext::new("User".to_string())
            .with_correlation_id("corr123".to_string())
            .with_user_id("user456".to_string())
            .with_delay(std::time::Duration::from_secs(5));

        assert_eq!(context.aggregate_type, "User");
        assert_eq!(context.correlation_id, Some("corr123".to_string()));
        assert_eq!(context.user_id, Some("user456".to_string()));
        assert_eq!(context.delay, Some(std::time::Duration::from_secs(5)));
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_delay, std::time::Duration::from_millis(100));
        assert_eq!(policy.backoff_multiplier, 2.0);
        assert!(policy.jitter);
    }

    #[test]
    fn test_processing_result() {
        let success = ProcessingResult::success();
        assert_eq!(success.status, ProcessingStatus::Completed);
        assert_eq!(success.attempts, 1);
        assert!(success.last_error.is_none());

        let failure = ProcessingResult::failure("Test error".to_string(), 2);
        assert_eq!(failure.status, ProcessingStatus::Failed);
        assert_eq!(failure.attempts, 2);
        assert_eq!(failure.last_error, Some("Test error".to_string()));
    }
}
