// =====================================================================================
// File: core-privacy/src/lib.rs
// Description: Data privacy and zero-knowledge proof system for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Privacy Module
//!
//! This module provides comprehensive data privacy and zero-knowledge proof capabilities
//! for the StableRWA platform, including ZK-SNARKs, homomorphic encryption, secure
//! multi-party computation, and differential privacy.

pub mod error;
pub mod service;
pub mod types;
pub mod zkp;

// Placeholder modules - to be implemented later
// pub mod homomorphic;
// pub mod secure_computation;
// pub mod differential_privacy;
// pub mod encryption;
// pub mod anonymization;
// pub mod commitment;
// pub mod merkle_tree;
// pub mod range_proof;
// pub mod signature;

// Re-export main types and traits
pub use error::{PrivacyError, PrivacyResult};
pub use service::{PrivacyHealthStatus, PrivacyService, PrivacyServiceImpl};
pub use types::{
    AnonymizationMethod, AnonymizedData, BlindSignature, CommitmentScheme, CurveType,
    EncryptedData, MerkleProof, PrivacyConfig, PrivacyLevel, ProofSystem, RangeProof,
    SecurityLevel, ZKProof,
};
pub use zkp::{
    CircuitBuilder, CircuitInfo, ConstraintSystem, GrothProver, GrothVerifier, Witness, ZKPConfig,
    ZKPService, ZKPServiceImpl,
};
// Placeholder exports - to be implemented later
// pub use homomorphic::{...};
// pub use secure_computation::{...};
// pub use differential_privacy::{...};
// pub use encryption::{...};
// pub use anonymization::{...};
// pub use commitment::{...};
// pub use merkle_tree::{...};
// pub use range_proof::{...};
// pub use signature::{...};
