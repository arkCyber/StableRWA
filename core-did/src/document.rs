// =====================================================================================
// DID Document Implementation
// 
// W3C DID Document specification compliant implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{Did, DidError, DidResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

/// W3C DID Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    /// DID subject identifier
    pub id: String,
    
    /// Context (JSON-LD)
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    
    /// Controller(s) of this DID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<Vec<String>>,
    
    /// Verification methods
    #[serde(rename = "verificationMethod", skip_serializing_if = "Vec::is_empty")]
    pub verification_method: Vec<VerificationMethod>,
    
    /// Authentication verification methods
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authentication: Vec<VerificationMethodReference>,
    
    /// Assertion method verification methods
    #[serde(rename = "assertionMethod", skip_serializing_if = "Vec::is_empty")]
    pub assertion_method: Vec<VerificationMethodReference>,
    
    /// Key agreement verification methods
    #[serde(rename = "keyAgreement", skip_serializing_if = "Vec::is_empty")]
    pub key_agreement: Vec<VerificationMethodReference>,
    
    /// Capability invocation verification methods
    #[serde(rename = "capabilityInvocation", skip_serializing_if = "Vec::is_empty")]
    pub capability_invocation: Vec<VerificationMethodReference>,
    
    /// Capability delegation verification methods
    #[serde(rename = "capabilityDelegation", skip_serializing_if = "Vec::is_empty")]
    pub capability_delegation: Vec<VerificationMethodReference>,
    
    /// Service endpoints
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub service: Vec<ServiceEndpoint>,
    
    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl DidDocument {
    /// Create a new DID document
    pub fn new(did: Did) -> Self {
        Self {
            id: did.to_string(),
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            ],
            controller: None,
            verification_method: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            service: Vec::new(),
            additional_properties: HashMap::new(),
        }
    }

    /// Add a verification method
    pub fn add_verification_method(&mut self, method: VerificationMethod) {
        self.verification_method.push(method);
    }

    /// Add authentication method
    pub fn add_authentication(&mut self, reference: VerificationMethodReference) {
        self.authentication.push(reference);
    }

    /// Add assertion method
    pub fn add_assertion_method(&mut self, reference: VerificationMethodReference) {
        self.assertion_method.push(reference);
    }

    /// Add key agreement method
    pub fn add_key_agreement(&mut self, reference: VerificationMethodReference) {
        self.key_agreement.push(reference);
    }

    /// Add capability invocation method
    pub fn add_capability_invocation(&mut self, reference: VerificationMethodReference) {
        self.capability_invocation.push(reference);
    }

    /// Add capability delegation method
    pub fn add_capability_delegation(&mut self, reference: VerificationMethodReference) {
        self.capability_delegation.push(reference);
    }

    /// Add service endpoint
    pub fn add_service(&mut self, service: ServiceEndpoint) {
        self.service.push(service);
    }

    /// Set controller
    pub fn set_controller(&mut self, controller: Vec<String>) {
        self.controller = Some(controller);
    }

    /// Get verification method by ID
    pub fn get_verification_method(&self, id: &str) -> Option<&VerificationMethod> {
        self.verification_method.iter().find(|vm| vm.id == id)
    }

    /// Get service by ID
    pub fn get_service(&self, id: &str) -> Option<&ServiceEndpoint> {
        self.service.iter().find(|s| s.id == id)
    }

    /// Validate the DID document
    pub fn validate(&self) -> DidResult<()> {
        // Validate DID format
        Did::from_str(&self.id)?;

        // Validate verification methods
        for vm in &self.verification_method {
            vm.validate()?;
        }

        // Validate services
        for service in &self.service {
            service.validate()?;
        }

        Ok(())
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> DidResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| DidError::SerializationError(e.to_string()))
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> DidResult<Self> {
        let document: DidDocument = serde_json::from_str(json)
            .map_err(|e| DidError::SerializationError(e.to_string()))?;

        document.validate()?;
        Ok(document)
    }
}

