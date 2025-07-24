// =====================================================================================
// File: core-bridge/src/validator.rs
// Description: Bridge validator service for transaction validation and consensus
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{BridgeError, BridgeResult},
    types::{BridgeStatus, ChainId},
};

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Validator node ID
    pub node_id: String,
    /// Minimum validators required for consensus
    pub min_validators: u32,
    /// Consensus threshold (percentage)
    pub consensus_threshold: Decimal,
    /// Validation timeout in seconds
    pub validation_timeout_seconds: u64,
    /// Maximum validation attempts
    pub max_validation_attempts: u32,
    /// Validator stake requirement
    pub min_stake_amount: Decimal,
    /// Slashing penalty percentage
    pub slashing_penalty: Decimal,
    /// Reward per validation
    pub validation_reward: Decimal,
    /// Enable fraud detection
    pub fraud_detection_enabled: bool,
    /// Challenge period in seconds
    pub challenge_period_seconds: u64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            node_id: format!("validator-{}", Uuid::new_v4()),
            min_validators: 5,
            consensus_threshold: Decimal::new(67, 2), // 67%
            validation_timeout_seconds: 300,          // 5 minutes
            max_validation_attempts: 3,
            min_stake_amount: Decimal::new(10000000, 2), // $100,000
            slashing_penalty: Decimal::new(10, 2),       // 10%
            validation_reward: Decimal::new(1000, 2),    // $10.00
            fraud_detection_enabled: true,
            challenge_period_seconds: 86400, // 24 hours
        }
    }
}

/// Validation request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub source_tx_hash: String,
    pub source_block_number: u64,
    pub source_block_hash: String,
    pub message_data: Vec<u8>,
    pub proof_data: Vec<u8>,
    pub merkle_root: String,
    pub merkle_proof: Vec<String>,
    pub requested_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
}

impl ValidationRequest {
    /// Create a new validation request
    pub fn new(
        transaction_id: Uuid,
        source_chain: ChainId,
        destination_chain: ChainId,
        source_tx_hash: String,
        source_block_number: u64,
        source_block_hash: String,
        message_data: Vec<u8>,
        proof_data: Vec<u8>,
        merkle_root: String,
        merkle_proof: Vec<String>,
    ) -> Self {
        let requested_at = Utc::now();
        let deadline = requested_at + chrono::Duration::minutes(30);

        Self {
            id: Uuid::new_v4(),
            transaction_id,
            source_chain,
            destination_chain,
            source_tx_hash,
            source_block_number,
            source_block_hash,
            message_data,
            proof_data,
            merkle_root,
            merkle_proof,
            requested_at,
            deadline,
        }
    }

    /// Validate the request structure
    pub fn validate(&self) -> BridgeResult<()> {
        if self.source_chain == self.destination_chain {
            return Err(BridgeError::validation_error(
                "chains",
                "Source and destination chains cannot be the same",
            ));
        }

        if self.source_tx_hash.is_empty() {
            return Err(BridgeError::validation_error(
                "source_tx_hash",
                "Source transaction hash cannot be empty",
            ));
        }

        if self.source_block_hash.is_empty() {
            return Err(BridgeError::validation_error(
                "source_block_hash",
                "Source block hash cannot be empty",
            ));
        }

        if self.merkle_root.is_empty() {
            return Err(BridgeError::validation_error(
                "merkle_root",
                "Merkle root cannot be empty",
            ));
        }

        if self.merkle_proof.is_empty() {
            return Err(BridgeError::validation_error(
                "merkle_proof",
                "Merkle proof cannot be empty",
            ));
        }

        if self.deadline <= Utc::now() {
            return Err(BridgeError::validation_error(
                "deadline",
                "Validation deadline has passed",
            ));
        }

        Ok(())
    }
}

/// Validation response from a validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub request_id: Uuid,
    pub validator_id: String,
    pub is_valid: bool,
    pub confidence_score: f64,
    pub validation_data: ValidationData,
    pub signature: String,
    pub validated_at: DateTime<Utc>,
}

/// Validation data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationData {
    pub block_confirmed: bool,
    pub transaction_confirmed: bool,
    pub merkle_proof_valid: bool,
    pub message_integrity_valid: bool,
    pub gas_estimation: Option<u64>,
    pub additional_checks: HashMap<String, bool>,
}

/// Validation result after consensus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub request_id: Uuid,
    pub transaction_id: Uuid,
    pub consensus_reached: bool,
    pub is_valid: bool,
    pub validator_count: u32,
    pub positive_votes: u32,
    pub negative_votes: u32,
    pub consensus_percentage: Decimal,
    pub average_confidence: f64,
    pub participating_validators: Vec<String>,
    pub finalized_at: DateTime<Utc>,
    pub challenge_period_ends: DateTime<Utc>,
}

