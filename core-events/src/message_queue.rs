// =====================================================================================
// File: core-events/src/message_queue.rs
// Description: Message queue implementation for asynchronous processing
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{EventError, EventResult, ProcessingStatus, RetryPolicy};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Message queue trait
#[async_trait]
pub trait MessageQueue: Send + Sync {
    /// Send a message to the queue
    async fn send(&self, queue_name: &str, message: QueueMessage) -> EventResult<String>;
    
    /// Receive messages from the queue
    async fn receive(&self, queue_name: &str, max_messages: u32) -> EventResult<Vec<QueueMessage>>;
    
    /// Acknowledge message processing
    async fn acknowledge(&self, queue_name: &str, message_id: &str) -> EventResult<()>;
    
    /// Reject message and optionally requeue
    async fn reject(&self, queue_name: &str, message_id: &str, requeue: bool) -> EventResult<()>;
    
    /// Get queue statistics
    async fn get_queue_stats(&self, queue_name: &str) -> EventResult<QueueStats>;
    
    /// Create a new queue
    async fn create_queue(&self, queue_name: &str, config: QueueConfig) -> EventResult<()>;
    
    /// Delete a queue
    async fn delete_queue(&self, queue_name: &str) -> EventResult<()>;
    
    /// Purge all messages from a queue
    async fn purge_queue(&self, queue_name: &str) -> EventResult<u64>;
}

/// Message processor trait
#[async_trait]
pub trait MessageProcessor: Send + Sync {
    /// Process a message
    async fn process(&self, message: &QueueMessage) -> EventResult<ProcessingResult>;
    
    /// Get processor name
    fn processor_name(&self) -> &str;
    
    /// Check if this processor can handle the message type
    fn can_process(&self, message_type: &str) -> bool;
}

