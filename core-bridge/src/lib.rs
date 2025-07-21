// =====================================================================================
// File: core-bridge/src/lib.rs
// Description: Cross-chain bridge services for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Bridge Module
//! 
//! This module provides comprehensive cross-chain bridge functionality for the
//! StableRWA platform, including multi-chain asset transfers, liquidity aggregation,
//! atomic swaps, and bridge security monitoring.

pub mod transfer;
pub mod liquidity;
pub mod atomic_swap;
pub mod security;
pub mod relayer;
pub mod validator;
pub mod error;
pub mod types;
pub mod service;

// Re-export main types and traits
pub use error::{BridgeError, BridgeResult};
pub use types::{
    BridgeTransaction, BridgeStatus, ChainId, AssetTransfer, LiquidityPool,
    AtomicSwap, SwapStatus, BridgeConfig, SecurityAlert
};
pub use service::BridgeService;
pub use transfer::{TransferService, TransferRequest};
pub use liquidity::{LiquidityService, LiquidityRequest};
pub use atomic_swap::{AtomicSwapService, SwapRequest};
pub use security::{SecurityService, SecurityMonitor};
pub use relayer::{RelayerService, RelayerNode};
pub use validator::{ValidatorService, BridgeValidator};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeServiceConfig {
    /// Supported blockchain networks
    pub supported_chains: Vec<ChainConfig>,
    /// Transfer service configuration
    pub transfer_config: transfer::TransferConfig,
    /// Liquidity service configuration
    pub liquidity_config: liquidity::LiquidityConfig,
    /// Atomic swap configuration
    pub atomic_swap_config: atomic_swap::AtomicSwapConfig,
    /// Security monitoring configuration
    pub security_config: security::SecurityConfig,
    /// Relayer network configuration
    pub relayer_config: relayer::RelayerConfig,
    /// Validator configuration
    pub validator_config: validator::ValidatorConfig,
    /// Global bridge settings
    pub global_settings: GlobalBridgeSettings,
}

impl Default for BridgeServiceConfig {
    fn default() -> Self {
        Self {
            supported_chains: vec![
                ChainConfig::ethereum_mainnet(),
                ChainConfig::polygon_mainnet(),
                ChainConfig::bsc_mainnet(),
                ChainConfig::bitcoin_mainnet(),
            ],
            transfer_config: transfer::TransferConfig::default(),
            liquidity_config: liquidity::LiquidityConfig::default(),
            atomic_swap_config: atomic_swap::AtomicSwapConfig::default(),
            security_config: security::SecurityConfig::default(),
            relayer_config: relayer::RelayerConfig::default(),
            validator_config: validator::ValidatorConfig::default(),
            global_settings: GlobalBridgeSettings::default(),
        }
    }
}

/// Chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: ChainId,
    pub name: String,
    pub network_type: NetworkType,
    pub rpc_endpoints: Vec<String>,
    pub explorer_url: String,
    pub native_token: TokenConfig,
    pub supported_tokens: Vec<TokenConfig>,
    pub bridge_contract: Option<String>,
    pub confirmation_blocks: u64,
    pub gas_settings: GasSettings,
    pub enabled: bool,
}

