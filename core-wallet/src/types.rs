// =====================================================================================
// File: core-wallet/src/types.rs
// Description: Core types for wallet operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Wallet types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalletType {
    /// Single signature wallet
    SingleSig,
    /// Multi-signature wallet
    MultiSig,
    /// Hardware wallet
    Hardware,
    /// Smart contract wallet
    SmartContract,
    /// Gnosis Safe wallet
    GnosisSafe,
    /// Timelock wallet
    Timelock,
}

/// Signature schemes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignatureScheme {
    /// Elliptic Curve Digital Signature Algorithm
    ECDSA,
    /// Edwards-curve Digital Signature Algorithm
    EdDSA,
    /// Schnorr signatures
    Schnorr,
    /// BLS signatures
    BLS,
}

/// Wallet structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub name: String,
    pub wallet_type: WalletType,
    pub address: Address,
    pub public_key: Option<PublicKey>,
    pub signature_scheme: SignatureScheme,
    pub derivation_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Multi-signature wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigWallet {
    pub id: Uuid,
    pub name: String,
    pub address: Address,
    pub threshold: u32,
    pub signers: Vec<Signer>,
    pub pending_transactions: Vec<MultiSigTransaction>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Hardware wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareWallet {
    pub id: Uuid,
    pub device_type: String,
    pub device_id: String,
    pub firmware_version: String,
    pub supported_coins: Vec<String>,
    pub derivation_paths: Vec<String>,
    pub status: DeviceStatus,
    pub last_connected: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Signer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signer {
    pub id: Uuid,
    pub address: Address,
    pub public_key: PublicKey,
    pub name: Option<String>,
    pub weight: u32,
    pub added_at: DateTime<Utc>,
    pub last_signed: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Multi-signature transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub transaction: Transaction,
    pub signatures: Vec<Signature>,
    pub required_signatures: u32,
    pub status: TransactionStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Pending signatures
    Pending,
    /// Ready for execution
    Ready,
    /// Executed successfully
    Executed,
    /// Failed execution
    Failed,
    /// Cancelled
    Cancelled,
    /// Expired
    Expired,
}

/// Device status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// Device is connected and ready
    Connected,
    /// Device is disconnected
    Disconnected,
    /// Device is locked
    Locked,
    /// Device has an error
    Error(String),
    /// Device is updating firmware
    Updating,
}

/// Key pair structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub private_key: PrivateKey,
    pub public_key: PublicKey,
    pub address: Address,
    pub signature_scheme: SignatureScheme,
    pub created_at: DateTime<Utc>,
}

/// Private key (encrypted in storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateKey {
    pub key_data: Vec<u8>, // Encrypted key data
    pub encryption_method: String,
    pub key_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    pub key_data: Vec<u8>,
    pub key_format: String,
    pub signature_scheme: SignatureScheme,
    pub key_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Address representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
    pub address_type: AddressType,
    pub checksum: Option<String>,
}

/// Address types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AddressType {
    /// Ethereum-style address
    Ethereum,
    /// Bitcoin-style address
    Bitcoin,
    /// Solana-style address
    Solana,
    /// Generic address
    Generic,
}

/// Digital signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signature_data: Vec<u8>,
    pub signature_scheme: SignatureScheme,
    pub signer_address: Address,
    pub message_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub from: Address,
    pub to: Address,
    pub value: Decimal,
    pub data: Vec<u8>,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<Decimal>,
    pub nonce: Option<u64>,
    pub chain_id: Option<u64>,
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Transaction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Simple transfer
    Transfer,
    /// Contract call
    ContractCall,
    /// Contract deployment
    ContractDeployment,
    /// Multi-signature transaction
    MultiSig,
    /// Timelock transaction
    Timelock,
    /// Recovery transaction
    Recovery,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: String,
    pub manufacturer: String,
    pub model: String,
    pub firmware_version: String,
    pub serial_number: Option<String>,
    pub supported_features: Vec<String>,
    pub connection_type: ConnectionType,
    pub last_seen: DateTime<Utc>,
}

/// Connection types for hardware devices
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// USB connection
    USB,
    /// Bluetooth connection
    Bluetooth,
    /// NFC connection
    NFC,
    /// Network connection
    Network,
}

impl Address {
    /// Create a new address
    pub fn new(address: String, address_type: AddressType) -> Self {
        let checksum = match address_type {
            AddressType::Ethereum => Some(Self::ethereum_checksum(&address)),
            _ => None,
        };
        
        Self {
            address,
            address_type,
            checksum,
        }
    }
    
    /// Validate address format
    pub fn is_valid(&self) -> bool {
        match self.address_type {
            AddressType::Ethereum => self.is_valid_ethereum(),
            AddressType::Bitcoin => self.is_valid_bitcoin(),
            AddressType::Solana => self.is_valid_solana(),
            AddressType::Generic => !self.address.is_empty(),
        }
    }
    
    fn is_valid_ethereum(&self) -> bool {
        self.address.len() == 42 && self.address.starts_with("0x")
    }
    
    fn is_valid_bitcoin(&self) -> bool {
        // Simplified Bitcoin address validation
        (self.address.len() >= 26 && self.address.len() <= 35) &&
        (self.address.starts_with('1') || self.address.starts_with('3') || self.address.starts_with("bc1"))
    }
    
    fn is_valid_solana(&self) -> bool {
        // Simplified Solana address validation
        self.address.len() >= 32 && self.address.len() <= 44
    }
    
    fn ethereum_checksum(address: &str) -> String {
        // Simplified checksum calculation
        format!("checksum_{}", address)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_creation() {
        let eth_address = Address::new("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b".to_string(), AddressType::Ethereum);
        assert_eq!(eth_address.address_type, AddressType::Ethereum);
        assert!(eth_address.checksum.is_some());
        
        let btc_address = Address::new("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(), AddressType::Bitcoin);
        assert_eq!(btc_address.address_type, AddressType::Bitcoin);
        assert!(btc_address.checksum.is_none());
    }

    #[test]
    fn test_address_validation() {
        let valid_eth = Address::new("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b".to_string(), AddressType::Ethereum);
        assert!(valid_eth.is_valid());
        
        let invalid_eth = Address::new("invalid".to_string(), AddressType::Ethereum);
        assert!(!invalid_eth.is_valid());
    }

    #[test]
    fn test_wallet_types() {
        assert_eq!(WalletType::SingleSig, WalletType::SingleSig);
        assert_ne!(WalletType::SingleSig, WalletType::MultiSig);
    }

    #[test]
    fn test_signature_schemes() {
        assert_eq!(SignatureScheme::ECDSA, SignatureScheme::ECDSA);
        assert_ne!(SignatureScheme::ECDSA, SignatureScheme::EdDSA);
    }

    #[test]
    fn test_transaction_status() {
        assert_eq!(TransactionStatus::Pending, TransactionStatus::Pending);
        assert_ne!(TransactionStatus::Pending, TransactionStatus::Executed);
    }
}