/// Bridge validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeValidator {
    pub validator_id: String,
    pub operator_address: String,
    pub stake_amount: Decimal,
    pub reputation_score: f64,
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub slashed_amount: Decimal,
    pub rewards_earned: Decimal,
    pub last_active: DateTime<Utc>,
    pub status: ValidatorStatus,
    pub supported_chains: Vec<ChainId>,
}

/// Validator status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    Active,
    Inactive,
    Suspended,
    Slashed,
    Jailed,
}

/// Validator service trait
#[async_trait]
pub trait ValidatorService: Send + Sync {
    /// Submit a validation request
    async fn submit_validation(&self, request: ValidationRequest)
        -> BridgeResult<ValidationResult>;

    /// Get validation status
    async fn get_validation_status(
        &self,
        request_id: Uuid,
    ) -> BridgeResult<Option<ValidationResult>>;

    /// Submit validation response (for validators)
    async fn submit_validation_response(&self, response: ValidationResponse) -> BridgeResult<()>;

    /// Challenge a validation result
    async fn challenge_validation(
        &self,
        request_id: Uuid,
        challenger_id: String,
        evidence: Vec<u8>,
    ) -> BridgeResult<ChallengeResult>;

    /// Get validator information
    async fn get_validator(&self, validator_id: &str) -> BridgeResult<Option<BridgeValidator>>;

    /// Get all active validators
    async fn get_active_validators(&self) -> BridgeResult<Vec<BridgeValidator>>;

    /// Register as a validator
    async fn register_validator(&self, validator: BridgeValidator) -> BridgeResult<()>;

    /// Update validator status
    async fn update_validator_status(
        &self,
        validator_id: &str,
        status: ValidatorStatus,
    ) -> BridgeResult<()>;

    /// Slash validator for misbehavior
    async fn slash_validator(
        &self,
        validator_id: &str,
        reason: String,
        amount: Decimal,
    ) -> BridgeResult<()>;

    /// Distribute validation rewards
    async fn distribute_rewards(&self, request_id: Uuid) -> BridgeResult<RewardDistribution>;

    /// Health check
    async fn health_check(&self) -> BridgeResult<ValidatorHealthStatus>;
}

/// Challenge result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub challenge_id: Uuid,
    pub request_id: Uuid,
    pub challenger_id: String,
    pub challenge_accepted: bool,
    pub resolution_deadline: DateTime<Utc>,
    pub stake_at_risk: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Reward distribution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub request_id: Uuid,
    pub total_reward: Decimal,
    pub validator_rewards: HashMap<String, Decimal>,
    pub distributed_at: DateTime<Utc>,
}

/// Validator health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorHealthStatus {
    pub status: String,
    pub active_validators: u64,
    pub total_validators: u64,
    pub total_stake: Decimal,
    pub pending_validations: u64,
    pub completed_validations_24h: u64,
    pub consensus_success_rate: f64,
    pub average_validation_time_seconds: f64,
    pub challenges_pending: u64,
    pub last_check: DateTime<Utc>,
}

/// Validator network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorNetworkStats {
    pub total_validators: u64,
    pub active_validators: u64,
    pub total_stake: Decimal,
    pub average_reputation: f64,
    pub total_validations: u64,
    pub consensus_success_rate: f64,
    pub total_rewards_distributed: Decimal,
    pub total_slashed_amount: Decimal,
    pub chain_coverage: HashMap<ChainId, u64>,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_request_creation() {
        let request = ValidationRequest::new(
            Uuid::new_v4(),
            ChainId::Ethereum,
            ChainId::Polygon,
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            12345,
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 10],
            "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            vec!["0x2222222222222222222222222222222222222222222222222222222222222222".to_string()],
        );

        assert_eq!(request.source_chain, ChainId::Ethereum);
        assert_eq!(request.destination_chain, ChainId::Polygon);
        assert_eq!(request.source_block_number, 12345);
        assert!(!request.merkle_proof.is_empty());
    }

    #[test]
    fn test_validation_request_validation() {
        let valid_request = ValidationRequest::new(
            Uuid::new_v4(),
            ChainId::Ethereum,
            ChainId::Polygon,
            "0x1234567890123456789012345678901234567890123456789012345678901234".to_string(),
            12345,
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            vec![1, 2, 3, 4, 5],
            vec![6, 7, 8, 9, 10],
            "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            vec!["0x2222222222222222222222222222222222222222222222222222222222222222".to_string()],
        );

        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_validator_config_default() {
        let config = ValidatorConfig::default();
        assert!(!config.node_id.is_empty());
        assert_eq!(config.min_validators, 5);
        assert_eq!(config.consensus_threshold, Decimal::new(67, 2));
        assert!(config.fraud_detection_enabled);
    }
}
