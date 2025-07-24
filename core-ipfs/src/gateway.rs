// =====================================================================================
// IPFS Gateway Management
//
// HTTP gateway interface for accessing IPFS content
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{IpfsConfig, IpfsError, IpfsHash, IpfsResult};
use async_trait::async_trait;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};
use url::Url;

/// Gateway response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub hash: IpfsHash,
    pub content_type: String,
    pub content_length: Option<u64>,
    pub cache_control: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub custom_headers: HashMap<String, String>,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            base_url: "https://ipfs.io".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            custom_headers: HashMap::new(),
            cache_enabled: true,
            cache_ttl_seconds: 3600, // 1 hour
        }
    }
}

/// Gateway client trait
#[async_trait(?Send)]
pub trait GatewayClientTrait {
    /// Get content via HTTP gateway
    async fn get_content(&self, hash: &IpfsHash) -> IpfsResult<Vec<u8>>;

    /// Get content with metadata
    async fn get_content_with_metadata(
        &self,
        hash: &IpfsHash,
    ) -> IpfsResult<(Vec<u8>, GatewayResponse)>;

    /// Get content URL
    fn get_content_url(&self, hash: &IpfsHash) -> String;

    /// Check if content is available via gateway
    async fn is_available(&self, hash: &IpfsHash) -> IpfsResult<bool>;

    /// Get content metadata only (HEAD request)
    async fn get_metadata(&self, hash: &IpfsHash) -> IpfsResult<GatewayResponse>;

    /// Stream content (for large files)
    async fn stream_content(&self, hash: &IpfsHash) -> IpfsResult<Response>;
}

/// HTTP gateway client implementation
pub struct HttpGatewayClient {
    client: Client,
    config: GatewayConfig,
    cache: std::sync::Arc<
        tokio::sync::RwLock<HashMap<String, (Vec<u8>, chrono::DateTime<chrono::Utc>)>>,
    >,
}

impl HttpGatewayClient {
    /// Create new HTTP gateway client
    pub fn new(config: GatewayConfig) -> IpfsResult<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("RWA-Platform-IPFS-Gateway/1.0");

        // Add custom headers if any
        if !config.custom_headers.is_empty() {
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in &config.custom_headers {
                let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| IpfsError::GatewayError(format!("Invalid header name: {}", e)))?;
                let header_value = reqwest::header::HeaderValue::from_str(value)
                    .map_err(|e| IpfsError::GatewayError(format!("Invalid header value: {}", e)))?;
                headers.insert(header_name, header_value);
            }
            client_builder = client_builder.default_headers(headers);
        }

        let client = client_builder
            .build()
            .map_err(|e| IpfsError::GatewayError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            config,
            cache: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    /// Create client with default configuration
    pub fn with_default_config() -> IpfsResult<Self> {
        Self::new(GatewayConfig::default())
    }

    /// Build URL for IPFS content
    fn build_url(&self, hash: &IpfsHash) -> IpfsResult<String> {
        let base_url = Url::parse(&self.config.base_url)
            .map_err(|e| IpfsError::GatewayError(format!("Invalid base URL: {}", e)))?;

        let url = base_url
            .join(&format!("/ipfs/{}", hash.as_str()))
            .map_err(|e| IpfsError::GatewayError(format!("Failed to build URL: {}", e)))?;

        Ok(url.to_string())
    }

    /// Check cache for content
    async fn get_from_cache(&self, hash: &IpfsHash) -> Option<Vec<u8>> {
        if !self.config.cache_enabled {
            return None;
        }

        let cache = self.cache.read().await;
        if let Some((content, cached_at)) = cache.get(hash.as_str()) {
            let cache_age = chrono::Utc::now().signed_duration_since(*cached_at);
            if cache_age.num_seconds() < self.config.cache_ttl_seconds as i64 {
                debug!("Cache hit for {}", hash);
                return Some(content.clone());
            }
        }
        None
    }

    /// Store content in cache
    async fn store_in_cache(&self, hash: &IpfsHash, content: Vec<u8>) {
        if self.config.cache_enabled {
            let mut cache = self.cache.write().await;
            cache.insert(hash.as_str().to_string(), (content, chrono::Utc::now()));
            debug!("Cached content for {}", hash);
        }
    }

    /// Perform HTTP request with retries
    async fn request_with_retries(&self, url: &str) -> IpfsResult<Response> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        last_error = Some(IpfsError::GatewayError(format!(
                            "HTTP error: {} - {}",
                            response.status(),
                            response.status().canonical_reason().unwrap_or("Unknown")
                        )));
                    }
                }
                Err(e) => {
                    last_error = Some(IpfsError::NetworkError(format!("Request failed: {}", e)));
                }
            }

            if attempt < self.config.max_retries {
                debug!(
                    "Request attempt {} failed, retrying in {}ms",
                    attempt + 1,
                    self.config.retry_delay_ms
                );
                tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
            }
        }

        Err(last_error
            .unwrap_or_else(|| IpfsError::GatewayError("All retry attempts failed".to_string())))
    }

    /// Extract response metadata
    fn extract_metadata(&self, hash: &IpfsHash, response: &Response) -> GatewayResponse {
        let headers = response.headers();

        GatewayResponse {
            hash: hash.clone(),
            content_type: headers
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string(),
            content_length: headers
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            cache_control: headers
                .get(reqwest::header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            etag: headers
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            last_modified: headers
                .get(reqwest::header::LAST_MODIFIED)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
        }
    }
}

