// =====================================================================================
// File: core-events/src/event_bus.rs
// Description: Event bus implementation for distributed event handling
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    Event, EventEnvelope, EventHandler, EventPublisher, EventSubscriber, EventError, EventResult,
    PublishContext, ProcessingResult, ProcessingStatus, RetryPolicy,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::{sleep, Duration, Instant};
use fastrand;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// In-memory event bus implementation
pub struct InMemoryEventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler<dyn Event>>>>>>,
    subscriptions: Arc<RwLock<HashMap<String, SubscriptionInfo>>>,
    event_channel: Arc<Mutex<Option<mpsc::UnboundedSender<EventMessage>>>>,
    processing_stats: Arc<RwLock<ProcessingStats>>,
    config: EventBusConfig,
}

impl InMemoryEventBus {
    pub fn new(config: EventBusConfig) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_channel: Arc::new(Mutex::new(None)),
            processing_stats: Arc::new(RwLock::new(ProcessingStats::default())),
            config,
        }
    }

    /// Start the event processing loop
    pub async fn start(&self) -> EventResult<()> {
        let (sender, mut receiver) = mpsc::unbounded_channel::<EventMessage>();
        
        // Store the sender for publishing events
        {
            let mut channel = self.event_channel.lock().await;
            *channel = Some(sender);
        }

        // Clone necessary data for the processing task
        let handlers = Arc::clone(&self.handlers);
        let stats = Arc::clone(&self.processing_stats);
        let config = self.config.clone();

        // Start event processing task
        tokio::spawn(async move {
            info!("Event bus processing started");
            
            while let Some(event_message) = receiver.recv().await {
                Self::process_event_message(event_message, &handlers, &stats, &config).await;
            }
            
            warn!("Event bus processing stopped");
        });

        Ok(())
    }

    /// Process a single event message
    async fn process_event_message(
        event_message: EventMessage,
        handlers: &Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler<dyn Event>>>>>>,
        stats: &Arc<RwLock<ProcessingStats>>,
        config: &EventBusConfig,
    ) {
        let start_time = Instant::now();
        let event_type = &event_message.envelope.event_type;
        
        debug!(
            event_id = %event_message.envelope.event_id,
            event_type = %event_type,
            "Processing event"
        );

        // Get handlers for this event type
        let event_handlers = {
            let handlers_map = handlers.read().await;
            handlers_map.get(event_type).cloned().unwrap_or_default()
        };

        if event_handlers.is_empty() {
            warn!(
                event_type = %event_type,
                "No handlers registered for event type"
            );
            return;
        }

        // Process event with each handler
        for handler in event_handlers {
            let handler_start = Instant::now();
            
            match Self::process_with_handler(&handler, &event_message, config).await {
                Ok(_) => {
                    let processing_time = handler_start.elapsed();
                    debug!(
                        event_id = %event_message.envelope.event_id,
                        handler = %handler.handler_name(),
                        processing_time_ms = %processing_time.as_millis(),
                        "Event processed successfully"
                    );
                    
                    // Update stats
                    let mut stats_guard = stats.write().await;
                    stats_guard.total_processed += 1;
                    stats_guard.successful_processed += 1;
                    stats_guard.total_processing_time += processing_time;
                }
                Err(e) => {
                    error!(
                        event_id = %event_message.envelope.event_id,
                        handler = %handler.handler_name(),
                        error = %e,
                        "Event processing failed"
                    );
                    
                    // Update stats
                    let mut stats_guard = stats.write().await;
                    stats_guard.total_processed += 1;
                    stats_guard.failed_processed += 1;
                }
            }
        }

        let total_time = start_time.elapsed();
        debug!(
            event_id = %event_message.envelope.event_id,
            total_time_ms = %total_time.as_millis(),
            handlers_count = %event_handlers.len(),
            "Event processing completed"
        );
    }

    /// Process event with a specific handler, including retry logic
    async fn process_with_handler(
        handler: &Arc<dyn EventHandler<dyn Event>>,
        event_message: &EventMessage,
        config: &EventBusConfig,
    ) -> EventResult<()> {
        let retry_policy = event_message.retry_policy.as_ref()
            .unwrap_or(&config.default_retry_policy);

        let mut attempts = 0;
        let mut last_error = None;

        while attempts < retry_policy.max_attempts {
            attempts += 1;
            
            match Self::execute_handler(handler, event_message).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    if attempts < retry_policy.max_attempts {
                        let delay = Self::calculate_retry_delay(retry_policy, attempts);
                        
                        warn!(
                            event_id = %event_message.envelope.event_id,
                            handler = %handler.handler_name(),
                            attempt = %attempts,
                            delay_ms = %delay.as_millis(),
                            error = %e,
                            "Handler failed, retrying"
                        );
                        
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| EventError::ProcessingError("Unknown error".to_string())))
    }

    /// Execute handler without retry logic
    async fn execute_handler(
        handler: &Arc<dyn EventHandler<dyn Event>>,
        event_message: &EventMessage,
    ) -> EventResult<()> {
        // Create a dummy event for the handler
        // In a real implementation, you would deserialize the actual event
        let dummy_event = DummyEvent {
            event_id: event_message.envelope.event_id.clone(),
            event_type: event_message.envelope.event_type.clone(),
            aggregate_id: event_message.envelope.aggregate_id.clone(),
            timestamp: event_message.envelope.timestamp,
            version: event_message.envelope.version,
            metadata: event_message.envelope.metadata.clone(),
        };

        handler.handle(&dummy_event, &event_message.envelope).await
    }

    /// Calculate retry delay with exponential backoff
    fn calculate_retry_delay(retry_policy: &RetryPolicy, attempt: u32) -> Duration {
        let base_delay = retry_policy.initial_delay.as_millis() as f64;
        let multiplier = retry_policy.backoff_multiplier.powi((attempt - 1) as i32);
        let delay_ms = (base_delay * multiplier) as u64;
        
        let delay = Duration::from_millis(delay_ms.min(retry_policy.max_delay.as_millis() as u64));
        
        if retry_policy.jitter {
            let jitter = fastrand::f64() * 0.1; // 10% jitter
            let jitter_ms = (delay.as_millis() as f64 * jitter) as u64;
            delay + Duration::from_millis(jitter_ms)
        } else {
            delay
        }
    }

    /// Get processing statistics
    pub async fn get_stats(&self) -> ProcessingStats {
        self.processing_stats.read().await.clone()
    }

    /// Reset processing statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.processing_stats.write().await;
        *stats = ProcessingStats::default();
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventBus {
    async fn publish<T: Event + Serialize>(&self, event: &T, context: PublishContext) -> EventResult<()> {
        let envelope = EventEnvelope::new(
            event,
            context.aggregate_type,
            context.correlation_id,
            context.causation_id,
            context.user_id,
        )?;

        let event_message = EventMessage {
            envelope,
            retry_policy: context.retry_policy,
            published_at: chrono::Utc::now(),
        };

        // Send to processing channel
        let channel = self.event_channel.lock().await;
        if let Some(sender) = channel.as_ref() {
            sender.send(event_message)
                .map_err(|e| EventError::PublishingFailed(e.to_string()))?;
        } else {
            return Err(EventError::PublishingFailed("Event bus not started".to_string()));
        }

        debug!(
            event_id = %event.event_id(),
            event_type = %event.event_type(),
            "Event published successfully"
        );

        Ok(())
    }

    async fn publish_batch<T: Event + Serialize>(&self, events: &[T], context: PublishContext) -> EventResult<()> {
        for event in events {
            self.publish(event, context.clone()).await?;
        }
        Ok(())
    }

    async fn publish_and_wait<T: Event + Serialize>(&self, event: &T, context: PublishContext) -> EventResult<()> {
        // For in-memory implementation, this is the same as regular publish
        // In a real distributed system, this would wait for acknowledgments
        self.publish(event, context).await
    }
}