impl ChainConfig {
    /// Create Ethereum mainnet configuration
    pub fn ethereum_mainnet() -> Self {
        Self {
            chain_id: ChainId::Ethereum,
            name: "Ethereum Mainnet".to_string(),
            network_type: NetworkType::Mainnet,
            rpc_endpoints: vec![
                "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
                "https://eth-mainnet.alchemyapi.io/v2/YOUR_API_KEY".to_string(),
            ],
            explorer_url: "https://etherscan.io".to_string(),
            native_token: TokenConfig {
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                decimals: 18,
                contract_address: None,
            },
            supported_tokens: vec![
                TokenConfig {
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    decimals: 6,
                    contract_address: Some("0xA0b86a33E6441b8C4505B8C4505B8C4505B8C450".to_string()),
                },
                TokenConfig {
                    symbol: "USDT".to_string(),
                    name: "Tether USD".to_string(),
                    decimals: 6,
                    contract_address: Some("0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()),
                },
            ],
            bridge_contract: Some("0x1234567890123456789012345678901234567890".to_string()),
            confirmation_blocks: 12,
            gas_settings: GasSettings {
                gas_limit: 200000,
                max_gas_price: Decimal::new(100, 9), // 100 Gwei
                priority_fee: Decimal::new(2, 9),    // 2 Gwei
            },
            enabled: true,
        }
    }

    /// Create Polygon mainnet configuration
    pub fn polygon_mainnet() -> Self {
        Self {
            chain_id: ChainId::Polygon,
            name: "Polygon Mainnet".to_string(),
            network_type: NetworkType::Mainnet,
            rpc_endpoints: vec![
                "https://polygon-rpc.com".to_string(),
                "https://rpc-mainnet.matic.network".to_string(),
            ],
            explorer_url: "https://polygonscan.com".to_string(),
            native_token: TokenConfig {
                symbol: "MATIC".to_string(),
                name: "Polygon".to_string(),
                decimals: 18,
                contract_address: None,
            },
            supported_tokens: vec![
                TokenConfig {
                    symbol: "USDC".to_string(),
                    name: "USD Coin".to_string(),
                    decimals: 6,
                    contract_address: Some("0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string()),
                },
            ],
            bridge_contract: Some("0x2345678901234567890123456789012345678901".to_string()),
            confirmation_blocks: 20,
            gas_settings: GasSettings {
                gas_limit: 150000,
                max_gas_price: Decimal::new(50, 9), // 50 Gwei
                priority_fee: Decimal::new(1, 9),   // 1 Gwei
            },
            enabled: true,
        }
    }

    /// Create BSC mainnet configuration
    pub fn bsc_mainnet() -> Self {
        Self {
            chain_id: ChainId::BSC,
            name: "Binance Smart Chain".to_string(),
            network_type: NetworkType::Mainnet,
            rpc_endpoints: vec![
                "https://bsc-dataseed.binance.org".to_string(),
                "https://bsc-dataseed1.defibit.io".to_string(),
            ],
            explorer_url: "https://bscscan.com".to_string(),
            native_token: TokenConfig {
                symbol: "BNB".to_string(),
                name: "Binance Coin".to_string(),
                decimals: 18,
                contract_address: None,
            },
            supported_tokens: vec![
                TokenConfig {
                    symbol: "USDT".to_string(),
                    name: "Tether USD".to_string(),
                    decimals: 18,
                    contract_address: Some("0x55d398326f99059fF775485246999027B3197955".to_string()),
                },
            ],
            bridge_contract: Some("0x3456789012345678901234567890123456789012".to_string()),
            confirmation_blocks: 15,
            gas_settings: GasSettings {
                gas_limit: 100000,
                max_gas_price: Decimal::new(20, 9), // 20 Gwei
                priority_fee: Decimal::new(1, 9),   // 1 Gwei
            },
            enabled: true,
        }
    }

    /// Create Bitcoin mainnet configuration
    pub fn bitcoin_mainnet() -> Self {
        Self {
            chain_id: ChainId::Bitcoin,
            name: "Bitcoin Mainnet".to_string(),
            network_type: NetworkType::Mainnet,
            rpc_endpoints: vec![
                "https://bitcoin-rpc.example.com".to_string(),
            ],
            explorer_url: "https://blockstream.info".to_string(),
            native_token: TokenConfig {
                symbol: "BTC".to_string(),
                name: "Bitcoin".to_string(),
                decimals: 8,
                contract_address: None,
            },
            supported_tokens: vec![], // Bitcoin doesn't support tokens natively
            bridge_contract: None, // Bitcoin uses different bridge mechanisms
            confirmation_blocks: 6,
            gas_settings: GasSettings {
                gas_limit: 0, // Not applicable for Bitcoin
                max_gas_price: Decimal::new(50, 0), // 50 sat/byte
                priority_fee: Decimal::new(10, 0),  // 10 sat/byte
            },
            enabled: true,
        }
    }
}

/// Network type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub contract_address: Option<String>,
}

/// Gas settings for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasSettings {
    pub gas_limit: u64,
    pub max_gas_price: Decimal,
    pub priority_fee: Decimal,
}

/// Global bridge settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalBridgeSettings {
    /// Minimum transfer amount (in USD)
    pub min_transfer_amount: Decimal,
    /// Maximum transfer amount (in USD)
    pub max_transfer_amount: Decimal,
    /// Bridge fee percentage
    pub bridge_fee_percentage: Decimal,
    /// Maximum slippage tolerance
    pub max_slippage: Decimal,
    /// Transfer timeout in seconds
    pub transfer_timeout_seconds: u64,
    /// Enable emergency pause
    pub emergency_pause_enabled: bool,
    /// Rate limiting settings
    pub rate_limits: RateLimitSettings,
}

impl Default for GlobalBridgeSettings {
    fn default() -> Self {
        Self {
            min_transfer_amount: Decimal::new(1000, 2), // $10.00
            max_transfer_amount: Decimal::new(100000000, 2), // $1,000,000.00
            bridge_fee_percentage: Decimal::new(30, 4), // 0.30%
            max_slippage: Decimal::new(500, 4), // 5.00%
            transfer_timeout_seconds: 3600, // 1 hour
            emergency_pause_enabled: true,
            rate_limits: RateLimitSettings::default(),
        }
    }
}

/// Rate limiting settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSettings {
    /// Maximum transfers per user per hour
    pub max_transfers_per_user_per_hour: u32,
    /// Maximum transfer volume per user per day (in USD)
    pub max_volume_per_user_per_day: Decimal,
    /// Maximum total bridge volume per hour (in USD)
    pub max_total_volume_per_hour: Decimal,
}

impl Default for RateLimitSettings {
    fn default() -> Self {
        Self {
            max_transfers_per_user_per_hour: 10,
            max_volume_per_user_per_day: Decimal::new(5000000, 2), // $50,000
            max_total_volume_per_hour: Decimal::new(1000000000, 2), // $10,000,000
        }
    }
}

