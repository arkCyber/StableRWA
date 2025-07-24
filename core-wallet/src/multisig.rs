// =====================================================================================
// File: core-wallet/src/multisig.rs
// Description: Multi-signature wallet implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{WalletError, WalletResult};
use crate::types::{
    MultiSigWallet, MultiSigTransaction, Signer, Transaction, Signature, 
    Address, TransactionStatus, SignatureScheme
};

/// Multi-signature service trait
#[async_trait]
pub trait MultiSigService: Send + Sync {
    /// Create a new multi-signature wallet
    async fn create_wallet(&self, config: CreateWalletRequest) -> WalletResult<MultiSigWallet>;
    
    /// Add a signer to the wallet
    async fn add_signer(&self, wallet_id: Uuid, signer: Signer) -> WalletResult<()>;
    
    /// Remove a signer from the wallet
    async fn remove_signer(&self, wallet_id: Uuid, signer_address: Address) -> WalletResult<()>;
    
    /// Update wallet threshold
    async fn update_threshold(&self, wallet_id: Uuid, new_threshold: u32) -> WalletResult<()>;
    
    /// Propose a new transaction
    async fn propose_transaction(&self, wallet_id: Uuid, transaction: Transaction) -> WalletResult<MultiSigTransaction>;
    
    /// Sign a pending transaction
    async fn sign_transaction(&self, transaction_id: Uuid, signature: Signature) -> WalletResult<()>;
    
    /// Execute a transaction when threshold is met
    async fn execute_transaction(&self, transaction_id: Uuid) -> WalletResult<String>;
    
    /// Cancel a pending transaction
    async fn cancel_transaction(&self, transaction_id: Uuid, canceller: Address) -> WalletResult<()>;
    
    /// Get wallet information
    async fn get_wallet(&self, wallet_id: Uuid) -> WalletResult<MultiSigWallet>;
    
    /// Get pending transactions for a wallet
    async fn get_pending_transactions(&self, wallet_id: Uuid) -> WalletResult<Vec<MultiSigTransaction>>;
    
    /// Get transaction by ID
    async fn get_transaction(&self, transaction_id: Uuid) -> WalletResult<MultiSigTransaction>;
}

/// Multi-signature service implementation
pub struct MultiSigServiceImpl {
    wallets: Arc<Mutex<HashMap<Uuid, MultiSigWallet>>>,
    transactions: Arc<Mutex<HashMap<Uuid, MultiSigTransaction>>>,
    config: MultiSigConfig,
}