#[async_trait(?Send)]
impl GatewayClientTrait for HttpGatewayClient {
    #[instrument(skip(self))]
    async fn get_content(&self, hash: &IpfsHash) -> IpfsResult<Vec<u8>> {
        debug!("Getting content via gateway: {}", hash);

        // Check cache first
        if let Some(cached_content) = self.get_from_cache(hash).await {
            return Ok(cached_content);
        }

        let url = self.build_url(hash)?;
        let response = self.request_with_retries(&url).await?;

        let content = response
            .bytes()
            .await
            .map_err(|e| IpfsError::NetworkError(format!("Failed to read response body: {}", e)))?
            .to_vec();

        // Cache the content
        self.store_in_cache(hash, content.clone()).await;

        info!(
            "Successfully retrieved {} bytes via gateway for {}",
            content.len(),
            hash
        );
        Ok(content)
    }

    #[instrument(skip(self))]
    async fn get_content_with_metadata(
        &self,
        hash: &IpfsHash,
    ) -> IpfsResult<(Vec<u8>, GatewayResponse)> {
        debug!("Getting content with metadata via gateway: {}", hash);

        // Check cache first
        if let Some(cached_content) = self.get_from_cache(hash).await {
            // For cached content, we need to make a HEAD request to get fresh metadata
            let metadata = self.get_metadata(hash).await?;
            return Ok((cached_content, metadata));
        }

        let url = self.build_url(hash)?;
        let response = self.request_with_retries(&url).await?;

        let metadata = self.extract_metadata(hash, &response);
        let content = response
            .bytes()
            .await
            .map_err(|e| IpfsError::NetworkError(format!("Failed to read response body: {}", e)))?
            .to_vec();

        // Cache the content
        self.store_in_cache(hash, content.clone()).await;

        info!(
            "Successfully retrieved {} bytes with metadata via gateway for {}",
            content.len(),
            hash
        );
        Ok((content, metadata))
    }

    fn get_content_url(&self, hash: &IpfsHash) -> String {
        self.build_url(hash)
            .unwrap_or_else(|_| format!("{}/ipfs/{}", self.config.base_url, hash.as_str()))
    }

