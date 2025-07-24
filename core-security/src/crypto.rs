// =====================================================================================
// File: core-security/src/crypto.rs
// Description: Cryptographic utilities for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::SecurityError;
use base64::{engine::general_purpose, Engine as _};
use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM},
    digest::{self, SHA256},
    hkdf, pbkdf2,
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
// Logging imports would go here when needed

/// Encryption service for sensitive data
pub struct EncryptionService {
    key: LessSafeKey,
    rng: SystemRandom,
}

impl EncryptionService {
    /// Create new encryption service with master key
    pub fn new(master_key: &[u8]) -> Result<Self, SecurityError> {
        if master_key.len() < 32 {
            return Err(SecurityError::EncryptionError(
                "Master key must be at least 32 bytes".to_string(),
            ));
        }

        let unbound_key = UnboundKey::new(&AES_256_GCM, master_key).map_err(|e| {
            SecurityError::EncryptionError(format!("Failed to create key: {:?}", e))
        })?;

        let key = LessSafeKey::new(unbound_key);
        let rng = SystemRandom::new();

        Ok(Self { key, rng })
    }

    /// Encrypt data with AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData, SecurityError> {
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes).map_err(|e| {
            SecurityError::EncryptionError(format!("Failed to generate nonce: {:?}", e))
        })?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut ciphertext = plaintext.to_vec();

        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|e| SecurityError::EncryptionError(format!("Encryption failed: {:?}", e)))?;

        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Decrypt data
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>, SecurityError> {
        if encrypted_data.nonce.len() != 12 {
            return Err(SecurityError::EncryptionError(
                "Invalid nonce length".to_string(),
            ));
        }

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes.copy_from_slice(&encrypted_data.nonce);
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut ciphertext = encrypted_data.ciphertext.clone();
        let plaintext = self
            .key
            .open_in_place(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|e| SecurityError::EncryptionError(format!("Decryption failed: {:?}", e)))?;

        Ok(plaintext.to_vec())
    }

    /// Encrypt string data
    pub fn encrypt_string(&self, plaintext: &str) -> Result<String, SecurityError> {
        let encrypted = self.encrypt(plaintext.as_bytes())?;
        Ok(general_purpose::STANDARD.encode(
            serde_json::to_vec(&encrypted).map_err(|e| {
                SecurityError::EncryptionError(format!("Serialization failed: {}", e))
            })?,
        ))
    }

    /// Decrypt string data
    pub fn decrypt_string(&self, encrypted_base64: &str) -> Result<String, SecurityError> {
        let encrypted_bytes = general_purpose::STANDARD
            .decode(encrypted_base64)
            .map_err(|e| SecurityError::EncryptionError(format!("Base64 decode failed: {}", e)))?;

        let encrypted_data: EncryptedData =
            serde_json::from_slice(&encrypted_bytes).map_err(|e| {
                SecurityError::EncryptionError(format!("Deserialization failed: {}", e))
            })?;

        let plaintext_bytes = self.decrypt(&encrypted_data)?;
        String::from_utf8(plaintext_bytes)
            .map_err(|e| SecurityError::EncryptionError(format!("UTF-8 decode failed: {}", e)))
    }
}

/// Encrypted data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Key derivation utilities
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive key using PBKDF2
    pub fn derive_key_pbkdf2(
        password: &str,
        salt: &[u8],
        iterations: u32,
        key_length: usize,
    ) -> Result<Vec<u8>, SecurityError> {
        let iterations = NonZeroU32::new(iterations).ok_or_else(|| {
            SecurityError::EncryptionError("Iterations must be non-zero".to_string())
        })?;

        let mut key = vec![0u8; key_length];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            salt,
            password.as_bytes(),
            &mut key,
        );

        Ok(key)
    }

    /// Derive key using HKDF
    pub fn derive_key_hkdf(
        input_key_material: &[u8],
        salt: Option<&[u8]>,
        info: &[u8],
        key_length: usize,
    ) -> Result<Vec<u8>, SecurityError> {
        let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, salt.unwrap_or(&[]));
        let prk = salt.extract(input_key_material);

        let mut key = vec![0u8; key_length];
        prk.expand(&[info], hkdf::HKDF_SHA256)
            .map_err(|e| SecurityError::EncryptionError(format!("HKDF expand failed: {:?}", e)))?
            .fill(&mut key)
            .map_err(|e| SecurityError::EncryptionError(format!("HKDF fill failed: {:?}", e)))?;

        Ok(key)
    }

    /// Generate random salt
    pub fn generate_salt(length: usize) -> Result<Vec<u8>, SecurityError> {
        let rng = SystemRandom::new();
        let mut salt = vec![0u8; length];
        rng.fill(&mut salt).map_err(|e| {
            SecurityError::EncryptionError(format!("Failed to generate salt: {:?}", e))
        })?;
        Ok(salt)
    }
}

/// Hash utilities
pub struct HashUtils;

impl HashUtils {
    /// SHA-256 hash
    pub fn sha256(data: &[u8]) -> Vec<u8> {
        digest::digest(&SHA256, data).as_ref().to_vec()
    }

    /// SHA-256 hash of string
    pub fn sha256_string(data: &str) -> String {
        let hash = Self::sha256(data.as_bytes());
        hex::encode(hash)
    }

