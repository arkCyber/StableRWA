// =====================================================================================
// DID Service Implementation
// 
// High-level DID service for the RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    DidService, DidDocument, DidError, DidResult, DidRegistry, DidResolver, DidVerifier,
    KeyManager, KeyType, KeyPurpose, VerifiableCredential, VerifiablePresentation,
    CredentialIssuer, CredentialSubject, Did, VerificationMethodReference,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// RWA DID service implementation
pub struct RwaDidService {
    /// DID registry for storage
    registry: Arc<dyn DidRegistry>,
    /// DID resolver for resolution
    resolver: Arc<dyn DidResolver>,
    /// DID verifier for verification
    verifier: Arc<dyn DidVerifier>,
    /// Key manager for cryptographic operations
    key_manager: Arc<KeyManager>,
}

impl RwaDidService {
    /// Create a new RWA DID service
    pub fn new(
        registry: Arc<dyn DidRegistry>,
        resolver: Arc<dyn DidResolver>,
        verifier: Arc<dyn DidVerifier>,
        key_manager: Arc<KeyManager>,
    ) -> Self {
        Self {
            registry,
            resolver,
            verifier,
            key_manager,
        }
    }

    /// Create a DID with specific key types and purposes
    pub async fn create_did_with_keys(
        &self,
        controller: Option<String>,
        key_configs: Vec<(KeyType, Vec<KeyPurpose>)>,
    ) -> DidResult<(Did, DidDocument)> {
        // Generate new DID
        let did = Did::generate();
        let mut document = DidDocument::new(did.clone());

        // Set controller if provided
        if let Some(controller) = controller {
            document.set_controller(vec![controller]);
        }

        // Generate keys and add to document
        for (i, (key_type, purposes)) in key_configs.into_iter().enumerate() {
            let key_id = format!("{}#key-{}", did.to_string(), i + 1);
            
            // Generate key pair
            let keypair = self.key_manager.generate_key(
                key_id.clone(),
                key_type,
                purposes.clone(),
            ).await?;

            // Add verification method to document
            let vm = keypair.to_verification_method(did.to_string());
            document.add_verification_method(vm);

            // Add to appropriate verification relationships
            let vm_ref = VerificationMethodReference::Id(key_id);
            for purpose in purposes {
                match purpose {
                    KeyPurpose::Authentication => {
                        document.add_authentication(vm_ref.clone());
                    }
                    KeyPurpose::AssertionMethod => {
                        document.add_assertion_method(vm_ref.clone());
                    }
                    KeyPurpose::KeyAgreement => {
                        document.add_key_agreement(vm_ref.clone());
                    }
                    KeyPurpose::CapabilityInvocation => {
                        document.add_capability_invocation(vm_ref.clone());
                    }
                    KeyPurpose::CapabilityDelegation => {
                        document.add_capability_delegation(vm_ref.clone());
                    }
                }
            }
        }

        // Store in registry
        self.registry.create(did.to_string(), document.clone()).await?;

        Ok((did, document))
    }

    /// Add a service endpoint to a DID document
    pub async fn add_service_endpoint(
        &self,
        did: &str,
        service_id: String,
        service_type: String,
        service_endpoint: crate::ServiceEndpointUrl,
    ) -> DidResult<()> {
        // Resolve current document
        let mut document = self.registry.read(did).await?
            .ok_or_else(|| DidError::DidNotFound(did.to_string()))?;

        // Add service endpoint
        let service = crate::ServiceEndpoint::new(service_id, service_type, service_endpoint);
        document.add_service(service);

        // Update in registry
        self.registry.update(did.to_string(), document).await?;

        Ok(())
    }

