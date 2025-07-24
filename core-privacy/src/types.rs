// =====================================================================================
// File: core-privacy/src/types.rs
// Description: Core types for privacy and cryptographic operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Privacy service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub zkp_config: ZKPConfig,
    pub homomorphic_config: HomomorphicConfig,
    pub differential_privacy_config: DPConfig,
    pub encryption_config: EncryptionConfig,
    pub commitment_config: CommitmentConfig,
    pub anonymization_config: AnonymizationConfig,
}

/// Zero-knowledge proof configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKPConfig {
    pub curve_type: CurveType,
    pub proof_system: ProofSystem,
    pub security_level: SecurityLevel,
    pub circuit_cache_size: usize,
    pub proving_key_cache_size: usize,
    pub verification_key_cache_size: usize,
}

/// Homomorphic encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomomorphicConfig {
    pub scheme: FHEScheme,
    pub security_level: SecurityLevel,
    pub polynomial_degree: u32,
    pub coefficient_modulus: Vec<u64>,
    pub plain_modulus: u64,
}

/// Differential privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DPConfig {
    pub epsilon: f64,
    pub delta: f64,
    pub sensitivity: f64,
    pub noise_mechanism: NoiseMechanism,
    pub privacy_budget_manager: PrivacyBudgetConfig,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub symmetric_algorithm: SymmetricAlgorithm,
    pub asymmetric_algorithm: AsymmetricAlgorithm,
    pub key_size: u32,
    pub key_derivation: KeyDerivationConfig,
}

/// Commitment scheme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentConfig {
    pub scheme_type: CommitmentSchemeType,
    pub hash_function: HashFunction,
    pub security_level: SecurityLevel,
}

/// Anonymization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationConfig {
    pub k_anonymity: u32,
    pub l_diversity: u32,
    pub t_closeness: f64,
    pub suppression_threshold: f64,
    pub generalization_hierarchy: HashMap<String, Vec<String>>,
}

/// Elliptic curve types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CurveType {
    BN254,
    BLS12_381,
    Secp256k1,
    Ed25519,
}

/// Zero-knowledge proof systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofSystem {
    Groth16,
    PLONK,
    Bulletproofs,
    STARKs,
    Marlin,
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Low,    // 80-bit security
    Medium, // 128-bit security
    High,   // 192-bit security
    Ultra,  // 256-bit security
}

/// Fully homomorphic encryption schemes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FHEScheme {
    BFV,
    CKKS,
    TFHE,
    FHEW,
}

/// Noise mechanisms for differential privacy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NoiseMechanism {
    Laplace,
    Gaussian,
    Exponential,
    Geometric,
}

/// Symmetric encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SymmetricAlgorithm {
    AES256GCM,
    ChaCha20Poly1305,
    XSalsa20Poly1305,
}

/// Asymmetric encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AsymmetricAlgorithm {
    RSA,
    ECC,
    Kyber,
    NTRU,
}

/// Commitment scheme types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommitmentSchemeType {
    Pedersen,
    KZG,
    Merkle,
    Blake3,
}

/// Hash functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HashFunction {
    SHA256,
    SHA3_256,
    Blake3,
    Poseidon,
    Rescue,
}

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    pub algorithm: KeyDerivationAlgorithm,
    pub iterations: u32,
    pub salt_size: u32,
    pub output_size: u32,
}

/// Key derivation algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyDerivationAlgorithm {
    PBKDF2,
    Scrypt,
    Argon2,
    HKDF,
}

/// Privacy budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyBudgetConfig {
    pub total_epsilon: f64,
    pub total_delta: f64,
    pub time_window_hours: u32,
    pub auto_reset: bool,
    pub alert_threshold: f64,
}

/// Zero-knowledge proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProof {
    pub proof_id: Uuid,
    pub circuit_id: String,
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<String>,
    pub proof_system: ProofSystem,
    pub curve_type: CurveType,
    pub created_at: DateTime<Utc>,
    pub verified: bool,
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data_id: Uuid,
    pub ciphertext: Vec<u8>,
    pub algorithm: SymmetricAlgorithm,
    pub nonce: Vec<u8>,
    pub tag: Vec<u8>,
    pub key_id: String,
    pub encrypted_at: DateTime<Utc>,
}

/// Anonymized data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedData {
    pub data_id: Uuid,
    pub anonymized_data: HashMap<String, String>,
    pub anonymization_method: AnonymizationMethod,
    pub privacy_level: PrivacyLevel,
    pub suppressed_fields: Vec<String>,
    pub generalized_fields: Vec<String>,
    pub anonymized_at: DateTime<Utc>,
}

/// Anonymization methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnonymizationMethod {
    KAnonymity,
    LDiversity,
    TCloseness,
    DifferentialPrivacy,
    Suppression,
    Generalization,
}

/// Privacy levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrivacyLevel {
    Low,
    Medium,
    High,
    Maximum,
}

/// Commitment scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentScheme {
    pub commitment_id: Uuid,
    pub commitment: Vec<u8>,
    pub randomness: Vec<u8>,
    pub scheme_type: CommitmentSchemeType,
    pub hash_function: HashFunction,
    pub created_at: DateTime<Utc>,
}

/// Merkle proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub proof_id: Uuid,
    pub leaf_index: u64,
    pub leaf_hash: Vec<u8>,
    pub proof_path: Vec<MerkleNode>,
    pub root_hash: Vec<u8>,
    pub tree_size: u64,
    pub created_at: DateTime<Utc>,
}

