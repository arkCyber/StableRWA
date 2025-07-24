// =====================================================================================
// File: core-bridge/src/types.rs
// Description: Core types for cross-chain bridge
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Supported blockchain chain IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainId {
    /// Ethereum Mainnet
    Ethereum = 1,
    /// Polygon Mainnet
    Polygon = 137,
    /// Binance Smart Chain
    BSC = 56,
    /// Avalanche C-Chain
    Avalanche = 43114,
    /// Arbitrum One
    Arbitrum = 42161,
    /// Optimism
    Optimism = 10,
    /// Fantom Opera
    Fantom = 250,
    /// Bitcoin Mainnet
    Bitcoin = 0,
    /// Solana Mainnet
    Solana = 101,
    /// Cosmos Hub
    Cosmos = 1000,
    /// Polkadot
    Polkadot = 2000,
}

impl ChainId {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ChainId::Ethereum => "Ethereum",
            ChainId::Polygon => "Polygon",
            ChainId::BSC => "Binance Smart Chain",
            ChainId::Avalanche => "Avalanche",
            ChainId::Arbitrum => "Arbitrum",
            ChainId::Optimism => "Optimism",
            ChainId::Fantom => "Fantom",
            ChainId::Bitcoin => "Bitcoin",
            ChainId::Solana => "Solana",
            ChainId::Cosmos => "Cosmos",
            ChainId::Polkadot => "Polkadot",
        }
    }

    /// Get chain type
    pub fn chain_type(&self) -> ChainType {
        match self {
            ChainId::Ethereum
            | ChainId::Polygon
            | ChainId::BSC
            | ChainId::Avalanche
            | ChainId::Arbitrum
            | ChainId::Optimism
            | ChainId::Fantom => ChainType::EVM,
            ChainId::Bitcoin => ChainType::UTXO,
            ChainId::Solana => ChainType::Solana,
            ChainId::Cosmos => ChainType::Cosmos,
            ChainId::Polkadot => ChainType::Substrate,
        }
    }

    /// Check if chain supports smart contracts
    pub fn supports_smart_contracts(&self) -> bool {
        matches!(
            self,
            ChainId::Ethereum
                | ChainId::Polygon
                | ChainId::BSC
                | ChainId::Avalanche
                | ChainId::Arbitrum
                | ChainId::Optimism
                | ChainId::Fantom
                | ChainId::Solana
        )
    }
}

/// Chain type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    /// Ethereum Virtual Machine compatible chains
    EVM,
    /// Bitcoin-like UTXO chains
    UTXO,
    /// Solana blockchain
    Solana,
    /// Cosmos SDK chains
    Cosmos,
    /// Substrate-based chains
    Substrate,
}

/// Bridge transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeStatus {
    /// Transaction initiated
    Initiated,
    /// Waiting for confirmations on source chain
    Pending,
    /// Confirmed on source chain, processing
    Confirmed,
    /// Being validated by bridge validators
    Validating,
    /// Executing on destination chain
    Executing,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled by user or system
    Cancelled,
    /// Refunded due to failure
    Refunded,
    /// Expired due to timeout
    Expired,
}

impl BridgeStatus {
    /// Check if status is final (no further changes expected)
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            BridgeStatus::Completed
                | BridgeStatus::Failed
                | BridgeStatus::Cancelled
                | BridgeStatus::Refunded
                | BridgeStatus::Expired
        )
    }

    /// Check if status indicates success
    pub fn is_successful(&self) -> bool {
        matches!(self, BridgeStatus::Completed)
    }

    /// Check if status indicates failure
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            BridgeStatus::Failed | BridgeStatus::Cancelled | BridgeStatus::Expired
        )
    }
}

/// Bridge transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: Uuid,
    pub user_id: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub asset_transfer: AssetTransfer,
    pub status: BridgeStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub bridge_fee: Decimal,
    pub gas_fee: Option<Decimal>,
    pub exchange_rate: Option<Decimal>,
    pub slippage: Option<Decimal>,
    pub confirmations: u64,
    pub required_confirmations: u64,
    pub validator_signatures: Vec<ValidatorSignature>,
    pub proof_data: Option<BridgeProof>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl BridgeTransaction {
    /// Create a new bridge transaction
    pub fn new(
        user_id: String,
        source_chain: ChainId,
        destination_chain: ChainId,
        asset_transfer: AssetTransfer,
        required_confirmations: u64,
        timeout_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            source_chain,
            destination_chain,
            asset_transfer,
            status: BridgeStatus::Initiated,
            source_tx_hash: None,
            destination_tx_hash: None,
            bridge_fee: Decimal::ZERO,
            gas_fee: None,
            exchange_rate: None,
            slippage: None,
            confirmations: 0,
            required_confirmations,
            validator_signatures: Vec::new(),
            proof_data: None,
            created_at: now,
            updated_at: now,
            expires_at: now + chrono::Duration::seconds(timeout_seconds as i64),
            metadata: HashMap::new(),
        }
    }

    /// Check if transaction has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if transaction has enough confirmations
    pub fn has_enough_confirmations(&self) -> bool {
        self.confirmations >= self.required_confirmations
    }

    /// Check if transaction has enough validator signatures
    pub fn has_enough_signatures(&self, required_signatures: usize) -> bool {
        self.validator_signatures.len() >= required_signatures
    }
}

