// =====================================================================================
// DID Verifier Implementation
// 
// Verifies DID-based signatures and credentials
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{DidError, DidResult, DidResolver};
use async_trait::async_trait;
use std::sync::Arc;

/// DID-based signature verifier
#[async_trait]
pub trait DidVerifier: Send + Sync {
    /// Verify a signature against a DID document
    async fn verify_signature(
        &self,
        did: &str,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
    ) -> DidResult<bool>;
    
    /// Verify that a DID has a specific capability
    async fn verify_capability(
        &self,
        did: &str,
        capability: &str,
        key_id: &str,
    ) -> DidResult<bool>;
}

/// Standard DID verifier implementation
pub struct StandardVerifier {
    /// DID resolver for fetching documents
    resolver: Arc<dyn DidResolver>,
}

impl StandardVerifier {
    /// Create a new standard verifier
    pub fn new(resolver: Arc<dyn DidResolver>) -> Self {
        Self { resolver }
    }
}

#[async_trait]
impl DidVerifier for StandardVerifier {
    async fn verify_signature(
        &self,
        did: &str,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
    ) -> DidResult<bool> {
        // Resolve DID document
        let resolution_result = self.resolver.resolve(did).await?;
        
        let document = resolution_result.did_document
            .ok_or_else(|| DidError::DidNotFound(did.to_string()))?;
        
        // Find verification method
        let verification_method = document.get_verification_method(key_id)
            .ok_or_else(|| DidError::KeyNotFound(key_id.to_string()))?;
        
        // Verify signature based on key type
        self.verify_with_method(verification_method, data, signature).await
    }
    
    async fn verify_capability(
        &self,
        did: &str,
        capability: &str,
        key_id: &str,
    ) -> DidResult<bool> {
        // Resolve DID document
        let resolution_result = self.resolver.resolve(did).await?;
        
        let document = resolution_result.did_document
            .ok_or_else(|| DidError::DidNotFound(did.to_string()))?;
        
        // Check if key is authorized for the capability
        match capability {
            "authentication" => {
                Ok(document.authentication.iter().any(|auth| {
                    match auth {
                        crate::VerificationMethodReference::Id(id) => id == key_id,
                        crate::VerificationMethodReference::Embedded(vm) => vm.id == key_id,
                    }
                }))
            }
            "assertionMethod" => {
                Ok(document.assertion_method.iter().any(|assertion| {
                    match assertion {
                        crate::VerificationMethodReference::Id(id) => id == key_id,
                        crate::VerificationMethodReference::Embedded(vm) => vm.id == key_id,
                    }
                }))
            }
            "keyAgreement" => {
                Ok(document.key_agreement.iter().any(|agreement| {
                    match agreement {
                        crate::VerificationMethodReference::Id(id) => id == key_id,
                        crate::VerificationMethodReference::Embedded(vm) => vm.id == key_id,
                    }
                }))
            }
            "capabilityInvocation" => {
                Ok(document.capability_invocation.iter().any(|invocation| {
                    match invocation {
                        crate::VerificationMethodReference::Id(id) => id == key_id,
                        crate::VerificationMethodReference::Embedded(vm) => vm.id == key_id,
                    }
                }))
            }
            "capabilityDelegation" => {
                Ok(document.capability_delegation.iter().any(|delegation| {
                    match delegation {
                        crate::VerificationMethodReference::Id(id) => id == key_id,
                        crate::VerificationMethodReference::Embedded(vm) => vm.id == key_id,
                    }
                }))
            }
            _ => Err(DidError::InvalidVerificationMethod(
                format!("Unknown capability: {}", capability)
            )),
        }
    }
}

impl StandardVerifier {
    /// Verify signature with a specific verification method
    async fn verify_with_method(
        &self,
        verification_method: &crate::VerificationMethod,
        data: &[u8],
        signature: &[u8],
    ) -> DidResult<bool> {
        match verification_method.method_type.as_str() {
            "Ed25519VerificationKey2020" => {
                self.verify_ed25519(verification_method, data, signature).await
            }
            "EcdsaSecp256k1VerificationKey2019" => {
                self.verify_secp256k1(verification_method, data, signature).await
            }
            _ => Err(DidError::InvalidVerificationMethod(
                format!("Unsupported verification method type: {}", verification_method.method_type)
            )),
        }
    }
    
