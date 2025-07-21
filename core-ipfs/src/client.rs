// =====================================================================================
// IPFS Client Implementation
// 
// Provides high-level IPFS client functionality for the RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{IpfsError, IpfsResult, IpfsHash, IpfsConfig, ContentMetadata};
use async_trait::async_trait;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient as IpfsApiClient, TryFromUri};
use ipfs_api_prelude::response::AddResponse;
use futures_util::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tracing::{debug, error, info, warn, instrument};

/// IPFS client trait for dependency injection and testing
#[async_trait(?Send)]
pub trait IpfsClientTrait {
    /// Add content to IPFS
    async fn add_bytes(&self, data: Vec<u8>) -> IpfsResult<IpfsHash>;

    /// Get content from IPFS
    async fn get_bytes(&self, hash: &IpfsHash) -> IpfsResult<Vec<u8>>;

    /// Pin content to prevent garbage collection
    async fn pin(&self, hash: &IpfsHash) -> IpfsResult<()>;

    /// Unpin content
    async fn unpin(&self, hash: &IpfsHash) -> IpfsResult<()>;

    /// Check if content exists
    async fn exists(&self, hash: &IpfsHash) -> IpfsResult<bool>;

    /// Get content size
    async fn size(&self, hash: &IpfsHash) -> IpfsResult<u64>;

    /// List pinned content
    async fn list_pins(&self) -> IpfsResult<Vec<IpfsHash>>;
}

/// Extended IPFS client trait with file operations (not dyn-compatible due to generics)
#[async_trait(?Send)]
pub trait IpfsClientFileOps: IpfsClientTrait {
    /// Add file to IPFS
    async fn add_file<P: AsRef<Path> + Send>(&self, path: P) -> IpfsResult<IpfsHash>;

    /// Get file from IPFS and save to path
    async fn get_file<P: AsRef<Path> + Send>(&self, hash: &IpfsHash, path: P) -> IpfsResult<()>;
}

/// IPFS client implementation
pub struct IpfsClient {
    client: IpfsApiClient,
    config: IpfsConfig,
}

impl IpfsClient {
    /// Create new IPFS client
    pub fn new(config: IpfsConfig) -> IpfsResult<Self> {
        let client = IpfsApiClient::from_str(&config.api_url)
            .map_err(|e| IpfsError::ClientError(format!("Failed to create IPFS client: {}", e)))?;
        
        Ok(Self { client, config })
    }
    
    /// Create client with default configuration
    pub fn with_default_config() -> IpfsResult<Self> {
        Self::new(IpfsConfig::default())
    }
    
    /// Get client configuration
    pub fn config(&self) -> &IpfsConfig {
        &self.config
    }
    
    /// Validate file size
    fn validate_file_size(&self, size: u64) -> IpfsResult<()> {
        if size > self.config.max_file_size {
            return Err(IpfsError::ValidationError(format!(
                "File size {} exceeds maximum allowed size {}",
                size, self.config.max_file_size
            )));
        }
        Ok(())
    }
    
    /// Validate MIME type
    fn validate_mime_type(&self, mime_type: &str) -> IpfsResult<()> {
        if !self.config.allowed_mime_types.is_empty() 
            && !self.config.allowed_mime_types.contains(&mime_type.to_string()) {
            return Err(IpfsError::ValidationError(format!(
                "MIME type {} is not allowed", mime_type
            )));
        }
        Ok(())
    }
    
    /// Convert IPFS API response to our hash type
    fn response_to_hash(response: AddResponse) -> IpfsResult<IpfsHash> {
        IpfsHash::new(response.hash)
    }
}

#[async_trait(?Send)]
impl IpfsClientTrait for IpfsClient {
    async fn add_bytes(&self, data: Vec<u8>) -> IpfsResult<IpfsHash> {
        debug!("Adding {} bytes to IPFS", data.len());
        
        // Validate file size
        self.validate_file_size(data.len() as u64)?;
        
        let cursor = Cursor::new(data);
        let response = self.client
            .add(cursor)
            .await
            .map_err(|e| IpfsError::ClientError(format!("Failed to add content: {}", e)))?;
        
        let hash = Self::response_to_hash(response)?;
        
        // Auto-pin if configured
        if self.config.auto_pin {
            if let Err(e) = self.pin(&hash).await {
                warn!("Failed to auto-pin content {}: {}", hash, e);
            }
        }
        
        info!("Successfully added content to IPFS: {}", hash);
        Ok(hash)
    }
    

    
    async fn get_bytes(&self, hash: &IpfsHash) -> IpfsResult<Vec<u8>> {
        debug!("Getting content from IPFS: {}", hash);
        
        let response = self.client
            .cat(hash.as_str())
            .map_ok(|chunk| chunk.to_vec())
            .try_concat()
            .await
            .map_err(|e| IpfsError::ClientError(format!("Failed to get content: {}", e)))?;
        
        info!("Successfully retrieved {} bytes from IPFS: {}", response.len(), hash);
        Ok(response)
    }
    

    
    async fn pin(&self, hash: &IpfsHash) -> IpfsResult<()> {
        debug!("Pinning content: {}", hash);
        
        self.client
            .pin_add(hash.as_str(), true)
            .await
            .map_err(|e| IpfsError::PinningError(format!("Failed to pin content: {}", e)))?;
        
        info!("Successfully pinned content: {}", hash);
        Ok(())
    }
    
