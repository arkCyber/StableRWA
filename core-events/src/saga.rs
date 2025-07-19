// =====================================================================================
// File: core-events/src/saga.rs
// Description: Saga pattern implementation for distributed transactions
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{Event, EventEnvelope, EventError, EventResult, EventPublisher, PublishContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Saga orchestrator trait
#[async_trait]
pub trait SagaOrchestrator: Send + Sync {
    /// Start a new saga
    async fn start_saga(&self, saga_definition: SagaDefinition, context: SagaContext) -> EventResult<String>;
    
    /// Handle an event in the context of a saga
    async fn handle_event(&self, saga_id: &str, event: &EventEnvelope) -> EventResult<()>;
    
    /// Get saga status
    async fn get_saga_status(&self, saga_id: &str) -> EventResult<Option<SagaStatus>>;
    
    /// Cancel a running saga
    async fn cancel_saga(&self, saga_id: &str, reason: &str) -> EventResult<()>;
    
    /// Retry a failed saga step
    async fn retry_saga_step(&self, saga_id: &str, step_id: &str) -> EventResult<()>;
    
    /// Get all active sagas
    async fn get_active_sagas(&self) -> EventResult<Vec<SagaInstance>>;
}

/// Saga step trait
#[async_trait]
pub trait SagaStep: Send + Sync {
    /// Execute the step
    async fn execute(&self, context: &SagaContext, publisher: &dyn EventPublisher) -> EventResult<SagaStepResult>;
    
    /// Compensate (rollback) the step
    async fn compensate(&self, context: &SagaContext, publisher: &dyn EventPublisher) -> EventResult<SagaStepResult>;
    
    /// Get step name
    fn step_name(&self) -> &str;
    
    /// Check if this step can be retried
    fn can_retry(&self) -> bool;
    
    /// Get maximum retry attempts
    fn max_retries(&self) -> u32;
}

/// Saga definition
#[derive(Debug, Clone)]
pub struct SagaDefinition {
    pub name: String,
    pub description: String,
    pub steps: Vec<Arc<dyn SagaStep>>,
    pub timeout: Option<std::time::Duration>,
    pub retry_policy: SagaRetryPolicy,
}

/// Saga context containing data and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaContext {
    pub saga_id: String,
    pub correlation_id: String,
    pub user_id: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl SagaContext {
    pub fn new(correlation_id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            saga_id: Uuid::new_v4().to_string(),
            correlation_id,
            user_id: None,
            data: HashMap::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn set_data(&mut self, key: &str, value: serde_json::Value) {
        self.data.insert(key.to_string(), value);
        self.updated_at = chrono::Utc::now();
    }

    pub fn get_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
        self.updated_at = chrono::Utc::now();
    }
}

/// Saga instance representing a running saga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaInstance {
    pub saga_id: String,
    pub saga_name: String,
    pub status: SagaStatus,
    pub context: SagaContext,
    pub current_step: usize,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    pub compensated_steps: Vec<String>,
    pub error_message: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl SagaInstance {
    pub fn new(saga_id: String, saga_name: String, context: SagaContext) -> Self {
        let now = chrono::Utc::now();
        Self {
            saga_id,
            saga_name,
            status: SagaStatus::Running,
            context,
            current_step: 0,
            completed_steps: Vec::new(),
            failed_steps: Vec::new(),
            compensated_steps: Vec::new(),
            error_message: None,
            started_at: now,
            completed_at: None,
            last_activity: now,
        }
    }

    pub fn mark_step_completed(&mut self, step_name: String) {
        self.completed_steps.push(step_name);
        self.current_step += 1;
        self.last_activity = chrono::Utc::now();
    }

    pub fn mark_step_failed(&mut self, step_name: String, error: String) {
        self.failed_steps.push(step_name);
        self.error_message = Some(error);
        self.status = SagaStatus::Failed;
        self.last_activity = chrono::Utc::now();
    }

    pub fn mark_step_compensated(&mut self, step_name: String) {
        self.compensated_steps.push(step_name);
        self.last_activity = chrono::Utc::now();
    }

    pub fn complete(&mut self) {
        self.status = SagaStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
        self.last_activity = chrono::Utc::now();
    }

    pub fn cancel(&mut self, reason: String) {
        self.status = SagaStatus::Cancelled;
        self.error_message = Some(reason);
        self.completed_at = Some(chrono::Utc::now());
        self.last_activity = chrono::Utc::now();
    }
}

/// Saga status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SagaStatus {
    Running,
    Completed,
    Failed,
    Compensating,
    Compensated,
    Cancelled,
}