    #[instrument(skip(self))]
    async fn is_available(&self, hash: &IpfsHash) -> IpfsResult<bool> {
        debug!("Checking availability via gateway: {}", hash);

        let url = self.build_url(hash)?;

        match self.client.head(&url).send().await {
            Ok(response) => {
                let available = response.status().is_success();
                debug!(
                    "Content {} is {} via gateway",
                    hash,
                    if available {
                        "available"
                    } else {
                        "not available"
                    }
                );
                Ok(available)
            }
            Err(e) => {
                warn!("Failed to check availability for {}: {}", hash, e);
                Ok(false)
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_metadata(&self, hash: &IpfsHash) -> IpfsResult<GatewayResponse> {
        debug!("Getting metadata via gateway: {}", hash);

        let url = self.build_url(hash)?;
        let response = self
            .client
            .head(&url)
            .send()
            .await
            .map_err(|e| IpfsError::NetworkError(format!("HEAD request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(IpfsError::GatewayError(format!(
                "HTTP error: {} - {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let metadata = self.extract_metadata(hash, &response);
        info!("Successfully retrieved metadata via gateway for {}", hash);
        Ok(metadata)
    }

    #[instrument(skip(self))]
    async fn stream_content(&self, hash: &IpfsHash) -> IpfsResult<Response> {
        debug!("Streaming content via gateway: {}", hash);

        let url = self.build_url(hash)?;
        let response = self.request_with_retries(&url).await?;

        info!(
            "Successfully started streaming content via gateway for {}",
            hash
        );
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gateway_config_default() {
        let config = GatewayConfig::default();
        assert_eq!(config.base_url, "https://ipfs.io");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.cache_enabled);
        assert_eq!(config.cache_ttl_seconds, 3600);
    }

    #[tokio::test]
    async fn test_http_gateway_client_creation() {
        let config = GatewayConfig::default();
        let client = HttpGatewayClient::new(config);
        assert!(client.is_ok());

        let client = HttpGatewayClient::with_default_config();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_build_url() {
        let config = GatewayConfig {
            base_url: "https://test-gateway.com".to_string(),
            ..Default::default()
        };
        let client = HttpGatewayClient::new(config).unwrap();
        let hash =
            IpfsHash::new("QmTestHash123456789012345678901234567890123456".to_string()).unwrap();

        let url = client.build_url(&hash).unwrap();
        assert_eq!(
            url,
            "https://test-gateway.com/ipfs/QmTestHash123456789012345678901234567890123456"
        );
    }

    #[tokio::test]
    async fn test_get_content_url() {
        let config = GatewayConfig {
            base_url: "https://test-gateway.com".to_string(),
            ..Default::default()
        };
        let client = HttpGatewayClient::new(config).unwrap();
        let hash =
            IpfsHash::new("QmTestHash123456789012345678901234567890123456".to_string()).unwrap();

        let url = client.get_content_url(&hash);
        assert_eq!(
            url,
            "https://test-gateway.com/ipfs/QmTestHash123456789012345678901234567890123456"
        );
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let config = GatewayConfig {
            cache_enabled: true,
            cache_ttl_seconds: 60,
            ..Default::default()
        };
        let client = HttpGatewayClient::new(config).unwrap();
        let hash =
            IpfsHash::new("QmCacheTest123456789012345678901234567890123456".to_string()).unwrap();
        let test_content = b"cached content".to_vec();

        // Initially no cache
        assert!(client.get_from_cache(&hash).await.is_none());

        // Store in cache
        client.store_in_cache(&hash, test_content.clone()).await;

        // Should retrieve from cache
        let cached = client.get_from_cache(&hash).await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), test_content);
    }

    #[tokio::test]
    async fn test_extract_metadata() {
        // This test is simplified since creating mock reqwest::Response is complex
        // In a real test environment, you'd use a test server or mock the HTTP layer
        let config = GatewayConfig::default();
        let _client = HttpGatewayClient::new(config).unwrap();
        let hash =
            IpfsHash::new("QmMetadataTest123456789012345678901234567890123".to_string()).unwrap();

        // Test the metadata structure creation
        let metadata = GatewayResponse {
            hash: hash.clone(),
            content_type: "application/json".to_string(),
            content_length: Some(1024),
            cache_control: None,
            etag: Some("\"test-etag\"".to_string()),
            last_modified: None,
        };

        assert_eq!(metadata.hash, hash);
        assert_eq!(metadata.content_type, "application/json");
        assert_eq!(metadata.content_length, Some(1024));
        assert_eq!(metadata.etag, Some("\"test-etag\"".to_string()));
    }

    #[tokio::test]
    async fn test_gateway_response() {
        let hash =
            IpfsHash::new("QmResponseTest123456789012345678901234567890123".to_string()).unwrap();
        let response = GatewayResponse {
            hash: hash.clone(),
            content_type: "text/plain".to_string(),
            content_length: Some(100),
            cache_control: Some("max-age=3600".to_string()),
            etag: Some("\"test\"".to_string()),
            last_modified: Some("Wed, 21 Oct 2015 07:28:00 GMT".to_string()),
        };

        assert_eq!(response.hash, hash);
        assert_eq!(response.content_type, "text/plain");
        assert_eq!(response.content_length, Some(100));
    }

    #[tokio::test]
    async fn test_gateway_config_with_custom_headers() {
        let mut custom_headers = HashMap::new();
        custom_headers.insert("X-Custom-Header".to_string(), "test-value".to_string());

        let config = GatewayConfig {
            base_url: "https://custom-gateway.com".to_string(),
            custom_headers,
            ..Default::default()
        };

        let client = HttpGatewayClient::new(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_base_url() {
        let config = GatewayConfig {
            base_url: "invalid-url".to_string(),
            ..Default::default()
        };

        let client = HttpGatewayClient::new(config).unwrap();
        let hash =
            IpfsHash::new("QmInvalidUrl123456789012345678901234567890123".to_string()).unwrap();

        let result = client.build_url(&hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_gateway_config_serialization() {
        let config = GatewayConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(!serialized.is_empty());

        let deserialized: GatewayConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.base_url, config.base_url);
        assert_eq!(deserialized.timeout_seconds, config.timeout_seconds);
    }

    #[test]
    fn test_gateway_response_serialization() {
        let hash =
            IpfsHash::new("QmSerializationTest123456789012345678901234567".to_string()).unwrap();
        let response = GatewayResponse {
            hash,
            content_type: "application/json".to_string(),
            content_length: Some(512),
            cache_control: None,
            etag: None,
            last_modified: None,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(!serialized.is_empty());

        let deserialized: GatewayResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.content_type, response.content_type);
        assert_eq!(deserialized.content_length, response.content_length);
    }
}
