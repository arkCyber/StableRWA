// =====================================================================================
// DID Utilities
//
// Utility functions for DID operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{DidError, DidResult};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Generate a deterministic DID from input data
pub fn generate_did_from_data(method: &str, data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let encoded = BASE64.encode(hash);

    // Use first 22 characters for a shorter identifier
    let short_id = &encoded[..22];
    format!("did:{}:{}", method, short_id)
}

/// Validate DID syntax according to W3C specification
pub fn validate_did_syntax(did: &str) -> DidResult<()> {
    // Basic format check
    if !did.starts_with("did:") {
        return Err(DidError::InvalidDidFormat(
            "DID must start with 'did:'".to_string(),
        ));
    }

    let parts: Vec<&str> = did.split(':').collect();
    if parts.len() < 3 {
        return Err(DidError::InvalidDidFormat(
            "DID must have at least method and method-specific-id".to_string(),
        ));
    }

    // Validate method
    let method = parts[1];
    if method.is_empty() {
        return Err(DidError::InvalidDidMethod(
            "Method cannot be empty".to_string(),
        ));
    }

    if !method
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        return Err(DidError::InvalidDidMethod(
            "Method must be lowercase alphanumeric".to_string(),
        ));
    }

    // Validate method-specific-id
    let method_specific_id = parts[2];
    if method_specific_id.is_empty() {
        return Err(DidError::InvalidDidFormat(
            "Method-specific-id cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Extract method from DID
pub fn extract_method(did: &str) -> DidResult<String> {
    validate_did_syntax(did)?;

    let parts: Vec<&str> = did.split(':').collect();
    Ok(parts[1].to_string())
}

/// Extract method-specific-id from DID
pub fn extract_method_specific_id(did: &str) -> DidResult<String> {
    validate_did_syntax(did)?;

    let parts: Vec<&str> = did.split(':').collect();
    Ok(parts[2].to_string())
}

/// Normalize DID by removing fragments and queries
pub fn normalize_did(did: &str) -> DidResult<String> {
    validate_did_syntax(did)?;

    // Remove fragment (#) and query (?) parts
    let base_did = did.split('#').next().unwrap_or(did);
    let base_did = base_did.split('?').next().unwrap_or(base_did);

    Ok(base_did.to_string())
}

/// Check if DID is a valid DID URL (includes fragment or query)
pub fn is_did_url(did: &str) -> bool {
    did.contains('#') || did.contains('?')
}

/// Parse DID URL components
pub fn parse_did_url(did_url: &str) -> DidResult<DidUrlComponents> {
    validate_did_syntax(did_url)?;

    let mut components = DidUrlComponents {
        did: String::new(),
        path: None,
        query: None,
        fragment: None,
    };

    // Split by fragment first
    let (base_part, fragment) = if let Some(pos) = did_url.find('#') {
        let (base, frag) = did_url.split_at(pos);
        (base, Some(frag[1..].to_string()))
    } else {
        (did_url, None)
    };

    // Split by query
    let (did_part, query) = if let Some(pos) = base_part.find('?') {
        let (did, q) = base_part.split_at(pos);
        (did, Some(q[1..].to_string()))
    } else {
        (base_part, None)
    };

    // Check for path (after method-specific-id)
    if let Some(path_pos) = did_part.find('/') {
        // Has path component
        let did_base = &did_part[..path_pos];
        let path = &did_part[path_pos + 1..];
        components.did = did_base.to_string();
        components.path = Some(path.to_string());
    } else {
        components.did = did_part.to_string();
    }

    components.query = query;
    components.fragment = fragment;

    Ok(components)
}

/// DID URL components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidUrlComponents {
    /// Base DID
    pub did: String,
    /// Path component
    pub path: Option<String>,
    /// Query component
    pub query: Option<String>,
    /// Fragment component
    pub fragment: Option<String>,
}

/// Generate a canonical representation of a DID document for signing
pub fn canonicalize_document(document: &crate::DidDocument) -> DidResult<String> {
    // This is a simplified canonicalization
    // In production, use proper JSON-LD canonicalization (RDF Dataset Canonicalization)
    let json =
        serde_json::to_string(document).map_err(|e| DidError::SerializationError(e.to_string()))?;

    Ok(json)
}

/// Create a proof value for a DID document
pub fn create_proof_value(data: &[u8], signature: &[u8]) -> String {
    let mut combined = Vec::new();
    combined.extend_from_slice(data);
    combined.extend_from_slice(signature);
    BASE64.encode(combined)
}

/// Verify a proof value
pub fn verify_proof_value(proof_value: &str, data: &[u8]) -> DidResult<Vec<u8>> {
    let decoded = BASE64
        .decode(proof_value)
        .map_err(|e| DidError::InvalidSignature(e.to_string()))?;

    if decoded.len() < data.len() {
        return Err(DidError::InvalidSignature(
            "Proof value too short".to_string(),
        ));
    }

    let signature = &decoded[data.len()..];
    Ok(signature.to_vec())
}

/// Generate a unique identifier
pub fn generate_unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Convert timestamp to ISO 8601 string
pub fn timestamp_to_iso8601(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    timestamp.to_rfc3339()
}

/// Parse ISO 8601 timestamp
pub fn parse_iso8601(timestamp: &str) -> DidResult<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| DidError::SerializationError(e.to_string()))
}