/// Saga step result
#[derive(Debug, Clone)]
pub enum SagaStepResult {
    Success,
    Failure(String),
    Retry(String, std::time::Duration),
}

/// Saga retry policy
#[derive(Debug, Clone)]
pub struct SagaRetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: std::time::Duration,
    pub max_delay: std::time::Duration,
    pub backoff_multiplier: f64,
}

impl Default for SagaRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: std::time::Duration::from_secs(1),
            max_delay: std::time::Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

/// In-memory saga orchestrator implementation
pub struct InMemorySagaOrchestrator {
    sagas: Arc<RwLock<HashMap<String, SagaInstance>>>,
    definitions: Arc<RwLock<HashMap<String, SagaDefinition>>>,
    publisher: Arc<dyn EventPublisher>,
}

impl InMemorySagaOrchestrator {
    pub fn new(publisher: Arc<dyn EventPublisher>) -> Self {
        Self {
            sagas: Arc::new(RwLock::new(HashMap::new())),
            definitions: Arc::new(RwLock::new(HashMap::new())),
            publisher,
        }
    }

    /// Register a saga definition
    pub async fn register_saga(&self, definition: SagaDefinition) {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.name.clone(), definition);
    }

    /// Execute the next step in a saga
    async fn execute_next_step(&self, saga_id: &str) -> EventResult<()> {
        let (saga_instance, step) = {
            let sagas = self.sagas.read().await;
            let definitions = self.definitions.read().await;
            
            let saga = sagas.get(saga_id)
                .ok_or_else(|| EventError::ProcessingError(format!("Saga not found: {}", saga_id)))?;
            
            let definition = definitions.get(&saga.saga_name)
                .ok_or_else(|| EventError::ProcessingError(format!("Saga definition not found: {}", saga.saga_name)))?;
            
            if saga.current_step >= definition.steps.len() {
                // All steps completed
                return self.complete_saga(saga_id).await;
            }
            
            let step = definition.steps[saga.current_step].clone();
            (saga.clone(), step)
        };

        info!(
            saga_id = %saga_id,
            step_name = %step.step_name(),
            current_step = %saga_instance.current_step,
            "Executing saga step"
        );

        match step.execute(&saga_instance.context, self.publisher.as_ref()).await {
            Ok(SagaStepResult::Success) => {
                self.mark_step_completed(saga_id, step.step_name().to_string()).await?;
                self.execute_next_step(saga_id).await
            }
            Ok(SagaStepResult::Failure(error)) => {
                self.mark_step_failed(saga_id, step.step_name().to_string(), error.clone()).await?;
                self.start_compensation(saga_id).await
            }
            Ok(SagaStepResult::Retry(error, delay)) => {
                warn!(
                    saga_id = %saga_id,
                    step_name = %step.step_name(),
                    error = %error,
                    delay_ms = %delay.as_millis(),
                    "Saga step failed, will retry"
                );
                
                // Schedule retry (simplified - in production use a scheduler)
                tokio::time::sleep(delay).await;
                self.execute_next_step(saga_id).await
            }
            Err(e) => {
                self.mark_step_failed(saga_id, step.step_name().to_string(), e.to_string()).await?;
                self.start_compensation(saga_id).await
            }
        }
    }

    /// Start compensation (rollback) process
    async fn start_compensation(&self, saga_id: &str) -> EventResult<()> {
        info!(saga_id = %saga_id, "Starting saga compensation");

        // Update saga status to compensating
        {
            let mut sagas = self.sagas.write().await;
            if let Some(saga) = sagas.get_mut(saga_id) {
                saga.status = SagaStatus::Compensating;
                saga.last_activity = chrono::Utc::now();
            }
        }

        self.compensate_next_step(saga_id).await
    }

    /// Compensate the next step in reverse order
    async fn compensate_next_step(&self, saga_id: &str) -> EventResult<()> {
        let (saga_instance, step_to_compensate) = {
            let sagas = self.sagas.read().await;
            let definitions = self.definitions.read().await;
            
            let saga = sagas.get(saga_id)
                .ok_or_else(|| EventError::ProcessingError(format!("Saga not found: {}", saga_id)))?;
            
            let definition = definitions.get(&saga.saga_name)
                .ok_or_else(|| EventError::ProcessingError(format!("Saga definition not found: {}", saga.saga_name)))?;
            
            // Find the last completed step to compensate
            if saga.completed_steps.is_empty() {
                // No steps to compensate, mark as compensated
                return self.complete_compensation(saga_id).await;
            }
            
            let last_completed_step_name = &saga.completed_steps[saga.completed_steps.len() - 1];
            let step = definition.steps.iter()
                .find(|s| s.step_name() == last_completed_step_name)
                .ok_or_else(|| EventError::ProcessingError(format!("Step not found: {}", last_completed_step_name)))?
                .clone();
            
            (saga.clone(), step)
        };

        info!(
            saga_id = %saga_id,
            step_name = %step_to_compensate.step_name(),
            "Compensating saga step"
        );

        match step_to_compensate.compensate(&saga_instance.context, self.publisher.as_ref()).await {
            Ok(SagaStepResult::Success) => {
                self.mark_step_compensated(saga_id, step_to_compensate.step_name().to_string()).await?;
                self.compensate_next_step(saga_id).await
            }
            Ok(SagaStepResult::Failure(error)) => {
                error!(
                    saga_id = %saga_id,
                    step_name = %step_to_compensate.step_name(),
                    error = %error,
                    "Saga step compensation failed"
                );
                // In production, this might require manual intervention
                Err(EventError::ProcessingError(format!("Compensation failed: {}", error)))
            }
            Ok(SagaStepResult::Retry(error, delay)) => {
                warn!(
                    saga_id = %saga_id,
                    step_name = %step_to_compensate.step_name(),
                    error = %error,
                    delay_ms = %delay.as_millis(),
                    "Saga step compensation failed, will retry"
                );
                
                tokio::time::sleep(delay).await;
                self.compensate_next_step(saga_id).await
            }
            Err(e) => {
                error!(
                    saga_id = %saga_id,
                    step_name = %step_to_compensate.step_name(),
                    error = %e,
                    "Saga step compensation error"
                );
                Err(e)
            }
        }
    }

    /// Mark a step as completed
    async fn mark_step_completed(&self, saga_id: &str, step_name: String) -> EventResult<()> {
        let mut sagas = self.sagas.write().await;
        if let Some(saga) = sagas.get_mut(saga_id) {
            saga.mark_step_completed(step_name);
        }
        Ok(())
    }

    /// Mark a step as failed
    async fn mark_step_failed(&self, saga_id: &str, step_name: String, error: String) -> EventResult<()> {
        let mut sagas = self.sagas.write().await;
        if let Some(saga) = sagas.get_mut(saga_id) {
            saga.mark_step_failed(step_name, error);
        }
        Ok(())
    }

    /// Mark a step as compensated
    async fn mark_step_compensated(&self, saga_id: &str, step_name: String) -> EventResult<()> {
        let mut sagas = self.sagas.write().await;
        if let Some(saga) = sagas.get_mut(saga_id) {
            saga.mark_step_compensated(step_name);
            // Remove from completed steps
            saga.completed_steps.retain(|s| s != &step_name);
        }
        Ok(())
    }

    /// Complete the saga successfully
    async fn complete_saga(&self, saga_id: &str) -> EventResult<()> {
        let mut sagas = self.sagas.write().await;
        if let Some(saga) = sagas.get_mut(saga_id) {
            saga.complete();
            info!(saga_id = %saga_id, "Saga completed successfully");
        }
        Ok(())
    }

    /// Complete the compensation process
    async fn complete_compensation(&self, saga_id: &str) -> EventResult<()> {
        let mut sagas = self.sagas.write().await;
        if let Some(saga) = sagas.get_mut(saga_id) {
            saga.status = SagaStatus::Compensated;
            saga.completed_at = Some(chrono::Utc::now());
            saga.last_activity = chrono::Utc::now();
            info!(saga_id = %saga_id, "Saga compensation completed");
        }
        Ok(())
    }
}

