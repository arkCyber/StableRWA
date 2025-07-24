// =====================================================================================
// IPFS Storage Management
//
// High-level storage operations for the RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    client::{IpfsClient, IpfsClientFileOps, IpfsClientTrait},
    ContentMetadata, IpfsConfig, IpfsError, IpfsHash, IpfsResult, StorageStats,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Storage operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult {
    pub hash: IpfsHash,
    pub metadata: ContentMetadata,
    pub operation_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Storage manager trait
#[async_trait(?Send)]
pub trait StorageManagerTrait {
    /// Store content with metadata
    async fn store_content(
        &self,
        data: Vec<u8>,
        metadata: ContentMetadata,
    ) -> IpfsResult<StorageResult>;

    /// Store file with automatic metadata generation
    async fn store_file<P: AsRef<Path> + Send>(
        &self,
        path: P,
        name: Option<String>,
    ) -> IpfsResult<StorageResult>;

    /// Retrieve content by hash
    async fn retrieve_content(&self, hash: &IpfsHash) -> IpfsResult<Vec<u8>>;

    /// Retrieve content metadata
    async fn get_metadata(&self, hash: &IpfsHash) -> IpfsResult<ContentMetadata>;

    /// Update content metadata
    async fn update_metadata(&self, hash: &IpfsHash, metadata: ContentMetadata) -> IpfsResult<()>;

    /// Delete content (unpin and remove metadata)
    async fn delete_content(&self, hash: &IpfsHash) -> IpfsResult<()>;

    /// List all stored content
    async fn list_content(&self) -> IpfsResult<Vec<ContentMetadata>>;

    /// Get storage statistics
    async fn get_stats(&self) -> IpfsResult<StorageStats>;

    /// Search content by tags
    async fn search_by_tags(&self, tags: &[String]) -> IpfsResult<Vec<ContentMetadata>>;

    /// Search content by MIME type
    async fn search_by_mime_type(&self, mime_type: &str) -> IpfsResult<Vec<ContentMetadata>>;
}

/// IPFS storage manager implementation
pub struct StorageManager {
    client: Arc<dyn IpfsClientTrait>,
    metadata_store: Arc<RwLock<HashMap<String, ContentMetadata>>>,
    config: IpfsConfig,
}

impl StorageManager {
    /// Create new storage manager
    pub fn new(client: Arc<dyn IpfsClientTrait>, config: IpfsConfig) -> Self {
        Self {
            client,
            metadata_store: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create storage manager with default IPFS client
    pub fn with_default_client(config: IpfsConfig) -> IpfsResult<Self> {
        let client = Arc::new(IpfsClient::new(config.clone())?);
        Ok(Self::new(client, config))
    }

    /// Generate metadata from file path
    async fn generate_file_metadata<P: AsRef<Path>>(
        &self,
        path: P,
        name: Option<String>,
        hash: IpfsHash,
    ) -> IpfsResult<ContentMetadata> {
        let path = path.as_ref();

        // Get file metadata
        let file_metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| IpfsError::StorageError(format!("Failed to read file metadata: {}", e)))?;

        // Determine name
        let file_name = name.unwrap_or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        // Guess MIME type
        let mime_type = mime_guess::from_path(path)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        Ok(ContentMetadata::new(
            hash,
            file_name,
            file_metadata.len(),
            mime_type,
        ))
    }

    /// Validate content against configuration
    fn validate_content(&self, metadata: &ContentMetadata) -> IpfsResult<()> {
        // Check file size
        if metadata.size > self.config.max_file_size {
            return Err(IpfsError::ValidationError(format!(
                "File size {} exceeds maximum allowed size {}",
                metadata.size, self.config.max_file_size
            )));
        }

        // Check MIME type
        if !self.config.allowed_mime_types.is_empty()
            && !self.config.allowed_mime_types.contains(&metadata.mime_type)
        {
            return Err(IpfsError::ValidationError(format!(
                "MIME type {} is not allowed",
                metadata.mime_type
            )));
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl StorageManagerTrait for StorageManager {
    #[instrument(skip(self, data, metadata))]
    async fn store_content(
        &self,
        data: Vec<u8>,
        mut metadata: ContentMetadata,
    ) -> IpfsResult<StorageResult> {
        debug!(
            "Storing content: {} bytes, name: {}",
            data.len(),
            metadata.name
        );

        // Validate content
        self.validate_content(&metadata)?;

        // Store in IPFS
        let hash = self.client.add_bytes(data).await?;

        // Update metadata with actual hash
        metadata.hash = hash.clone();

        // Store metadata
        self.metadata_store
            .write()
            .await
            .insert(hash.as_str().to_string(), metadata.clone());

        let result = StorageResult {
            hash,
            metadata,
            operation_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
        };

        info!("Successfully stored content: {}", result.hash);
        Ok(result)
    }

    async fn store_file<P: AsRef<Path> + Send>(
        &self,
        path: P,
        name: Option<String>,
    ) -> IpfsResult<StorageResult> {
        let path = path.as_ref();
        debug!("Storing file: {}", path.display());

        // Store file in IPFS - read file and use add_bytes
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| IpfsError::StorageError(format!("Failed to read file: {}", e)))?;
        let hash = self.client.add_bytes(data).await?;

        // Generate metadata
        let metadata = self
            .generate_file_metadata(path, name, hash.clone())
            .await?;

        // Validate content
        self.validate_content(&metadata)?;

        // Store metadata
        self.metadata_store
            .write()
            .await
            .insert(hash.as_str().to_string(), metadata.clone());

        let result = StorageResult {
            hash,
            metadata,
            operation_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
        };

        info!(
            "Successfully stored file: {} -> {}",
            path.display(),
            result.hash
        );
        Ok(result)
    }

    #[instrument(skip(self))]
    async fn retrieve_content(&self, hash: &IpfsHash) -> IpfsResult<Vec<u8>> {
        debug!("Retrieving content: {}", hash);

        let data = self.client.get_bytes(hash).await?;

        info!("Successfully retrieved {} bytes for {}", data.len(), hash);
        Ok(data)
    }

    #[instrument(skip(self))]
    async fn get_metadata(&self, hash: &IpfsHash) -> IpfsResult<ContentMetadata> {
        debug!("Getting metadata for: {}", hash);

        self.metadata_store
            .read()
            .await
            .get(hash.as_str())
            .cloned()
            .ok_or_else(|| {
                IpfsError::MetadataError(format!("Metadata not found for hash: {}", hash))
            })
    }

    #[instrument(skip(self, metadata))]
    async fn update_metadata(&self, hash: &IpfsHash, metadata: ContentMetadata) -> IpfsResult<()> {
        debug!("Updating metadata for: {}", hash);

        // Verify content exists
        if !self.client.exists(hash).await? {
            return Err(IpfsError::FileNotFound(format!(
                "Content not found: {}",
                hash
            )));
        }

        // Update metadata store
        self.metadata_store
            .write()
            .await
            .insert(hash.as_str().to_string(), metadata);

        info!("Successfully updated metadata for: {}", hash);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete_content(&self, hash: &IpfsHash) -> IpfsResult<()> {
        debug!("Deleting content: {}", hash);

        // Unpin content
        if let Err(e) = self.client.unpin(hash).await {
            warn!("Failed to unpin content {}: {}", hash, e);
        }

        // Remove metadata
        self.metadata_store.write().await.remove(hash.as_str());

        info!("Successfully deleted content: {}", hash);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_content(&self) -> IpfsResult<Vec<ContentMetadata>> {
        debug!("Listing all content");

        let metadata_store = self.metadata_store.read().await;
        let content: Vec<ContentMetadata> = metadata_store.values().cloned().collect();

        info!("Found {} content items", content.len());
        Ok(content)
    }

    #[instrument(skip(self))]
    async fn get_stats(&self) -> IpfsResult<StorageStats> {
        debug!("Getting storage statistics");

        let metadata_store = self.metadata_store.read().await;
        let total_files = metadata_store.len() as u64;
        let total_size: u64 = metadata_store.values().map(|m| m.size).sum();

        // Get pinned content
        let pinned_hashes = self.client.list_pins().await?;
        let pinned_files = pinned_hashes.len() as u64;

        // Calculate pinned size
        let mut pinned_size = 0u64;
        for hash in &pinned_hashes {
            if let Some(metadata) = metadata_store.get(hash.as_str()) {
                pinned_size += metadata.size;
            }
        }

        let stats = StorageStats {
            total_files,
            total_size,
            pinned_files,
            pinned_size,
            last_updated: chrono::Utc::now(),
        };

        info!(
            "Storage stats: {} files, {} bytes total",
            stats.total_files, stats.total_size
        );
        Ok(stats)
    }

    #[instrument(skip(self))]
    async fn search_by_tags(&self, tags: &[String]) -> IpfsResult<Vec<ContentMetadata>> {
        debug!("Searching content by tags: {:?}", tags);

        let metadata_store = self.metadata_store.read().await;
        let results: Vec<ContentMetadata> = metadata_store
            .values()
            .filter(|metadata| tags.iter().any(|tag| metadata.tags.contains(tag)))
            .cloned()
            .collect();

        info!("Found {} items matching tags: {:?}", results.len(), tags);
        Ok(results)
    }

    #[instrument(skip(self))]
    async fn search_by_mime_type(&self, mime_type: &str) -> IpfsResult<Vec<ContentMetadata>> {
        debug!("Searching content by MIME type: {}", mime_type);

        let metadata_store = self.metadata_store.read().await;
        let results: Vec<ContentMetadata> = metadata_store
            .values()
            .filter(|metadata| metadata.mime_type == mime_type)
            .cloned()
            .collect();

        info!(
            "Found {} items with MIME type: {}",
            results.len(),
            mime_type
        );
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockIpfsClient;
    use tempfile::NamedTempFile;

    async fn create_test_storage_manager() -> StorageManager {
        let client = Arc::new(MockIpfsClient::new());
        let config = IpfsConfig::default();
        StorageManager::new(client, config)
    }

    #[tokio::test]
    async fn test_store_and_retrieve_content() {
        let manager = create_test_storage_manager().await;
        let data = b"Test content for storage".to_vec();

        let hash =
            IpfsHash::new("QmTestHash123456789012345678901234567890123456".to_string()).unwrap();
        let metadata = ContentMetadata::new(
            hash.clone(),
            "test.txt".to_string(),
            data.len() as u64,
            "text/plain".to_string(),
        );

        // Store content
        let result = manager
            .store_content(data.clone(), metadata.clone())
            .await
            .unwrap();
        assert_eq!(result.metadata.name, "test.txt");
        assert_eq!(result.metadata.size, data.len() as u64);

        // Retrieve content
        let retrieved = manager.retrieve_content(&result.hash).await.unwrap();
        assert_eq!(data, retrieved);

        // Get metadata
        let stored_metadata = manager.get_metadata(&result.hash).await.unwrap();
        assert_eq!(stored_metadata.name, metadata.name);
        assert_eq!(stored_metadata.mime_type, metadata.mime_type);
    }

    #[tokio::test]
    async fn test_store_file() {
        let manager = create_test_storage_manager().await;

        // Create temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"File content for testing";
        std::io::Write::write_all(&mut temp_file, test_data).unwrap();
        std::io::Write::flush(&mut temp_file).unwrap();

        // Store file
        let result = manager
            .store_file(temp_file.path(), Some("my_test_file.txt".to_string()))
            .await
            .unwrap();
        assert_eq!(result.metadata.name, "my_test_file.txt");
        assert_eq!(result.metadata.size, test_data.len() as u64);

        // Retrieve and verify
        let retrieved = manager.retrieve_content(&result.hash).await.unwrap();
        assert_eq!(test_data.to_vec(), retrieved);
    }

    #[tokio::test]
    async fn test_metadata_operations() {
        let manager = create_test_storage_manager().await;
        let data = b"Metadata test".to_vec();

        let hash =
            IpfsHash::new("QmMetadataTest123456789012345678901234567890".to_string()).unwrap();
        let mut metadata = ContentMetadata::new(
            hash.clone(),
            "metadata_test.txt".to_string(),
            data.len() as u64,
            "text/plain".to_string(),
        );

        // Store content
        let result = manager.store_content(data, metadata.clone()).await.unwrap();

        // Update metadata
        metadata.add_tag("test".to_string());
        metadata.add_tag("metadata".to_string());
        metadata.add_custom_field(
            "author".to_string(),
            serde_json::Value::String("test_user".to_string()),
        );

        manager
            .update_metadata(&result.hash, metadata.clone())
            .await
            .unwrap();

        // Verify updated metadata
        let updated_metadata = manager.get_metadata(&result.hash).await.unwrap();
        assert!(updated_metadata.tags.contains(&"test".to_string()));
        assert!(updated_metadata.tags.contains(&"metadata".to_string()));
        assert!(updated_metadata.custom_fields.contains_key("author"));
    }

    #[tokio::test]
    async fn test_list_and_search_content() {
        let manager = create_test_storage_manager().await;

        // Store multiple content items
        let mut metadata1 = ContentMetadata::new(
            IpfsHash::new("QmTest1234567890123456789012345678901234567890".to_string()).unwrap(),
            "image1.jpg".to_string(),
            1024,
            "image/jpeg".to_string(),
        );
        metadata1.add_tag("photo".to_string());
        metadata1.add_tag("vacation".to_string());

        let mut metadata2 = ContentMetadata::new(
            IpfsHash::new("QmTest2345678901234567890123456789012345678901".to_string()).unwrap(),
            "document.pdf".to_string(),
            2048,
            "application/pdf".to_string(),
        );
        metadata2.add_tag("document".to_string());
        metadata2.add_tag("work".to_string());

        let mut metadata3 = ContentMetadata::new(
            IpfsHash::new("QmTest3456789012345678901234567890123456789012".to_string()).unwrap(),
            "image2.jpg".to_string(),
            1536,
            "image/jpeg".to_string(),
        );
        metadata3.add_tag("photo".to_string());
        metadata3.add_tag("family".to_string());

        manager
            .store_content(b"image1".to_vec(), metadata1)
            .await
            .unwrap();
        manager
            .store_content(b"document".to_vec(), metadata2)
            .await
            .unwrap();
        manager
            .store_content(b"image2".to_vec(), metadata3)
            .await
            .unwrap();

        // List all content
        let all_content = manager.list_content().await.unwrap();
        assert_eq!(all_content.len(), 3);

        // Search by tags
        let photo_content = manager
            .search_by_tags(&["photo".to_string()])
            .await
            .unwrap();
        assert_eq!(photo_content.len(), 2);

        let work_content = manager.search_by_tags(&["work".to_string()]).await.unwrap();
        assert_eq!(work_content.len(), 1);

        // Search by MIME type
        let images = manager.search_by_mime_type("image/jpeg").await.unwrap();
        assert_eq!(images.len(), 2);

        let pdfs = manager
            .search_by_mime_type("application/pdf")
            .await
            .unwrap();
        assert_eq!(pdfs.len(), 1);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let manager = create_test_storage_manager().await;

        // Store some content
        let metadata1 = ContentMetadata::new(
            IpfsHash::new("QmStats1234567890123456789012345678901234567890".to_string()).unwrap(),
            "file1.txt".to_string(),
            100,
            "text/plain".to_string(),
        );

        let metadata2 = ContentMetadata::new(
            IpfsHash::new("QmStats2345678901234567890123456789012345678901".to_string()).unwrap(),
            "file2.txt".to_string(),
            200,
            "text/plain".to_string(),
        );

        manager
            .store_content(b"content1".to_vec(), metadata1)
            .await
            .unwrap();
        manager
            .store_content(b"content2".to_vec(), metadata2)
            .await
            .unwrap();

        // Get stats
        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_size, 300);
    }

    #[tokio::test]
    async fn test_delete_content() {
        let manager = create_test_storage_manager().await;
        let data = b"Content to delete".to_vec();

        let hash =
            IpfsHash::new("QmDelete123456789012345678901234567890123456".to_string()).unwrap();
        let metadata = ContentMetadata::new(
            hash.clone(),
            "delete_me.txt".to_string(),
            data.len() as u64,
            "text/plain".to_string(),
        );

        // Store content
        let result = manager.store_content(data, metadata).await.unwrap();

        // Verify it exists
        assert!(manager.get_metadata(&result.hash).await.is_ok());

        // Delete content
        manager.delete_content(&result.hash).await.unwrap();

        // Verify it's gone from metadata store
        assert!(manager.get_metadata(&result.hash).await.is_err());
    }

    #[test]
    fn test_storage_manager_with_default_client() {
        let config = IpfsConfig::default();
        let manager = StorageManager::with_default_client(config);

        // Should succeed in creating the manager
        assert!(manager.is_ok());
    }

    #[test]
    fn test_storage_result() {
        let hash =
            IpfsHash::new("QmResult123456789012345678901234567890123456".to_string()).unwrap();
        let metadata = ContentMetadata::new(
            hash.clone(),
            "result_test.txt".to_string(),
            100,
            "text/plain".to_string(),
        );

        let result = StorageResult {
            hash: hash.clone(),
            metadata: metadata.clone(),
            operation_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(result.hash, hash);
        assert_eq!(result.metadata.name, "result_test.txt");
    }
}
