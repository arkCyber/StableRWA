// =====================================================================================
// File: core-wallet/src/service.rs
// Description: Main wallet service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{WalletError, WalletResult};
use crate::types::{Wallet, MultiSigWallet, HardwareWallet, KeyPair, Transaction, Signature, Address, SignatureScheme};
use crate::multisig::{MultiSigService, MultiSigServiceImpl, CreateWalletRequest};
use crate::hardware::{HardwareWalletService, HardwareWalletServiceImpl};
use crate::key_management::{KeyManager, KeyManagerImpl, SecureKeyStore};
use crate::recovery::{RecoveryService, RecoveryServiceImpl, RecoveryGuardian};
use crate::{WalletServiceConfig, WalletMetrics, WalletHealthStatus};

/// Main wallet service trait
#[async_trait]
pub trait WalletService: Send + Sync {
    /// Create a new single-signature wallet
    async fn create_wallet(&self, name: String, signature_scheme: SignatureScheme) -> WalletResult<Wallet>;
    
    /// Create a new multi-signature wallet
    async fn create_multisig_wallet(&self, request: CreateWalletRequest) -> WalletResult<MultiSigWallet>;
    
    /// Import wallet from private key
    async fn import_wallet(&self, name: String, private_key: &[u8], signature_scheme: SignatureScheme) -> WalletResult<Wallet>;
    
    /// Get wallet by ID
    async fn get_wallet(&self, wallet_id: Uuid) -> WalletResult<Wallet>;
    
    /// List all wallets
    async fn list_wallets(&self) -> WalletResult<Vec<Wallet>>;
    
    /// Sign transaction
    async fn sign_transaction(&self, wallet_id: Uuid, transaction: &Transaction) -> WalletResult<Signature>;
    
    /// Sign message
    async fn sign_message(&self, wallet_id: Uuid, message: &[u8]) -> WalletResult<Signature>;
    
    /// Verify signature
    async fn verify_signature(&self, wallet_id: Uuid, message: &[u8], signature: &Signature) -> WalletResult<bool>;
    
    /// Delete wallet
    async fn delete_wallet(&self, wallet_id: Uuid) -> WalletResult<()>;
    
    /// Setup social recovery
    async fn setup_recovery(&self, wallet_id: Uuid, guardians: Vec<RecoveryGuardian>) -> WalletResult<()>;
    
    /// Get wallet metrics
    async fn get_metrics(&self) -> WalletResult<WalletMetrics>;
    
    /// Get health status
    async fn get_health(&self) -> WalletResult<WalletHealthStatus>;
}

/// Main wallet service implementation
pub struct WalletServiceImpl {
    multisig_service: Arc<dyn MultiSigService>,
    hardware_service: Arc<dyn HardwareWalletService>,
    key_manager: Arc<dyn KeyManager>,
    recovery_service: Arc<dyn RecoveryService>,
    config: WalletServiceConfig,
}

impl WalletServiceImpl {
    pub async fn new(config: WalletServiceConfig) -> WalletResult<Self> {
        // Initialize key store with encryption key
        let encryption_key = vec![0x42; 32]; // In production, this would be derived securely
        let key_store = Arc::new(SecureKeyStore::new(encryption_key));
        
        // Initialize services
        let multisig_service = Arc::new(MultiSigServiceImpl::new(config.multisig_config.clone()));
        let hardware_service = Arc::new(HardwareWalletServiceImpl::new(config.hardware_config.clone()));
        let key_manager = Arc::new(KeyManagerImpl::new(key_store, config.key_management_config.clone()));
        let recovery_service = Arc::new(RecoveryServiceImpl::new(config.recovery_config.clone()));
        
        Ok(Self {
            multisig_service,
            hardware_service,
            key_manager,
            recovery_service,
            config,
        })
    }
    