/// Merge two JSON objects
pub fn merge_json_objects(
    base: &mut HashMap<String, serde_json::Value>,
    overlay: HashMap<String, serde_json::Value>,
) {
    for (key, value) in overlay {
        base.insert(key, value);
    }
}

/// Validate URL format
pub fn validate_url(url: &str) -> DidResult<()> {
    url::Url::parse(url).map_err(|e| DidError::InvalidServiceEndpoint(e.to_string()))?;
    Ok(())
}

/// Generate a multibase encoded string
pub fn encode_multibase(data: &[u8]) -> String {
    format!("z{}", BASE64.encode(data))
}

/// Decode a multibase encoded string
pub fn decode_multibase(encoded: &str) -> DidResult<Vec<u8>> {
    if !encoded.starts_with('z') {
        return Err(DidError::InvalidKeyFormat(
            "Invalid multibase format".to_string(),
        ));
    }

    BASE64
        .decode(&encoded[1..])
        .map_err(|e| DidError::InvalidKeyFormat(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_did_from_data() {
        let data = b"test data";
        let did = generate_did_from_data("rwa", data);

        assert!(did.starts_with("did:rwa:"));
        assert_eq!(did.len(), 30); // "did:rwa:" + 22 characters
    }

    #[test]
    fn test_validate_did_syntax() {
        assert!(validate_did_syntax("did:rwa:123456789").is_ok());
        assert!(validate_did_syntax("did:example:123").is_ok());

        assert!(validate_did_syntax("invalid").is_err());
        assert!(validate_did_syntax("did:").is_err());
        assert!(validate_did_syntax("did:method:").is_err());
        assert!(validate_did_syntax("did:INVALID:123").is_err());
    }

    #[test]
    fn test_extract_components() {
        let did = "did:rwa:123456789";

        assert_eq!(extract_method(did).unwrap(), "rwa");
        assert_eq!(extract_method_specific_id(did).unwrap(), "123456789");
        assert_eq!(normalize_did(did).unwrap(), did);
    }

    #[test]
    fn test_parse_did_url() {
        let did_url = "did:rwa:123456789/path?query=value#fragment";
        let components = parse_did_url(did_url).unwrap();

        assert_eq!(components.did, "did:rwa:123456789");
        assert_eq!(components.path, Some("path".to_string()));
        assert_eq!(components.query, Some("query=value".to_string()));
        assert_eq!(components.fragment, Some("fragment".to_string()));
    }

    #[test]
    fn test_is_did_url() {
        assert!(!is_did_url("did:rwa:123456789"));
        assert!(is_did_url("did:rwa:123456789#key-1"));
        assert!(is_did_url("did:rwa:123456789?version=1"));
    }

    #[test]
    fn test_multibase_encoding() {
        let data = b"test data";
        let encoded = encode_multibase(data);
        let decoded = decode_multibase(&encoded).unwrap();

        assert_eq!(data, decoded.as_slice());
        assert!(encoded.starts_with('z'));
    }

    #[test]
    fn test_timestamp_conversion() {
        let now = chrono::Utc::now();
        let iso_string = timestamp_to_iso8601(now);
        let parsed = parse_iso8601(&iso_string).unwrap();

        // Allow for small differences due to precision
        let diff = (now - parsed).num_milliseconds().abs();
        assert!(diff < 1000);
    }
}