/// Verification method for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// Verification method identifier
    pub id: String,

    /// Verification method type
    #[serde(rename = "type")]
    pub method_type: String,

    /// Controller of this verification method
    pub controller: String,

    /// Public key material (various formats)
    #[serde(flatten)]
    pub public_key: PublicKeyMaterial,
}

impl VerificationMethod {
    /// Create a new verification method
    pub fn new(
        id: String,
        method_type: String,
        controller: String,
        public_key: PublicKeyMaterial,
    ) -> Self {
        Self {
            id,
            method_type,
            controller,
            public_key,
        }
    }

    /// Validate verification method
    pub fn validate(&self) -> DidResult<()> {
        // Validate ID format
        if self.id.is_empty() {
            return Err(DidError::InvalidVerificationMethod(
                "ID cannot be empty".to_string()
            ));
        }

        // Validate controller
        Did::from_str(&self.controller)?;

        // Validate public key
        self.public_key.validate()?;

        Ok(())
    }
}

/// Public key material in various formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PublicKeyMaterial {
    /// Base58 encoded public key
    Base58 {
        #[serde(rename = "publicKeyBase58")]
        public_key_base58: String,
    },
    /// Multibase encoded public key
    Multibase {
        #[serde(rename = "publicKeyMultibase")]
        public_key_multibase: String,
    },
    /// JWK format public key
    Jwk {
        #[serde(rename = "publicKeyJwk")]
        public_key_jwk: serde_json::Value,
    },
    /// PEM format public key
    Pem {
        #[serde(rename = "publicKeyPem")]
        public_key_pem: String,
    },
}

impl PublicKeyMaterial {
    /// Validate public key material
    pub fn validate(&self) -> DidResult<()> {
        match self {
            PublicKeyMaterial::Base58 { public_key_base58 } => {
                if public_key_base58.is_empty() {
                    return Err(DidError::InvalidKeyFormat(
                        "Base58 key cannot be empty".to_string()
                    ));
                }
            }
            PublicKeyMaterial::Multibase { public_key_multibase } => {
                if public_key_multibase.is_empty() {
                    return Err(DidError::InvalidKeyFormat(
                        "Multibase key cannot be empty".to_string()
                    ));
                }
            }
            PublicKeyMaterial::Jwk { public_key_jwk: _ } => {
                // JWK validation would go here
            }
            PublicKeyMaterial::Pem { public_key_pem } => {
                if public_key_pem.is_empty() {
                    return Err(DidError::InvalidKeyFormat(
                        "PEM key cannot be empty".to_string()
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Reference to a verification method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VerificationMethodReference {
    /// Direct reference by ID
    Id(String),
    /// Embedded verification method
    Embedded(VerificationMethod),
}

/// Service endpoint for DID services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Service identifier
    pub id: String,

    /// Service type
    #[serde(rename = "type")]
    pub service_type: String,

    /// Service endpoint URL(s)
    #[serde(rename = "serviceEndpoint")]
    pub service_endpoint: ServiceEndpointUrl,
}

impl ServiceEndpoint {
    /// Create a new service endpoint
    pub fn new(id: String, service_type: String, service_endpoint: ServiceEndpointUrl) -> Self {
        Self {
            id,
            service_type,
            service_endpoint,
        }
    }

    /// Validate service endpoint
    pub fn validate(&self) -> DidResult<()> {
        if self.id.is_empty() {
            return Err(DidError::InvalidServiceEndpoint(
                "Service ID cannot be empty".to_string()
            ));
        }

        if self.service_type.is_empty() {
            return Err(DidError::InvalidServiceEndpoint(
                "Service type cannot be empty".to_string()
            ));
        }

        self.service_endpoint.validate()?;
        Ok(())
    }
}

/// Service endpoint URL(s)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceEndpointUrl {
    /// Single URL
    Single(String),
    /// Multiple URLs
    Multiple(Vec<String>),
    /// Complex endpoint with additional properties
    Complex(HashMap<String, serde_json::Value>),
}

impl ServiceEndpointUrl {
    /// Validate service endpoint URLs
    pub fn validate(&self) -> DidResult<()> {
        match self {
            ServiceEndpointUrl::Single(url) => {
                Url::parse(url).map_err(|_| {
                    DidError::InvalidServiceEndpoint(format!("Invalid URL: {}", url))
                })?;
            }
            ServiceEndpointUrl::Multiple(urls) => {
                for url in urls {
                    Url::parse(url).map_err(|_| {
                        DidError::InvalidServiceEndpoint(format!("Invalid URL: {}", url))
                    })?;
                }
            }
            ServiceEndpointUrl::Complex(_) => {
                // Complex validation would go here
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};


    #[test]
    fn test_did_document_new() {
        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did.clone());

        assert_eq!(document.id, did.to_string());
        assert_eq!(document.context.len(), 2);
        assert!(document.context.contains(&"https://www.w3.org/ns/did/v1".to_string()));
        assert!(document.context.contains(&"https://w3id.org/security/suites/ed25519-2020/v1".to_string()));
        assert!(document.controller.is_none());
        assert!(document.verification_method.is_empty());
        assert!(document.authentication.is_empty());
        assert!(document.assertion_method.is_empty());
        assert!(document.key_agreement.is_empty());
        assert!(document.capability_invocation.is_empty());
        assert!(document.capability_delegation.is_empty());
        assert!(document.service.is_empty());
        assert!(document.additional_properties.is_empty());
    }

    #[test]
    fn test_did_document_add_verification_method() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            "did:rwa:123456789#key-1".to_string(),
            "Ed25519VerificationKey2020".to_string(),
            "did:rwa:123456789".to_string(),
            public_key,
        );

        document.add_verification_method(vm.clone());
        assert_eq!(document.verification_method.len(), 1);
        assert_eq!(document.verification_method[0].id, vm.id);
    }

    #[test]
    fn test_did_document_add_authentication() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let auth_ref = VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string());
        document.add_authentication(auth_ref.clone());

        assert_eq!(document.authentication.len(), 1);
        match &document.authentication[0] {
            VerificationMethodReference::Id(id) => assert_eq!(id, "did:rwa:123456789#key-1"),
            _ => panic!("Expected Id reference"),
        }
    }

    #[test]
    fn test_did_document_add_assertion_method() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let assertion_ref = VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string());
        document.add_assertion_method(assertion_ref);

        assert_eq!(document.assertion_method.len(), 1);
    }

