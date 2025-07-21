// =====================================================================================
// DID Resolver Implementation
// 
// Resolves DIDs to their corresponding DID documents
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{DidDocument, DidError, DidResult, DidResolutionResult, DidResolutionMetadata, DidMetadata};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use reqwest::Client;

/// DID resolver trait
#[async_trait]
pub trait DidResolver: Send + Sync {
    /// Resolve a DID to its document
    async fn resolve(&self, did: &str) -> DidResult<DidResolutionResult>;
    
    /// Check if this resolver supports the given DID method
    fn supports_method(&self, method: &str) -> bool;
}

/// Universal DID resolver that delegates to method-specific resolvers
pub struct UniversalResolver {
    /// Method-specific resolvers
    resolvers: HashMap<String, Arc<dyn DidResolver>>,
    /// HTTP client for remote resolution
    client: Client,
    /// Cache for resolved documents
    cache: Arc<RwLock<HashMap<String, (DidResolutionResult, chrono::DateTime<chrono::Utc>)>>>,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

impl UniversalResolver {
    /// Create a new universal resolver
    pub fn new() -> Self {
        Self {
            resolvers: HashMap::new(),
            client: Client::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: 3600, // 1 hour
        }
    }

    /// Register a method-specific resolver
    pub fn register_resolver(&mut self, method: String, resolver: Arc<dyn DidResolver>) {
        self.resolvers.insert(method, resolver);
    }

    /// Set cache TTL
    pub fn set_cache_ttl(&mut self, ttl: u64) {
        self.cache_ttl = ttl;
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cached result if valid
    async fn get_cached(&self, did: &str) -> Option<DidResolutionResult> {
        let cache = self.cache.read().await;
        if let Some((result, timestamp)) = cache.get(did) {
            let now = chrono::Utc::now();
            let age = (now - *timestamp).num_seconds() as u64;
            if age < self.cache_ttl {
                return Some(result.clone());
            }
        }
        None
    }

    /// Cache resolution result
    async fn cache_result(&self, did: &str, result: DidResolutionResult) {
        let mut cache = self.cache.write().await;
        cache.insert(did.to_string(), (result, chrono::Utc::now()));
    }

    /// Parse DID to extract method
    fn parse_method(did: &str) -> DidResult<String> {
        if !did.starts_with("did:") {
            return Err(DidError::InvalidDidFormat("DID must start with 'did:'".to_string()));
        }

        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(DidError::InvalidDidFormat("Invalid DID format".to_string()));
        }

        Ok(parts[1].to_string())
    }
}

#[async_trait]
impl DidResolver for UniversalResolver {
    async fn resolve(&self, did: &str) -> DidResult<DidResolutionResult> {
        // Check cache first
        if let Some(cached) = self.get_cached(did).await {
            return Ok(cached);
        }

        // Parse method
        let method = Self::parse_method(did)?;

        // Find appropriate resolver
        if let Some(resolver) = self.resolvers.get(&method) {
            let result = resolver.resolve(did).await?;
            self.cache_result(did, result.clone()).await;
            Ok(result)
        } else {
            // Try HTTP resolution as fallback
            self.resolve_http(did).await
        }
    }

    fn supports_method(&self, method: &str) -> bool {
        self.resolvers.contains_key(method)
    }
}

impl UniversalResolver {
    /// Resolve DID via HTTP
    async fn resolve_http(&self, did: &str) -> DidResult<DidResolutionResult> {
        let url = format!("https://dev.uniresolver.io/1.0/identifiers/{}", did);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/did+ld+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DidError::ResolutionError(
                format!("HTTP resolution failed: {}", response.status())
            ));
        }

        let result: DidResolutionResult = response.json().await?;
        self.cache_result(did, result.clone()).await;
        Ok(result)
    }
}

/// In-memory DID resolver for testing and local development
#[derive(Debug)]
pub struct MemoryResolver {
    /// Stored DID documents
    documents: Arc<RwLock<HashMap<String, DidDocument>>>,
    /// Supported method
    method: String,
}