    /// Convert key pair to wallet
    fn key_pair_to_wallet(&self, key_pair: KeyPair, name: String) -> Wallet {
        Wallet {
            id: key_pair.public_key.key_id,
            name,
            wallet_type: crate::types::WalletType::SingleSig,
            address: key_pair.address,
            public_key: Some(key_pair.public_key),
            signature_scheme: key_pair.signature_scheme,
            derivation_path: None,
            created_at: key_pair.created_at,
            updated_at: key_pair.created_at,
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[async_trait]
impl WalletService for WalletServiceImpl {
    async fn create_wallet(&self, name: String, signature_scheme: SignatureScheme) -> WalletResult<Wallet> {
        let key_pair = self.key_manager.generate_key_pair(signature_scheme).await?;
        let wallet = self.key_pair_to_wallet(key_pair, name);
        Ok(wallet)
    }
    
    async fn create_multisig_wallet(&self, request: CreateWalletRequest) -> WalletResult<MultiSigWallet> {
        self.multisig_service.create_wallet(request).await
    }
    
    async fn import_wallet(&self, name: String, private_key: &[u8], signature_scheme: SignatureScheme) -> WalletResult<Wallet> {
        let key_pair = self.key_manager.import_private_key(private_key, signature_scheme).await?;
        let wallet = self.key_pair_to_wallet(key_pair, name);
        Ok(wallet)
    }
    
    async fn get_wallet(&self, wallet_id: Uuid) -> WalletResult<Wallet> {
        let public_key = self.key_manager.get_public_key(wallet_id).await?;
        
        // Derive address from public key (simplified)
        let address = match public_key.signature_scheme {
            SignatureScheme::ECDSA => {
                let address_bytes = &public_key.key_data[public_key.key_data.len()-20..];
                let address_str = format!("0x{}", hex::encode(address_bytes));
                Address::new(address_str, crate::types::AddressType::Ethereum)
            },
            SignatureScheme::EdDSA => {
                let address_str = hex::encode(&public_key.key_data[..32]);
                Address::new(address_str, crate::types::AddressType::Solana)
            },
            _ => return Err(WalletError::InvalidConfiguration("Unsupported signature scheme".to_string())),
        };
        
        Ok(Wallet {
            id: wallet_id,
            name: "Wallet".to_string(), // In production, this would be stored
            wallet_type: crate::types::WalletType::SingleSig,
            address,
            public_key: Some(public_key.clone()),
            signature_scheme: public_key.signature_scheme,
            derivation_path: None,
            created_at: public_key.created_at,
            updated_at: public_key.created_at,
            metadata: std::collections::HashMap::new(),
        })
    }
    
    async fn list_wallets(&self) -> WalletResult<Vec<Wallet>> {
        let key_pairs = self.key_manager.list_key_pairs().await?;
        let wallets = key_pairs.into_iter()
            .map(|kp| self.key_pair_to_wallet(kp, "Wallet".to_string()))
            .collect();
        Ok(wallets)
    }
    
    async fn sign_transaction(&self, wallet_id: Uuid, transaction: &Transaction) -> WalletResult<Signature> {
        // Serialize transaction for signing (simplified)
        let tx_data = serde_json::to_vec(transaction)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        
        let signature_data = self.key_manager.sign_data(wallet_id, &tx_data).await?;
        let public_key = self.key_manager.get_public_key(wallet_id).await?;
        
        // Derive address from public key
        let address = match public_key.signature_scheme {
            SignatureScheme::ECDSA => {
                let address_bytes = &public_key.key_data[public_key.key_data.len()-20..];
                let address_str = format!("0x{}", hex::encode(address_bytes));
                Address::new(address_str, crate::types::AddressType::Ethereum)
            },
            SignatureScheme::EdDSA => {
                let address_str = hex::encode(&public_key.key_data[..32]);
                Address::new(address_str, crate::types::AddressType::Solana)
            },
            _ => return Err(WalletError::InvalidConfiguration("Unsupported signature scheme".to_string())),
        };
        
        Ok(Signature {
            signature_data,
            signature_scheme: public_key.signature_scheme,
            signer_address: address,
            message_hash: tx_data,
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }
    
    async fn sign_message(&self, wallet_id: Uuid, message: &[u8]) -> WalletResult<Signature> {
        let signature_data = self.key_manager.sign_data(wallet_id, message).await?;
        let public_key = self.key_manager.get_public_key(wallet_id).await?;
        
        // Derive address from public key
        let address = match public_key.signature_scheme {
            SignatureScheme::ECDSA => {
                let address_bytes = &public_key.key_data[public_key.key_data.len()-20..];
                let address_str = format!("0x{}", hex::encode(address_bytes));
                Address::new(address_str, crate::types::AddressType::Ethereum)
            },
            SignatureScheme::EdDSA => {
                let address_str = hex::encode(&public_key.key_data[..32]);
                Address::new(address_str, crate::types::AddressType::Solana)
            },
            _ => return Err(WalletError::InvalidConfiguration("Unsupported signature scheme".to_string())),
        };
        
        Ok(Signature {
            signature_data,
            signature_scheme: public_key.signature_scheme,
            signer_address: address,
            message_hash: message.to_vec(),
            created_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }
    
    async fn verify_signature(&self, wallet_id: Uuid, message: &[u8], signature: &Signature) -> WalletResult<bool> {
        let public_key = self.key_manager.get_public_key(wallet_id).await?;
        self.key_manager.verify_signature(&public_key, message, &signature.signature_data).await
    }
    
    async fn delete_wallet(&self, wallet_id: Uuid) -> WalletResult<()> {
        self.key_manager.delete_key_pair(wallet_id).await
    }
    
    async fn setup_recovery(&self, wallet_id: Uuid, guardians: Vec<RecoveryGuardian>) -> WalletResult<()> {
        self.recovery_service.setup_social_recovery(wallet_id, guardians).await
    }
    
    async fn get_metrics(&self) -> WalletResult<WalletMetrics> {
        let wallets = self.list_wallets().await?;
        let total_wallets = wallets.len() as u64;
        
        // Simulate metrics (in production, these would be tracked)
        Ok(WalletMetrics {
            total_wallets,
            active_wallets: total_wallets,
            multisig_wallets: 0, // Would query multisig service
            hardware_wallets: 0, // Would query hardware service
            total_transactions_24h: 0,
            successful_transactions_24h: 0,
            failed_transactions_24h: 0,
            average_signing_time_ms: 150.0,
            key_rotations_24h: 0,
            recovery_requests_24h: 0,
            wallet_type_breakdown: std::collections::HashMap::new(),
            last_updated: chrono::Utc::now(),
        })
    }
    
    async fn get_health(&self) -> WalletResult<WalletHealthStatus> {
        // Simulate health check (in production, this would check all services)
        Ok(WalletHealthStatus {
            overall_status: "Healthy".to_string(),
            multisig_status: "Healthy".to_string(),
            hardware_status: "Healthy".to_string(),
            key_management_status: "Healthy".to_string(),
            recovery_status: "Healthy".to_string(),
            encryption_status: "Healthy".to_string(),
            device_statuses: std::collections::HashMap::new(),
            last_check: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SignatureScheme;

    #[tokio::test]
    async fn test_create_wallet() {
        let config = WalletServiceConfig::default();
        let service = WalletServiceImpl::new(config).await.unwrap();
        
        let wallet = service.create_wallet("Test Wallet".to_string(), SignatureScheme::ECDSA).await.unwrap();
        
        assert_eq!(wallet.name, "Test Wallet");
        assert_eq!(wallet.signature_scheme, SignatureScheme::ECDSA);
        assert!(wallet.address.is_valid());
    }

    #[tokio::test]
    async fn test_sign_and_verify() {
        let config = WalletServiceConfig::default();
        let service = WalletServiceImpl::new(config).await.unwrap();
        
        let wallet = service.create_wallet("Test Wallet".to_string(), SignatureScheme::ECDSA).await.unwrap();
        let message = b"Hello, World!";
        
        let signature = service.sign_message(wallet.id, message).await.unwrap();
        let is_valid = service.verify_signature(wallet.id, message, &signature).await.unwrap();
        
        assert!(is_valid);
        assert_eq!(signature.signer_address, wallet.address);
    }

    #[tokio::test]
    async fn test_import_wallet() {
        let config = WalletServiceConfig::default();
        let service = WalletServiceImpl::new(config).await.unwrap();
        
        let private_key = vec![0x42; 32];
        let wallet = service.import_wallet("Imported Wallet".to_string(), &private_key, SignatureScheme::ECDSA).await.unwrap();
        
        assert_eq!(wallet.name, "Imported Wallet");
        assert_eq!(wallet.signature_scheme, SignatureScheme::ECDSA);
    }

    #[tokio::test]
    async fn test_list_wallets() {
        let config = WalletServiceConfig::default();
        let service = WalletServiceImpl::new(config).await.unwrap();
        
        // Create multiple wallets
        service.create_wallet("Wallet 1".to_string(), SignatureScheme::ECDSA).await.unwrap();
        service.create_wallet("Wallet 2".to_string(), SignatureScheme::EdDSA).await.unwrap();
        
        let wallets = service.list_wallets().await.unwrap();
        assert_eq!(wallets.len(), 2);
    }

    #[tokio::test]
    async fn test_get_metrics() {
        let config = WalletServiceConfig::default();
        let service = WalletServiceImpl::new(config).await.unwrap();
        
        service.create_wallet("Test Wallet".to_string(), SignatureScheme::ECDSA).await.unwrap();
        
        let metrics = service.get_metrics().await.unwrap();
        assert_eq!(metrics.total_wallets, 1);
        assert_eq!(metrics.active_wallets, 1);
    }

    #[tokio::test]
    async fn test_get_health() {
        let config = WalletServiceConfig::default();
        let service = WalletServiceImpl::new(config).await.unwrap();
        
        let health = service.get_health().await.unwrap();
        assert_eq!(health.overall_status, "Healthy");
        assert_eq!(health.multisig_status, "Healthy");
        assert_eq!(health.hardware_status, "Healthy");
    }
}
