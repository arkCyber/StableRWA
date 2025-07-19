// =====================================================================================
// File: core-events/src/event_store.rs
// Description: Event store implementation for event sourcing
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{Event, EventEnvelope, EventError, EventResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Event store trait for persisting and retrieving events
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to the store
    async fn append_events(&self, stream_id: &str, expected_version: Option<u64>, events: Vec<EventEnvelope>) -> EventResult<u64>;
    
    /// Read events from a stream
    async fn read_stream(&self, stream_id: &str, from_version: Option<u64>, max_count: Option<usize>) -> EventResult<Vec<EventEnvelope>>;
    
    /// Read all events from a specific version
    async fn read_all_events(&self, from_position: Option<u64>, max_count: Option<usize>) -> EventResult<Vec<EventEnvelope>>;
    
    /// Get the current version of a stream
    async fn get_stream_version(&self, stream_id: &str) -> EventResult<Option<u64>>;
    
    /// Check if a stream exists
    async fn stream_exists(&self, stream_id: &str) -> EventResult<bool>;
    
    /// Delete a stream (soft delete)
    async fn delete_stream(&self, stream_id: &str, expected_version: Option<u64>) -> EventResult<()>;
    
    /// Get stream metadata
    async fn get_stream_metadata(&self, stream_id: &str) -> EventResult<Option<StreamMetadata>>;
    
    /// Set stream metadata
    async fn set_stream_metadata(&self, stream_id: &str, metadata: StreamMetadata) -> EventResult<()>;
    
    /// Create a snapshot of an aggregate
    async fn save_snapshot(&self, stream_id: &str, version: u64, snapshot: AggregateSnapshot) -> EventResult<()>;
    
    /// Load the latest snapshot for an aggregate
    async fn load_snapshot(&self, stream_id: &str) -> EventResult<Option<AggregateSnapshot>>;
}

/// Stream metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMetadata {
    pub stream_id: String,
    pub max_age: Option<chrono::Duration>,
    pub max_count: Option<u64>,
    pub cache_control: Option<chrono::Duration>,
    pub acl: Option<StreamAcl>,
    pub custom_metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Stream access control list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamAcl {
    pub read_roles: Vec<String>,
    pub write_roles: Vec<String>,
    pub delete_roles: Vec<String>,
    pub meta_read_roles: Vec<String>,
    pub meta_write_roles: Vec<String>,
}

/// Aggregate snapshot for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateSnapshot {
    pub stream_id: String,
    pub aggregate_type: String,
    pub version: u64,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Event stream information