    async fn unpin(&self, hash: &IpfsHash) -> IpfsResult<()> {
        debug!("Unpinning content: {}", hash);
        
        self.client
            .pin_rm(hash.as_str(), true)
            .await
            .map_err(|e| IpfsError::PinningError(format!("Failed to unpin content: {}", e)))?;
        
        info!("Successfully unpinned content: {}", hash);
        Ok(())
    }
    
    async fn exists(&self, hash: &IpfsHash) -> IpfsResult<bool> {
        debug!("Checking if content exists: {}", hash);
        
        match self.client.object_stat(hash.as_str()).await {
            Ok(_) => {
                debug!("Content exists: {}", hash);
                Ok(true)
            }
            Err(_) => {
                debug!("Content does not exist: {}", hash);
                Ok(false)
            }
        }
    }
    
    async fn size(&self, hash: &IpfsHash) -> IpfsResult<u64> {
        debug!("Getting content size: {}", hash);
        
        let stat = self.client
            .object_stat(hash.as_str())
            .await
            .map_err(|e| IpfsError::ClientError(format!("Failed to get content stats: {}", e)))?;
        
        let size = stat.cumulative_size;
        debug!("Content size: {} bytes for {}", size, hash);
        Ok(size)
    }
    
    async fn list_pins(&self) -> IpfsResult<Vec<IpfsHash>> {
        debug!("Listing pinned content");
        
        let pins = self.client
            .pin_ls(None, None)
            .await
            .map_err(|e| IpfsError::PinningError(format!("Failed to list pins: {}", e)))?;
        
        let hashes: Result<Vec<_>, _> = pins.keys
            .into_iter()
            .map(|(hash, _)| IpfsHash::new(hash))
            .collect();
        
        let hashes = hashes?;
        info!("Found {} pinned items", hashes.len());
        Ok(hashes)
    }
}

#[async_trait(?Send)]
impl IpfsClientFileOps for IpfsClient {
    async fn add_file<P: AsRef<Path> + Send>(&self, path: P) -> IpfsResult<IpfsHash> {
        let path = path.as_ref();
        debug!("Adding file to IPFS: {}", path.display());

        // Check if file exists
        if !path.exists() {
            return Err(IpfsError::FileNotFound(format!("File not found: {}", path.display())));
        }

        // Get file metadata
        let metadata = fs::metadata(path).await
            .map_err(|e| IpfsError::ClientError(format!("Failed to read file metadata: {}", e)))?;

        // Validate file size
        self.validate_file_size(metadata.len())?;

        // Guess MIME type and validate
        if let Some(mime_type) = mime_guess::from_path(path).first() {
            self.validate_mime_type(mime_type.as_ref())?;
        }

        // Read file content
        let data = fs::read(path).await
            .map_err(|e| IpfsError::ClientError(format!("Failed to read file: {}", e)))?;

        self.add_bytes(data).await
    }

    async fn get_file<P: AsRef<Path> + Send>(&self, hash: &IpfsHash, path: P) -> IpfsResult<()> {
        let path = path.as_ref();
        debug!("Getting file from IPFS {} to {}", hash, path.display());

        let data = self.get_bytes(hash).await?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| IpfsError::ClientError(format!("Failed to create directories: {}", e)))?;
        }

        fs::write(path, data).await
            .map_err(|e| IpfsError::ClientError(format!("Failed to write file: {}", e)))?;

        info!("Successfully saved file from IPFS: {} -> {}", hash, path.display());
        Ok(())
    }
}

/// Mock IPFS client for testing
#[cfg(test)]
pub struct MockIpfsClient {
    pub storage: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>>,
    pub pins: std::sync::Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
}

#[cfg(test)]
impl MockIpfsClient {
    pub fn new() -> Self {
        Self {
            storage: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            pins: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashSet::new())),
        }
    }
}

#[cfg(test)]
#[async_trait(?Send)]
impl IpfsClientTrait for MockIpfsClient {
    async fn add_bytes(&self, data: Vec<u8>) -> IpfsResult<IpfsHash> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash = format!("Qm{}", hex::encode(hasher.finalize())[..44].to_string());
        