    /// Verify Ed25519 signature
    async fn verify_ed25519(
        &self,
        verification_method: &crate::VerificationMethod,
        data: &[u8],
        signature: &[u8],
    ) -> DidResult<bool> {
        use ed25519_dalek::{VerifyingKey, Signature, Verifier};
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        
        // Extract public key
        let public_key_bytes = match &verification_method.public_key {
            crate::PublicKeyMaterial::Base58 { public_key_base58 } => {
                BASE64.decode(public_key_base58)
                    .map_err(|e| DidError::InvalidKeyFormat(e.to_string()))?
            }
            crate::PublicKeyMaterial::Multibase { public_key_multibase } => {
                // Remove multibase prefix and decode
                if public_key_multibase.starts_with('z') {
                    BASE64.decode(&public_key_multibase[1..])
                        .map_err(|e| DidError::InvalidKeyFormat(e.to_string()))?
                } else {
                    return Err(DidError::InvalidKeyFormat(
                        "Invalid multibase format".to_string()
                    ));
                }
            }
            _ => {
                return Err(DidError::InvalidKeyFormat(
                    "Unsupported public key format for Ed25519".to_string()
                ));
            }
        };
        
        // Create public key
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into()
            .map_err(|_| DidError::CryptographicError("Invalid public key length".to_string()))?)
            .map_err(|e| DidError::CryptographicError(e.to_string()))?;

        // Create signature
        let signature = Signature::from_bytes(&signature.try_into()
            .map_err(|_| DidError::InvalidSignature("Invalid signature length".to_string()))?);

        // Verify
        match verifying_key.verify(data, &signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Verify secp256k1 signature (placeholder)
    async fn verify_secp256k1(
        &self,
        _verification_method: &crate::VerificationMethod,
        _data: &[u8],
        _signature: &[u8],
    ) -> DidResult<bool> {
        // Secp256k1 verification would be implemented here
        Err(DidError::CryptographicError(
            "Secp256k1 verification not implemented".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Did, DidDocument, MemoryResolver, KeyManager, KeyType, KeyPurpose, VerificationMethodReference};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_signature_verification() {
        // Setup
        let resolver = Arc::new(MemoryResolver::new("rwa".to_string()));
        let verifier = StandardVerifier::new(resolver.clone());
        let key_manager = KeyManager::new();
        
        // Generate key
        let keypair = key_manager.generate_key(
            "did:rwa:123#key-1".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::AssertionMethod],
        ).await.unwrap();
        
        // Create DID document
        let did = Did::new("123".to_string());
        let mut document = DidDocument::new(did.clone());
        
        // Add verification method
        let vm = keypair.to_verification_method(did.to_string());
        document.add_verification_method(vm);
        document.add_assertion_method(VerificationMethodReference::Id("did:rwa:123#key-1".to_string()));
        
        // Store document
        resolver.store(did.to_string(), document).await;
        
        // Sign data
        let data = b"test message";
        let signature = key_manager.sign("did:rwa:123#key-1", data).await.unwrap();
        
        // Verify signature
        let is_valid = verifier.verify_signature(
            &did.to_string(),
            "did:rwa:123#key-1",
            data,
            &signature,
        ).await.unwrap();
        
        assert!(is_valid);
    }
    
    #[tokio::test]
    async fn test_capability_verification() {
        // Setup
        let resolver = Arc::new(MemoryResolver::new("rwa".to_string()));
        let verifier = StandardVerifier::new(resolver.clone());
        
        // Create DID document
        let did = Did::new("123".to_string());
        let mut document = DidDocument::new(did.clone());
        
        // Add authentication method
        document.add_authentication(VerificationMethodReference::Id("did:rwa:123#key-1".to_string()));
        
        // Store document
        resolver.store(did.to_string(), document).await;
        
        // Verify capability
        let has_capability = verifier.verify_capability(
            &did.to_string(),
            "authentication",
            "did:rwa:123#key-1",
        ).await.unwrap();
        
        assert!(has_capability);
        
        // Verify non-existent capability
        let no_capability = verifier.verify_capability(
            &did.to_string(),
            "keyAgreement",
            "did:rwa:123#key-1",
        ).await.unwrap();
        
        assert!(!no_capability);
    }
}