#[async_trait]
impl EventSubscriber for InMemoryEventBus {
    async fn subscribe(&self, event_type: &str, handler: Box<dyn EventHandler<dyn Event>>) -> EventResult<String> {
        let subscription_id = Uuid::new_v4().to_string();
        let handler_arc = Arc::from(handler);

        // Add handler to the handlers map
        {
            let mut handlers = self.handlers.write().await;
            handlers.entry(event_type.to_string())
                .or_insert_with(Vec::new)
                .push(handler_arc.clone());
        }

        // Store subscription info
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), SubscriptionInfo {
                event_type: event_type.to_string(),
                handler_name: handler_arc.handler_name().to_string(),
                subscribed_at: chrono::Utc::now(),
            });
        }

        info!(
            subscription_id = %subscription_id,
            event_type = %event_type,
            handler = %handler_arc.handler_name(),
            "Event subscription created"
        );

        Ok(subscription_id)
    }

    async fn unsubscribe(&self, subscription_id: &str) -> EventResult<()> {
        let subscription_info = {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(subscription_id)
        };

        if let Some(info) = subscription_info {
            // Remove handler from handlers map
            let mut handlers = self.handlers.write().await;
            if let Some(handler_list) = handlers.get_mut(&info.event_type) {
                handler_list.retain(|h| h.handler_name() != info.handler_name);
                if handler_list.is_empty() {
                    handlers.remove(&info.event_type);
                }
            }

            info!(
                subscription_id = %subscription_id,
                event_type = %info.event_type,
                handler = %info.handler_name,
                "Event subscription removed"
            );

            Ok(())
        } else {
            Err(EventError::SubscriptionFailed(format!("Subscription not found: {}", subscription_id)))
        }
    }

    async fn start_processing(&self) -> EventResult<()> {
        self.start().await
    }

    async fn stop_processing(&self) -> EventResult<()> {
        // For in-memory implementation, we would need to signal the processing task to stop
        // This is a simplified implementation
        Ok(())
    }
}

/// Event message structure for internal processing
#[derive(Debug, Clone)]
struct EventMessage {
    envelope: EventEnvelope,
    retry_policy: Option<RetryPolicy>,
    published_at: chrono::DateTime<chrono::Utc>,
}

