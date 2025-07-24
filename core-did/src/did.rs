// =====================================================================================
// DID Identifier Implementation
//
// W3C DID specification compliant identifier parsing and validation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{DidError, DidResult, DID_METHOD};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use uuid::Uuid;

/// DID identifier following W3C DID specification
/// Format: did:method:method-specific-id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did {
    /// DID method (e.g., "rwa")
    pub method: String,
    /// Method-specific identifier
    pub method_specific_id: String,
    /// Optional path component
    pub path: Option<String>,
    /// Optional query component
    pub query: Option<String>,
    /// Optional fragment component
    pub fragment: Option<String>,
}

impl Did {
    /// Create a new DID with the RWA method
    pub fn new(method_specific_id: String) -> Self {
        Self {
            method: DID_METHOD.to_string(),
            method_specific_id,
            path: None,
            query: None,
            fragment: None,
        }
    }

    /// Create a new DID with custom method
    pub fn new_with_method(method: String, method_specific_id: String) -> Self {
        Self {
            method,
            method_specific_id,
            path: None,
            query: None,
            fragment: None,
        }
    }

    /// Generate a new random DID using UUID
    pub fn generate() -> Self {
        let uuid = Uuid::new_v4();
        Self::new(uuid.to_string())
    }

    /// Generate a new DID from a seed
    pub fn generate_from_seed(seed: &[u8]) -> DidResult<Self> {
        if seed.len() < 16 {
            return Err(DidError::InvalidKeyFormat("Seed too short".to_string()));
        }

        // Use first 16 bytes to create a UUID
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&seed[..16]);
        let uuid = Uuid::from_bytes(uuid_bytes);

