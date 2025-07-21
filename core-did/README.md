# Core DID - Decentralized Identity System

A comprehensive W3C DID (Decentralized Identifier) specification compliant implementation for the RWA platform.

## 🌟 Features

### ✅ W3C DID Specification Compliance
- **DID Documents**: Full support for W3C DID Document specification
- **DID URLs**: Support for DID URLs with fragments and queries
- **Verification Methods**: Multiple key types (Ed25519, X25519, Secp256k1)
- **Service Endpoints**: Support for service discovery and communication

### 🔐 Cryptographic Operations
- **Key Generation**: Ed25519 and X25519 key pair generation
- **Digital Signatures**: Ed25519 signature creation and verification
- **Key Management**: Secure key storage and rotation
- **Proof Systems**: Cryptographic proof generation and verification

### 📋 Verifiable Credentials
- **Credential Issuance**: Issue W3C Verifiable Credentials
- **Credential Verification**: Verify credential authenticity and integrity
- **Presentations**: Create and verify Verifiable Presentations
- **Status Management**: Support for credential revocation checking

### 🗄️ Storage & Resolution
- **DID Registry**: Store and manage DID documents
- **DID Resolver**: Resolve DIDs to their documents
- **Caching**: Intelligent caching for performance
- **Audit Logging**: Complete audit trail for all operations

## 🚀 Quick Start

### Basic Usage

```rust
use core_did::{
    RwaDidService, MemoryRegistry, UniversalResolver, StandardVerifier,
    KeyManager, KeyType, KeyPurpose
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup components
    let registry = Arc::new(MemoryRegistry::new());
    let resolver = Arc::new(UniversalResolver::new());
    let verifier = Arc::new(StandardVerifier::new(resolver.clone()));
    let key_manager = Arc::new(KeyManager::new());
    
    // Create DID service
    let did_service = RwaDidService::new(
        registry,
        resolver,
        verifier,
        key_manager,
    );
    
    // Create a new DID
    let document = did_service.create_did(None).await?;
    println!("Created DID: {}", document.id);
    
    // Resolve the DID
    let resolved = did_service.resolve_did(&document.id).await?;
    println!("Resolved DID document: {}", resolved.to_json()?);
    
    Ok(())
}
```

### Advanced Usage

```rust
use core_did::{
    RwaDidService, KeyType, KeyPurpose, ServiceEndpointUrl,
    CredentialIssuer, CredentialSubject, VerifiableCredential
};
use std::collections::HashMap;

async fn advanced_example(did_service: &RwaDidService) -> Result<(), Box<dyn std::error::Error>> {
    // Create DID with specific key configuration
    let key_configs = vec![
        (KeyType::Ed25519, vec![KeyPurpose::Authentication, KeyPurpose::AssertionMethod]),
        (KeyType::X25519, vec![KeyPurpose::KeyAgreement]),
    ];
    
    let (did, mut document) = did_service.create_did_with_keys(None, key_configs).await?;
    
    // Add a service endpoint
    did_service.add_service_endpoint(
        &did.to_string(),
        format!("{}#messaging", did.to_string()),
        "DIDCommMessaging".to_string(),
        ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
    ).await?;
    
    // Issue a verifiable credential
    let mut claims = HashMap::new();
    claims.insert("name".to_string(), serde_json::json!("John Doe"));
    claims.insert("age".to_string(), serde_json::json!(30));
    
    let credential = did_service.issue_credential(
        &did.to_string(),
        "did:rwa:subject123",
        claims,
    ).await?;
    
    // Verify the credential
    let is_valid = did_service.verify_credential(&credential).await?;
    println!("Credential is valid: {}", is_valid);
    
    Ok(())
}
```

## 🏗️ Architecture

### Core Components

1. **DID**: DID identifier parsing and validation
2. **DidDocument**: W3C DID Document implementation
3. **KeyManager**: Cryptographic key management
4. **DidRegistry**: DID document storage
5. **DidResolver**: DID resolution
6. **DidVerifier**: Signature and capability verification
7. **VerifiableCredential**: W3C Verifiable Credentials
8. **RwaDidService**: High-level DID service

### Key Types Supported

- **Ed25519**: Digital signatures and authentication
- **X25519**: Key agreement and encryption
- **Secp256k1**: Blockchain compatibility (planned)

### Verification Relationships

- **Authentication**: Prove control of the DID
- **Assertion Method**: Issue credentials and assertions
- **Key Agreement**: Establish secure communication
- **Capability Invocation**: Invoke capabilities
- **Capability Delegation**: Delegate capabilities

## 🔧 Configuration

```rust
use core_did::DidConfig;

let config = DidConfig {
    method: "rwa".to_string(),
    registry_url: Some("https://registry.example.com".to_string()),
    default_key_type: "Ed25519VerificationKey2020".to_string(),
    enable_cache: true,
    cache_ttl: 3600, // 1 hour
    max_resolution_attempts: 3,
    resolution_timeout: 30,
};
```

## 🧪 Testing

Run the test suite:

```bash
cargo test -p core-did
```

Run specific test modules:

```bash
cargo test -p core-did did::tests
cargo test -p core-did document::tests
cargo test -p core-did key_manager::tests
```

## 📚 Examples

### Creating a DID Document

```rust
use core_did::{Did, DidDocument, VerificationMethod, PublicKeyMaterial};

let did = Did::new("123456789".to_string());
let mut document = DidDocument::new(did.clone());

// Add verification method
let public_key = PublicKeyMaterial::Base58 {
    public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
};

let vm = VerificationMethod::new(
    format!("{}#key-1", did.to_string()),
    "Ed25519VerificationKey2020".to_string(),
    did.to_string(),
    public_key,
);

document.add_verification_method(vm);
```

### Key Management

```rust
use core_did::{KeyManager, KeyType, KeyPurpose};

let key_manager = KeyManager::new();

// Generate a new key
let keypair = key_manager.generate_key(
    "my-key".to_string(),
    KeyType::Ed25519,
    vec![KeyPurpose::Authentication],
).await?;

// Sign data
let data = b"Hello, World!";
let signature = key_manager.sign("my-key", data).await?;

// Verify signature
let is_valid = key_manager.verify("my-key", data, &signature).await?;
```

## 🔒 Security Considerations

1. **Key Storage**: Private keys should be encrypted at rest
2. **Key Rotation**: Regular key rotation is recommended
3. **Access Control**: Implement proper access controls for DID operations
4. **Audit Logging**: Enable audit logging for compliance
5. **Network Security**: Use TLS for all network communications

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🔗 References

- [W3C DID Specification](https://www.w3.org/TR/did-core/)
- [W3C Verifiable Credentials](https://www.w3.org/TR/vc-data-model/)
- [DID Method Registry](https://w3c.github.io/did-spec-registries/)
- [Ed25519 Signature Scheme](https://tools.ietf.org/html/rfc8032)
