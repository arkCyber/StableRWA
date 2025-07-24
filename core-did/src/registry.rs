// =====================================================================================
// DID Registry Implementation
//
// Registry for storing and managing DID documents
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{DidDocument, DidError, DidMetadata, DidResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// DID registry trait
#[async_trait]
pub trait DidRegistry: Send + Sync {
    /// Create a new DID document
    async fn create(&self, did: String, document: DidDocument) -> DidResult<()>;

    /// Read a DID document
    async fn read(&self, did: &str) -> DidResult<Option<DidDocument>>;

    /// Update a DID document
    async fn update(&self, did: String, document: DidDocument) -> DidResult<()>;

    /// Deactivate a DID
    async fn deactivate(&self, did: &str) -> DidResult<()>;

    /// Get DID metadata
    async fn get_metadata(&self, did: &str) -> DidResult<Option<DidMetadata>>;

    /// List all DIDs (for admin purposes)
    async fn list_dids(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> DidResult<Vec<String>>;
}

/// In-memory DID registry for development and testing
#[derive(Debug)]
pub struct MemoryRegistry {
    /// Stored DID documents
    documents: Arc<RwLock<HashMap<String, DidDocument>>>,
    /// DID metadata
    metadata: Arc<RwLock<HashMap<String, DidMetadata>>>,
}

impl MemoryRegistry {
    /// Create a new memory registry
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Clear all data
    pub async fn clear(&self) {
        let mut documents = self.documents.write().await;
        let mut metadata = self.metadata.write().await;
        documents.clear();
        metadata.clear();
    }

    /// Get document count
    pub async fn count(&self) -> usize {
        let documents = self.documents.read().await;
        documents.len()
    }
}

#[async_trait]
impl DidRegistry for MemoryRegistry {
    async fn create(&self, did: String, document: DidDocument) -> DidResult<()> {
        // Validate document
        document.validate()?;

        let mut documents = self.documents.write().await;
        let mut metadata = self.metadata.write().await;

        // Check if DID already exists
        if documents.contains_key(&did) {
            return Err(DidError::DidAlreadyExists(did));
        }

        // Store document and metadata
        documents.insert(did.clone(), document);
        metadata.insert(did, DidMetadata::default());

        Ok(())
    }

    async fn read(&self, did: &str) -> DidResult<Option<DidDocument>> {
        let documents = self.documents.read().await;
        let metadata = self.metadata.read().await;

        // Check if DID is deactivated
        if let Some(meta) = metadata.get(did) {
            if meta.deactivated {
                return Err(DidError::DidDeactivated(did.to_string()));
            }
        }

        Ok(documents.get(did).cloned())
    }

    async fn update(&self, did: String, document: DidDocument) -> DidResult<()> {
        // Validate document
        document.validate()?;

        let mut documents = self.documents.write().await;
        let mut metadata = self.metadata.write().await;

        // Check if DID exists
        if !documents.contains_key(&did) {
            return Err(DidError::DidNotFound(did));
        }

        // Check if DID is deactivated
        if let Some(meta) = metadata.get(&did) {
            if meta.deactivated {
                return Err(DidError::DidDeactivated(did));
            }
        }

        // Update document and metadata
        documents.insert(did.clone(), document);
        if let Some(meta) = metadata.get_mut(&did) {
            meta.updated = Utc::now();
            meta.version += 1;
        }

        Ok(())
    }

    async fn deactivate(&self, did: &str) -> DidResult<()> {
        let mut metadata = self.metadata.write().await;

        // Check if DID exists
        if !metadata.contains_key(did) {
            return Err(DidError::DidNotFound(did.to_string()));
        }

        // Mark as deactivated
        if let Some(meta) = metadata.get_mut(did) {
            meta.deactivated = true;
            meta.updated = Utc::now();
            meta.version += 1;
        }

        Ok(())
    }

    async fn get_metadata(&self, did: &str) -> DidResult<Option<DidMetadata>> {
        let metadata = self.metadata.read().await;
        Ok(metadata.get(did).cloned())
    }

    async fn list_dids(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> DidResult<Vec<String>> {
        let documents = self.documents.read().await;
        let mut dids: Vec<String> = documents.keys().cloned().collect();
        dids.sort();

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(dids.len());

        Ok(dids.into_iter().skip(offset).take(limit).collect())
    }
}

impl Default for MemoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry operation for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryOperation {
    /// Operation ID
    pub id: String,
    /// Operation type
    pub operation_type: RegistryOperationType,
    /// Target DID
    pub did: String,
    /// Operation timestamp
    pub timestamp: DateTime<Utc>,
    /// Operation result
    pub result: RegistryOperationResult,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Registry operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryOperationType {
    /// Create operation
    Create,
    /// Read operation
    Read,
    /// Update operation
    Update,
    /// Deactivate operation
    Deactivate,
}

/// Registry operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryOperationResult {
    /// Operation succeeded
    Success,
    /// Operation failed
    Failed { error: String },
}

/// Auditable registry wrapper
pub struct AuditableRegistry {
    /// Underlying registry
    registry: Arc<dyn DidRegistry>,
    /// Operation log
    operations: Arc<RwLock<Vec<RegistryOperation>>>,
}

impl AuditableRegistry {
    /// Create a new auditable registry
    pub fn new(registry: Arc<dyn DidRegistry>) -> Self {
        Self {
            registry,
            operations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get operation history
    pub async fn get_operations(&self) -> Vec<RegistryOperation> {
        let operations = self.operations.read().await;
        operations.clone()
    }

    /// Get operations for a specific DID
    pub async fn get_operations_for_did(&self, did: &str) -> Vec<RegistryOperation> {
        let operations = self.operations.read().await;
        operations
            .iter()
            .filter(|op| op.did == did)
            .cloned()
            .collect()
    }

    /// Log an operation
    async fn log_operation(
        &self,
        operation_type: RegistryOperationType,
        did: String,
        result: RegistryOperationResult,
    ) {
        let operation = RegistryOperation {
            id: uuid::Uuid::new_v4().to_string(),
            operation_type,
            did,
            timestamp: Utc::now(),
            result,
            metadata: HashMap::new(),
        };

        let mut operations = self.operations.write().await;
        operations.push(operation);
    }
}

#[async_trait]
impl DidRegistry for AuditableRegistry {
    async fn create(&self, did: String, document: DidDocument) -> DidResult<()> {
        let result = self.registry.create(did.clone(), document).await;

        let operation_result = match &result {
            Ok(()) => RegistryOperationResult::Success,
            Err(e) => RegistryOperationResult::Failed {
                error: e.to_string(),
            },
        };

        self.log_operation(RegistryOperationType::Create, did, operation_result)
            .await;
        result
    }

    async fn read(&self, did: &str) -> DidResult<Option<DidDocument>> {
        let result = self.registry.read(did).await;

        let operation_result = match &result {
            Ok(_) => RegistryOperationResult::Success,
            Err(e) => RegistryOperationResult::Failed {
                error: e.to_string(),
            },
        };

        self.log_operation(
            RegistryOperationType::Read,
            did.to_string(),
            operation_result,
        )
        .await;
        result
    }

    async fn update(&self, did: String, document: DidDocument) -> DidResult<()> {
        let result = self.registry.update(did.clone(), document).await;

        let operation_result = match &result {
            Ok(()) => RegistryOperationResult::Success,
            Err(e) => RegistryOperationResult::Failed {
                error: e.to_string(),
            },
        };

        self.log_operation(RegistryOperationType::Update, did, operation_result)
            .await;
        result
    }

    async fn deactivate(&self, did: &str) -> DidResult<()> {
        let result = self.registry.deactivate(did).await;

        let operation_result = match &result {
            Ok(()) => RegistryOperationResult::Success,
            Err(e) => RegistryOperationResult::Failed {
                error: e.to_string(),
            },
        };

        self.log_operation(
            RegistryOperationType::Deactivate,
            did.to_string(),
            operation_result,
        )
        .await;
        result
    }

    async fn get_metadata(&self, did: &str) -> DidResult<Option<DidMetadata>> {
        self.registry.get_metadata(did).await
    }

    async fn list_dids(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> DidResult<Vec<String>> {
        self.registry.list_dids(limit, offset).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Did, DidDocument};

    #[tokio::test]
    async fn test_memory_registry() {
        let registry = MemoryRegistry::new();

        // Create DID document
        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        // Test create
        registry
            .create(did.to_string(), document.clone())
            .await
            .unwrap();

        // Test read
        let retrieved = registry.read(&did.to_string()).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, did.to_string());

        // Test metadata
        let metadata = registry.get_metadata(&did.to_string()).await.unwrap();
        assert!(metadata.is_some());
        assert_eq!(metadata.unwrap().version, 1);

        // Test deactivate
        registry.deactivate(&did.to_string()).await.unwrap();

        // Should fail to read deactivated DID
        assert!(registry.read(&did.to_string()).await.is_err());
    }

    #[tokio::test]
    async fn test_auditable_registry() {
        let memory_registry = Arc::new(MemoryRegistry::new());
        let auditable_registry = AuditableRegistry::new(memory_registry);

        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        // Perform operations
        auditable_registry
            .create(did.to_string(), document)
            .await
            .unwrap();
        auditable_registry.read(&did.to_string()).await.unwrap();

        // Check audit log
        let operations = auditable_registry.get_operations().await;
        assert_eq!(operations.len(), 2);

        let did_operations = auditable_registry
            .get_operations_for_did(&did.to_string())
            .await;
        assert_eq!(did_operations.len(), 2);
    }
}
