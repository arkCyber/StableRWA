// =====================================================================================
// File: core-wallet/src/recovery.rs
// Description: Social recovery and wallet recovery mechanisms
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::{WalletError, WalletResult};
use crate::types::{Address, Wallet};

/// Recovery service trait
#[async_trait]
pub trait RecoveryService: Send + Sync {
    /// Initialize social recovery for a wallet
    async fn setup_social_recovery(&self, wallet_id: Uuid, guardians: Vec<RecoveryGuardian>) -> WalletResult<()>;
    
    /// Initiate wallet recovery process
    async fn initiate_recovery(&self, wallet_id: Uuid, requester: Address) -> WalletResult<RecoveryRequest>;
    
    /// Guardian approves recovery request
    async fn approve_recovery(&self, request_id: Uuid, guardian: Address, signature: Vec<u8>) -> WalletResult<()>;
    
    /// Execute recovery after threshold is met and delay has passed
    async fn execute_recovery(&self, request_id: Uuid) -> WalletResult<Wallet>;
    
    /// Cancel recovery request
    async fn cancel_recovery(&self, request_id: Uuid, canceller: Address) -> WalletResult<()>;
    
    /// Get recovery status
    async fn get_recovery_status(&self, wallet_id: Uuid) -> WalletResult<Option<RecoveryRequest>>;
    
    /// Add guardian to existing recovery setup
    async fn add_guardian(&self, wallet_id: Uuid, guardian: RecoveryGuardian) -> WalletResult<()>;
    
    /// Remove guardian from recovery setup
    async fn remove_guardian(&self, wallet_id: Uuid, guardian_address: Address) -> WalletResult<()>;
    
    /// Update recovery threshold
    async fn update_threshold(&self, wallet_id: Uuid, new_threshold: u32) -> WalletResult<()>;
    
    /// Get guardians for a wallet
    async fn get_guardians(&self, wallet_id: Uuid) -> WalletResult<Vec<RecoveryGuardian>>;
}

/// Recovery service implementation
pub struct RecoveryServiceImpl {
    recovery_setups: Arc<Mutex<HashMap<Uuid, SocialRecovery>>>,
    recovery_requests: Arc<Mutex<HashMap<Uuid, RecoveryRequest>>>,
    config: RecoveryConfig,
}

impl RecoveryServiceImpl {
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            recovery_setups: Arc::new(Mutex::new(HashMap::new())),
            recovery_requests: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
    
