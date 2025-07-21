// =====================================================================================
// Decentralized Identity (DID) System for RWA Platform
// 
// This module provides W3C DID specification compliant decentralized identity
// management including DID documents, verifiable credentials, and identity verification.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod did;
pub mod document;
pub mod resolver;
pub mod verifier;
pub mod credential;
pub mod key_manager;
pub mod registry;
pub mod error;
pub mod utils;
pub mod service;

// Re-export main types and traits
pub use did::*;
pub use document::*;
pub use resolver::*;
pub use verifier::*;
pub use credential::*;
pub use key_manager::*;
pub use registry::*;
pub use error::*;
pub use utils::*;
pub use service::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// DID method identifier for RWA platform
pub const DID_METHOD: &str = "rwa";

/// DID method specification version
pub const DID_SPEC_VERSION: &str = "1.0";

/// Default key type for DID operations
pub const DEFAULT_KEY_TYPE: &str = "Ed25519VerificationKey2020";

/// DID configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidConfig {
    /// DID method name
    pub method: String,
    /// Registry endpoint URL
    pub registry_url: Option<String>,
    /// Default key type
    pub default_key_type: String,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    /// Maximum resolution attempts
    pub max_resolution_attempts: u32,
    /// Resolution timeout in seconds
    pub resolution_timeout: u64,
}

impl Default for DidConfig {
    fn default() -> Self {
        Self {
            method: DID_METHOD.to_string(),
            registry_url: None,
            default_key_type: DEFAULT_KEY_TYPE.to_string(),
            enable_cache: true,
            cache_ttl: 3600, // 1 hour
            max_resolution_attempts: 3,
            resolution_timeout: 30,
        }
    }
}

/// DID operation result
pub type DidResult<T> = Result<T, DidError>;

/// DID service trait for high-level operations
#[async_trait::async_trait]
pub trait DidService: Send + Sync {
    /// Create a new DID
    async fn create_did(&self, controller: Option<String>) -> DidResult<DidDocument>;
    
    /// Resolve a DID to its document
    async fn resolve_did(&self, did: &str) -> DidResult<DidDocument>;
    
    /// Update a DID document
    async fn update_did(&self, did: &str, document: DidDocument) -> DidResult<()>;
    
    /// Deactivate a DID
    async fn deactivate_did(&self, did: &str) -> DidResult<()>;
    
    /// Issue a verifiable credential
    async fn issue_credential(&self, issuer_did: &str, subject_did: &str, claims: HashMap<String, serde_json::Value>) -> DidResult<VerifiableCredential>;
    
    /// Verify a verifiable credential
    async fn verify_credential(&self, credential: &VerifiableCredential) -> DidResult<bool>;
    
    /// Create a verifiable presentation
    async fn create_presentation(&self, holder_did: &str, credentials: Vec<VerifiableCredential>) -> DidResult<VerifiablePresentation>;
    
    /// Verify a verifiable presentation
    async fn verify_presentation(&self, presentation: &VerifiablePresentation) -> DidResult<bool>;
}

/// DID metadata for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidMetadata {
    /// Creation timestamp
    pub created: DateTime<Utc>,
    /// Last update timestamp
    pub updated: DateTime<Utc>,
    /// Version number
    pub version: u64,
    /// Deactivated flag
    pub deactivated: bool,
    /// Next update key
    pub next_update: Option<String>,
    /// Equivalent IDs
    pub equivalent_id: Vec<String>,
    /// Canonical ID
    pub canonical_id: Option<String>,
}

impl Default for DidMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            created: now,
            updated: now,
            version: 1,
            deactivated: false,
            next_update: None,
            equivalent_id: Vec::new(),
            canonical_id: None,
        }
    }
}

/// DID resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidResolutionResult {
    /// DID document
    pub did_document: Option<DidDocument>,
    /// Resolution metadata
    pub did_resolution_metadata: DidResolutionMetadata,
    /// Document metadata
    pub did_document_metadata: DidMetadata,
}

/// DID resolution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidResolutionMetadata {
    /// Content type
    pub content_type: Option<String>,
    /// Error code
    pub error: Option<String>,
    /// Error message
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_config_default() {
        let config = DidConfig::default();
        assert_eq!(config.method, DID_METHOD);
        assert_eq!(config.default_key_type, DEFAULT_KEY_TYPE);
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl, 3600);
    }

    #[test]
    fn test_did_metadata_default() {
        let metadata = DidMetadata::default();
        assert_eq!(metadata.version, 1);
        assert!(!metadata.deactivated);
        assert!(metadata.equivalent_id.is_empty());
    }
}
