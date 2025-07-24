// =====================================================================================
// Verifiable Credentials Implementation
//
// W3C Verifiable Credentials specification compliant implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{DidError, DidResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// W3C Verifiable Credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiableCredential {
    /// Context (JSON-LD)
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// Credential identifier
    pub id: Option<String>,

    /// Credential type
    #[serde(rename = "type")]
    pub credential_type: Vec<String>,

    /// Credential issuer
    pub issuer: CredentialIssuer,

    /// Issuance date
    #[serde(rename = "issuanceDate")]
    pub issuance_date: DateTime<Utc>,

    /// Expiration date
    #[serde(rename = "expirationDate", skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<DateTime<Utc>>,

    /// Credential subject
    #[serde(rename = "credentialSubject")]
    pub credential_subject: CredentialSubject,

    /// Credential status
    #[serde(rename = "credentialStatus", skip_serializing_if = "Option::is_none")]
    pub credential_status: Option<CredentialStatus>,

    /// Proof
    pub proof: Option<Proof>,

    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// Credential issuer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CredentialIssuer {
    /// Simple issuer DID
    Did(String),
    /// Complex issuer object
    Object {
        /// Issuer DID
        id: String,
        /// Additional properties
        #[serde(flatten)]
        properties: HashMap<String, serde_json::Value>,
    },
}

/// Credential subject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSubject {
    /// Subject DID
    pub id: Option<String>,

    /// Subject claims
    #[serde(flatten)]
    pub claims: HashMap<String, serde_json::Value>,
}

/// Credential status for revocation checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    /// Status identifier
    pub id: String,

    /// Status type
    #[serde(rename = "type")]
    pub status_type: String,

    /// Additional properties
    #[serde(flatten)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Cryptographic proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    /// Proof type
    #[serde(rename = "type")]
    pub proof_type: String,

    /// Creation timestamp
    pub created: DateTime<Utc>,

    /// Verification method
    #[serde(rename = "verificationMethod")]
    pub verification_method: String,

    /// Proof purpose
    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,

    /// Proof value
    #[serde(rename = "proofValue")]
    pub proof_value: String,

    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// W3C Verifiable Presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiablePresentation {
    /// Context (JSON-LD)
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// Presentation identifier
    pub id: Option<String>,

    /// Presentation type
    #[serde(rename = "type")]
    pub presentation_type: Vec<String>,

    /// Presentation holder
    pub holder: Option<String>,

    /// Verifiable credentials
    #[serde(rename = "verifiableCredential")]
    pub verifiable_credential: Vec<VerifiableCredential>,

    /// Proof
    pub proof: Option<Proof>,

    /// Additional properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

impl VerifiableCredential {
    /// Create a new verifiable credential
    pub fn new(
        issuer: CredentialIssuer,
        subject: CredentialSubject,
        credential_type: Vec<String>,
    ) -> Self {
        Self {
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            id: None,
            credential_type,
            issuer,
            issuance_date: Utc::now(),
            expiration_date: None,
            credential_subject: subject,
            credential_status: None,
            proof: None,
            additional_properties: HashMap::new(),
        }
    }

    /// Set credential ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Set expiration date
    pub fn with_expiration(mut self, expiration: DateTime<Utc>) -> Self {
        self.expiration_date = Some(expiration);
        self
    }

    /// Add credential status
    pub fn with_status(mut self, status: CredentialStatus) -> Self {
        self.credential_status = Some(status);
        self
    }

    /// Add proof
    pub fn with_proof(mut self, proof: Proof) -> Self {
        self.proof = Some(proof);
        self
    }

    /// Check if credential is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration_date {
            Utc::now() > expiration
        } else {
            false
        }
    }

    /// Validate credential structure
    pub fn validate(&self) -> DidResult<()> {
        // Check required fields
        if self.credential_type.is_empty() {
            return Err(DidError::InvalidCredential(
                "Credential type cannot be empty".to_string(),
            ));
        }

        if !self
            .credential_type
            .contains(&"VerifiableCredential".to_string())
        {
            return Err(DidError::InvalidCredential(
                "Credential must include 'VerifiableCredential' type".to_string(),
            ));
        }

        // Check expiration
        if self.is_expired() {
            return Err(DidError::CredentialExpired(
                self.id.clone().unwrap_or_else(|| "unknown".to_string()),
            ));
        }

        Ok(())
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> DidResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| DidError::SerializationError(e.to_string()))
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> DidResult<Self> {
        let credential: VerifiableCredential =
            serde_json::from_str(json).map_err(|e| DidError::SerializationError(e.to_string()))?;

        credential.validate()?;
        Ok(credential)
    }
}

impl VerifiablePresentation {
    /// Create a new verifiable presentation
    pub fn new(holder: Option<String>, credentials: Vec<VerifiableCredential>) -> Self {
        Self {
            context: vec!["https://www.w3.org/2018/credentials/v1".to_string()],
            id: None,
            presentation_type: vec!["VerifiablePresentation".to_string()],
            holder,
            verifiable_credential: credentials,
            proof: None,
            additional_properties: HashMap::new(),
        }
    }

    /// Set presentation ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Add proof
    pub fn with_proof(mut self, proof: Proof) -> Self {
        self.proof = Some(proof);
        self
    }

    /// Validate presentation structure
    pub fn validate(&self) -> DidResult<()> {
        // Check required fields
        if !self
            .presentation_type
            .contains(&"VerifiablePresentation".to_string())
        {
            return Err(DidError::InvalidPresentation(
                "Presentation must include 'VerifiablePresentation' type".to_string(),
            ));
        }

        // Validate all credentials
        for credential in &self.verifiable_credential {
            credential.validate()?;
        }

        Ok(())
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> DidResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| DidError::SerializationError(e.to_string()))
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> DidResult<Self> {
        let presentation: VerifiablePresentation =
            serde_json::from_str(json).map_err(|e| DidError::SerializationError(e.to_string()))?;

        presentation.validate()?;
        Ok(presentation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_creation() {
        let issuer = CredentialIssuer::Did("did:rwa:issuer".to_string());
        let mut subject = CredentialSubject {
            id: Some("did:rwa:subject".to_string()),
            claims: HashMap::new(),
        };
        subject
            .claims
            .insert("name".to_string(), serde_json::json!("John Doe"));

        let credential = VerifiableCredential::new(
            issuer,
            subject,
            vec![
                "VerifiableCredential".to_string(),
                "IdentityCredential".to_string(),
            ],
        );

        assert!(credential.validate().is_ok());
        assert!(!credential.is_expired());
    }

    #[test]
    fn test_presentation_creation() {
        let issuer = CredentialIssuer::Did("did:rwa:issuer".to_string());
        let subject = CredentialSubject {
            id: Some("did:rwa:subject".to_string()),
            claims: HashMap::new(),
        };

        let credential =
            VerifiableCredential::new(issuer, subject, vec!["VerifiableCredential".to_string()]);

        let presentation =
            VerifiablePresentation::new(Some("did:rwa:holder".to_string()), vec![credential]);

        assert!(presentation.validate().is_ok());
    }

    #[test]
    fn test_credential_serialization() {
        let issuer = CredentialIssuer::Did("did:rwa:issuer".to_string());
        let subject = CredentialSubject {
            id: Some("did:rwa:subject".to_string()),
            claims: HashMap::new(),
        };

        let credential =
            VerifiableCredential::new(issuer, subject, vec!["VerifiableCredential".to_string()]);

        let json = credential.to_json().unwrap();
        let parsed = VerifiableCredential::from_json(&json).unwrap();

        assert_eq!(credential.credential_type, parsed.credential_type);
    }
}