impl MultiSigServiceImpl {
    pub fn new(config: MultiSigConfig) -> Self {
        Self {
            wallets: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
    
    /// Validate wallet configuration
    fn validate_wallet_config(&self, config: &CreateWalletRequest) -> WalletResult<()> {
        if config.threshold == 0 {
            return Err(WalletError::InvalidConfiguration("Threshold cannot be zero".to_string()));
        }
        
        if config.threshold > config.signers.len() as u32 {
            return Err(WalletError::InvalidConfiguration(
                format!("Threshold {} cannot exceed number of signers {}", 
                    config.threshold, config.signers.len())
            ));
        }
        
        if config.signers.len() > self.config.max_signers as usize {
            return Err(WalletError::InvalidConfiguration(
                format!("Number of signers {} exceeds maximum {}", 
                    config.signers.len(), self.config.max_signers)
            ));
        }
        
        // Check for duplicate signers
        let mut addresses = std::collections::HashSet::new();
        for signer in &config.signers {
            if !addresses.insert(&signer.address) {
                return Err(WalletError::InvalidConfiguration(
                    format!("Duplicate signer address: {}", signer.address)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check if transaction has enough signatures
    fn has_enough_signatures(&self, transaction: &MultiSigTransaction) -> bool {
        transaction.signatures.len() as u32 >= transaction.required_signatures
    }
    
    /// Validate signature
    fn validate_signature(&self, signature: &Signature, transaction: &Transaction) -> WalletResult<()> {
        // In a real implementation, this would verify the cryptographic signature
        // For now, we'll do basic validation
        if signature.signature_data.is_empty() {
            return Err(WalletError::InvalidSignature("Empty signature data".to_string()));
        }
        
        if signature.message_hash.is_empty() {
            return Err(WalletError::InvalidSignature("Empty message hash".to_string()));
        }
        
        Ok(())
    }
}

#[async_trait]
impl MultiSigService for MultiSigServiceImpl {
    async fn create_wallet(&self, config: CreateWalletRequest) -> WalletResult<MultiSigWallet> {
        self.validate_wallet_config(&config)?;
        
        let wallet_id = Uuid::new_v4();
        let now = Utc::now();
        
        // Generate wallet address (simplified)
        let address = Address::new(
            format!("0x{}", hex::encode(&wallet_id.as_bytes()[..20])),
            crate::types::AddressType::Ethereum
        );
        
        let wallet = MultiSigWallet {
            id: wallet_id,
            name: config.name,
            address,
            threshold: config.threshold,
            signers: config.signers,
            pending_transactions: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: config.metadata,
        };
        
        let mut wallets = self.wallets.lock().await;
        wallets.insert(wallet_id, wallet.clone());
        
        Ok(wallet)
    }
    
    async fn add_signer(&self, wallet_id: Uuid, signer: Signer) -> WalletResult<()> {
        let mut wallets = self.wallets.lock().await;
        let wallet = wallets.get_mut(&wallet_id)
            .ok_or_else(|| WalletError::WalletNotFound(wallet_id.to_string()))?;
        
        // Check if signer already exists
        if wallet.signers.iter().any(|s| s.address == signer.address) {
            return Err(WalletError::InvalidConfiguration(
                format!("Signer {} already exists", signer.address)
            ));
        }
        
        // Check maximum signers limit
        if wallet.signers.len() >= self.config.max_signers as usize {
            return Err(WalletError::InvalidConfiguration(
                format!("Maximum number of signers ({}) reached", self.config.max_signers)
            ));
        }
        
        wallet.signers.push(signer);
        wallet.updated_at = Utc::now();
        
        Ok(())
    }
    
    async fn remove_signer(&self, wallet_id: Uuid, signer_address: Address) -> WalletResult<()> {
        let mut wallets = self.wallets.lock().await;
        let wallet = wallets.get_mut(&wallet_id)
            .ok_or_else(|| WalletError::WalletNotFound(wallet_id.to_string()))?;
        
        let initial_len = wallet.signers.len();
        wallet.signers.retain(|s| s.address != signer_address);
        
        if wallet.signers.len() == initial_len {
            return Err(WalletError::InvalidConfiguration(
                format!("Signer {} not found", signer_address)
            ));
        }
        
        // Ensure threshold is still valid
        if wallet.threshold > wallet.signers.len() as u32 {
            return Err(WalletError::InvalidConfiguration(
                format!("Removing signer would make threshold {} invalid for {} remaining signers", 
                    wallet.threshold, wallet.signers.len())
            ));
        }
        
        wallet.updated_at = Utc::now();
        
        Ok(())
    }
    
    async fn update_threshold(&self, wallet_id: Uuid, new_threshold: u32) -> WalletResult<()> {
        let mut wallets = self.wallets.lock().await;
        let wallet = wallets.get_mut(&wallet_id)
            .ok_or_else(|| WalletError::WalletNotFound(wallet_id.to_string()))?;
        
        if new_threshold == 0 {
            return Err(WalletError::InvalidConfiguration("Threshold cannot be zero".to_string()));
        }
        
        if new_threshold > wallet.signers.len() as u32 {
            return Err(WalletError::InvalidConfiguration(
                format!("Threshold {} cannot exceed number of signers {}", 
                    new_threshold, wallet.signers.len())
            ));
        }
        
        wallet.threshold = new_threshold;
        wallet.updated_at = Utc::now();
        
        Ok(())
    }
    
    async fn propose_transaction(&self, wallet_id: Uuid, transaction: Transaction) -> WalletResult<MultiSigTransaction> {
        let wallets = self.wallets.lock().await;
        let wallet = wallets.get(&wallet_id)
            .ok_or_else(|| WalletError::WalletNotFound(wallet_id.to_string()))?;
        
        let transaction_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = Some(now + Duration::minutes(self.config.signing_timeout_minutes as i64));
        
        let multisig_tx = MultiSigTransaction {
            id: transaction_id,
            wallet_id,
            transaction,
            signatures: Vec::new(),
            required_signatures: wallet.threshold,
            status: TransactionStatus::Pending,
            created_at: now,
            expires_at,
            executed_at: None,
            metadata: HashMap::new(),
        };
        
        let mut transactions = self.transactions.lock().await;
        transactions.insert(transaction_id, multisig_tx.clone());
        
        Ok(multisig_tx)
    }
    
    async fn sign_transaction(&self, transaction_id: Uuid, signature: Signature) -> WalletResult<()> {
        let mut transactions = self.transactions.lock().await;
        let transaction = transactions.get_mut(&transaction_id)
            .ok_or_else(|| WalletError::TransactionError(format!("Transaction {} not found", transaction_id)))?;
        
        // Check if transaction is still pending
        if transaction.status != TransactionStatus::Pending {
            return Err(WalletError::TransactionError(
                format!("Transaction {} is not pending", transaction_id)
            ));
        }
        
        // Check if transaction has expired
        if let Some(expires_at) = transaction.expires_at {
            if Utc::now() > expires_at {
                transaction.status = TransactionStatus::Expired;
                return Err(WalletError::SigningTimeout { 
                    timeout_minutes: self.config.signing_timeout_minutes 
                });
            }
        }
        
        // Validate signature
        self.validate_signature(&signature, &transaction.transaction)?;
        
        // Check if signer already signed
        if transaction.signatures.iter().any(|s| s.signer_address == signature.signer_address) {
            return Err(WalletError::InvalidSignature(
                format!("Signer {} already signed this transaction", signature.signer_address)
            ));
        }
        
        // Verify signer is authorized
        let wallets = self.wallets.lock().await;
        let wallet = wallets.get(&transaction.wallet_id)
            .ok_or_else(|| WalletError::WalletNotFound(transaction.wallet_id.to_string()))?;
        
        if !wallet.signers.iter().any(|s| s.address == signature.signer_address) {
            return Err(WalletError::PermissionDenied(
                format!("Address {} is not authorized to sign", signature.signer_address)
            ));
        }
        
        transaction.signatures.push(signature);
        
        // Check if we have enough signatures
        if self.has_enough_signatures(transaction) {
            transaction.status = TransactionStatus::Ready;
        }
        
        Ok(())
    }
    
    async fn execute_transaction(&self, transaction_id: Uuid) -> WalletResult<String> {
        let mut transactions = self.transactions.lock().await;
        let transaction = transactions.get_mut(&transaction_id)
            .ok_or_else(|| WalletError::TransactionError(format!("Transaction {} not found", transaction_id)))?;
        
        if transaction.status != TransactionStatus::Ready {
            return Err(WalletError::TransactionError(
                format!("Transaction {} is not ready for execution", transaction_id)
            ));
        }
        
        if !self.has_enough_signatures(transaction) {
            return Err(WalletError::InsufficientSigners {
                required: transaction.required_signatures,
                actual: transaction.signatures.len() as u32,
            });
        }
        
        // Simulate transaction execution
        let tx_hash = format!("0x{}", hex::encode(&transaction_id.as_bytes()));
        
        transaction.status = TransactionStatus::Executed;
        transaction.executed_at = Some(Utc::now());
        
        Ok(tx_hash)
    }
    
    async fn cancel_transaction(&self, transaction_id: Uuid, canceller: Address) -> WalletResult<()> {
        let mut transactions = self.transactions.lock().await;
        let transaction = transactions.get_mut(&transaction_id)
            .ok_or_else(|| WalletError::TransactionError(format!("Transaction {} not found", transaction_id)))?;
        
        if transaction.status != TransactionStatus::Pending {
            return Err(WalletError::TransactionError(
                format!("Transaction {} cannot be cancelled", transaction_id)
            ));
        }
        
        // Verify canceller is authorized (wallet signer or transaction proposer)
        let wallets = self.wallets.lock().await;
        let wallet = wallets.get(&transaction.wallet_id)
            .ok_or_else(|| WalletError::WalletNotFound(transaction.wallet_id.to_string()))?;
        
        let is_signer = wallet.signers.iter().any(|s| s.address == canceller);
        let is_proposer = transaction.transaction.from == canceller;
        
        if !is_signer && !is_proposer {
            return Err(WalletError::PermissionDenied(
                format!("Address {} is not authorized to cancel", canceller)
            ));
        }
        
        transaction.status = TransactionStatus::Cancelled;
        
        Ok(())
    }
    
    async fn get_wallet(&self, wallet_id: Uuid) -> WalletResult<MultiSigWallet> {
        let wallets = self.wallets.lock().await;
        wallets.get(&wallet_id)
            .cloned()
            .ok_or_else(|| WalletError::WalletNotFound(wallet_id.to_string()))
    }
    
    async fn get_pending_transactions(&self, wallet_id: Uuid) -> WalletResult<Vec<MultiSigTransaction>> {
        let transactions = self.transactions.lock().await;
        let pending: Vec<MultiSigTransaction> = transactions.values()
            .filter(|tx| tx.wallet_id == wallet_id && tx.status == TransactionStatus::Pending)
            .cloned()
            .collect();
        
        Ok(pending)
    }
    
    async fn get_transaction(&self, transaction_id: Uuid) -> WalletResult<MultiSigTransaction> {
        let transactions = self.transactions.lock().await;
        transactions.get(&transaction_id)
            .cloned()
            .ok_or_else(|| WalletError::TransactionError(format!("Transaction {} not found", transaction_id)))
    }
}

/// Multi-signature configuration
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

/// Request to create a new multi-signature wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub name: String,
    pub threshold: u32,
    pub signers: Vec<Signer>,
    pub metadata: HashMap<String, String>,
}

/// Signing policy for multi-signature operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningPolicy {
    pub require_all_signers: bool,
    pub allow_partial_signing: bool,
    pub signing_order_enforced: bool,
    pub timeout_minutes: u32,
}

/// Threshold policy for multi-signature operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdPolicy {
    pub threshold: u32,
    pub weighted_voting: bool,
    pub minimum_weight: u32,
    pub dynamic_threshold: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PublicKey, AddressType};

    fn create_test_signer(address: &str) -> Signer {
        Signer {
            id: Uuid::new_v4(),
            address: Address::new(address.to_string(), AddressType::Ethereum),
            public_key: PublicKey {
                key_data: vec![1, 2, 3, 4],
                key_format: "raw".to_string(),
                signature_scheme: SignatureScheme::ECDSA,
                key_id: Uuid::new_v4(),
                created_at: Utc::now(),
            },
            name: Some("Test Signer".to_string()),
            weight: 1,
            added_at: Utc::now(),
            last_signed: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_create_multisig_wallet() {
        let config = MultiSigConfig::default();
        let service = MultiSigServiceImpl::new(config);
        
        let signers = vec![
            create_test_signer("0x1111111111111111111111111111111111111111"),
            create_test_signer("0x2222222222222222222222222222222222222222"),
            create_test_signer("0x3333333333333333333333333333333333333333"),
        ];
        
        let request = CreateWalletRequest {
            name: "Test Wallet".to_string(),
            threshold: 2,
            signers,
            metadata: HashMap::new(),
        };
        
        let wallet = service.create_wallet(request).await.unwrap();
        assert_eq!(wallet.name, "Test Wallet");
        assert_eq!(wallet.threshold, 2);
        assert_eq!(wallet.signers.len(), 3);
    }

    #[tokio::test]
    async fn test_invalid_threshold() {
        let config = MultiSigConfig::default();
        let service = MultiSigServiceImpl::new(config);
        
        let signers = vec![
            create_test_signer("0x1111111111111111111111111111111111111111"),
        ];
        
        let request = CreateWalletRequest {
            name: "Test Wallet".to_string(),
            threshold: 2, // More than number of signers
            signers,
            metadata: HashMap::new(),
        };
        
        let result = service.create_wallet(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WalletError::InvalidConfiguration(_)));
    }
}