        let ipfs_hash = IpfsHash::new(hash.clone())?;
        self.storage.write().await.insert(hash, data);
        Ok(ipfs_hash)
    }
    

    
    async fn get_bytes(&self, hash: &IpfsHash) -> IpfsResult<Vec<u8>> {
        self.storage.read().await
            .get(hash.as_str())
            .cloned()
            .ok_or_else(|| IpfsError::FileNotFound(hash.to_string()))
    }
    

    
    async fn pin(&self, hash: &IpfsHash) -> IpfsResult<()> {
        self.pins.write().await.insert(hash.as_str().to_string());
        Ok(())
    }
    
    async fn unpin(&self, hash: &IpfsHash) -> IpfsResult<()> {
        self.pins.write().await.remove(hash.as_str());
        Ok(())
    }
    
    async fn exists(&self, hash: &IpfsHash) -> IpfsResult<bool> {
        Ok(self.storage.read().await.contains_key(hash.as_str()))
    }
    
    async fn size(&self, hash: &IpfsHash) -> IpfsResult<u64> {
        self.storage.read().await
            .get(hash.as_str())
            .map(|data| data.len() as u64)
            .ok_or_else(|| IpfsError::FileNotFound(hash.to_string()))
    }
    
    async fn list_pins(&self) -> IpfsResult<Vec<IpfsHash>> {
        let pins = self.pins.read().await;
        let hashes: Result<Vec<_>, _> = pins.iter()
            .map(|hash| IpfsHash::new(hash.clone()))
            .collect();
        hashes
    }
}

#[cfg(test)]
#[async_trait(?Send)]
impl IpfsClientFileOps for MockIpfsClient {
    async fn add_file<P: AsRef<Path> + Send>(&self, path: P) -> IpfsResult<IpfsHash> {
        let data = tokio::fs::read(path).await
            .map_err(|e| IpfsError::ClientError(e.to_string()))?;
        self.add_bytes(data).await
    }

    async fn get_file<P: AsRef<Path> + Send>(&self, hash: &IpfsHash, path: P) -> IpfsResult<()> {
        let data = self.get_bytes(hash).await?;
        tokio::fs::write(path, data).await
            .map_err(|e| IpfsError::ClientError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_mock_client_add_and_get_bytes() {
        let client = MockIpfsClient::new();
        let data = b"Hello, IPFS!".to_vec();

        let hash = client.add_bytes(data.clone()).await.unwrap();
        assert!(IpfsHash::is_valid(hash.as_str()));

        let retrieved = client.get_bytes(&hash).await.unwrap();
        assert_eq!(data, retrieved);
    }

    #[tokio::test]
    async fn test_mock_client_file_operations() {
        let client = MockIpfsClient::new();

        // Create temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        let test_data = b"Test file content";
        std::io::Write::write_all(&mut temp_file, test_data).unwrap();
        std::io::Write::flush(&mut temp_file).unwrap();

        // Add file to IPFS
        let hash = client.add_file(temp_file.path()).await.unwrap();

        // Get file from IPFS
        let output_file = NamedTempFile::new().unwrap();
        client.get_file(&hash, output_file.path()).await.unwrap();

        // Verify content
        let retrieved_data = tokio::fs::read(output_file.path()).await.unwrap();
        assert_eq!(test_data, retrieved_data.as_slice());
    }

    #[tokio::test]
    async fn test_mock_client_pinning() {
        let client = MockIpfsClient::new();
        let data = b"Pin me!".to_vec();

        let hash = client.add_bytes(data).await.unwrap();

        // Pin the content
        client.pin(&hash).await.unwrap();

        // Check if pinned
        let pins = client.list_pins().await.unwrap();
        assert!(pins.contains(&hash));

        // Unpin the content
        client.unpin(&hash).await.unwrap();

        // Check if unpinned
        let pins = client.list_pins().await.unwrap();
        assert!(!pins.contains(&hash));
    }

    #[tokio::test]
    async fn test_mock_client_exists_and_size() {
        let client = MockIpfsClient::new();
        let data = b"Size test data".to_vec();
        let data_len = data.len() as u64;

        let hash = client.add_bytes(data).await.unwrap();

        // Check existence
        assert!(client.exists(&hash).await.unwrap());

        // Check size
        let size = client.size(&hash).await.unwrap();
        assert_eq!(size, data_len);

        // Check non-existent content
        let fake_hash = IpfsHash::new("QmFakeHashThatDoesNotExist123456789012345678".to_string()).unwrap();
        assert!(!client.exists(&fake_hash).await.unwrap());
        assert!(client.size(&fake_hash).await.is_err());
    }

    #[test]
    fn test_ipfs_client_config() {
        let config = IpfsConfig::default();
        let client = IpfsClient::new(config.clone());

        // Should fail with default config since no IPFS node is running
        assert!(client.is_ok()); // Client creation should succeed

        let client = client.unwrap();
        assert_eq!(client.config().api_url, config.api_url);
    }

    #[test]
    fn test_ipfs_client_validation() {
        let mut config = IpfsConfig::default();
        config.max_file_size = 100;
        config.allowed_mime_types = vec!["text/plain".to_string()];

        let client = IpfsClient::new(config).unwrap();

        // Test file size validation
        assert!(client.validate_file_size(50).is_ok());
        assert!(client.validate_file_size(150).is_err());

        // Test MIME type validation
        assert!(client.validate_mime_type("text/plain").is_ok());
        assert!(client.validate_mime_type("image/jpeg").is_err());
    }

    #[test]
    fn test_ipfs_client_with_default_config() {
        let client = IpfsClient::with_default_config();
        assert!(client.is_ok());
    }
}
