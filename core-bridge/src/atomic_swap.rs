// =====================================================================================
// File: core-bridge/src/atomic_swap.rs
// Description: Atomic swap service for trustless cross-chain exchanges
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    error::{BridgeError, BridgeResult},
    types::{ChainId, SwapStatus},
};

/// Atomic swap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicSwapConfig {
    /// Default timelock duration in hours
    pub default_timelock_hours: u32,
    /// Minimum timelock duration in hours
    pub min_timelock_hours: u32,
    /// Maximum timelock duration in hours
    pub max_timelock_hours: u32,
    /// Hash function to use for HTLC
    pub hash_function: HashFunction,
    /// Enable automatic refund
    pub auto_refund: bool,
    /// Refund check interval in minutes
    pub refund_check_interval_minutes: u32,
    /// Maximum swap amount (in USD)
    pub max_swap_amount: Decimal,
    /// Minimum swap amount (in USD)
    pub min_swap_amount: Decimal,
    /// Swap fee percentage
    pub swap_fee_percentage: Decimal,
}

impl Default for AtomicSwapConfig {
    fn default() -> Self {
        Self {
            default_timelock_hours: 24,
            min_timelock_hours: 1,
            max_timelock_hours: 168, // 7 days
            hash_function: HashFunction::Sha256,
            auto_refund: true,
            refund_check_interval_minutes: 30,
            max_swap_amount: Decimal::new(100000000, 2), // $1,000,000
            min_swap_amount: Decimal::new(1000, 2),      // $10
            swap_fee_percentage: Decimal::new(25, 4),    // 0.25%
        }
    }
}

/// Hash function enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashFunction {
    Sha256,
    Sha3,
    Blake2b,
}

/// Atomic swap request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    pub id: Uuid,
    pub initiator_id: String,
    pub participant_id: String,
    pub initiator_chain: ChainId,
    pub participant_chain: ChainId,
    pub initiator_asset: String,
    pub participant_asset: String,
    pub initiator_amount: Decimal,
    pub participant_amount: Decimal,
    pub initiator_address: String,
    pub participant_address: String,
    pub hash_lock: String,
    pub secret: Option<String>,
    pub timelock_duration: Duration,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl SwapRequest {
    /// Create a new atomic swap request
    pub fn new(
        initiator_id: String,
        participant_id: String,
        initiator_chain: ChainId,
        participant_chain: ChainId,
        initiator_asset: String,
        participant_asset: String,
        initiator_amount: Decimal,
        participant_amount: Decimal,
        initiator_address: String,
        participant_address: String,
        timelock_hours: u32,
    ) -> BridgeResult<Self> {
        let secret = generate_secret();
        let hash_lock = generate_hash_lock(&secret);
        let timelock_duration = Duration::hours(timelock_hours as i64);
        let created_at = Utc::now();
        let expires_at = created_at + timelock_duration;

        Ok(Self {
            id: Uuid::new_v4(),
            initiator_id,
            participant_id,
            initiator_chain,
            participant_chain,
            initiator_asset,
            participant_asset,
            initiator_amount,
            participant_amount,
            initiator_address,
            participant_address,
            hash_lock,
            secret: Some(secret),
            timelock_duration,
            created_at,
            expires_at,
        })
    }

    /// Validate the swap request
    pub fn validate(&self, config: &AtomicSwapConfig) -> BridgeResult<()> {
        // Check timelock duration
        let timelock_hours = self.timelock_duration.num_hours() as u32;
        if timelock_hours < config.min_timelock_hours {
            return Err(BridgeError::validation_error(
                "timelock",
                format!(
                    "Timelock {} hours is below minimum {}",
                    timelock_hours, config.min_timelock_hours
                ),
            ));
        }

        if timelock_hours > config.max_timelock_hours {
            return Err(BridgeError::validation_error(
                "timelock",
                format!(
                    "Timelock {} hours exceeds maximum {}",
                    timelock_hours, config.max_timelock_hours
                ),
            ));
        }

        // Check amounts
        if self.initiator_amount <= Decimal::ZERO {
            return Err(BridgeError::validation_error(
                "initiator_amount",
                "Initiator amount must be positive",
            ));
        }

        if self.participant_amount <= Decimal::ZERO {
            return Err(BridgeError::validation_error(
                "participant_amount",
                "Participant amount must be positive",
            ));
        }

        // Check addresses
        if self.initiator_address.is_empty() {
            return Err(BridgeError::validation_error(
                "initiator_address",
                "Initiator address cannot be empty",
            ));
        }

        if self.participant_address.is_empty() {
            return Err(BridgeError::validation_error(
                "participant_address",
                "Participant address cannot be empty",
            ));
        }

        // Check expiration
        if self.expires_at <= Utc::now() {
            return Err(BridgeError::validation_error(
                "expires_at",
                "Swap has already expired",
            ));
        }

        Ok(())
    }

    /// Check if the swap has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get time remaining until expiration
    pub fn time_remaining(&self) -> Option<Duration> {
        let now = Utc::now();
        if now < self.expires_at {
            Some(self.expires_at - now)
        } else {
            None
        }
    }
}