#[async_trait]
impl SagaOrchestrator for InMemorySagaOrchestrator {
    async fn start_saga(&self, saga_definition: SagaDefinition, context: SagaContext) -> EventResult<String> {
        let saga_id = context.saga_id.clone();
        
        info!(
            saga_id = %saga_id,
            saga_name = %saga_definition.name,
            "Starting new saga"
        );

        // Register the definition if not already registered
        {
            let mut definitions = self.definitions.write().await;
            definitions.insert(saga_definition.name.clone(), saga_definition.clone());
        }

        // Create saga instance
        let saga_instance = SagaInstance::new(saga_id.clone(), saga_definition.name.clone(), context);
        
        // Store saga instance
        {
            let mut sagas = self.sagas.write().await;
            sagas.insert(saga_id.clone(), saga_instance);
        }

        // Start executing steps
        self.execute_next_step(&saga_id).await?;

        Ok(saga_id)
    }

    async fn handle_event(&self, saga_id: &str, event: &EventEnvelope) -> EventResult<()> {
        debug!(
            saga_id = %saga_id,
            event_type = %event.event_type,
            "Handling event for saga"
        );

        // In a real implementation, this would handle events that affect saga state
        // For now, we'll just continue execution if the saga is waiting
        self.execute_next_step(saga_id).await
    }

