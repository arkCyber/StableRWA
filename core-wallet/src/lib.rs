// =====================================================================================
// File: core-wallet/src/lib.rs
// Description: Multi-signature wallet and key management for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Wallet Module
//! 
//! This module provides comprehensive wallet and key management services for the
//! StableRWA platform, including multi-signature wallets, hardware wallet integration,
//! key derivation, and secure key storage.

pub mod error;
pub mod types;
pub mod multisig;
pub mod hardware;
pub mod key_management;
pub mod recovery;
pub mod timelock;
pub mod gnosis_safe;
pub mod mnemonic;
pub mod encryption;
pub mod derivation;
pub mod service;

// Re-export main types and traits
pub use error::{WalletError, WalletResult};
pub use types::{
    Wallet, MultiSigWallet, HardwareWallet, KeyPair, PrivateKey, PublicKey,
    Address, Signature, Transaction, WalletType, SignatureScheme
};
pub use multisig::{
    MultiSigService, MultiSigConfig, MultiSigTransaction,
    Signer, SigningPolicy, ThresholdPolicy
};
pub use hardware::{
    HardwareWalletService, LedgerWallet, TrezorWallet,
    HardwareDevice, DeviceInfo, DeviceStatus
};
pub use key_management::{
    KeyManager, KeyStore, SecureKeyStore,
    KeyDerivation, KeyRotation, KeyBackup
};
pub use recovery::{
    RecoveryService, SocialRecovery, RecoveryGuardian,
    RecoveryRequest, RecoveryProposal
};
pub use mnemonic::{
    MnemonicGenerator, MnemonicValidator, SeedPhrase,
    WordList, EntropySource
};
pub use encryption::{
    EncryptionService, AESEncryption, ChaChaEncryption,
    EncryptedData, EncryptionKey
};
pub use derivation::{
    HDWallet, DerivationPath, ExtendedKey,
    BIP32Derivation, BIP44Derivation
};
pub use service::WalletService;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main Wallet service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletServiceConfig {
    /// Multi-signature configuration
    pub multisig_config: multisig::MultiSigConfig,
    /// Hardware wallet configuration
    pub hardware_config: hardware::HardwareConfig,
    /// Key management configuration
    pub key_management_config: key_management::KeyManagementConfig,
    /// Recovery configuration
    pub recovery_config: recovery::RecoveryConfig,
    /// Encryption configuration
    pub encryption_config: encryption::EncryptionConfig,
    /// Global wallet settings
    pub global_settings: GlobalWalletSettings,
}

impl Default for WalletServiceConfig {
    fn default() -> Self {
        Self {
            multisig_config: multisig::MultiSigConfig::default(),
            hardware_config: hardware::HardwareConfig::default(),
            key_management_config: key_management::KeyManagementConfig::default(),
            recovery_config: recovery::RecoveryConfig::default(),
            encryption_config: encryption::EncryptionConfig::default(),
            global_settings: GlobalWalletSettings::default(),
        }
    }
}

/// Global wallet settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalWalletSettings {
    /// Default signature scheme
    pub default_signature_scheme: SignatureScheme,
    /// Enable hardware wallet support
    pub enable_hardware_wallets: bool,
    /// Enable social recovery
    pub enable_social_recovery: bool,
    /// Enable timelock transactions
    pub enable_timelock: bool,
    /// Default key derivation path
    pub default_derivation_path: String,
    /// Key rotation interval in days
    pub key_rotation_interval_days: u32,
    /// Backup encryption enabled
    pub enable_backup_encryption: bool,
    /// Transaction signing timeout in minutes
    pub signing_timeout_minutes: u32,
    /// Maximum concurrent signers
    pub max_concurrent_signers: u32,
}

impl Default for GlobalWalletSettings {
    fn default() -> Self {
        Self {
            default_signature_scheme: SignatureScheme::ECDSA,
            enable_hardware_wallets: true,
            enable_social_recovery: true,
            enable_timelock: true,
            default_derivation_path: "m/44'/60'/0'/0".to_string(), // Ethereum BIP44
            key_rotation_interval_days: 90,
            enable_backup_encryption: true,
            signing_timeout_minutes: 30,
            max_concurrent_signers: 10,
        }
    }
}

/// Wallet metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetrics {
    pub total_wallets: u64,
    pub active_wallets: u64,
    pub multisig_wallets: u64,
    pub hardware_wallets: u64,
    pub total_transactions_24h: u64,
    pub successful_transactions_24h: u64,
    pub failed_transactions_24h: u64,
    pub average_signing_time_ms: f64,
    pub key_rotations_24h: u64,
    pub recovery_requests_24h: u64,
    pub wallet_type_breakdown: HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

/// Wallet health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletHealthStatus {
    pub overall_status: String,
    pub multisig_status: String,
    pub hardware_status: String,
    pub key_management_status: String,
    pub recovery_status: String,
    pub encryption_status: String,
    pub device_statuses: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
}

// Stub modules for compilation
pub mod multisig {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MultiSigConfig {
        pub default_threshold: u32,
        pub max_signers: u32,
        pub signing_timeout_minutes: u32,
    }
    