/// Asset transfer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransfer {
    pub token_symbol: String,
    pub token_address: Option<String>,
    pub amount: Decimal,
    pub decimals: u8,
    pub source_address: String,
    pub destination_address: String,
    pub memo: Option<String>,
}

/// Validator signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    pub validator_id: String,
    pub signature: String,
    pub public_key: String,
    pub signed_at: DateTime<Utc>,
}

/// Bridge proof data for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeProof {
    pub proof_type: ProofType,
    pub merkle_root: String,
    pub merkle_proof: Vec<String>,
    pub block_hash: String,
    pub block_number: u64,
    pub transaction_index: u32,
    pub receipt_proof: String,
}

/// Proof type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofType {
    MerkleProof,
    SPVProof,
    StateProof,
    ReceiptProof,
}

/// Liquidity pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub id: Uuid,
    pub pool_name: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub token_a: TokenInfo,
    pub token_b: TokenInfo,
    pub reserve_a: Decimal,
    pub reserve_b: Decimal,
    pub total_liquidity: Decimal,
    pub fee_rate: Decimal,
    pub providers: Vec<LiquidityProvider>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub active: bool,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub contract_address: Option<String>,
    pub chain_id: ChainId,
}

/// Liquidity provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityProvider {
    pub provider_id: String,
    pub liquidity_amount: Decimal,
    pub share_percentage: Decimal,
    pub provided_at: DateTime<Utc>,
    pub rewards_earned: Decimal,
}

/// Atomic swap structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicSwap {
    pub id: Uuid,
    pub initiator: String,
    pub participant: String,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub source_asset: AssetInfo,
    pub destination_asset: AssetInfo,
    pub hash_lock: String,
    pub time_lock: DateTime<Utc>,
    pub secret: Option<String>,
    pub status: SwapStatus,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub refund_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset information for swaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub token_symbol: String,
    pub amount: Decimal,
    pub address: String,
    pub contract_address: Option<String>,
}

/// Atomic swap status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapStatus {
    /// Swap initiated by initiator
    Initiated,
    /// Participant has locked funds
    Participated,
    /// Secret revealed, swap can be completed
    SecretRevealed,
    /// Swap completed successfully
    Completed,
    /// Swap refunded due to timeout
    Refunded,
    /// Swap cancelled
    Cancelled,
    /// Swap expired
    Expired,
}

/// Security alert for bridge monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub chain_id: Option<ChainId>,
    pub transaction_id: Option<Uuid>,
    pub description: String,
    pub details: serde_json::Value,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub actions_taken: Vec<String>,
}

/// Alert type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    /// Unusual transaction pattern
    SuspiciousActivity,
    /// Large transaction detected
    LargeTransaction,
    /// Multiple failed transactions
    FailedTransactions,
    /// Validator misbehavior
    ValidatorMisbehavior,
    /// Smart contract anomaly
    ContractAnomaly,
    /// Rate limit exceeded
    RateLimitExceeded,
    /// System performance issue
    PerformanceIssue,
    /// Security breach attempt
    SecurityBreach,
}

/// Alert severity enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info = 1,
    Warning = 2,
    High = 3,
    Critical = 4,
    Emergency = 5,
}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub bridge_id: String,
    pub name: String,
    pub supported_chains: Vec<ChainId>,
    pub supported_tokens: HashMap<ChainId, Vec<String>>,
    pub fee_structure: FeeStructure,
    pub security_settings: SecuritySettings,
    pub operational_limits: OperationalLimits,
    pub emergency_controls: EmergencyControls,
}

/// Fee structure for bridge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    pub base_fee: Decimal,
    pub percentage_fee: Decimal,
    pub minimum_fee: Decimal,
    pub maximum_fee: Decimal,
    pub gas_fee_multiplier: Decimal,
    pub fee_token: String,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub required_validator_signatures: usize,
    pub confirmation_blocks: HashMap<ChainId, u64>,
    pub timeout_seconds: u64,
    pub max_transaction_amount: Decimal,
    pub daily_volume_limit: Decimal,
    pub enable_whitelist: bool,
    pub whitelisted_addresses: Vec<String>,
}

/// Operational limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalLimits {
    pub min_transfer_amount: Decimal,
    pub max_transfer_amount: Decimal,
    pub daily_transfer_limit: Decimal,
    pub monthly_transfer_limit: Decimal,
    pub max_pending_transactions: u32,
    pub rate_limit_per_user: u32,
    pub rate_limit_window_seconds: u64,
}