    #[test]
    fn test_did_document_add_key_agreement() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let key_agreement_ref = VerificationMethodReference::Id("did:rwa:123456789#key-2".to_string());
        document.add_key_agreement(key_agreement_ref);

        assert_eq!(document.key_agreement.len(), 1);
    }

    #[test]
    fn test_did_document_add_capability_invocation() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let capability_ref = VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string());
        document.add_capability_invocation(capability_ref);

        assert_eq!(document.capability_invocation.len(), 1);
    }

    #[test]
    fn test_did_document_add_capability_delegation() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let delegation_ref = VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string());
        document.add_capability_delegation(delegation_ref);

        assert_eq!(document.capability_delegation.len(), 1);
    }

    #[test]
    fn test_did_document_add_service() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let service = ServiceEndpoint::new(
            "did:rwa:123456789#messaging".to_string(),
            "DIDCommMessaging".to_string(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        );

        document.add_service(service.clone());
        assert_eq!(document.service.len(), 1);
        assert_eq!(document.service[0].id, service.id);
    }

    #[test]
    fn test_did_document_set_controller() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let controllers = vec!["did:rwa:controller1".to_string(), "did:rwa:controller2".to_string()];
        document.set_controller(controllers.clone());

        assert_eq!(document.controller, Some(controllers));
    }

    #[test]
    fn test_did_document_get_verification_method() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            "did:rwa:123456789#key-1".to_string(),
            "Ed25519VerificationKey2020".to_string(),
            "did:rwa:123456789".to_string(),
            public_key,
        );

        document.add_verification_method(vm.clone());

        let found_vm = document.get_verification_method("did:rwa:123456789#key-1");
        assert!(found_vm.is_some());
        assert_eq!(found_vm.unwrap().id, vm.id);

        let not_found = document.get_verification_method("did:rwa:123456789#nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_did_document_get_service() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        let service = ServiceEndpoint::new(
            "did:rwa:123456789#messaging".to_string(),
            "DIDCommMessaging".to_string(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        );

        document.add_service(service.clone());

        let found_service = document.get_service("did:rwa:123456789#messaging");
        assert!(found_service.is_some());
        assert_eq!(found_service.unwrap().id, service.id);

        let not_found = document.get_service("did:rwa:123456789#nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_did_document_validate_valid() {
        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did);

        assert!(document.validate().is_ok());
    }

    #[test]
    fn test_did_document_validate_invalid_did() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);
        document.id = "invalid-did".to_string();

        let result = document.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_did_document_to_json() {
        let did = Did::new("123456789".to_string());
        let document = DidDocument::new(did);

        let json = document.to_json().unwrap();
        assert!(json.contains("did:rwa:123456789"));
        assert!(json.contains("@context"));
        assert!(json.contains("https://www.w3.org/ns/did/v1"));
    }

    #[test]
    fn test_did_document_from_json() {
        let json = r#"{
            "@context": ["https://www.w3.org/ns/did/v1"],
            "id": "did:rwa:123456789",
            "verificationMethod": [],
            "authentication": [],
            "assertionMethod": [],
            "keyAgreement": [],
            "capabilityInvocation": [],
            "capabilityDelegation": [],
            "service": []
        }"#;

        let document = DidDocument::from_json(json).unwrap();
        assert_eq!(document.id, "did:rwa:123456789");
    }

    #[test]
    fn test_did_document_from_json_invalid() {
        let invalid_json = r#"{"invalid": "json"}"#;
        let result = DidDocument::from_json(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_verification_method_new() {
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            "did:rwa:123#key-1".to_string(),
            "Ed25519VerificationKey2020".to_string(),
            "did:rwa:123".to_string(),
            public_key.clone(),
        );

        assert_eq!(vm.id, "did:rwa:123#key-1");
        assert_eq!(vm.method_type, "Ed25519VerificationKey2020");
        assert_eq!(vm.controller, "did:rwa:123");
        match vm.public_key {
            PublicKeyMaterial::Base58 { public_key_base58 } => {
                assert_eq!(public_key_base58, "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV");
            }
            _ => panic!("Expected Base58 public key"),
        }
    }

    #[test]
    fn test_verification_method_validate_valid() {
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            "did:rwa:123#key-1".to_string(),
            "Ed25519VerificationKey2020".to_string(),
            "did:rwa:123".to_string(),
            public_key,
        );

        assert!(vm.validate().is_ok());
    }

    #[test]
    fn test_verification_method_validate_empty_id() {
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            String::new(),
            "Ed25519VerificationKey2020".to_string(),
            "did:rwa:123".to_string(),
            public_key,
        );

        let result = vm.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidVerificationMethod(_)));
    }

    #[test]
    fn test_verification_method_validate_invalid_controller() {
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            "did:rwa:123#key-1".to_string(),
            "Ed25519VerificationKey2020".to_string(),
            "invalid-controller".to_string(),
            public_key,
        );

        let result = vm.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_public_key_material_base58_validate() {
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        assert!(public_key.validate().is_ok());
    }

    #[test]
    fn test_public_key_material_base58_validate_empty() {
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: String::new(),
        };

        let result = public_key.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidKeyFormat(_)));
    }

    #[test]
    fn test_public_key_material_multibase_validate() {
        let public_key = PublicKeyMaterial::Multibase {
            public_key_multibase: "zH3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        assert!(public_key.validate().is_ok());
    }

    #[test]
    fn test_public_key_material_multibase_validate_empty() {
        let public_key = PublicKeyMaterial::Multibase {
            public_key_multibase: String::new(),
        };

        let result = public_key.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidKeyFormat(_)));
    }

    #[test]
    fn test_public_key_material_pem_validate() {
        let public_key = PublicKeyMaterial::Pem {
            public_key_pem: "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEA...\n-----END PUBLIC KEY-----".to_string(),
        };

        assert!(public_key.validate().is_ok());
    }

    #[test]
    fn test_public_key_material_pem_validate_empty() {
        let public_key = PublicKeyMaterial::Pem {
            public_key_pem: String::new(),
        };

        let result = public_key.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidKeyFormat(_)));
    }

    #[test]
    fn test_public_key_material_jwk_validate() {
        let jwk_value = serde_json::json!({
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"
        });

        let public_key = PublicKeyMaterial::Jwk {
            public_key_jwk: jwk_value,
        };

        assert!(public_key.validate().is_ok());
    }

    #[test]
    fn test_verification_method_reference_id() {
        let reference = VerificationMethodReference::Id("did:rwa:123#key-1".to_string());

        match reference {
            VerificationMethodReference::Id(id) => assert_eq!(id, "did:rwa:123#key-1"),
            _ => panic!("Expected Id reference"),
        }
    }

    #[test]
    fn test_verification_method_reference_embedded() {
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            "did:rwa:123#key-1".to_string(),
            "Ed25519VerificationKey2020".to_string(),
            "did:rwa:123".to_string(),
            public_key,
        );

        let reference = VerificationMethodReference::Embedded(vm.clone());

        match reference {
            VerificationMethodReference::Embedded(embedded_vm) => assert_eq!(embedded_vm.id, vm.id),
            _ => panic!("Expected Embedded reference"),
        }
    }

    #[test]
    fn test_service_endpoint_new() {
        let service = ServiceEndpoint::new(
            "did:rwa:123#messaging".to_string(),
            "DIDCommMessaging".to_string(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        );

        assert_eq!(service.id, "did:rwa:123#messaging");
        assert_eq!(service.service_type, "DIDCommMessaging");
        match service.service_endpoint {
            ServiceEndpointUrl::Single(url) => assert_eq!(url, "https://example.com/messaging"),
            _ => panic!("Expected Single URL"),
        }
    }

    #[test]
    fn test_service_endpoint_validate_valid() {
        let service = ServiceEndpoint::new(
            "did:rwa:123#messaging".to_string(),
            "DIDCommMessaging".to_string(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        );

        assert!(service.validate().is_ok());
    }

    #[test]
    fn test_service_endpoint_validate_empty_id() {
        let service = ServiceEndpoint::new(
            String::new(),
            "DIDCommMessaging".to_string(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        );

        let result = service.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidServiceEndpoint(_)));
    }

    #[test]
    fn test_service_endpoint_validate_empty_type() {
        let service = ServiceEndpoint::new(
            "did:rwa:123#messaging".to_string(),
            String::new(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        );

        let result = service.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidServiceEndpoint(_)));
    }

    #[test]
    fn test_service_endpoint_url_single() {
        let url = ServiceEndpointUrl::Single("https://example.com/service".to_string());
        assert!(url.validate().is_ok());
    }

    #[test]
    fn test_service_endpoint_url_single_invalid() {
        let url = ServiceEndpointUrl::Single("invalid-url".to_string());
        let result = url.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidServiceEndpoint(_)));
    }

    #[test]
    fn test_service_endpoint_url_multiple() {
        let urls = ServiceEndpointUrl::Multiple(vec![
            "https://example.com/service1".to_string(),
            "https://example.com/service2".to_string(),
        ]);
        assert!(urls.validate().is_ok());
    }

    #[test]
    fn test_service_endpoint_url_multiple_invalid() {
        let urls = ServiceEndpointUrl::Multiple(vec![
            "https://example.com/service1".to_string(),
            "invalid-url".to_string(),
        ]);
        let result = urls.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DidError::InvalidServiceEndpoint(_)));
    }

    #[test]
    fn test_service_endpoint_url_complex() {
        let mut complex_endpoint = std::collections::HashMap::new();
        complex_endpoint.insert("uri".to_string(), serde_json::json!("https://example.com/service"));
        complex_endpoint.insert("accept".to_string(), serde_json::json!(["didcomm/v2"]));

        let url = ServiceEndpointUrl::Complex(complex_endpoint);
        assert!(url.validate().is_ok());
    }

    #[test]
    fn test_did_document_roundtrip_serialization() {
        let did = Did::new("123456789".to_string());
        let mut document = DidDocument::new(did);

        // Add verification method
        let public_key = PublicKeyMaterial::Base58 {
            public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
        };

        let vm = VerificationMethod::new(
            "did:rwa:123456789#key-1".to_string(),
            "Ed25519VerificationKey2020".to_string(),
            "did:rwa:123456789".to_string(),
            public_key,
        );

        document.add_verification_method(vm);
        document.add_authentication(VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string()));
        document.add_assertion_method(VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string()));
        document.add_key_agreement(VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string()));
        document.add_capability_invocation(VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string()));
        document.add_capability_delegation(VerificationMethodReference::Id("did:rwa:123456789#key-1".to_string()));

        // Add service
        let service = ServiceEndpoint::new(
            "did:rwa:123456789#messaging".to_string(),
            "DIDCommMessaging".to_string(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        );
        document.add_service(service);

        // Set controller
        document.set_controller(vec!["did:rwa:controller".to_string()]);

        // Serialize and deserialize
        let json = document.to_json().unwrap();
        let parsed_document = DidDocument::from_json(&json).unwrap();

        assert_eq!(document.id, parsed_document.id);
        assert_eq!(document.verification_method.len(), parsed_document.verification_method.len());
        assert_eq!(document.authentication.len(), parsed_document.authentication.len());
        assert_eq!(document.service.len(), parsed_document.service.len());
        assert_eq!(document.controller, parsed_document.controller);
    }

    #[test]
    fn test_did_document_complex_scenario() {
        let did = Did::new("complex-test".to_string());
        let mut document = DidDocument::new(did);

        // Add multiple verification methods
        for i in 1..=3 {
            let public_key = PublicKeyMaterial::Multibase {
                public_key_multibase: format!("z{}", BASE64.encode(format!("key{}", i))),
            };

            let vm = VerificationMethod::new(
                format!("did:rwa:complex-test#key-{}", i),
                "Ed25519VerificationKey2020".to_string(),
                "did:rwa:complex-test".to_string(),
                public_key,
            );

            document.add_verification_method(vm);
        }

        // Add different verification relationships
        document.add_authentication(VerificationMethodReference::Id("did:rwa:complex-test#key-1".to_string()));
        document.add_assertion_method(VerificationMethodReference::Id("did:rwa:complex-test#key-2".to_string()));
        document.add_key_agreement(VerificationMethodReference::Id("did:rwa:complex-test#key-3".to_string()));

        // Add multiple services
        let services = vec![
            ("messaging", "DIDCommMessaging", "https://example.com/messaging"),
            ("storage", "DecentralizedStorage", "https://example.com/storage"),
            ("identity", "IdentityHub", "https://example.com/identity"),
        ];

        for (name, service_type, url) in services {
            let service = ServiceEndpoint::new(
                format!("did:rwa:complex-test#{}", name),
                service_type.to_string(),
                ServiceEndpointUrl::Single(url.to_string()),
            );
            document.add_service(service);
        }

        // Set multiple controllers
        document.set_controller(vec![
            "did:rwa:controller1".to_string(),
            "did:rwa:controller2".to_string(),
        ]);

        // Validate the complex document
        assert!(document.validate().is_ok());
        assert_eq!(document.verification_method.len(), 3);
        assert_eq!(document.authentication.len(), 1);
        assert_eq!(document.assertion_method.len(), 1);
        assert_eq!(document.key_agreement.len(), 1);
        assert_eq!(document.service.len(), 3);
        assert_eq!(document.controller.as_ref().unwrap().len(), 2);

        // Test serialization of complex document
        let json = document.to_json().unwrap();
        assert!(json.contains("did:rwa:complex-test"));
        assert!(json.contains("key-1"));
        assert!(json.contains("key-2"));
        assert!(json.contains("key-3"));
        assert!(json.contains("messaging"));
        assert!(json.contains("storage"));
        assert!(json.contains("identity"));
    }
}
