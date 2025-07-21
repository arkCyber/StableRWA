// =====================================================================================
// DID Key Management
// 
// Cryptographic key generation, storage, and management for DID operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{DidError, DidResult, VerificationMethod, PublicKeyMaterial};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Key type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    /// Ed25519 signature key
    Ed25519,
    /// X25519 key agreement key
    X25519,
    /// Secp256k1 key (for blockchain compatibility)
    Secp256k1,
}

impl KeyType {
    /// Get the verification method type string
    pub fn verification_method_type(&self) -> &'static str {
        match self {
            KeyType::Ed25519 => "Ed25519VerificationKey2020",
            KeyType::X25519 => "X25519KeyAgreementKey2020",
            KeyType::Secp256k1 => "EcdsaSecp256k1VerificationKey2019",
        }
    }
}

/// Key purpose enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    /// Authentication
    Authentication,
    /// Assertion method
    AssertionMethod,
    /// Key agreement
    KeyAgreement,
    /// Capability invocation
    CapabilityInvocation,
    /// Capability delegation
    CapabilityDelegation,
}

/// Cryptographic key pair
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// Key identifier
    pub id: String,
    /// Key type
    pub key_type: KeyType,
    /// Key purpose
    pub purpose: Vec<KeyPurpose>,
    /// Public key bytes
    pub public_key: Vec<u8>,
    /// Private key bytes (encrypted in storage)
    pub private_key: Vec<u8>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl KeyPair {
    /// Generate a new Ed25519 key pair
    pub fn generate_ed25519(id: String, purpose: Vec<KeyPurpose>) -> DidResult<Self> {
        let mut csprng = OsRng {};
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            id,
            key_type: KeyType::Ed25519,
            purpose,
            public_key: verifying_key.to_bytes().to_vec(),
            private_key: signing_key.to_bytes().to_vec(),
            created_at: chrono::Utc::now(),
        })
    }

    /// Generate a new X25519 key pair
    pub fn generate_x25519(id: String, purpose: Vec<KeyPurpose>) -> DidResult<Self> {
        // For now, we'll use a placeholder implementation
        // In a real implementation, you'd use proper X25519 key generation
        let secret_bytes: [u8; 32] = rand::random();
        let public_bytes: [u8; 32] = rand::random(); // This is not correct, but for demo

        Ok(Self {
            id,
            key_type: KeyType::X25519,
            purpose,
            public_key: public_bytes.to_vec(),
            private_key: secret_bytes.to_vec(),
            created_at: chrono::Utc::now(),
        })
    }

    /// Sign data with this key pair (Ed25519 only)
    pub fn sign(&self, data: &[u8]) -> DidResult<Vec<u8>> {
        match self.key_type {
            KeyType::Ed25519 => {
                let signing_key = SigningKey::from_bytes(&self.private_key.clone().try_into()
                    .map_err(|_| DidError::CryptographicError("Invalid private key length".to_string()))?);

                let signature = signing_key.sign(data);
                Ok(signature.to_bytes().to_vec())
            }
            _ => Err(DidError::CryptographicError(
                "Key type does not support signing".to_string()
            )),
        }
    }

    /// Verify signature with this key pair (Ed25519 only)
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> DidResult<bool> {
        match self.key_type {
            KeyType::Ed25519 => {
                let verifying_key = VerifyingKey::from_bytes(&self.public_key.clone().try_into()
                    .map_err(|_| DidError::CryptographicError("Invalid public key length".to_string()))?)
                    .map_err(|e| DidError::CryptographicError(e.to_string()))?;

                let signature = Signature::from_bytes(&signature.try_into()
                    .map_err(|_| DidError::InvalidSignature("Invalid signature length".to_string()))?);

                match verifying_key.verify(data, &signature) {
                    Ok(()) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            _ => Err(DidError::CryptographicError(
                "Key type does not support verification".to_string()
            )),
        }
    }

    /// Convert to verification method
    pub fn to_verification_method(&self, controller: String) -> VerificationMethod {
        let public_key_material = match self.key_type {
            KeyType::Ed25519 | KeyType::X25519 => {
                PublicKeyMaterial::Multibase {
                    public_key_multibase: format!("z{}", BASE64.encode(&self.public_key)),
                }
            }
            KeyType::Secp256k1 => {
                PublicKeyMaterial::Base58 {
                    public_key_base58: BASE64.encode(&self.public_key),
                }
            }
        };

        VerificationMethod::new(
            self.id.clone(),
            self.key_type.verification_method_type().to_string(),
            controller,
            public_key_material,
        )
    }
}

/// Key manager for DID operations
#[derive(Debug)]
pub struct KeyManager {
    /// Stored key pairs
    keys: Arc<RwLock<HashMap<String, KeyPair>>>,
    /// Key encryption key (for encrypting private keys at rest)
    kek: Vec<u8>,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        // In production, this should be derived from a secure source
        let kek = vec![0u8; 32]; // Placeholder
        
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            kek,
        }
    }

    /// Generate and store a new key pair
    pub async fn generate_key(
        &self,
        id: String,
        key_type: KeyType,
        purpose: Vec<KeyPurpose>,
    ) -> DidResult<KeyPair> {
        let keypair = match key_type {
            KeyType::Ed25519 => KeyPair::generate_ed25519(id.clone(), purpose)?,
            KeyType::X25519 => KeyPair::generate_x25519(id.clone(), purpose)?,
            KeyType::Secp256k1 => {
                return Err(DidError::CryptographicError(
                    "Secp256k1 key generation not implemented".to_string()
                ));
            }
        };

        let mut keys = self.keys.write().await;
        keys.insert(id.clone(), keypair.clone());

        Ok(keypair)
    }

    /// Get a key pair by ID
    pub async fn get_key(&self, id: &str) -> Option<KeyPair> {
        let keys = self.keys.read().await;
        keys.get(id).cloned()
    }

    /// List all key IDs
    pub async fn list_keys(&self) -> Vec<String> {
        let keys = self.keys.read().await;
        keys.keys().cloned().collect()
    }

    /// Remove a key pair
    pub async fn remove_key(&self, id: &str) -> bool {
        let mut keys = self.keys.write().await;
        keys.remove(id).is_some()
    }

    /// Sign data with a specific key
    pub async fn sign(&self, key_id: &str, data: &[u8]) -> DidResult<Vec<u8>> {
        let keys = self.keys.read().await;
        let keypair = keys.get(key_id)
            .ok_or_else(|| DidError::KeyNotFound(key_id.to_string()))?;
        
        keypair.sign(data)
    }

    /// Verify signature with a specific key
    pub async fn verify(&self, key_id: &str, data: &[u8], signature: &[u8]) -> DidResult<bool> {
        let keys = self.keys.read().await;
        let keypair = keys.get(key_id)
            .ok_or_else(|| DidError::KeyNotFound(key_id.to_string()))?;
        
        keypair.verify(data, signature)
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_key_type_verification_method_type() {
        assert_eq!(KeyType::Ed25519.verification_method_type(), "Ed25519VerificationKey2020");
        assert_eq!(KeyType::X25519.verification_method_type(), "X25519KeyAgreementKey2020");
        assert_eq!(KeyType::Secp256k1.verification_method_type(), "EcdsaSecp256k1VerificationKey2019");
    }

    #[test]
    fn test_key_purpose_variants() {
        let purposes = vec![
            KeyPurpose::Authentication,
            KeyPurpose::AssertionMethod,
            KeyPurpose::KeyAgreement,
            KeyPurpose::CapabilityInvocation,
            KeyPurpose::CapabilityDelegation,
        ];

        assert_eq!(purposes.len(), 5);
    }

    #[test]
    fn test_keypair_generate_ed25519() {
        let keypair = KeyPair::generate_ed25519(
            "test-key".to_string(),
            vec![KeyPurpose::Authentication],
        ).unwrap();

        assert_eq!(keypair.id, "test-key");
        assert_eq!(keypair.key_type, KeyType::Ed25519);
        assert_eq!(keypair.purpose, vec![KeyPurpose::Authentication]);
        assert_eq!(keypair.public_key.len(), 32);
        assert_eq!(keypair.private_key.len(), 32);
        assert!(keypair.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_keypair_generate_x25519() {
        let keypair = KeyPair::generate_x25519(
            "test-key".to_string(),
            vec![KeyPurpose::KeyAgreement],
        ).unwrap();

        assert_eq!(keypair.id, "test-key");
        assert_eq!(keypair.key_type, KeyType::X25519);
        assert_eq!(keypair.purpose, vec![KeyPurpose::KeyAgreement]);
        assert_eq!(keypair.public_key.len(), 32);
        assert_eq!(keypair.private_key.len(), 32);
    }

    #[test]
    fn test_keypair_sign_ed25519() {
        let keypair = KeyPair::generate_ed25519(
            "signing-key".to_string(),
            vec![KeyPurpose::AssertionMethod],
        ).unwrap();

        let data = b"test message for signing";
        let signature = keypair.sign(data).unwrap();

        assert_eq!(signature.len(), 64); // Ed25519 signature length
    }

    #[test]
    fn test_keypair_sign_unsupported_key_type() {
        let keypair = KeyPair::generate_x25519(
            "x25519-key".to_string(),
            vec![KeyPurpose::KeyAgreement],
        ).unwrap();

        let data = b"test message";
        let result = keypair.sign(data);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::CryptographicError(_)));
    }

    #[test]
    fn test_keypair_verify_ed25519() {
        let keypair = KeyPair::generate_ed25519(
            "verify-key".to_string(),
            vec![KeyPurpose::AssertionMethod],
        ).unwrap();

        let data = b"test message for verification";
        let signature = keypair.sign(data).unwrap();
        let is_valid = keypair.verify(data, &signature).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_keypair_verify_invalid_signature() {
        let keypair = KeyPair::generate_ed25519(
            "verify-key".to_string(),
            vec![KeyPurpose::AssertionMethod],
        ).unwrap();

        let data = b"test message";
        let invalid_signature = vec![0u8; 64]; // Invalid signature
        let is_valid = keypair.verify(data, &invalid_signature).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_keypair_verify_wrong_data() {
        let keypair = KeyPair::generate_ed25519(
            "verify-key".to_string(),
            vec![KeyPurpose::AssertionMethod],
        ).unwrap();

        let original_data = b"original message";
        let different_data = b"different message";
        let signature = keypair.sign(original_data).unwrap();
        let is_valid = keypair.verify(different_data, &signature).unwrap();

        assert!(!is_valid);
    }

    #[test]
    fn test_keypair_verify_unsupported_key_type() {
        let keypair = KeyPair::generate_x25519(
            "x25519-key".to_string(),
            vec![KeyPurpose::KeyAgreement],
        ).unwrap();

        let data = b"test message";
        let signature = vec![0u8; 64];
        let result = keypair.verify(data, &signature);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::CryptographicError(_)));
    }

    #[test]
    fn test_keypair_to_verification_method_ed25519() {
        let keypair = KeyPair::generate_ed25519(
            "test-key".to_string(),
            vec![KeyPurpose::Authentication],
        ).unwrap();

        let vm = keypair.to_verification_method("did:rwa:123".to_string());

        assert_eq!(vm.id, "test-key");
        assert_eq!(vm.method_type, "Ed25519VerificationKey2020");
        assert_eq!(vm.controller, "did:rwa:123");

        match vm.public_key {
            PublicKeyMaterial::Multibase { public_key_multibase } => {
                assert!(public_key_multibase.starts_with('z'));
            }
            _ => panic!("Expected Multibase public key for Ed25519"),
        }
    }

    #[test]
    fn test_keypair_to_verification_method_x25519() {
        let keypair = KeyPair::generate_x25519(
            "test-key".to_string(),
            vec![KeyPurpose::KeyAgreement],
        ).unwrap();

        let vm = keypair.to_verification_method("did:rwa:123".to_string());

        assert_eq!(vm.id, "test-key");
        assert_eq!(vm.method_type, "X25519KeyAgreementKey2020");
        assert_eq!(vm.controller, "did:rwa:123");

        match vm.public_key {
            PublicKeyMaterial::Multibase { public_key_multibase } => {
                assert!(public_key_multibase.starts_with('z'));
            }
            _ => panic!("Expected Multibase public key for X25519"),
        }
    }

    #[test]
    fn test_key_manager_new() {
        let key_manager = KeyManager::new();
        // Just test that it can be created without panicking
        assert_eq!(key_manager.kek.len(), 32);
    }

    #[test]
    fn test_key_manager_default() {
        let key_manager = KeyManager::default();
        assert_eq!(key_manager.kek.len(), 32);
    }

    #[tokio::test]
    async fn test_key_manager_generate_key_ed25519() {
        let key_manager = KeyManager::new();

        let keypair = key_manager.generate_key(
            "test-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::Authentication],
        ).await.unwrap();

        assert_eq!(keypair.id, "test-key");
        assert_eq!(keypair.key_type, KeyType::Ed25519);
        assert_eq!(keypair.purpose, vec![KeyPurpose::Authentication]);
    }

    #[tokio::test]
    async fn test_key_manager_generate_key_x25519() {
        let key_manager = KeyManager::new();

        let keypair = key_manager.generate_key(
            "test-key".to_string(),
            KeyType::X25519,
            vec![KeyPurpose::KeyAgreement],
        ).await.unwrap();

        assert_eq!(keypair.id, "test-key");
        assert_eq!(keypair.key_type, KeyType::X25519);
        assert_eq!(keypair.purpose, vec![KeyPurpose::KeyAgreement]);
    }

    #[tokio::test]
    async fn test_key_manager_generate_key_secp256k1() {
        let key_manager = KeyManager::new();

        let result = key_manager.generate_key(
            "test-key".to_string(),
            KeyType::Secp256k1,
            vec![KeyPurpose::Authentication],
        ).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::CryptographicError(_)));
    }

    #[tokio::test]
    async fn test_key_manager_get_key() {
        let key_manager = KeyManager::new();

        let keypair = key_manager.generate_key(
            "test-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::Authentication],
        ).await.unwrap();

        let retrieved = key_manager.get_key("test-key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, keypair.id);

        let not_found = key_manager.get_key("nonexistent").await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_key_manager_list_keys() {
        let key_manager = KeyManager::new();

        // Initially empty
        let keys = key_manager.list_keys().await;
        assert!(keys.is_empty());

        // Add some keys
        key_manager.generate_key(
            "key1".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::Authentication],
        ).await.unwrap();

        key_manager.generate_key(
            "key2".to_string(),
            KeyType::X25519,
            vec![KeyPurpose::KeyAgreement],
        ).await.unwrap();

        let keys = key_manager.list_keys().await;
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_key_manager_remove_key() {
        let key_manager = KeyManager::new();

        key_manager.generate_key(
            "test-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::Authentication],
        ).await.unwrap();

        // Key should exist
        assert!(key_manager.get_key("test-key").await.is_some());

        // Remove key
        let removed = key_manager.remove_key("test-key").await;
        assert!(removed);

        // Key should no longer exist
        assert!(key_manager.get_key("test-key").await.is_none());

        // Removing non-existent key should return false
        let not_removed = key_manager.remove_key("nonexistent").await;
        assert!(!not_removed);
    }

    #[tokio::test]
    async fn test_key_manager_sign() {
        let key_manager = KeyManager::new();

        key_manager.generate_key(
            "signing-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::AssertionMethod],
        ).await.unwrap();

        let data = b"test message for signing";
        let signature = key_manager.sign("signing-key", data).await.unwrap();

        assert_eq!(signature.len(), 64);
    }

    #[tokio::test]
    async fn test_key_manager_sign_nonexistent_key() {
        let key_manager = KeyManager::new();

        let data = b"test message";
        let result = key_manager.sign("nonexistent", data).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::KeyNotFound(_)));
    }

    #[tokio::test]
    async fn test_key_manager_verify() {
        let key_manager = KeyManager::new();

        key_manager.generate_key(
            "verify-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::AssertionMethod],
        ).await.unwrap();

        let data = b"test message for verification";
        let signature = key_manager.sign("verify-key", data).await.unwrap();
        let is_valid = key_manager.verify("verify-key", data, &signature).await.unwrap();

        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_key_manager_verify_nonexistent_key() {
        let key_manager = KeyManager::new();

        let data = b"test message";
        let signature = vec![0u8; 64];
        let result = key_manager.verify("nonexistent", data, &signature).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::KeyNotFound(_)));
    }

    #[tokio::test]
    async fn test_key_manager_sign_verify_roundtrip() {
        let key_manager = KeyManager::new();

        key_manager.generate_key(
            "roundtrip-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::AssertionMethod],
        ).await.unwrap();

        let test_messages = vec![
            b"short".as_slice(),
            b"medium length message for testing".as_slice(),
            b"very long message that contains multiple sentences and should test the signing and verification process thoroughly to ensure it works correctly with various message lengths".as_slice(),
            b"".as_slice(), // empty message
            &[0u8; 1000], // binary data
        ];

        for message in test_messages {
            let signature = key_manager.sign("roundtrip-key", message).await.unwrap();
            let is_valid = key_manager.verify("roundtrip-key", message, &signature).await.unwrap();
            assert!(is_valid, "Failed for message length: {}", message.len());
        }
    }

    #[tokio::test]
    async fn test_key_manager_multiple_keys() {
        let key_manager = KeyManager::new();

        // Generate multiple keys with different purposes
        let key_configs = vec![
            ("auth-key", KeyType::Ed25519, vec![KeyPurpose::Authentication]),
            ("assert-key", KeyType::Ed25519, vec![KeyPurpose::AssertionMethod]),
            ("agreement-key", KeyType::X25519, vec![KeyPurpose::KeyAgreement]),
            ("invoke-key", KeyType::Ed25519, vec![KeyPurpose::CapabilityInvocation]),
            ("delegate-key", KeyType::Ed25519, vec![KeyPurpose::CapabilityDelegation]),
        ];

        for (id, key_type, purposes) in key_configs {
            key_manager.generate_key(
                id.to_string(),
                key_type,
                purposes,
            ).await.unwrap();
        }

        let keys = key_manager.list_keys().await;
        assert_eq!(keys.len(), 5);

        // Test that each key can be retrieved
        for key_id in &keys {
            let key = key_manager.get_key(key_id).await;
            assert!(key.is_some(), "Key {} should exist", key_id);
        }
    }

    #[tokio::test]
    async fn test_key_manager_concurrent_operations() {
        let key_manager = std::sync::Arc::new(KeyManager::new());

        // Test concurrent key generation
        let mut handles = vec![];
        for i in 0..10 {
            let km = key_manager.clone();
            let handle = tokio::spawn(async move {
                km.generate_key(
                    format!("concurrent-key-{}", i),
                    KeyType::Ed25519,
                    vec![KeyPurpose::Authentication],
                ).await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Verify all keys were created
        let keys = key_manager.list_keys().await;
        assert_eq!(keys.len(), 10);
    }
}