/// Emergency controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyControls {
    pub pause_enabled: bool,
    pub emergency_pause: bool,
    pub authorized_pausers: Vec<String>,
    pub auto_pause_triggers: Vec<AutoPauseTrigger>,
    pub recovery_procedures: Vec<String>,
}

/// Auto pause trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPauseTrigger {
    pub trigger_type: TriggerType,
    pub threshold: Decimal,
    pub time_window_seconds: u64,
    pub enabled: bool,
}

/// Trigger type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    FailureRate,
    VolumeSpike,
    LargeTransaction,
    ValidatorFailure,
    SecurityAlert,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_properties() {
        assert_eq!(ChainId::Ethereum.name(), "Ethereum");
        assert_eq!(ChainId::Ethereum.chain_type(), ChainType::EVM);
        assert!(ChainId::Ethereum.supports_smart_contracts());

        assert_eq!(ChainId::Bitcoin.name(), "Bitcoin");
        assert_eq!(ChainId::Bitcoin.chain_type(), ChainType::UTXO);
        assert!(!ChainId::Bitcoin.supports_smart_contracts());
    }

    #[test]
    fn test_bridge_status_properties() {
        assert!(BridgeStatus::Completed.is_final());
        assert!(BridgeStatus::Completed.is_successful());
        assert!(!BridgeStatus::Completed.is_failed());

        assert!(BridgeStatus::Failed.is_final());
        assert!(!BridgeStatus::Failed.is_successful());
        assert!(BridgeStatus::Failed.is_failed());

        assert!(!BridgeStatus::Pending.is_final());
        assert!(!BridgeStatus::Pending.is_successful());
        assert!(!BridgeStatus::Pending.is_failed());
    }

    #[test]
    fn test_bridge_transaction_creation() {
        let asset_transfer = AssetTransfer {
            token_symbol: "USDC".to_string(),
            token_address: Some("0xA0b86a33E6441b8C4505B8C4505B8C4505B8C450".to_string()),
            amount: Decimal::new(100000000, 6), // 100 USDC
            decimals: 6,
            source_address: "0x1234567890123456789012345678901234567890".to_string(),
            destination_address: "0x0987654321098765432109876543210987654321".to_string(),
            memo: None,
        };

        let tx = BridgeTransaction::new(
            "user123".to_string(),
            ChainId::Ethereum,
            ChainId::Polygon,
            asset_transfer,
            12,
            3600,
        );

        assert_eq!(tx.status, BridgeStatus::Initiated);
        assert_eq!(tx.source_chain, ChainId::Ethereum);
        assert_eq!(tx.destination_chain, ChainId::Polygon);
        assert_eq!(tx.required_confirmations, 12);
        assert!(!tx.is_expired());
        assert!(!tx.has_enough_confirmations());
    }

    #[test]
    fn test_atomic_swap_creation() {
        let swap = AtomicSwap {
            id: Uuid::new_v4(),
            initiator: "user1".to_string(),
            participant: "user2".to_string(),
            source_chain: ChainId::Ethereum,
            destination_chain: ChainId::Bitcoin,
            source_asset: AssetInfo {
                token_symbol: "ETH".to_string(),
                amount: Decimal::new(1000000000000000000u64, 0), // 1 ETH
                address: "0x1234567890123456789012345678901234567890".to_string(),
                contract_address: None,
            },
            destination_asset: AssetInfo {
                token_symbol: "BTC".to_string(),
                amount: Decimal::new(100000000, 0), // 1 BTC
                address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(),
                contract_address: None,
            },
            hash_lock: "0xabcdef1234567890".to_string(),
            time_lock: Utc::now() + chrono::Duration::hours(24),
            secret: None,
            status: SwapStatus::Initiated,
            source_tx_hash: None,
            destination_tx_hash: None,
            refund_tx_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(swap.status, SwapStatus::Initiated);
        assert_eq!(swap.source_chain, ChainId::Ethereum);
        assert_eq!(swap.destination_chain, ChainId::Bitcoin);
    }

    #[test]
    fn test_security_alert_creation() {
        let alert = SecurityAlert {
            id: Uuid::new_v4(),
            alert_type: AlertType::SuspiciousActivity,
            severity: AlertSeverity::High,
            chain_id: Some(ChainId::Ethereum),
            transaction_id: Some(Uuid::new_v4()),
            description: "Unusual transaction pattern detected".to_string(),
            details: serde_json::json!({"pattern": "multiple_large_transfers"}),
            triggered_at: Utc::now(),
            resolved_at: None,
            resolved: false,
            actions_taken: vec![],
        };

        assert_eq!(alert.alert_type, AlertType::SuspiciousActivity);
        assert_eq!(alert.severity, AlertSeverity::High);
        assert!(!alert.resolved);
    }

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Info < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::High);
        assert!(AlertSeverity::High < AlertSeverity::Critical);
        assert!(AlertSeverity::Critical < AlertSeverity::Emergency);
    }
}