    /// Verify SHA-256 hash
    pub fn verify_sha256(data: &[u8], expected_hash: &[u8]) -> bool {
        let actual_hash = Self::sha256(data);
        actual_hash == expected_hash
    }

    /// HMAC-SHA256
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
        use ring::hmac;
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        hmac::sign(&key, data).as_ref().to_vec()
    }

    /// Verify HMAC-SHA256
    pub fn verify_hmac_sha256(key: &[u8], data: &[u8], expected_hmac: &[u8]) -> bool {
        let actual_hmac = Self::hmac_sha256(key, data);
        actual_hmac == expected_hmac
    }
}

/// Digital signature utilities (placeholder for future implementation)
pub struct SignatureUtils;

impl SignatureUtils {
    /// Generate key pair for signing (placeholder)
    pub fn generate_keypair() -> Result<(Vec<u8>, Vec<u8>), SecurityError> {
        // This would implement Ed25519 or ECDSA key generation
        Err(SecurityError::EncryptionError(
            "Not implemented".to_string(),
        ))
    }

    /// Sign data (placeholder)
    pub fn sign(_private_key: &[u8], _data: &[u8]) -> Result<Vec<u8>, SecurityError> {
        // This would implement digital signing
        Err(SecurityError::EncryptionError(
            "Not implemented".to_string(),
        ))
    }

    /// Verify signature (placeholder)
    pub fn verify(
        _public_key: &[u8],
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, SecurityError> {
        // This would implement signature verification
        Err(SecurityError::EncryptionError(
            "Not implemented".to_string(),
        ))
    }
}

/// Secure random number generation
pub struct SecureRandomGenerator;

impl SecureRandomGenerator {
    /// Generate random bytes
    pub fn generate_bytes(length: usize) -> Result<Vec<u8>, SecurityError> {
        let rng = SystemRandom::new();
        let mut bytes = vec![0u8; length];
        rng.fill(&mut bytes).map_err(|e| {
            SecurityError::EncryptionError(format!("Failed to generate random bytes: {:?}", e))
        })?;
        Ok(bytes)
    }

    /// Generate random string (base64 encoded)
    pub fn generate_string(byte_length: usize) -> Result<String, SecurityError> {
        let bytes = Self::generate_bytes(byte_length)?;
        Ok(general_purpose::STANDARD.encode(bytes))
    }

    /// Generate random UUID-like string
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Generate random number in range
    pub fn generate_u32_in_range(min: u32, max: u32) -> Result<u32, SecurityError> {
        if min >= max {
            return Err(SecurityError::EncryptionError(
                "Invalid range: min must be less than max".to_string(),
            ));
        }

        let range = max - min;
        let bytes = Self::generate_bytes(4)?;
        let random_u32 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(min + (random_u32 % range))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_service() {
        let master_key = b"this_is_a_32_byte_master_key_123";
        let service = EncryptionService::new(master_key).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_string_encryption() {
        let master_key = b"this_is_a_32_byte_master_key_123";
        let service = EncryptionService::new(master_key).unwrap();

        let plaintext = "Hello, World!";
        let encrypted = service.encrypt_string(plaintext).unwrap();
        let decrypted = service.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = b"test_salt_123456";

        let key1 = KeyDerivation::derive_key_pbkdf2(password, salt, 10000, 32).unwrap();
        let key2 = KeyDerivation::derive_key_pbkdf2(password, salt, 10000, 32).unwrap();

        assert_eq!(key1, key2); // Same inputs should produce same key
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_hash_utils() {
        let data = b"test data";
        let hash1 = HashUtils::sha256(data);
        let hash2 = HashUtils::sha256(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32); // SHA-256 produces 32-byte hash

        assert!(HashUtils::verify_sha256(data, &hash1));
        assert!(!HashUtils::verify_sha256(b"different data", &hash1));
    }

    #[test]
    fn test_hmac() {
        let key = b"secret_key";
        let data = b"test data";

        let hmac1 = HashUtils::hmac_sha256(key, data);
        let hmac2 = HashUtils::hmac_sha256(key, data);

        assert_eq!(hmac1, hmac2);
        assert!(HashUtils::verify_hmac_sha256(key, data, &hmac1));
        assert!(!HashUtils::verify_hmac_sha256(b"wrong_key", data, &hmac1));
    }

    #[test]
    fn test_secure_random() {
        let bytes1 = SecureRandomGenerator::generate_bytes(16).unwrap();
        let bytes2 = SecureRandomGenerator::generate_bytes(16).unwrap();

        assert_eq!(bytes1.len(), 16);
        assert_eq!(bytes2.len(), 16);
        assert_ne!(bytes1, bytes2); // Should be different

        let string = SecureRandomGenerator::generate_string(16).unwrap();
        assert!(!string.is_empty());

        let uuid = SecureRandomGenerator::generate_uuid();
        assert_eq!(uuid.len(), 36); // UUID format length
    }

    #[test]
    fn test_random_range() {
        let random = SecureRandomGenerator::generate_u32_in_range(10, 20).unwrap();
        assert!(random >= 10 && random < 20);

        let invalid = SecureRandomGenerator::generate_u32_in_range(20, 10);
        assert!(invalid.is_err());
    }
}