        Ok(Self::new(uuid.to_string()))
    }

    /// Add path component
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Add query component
    pub fn with_query(mut self, query: String) -> Self {
        self.query = Some(query);
        self
    }

    /// Add fragment component
    pub fn with_fragment(mut self, fragment: String) -> Self {
        self.fragment = Some(fragment);
        self
    }

    /// Get the base DID without path, query, or fragment
    pub fn base(&self) -> String {
        format!("did:{}:{}", self.method, self.method_specific_id)
    }

    /// Get the full DID string
    pub fn to_string(&self) -> String {
        let mut did = self.base();

        if let Some(path) = &self.path {
            did.push('/');
            did.push_str(path);
        }

        if let Some(query) = &self.query {
            did.push('?');
            did.push_str(query);
        }

        if let Some(fragment) = &self.fragment {
            did.push('#');
            did.push_str(fragment);
        }

        did
    }

    /// Validate DID format
    pub fn validate(&self) -> DidResult<()> {
        // Validate method
        if self.method.is_empty() {
            return Err(DidError::InvalidDidMethod(
                "Method cannot be empty".to_string(),
            ));
        }

        // Method must be lowercase alphanumeric
        if !self
            .method
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err(DidError::InvalidDidMethod(
                "Method must be lowercase alphanumeric".to_string(),
            ));
        }

        // Validate method-specific ID
        if self.method_specific_id.is_empty() {
            return Err(DidError::InvalidDidFormat(
                "Method-specific ID cannot be empty".to_string(),
            ));
        }

        // Method-specific ID must not contain invalid characters
        if self.method_specific_id.contains(':') {
            return Err(DidError::InvalidDidFormat(
                "Method-specific ID cannot contain ':'".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if this DID is for the RWA method
    pub fn is_rwa_method(&self) -> bool {
        self.method == DID_METHOD
    }

    /// Extract key reference from fragment
    pub fn key_reference(&self) -> Option<&str> {
        self.fragment.as_deref()
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for Did {
    type Err = DidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Basic DID format validation
        if !s.starts_with("did:") {
            return Err(DidError::InvalidDidFormat(
                "DID must start with 'did:'".to_string(),
            ));
        }

        // Handle fragment first
        let mut fragment = None;
        let base_part = if let Some(fragment_pos) = s.find('#') {
            fragment = Some(s[fragment_pos + 1..].to_string());
            &s[..fragment_pos]
        } else {
            s
        };

        // Handle query
        let mut query = None;
        let did_part = if let Some(query_pos) = base_part.find('?') {
            query = Some(base_part[query_pos + 1..].to_string());
            &base_part[..query_pos]
        } else {
            base_part
        };

        // Handle path
        let mut path = None;
        let core_did_part = if let Some(path_pos) = did_part.find('/') {
            path = Some(did_part[path_pos + 1..].to_string());
            &did_part[..path_pos]
        } else {
            did_part
        };

        // Split by colon to get method and method-specific-id
        let parts: Vec<&str> = core_did_part.split(':').collect();
        if parts.len() < 3 {
            return Err(DidError::InvalidDidFormat(
                "DID must have at least method and method-specific-id".to_string(),
            ));
        }

        let method = parts[1].to_string();
        let method_specific_id = parts[2..].join(":");

        let did = Did {
            method,
            method_specific_id,
            path,
            query,
            fragment,
        };

        did.validate()?;
        Ok(did)
    }
}

/// DID URL for referencing specific resources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DidUrl {
    /// Base DID
    pub did: Did,
    /// Service name or verification method ID
    pub resource: Option<String>,
}

impl DidUrl {
    /// Create a new DID URL
    pub fn new(did: Did) -> Self {
        Self {
            did,
            resource: None,
        }
    }

    /// Create DID URL with resource reference
    pub fn with_resource(did: Did, resource: String) -> Self {
        Self {
            did,
            resource: Some(resource),
        }
    }

    /// Get the full DID URL string
    pub fn to_string(&self) -> String {
        let mut url = self.did.to_string();
        if let Some(resource) = &self.resource {
            url.push('#');
            url.push_str(resource);
        }
        url
    }
}

impl fmt::Display for DidUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FromStr for DidUrl {
    type Err = DidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((did_part, resource)) = s.split_once('#') {
            let did = Did::from_str(did_part)?;
            Ok(DidUrl::with_resource(did, resource.to_string()))
        } else {
            let did = Did::from_str(s)?;
            Ok(DidUrl::new(did))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_new() {
        let did = Did::new("123456789".to_string());
        assert_eq!(did.method, DID_METHOD);
        assert_eq!(did.method_specific_id, "123456789");
        assert_eq!(did.base(), "did:rwa:123456789");
        assert!(did.path.is_none());
        assert!(did.query.is_none());
        assert!(did.fragment.is_none());
    }

    #[test]
    fn test_did_new_with_method() {
        let did = Did::new_with_method("example".to_string(), "123456789".to_string());
        assert_eq!(did.method, "example");
        assert_eq!(did.method_specific_id, "123456789");
        assert_eq!(did.base(), "did:example:123456789");
    }

    #[test]
    fn test_did_generate() {
        let did = Did::generate();
        assert_eq!(did.method, DID_METHOD);
        assert!(!did.method_specific_id.is_empty());
        assert!(uuid::Uuid::parse_str(&did.method_specific_id).is_ok());
    }

    #[test]
    fn test_did_generate_from_seed() {
        let seed = b"this is a test seed for did generation";
        let did1 = Did::generate_from_seed(seed).unwrap();
        let did2 = Did::generate_from_seed(seed).unwrap();

        // Same seed should produce same DID
        assert_eq!(did1.method_specific_id, did2.method_specific_id);
        assert_eq!(did1.method, DID_METHOD);
    }

    #[test]
    fn test_did_generate_from_seed_short() {
        let short_seed = b"short";
        let result = Did::generate_from_seed(short_seed);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidKeyFormat(_)));
    }

    #[test]
    fn test_did_with_path() {
        let did = Did::new("123456789".to_string()).with_path("service".to_string());

        assert_eq!(did.path, Some("service".to_string()));
        assert_eq!(did.to_string(), "did:rwa:123456789/service");
    }

    #[test]
    fn test_did_with_query() {
        let did = Did::new("123456789".to_string()).with_query("version=1".to_string());

        assert_eq!(did.query, Some("version=1".to_string()));
        assert_eq!(did.to_string(), "did:rwa:123456789?version=1");
    }

    #[test]
    fn test_did_with_fragment() {
        let did = Did::new("123456789".to_string()).with_fragment("key-1".to_string());

        assert_eq!(did.fragment, Some("key-1".to_string()));
        assert_eq!(did.to_string(), "did:rwa:123456789#key-1");
    }

    #[test]
    fn test_did_with_all_components() {
        let did = Did::new("123456789".to_string())
            .with_path("service".to_string())
            .with_query("version=1".to_string())
            .with_fragment("key-1".to_string());

        assert_eq!(did.to_string(), "did:rwa:123456789/service?version=1#key-1");
    }

    #[test]
    fn test_did_base() {
        let did = Did::new("123456789".to_string())
            .with_path("service".to_string())
            .with_query("version=1".to_string())
            .with_fragment("key-1".to_string());

        assert_eq!(did.base(), "did:rwa:123456789");
    }

    #[test]
    fn test_did_validate_valid() {
        let did = Did::new("123456789".to_string());
        assert!(did.validate().is_ok());
    }

    #[test]
    fn test_did_validate_empty_method() {
        let mut did = Did::new("123456789".to_string());
        did.method = String::new();

        let result = did.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidDidMethod(_)));
    }

    #[test]
    fn test_did_validate_invalid_method() {
        let mut did = Did::new("123456789".to_string());
        did.method = "INVALID-METHOD".to_string();

        let result = did.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidDidMethod(_)));
    }

    #[test]
    fn test_did_validate_empty_method_specific_id() {
        let mut did = Did::new("123456789".to_string());
        did.method_specific_id = String::new();

        let result = did.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidDidFormat(_)));
    }

    #[test]
    fn test_did_validate_invalid_method_specific_id() {
        let mut did = Did::new("123456789".to_string());
        did.method_specific_id = "invalid:id".to_string();

        let result = did.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidDidFormat(_)));
    }

    #[test]
    fn test_did_is_rwa_method() {
        let rwa_did = Did::new("123456789".to_string());
        assert!(rwa_did.is_rwa_method());

        let other_did = Did::new_with_method("example".to_string(), "123456789".to_string());
        assert!(!other_did.is_rwa_method());
    }

    #[test]
    fn test_did_key_reference() {
        let did_without_fragment = Did::new("123456789".to_string());
        assert!(did_without_fragment.key_reference().is_none());

        let did_with_fragment =
            Did::new("123456789".to_string()).with_fragment("key-1".to_string());
        assert_eq!(did_with_fragment.key_reference(), Some("key-1"));
    }

    #[test]
    fn test_did_display() {
        let did = Did::new("123456789".to_string()).with_fragment("key-1".to_string());

        let display_str = format!("{}", did);
        assert_eq!(display_str, "did:rwa:123456789#key-1");
    }

    #[test]
    fn test_did_from_str_valid() {
        let did_str = "did:rwa:123456789";
        let did = Did::from_str(did_str).unwrap();

        assert_eq!(did.method, "rwa");
        assert_eq!(did.method_specific_id, "123456789");
        assert!(did.path.is_none());
        assert!(did.query.is_none());
        assert!(did.fragment.is_none());
    }

    #[test]
    fn test_did_from_str_with_fragment() {
        let did_str = "did:rwa:123456789#key-1";
        let did = Did::from_str(did_str).unwrap();

        assert_eq!(did.method, "rwa");
        assert_eq!(did.method_specific_id, "123456789");
        assert_eq!(did.fragment, Some("key-1".to_string()));
    }

    #[test]
    fn test_did_from_str_with_query() {
        let did_str = "did:rwa:123456789?version=1";
        let did = Did::from_str(did_str).unwrap();

        assert_eq!(did.method, "rwa");
        assert_eq!(did.method_specific_id, "123456789");
        assert_eq!(did.query, Some("version=1".to_string()));
    }

    #[test]
    fn test_did_from_str_invalid_format() {
        let invalid_dids = vec!["invalid", "did:", "did:method:", "not-a-did", "did", ""];

        for invalid_did in invalid_dids {
            let result = Did::from_str(invalid_did);
            assert!(result.is_err(), "Should fail for: {}", invalid_did);
        }
    }

    #[test]
    fn test_did_url_new() {
        let did = Did::new("123456789".to_string());
        let did_url = DidUrl::new(did.clone());

        assert_eq!(did_url.did, did);
        assert!(did_url.resource.is_none());
    }

    #[test]
    fn test_did_url_with_resource() {
        let did = Did::new("123456789".to_string());
        let did_url = DidUrl::with_resource(did.clone(), "key-1".to_string());

        assert_eq!(did_url.did, did);
        assert_eq!(did_url.resource, Some("key-1".to_string()));
    }

    #[test]
    fn test_did_url_to_string() {
        let did = Did::new("123456789".to_string());

        let did_url_without_resource = DidUrl::new(did.clone());
        assert_eq!(did_url_without_resource.to_string(), "did:rwa:123456789");

        let did_url_with_resource = DidUrl::with_resource(did, "key-1".to_string());
        assert_eq!(did_url_with_resource.to_string(), "did:rwa:123456789#key-1");
    }

    #[test]
    fn test_did_url_display() {
        let did = Did::new("123456789".to_string());
        let did_url = DidUrl::with_resource(did, "key-1".to_string());

        let display_str = format!("{}", did_url);
        assert_eq!(display_str, "did:rwa:123456789#key-1");
    }

    #[test]
    fn test_did_url_from_str_without_resource() {
        let did_url_str = "did:rwa:123456789";
        let did_url = DidUrl::from_str(did_url_str).unwrap();

        assert_eq!(did_url.did.method, "rwa");
        assert_eq!(did_url.did.method_specific_id, "123456789");
        assert!(did_url.resource.is_none());
    }

    #[test]
    fn test_did_url_from_str_with_resource() {
        let did_url_str = "did:rwa:123456789#key-1";
        let did_url = DidUrl::from_str(did_url_str).unwrap();

        assert_eq!(did_url.did.method, "rwa");
        assert_eq!(did_url.did.method_specific_id, "123456789");
        assert_eq!(did_url.resource, Some("key-1".to_string()));
    }

    #[test]
    fn test_did_url_from_str_invalid() {
        let invalid_urls = vec!["invalid", "did:", "not-a-did#resource"];

        for invalid_url in invalid_urls {
            let result = DidUrl::from_str(invalid_url);
            assert!(result.is_err(), "Should fail for: {}", invalid_url);
        }
    }

    #[test]
    fn test_did_roundtrip() {
        let original_did = Did::new("123456789".to_string())
            .with_path("service".to_string())
            .with_query("version=1".to_string())
            .with_fragment("key-1".to_string());

        let did_str = original_did.to_string();
        let parsed_did = Did::from_str(&did_str).unwrap();

        assert_eq!(original_did.method, parsed_did.method);
        assert_eq!(
            original_did.method_specific_id,
            parsed_did.method_specific_id
        );
        assert_eq!(original_did.path, parsed_did.path);
        assert_eq!(original_did.query, parsed_did.query);
        assert_eq!(original_did.fragment, parsed_did.fragment);
        assert_eq!(original_did, parsed_did);
    }

    #[test]
    fn test_did_url_roundtrip() {
        let did = Did::new("123456789".to_string());
        let original_url = DidUrl::with_resource(did, "key-1".to_string());

        let url_str = original_url.to_string();
        let parsed_url = DidUrl::from_str(&url_str).unwrap();

        assert_eq!(original_url.did.method, parsed_url.did.method);
        assert_eq!(
            original_url.did.method_specific_id,
            parsed_url.did.method_specific_id
        );
        assert_eq!(original_url.resource, parsed_url.resource);
    }
}
