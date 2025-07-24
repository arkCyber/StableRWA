// =====================================================================================
// Basic DID Usage Example
//
// Demonstrates how to use the core-did library for basic DID operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use core_did::{
    DidService, KeyManager, KeyPurpose, KeyType, MemoryRegistry, MemoryResolver, RwaDidService,
    ServiceEndpointUrl, StandardVerifier, UniversalResolver,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Core DID - Basic Usage Example");
    println!("==================================\n");

    // 1. Setup DID infrastructure
    println!("1. Setting up DID infrastructure...");
    let registry = Arc::new(MemoryRegistry::new());
    let memory_resolver = Arc::new(MemoryResolver::new("rwa".to_string()));
    let mut universal_resolver = UniversalResolver::new();
    universal_resolver.register_resolver("rwa".to_string(), memory_resolver.clone());
    let resolver = Arc::new(universal_resolver);
    let verifier = Arc::new(StandardVerifier::new(resolver.clone()));
    let key_manager = Arc::new(KeyManager::new());

    let did_service = RwaDidService::new(
        registry.clone(),
        resolver.clone(),
        verifier,
        key_manager.clone(),
    );
    println!("✅ DID infrastructure setup complete\n");

    // 2. Create a new DID
    println!("2. Creating a new DID...");
    let document = did_service.create_did(None).await?;
    println!("✅ Created DID: {}", document.id);
    println!(
        "   - Verification methods: {}",
        document.verification_method.len()
    );
    println!(
        "   - Authentication methods: {}",
        document.authentication.len()
    );
    println!(
        "   - Key agreement methods: {}\n",
        document.key_agreement.len()
    );

    // Store the document in the memory resolver for resolution
    memory_resolver
        .store(document.id.clone(), document.clone())
        .await;

    // 3. Resolve the DID
    println!("3. Resolving the DID...");
    let resolved_document = did_service.resolve_did(&document.id).await?;
    println!("✅ Successfully resolved DID");
    println!("   - ID: {}", resolved_document.id);
    println!("   - Context: {:?}\n", resolved_document.context);

    // 4. Add a service endpoint
    println!("4. Adding a service endpoint...");
    did_service
        .add_service_endpoint(
            &document.id,
            format!("{}#messaging", document.id),
            "DIDCommMessaging".to_string(),
            ServiceEndpointUrl::Single("https://example.com/messaging".to_string()),
        )
        .await?;
    println!("✅ Added messaging service endpoint\n");

    // 5. Create a custom DID with specific key configuration
    println!("5. Creating a custom DID with specific keys...");
    let key_configs = vec![
        (
            KeyType::Ed25519,
            vec![KeyPurpose::Authentication, KeyPurpose::AssertionMethod],
        ),
        (KeyType::X25519, vec![KeyPurpose::KeyAgreement]),
        (KeyType::Ed25519, vec![KeyPurpose::CapabilityInvocation]),
    ];

    let (custom_did, custom_document) = did_service.create_did_with_keys(None, key_configs).await?;
    println!("✅ Created custom DID: {}", custom_did.to_string());
    println!(
        "   - Total verification methods: {}",
        custom_document.verification_method.len()
    );

    // Store in resolver
    memory_resolver
        .store(custom_did.to_string(), custom_document.clone())
        .await;
    println!();

    // 6. Issue a verifiable credential
    println!("6. Issuing a verifiable credential...");
    let mut claims = HashMap::new();
    claims.insert("name".to_string(), serde_json::json!("Alice Smith"));
    claims.insert("age".to_string(), serde_json::json!(30));
    claims.insert("role".to_string(), serde_json::json!("Developer"));
    claims.insert("company".to_string(), serde_json::json!("RWA Corp"));

    let credential = did_service
        .issue_credential(
            &document.id,            // issuer
            &custom_did.to_string(), // subject
            claims,
        )
        .await?;

    println!("✅ Issued verifiable credential");
    println!("   - Issuer: {:?}", credential.issuer);
    println!("   - Subject: {:?}", credential.credential_subject.id);
    println!("   - Types: {:?}", credential.credential_type);
    println!("   - Issuance date: {}\n", credential.issuance_date);

    // 7. Verify the credential
    println!("7. Verifying the credential...");
    let is_valid = did_service.verify_credential(&credential).await?;
    println!("✅ Credential verification result: {}\n", is_valid);

    // 8. Create a verifiable presentation
    println!("8. Creating a verifiable presentation...");
    let presentation = did_service
        .create_presentation(
            &custom_did.to_string(), // holder
            vec![credential],
        )
        .await?;

    println!("✅ Created verifiable presentation");
    println!("   - Holder: {:?}", presentation.holder);
    println!(
        "   - Credentials count: {}",
        presentation.verifiable_credential.len()
    );
    println!("   - Types: {:?}\n", presentation.presentation_type);

    // 9. Verify the presentation
    println!("9. Verifying the presentation...");
    let presentation_valid = did_service.verify_presentation(&presentation).await?;
    println!(
        "✅ Presentation verification result: {}\n",
        presentation_valid
    );

    // 10. Key management operations
    println!("10. Demonstrating key management...");

    // Generate a standalone key
    let standalone_key = key_manager
        .generate_key(
            "standalone-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::Authentication],
        )
        .await?;

    println!("✅ Generated standalone key: {}", standalone_key.id);

    // Sign some data
    let data = b"Hello, DID World!";
    let signature = key_manager.sign("standalone-key", data).await?;
    println!(
        "✅ Signed data, signature length: {} bytes",
        signature.len()
    );

    // Verify the signature
    let signature_valid = key_manager
        .verify("standalone-key", data, &signature)
        .await?;
    println!("✅ Signature verification result: {}\n", signature_valid);

    // 11. Display final statistics
    println!("11. Final statistics:");
    println!("   - Total DIDs created: 2");
    println!(
        "   - Total keys generated: {}",
        key_manager.list_keys().await.len()
    );
    println!("   - Registry entries: {}", registry.count().await);
    println!(
        "   - Resolver entries: {}",
        memory_resolver.list_dids().await.len()
    );

    println!("\n🎉 DID operations completed successfully!");
    println!("This example demonstrated:");
    println!("  • DID creation and resolution");
    println!("  • Service endpoint management");
    println!("  • Verifiable credential issuance and verification");
    println!("  • Verifiable presentation creation and verification");
    println!("  • Key management and digital signatures");

    Ok(())
}