/// Merkle tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: Vec<u8>,
    pub is_left: bool,
}

/// Range proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProof {
    pub proof_id: Uuid,
    pub commitment: Vec<u8>,
    pub proof_data: Vec<u8>,
    pub range_min: u64,
    pub range_max: u64,
    pub proof_system: ProofSystem,
    pub created_at: DateTime<Utc>,
}

/// Blind signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSignature {
    pub signature_id: Uuid,
    pub blinded_message: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub algorithm: SignatureAlgorithm,
    pub created_at: DateTime<Utc>,
}

/// Signature algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureAlgorithm {
    RSA_PSS,
    ECDSA,
    EdDSA,
    BLS,
    Schnorr,
}

/// Homomorphic key pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomomorphicKey {
    pub key_id: Uuid,
    pub public_key: Vec<u8>,
    pub secret_key: Option<Vec<u8>>, // None for public key only
    pub scheme: FHEScheme,
    pub parameters: HomomorphicParameters,
    pub created_at: DateTime<Utc>,
}

/// Homomorphic encryption parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomomorphicParameters {
    pub polynomial_degree: u32,
    pub coefficient_modulus: Vec<u64>,
    pub plain_modulus: u64,
    pub noise_standard_deviation: f64,
}

/// Privacy budget tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyBudget {
    pub budget_id: Uuid,
    pub total_epsilon: f64,
    pub total_delta: f64,
    pub consumed_epsilon: f64,
    pub consumed_delta: f64,
    pub remaining_epsilon: f64,
    pub remaining_delta: f64,
    pub time_window_start: DateTime<Utc>,
    pub time_window_end: DateTime<Utc>,
    pub queries: Vec<PrivacyQuery>,
}

/// Privacy query record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyQuery {
    pub query_id: Uuid,
    pub epsilon_cost: f64,
    pub delta_cost: f64,
    pub mechanism: NoiseMechanism,
    pub sensitivity: f64,
    pub executed_at: DateTime<Utc>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            zkp_config: ZKPConfig {
                curve_type: CurveType::BN254,
                proof_system: ProofSystem::Groth16,
                security_level: SecurityLevel::High,
                circuit_cache_size: 100,
                proving_key_cache_size: 50,
                verification_key_cache_size: 50,
            },
            homomorphic_config: HomomorphicConfig {
                scheme: FHEScheme::BFV,
                security_level: SecurityLevel::High,
                polynomial_degree: 4096,
                coefficient_modulus: vec![1099511627689, 1099511627691],
                plain_modulus: 1032193,
            },
            differential_privacy_config: DPConfig {
                epsilon: 1.0,
                delta: 1e-5,
                sensitivity: 1.0,
                noise_mechanism: NoiseMechanism::Laplace,
                privacy_budget_manager: PrivacyBudgetConfig {
                    total_epsilon: 10.0,
                    total_delta: 1e-4,
                    time_window_hours: 24,
                    auto_reset: true,
                    alert_threshold: 0.8,
                },
            },
            encryption_config: EncryptionConfig {
                symmetric_algorithm: SymmetricAlgorithm::AES256GCM,
                asymmetric_algorithm: AsymmetricAlgorithm::ECC,
                key_size: 256,
                key_derivation: KeyDerivationConfig {
                    algorithm: KeyDerivationAlgorithm::Argon2,
                    iterations: 100000,
                    salt_size: 32,
                    output_size: 32,
                },
            },
            commitment_config: CommitmentConfig {
                scheme_type: CommitmentSchemeType::Pedersen,
                hash_function: HashFunction::SHA256,
                security_level: SecurityLevel::High,
            },
            anonymization_config: AnonymizationConfig {
                k_anonymity: 5,
                l_diversity: 3,
                t_closeness: 0.2,
                suppression_threshold: 0.05,
                generalization_hierarchy: HashMap::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_config_default() {
        let config = PrivacyConfig::default();
        assert_eq!(config.zkp_config.curve_type, CurveType::BN254);
        assert_eq!(config.zkp_config.proof_system, ProofSystem::Groth16);
        assert_eq!(config.homomorphic_config.scheme, FHEScheme::BFV);
        assert_eq!(config.differential_privacy_config.epsilon, 1.0);
    }

    #[test]
    fn test_security_levels() {
        let levels = vec![
            SecurityLevel::Low,
            SecurityLevel::Medium,
            SecurityLevel::High,
            SecurityLevel::Ultra,
        ];

        for level in levels {
            assert!(matches!(
                level,
                SecurityLevel::Low
                    | SecurityLevel::Medium
                    | SecurityLevel::High
                    | SecurityLevel::Ultra
            ));
        }
    }

    #[test]
    fn test_zkproof_creation() {
        let proof = ZKProof {
            proof_id: Uuid::new_v4(),
            circuit_id: "test_circuit".to_string(),
            proof_data: vec![1, 2, 3, 4],
            public_inputs: vec!["input1".to_string(), "input2".to_string()],
            proof_system: ProofSystem::Groth16,
            curve_type: CurveType::BN254,
            created_at: Utc::now(),
            verified: false,
        };

        assert_eq!(proof.circuit_id, "test_circuit");
        assert_eq!(proof.public_inputs.len(), 2);
        assert!(!proof.verified);
    }
}