    impl Default for MultiSigConfig {
        fn default() -> Self {
            Self {
                default_threshold: 2,
                max_signers: 10,
                signing_timeout_minutes: 30,
            }
        }
    }
    
    pub struct MultiSigService;
    pub struct MultiSigTransaction;
    pub struct Signer;
    pub struct SigningPolicy;
    pub struct ThresholdPolicy;
}

pub mod hardware {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HardwareConfig {
        pub enable_ledger: bool,
        pub enable_trezor: bool,
        pub connection_timeout_seconds: u32,
    }
    
    impl Default for HardwareConfig {
        fn default() -> Self {
            Self {
                enable_ledger: true,
                enable_trezor: true,
                connection_timeout_seconds: 30,
            }
        }
    }
    
    pub struct HardwareWalletService;
    pub struct LedgerWallet;
    pub struct TrezorWallet;
    pub struct HardwareDevice;
    pub struct DeviceInfo;
    pub struct DeviceStatus;
}

pub mod key_management {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct KeyManagementConfig {
        pub encryption_algorithm: String,
        pub key_derivation_iterations: u32,
        pub backup_enabled: bool,
    }
    
    impl Default for KeyManagementConfig {
        fn default() -> Self {
            Self {
                encryption_algorithm: "AES-256-GCM".to_string(),
                key_derivation_iterations: 100000,
                backup_enabled: true,
            }
        }
    }
    
    pub struct KeyManager;
    pub struct KeyStore;
    pub struct SecureKeyStore;
    pub struct KeyDerivation;
    pub struct KeyRotation;
    pub struct KeyBackup;
}

pub mod recovery {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RecoveryConfig {
        pub min_guardians: u32,
        pub recovery_threshold: u32,
        pub recovery_delay_hours: u32,
    }
    
    impl Default for RecoveryConfig {
        fn default() -> Self {
            Self {
                min_guardians: 3,
                recovery_threshold: 2,
                recovery_delay_hours: 24,
            }
        }
    }
    
    pub struct RecoveryService;
    pub struct SocialRecovery;
    pub struct RecoveryGuardian;
    pub struct RecoveryRequest;
    pub struct RecoveryProposal;
}

pub mod encryption {
    use super::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EncryptionConfig {
        pub default_algorithm: String,
        pub key_size_bits: u32,
        pub enable_hardware_encryption: bool,
    }
    
    impl Default for EncryptionConfig {
        fn default() -> Self {
            Self {
                default_algorithm: "AES-256-GCM".to_string(),
                key_size_bits: 256,
                enable_hardware_encryption: true,
            }
        }
    }
    
    pub struct EncryptionService;
    pub struct AESEncryption;
    pub struct ChaChaEncryption;
    pub struct EncryptedData;
    pub struct EncryptionKey;
}

pub mod mnemonic {
    use super::*;
    
    pub struct MnemonicGenerator;
    pub struct MnemonicValidator;
    pub struct SeedPhrase;
    pub struct WordList;
    pub struct EntropySource;
}

pub mod derivation {
    use super::*;
    
    pub struct HDWallet;
    pub struct DerivationPath;
    pub struct ExtendedKey;
    pub struct BIP32Derivation;
    pub struct BIP44Derivation;
}

pub mod timelock {
    use super::*;
    
    pub struct TimelockService;
    pub struct TimelockTransaction;
    pub struct DelayedExecution;
}

pub mod gnosis_safe {
    use super::*;
    
    pub struct GnosisSafeService;
    pub struct SafeTransaction;
    pub struct SafeConfig;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_config_default() {
        let config = WalletServiceConfig::default();
        assert_eq!(config.multisig_config.default_threshold, 2);
        assert_eq!(config.multisig_config.max_signers, 10);
        assert!(config.hardware_config.enable_ledger);
        assert!(config.hardware_config.enable_trezor);
        assert!(config.global_settings.enable_hardware_wallets);
        assert!(config.global_settings.enable_social_recovery);
    }

    #[test]
    fn test_global_wallet_settings() {
        let settings = GlobalWalletSettings::default();
        assert_eq!(settings.default_signature_scheme, SignatureScheme::ECDSA);
        assert_eq!(settings.default_derivation_path, "m/44'/60'/0'/0");
        assert_eq!(settings.key_rotation_interval_days, 90);
        assert_eq!(settings.signing_timeout_minutes, 30);
        assert_eq!(settings.max_concurrent_signers, 10);
    }

    #[test]
    fn test_multisig_config() {
        let config = multisig::MultiSigConfig::default();
        assert_eq!(config.default_threshold, 2);
        assert_eq!(config.max_signers, 10);
        assert_eq!(config.signing_timeout_minutes, 30);
    }

    #[test]
    fn test_recovery_config() {
        let config = recovery::RecoveryConfig::default();
        assert_eq!(config.min_guardians, 3);
        assert_eq!(config.recovery_threshold, 2);
        assert_eq!(config.recovery_delay_hours, 24);
    }
}