#[derive(Debug, Clone)]
pub struct EventStream {
    pub stream_id: String,
    pub version: u64,
    pub events: Vec<EventEnvelope>,
    pub metadata: Option<StreamMetadata>,
    pub is_deleted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl EventStream {
    pub fn new(stream_id: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            stream_id,
            version: 0,
            events: Vec::new(),
            metadata: None,
            is_deleted: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn append_events(&mut self, events: Vec<EventEnvelope>) -> EventResult<u64> {
        for event in events {
            if event.version != self.version + 1 {
                return Err(EventError::StorageError(format!(
                    "Version mismatch: expected {}, got {}",
                    self.version + 1,
                    event.version
                )));
            }
            self.events.push(event);
            self.version += 1;
        }
        self.updated_at = chrono::Utc::now();
        Ok(self.version)
    }

    pub fn get_events_from_version(&self, from_version: u64, max_count: Option<usize>) -> Vec<EventEnvelope> {
        let start_index = if from_version == 0 { 0 } else { from_version as usize - 1 };
        let events = &self.events[start_index..];
        
        if let Some(max) = max_count {
            events.iter().take(max).cloned().collect()
        } else {
            events.to_vec()
        }
    }
}

/// In-memory event store implementation
pub struct InMemoryEventStore {
    streams: Arc<RwLock<HashMap<String, EventStream>>>,
    snapshots: Arc<RwLock<HashMap<String, AggregateSnapshot>>>,
    global_position: Arc<RwLock<u64>>,
    all_events: Arc<RwLock<Vec<(u64, EventEnvelope)>>>, // (position, event)
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            global_position: Arc::new(RwLock::new(0)),
            all_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get or create a stream
    async fn get_or_create_stream(&self, stream_id: &str) -> EventStream {
        let mut streams = self.streams.write().await;
        streams.entry(stream_id.to_string())
            .or_insert_with(|| EventStream::new(stream_id.to_string()))
            .clone()
    }

    /// Validate expected version
    fn validate_expected_version(current_version: u64, expected_version: Option<u64>) -> EventResult<()> {
        if let Some(expected) = expected_version {
            if current_version != expected {
                return Err(EventError::StorageError(format!(
                    "Concurrency conflict: expected version {}, current version {}",
                    expected, current_version
                )));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append_events(&self, stream_id: &str, expected_version: Option<u64>, events: Vec<EventEnvelope>) -> EventResult<u64> {
        if events.is_empty() {
            return Err(EventError::StorageError("No events to append".to_string()));
        }

        debug!(
            stream_id = %stream_id,
            event_count = %events.len(),
            expected_version = ?expected_version,
            "Appending events to stream"
        );

        let mut streams = self.streams.write().await;
        let mut stream = streams.entry(stream_id.to_string())
            .or_insert_with(|| EventStream::new(stream_id.to_string()));

        // Validate expected version for optimistic concurrency control
        Self::validate_expected_version(stream.version, expected_version)?;

        // Validate event versions
        for (i, event) in events.iter().enumerate() {
            let expected_event_version = stream.version + i as u64 + 1;
            if event.version != expected_event_version {
                return Err(EventError::StorageError(format!(
                    "Event version mismatch: expected {}, got {}",
                    expected_event_version, event.version
                )));
            }
        }

        // Append events to stream
        let new_version = stream.append_events(events.clone())?;

        // Add events to global event log
        let mut global_position = self.global_position.write().await;
        let mut all_events = self.all_events.write().await;
        
        for event in events {
            *global_position += 1;
            all_events.push((*global_position, event));
        }

        info!(
            stream_id = %stream_id,
            new_version = %new_version,
            global_position = %*global_position,
            "Events appended successfully"
        );

        Ok(new_version)
    }

    async fn read_stream(&self, stream_id: &str, from_version: Option<u64>, max_count: Option<usize>) -> EventResult<Vec<EventEnvelope>> {
        debug!(
            stream_id = %stream_id,
            from_version = ?from_version,
            max_count = ?max_count,
            "Reading events from stream"
        );

        let streams = self.streams.read().await;
        
        if let Some(stream) = streams.get(stream_id) {
            if stream.is_deleted {
                return Err(EventError::StorageError(format!("Stream {} is deleted", stream_id)));
            }

            let from_ver = from_version.unwrap_or(1);
            let events = stream.get_events_from_version(from_ver, max_count);
            
            debug!(
                stream_id = %stream_id,
                events_count = %events.len(),
                "Events read from stream"
            );
            
            Ok(events)
        } else {
            Ok(Vec::new())
        }
    }

    async fn read_all_events(&self, from_position: Option<u64>, max_count: Option<usize>) -> EventResult<Vec<EventEnvelope>> {
        debug!(
            from_position = ?from_position,
            max_count = ?max_count,
            "Reading all events"
        );

        let all_events = self.all_events.read().await;
        let start_pos = from_position.unwrap_or(1);
        
        let events: Vec<EventEnvelope> = all_events
            .iter()
            .filter(|(pos, _)| *pos >= start_pos)
            .map(|(_, event)| event.clone())
            .take(max_count.unwrap_or(usize::MAX))
            .collect();

        debug!(events_count = %events.len(), "All events read");
        Ok(events)
    }

    async fn get_stream_version(&self, stream_id: &str) -> EventResult<Option<u64>> {
        let streams = self.streams.read().await;
        Ok(streams.get(stream_id).map(|stream| stream.version))
    }

    async fn stream_exists(&self, stream_id: &str) -> EventResult<bool> {
        let streams = self.streams.read().await;
        Ok(streams.contains_key(stream_id) && !streams[stream_id].is_deleted)
    }

    async fn delete_stream(&self, stream_id: &str, expected_version: Option<u64>) -> EventResult<()> {
        info!(stream_id = %stream_id, "Deleting stream");

        let mut streams = self.streams.write().await;
        
        if let Some(stream) = streams.get_mut(stream_id) {
            Self::validate_expected_version(stream.version, expected_version)?;
            stream.is_deleted = true;
            stream.updated_at = chrono::Utc::now();
            
            info!(stream_id = %stream_id, "Stream deleted");
            Ok(())
        } else {
            Err(EventError::StorageError(format!("Stream not found: {}", stream_id)))
        }
    }

    async fn get_stream_metadata(&self, stream_id: &str) -> EventResult<Option<StreamMetadata>> {
        let streams = self.streams.read().await;
        Ok(streams.get(stream_id).and_then(|stream| stream.metadata.clone()))
    }

    async fn set_stream_metadata(&self, stream_id: &str, metadata: StreamMetadata) -> EventResult<()> {
        info!(stream_id = %stream_id, "Setting stream metadata");

        let mut streams = self.streams.write().await;
        let stream = streams.entry(stream_id.to_string())
            .or_insert_with(|| EventStream::new(stream_id.to_string()));
        
        stream.metadata = Some(metadata);
        stream.updated_at = chrono::Utc::now();
        
        info!(stream_id = %stream_id, "Stream metadata set");
        Ok(())
    }

    async fn save_snapshot(&self, stream_id: &str, version: u64, snapshot: AggregateSnapshot) -> EventResult<()> {
        info!(
            stream_id = %stream_id,
            version = %version,
            "Saving aggregate snapshot"
        );

        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(stream_id.to_string(), snapshot);
        
        info!(stream_id = %stream_id, "Snapshot saved");
        Ok(())
    }

    async fn load_snapshot(&self, stream_id: &str) -> EventResult<Option<AggregateSnapshot>> {
        debug!(stream_id = %stream_id, "Loading aggregate snapshot");

        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots.get(stream_id).cloned();
        
        if snapshot.is_some() {
            debug!(stream_id = %stream_id, "Snapshot loaded");
        } else {
            debug!(stream_id = %stream_id, "No snapshot found");
        }
        
        Ok(snapshot)
    }
}

/// Event store builder for configuration
pub struct EventStoreBuilder {
    config: EventStoreConfig,
}

impl EventStoreBuilder {
    pub fn new() -> Self {
        Self {
            config: EventStoreConfig::default(),
        }
    }

    pub fn with_snapshot_frequency(mut self, frequency: u64) -> Self {
        self.config.snapshot_frequency = frequency;
        self
    }

    pub fn with_max_events_per_stream(mut self, max_events: u64) -> Self {
        self.config.max_events_per_stream = Some(max_events);
        self
    }

    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.config.enable_compression = enabled;
        self
    }

    pub fn build(self) -> InMemoryEventStore {
        InMemoryEventStore::new()
    }
}

/// Event store configuration
#[derive(Debug, Clone)]
pub struct EventStoreConfig {
    pub snapshot_frequency: u64,
    pub max_events_per_stream: Option<u64>,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub retention_policy: Option<chrono::Duration>,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            snapshot_frequency: 100,
            max_events_per_stream: None,
            enable_compression: false,
            enable_encryption: false,
            retention_policy: None,
        }
    }
}

/// Event store statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreStats {
    pub total_streams: u64,
    pub total_events: u64,
    pub total_snapshots: u64,
    pub average_events_per_stream: f64,
    pub storage_size_bytes: u64,
    pub last_event_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl InMemoryEventStore {
    /// Get event store statistics
    pub async fn get_stats(&self) -> EventStoreStats {
        let streams = self.streams.read().await;
        let snapshots = self.snapshots.read().await;
        let all_events = self.all_events.read().await;

        let total_streams = streams.len() as u64;
        let total_events = all_events.len() as u64;
        let total_snapshots = snapshots.len() as u64;
        
        let average_events_per_stream = if total_streams > 0 {
            total_events as f64 / total_streams as f64
        } else {
            0.0
        };

        let last_event_timestamp = all_events.last()
            .map(|(_, event)| event.timestamp);

        EventStoreStats {
            total_streams,
            total_events,
            total_snapshots,
            average_events_per_stream,
            storage_size_bytes: 0, // Would calculate actual size in real implementation
            last_event_timestamp,
        }
    }

    /// Compact a stream by creating a snapshot and removing old events
    pub async fn compact_stream(&self, stream_id: &str, keep_events: u64) -> EventResult<()> {
        info!(
            stream_id = %stream_id,
            keep_events = %keep_events,
            "Compacting stream"
        );

        let mut streams = self.streams.write().await;
        
        if let Some(stream) = streams.get_mut(stream_id) {
            if stream.events.len() as u64 > keep_events {
                let events_to_remove = stream.events.len() as u64 - keep_events;
                stream.events.drain(0..events_to_remove as usize);
                stream.updated_at = chrono::Utc::now();
                
                info!(
                    stream_id = %stream_id,
                    events_removed = %events_to_remove,
                    "Stream compacted"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_events::UserRegistered;

    #[tokio::test]
    async fn test_event_store_append_and_read() {
        let store = InMemoryEventStore::new();
        let stream_id = "user-123";

        // Create test event
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

        let envelope = EventEnvelope::new(
            &event,
            "User".to_string(),
            None,
            None,
            None,
        ).unwrap();

        // Append event
        let version = store.append_events(stream_id, Some(0), vec![envelope.clone()]).await.unwrap();
        assert_eq!(version, 1);

        // Read events
        let events = store.read_stream(stream_id, None, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_id, envelope.event_id);
    }

    #[tokio::test]
    async fn test_event_store_concurrency_control() {
        let store = InMemoryEventStore::new();
        let stream_id = "user-456";

        // Create test event
        let event = UserRegistered {
            event_id: Uuid::new_v4().to_string(),
            user_id: "user456".to_string(),
            email: "test@example.com".to_string(),
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            timestamp: chrono::Utc::now(),
            version: 1,
            metadata: HashMap::new(),
        };

        let envelope = EventEnvelope::new(
            &event,
            "User".to_string(),
            None,
            None,
            None,
        ).unwrap();

        // First append should succeed
        store.append_events(stream_id, Some(0), vec![envelope.clone()]).await.unwrap();

        // Second append with wrong expected version should fail
        let result = store.append_events(stream_id, Some(0), vec![envelope]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_store_snapshots() {
        let store = InMemoryEventStore::new();
        let stream_id = "user-789";

        // Create snapshot
        let snapshot = AggregateSnapshot {
            stream_id: stream_id.to_string(),
            aggregate_type: "User".to_string(),
            version: 10,
            data: serde_json::json!({"name": "John Doe", "email": "john@example.com"}),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        // Save snapshot
        store.save_snapshot(stream_id, 10, snapshot.clone()).await.unwrap();

        // Load snapshot
        let loaded_snapshot = store.load_snapshot(stream_id).await.unwrap();
        assert!(loaded_snapshot.is_some());
        assert_eq!(loaded_snapshot.unwrap().version, 10);
    }

    #[tokio::test]
    async fn test_event_store_metadata() {
        let store = InMemoryEventStore::new();
        let stream_id = "user-metadata";

        // Create metadata
        let metadata = StreamMetadata {
            stream_id: stream_id.to_string(),
            max_age: Some(chrono::Duration::days(30)),
            max_count: Some(1000),
            cache_control: None,
            acl: None,
            custom_metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Set metadata
        store.set_stream_metadata(stream_id, metadata.clone()).await.unwrap();

        // Get metadata
        let loaded_metadata = store.get_stream_metadata(stream_id).await.unwrap();
        assert!(loaded_metadata.is_some());
        assert_eq!(loaded_metadata.unwrap().max_count, Some(1000));
    }

    #[tokio::test]
    async fn test_event_store_stats() {
        let store = InMemoryEventStore::new();

        // Initially empty
        let stats = store.get_stats().await;
        assert_eq!(stats.total_streams, 0);
        assert_eq!(stats.total_events, 0);

        // Add some events
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

        let envelope = EventEnvelope::new(&event, "User".to_string(), None, None, None).unwrap();
        store.append_events("user-123", Some(0), vec![envelope]).await.unwrap();

        // Check updated stats
        let stats = store.get_stats().await;
        assert_eq!(stats.total_streams, 1);
        assert_eq!(stats.total_events, 1);
    }
}