/// Subscription information
#[derive(Debug, Clone)]
struct SubscriptionInfo {
    event_type: String,
    handler_name: String,
    subscribed_at: chrono::DateTime<chrono::Utc>,
}

/// Event bus configuration
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub max_concurrent_handlers: usize,
    pub default_retry_policy: RetryPolicy,
    pub enable_dead_letter_queue: bool,
    pub processing_timeout: Duration,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            max_concurrent_handlers: 100,
            default_retry_policy: RetryPolicy::default(),
            enable_dead_letter_queue: true,
            processing_timeout: Duration::from_secs(30),
        }
    }
}

/// Processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    pub total_processed: u64,
    pub successful_processed: u64,
    pub failed_processed: u64,
    pub total_processing_time: Duration,
    pub average_processing_time: Duration,
}

impl ProcessingStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_processed == 0 {
            0.0
        } else {
            (self.successful_processed as f64) / (self.total_processed as f64)
        }
    }

    pub fn failure_rate(&self) -> f64 {
        1.0 - self.success_rate()
    }

    pub fn update_average_processing_time(&mut self) {
        if self.total_processed > 0 {
            self.average_processing_time = self.total_processing_time / self.total_processed as u32;
        }
    }
}

/// Dummy event implementation for testing
#[derive(Debug, Clone)]
struct DummyEvent {
    event_id: String,
    event_type: String,
    aggregate_id: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    version: u64,
    metadata: HashMap<String, serde_json::Value>,
}

impl Event for DummyEvent {
    fn event_type(&self) -> &str { &self.event_type }
    fn event_id(&self) -> &str { &self.event_id }
    fn aggregate_id(&self) -> &str { &self.aggregate_id }
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc> { self.timestamp }
    fn version(&self) -> u64 { self.version }
    fn metadata(&self) -> &HashMap<String, serde_json::Value> { &self.metadata }
    
    fn to_json(&self) -> EventResult<String> {
        Ok(format!("{{\"event_type\":\"{}\",\"event_id\":\"{}\"}}", self.event_type, self.event_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_events::UserRegistered;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TestHandler {
        name: String,
        call_count: Arc<AtomicU32>,
    }

    impl TestHandler {
        fn new(name: String) -> Self {
            Self {
                name,
                call_count: Arc::new(AtomicU32::new(0)),
            }
        }

        fn get_call_count(&self) -> u32 {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl EventHandler<dyn Event> for TestHandler {
        async fn handle(&self, _event: &dyn Event, _envelope: &EventEnvelope) -> EventResult<()> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        fn handler_name(&self) -> &str {
            &self.name
        }

        fn can_handle(&self, event_type: &str) -> bool {
            event_type == "UserRegistered"
        }
    }

    #[tokio::test]
    async fn test_event_bus_publish_and_subscribe() {
        let config = EventBusConfig::default();
        let event_bus = InMemoryEventBus::new(config);
        
        // Start the event bus
        event_bus.start().await.unwrap();

        // Create a test handler
        let handler = TestHandler::new("test_handler".to_string());
        let call_count = handler.call_count.clone();

        // Subscribe to events
        let subscription_id = event_bus.subscribe("UserRegistered", Box::new(handler)).await.unwrap();

        // Create and publish an event
        let event = UserRegistered {
            event_id: Uuid::new_v4().to_string(),
            user_id: "user123".to_string(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            timestamp: chrono::Utc::now(),
            version: 1,
            metadata: HashMap::new(),
        };

        let context = PublishContext::new("User".to_string());
        event_bus.publish(&event, context).await.unwrap();

        // Wait a bit for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check that the handler was called
        assert_eq!(call_count.load(Ordering::Relaxed), 1);

        // Unsubscribe
        event_bus.unsubscribe(&subscription_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_event_bus_stats() {
        let config = EventBusConfig::default();
        let event_bus = InMemoryEventBus::new(config);
        
        event_bus.start().await.unwrap();

        let handler = TestHandler::new("stats_handler".to_string());
        event_bus.subscribe("UserRegistered", Box::new(handler)).await.unwrap();

        // Publish multiple events
        for i in 0..5 {
            let event = UserRegistered {
                event_id: Uuid::new_v4().to_string(),
                user_id: format!("user{}", i),
                email: format!("test{}@example.com", i),
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                timestamp: chrono::Utc::now(),
                version: 1,
                metadata: HashMap::new(),
            };

            let context = PublishContext::new("User".to_string());
            event_bus.publish(&event, context).await.unwrap();
        }

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        let stats = event_bus.get_stats().await;
        assert_eq!(stats.total_processed, 5);
        assert_eq!(stats.successful_processed, 5);
        assert_eq!(stats.failed_processed, 0);
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let retry_policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        let delay1 = InMemoryEventBus::calculate_retry_delay(&retry_policy, 1);
        let delay2 = InMemoryEventBus::calculate_retry_delay(&retry_policy, 2);
        let delay3 = InMemoryEventBus::calculate_retry_delay(&retry_policy, 3);

        assert_eq!(delay1, Duration::from_millis(100));
        assert_eq!(delay2, Duration::from_millis(200));
        assert_eq!(delay3, Duration::from_millis(400));
    }
}