impl MemoryResolver {
    /// Create a new memory resolver
    pub fn new(method: String) -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            method,
        }
    }

    /// Store a DID document
    pub async fn store(&self, did: String, document: DidDocument) {
        let mut documents = self.documents.write().await;
        documents.insert(did, document);
    }

    /// Remove a DID document
    pub async fn remove(&self, did: &str) -> bool {
        let mut documents = self.documents.write().await;
        documents.remove(did).is_some()
    }

    /// List all stored DIDs
    pub async fn list_dids(&self) -> Vec<String> {
        let documents = self.documents.read().await;
        documents.keys().cloned().collect()
    }
}

#[async_trait]
impl DidResolver for MemoryResolver {
    async fn resolve(&self, did: &str) -> DidResult<DidResolutionResult> {
        let documents = self.documents.read().await;
        
        if let Some(document) = documents.get(did) {
            Ok(DidResolutionResult {
                did_document: Some(document.clone()),
                did_resolution_metadata: DidResolutionMetadata {
                    content_type: Some("application/did+ld+json".to_string()),
                    error: None,
                    error_message: None,
                },
                did_document_metadata: DidMetadata::default(),
            })
        } else {
            Ok(DidResolutionResult {
                did_document: None,
                did_resolution_metadata: DidResolutionMetadata {
                    content_type: None,
                    error: Some("notFound".to_string()),
                    error_message: Some(format!("DID not found: {}", did)),
                },
                did_document_metadata: DidMetadata::default(),
            })
        }
    }

    fn supports_method(&self, method: &str) -> bool {
        self.method == method
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Did, DidDocument};
    use tokio::time::{sleep, Duration};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_universal_resolver_new() {
        let resolver = UniversalResolver::new();
        assert_eq!(resolver.resolvers.len(), 0);
        assert_eq!(resolver.cache_ttl, 3600);
    }