/// Atomic swap contract information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapContract {
    pub swap_id: Uuid,
    pub chain_id: ChainId,
    pub contract_address: String,
    pub hash_lock: String,
    pub amount: Decimal,
    pub recipient: String,
    pub timelock: DateTime<Utc>,
    pub status: SwapStatus,
    pub tx_hash: Option<String>,
    pub block_number: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Atomic swap service trait
#[async_trait]
pub trait AtomicSwapService: Send + Sync {
    /// Initiate an atomic swap
    async fn initiate_swap(&self, request: SwapRequest) -> BridgeResult<SwapContract>;

    /// Participate in an atomic swap
    async fn participate_swap(
        &self,
        swap_id: Uuid,
        participant_contract: SwapContract,
    ) -> BridgeResult<SwapContract>;

    /// Redeem from atomic swap using secret
    async fn redeem_swap(&self, swap_id: Uuid, secret: String) -> BridgeResult<RedeemResult>;

    /// Refund expired atomic swap
    async fn refund_swap(&self, swap_id: Uuid) -> BridgeResult<RefundResult>;

    /// Get swap status
    async fn get_swap_status(&self, swap_id: Uuid) -> BridgeResult<Option<SwapStatus>>;

    /// Get swap details
    async fn get_swap_details(&self, swap_id: Uuid) -> BridgeResult<Option<SwapDetails>>;

    /// Get user's swap history
    async fn get_user_swaps(&self, user_id: &str) -> BridgeResult<Vec<SwapDetails>>;

    /// Verify hash lock and secret
    async fn verify_secret(&self, hash_lock: &str, secret: &str) -> BridgeResult<bool>;

    /// Health check
    async fn health_check(&self) -> BridgeResult<AtomicSwapHealthStatus>;
}

/// Redeem result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemResult {
    pub swap_id: Uuid,
    pub amount_redeemed: Decimal,
    pub secret_revealed: String,
    pub tx_hash: String,
    pub gas_used: u64,
    pub redeemed_at: DateTime<Utc>,
}

/// Refund result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResult {
    pub swap_id: Uuid,
    pub amount_refunded: Decimal,
    pub tx_hash: String,
    pub gas_used: u64,
    pub refunded_at: DateTime<Utc>,
}

/// Complete swap details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapDetails {
    pub request: SwapRequest,
    pub initiator_contract: Option<SwapContract>,
    pub participant_contract: Option<SwapContract>,
    pub status: SwapStatus,
    pub secret_revealed: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

/// Atomic swap health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicSwapHealthStatus {
    pub status: String,
    pub active_swaps: u64,
    pub completed_swaps_24h: u64,
    pub failed_swaps_24h: u64,
    pub expired_swaps_pending_refund: u64,
    pub average_completion_time_minutes: f64,
    pub supported_chains: Vec<ChainId>,
    pub last_check: DateTime<Utc>,
}

/// Generate a random secret for HTLC
fn generate_secret() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let secret: [u8; 32] = rng.gen();
    hex::encode(secret)
}

/// Generate hash lock from secret
fn generate_hash_lock(secret: &str) -> String {
    let secret_bytes = hex::decode(secret).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&secret_bytes);
    let hash = hasher.finalize();
    hex::encode(hash)
}

/// Verify that a secret matches a hash lock
pub fn verify_hash_lock(hash_lock: &str, secret: &str) -> bool {
    let computed_hash = generate_hash_lock(secret);
    computed_hash == hash_lock
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_request_creation() {
        let request = SwapRequest::new(
            "user1".to_string(),
            "user2".to_string(),
            ChainId::Ethereum,
            ChainId::Bitcoin,
            "ETH".to_string(),
            "BTC".to_string(),
            Decimal::new(1, 18), // 1 ETH
            Decimal::new(5, 8),  // 0.05 BTC
            "0x1234567890123456789012345678901234567890".to_string(),
            "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
            24,
        )
        .unwrap();

        assert_eq!(request.initiator_id, "user1");
        assert_eq!(request.participant_id, "user2");
        assert_eq!(request.initiator_chain, ChainId::Ethereum);
        assert_eq!(request.participant_chain, ChainId::Bitcoin);
        assert!(!request.hash_lock.is_empty());
        assert!(request.secret.is_some());
    }

    #[test]
    fn test_hash_lock_verification() {
        let secret = generate_secret();
        let hash_lock = generate_hash_lock(&secret);

        assert!(verify_hash_lock(&hash_lock, &secret));
        assert!(!verify_hash_lock(&hash_lock, "wrong_secret"));
    }

    #[test]
    fn test_swap_expiration() {
        let mut request = SwapRequest::new(
            "user1".to_string(),
            "user2".to_string(),
            ChainId::Ethereum,
            ChainId::Bitcoin,
            "ETH".to_string(),
            "BTC".to_string(),
            Decimal::new(1, 18),
            Decimal::new(5, 8),
            "0x1234567890123456789012345678901234567890".to_string(),
            "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
            24,
        )
        .unwrap();

        assert!(!request.is_expired());
        assert!(request.time_remaining().is_some());

        // Simulate expiration
        request.expires_at = Utc::now() - Duration::hours(1);
        assert!(request.is_expired());
        assert!(request.time_remaining().is_none());
    }
}