/// Queue message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: String,
    pub message_type: String,
    pub payload: serde_json::Value,
    pub routing_key: Option<String>,
    pub correlation_id: Option<String>,
    pub reply_to: Option<String>,
    pub expiration: Option<chrono::DateTime<chrono::Utc>>,
    pub priority: u8,
    pub retry_count: u32,
    pub max_retries: u32,
    pub delay_until: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl QueueMessage {
    pub fn new(message_type: String, payload: serde_json::Value) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            message_type,
            payload,
            routing_key: None,
            correlation_id: None,
            reply_to: None,
            expiration: None,
            priority: 0,
            retry_count: 0,
            max_retries: 3,
            delay_until: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_routing_key(mut self, routing_key: String) -> Self {
        self.routing_key = Some(routing_key);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_expiration(mut self, expiration: chrono::DateTime<chrono::Utc>) -> Self {
        self.expiration = Some(expiration);
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay_until = Some(chrono::Utc::now() + chrono::Duration::from_std(delay).unwrap());
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration {
            chrono::Utc::now() > expiration
        } else {
            false
        }
    }

    pub fn is_delayed(&self) -> bool {
        if let Some(delay_until) = self.delay_until {
            chrono::Utc::now() < delay_until
        } else {
            false
        }
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
        self.updated_at = chrono::Utc::now();
    }
}

/// Queue configuration
#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub max_length: Option<u64>,
    pub message_ttl: Option<Duration>,
    pub dead_letter_queue: Option<String>,
    pub max_priority: u8,
    pub enable_deduplication: bool,
    pub visibility_timeout: Duration,
    pub receive_wait_time: Duration,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_length: Some(10000),
            message_ttl: Some(Duration::from_secs(3600)), // 1 hour
            dead_letter_queue: None,
            max_priority: 10,
            enable_deduplication: false,
            visibility_timeout: Duration::from_secs(30),
            receive_wait_time: Duration::from_secs(20),
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub queue_name: String,
    pub message_count: u64,
    pub in_flight_count: u64,
    pub delayed_count: u64,
    pub dead_letter_count: u64,
    pub total_sent: u64,
    pub total_received: u64,
    pub total_acknowledged: u64,
    pub total_rejected: u64,
    pub average_processing_time: Duration,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Processing result for message processing
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub status: ProcessingStatus,
    pub error_message: Option<String>,
    pub retry_after: Option<Duration>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ProcessingResult {
    pub fn success() -> Self {
        Self {
            status: ProcessingStatus::Completed,
            error_message: None,
            retry_after: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            status: ProcessingStatus::Failed,
            error_message: Some(error),
            retry_after: None,
            metadata: HashMap::new(),
        }
    }

    pub fn retry(error: String, retry_after: Duration) -> Self {
        Self {
            status: ProcessingStatus::Retrying,
            error_message: Some(error),
            retry_after: Some(retry_after),
            metadata: HashMap::new(),
        }
    }
}

/// In-memory message queue implementation
pub struct InMemoryMessageQueue {
    queues: Arc<RwLock<HashMap<String, QueueData>>>,
    processors: Arc<RwLock<HashMap<String, Arc<dyn MessageProcessor>>>>,
    processing_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl InMemoryMessageQueue {
    pub fn new() -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            processors: Arc::new(RwLock::new(HashMap::new())),
            processing_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a message processor
    pub async fn register_processor(&self, processor: Arc<dyn MessageProcessor>) {
        let mut processors = self.processors.write().await;
        processors.insert(processor.processor_name().to_string(), processor);
    }

    /// Start processing messages for a queue
    pub async fn start_processing(&self, queue_name: &str) -> EventResult<()> {
        let queues = Arc::clone(&self.queues);
        let processors = Arc::clone(&self.processors);
        let queue_name_owned = queue_name.to_string();

        let task = tokio::spawn(async move {
            Self::process_queue_messages(queue_name_owned, queues, processors).await;
        });

        let mut tasks = self.processing_tasks.write().await;
        tasks.insert(queue_name.to_string(), task);

        info!(queue_name = %queue_name, "Started message processing");
        Ok(())
    }

    /// Stop processing messages for a queue
    pub async fn stop_processing(&self, queue_name: &str) -> EventResult<()> {
        let mut tasks = self.processing_tasks.write().await;
        if let Some(task) = tasks.remove(queue_name) {
            task.abort();
            info!(queue_name = %queue_name, "Stopped message processing");
        }
        Ok(())
    }

    /// Process messages in a queue
    async fn process_queue_messages(
        queue_name: String,
        queues: Arc<RwLock<HashMap<String, QueueData>>>,
        processors: Arc<RwLock<HashMap<String, Arc<dyn MessageProcessor>>>>,
    ) {
        loop {
            // Get messages from queue
            let messages = {
                let mut queues_guard = queues.write().await;
                if let Some(queue_data) = queues_guard.get_mut(&queue_name) {
                    let mut available_messages = Vec::new();
                    
                    // Get up to 10 messages that are ready for processing
                    while available_messages.len() < 10 {
                        if let Some(message) = queue_data.messages.pop_front() {
                            if !message.is_delayed() && !message.is_expired() {
                                available_messages.push(message);
                            } else if message.is_expired() {
                                // Move expired messages to dead letter queue
                                if let Some(ref dlq_name) = queue_data.config.dead_letter_queue {
                                    // In a real implementation, move to DLQ
                                    warn!(
                                        message_id = %message.id,
                                        queue_name = %queue_name,
                                        dlq_name = %dlq_name,
                                        "Message expired, moving to dead letter queue"
                                    );
                                }
                                queue_data.stats.dead_letter_count += 1;
                            } else {
                                // Put delayed message back
                                queue_data.messages.push_back(message);
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    
                    available_messages
                } else {
                    Vec::new()
                }
            };

            if messages.is_empty() {
                // No messages to process, wait a bit
                sleep(Duration::from_millis(100)).await;
                continue;
            }

            // Process each message
            for message in messages {
                let processors_guard = processors.read().await;
                
                // Find a processor for this message type
                let processor = processors_guard.values()
                    .find(|p| p.can_process(&message.message_type));

                if let Some(processor) = processor {
                    let start_time = Instant::now();
                    
                    match processor.process(&message).await {
                        Ok(result) => {
                            let processing_time = start_time.elapsed();
                            
                            match result.status {
                                ProcessingStatus::Completed => {
                                    debug!(
                                        message_id = %message.id,
                                        queue_name = %queue_name,
                                        processor = %processor.processor_name(),
                                        processing_time_ms = %processing_time.as_millis(),
                                        "Message processed successfully"
                                    );
                                    
                                    // Update stats
                                    let mut queues_guard = queues.write().await;
                                    if let Some(queue_data) = queues_guard.get_mut(&queue_name) {
                                        queue_data.stats.total_acknowledged += 1;
                                        queue_data.stats.last_activity = Some(chrono::Utc::now());
                                    }
                                }
                                ProcessingStatus::Failed => {
                                    error!(
                                        message_id = %message.id,
                                        queue_name = %queue_name,
                                        error = ?result.error_message,
                                        "Message processing failed"
                                    );
                                    
                                    // Handle failed message
                                    Self::handle_failed_message(message, &queues, &queue_name).await;
                                }
                                ProcessingStatus::Retrying => {
                                    warn!(
                                        message_id = %message.id,
                                        queue_name = %queue_name,
                                        retry_after = ?result.retry_after,
                                        "Message processing failed, will retry"
                                    );
                                    
                                    // Schedule retry
                                    Self::schedule_retry(message, result.retry_after, &queues, &queue_name).await;
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            error!(
                                message_id = %message.id,
                                queue_name = %queue_name,
                                error = %e,
                                "Message processing error"
                            );
                            
                            Self::handle_failed_message(message, &queues, &queue_name).await;
                        }
                    }
                } else {
                    warn!(
                        message_id = %message.id,
                        message_type = %message.message_type,
                        queue_name = %queue_name,
                        "No processor found for message type"
                    );
                    
                    // Put message back in queue or move to DLQ
                    Self::handle_failed_message(message, &queues, &queue_name).await;
                }
            }
        }
    }

    /// Handle a failed message
    async fn handle_failed_message(
        mut message: QueueMessage,
        queues: &Arc<RwLock<HashMap<String, QueueData>>>,
        queue_name: &str,
    ) {
        let mut queues_guard = queues.write().await;
        if let Some(queue_data) = queues_guard.get_mut(queue_name) {
            if message.can_retry() {
                message.increment_retry();
                // Add exponential backoff delay
                let delay = Duration::from_millis(100 * (2_u64.pow(message.retry_count)));
                message.delay_until = Some(chrono::Utc::now() + chrono::Duration::from_std(delay).unwrap());
                queue_data.messages.push_back(message);
                queue_data.stats.delayed_count += 1;
            } else {
                // Move to dead letter queue
                if let Some(ref dlq_name) = queue_data.config.dead_letter_queue {
                    // In a real implementation, move to DLQ
                    warn!(
                        message_id = %message.id,
                        queue_name = %queue_name,
                        dlq_name = %dlq_name,
                        "Message exceeded max retries, moving to dead letter queue"
                    );
                }
                queue_data.stats.dead_letter_count += 1;
                queue_data.stats.total_rejected += 1;
            }
        }
    }

    /// Schedule a message retry
    async fn schedule_retry(
        mut message: QueueMessage,
        retry_after: Option<Duration>,
        queues: &Arc<RwLock<HashMap<String, QueueData>>>,
        queue_name: &str,
    ) {
        message.increment_retry();
        
        if let Some(delay) = retry_after {
            message.delay_until = Some(chrono::Utc::now() + chrono::Duration::from_std(delay).unwrap());
        }

        let mut queues_guard = queues.write().await;
        if let Some(queue_data) = queues_guard.get_mut(queue_name) {
            queue_data.messages.push_back(message);
            queue_data.stats.delayed_count += 1;
        }
    }
}

#[async_trait]
impl MessageQueue for InMemoryMessageQueue {
    async fn send(&self, queue_name: &str, message: QueueMessage) -> EventResult<String> {
        let mut queues = self.queues.write().await;
        
        let queue_data = queues.get_mut(queue_name)
            .ok_or_else(|| EventError::QueueError(format!("Queue not found: {}", queue_name)))?;

        // Check queue limits
        if let Some(max_length) = queue_data.config.max_length {
            if queue_data.messages.len() as u64 >= max_length {
                return Err(EventError::QueueError("Queue is full".to_string()));
            }
        }

        let message_id = message.id.clone();
        queue_data.messages.push_back(message);
        queue_data.stats.message_count += 1;
        queue_data.stats.total_sent += 1;
        queue_data.stats.last_activity = Some(chrono::Utc::now());

        debug!(
            queue_name = %queue_name,
            message_id = %message_id,
            "Message sent to queue"
        );

        Ok(message_id)
    }

    async fn receive(&self, queue_name: &str, max_messages: u32) -> EventResult<Vec<QueueMessage>> {
        let mut queues = self.queues.write().await;
        
        let queue_data = queues.get_mut(queue_name)
            .ok_or_else(|| EventError::QueueError(format!("Queue not found: {}", queue_name)))?;

        let mut messages = Vec::new();
        let mut count = 0;

        while count < max_messages {
            if let Some(message) = queue_data.messages.pop_front() {
                if !message.is_delayed() && !message.is_expired() {
                    messages.push(message);
                    count += 1;
                } else if message.is_expired() {
                    // Skip expired messages
                    queue_data.stats.dead_letter_count += 1;
                } else {
                    // Put delayed message back
                    queue_data.messages.push_back(message);
                    break;
                }
            } else {
                break;
            }
        }

        queue_data.stats.total_received += messages.len() as u64;
        queue_data.stats.in_flight_count += messages.len() as u64;
        queue_data.stats.last_activity = Some(chrono::Utc::now());

        Ok(messages)
    }

    async fn acknowledge(&self, queue_name: &str, message_id: &str) -> EventResult<()> {
        let mut queues = self.queues.write().await;
        
        let queue_data = queues.get_mut(queue_name)
            .ok_or_else(|| EventError::QueueError(format!("Queue not found: {}", queue_name)))?;

        queue_data.stats.total_acknowledged += 1;
        queue_data.stats.in_flight_count = queue_data.stats.in_flight_count.saturating_sub(1);
        queue_data.stats.last_activity = Some(chrono::Utc::now());

        debug!(
            queue_name = %queue_name,
            message_id = %message_id,
            "Message acknowledged"
        );

        Ok(())
    }

    async fn reject(&self, queue_name: &str, message_id: &str, requeue: bool) -> EventResult<()> {
        let mut queues = self.queues.write().await;
        
        let queue_data = queues.get_mut(queue_name)
            .ok_or_else(|| EventError::QueueError(format!("Queue not found: {}", queue_name)))?;

        queue_data.stats.total_rejected += 1;
        queue_data.stats.in_flight_count = queue_data.stats.in_flight_count.saturating_sub(1);
        
        if !requeue {
            queue_data.stats.dead_letter_count += 1;
        }
        
        queue_data.stats.last_activity = Some(chrono::Utc::now());

        debug!(
            queue_name = %queue_name,
            message_id = %message_id,
            requeue = %requeue,
            "Message rejected"
        );

        Ok(())
    }

    async fn get_queue_stats(&self, queue_name: &str) -> EventResult<QueueStats> {
        let queues = self.queues.read().await;
        
        let queue_data = queues.get(queue_name)
            .ok_or_else(|| EventError::QueueError(format!("Queue not found: {}", queue_name)))?;

        Ok(queue_data.stats.clone())
    }

    async fn create_queue(&self, queue_name: &str, config: QueueConfig) -> EventResult<()> {
        let mut queues = self.queues.write().await;
        
        if queues.contains_key(queue_name) {
            return Err(EventError::QueueError(format!("Queue already exists: {}", queue_name)));
        }

        let queue_data = QueueData {
            messages: VecDeque::new(),
            config,
            stats: QueueStats {
                queue_name: queue_name.to_string(),
                message_count: 0,
                in_flight_count: 0,
                delayed_count: 0,
                dead_letter_count: 0,
                total_sent: 0,
                total_received: 0,
                total_acknowledged: 0,
                total_rejected: 0,
                average_processing_time: Duration::from_millis(0),
                created_at: chrono::Utc::now(),
                last_activity: None,
            },
        };

        queues.insert(queue_name.to_string(), queue_data);

        info!(queue_name = %queue_name, "Queue created");
        Ok(())
    }

    async fn delete_queue(&self, queue_name: &str) -> EventResult<()> {
        let mut queues = self.queues.write().await;
        
        if queues.remove(queue_name).is_some() {
            info!(queue_name = %queue_name, "Queue deleted");
            Ok(())
        } else {
            Err(EventError::QueueError(format!("Queue not found: {}", queue_name)))
        }
    }

    async fn purge_queue(&self, queue_name: &str) -> EventResult<u64> {
        let mut queues = self.queues.write().await;
        
        let queue_data = queues.get_mut(queue_name)
            .ok_or_else(|| EventError::QueueError(format!("Queue not found: {}", queue_name)))?;

        let purged_count = queue_data.messages.len() as u64;
        queue_data.messages.clear();
        queue_data.stats.message_count = 0;
        queue_data.stats.last_activity = Some(chrono::Utc::now());

        info!(
            queue_name = %queue_name,
            purged_count = %purged_count,
            "Queue purged"
        );

        Ok(purged_count)
    }
}

/// Internal queue data structure
struct QueueData {
    messages: VecDeque<QueueMessage>,
    config: QueueConfig,
    stats: QueueStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProcessor {
        name: String,
        should_fail: bool,
    }

    impl TestProcessor {
        fn new(name: String, should_fail: bool) -> Self {
            Self { name, should_fail }
        }
    }

    #[async_trait]
    impl MessageProcessor for TestProcessor {
        async fn process(&self, _message: &QueueMessage) -> EventResult<ProcessingResult> {
            if self.should_fail {
                Ok(ProcessingResult::failure("Test failure".to_string()))
            } else {
                Ok(ProcessingResult::success())
            }
        }

        fn processor_name(&self) -> &str {
            &self.name
        }

        fn can_process(&self, message_type: &str) -> bool {
            message_type == "test_message"
        }
    }

    #[tokio::test]
    async fn test_message_queue_basic_operations() {
        let queue = InMemoryMessageQueue::new();
        let queue_name = "test_queue";
        let config = QueueConfig::default();

        // Create queue
        queue.create_queue(queue_name, config).await.unwrap();

        // Send message
        let message = QueueMessage::new(
            "test_message".to_string(),
            serde_json::json!({"data": "test"}),
        );
        let message_id = queue.send(queue_name, message).await.unwrap();

        // Receive message
        let messages = queue.receive(queue_name, 1).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message_id);

        // Acknowledge message
        queue.acknowledge(queue_name, &message_id).await.unwrap();

        // Check stats
        let stats = queue.get_queue_stats(queue_name).await.unwrap();
        assert_eq!(stats.total_sent, 1);
        assert_eq!(stats.total_received, 1);
        assert_eq!(stats.total_acknowledged, 1);
    }

    #[tokio::test]
    async fn test_message_processing() {
        let queue = InMemoryMessageQueue::new();
        let queue_name = "processing_queue";
        let config = QueueConfig::default();

        // Create queue
        queue.create_queue(queue_name, config).await.unwrap();

        // Register processor
        let processor = Arc::new(TestProcessor::new("test_processor".to_string(), false));
        queue.register_processor(processor).await;

        // Send message
        let message = QueueMessage::new(
            "test_message".to_string(),
            serde_json::json!({"data": "test"}),
        );
        queue.send(queue_name, message).await.unwrap();

        // Start processing
        queue.start_processing(queue_name).await.unwrap();

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Stop processing
        queue.stop_processing(queue_name).await.unwrap();

        // Check stats
        let stats = queue.get_queue_stats(queue_name).await.unwrap();
        assert_eq!(stats.total_sent, 1);
    }

    #[tokio::test]
    async fn test_message_expiration() {
        let mut message = QueueMessage::new(
            "test_message".to_string(),
            serde_json::json!({"data": "test"}),
        );

        // Set expiration in the past
        message.expiration = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

        assert!(message.is_expired());
    }

    #[tokio::test]
    async fn test_message_delay() {
        let message = QueueMessage::new(
            "test_message".to_string(),
            serde_json::json!({"data": "test"}),
        ).with_delay(Duration::from_secs(1));

        assert!(message.is_delayed());
    }

    #[tokio::test]
    async fn test_queue_purge() {
        let queue = InMemoryMessageQueue::new();
        let queue_name = "purge_queue";
        let config = QueueConfig::default();

        // Create queue
        queue.create_queue(queue_name, config).await.unwrap();

        // Send multiple messages
        for i in 0..5 {
            let message = QueueMessage::new(
                "test_message".to_string(),
                serde_json::json!({"data": format!("test{}", i)}),
            );
            queue.send(queue_name, message).await.unwrap();
        }

        // Purge queue
        let purged_count = queue.purge_queue(queue_name).await.unwrap();
        assert_eq!(purged_count, 5);

        // Verify queue is empty
        let messages = queue.receive(queue_name, 10).await.unwrap();
        assert_eq!(messages.len(), 0);
    }
}