/// Bridge operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeOperationResult {
    pub operation_id: Uuid,
    pub operation_type: BridgeOperationType,
    pub status: BridgeStatus,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub asset_symbol: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub user_address: String,
    pub destination_address: String,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Bridge operation type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeOperationType {
    Transfer,
    LiquidityProvision,
    LiquidityWithdrawal,
    AtomicSwap,
    EmergencyWithdrawal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_service_config_default() {
        let config = BridgeServiceConfig::default();
        assert!(!config.supported_chains.is_empty());
        assert!(config.global_settings.emergency_pause_enabled);
        assert_eq!(config.global_settings.bridge_fee_percentage, Decimal::new(30, 4));
    }

    #[test]
    fn test_chain_config_ethereum() {
        let eth_config = ChainConfig::ethereum_mainnet();
        assert_eq!(eth_config.chain_id, ChainId::Ethereum);
        assert_eq!(eth_config.native_token.symbol, "ETH");
        assert_eq!(eth_config.confirmation_blocks, 12);
        assert!(eth_config.enabled);
    }

    #[test]
    fn test_chain_config_polygon() {
        let polygon_config = ChainConfig::polygon_mainnet();
        assert_eq!(polygon_config.chain_id, ChainId::Polygon);
        assert_eq!(polygon_config.native_token.symbol, "MATIC");
        assert_eq!(polygon_config.confirmation_blocks, 20);
    }

    #[test]
    fn test_chain_config_bsc() {
        let bsc_config = ChainConfig::bsc_mainnet();
        assert_eq!(bsc_config.chain_id, ChainId::BSC);
        assert_eq!(bsc_config.native_token.symbol, "BNB");
        assert_eq!(bsc_config.confirmation_blocks, 15);
    }

    #[test]
    fn test_chain_config_bitcoin() {
        let btc_config = ChainConfig::bitcoin_mainnet();
        assert_eq!(btc_config.chain_id, ChainId::Bitcoin);
        assert_eq!(btc_config.native_token.symbol, "BTC");
        assert_eq!(btc_config.confirmation_blocks, 6);
        assert!(btc_config.bridge_contract.is_none());
    }

    #[test]
    fn test_global_bridge_settings_default() {
        let settings = GlobalBridgeSettings::default();
        assert_eq!(settings.min_transfer_amount, Decimal::new(1000, 2));
        assert_eq!(settings.max_transfer_amount, Decimal::new(100000000, 2));
        assert_eq!(settings.bridge_fee_percentage, Decimal::new(30, 4));
        assert_eq!(settings.max_slippage, Decimal::new(500, 4));
        assert!(settings.emergency_pause_enabled);
    }

    #[test]
    fn test_rate_limit_settings_default() {
        let rate_limits = RateLimitSettings::default();
        assert_eq!(rate_limits.max_transfers_per_user_per_hour, 10);
        assert_eq!(rate_limits.max_volume_per_user_per_day, Decimal::new(5000000, 2));
        assert_eq!(rate_limits.max_total_volume_per_hour, Decimal::new(1000000000, 2));
    }

    #[test]
    fn test_bridge_operation_result_creation() {
        let result = BridgeOperationResult {
            operation_id: Uuid::new_v4(),
            operation_type: BridgeOperationType::Transfer,
            status: BridgeStatus::Pending,
            source_chain: ChainId::Ethereum,
            destination_chain: ChainId::Polygon,
            asset_symbol: "USDC".to_string(),
            amount: Decimal::new(100000, 6), // 100 USDC
            fee: Decimal::new(300, 6), // 0.3 USDC
            user_address: "0x1234567890123456789012345678901234567890".to_string(),
            destination_address: "0x0987654321098765432109876543210987654321".to_string(),
            source_tx_hash: None,
            destination_tx_hash: None,
            created_at: Utc::now(),
            completed_at: None,
            error_message: None,
        };

        assert_eq!(result.operation_type, BridgeOperationType::Transfer);
        assert_eq!(result.status, BridgeStatus::Pending);
        assert_eq!(result.source_chain, ChainId::Ethereum);
        assert_eq!(result.destination_chain, ChainId::Polygon);
    }

    #[test]
    fn test_token_config_creation() {
        let usdc_config = TokenConfig {
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            contract_address: Some("0xA0b86a33E6441b8C4505B8C4505B8C4505B8C450".to_string()),
        };

        assert_eq!(usdc_config.symbol, "USDC");
        assert_eq!(usdc_config.decimals, 6);
        assert!(usdc_config.contract_address.is_some());
    }

    #[test]
    fn test_gas_settings_creation() {
        let gas_settings = GasSettings {
            gas_limit: 200000,
            max_gas_price: Decimal::new(100, 9), // 100 Gwei
            priority_fee: Decimal::new(2, 9),    // 2 Gwei
        };

        assert_eq!(gas_settings.gas_limit, 200000);
        assert_eq!(gas_settings.max_gas_price, Decimal::new(100, 9));
        assert_eq!(gas_settings.priority_fee, Decimal::new(2, 9));
    }
}