    /// Validate guardian setup
    fn validate_guardian_setup(&self, guardians: &[RecoveryGuardian], threshold: u32) -> WalletResult<()> {
        if guardians.len() < self.config.min_guardians as usize {
            return Err(WalletError::InvalidConfiguration(
                format!("Minimum {} guardians required", self.config.min_guardians)
            ));
        }
        
        if threshold == 0 {
            return Err(WalletError::InvalidConfiguration("Threshold cannot be zero".to_string()));
        }
        
        if threshold > guardians.len() as u32 {
            return Err(WalletError::InvalidConfiguration(
                format!("Threshold {} cannot exceed number of guardians {}", threshold, guardians.len())
            ));
        }
        
        // Check for duplicate guardians
        let mut addresses = std::collections::HashSet::new();
        for guardian in guardians {
            if !addresses.insert(&guardian.address) {
                return Err(WalletError::InvalidConfiguration(
                    format!("Duplicate guardian address: {}", guardian.address)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check if recovery delay has elapsed
    fn is_recovery_delay_elapsed(&self, request: &RecoveryRequest) -> bool {
        let delay = Duration::hours(self.config.recovery_delay_hours as i64);
        Utc::now() >= request.created_at + delay
    }
    
    /// Check if recovery has enough approvals
    fn has_enough_approvals(&self, request: &RecoveryRequest, recovery_setup: &SocialRecovery) -> bool {
        request.approvals.len() as u32 >= recovery_setup.threshold
    }
}

#[async_trait]
impl RecoveryService for RecoveryServiceImpl {
    async fn setup_social_recovery(&self, wallet_id: Uuid, guardians: Vec<RecoveryGuardian>) -> WalletResult<()> {
        let threshold = self.config.recovery_threshold;
        self.validate_guardian_setup(&guardians, threshold)?;
        
        let recovery_setup = SocialRecovery {
            wallet_id,
            guardians,
            threshold,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        };
        
        let mut setups = self.recovery_setups.lock().await;
        setups.insert(wallet_id, recovery_setup);
        
        Ok(())
    }
    
    async fn initiate_recovery(&self, wallet_id: Uuid, requester: Address) -> WalletResult<RecoveryRequest> {
        let setups = self.recovery_setups.lock().await;
        let recovery_setup = setups.get(&wallet_id)
            .ok_or_else(|| WalletError::RecoveryError("No recovery setup found for wallet".to_string()))?;
        
        if !recovery_setup.is_active {
            return Err(WalletError::RecoveryError("Recovery is not active for this wallet".to_string()));
        }
        
        // Check if requester is a guardian
        let is_guardian = recovery_setup.guardians.iter()
            .any(|g| g.address == requester);
        
        if !is_guardian {
            return Err(WalletError::PermissionDenied(
                "Only guardians can initiate recovery".to_string()
            ));
        }
        
        let request_id = Uuid::new_v4();
        let request = RecoveryRequest {
            id: request_id,
            wallet_id,
            requester,
            approvals: Vec::new(),
            status: RecoveryStatus::Pending,
            created_at: Utc::now(),
            executed_at: None,
            new_owner: None, // Will be set during execution
        };
        
        let mut requests = self.recovery_requests.lock().await;
        requests.insert(request_id, request.clone());
        
        Ok(request)
    }
    
    async fn approve_recovery(&self, request_id: Uuid, guardian: Address, signature: Vec<u8>) -> WalletResult<()> {
        let mut requests = self.recovery_requests.lock().await;
        let request = requests.get_mut(&request_id)
            .ok_or_else(|| WalletError::RecoveryRequestNotFound(request_id.to_string()))?;
        
        if request.status != RecoveryStatus::Pending {
            return Err(WalletError::RecoveryError(
                format!("Recovery request {} is not pending", request_id)
            ));
        }
        
        // Verify guardian is authorized
        let setups = self.recovery_setups.lock().await;
        let recovery_setup = setups.get(&request.wallet_id)
            .ok_or_else(|| WalletError::RecoveryError("Recovery setup not found".to_string()))?;
        
        let is_guardian = recovery_setup.guardians.iter()
            .any(|g| g.address == guardian);
        
        if !is_guardian {
            return Err(WalletError::PermissionDenied(
                "Address is not an authorized guardian".to_string()
            ));
        }
        
        // Check if guardian already approved
        if request.approvals.iter().any(|a| a.guardian == guardian) {
            return Err(WalletError::RecoveryError(
                "Guardian has already approved this request".to_string()
            ));
        }
        
        // Add approval
        let approval = RecoveryApproval {
            guardian,
            signature,
            approved_at: Utc::now(),
        };
        
        request.approvals.push(approval);
        
        // Check if we have enough approvals
        if self.has_enough_approvals(request, recovery_setup) {
            request.status = RecoveryStatus::ReadyForExecution;
        }
        
        Ok(())
    }
    
    async fn execute_recovery(&self, request_id: Uuid) -> WalletResult<Wallet> {
        let mut requests = self.recovery_requests.lock().await;
        let request = requests.get_mut(&request_id)
            .ok_or_else(|| WalletError::RecoveryRequestNotFound(request_id.to_string()))?;
        
        if request.status != RecoveryStatus::ReadyForExecution {
            return Err(WalletError::RecoveryError(
                "Recovery request is not ready for execution".to_string()
            ));
        }
        
        // Check if delay has elapsed
        if !self.is_recovery_delay_elapsed(request) {
            let remaining_hours = self.config.recovery_delay_hours - 
                (Utc::now() - request.created_at).num_hours() as u32;
            return Err(WalletError::RecoveryDelayNotElapsed { remaining_hours });
        }
        
        // Simulate wallet recovery (in production, this would generate new keys)
        let recovered_wallet = Wallet {
            id: request.wallet_id,
            name: "Recovered Wallet".to_string(),
            wallet_type: crate::types::WalletType::SingleSig,
            address: crate::types::Address::new(
                format!("0x{}", hex::encode(&request.wallet_id.as_bytes()[..20])),
                crate::types::AddressType::Ethereum
            ),
            public_key: None,
            signature_scheme: crate::types::SignatureScheme::ECDSA,
            derivation_path: Some("m/44'/60'/0'/0".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        request.status = RecoveryStatus::Executed;
        request.executed_at = Some(Utc::now());
        request.new_owner = Some(recovered_wallet.address.clone());
        
        Ok(recovered_wallet)
    }
    
    async fn cancel_recovery(&self, request_id: Uuid, canceller: Address) -> WalletResult<()> {
        let mut requests = self.recovery_requests.lock().await;
        let request = requests.get_mut(&request_id)
            .ok_or_else(|| WalletError::RecoveryRequestNotFound(request_id.to_string()))?;
        
        if request.status == RecoveryStatus::Executed || request.status == RecoveryStatus::Cancelled {
            return Err(WalletError::RecoveryError(
                "Recovery request cannot be cancelled".to_string()
            ));
        }
        
        // Verify canceller is authorized (original requester or wallet owner)
        if request.requester != canceller {
            return Err(WalletError::PermissionDenied(
                "Only the requester can cancel recovery".to_string()
            ));
        }
        
        request.status = RecoveryStatus::Cancelled;
        
        Ok(())
    }
    
    async fn get_recovery_status(&self, wallet_id: Uuid) -> WalletResult<Option<RecoveryRequest>> {
        let requests = self.recovery_requests.lock().await;
        let active_request = requests.values()
            .find(|r| r.wallet_id == wallet_id && 
                (r.status == RecoveryStatus::Pending || r.status == RecoveryStatus::ReadyForExecution))
            .cloned();
        
        Ok(active_request)
    }
    
    async fn add_guardian(&self, wallet_id: Uuid, guardian: RecoveryGuardian) -> WalletResult<()> {
        let mut setups = self.recovery_setups.lock().await;
        let recovery_setup = setups.get_mut(&wallet_id)
            .ok_or_else(|| WalletError::RecoveryError("Recovery setup not found".to_string()))?;
        
        // Check if guardian already exists
        if recovery_setup.guardians.iter().any(|g| g.address == guardian.address) {
            return Err(WalletError::InvalidConfiguration(
                format!("Guardian {} already exists", guardian.address)
            ));
        }
        
        recovery_setup.guardians.push(guardian);
        recovery_setup.updated_at = Utc::now();
        
        Ok(())
    }
    
    async fn remove_guardian(&self, wallet_id: Uuid, guardian_address: Address) -> WalletResult<()> {
        let mut setups = self.recovery_setups.lock().await;
        let recovery_setup = setups.get_mut(&wallet_id)
            .ok_or_else(|| WalletError::RecoveryError("Recovery setup not found".to_string()))?;
        
        let initial_len = recovery_setup.guardians.len();
        recovery_setup.guardians.retain(|g| g.address != guardian_address);
        
        if recovery_setup.guardians.len() == initial_len {
            return Err(WalletError::InvalidConfiguration(
                format!("Guardian {} not found", guardian_address)
            ));
        }
        
        // Ensure threshold is still valid
        if recovery_setup.threshold > recovery_setup.guardians.len() as u32 {
            return Err(WalletError::InvalidConfiguration(
                format!("Removing guardian would make threshold {} invalid for {} remaining guardians", 
                    recovery_setup.threshold, recovery_setup.guardians.len())
            ));
        }
        
        recovery_setup.updated_at = Utc::now();
        
        Ok(())
    }
    
    async fn update_threshold(&self, wallet_id: Uuid, new_threshold: u32) -> WalletResult<()> {
        let mut setups = self.recovery_setups.lock().await;
        let recovery_setup = setups.get_mut(&wallet_id)
            .ok_or_else(|| WalletError::RecoveryError("Recovery setup not found".to_string()))?;
        
        self.validate_guardian_setup(&recovery_setup.guardians, new_threshold)?;
        
        recovery_setup.threshold = new_threshold;
        recovery_setup.updated_at = Utc::now();
        
        Ok(())
    }
    
    async fn get_guardians(&self, wallet_id: Uuid) -> WalletResult<Vec<RecoveryGuardian>> {
        let setups = self.recovery_setups.lock().await;
        let recovery_setup = setups.get(&wallet_id)
            .ok_or_else(|| WalletError::RecoveryError("Recovery setup not found".to_string()))?;
        
        Ok(recovery_setup.guardians.clone())
    }
}

/// Recovery configuration
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

/// Social recovery setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialRecovery {
    pub wallet_id: Uuid,
    pub guardians: Vec<RecoveryGuardian>,
    pub threshold: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Recovery guardian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryGuardian {
    pub address: Address,
    pub name: Option<String>,
    pub contact_info: Option<String>,
    pub added_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Recovery request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRequest {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub requester: Address,
    pub approvals: Vec<RecoveryApproval>,
    pub status: RecoveryStatus,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub new_owner: Option<Address>,
}

/// Recovery approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryApproval {
    pub guardian: Address,
    pub signature: Vec<u8>,
    pub approved_at: DateTime<Utc>,
}

/// Recovery status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStatus {
    Pending,
    ReadyForExecution,
    Executed,
    Cancelled,
    Expired,
}

/// Recovery proposal (for future use)
pub struct RecoveryProposal {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub proposed_changes: RecoveryChanges,
    pub proposer: Address,
    pub votes: Vec<RecoveryVote>,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub voting_deadline: DateTime<Utc>,
}

/// Recovery changes proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryChanges {
    AddGuardian(RecoveryGuardian),
    RemoveGuardian(Address),
    UpdateThreshold(u32),
    UpdateDelay(u32),
}

/// Recovery vote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryVote {
    pub voter: Address,
    pub vote: bool, // true for approve, false for reject
    pub voted_at: DateTime<Utc>,
}

/// Proposal status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
    Executed,
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AddressType;

    fn create_test_guardian(address: &str) -> RecoveryGuardian {
        RecoveryGuardian {
            address: Address::new(address.to_string(), AddressType::Ethereum),
            name: Some("Test Guardian".to_string()),
            contact_info: Some("test@example.com".to_string()),
            added_at: Utc::now(),
            is_active: true,
        }
    }

    #[tokio::test]
    async fn test_setup_social_recovery() {
        let config = RecoveryConfig::default();
        let service = RecoveryServiceImpl::new(config);
        
        let wallet_id = Uuid::new_v4();
        let guardians = vec![
            create_test_guardian("0x1111111111111111111111111111111111111111"),
            create_test_guardian("0x2222222222222222222222222222222222222222"),
            create_test_guardian("0x3333333333333333333333333333333333333333"),
        ];
        
        let result = service.setup_social_recovery(wallet_id, guardians).await;
        assert!(result.is_ok());
        
        let retrieved_guardians = service.get_guardians(wallet_id).await.unwrap();
        assert_eq!(retrieved_guardians.len(), 3);
    }

    #[tokio::test]
    async fn test_recovery_flow() {
        let config = RecoveryConfig::default();
        let service = RecoveryServiceImpl::new(config);
        
        let wallet_id = Uuid::new_v4();
        let guardians = vec![
            create_test_guardian("0x1111111111111111111111111111111111111111"),
            create_test_guardian("0x2222222222222222222222222222222222222222"),
            create_test_guardian("0x3333333333333333333333333333333333333333"),
        ];
        
        // Setup recovery
        service.setup_social_recovery(wallet_id, guardians.clone()).await.unwrap();
        
        // Initiate recovery
        let request = service.initiate_recovery(wallet_id, guardians[0].address.clone()).await.unwrap();
        assert_eq!(request.status, RecoveryStatus::Pending);
        
        // Approve recovery
        service.approve_recovery(request.id, guardians[0].address.clone(), vec![1, 2, 3]).await.unwrap();
        service.approve_recovery(request.id, guardians[1].address.clone(), vec![4, 5, 6]).await.unwrap();
        
        // Check status
        let status = service.get_recovery_status(wallet_id).await.unwrap();
        assert!(status.is_some());
        assert_eq!(status.unwrap().status, RecoveryStatus::ReadyForExecution);
    }

    #[tokio::test]
    async fn test_invalid_guardian_setup() {
        let config = RecoveryConfig::default();
        let service = RecoveryServiceImpl::new(config);
        
        let wallet_id = Uuid::new_v4();
        
        // Too few guardians
        let guardians = vec![
            create_test_guardian("0x1111111111111111111111111111111111111111"),
        ];
        
        let result = service.setup_social_recovery(wallet_id, guardians).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WalletError::InvalidConfiguration(_)));
    }
}