    #[tokio::test]
    async fn test_universal_resolver_register_resolver() {
        let mut universal = UniversalResolver::new();
        let memory_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));

        universal.register_resolver("rwa".to_string(), memory_resolver);

        assert!(universal.supports_method("rwa"));
        assert!(!universal.supports_method("example"));
    }

    #[tokio::test]
    async fn test_universal_resolver_set_cache_ttl() {
        let mut universal = UniversalResolver::new();
        universal.set_cache_ttl(1800);
        assert_eq!(universal.cache_ttl, 1800);
    }

    #[tokio::test]
    async fn test_universal_resolver_clear_cache() {
        let mut universal = UniversalResolver::new();
        let memory_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));

        universal.register_resolver("rwa".to_string(), memory_resolver.clone());

        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        memory_resolver.store(did.to_string(), document.clone()).await;

        // Resolve to populate cache
        universal.resolve(&did.to_string()).await.unwrap();

        // Clear cache
        universal.clear_cache().await;

        // Should still resolve (from resolver, not cache)
        let result = universal.resolve(&did.to_string()).await.unwrap();
        assert!(result.did_document.is_some());
    }

    #[tokio::test]
    async fn test_universal_resolver_parse_method() {
        assert_eq!(UniversalResolver::parse_method("did:rwa:123456789").unwrap(), "rwa");
        assert_eq!(UniversalResolver::parse_method("did:example:test").unwrap(), "example");

        assert!(UniversalResolver::parse_method("invalid").is_err());
        assert!(UniversalResolver::parse_method("did:").is_err());
        assert!(UniversalResolver::parse_method("did:method").is_err());
    }

    #[tokio::test]
    async fn test_universal_resolver_resolve_with_registered_resolver() {
        let mut universal = UniversalResolver::new();
        let memory_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));

        universal.register_resolver("rwa".to_string(), memory_resolver.clone());

        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        memory_resolver.store(did.to_string(), document.clone()).await;

        let result = universal.resolve(&did.to_string()).await.unwrap();

        assert!(result.did_document.is_some());
        assert_eq!(result.did_document.unwrap().id, did.to_string());
        assert!(result.did_resolution_metadata.error.is_none());
    }

    #[tokio::test]
    async fn test_universal_resolver_resolve_unregistered_method() {
        let universal = UniversalResolver::new();

        let did = Did::new("123456789".to_string());

        // This should attempt HTTP resolution and likely fail
        let result = universal.resolve(&did.to_string()).await;

        // We expect this to fail since we don't have a real HTTP resolver
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_universal_resolver_caching() {
        let mut universal = UniversalResolver::new();
        let memory_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));

        universal.register_resolver("rwa".to_string(), memory_resolver.clone());
        universal.set_cache_ttl(60); // 1 minute

        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        memory_resolver.store(did.to_string(), document.clone()).await;

        // First resolution
        let start = std::time::Instant::now();
        let result1 = universal.resolve(&did.to_string()).await.unwrap();
        let first_duration = start.elapsed();

        // Second resolution (should be cached and faster)
        let start = std::time::Instant::now();
        let result2 = universal.resolve(&did.to_string()).await.unwrap();
        let second_duration = start.elapsed();

        assert!(result1.did_document.is_some());
        assert!(result2.did_document.is_some());
        assert_eq!(result1.did_document.as_ref().unwrap().id, result2.did_document.as_ref().unwrap().id);

        // Second resolution should be faster (cached)
        assert!(second_duration <= first_duration);
    }

    #[tokio::test]
    async fn test_universal_resolver_cache_expiration() {
        let mut universal = UniversalResolver::new();
        let memory_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));

        universal.register_resolver("rwa".to_string(), memory_resolver.clone());
        universal.set_cache_ttl(1); // 1 second TTL

        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        memory_resolver.store(did.to_string(), document.clone()).await;

        // First resolution
        let result1 = universal.resolve(&did.to_string()).await.unwrap();

        // Wait for cache to expire
        sleep(Duration::from_secs(2)).await;

        // Second resolution (cache should be expired)
        let result2 = universal.resolve(&did.to_string()).await.unwrap();

        assert!(result1.did_document.is_some());
        assert!(result2.did_document.is_some());
    }

    #[tokio::test]
    async fn test_memory_resolver_new() {
        let resolver = MemoryResolver::new("rwa".to_string());
        assert_eq!(resolver.method, "rwa");
        assert!(resolver.supports_method("rwa"));
        assert!(!resolver.supports_method("example"));
    }

    #[tokio::test]
    async fn test_memory_resolver_store_and_resolve() {
        let resolver = MemoryResolver::new("rwa".to_string());

        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        resolver.store(did.to_string(), document.clone()).await;

        let result = resolver.resolve(&did.to_string()).await.unwrap();

        assert!(result.did_document.is_some());
        assert_eq!(result.did_document.unwrap().id, did.to_string());
        assert!(result.did_resolution_metadata.error.is_none());
        assert_eq!(result.did_resolution_metadata.content_type, Some("application/did+ld+json".to_string()));
    }

    #[tokio::test]
    async fn test_memory_resolver_resolve_not_found() {
        let resolver = MemoryResolver::new("rwa".to_string());

        let result = resolver.resolve("did:rwa:nonexistent").await.unwrap();

        assert!(result.did_document.is_none());
        assert_eq!(result.did_resolution_metadata.error, Some("notFound".to_string()));
        assert!(result.did_resolution_metadata.error_message.is_some());
    }

    #[tokio::test]
    async fn test_memory_resolver_remove() {
        let resolver = MemoryResolver::new("rwa".to_string());

        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        resolver.store(did.to_string(), document.clone()).await;

        // Verify it exists
        let result = resolver.resolve(&did.to_string()).await.unwrap();
        assert!(result.did_document.is_some());

        // Remove it
        let removed = resolver.remove(&did.to_string()).await;
        assert!(removed);

        // Verify it's gone
        let result = resolver.resolve(&did.to_string()).await.unwrap();
        assert!(result.did_document.is_none());

        // Try to remove again
        let not_removed = resolver.remove(&did.to_string()).await;
        assert!(!not_removed);
    }

    #[tokio::test]
    async fn test_memory_resolver_list_dids() {
        let resolver = MemoryResolver::new("rwa".to_string());

        // Initially empty
        let dids = resolver.list_dids().await;
        assert!(dids.is_empty());

        // Add some DIDs
        let test_dids = vec!["did:rwa:123", "did:rwa:456", "did:rwa:789"];
        for did_str in &test_dids {
            let did = Did::from_str(did_str).unwrap();
            let document = DidDocument::new(did);
            resolver.store(did_str.to_string(), document).await;
        }

        let dids = resolver.list_dids().await;
        assert_eq!(dids.len(), 3);
        for did_str in test_dids {
            assert!(dids.contains(&did_str.to_string()));
        }
    }

    #[tokio::test]
    async fn test_memory_resolver_supports_method() {
        let resolver = MemoryResolver::new("rwa".to_string());

        assert!(resolver.supports_method("rwa"));
        assert!(!resolver.supports_method("example"));
        assert!(!resolver.supports_method("web"));
        assert!(!resolver.supports_method(""));
    }

    #[tokio::test]
    async fn test_multiple_resolvers() {
        let mut universal = UniversalResolver::new();

        let rwa_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));
        let example_resolver = Arc::new(MemoryResolver::new("example".to_string()));

        universal.register_resolver("rwa".to_string(), rwa_resolver.clone());
        universal.register_resolver("example".to_string(), example_resolver.clone());

        // Store documents in different resolvers
        let rwa_did = Did::new("123456789".to_string());
        let rwa_document = DidDocument::new(rwa_did.clone());
        rwa_resolver.store(rwa_did.to_string(), rwa_document).await;

        let example_did = Did::new_with_method("example".to_string(), "987654321".to_string());
        let example_document = DidDocument::new(example_did.clone());
        example_resolver.store(example_did.to_string(), example_document).await;

        // Resolve both
        let rwa_result = universal.resolve(&rwa_did.to_string()).await.unwrap();
        let example_result = universal.resolve(&example_did.to_string()).await.unwrap();

        assert!(rwa_result.did_document.is_some());
        assert!(example_result.did_document.is_some());
        assert_eq!(rwa_result.did_document.unwrap().id, rwa_did.to_string());
        assert_eq!(example_result.did_document.unwrap().id, example_did.to_string());
    }

    #[tokio::test]
    async fn test_concurrent_resolution() {
        let mut universal = UniversalResolver::new();
        let memory_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));

        universal.register_resolver("rwa".to_string(), memory_resolver.clone());

        // Store multiple documents
        let mut dids = Vec::new();
        for i in 0..10 {
            let did = Did::new(format!("concurrent-test-{}", i));
            let document = DidDocument::new(did.clone());
            memory_resolver.store(did.to_string(), document).await;
            dids.push(did);
        }

        let universal = Arc::new(universal);

        // Resolve concurrently
        let mut handles = Vec::new();
        for did in dids {
            let resolver = universal.clone();
            let did_string = did.to_string();
            let handle = tokio::spawn(async move {
                resolver.resolve(&did_string).await
            });
            handles.push(handle);
        }

        // Wait for all resolutions
        for handle in handles {
            let result = handle.await.unwrap().unwrap();
            assert!(result.did_document.is_some());
        }
    }

    #[tokio::test]
    async fn test_resolver_error_handling() {
        let resolver = MemoryResolver::new("rwa".to_string());

        // Test with invalid DID format
        let result = resolver.resolve("invalid-did").await;
        // The resolver itself doesn't validate DID format, it just looks for the key
        // So this will return "not found" rather than a format error
        assert!(result.is_ok());
        let resolution_result = result.unwrap();
        assert!(resolution_result.did_document.is_none());
        assert_eq!(resolution_result.did_resolution_metadata.error, Some("notFound".to_string()));
    }

    #[tokio::test]
    async fn test_resolver_metadata() {
        let resolver = MemoryResolver::new("rwa".to_string());

        let did = Did::new("metadata-test".to_string());
        let document = DidDocument::new(did.clone());

        resolver.store(did.to_string(), document).await;

        let result = resolver.resolve(&did.to_string()).await.unwrap();

        assert!(result.did_document.is_some());
        assert_eq!(result.did_resolution_metadata.content_type, Some("application/did+ld+json".to_string()));
        assert!(result.did_resolution_metadata.error.is_none());
        assert!(result.did_resolution_metadata.error_message.is_none());

        // Check document metadata
        assert_eq!(result.did_document_metadata.version, 1);
        assert!(!result.did_document_metadata.deactivated);
        assert!(result.did_document_metadata.equivalent_id.is_empty());
        assert!(result.did_document_metadata.canonical_id.is_none());
    }
}
