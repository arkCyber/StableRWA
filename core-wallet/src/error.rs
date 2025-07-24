// =====================================================================================
// File: core-wallet/src/error.rs
// Description: Error types for wallet operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;

/// Result type for wallet operations
pub type WalletResult<T> = Result<T, WalletError>;

/// Comprehensive error types for wallet operations
#[derive(Error, Debug, Clone)]
pub enum WalletError {
    /// Invalid wallet configuration
    #[error("Invalid wallet configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Wallet not found
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    
    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    
    /// Invalid public key
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    
    /// Invalid address format
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    
    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    /// Insufficient signers for multi-sig operation
    #[error("Insufficient signers: required {required}, got {actual}")]
    InsufficientSigners { required: u32, actual: u32 },
    
    /// Signing timeout
    #[error("Signing timeout after {timeout_minutes} minutes")]
    SigningTimeout { timeout_minutes: u32 },
    
    /// Hardware wallet error
    #[error("Hardware wallet error: {0}")]
    HardwareWalletError(String),
    
    /// Device not connected
    #[error("Hardware device not connected: {device_type}")]
    DeviceNotConnected { device_type: String },
    
    /// Device communication error
    #[error("Device communication error: {0}")]
    DeviceCommunicationError(String),
    
    /// Key derivation error
    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),
    
    /// Invalid derivation path
    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),
    
    /// Encryption error
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    /// Decryption error
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    /// Invalid mnemonic phrase
    #[error("Invalid mnemonic phrase: {0}")]
    InvalidMnemonic(String),
    
    /// Key store error
    #[error("Key store error: {0}")]
    KeyStoreError(String),
    
    /// Recovery error
    #[error("Recovery error: {0}")]
    RecoveryError(String),
    
    /// Insufficient guardians for recovery
    #[error("Insufficient guardians: required {required}, available {available}")]
    InsufficientGuardians { required: u32, available: u32 },
    
    /// Recovery request not found
    #[error("Recovery request not found: {0}")]
    RecoveryRequestNotFound(String),
    
    /// Recovery delay not elapsed
    #[error("Recovery delay not elapsed: {remaining_hours} hours remaining")]
    RecoveryDelayNotElapsed { remaining_hours: u32 },
    
    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    /// Timelock error
    #[error("Timelock error: {0}")]
    TimelockError(String),
    
    /// Timelock not expired
    #[error("Timelock not expired: {remaining_seconds} seconds remaining")]
    TimelockNotExpired { remaining_seconds: u64 },
    
    /// Gnosis Safe error
    #[error("Gnosis Safe error: {0}")]
    GnosisSafeError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl WalletError {
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            WalletError::NetworkError(_)
                | WalletError::ServiceUnavailable(_)
                | WalletError::RateLimitExceeded(_)
                | WalletError::SigningTimeout { .. }
                | WalletError::DeviceCommunicationError(_)
        )
    }
    
    /// Check if error is related to hardware wallet
    pub fn is_hardware_error(&self) -> bool {
        matches!(
            self,
            WalletError::HardwareWalletError(_)
                | WalletError::DeviceNotConnected { .. }
                | WalletError::DeviceCommunicationError(_)
        )
    }
    
    /// Check if error is related to multi-signature operations
    pub fn is_multisig_error(&self) -> bool {
        matches!(
            self,
            WalletError::InsufficientSigners { .. }
                | WalletError::SigningTimeout { .. }
        )
    }
    
    /// Check if error is related to recovery operations
    pub fn is_recovery_error(&self) -> bool {
        matches!(
            self,
            WalletError::RecoveryError(_)
                | WalletError::InsufficientGuardians { .. }
                | WalletError::RecoveryRequestNotFound(_)
                | WalletError::RecoveryDelayNotElapsed { .. }
        )
    }
    
    /// Get error category
    pub fn category(&self) -> &'static str {
        match self {
            WalletError::InvalidConfiguration(_) => "configuration",
            WalletError::WalletNotFound(_) => "wallet",
            WalletError::InvalidPrivateKey(_) | WalletError::InvalidPublicKey(_) => "key",
            WalletError::InvalidAddress(_) => "address",
            WalletError::InvalidSignature(_) => "signature",
            WalletError::InsufficientSigners { .. } | WalletError::SigningTimeout { .. } => "multisig",
            WalletError::HardwareWalletError(_) | WalletError::DeviceNotConnected { .. } | WalletError::DeviceCommunicationError(_) => "hardware",
            WalletError::KeyDerivationError(_) | WalletError::InvalidDerivationPath(_) => "derivation",
            WalletError::EncryptionError(_) | WalletError::DecryptionError(_) => "encryption",
            WalletError::InvalidMnemonic(_) => "mnemonic",
            WalletError::KeyStoreError(_) => "keystore",
            WalletError::RecoveryError(_) | WalletError::InsufficientGuardians { .. } | WalletError::RecoveryRequestNotFound(_) | WalletError::RecoveryDelayNotElapsed { .. } => "recovery",
            WalletError::TransactionError(_) => "transaction",
            WalletError::TimelockError(_) | WalletError::TimelockNotExpired { .. } => "timelock",
            WalletError::GnosisSafeError(_) => "gnosis_safe",
            WalletError::NetworkError(_) => "network",
            WalletError::SerializationError(_) => "serialization",
            WalletError::DatabaseError(_) => "database",
            WalletError::PermissionDenied(_) => "permission",
            WalletError::RateLimitExceeded(_) => "rate_limit",
            WalletError::ServiceUnavailable(_) => "service",
            WalletError::InternalError(_) => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let error = WalletError::InvalidConfiguration("test".to_string());
        assert_eq!(error.category(), "configuration");
        assert!(!error.is_recoverable());
        
        let error = WalletError::NetworkError("test".to_string());
        assert_eq!(error.category(), "network");
        assert!(error.is_recoverable());
        
        let error = WalletError::HardwareWalletError("test".to_string());
        assert_eq!(error.category(), "hardware");
        assert!(error.is_hardware_error());
        
        let error = WalletError::InsufficientSigners { required: 3, actual: 2 };
        assert_eq!(error.category(), "multisig");
        assert!(error.is_multisig_error());
        
        let error = WalletError::RecoveryError("test".to_string());
        assert_eq!(error.category(), "recovery");
        assert!(error.is_recovery_error());
    }

    #[test]
    fn test_error_display() {
        let error = WalletError::InsufficientSigners { required: 3, actual: 2 };
        assert_eq!(error.to_string(), "Insufficient signers: required 3, got 2");
        
        let error = WalletError::SigningTimeout { timeout_minutes: 30 };
        assert_eq!(error.to_string(), "Signing timeout after 30 minutes");
        
        let error = WalletError::DeviceNotConnected { device_type: "Ledger".to_string() };
        assert_eq!(error.to_string(), "Hardware device not connected: Ledger");
    }
}