    /// Rotate a key in a DID document
    pub async fn rotate_key(
        &self,
        did: &str,
        old_key_id: &str,
        new_key_type: KeyType,
        purposes: Vec<KeyPurpose>,
    ) -> DidResult<String> {
        // Resolve current document
        let mut document = self.registry.read(did).await?
            .ok_or_else(|| DidError::DidNotFound(did.to_string()))?;

        // Generate new key
        let new_key_id = format!("{}#key-{}", did, chrono::Utc::now().timestamp());
        let keypair = self.key_manager.generate_key(
            new_key_id.clone(),
            new_key_type,
            purposes.clone(),
        ).await?;

        // Remove old verification method
        document.verification_method.retain(|vm| vm.id != old_key_id);

        // Add new verification method
        let vm = keypair.to_verification_method(did.to_string());
        document.add_verification_method(vm);

        // Update verification relationships
        let new_vm_ref = VerificationMethodReference::Id(new_key_id.clone());
        
        // Remove old references and add new ones
        for purpose in purposes {
            match purpose {
                KeyPurpose::Authentication => {
                    document.authentication.retain(|auth| {
                        match auth {
                            VerificationMethodReference::Id(id) => id != old_key_id,
                            _ => true,
                        }
                    });
                    document.add_authentication(new_vm_ref.clone());
                }
                KeyPurpose::AssertionMethod => {
                    document.assertion_method.retain(|assertion| {
                        match assertion {
                            VerificationMethodReference::Id(id) => id != old_key_id,
                            _ => true,
                        }
                    });
                    document.add_assertion_method(new_vm_ref.clone());
                }
                KeyPurpose::KeyAgreement => {
                    document.key_agreement.retain(|agreement| {
                        match agreement {
                            VerificationMethodReference::Id(id) => id != old_key_id,
                            _ => true,
                        }
                    });
                    document.add_key_agreement(new_vm_ref.clone());
                }
                KeyPurpose::CapabilityInvocation => {
                    document.capability_invocation.retain(|invocation| {
                        match invocation {
                            VerificationMethodReference::Id(id) => id != old_key_id,
                            _ => true,
                        }
                    });
                    document.add_capability_invocation(new_vm_ref.clone());
                }
                KeyPurpose::CapabilityDelegation => {
                    document.capability_delegation.retain(|delegation| {
                        match delegation {
                            VerificationMethodReference::Id(id) => id != old_key_id,
                            _ => true,
                        }
                    });
                    document.add_capability_delegation(new_vm_ref.clone());
                }
            }
        }

        // Remove old key from key manager
        self.key_manager.remove_key(old_key_id).await;

        // Update in registry
        self.registry.update(did.to_string(), document).await?;

        Ok(new_key_id)
    }
}

#[async_trait]
impl DidService for RwaDidService {
    async fn create_did(&self, controller: Option<String>) -> DidResult<DidDocument> {
        // Create DID with default key configuration
        let key_configs = vec![
            (KeyType::Ed25519, vec![KeyPurpose::Authentication, KeyPurpose::AssertionMethod]),
            (KeyType::X25519, vec![KeyPurpose::KeyAgreement]),
        ];

        let (_, document) = self.create_did_with_keys(controller, key_configs).await?;
        Ok(document)
    }

    async fn resolve_did(&self, did: &str) -> DidResult<DidDocument> {
        let resolution_result = self.resolver.resolve(did).await?;
        resolution_result.did_document
            .ok_or_else(|| DidError::DidNotFound(did.to_string()))
    }

    async fn update_did(&self, did: &str, document: DidDocument) -> DidResult<()> {
        self.registry.update(did.to_string(), document).await
    }

    async fn deactivate_did(&self, did: &str) -> DidResult<()> {
        self.registry.deactivate(did).await
    }

    async fn issue_credential(
        &self,
        issuer_did: &str,
        subject_did: &str,
        claims: HashMap<String, serde_json::Value>,
    ) -> DidResult<VerifiableCredential> {
        // Create credential subject
        let credential_subject = CredentialSubject {
            id: Some(subject_did.to_string()),
            claims,
        };

        // Create credential
        let issuer = CredentialIssuer::Did(issuer_did.to_string());
        let credential_types = vec![
            "VerifiableCredential".to_string(),
            "RwaCredential".to_string(),
        ];

        let credential = VerifiableCredential::new(issuer, credential_subject, credential_types);

        // TODO: Add proof by signing with issuer's key
        // This would require implementing the signing logic

        Ok(credential)
    }

    async fn verify_credential(&self, credential: &VerifiableCredential) -> DidResult<bool> {
        // Validate credential structure
        credential.validate()?;

        // TODO: Verify proof signature
        // This would require extracting the proof and verifying with the issuer's key

        Ok(true) // Placeholder
    }

    async fn create_presentation(
        &self,
        holder_did: &str,
        credentials: Vec<VerifiableCredential>,
    ) -> DidResult<VerifiablePresentation> {
        let presentation = VerifiablePresentation::new(
            Some(holder_did.to_string()),
            credentials,
        );

        // TODO: Add proof by signing with holder's key

        Ok(presentation)
    }

    async fn verify_presentation(&self, presentation: &VerifiablePresentation) -> DidResult<bool> {
        // Validate presentation structure
        presentation.validate()?;

        // Verify all credentials in the presentation
        for credential in &presentation.verifiable_credential {
            if !self.verify_credential(credential).await? {
                return Ok(false);
            }
        }

        // TODO: Verify presentation proof

        Ok(true) // Placeholder
    }
}