    async fn get_saga_status(&self, saga_id: &str) -> EventResult<Option<SagaStatus>> {
        let sagas = self.sagas.read().await;
        Ok(sagas.get(saga_id).map(|saga| saga.status.clone()))
    }

    async fn cancel_saga(&self, saga_id: &str, reason: &str) -> EventResult<()> {
        info!(saga_id = %saga_id, reason = %reason, "Cancelling saga");

        let mut sagas = self.sagas.write().await;
        if let Some(saga) = sagas.get_mut(saga_id) {
            saga.cancel(reason.to_string());
            Ok(())
        } else {
            Err(EventError::ProcessingError(format!("Saga not found: {}", saga_id)))
        }
    }

    async fn retry_saga_step(&self, saga_id: &str, step_id: &str) -> EventResult<()> {
        info!(saga_id = %saga_id, step_id = %step_id, "Retrying saga step");

        // Reset the saga to retry the failed step
        {
            let mut sagas = self.sagas.write().await;
            if let Some(saga) = sagas.get_mut(saga_id) {
                saga.status = SagaStatus::Running;
                saga.error_message = None;
                saga.last_activity = chrono::Utc::now();
            }
        }

        self.execute_next_step(saga_id).await
    }

    async fn get_active_sagas(&self) -> EventResult<Vec<SagaInstance>> {
        let sagas = self.sagas.read().await;
        let active_sagas = sagas.values()
            .filter(|saga| matches!(saga.status, SagaStatus::Running | SagaStatus::Compensating))
            .cloned()
            .collect();
        Ok(active_sagas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InMemoryEventBus;

    struct TestStep {
        name: String,
        should_fail: bool,
    }

    impl TestStep {
        fn new(name: String, should_fail: bool) -> Self {
            Self { name, should_fail }
        }
    }

    #[async_trait]
    impl SagaStep for TestStep {
        async fn execute(&self, _context: &SagaContext, _publisher: &dyn EventPublisher) -> EventResult<SagaStepResult> {
            if self.should_fail {
                Ok(SagaStepResult::Failure("Test failure".to_string()))
            } else {
                Ok(SagaStepResult::Success)
            }
        }

        async fn compensate(&self, _context: &SagaContext, _publisher: &dyn EventPublisher) -> EventResult<SagaStepResult> {
            Ok(SagaStepResult::Success)
        }

        fn step_name(&self) -> &str {
            &self.name
        }

        fn can_retry(&self) -> bool {
            true
        }

        fn max_retries(&self) -> u32 {
            3
        }
    }

    #[tokio::test]
    async fn test_saga_success_flow() {
        let event_bus = Arc::new(InMemoryEventBus::new(Default::default()));
        let orchestrator = InMemorySagaOrchestrator::new(event_bus);

        // Create saga definition
        let definition = SagaDefinition {
            name: "test_saga".to_string(),
            description: "Test saga".to_string(),
            steps: vec![
                Arc::new(TestStep::new("step1".to_string(), false)),
                Arc::new(TestStep::new("step2".to_string(), false)),
            ],
            timeout: None,
            retry_policy: SagaRetryPolicy::default(),
        };

        // Create context
        let context = SagaContext::new("correlation123".to_string());
        let saga_id = context.saga_id.clone();

        // Start saga
        orchestrator.start_saga(definition, context).await.unwrap();

        // Wait for completion
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Check status
        let status = orchestrator.get_saga_status(&saga_id).await.unwrap();
        assert_eq!(status, Some(SagaStatus::Completed));
    }

    #[tokio::test]
    async fn test_saga_failure_and_compensation() {
        let event_bus = Arc::new(InMemoryEventBus::new(Default::default()));
        let orchestrator = InMemorySagaOrchestrator::new(event_bus);

        // Create saga definition with failing step
        let definition = SagaDefinition {
            name: "failing_saga".to_string(),
            description: "Failing saga".to_string(),
            steps: vec![
                Arc::new(TestStep::new("step1".to_string(), false)),
                Arc::new(TestStep::new("step2".to_string(), true)), // This will fail
            ],
            timeout: None,
            retry_policy: SagaRetryPolicy::default(),
        };

        // Create context
        let context = SagaContext::new("correlation456".to_string());
        let saga_id = context.saga_id.clone();

        // Start saga
        orchestrator.start_saga(definition, context).await.unwrap();

        // Wait for completion
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Check status - should be compensated
        let status = orchestrator.get_saga_status(&saga_id).await.unwrap();
        assert_eq!(status, Some(SagaStatus::Compensated));
    }

    #[tokio::test]
    async fn test_saga_cancellation() {
        let event_bus = Arc::new(InMemoryEventBus::new(Default::default()));
        let orchestrator = InMemorySagaOrchestrator::new(event_bus);

        // Create context
        let context = SagaContext::new("correlation789".to_string());
        let saga_id = context.saga_id.clone();

        // Create saga definition
        let definition = SagaDefinition {
            name: "cancellable_saga".to_string(),
            description: "Cancellable saga".to_string(),
            steps: vec![Arc::new(TestStep::new("step1".to_string(), false))],
            timeout: None,
            retry_policy: SagaRetryPolicy::default(),
        };

        // Start saga
        orchestrator.start_saga(definition, context).await.unwrap();

        // Cancel saga
        orchestrator.cancel_saga(&saga_id, "User requested cancellation").await.unwrap();

        // Check status
        let status = orchestrator.get_saga_status(&saga_id).await.unwrap();
        assert_eq!(status, Some(SagaStatus::Cancelled));
    }
}
