// =====================================================================================
// File: core-bridge/src/service.rs
// Description: Main bridge service implementation integrating all bridge components
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    atomic_swap::{AtomicSwapService, SwapDetails, SwapRequest},
    error::{BridgeError, BridgeResult},
    liquidity::{LiquidityPosition, LiquidityRequest, LiquidityService},
    relayer::{RelayRequest, RelayResult, RelayerService},
    security::{SecurityCheckRequest, SecurityCheckResult, SecurityService},
    transfer::{TransferRequest, TransferResult, TransferService},
    types::{BridgeStatus, BridgeTransaction, ChainId},
    validator::{ValidationRequest, ValidationResult, ValidatorService},
    BridgeServiceConfig,
};

/// Main bridge service implementation
pub struct BridgeService {
    config: Arc<RwLock<BridgeServiceConfig>>,
    transfer_service: Arc<dyn TransferService>,
    liquidity_service: Arc<dyn LiquidityService>,
    atomic_swap_service: Arc<dyn AtomicSwapService>,
    security_service: Arc<dyn SecurityService>,
    relayer_service: Arc<dyn RelayerService>,
    validator_service: Arc<dyn ValidatorService>,
}

impl BridgeService {
    /// Create a new bridge service
    pub fn new(
        config: BridgeServiceConfig,
        transfer_service: Arc<dyn TransferService>,
        liquidity_service: Arc<dyn LiquidityService>,
        atomic_swap_service: Arc<dyn AtomicSwapService>,
        security_service: Arc<dyn SecurityService>,
        relayer_service: Arc<dyn RelayerService>,
        validator_service: Arc<dyn ValidatorService>,
    ) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            transfer_service,
            liquidity_service,
            atomic_swap_service,
            security_service,
            relayer_service,
            validator_service,
        }
    }

    /// Process a cross-chain transfer with full security and validation
    pub async fn process_transfer(
        &self,
        request: TransferRequest,
    ) -> BridgeResult<BridgeTransaction> {
        // Step 1: Security check
        let security_request = SecurityCheckRequest {
            transaction_id: request.id,
            user_id: request.user_id.clone(),
            source_chain: request.source_chain,
            destination_chain: request.destination_chain,
            amount: request.amount,
            asset_symbol: request.asset_symbol.clone(),
            source_address: request.source_address.clone(),
            destination_address: request.destination_address.clone(),
            timestamp: request.created_at,
            user_agent: None,
            ip_address: None,
            session_id: None,
        };

        let security_result = self
            .security_service
            .check_transaction(security_request)
            .await?;

        if !security_result.approved {
            return Err(BridgeError::security_error(format!(
                "Transaction blocked: {:?}",
                security_result.blocked_reasons
            )));
        }

        // Step 2: Submit transfer
        let transfer_result = self
            .transfer_service
            .submit_transfer(request.clone())
            .await?;

        // Step 3: Create validation request if needed
        if security_result.requires_manual_review {
            let validation_request = ValidationRequest::new(
                request.id,
                request.source_chain,
                request.destination_chain,
                transfer_result.source_tx_hash.clone().unwrap_or_default(),
                0,             // Block number would be filled by the actual implementation
                String::new(), // Block hash would be filled by the actual implementation
                vec![],        // Message data would be filled by the actual implementation
                vec![],        // Proof data would be filled by the actual implementation
                String::new(), // Merkle root would be filled by the actual implementation
                vec![],        // Merkle proof would be filled by the actual implementation
            );

            let _validation_result = self
                .validator_service
                .submit_validation(validation_request)
                .await?;
        }

        // Step 4: Create bridge transaction record
        let bridge_transaction = BridgeTransaction {
            id: request.id,
            user_id: request.user_id,
            source_chain: request.source_chain,
            destination_chain: request.destination_chain,
            asset_symbol: request.asset_symbol,
            amount: request.amount,
            fee: transfer_result.fee_amount,
            source_address: request.source_address,
            destination_address: request.destination_address,
            source_tx_hash: transfer_result.source_tx_hash,
            destination_tx_hash: transfer_result.destination_tx_hash,
            status: transfer_result.status,
            created_at: request.created_at,
            updated_at: Utc::now(),
            completed_at: transfer_result.completed_at,
            metadata: serde_json::Value::Null,
        };

        Ok(bridge_transaction)
    }

    /// Get comprehensive bridge statistics
    pub async fn get_bridge_statistics(&self) -> BridgeResult<BridgeStatistics> {
        let transfer_health = self.transfer_service.health_check().await?;
        let liquidity_health = self.liquidity_service.health_check().await?;
        let security_health = self.security_service.health_check().await?;
        let relayer_health = self.relayer_service.health_check().await?;
        let validator_health = self.validator_service.health_check().await?;

        Ok(BridgeStatistics {
            total_transfers: transfer_health.active_transfers + transfer_health.pending_transfers,
            active_transfers: transfer_health.active_transfers,
            total_volume_24h: Decimal::ZERO, // Would be calculated from actual data
            total_fees_collected: Decimal::ZERO, // Would be calculated from actual data
            total_liquidity: liquidity_health.total_tvl,
            active_pools: liquidity_health.active_pools,
            security_alerts: security_health.alerts_pending,
            active_relayers: relayer_health.active_relayers,
            active_validators: validator_health.active_validators,
            success_rate: transfer_health.average_completion_time_minutes / 60.0, // Simplified calculation
            average_completion_time: transfer_health.average_completion_time_minutes,
            supported_chains: transfer_health.supported_chains.len() as u64,
            last_updated: Utc::now(),
        })
    }

    /// Update bridge configuration
    pub async fn update_config(&self, new_config: BridgeServiceConfig) -> BridgeResult<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }

    /// Get current bridge configuration
    pub async fn get_config(&self) -> BridgeServiceConfig {
        self.config.read().await.clone()
    }

    /// Emergency pause all bridge operations
    pub async fn emergency_pause(&self) -> BridgeResult<()> {
        let mut config = self.config.write().await;
        config.global_settings.emergency_pause_enabled = true;
        // Additional emergency pause logic would be implemented here
        Ok(())
    }

    /// Resume bridge operations after emergency pause
    pub async fn resume_operations(&self) -> BridgeResult<()> {
        let mut config = self.config.write().await;
        config.global_settings.emergency_pause_enabled = false;
        // Additional resume logic would be implemented here
        Ok(())
    }

    /// Comprehensive health check of all bridge components
    pub async fn comprehensive_health_check(&self) -> BridgeResult<BridgeHealthStatus> {
        let transfer_health = self.transfer_service.health_check().await?;
        let liquidity_health = self.liquidity_service.health_check().await?;
        let security_health = self.security_service.health_check().await?;
        let relayer_health = self.relayer_service.health_check().await?;
        let validator_health = self.validator_service.health_check().await?;

        let overall_status = if transfer_health.status == "healthy"
            && liquidity_health.status == "healthy"
            && security_health.status == "healthy"
            && relayer_health.status == "healthy"
            && validator_health.status == "healthy"
        {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        };

        Ok(BridgeHealthStatus {
            overall_status,
            transfer_service: transfer_health.status,
            liquidity_service: liquidity_health.status,
            security_service: security_health.status,
            relayer_service: relayer_health.status,
            validator_service: validator_health.status,
            emergency_pause_active: self
                .config
                .read()
                .await
                .global_settings
                .emergency_pause_enabled,
            last_check: Utc::now(),
        })
    }
}

/// Bridge statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatistics {
    pub total_transfers: u64,
    pub active_transfers: u64,
    pub total_volume_24h: Decimal,
    pub total_fees_collected: Decimal,
    pub total_liquidity: Decimal,
    pub active_pools: u64,
    pub security_alerts: u64,
    pub active_relayers: u64,
    pub active_validators: u64,
    pub success_rate: f64,
    pub average_completion_time: f64,
    pub supported_chains: u64,
    pub last_updated: DateTime<Utc>,
}

/// Bridge health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHealthStatus {
    pub overall_status: String,
    pub transfer_service: String,
    pub liquidity_service: String,
    pub security_service: String,
    pub relayer_service: String,
    pub validator_service: String,
    pub emergency_pause_active: bool,
    pub last_check: DateTime<Utc>,
}

/// Bridge service trait for external implementations
#[async_trait]
pub trait BridgeServiceTrait: Send + Sync {
    /// Process a cross-chain transfer
    async fn process_transfer(&self, request: TransferRequest) -> BridgeResult<BridgeTransaction>;

    /// Get bridge statistics
    async fn get_statistics(&self) -> BridgeResult<BridgeStatistics>;

    /// Health check
    async fn health_check(&self) -> BridgeResult<BridgeHealthStatus>;

    /// Emergency pause
    async fn emergency_pause(&self) -> BridgeResult<()>;

    /// Resume operations
    async fn resume_operations(&self) -> BridgeResult<()>;
}

#[async_trait]
impl BridgeServiceTrait for BridgeService {
    async fn process_transfer(&self, request: TransferRequest) -> BridgeResult<BridgeTransaction> {
        self.process_transfer(request).await
    }

    async fn get_statistics(&self) -> BridgeResult<BridgeStatistics> {
        self.get_bridge_statistics().await
    }

    async fn health_check(&self) -> BridgeResult<BridgeHealthStatus> {
        self.comprehensive_health_check().await
    }

    async fn emergency_pause(&self) -> BridgeResult<()> {
        self.emergency_pause().await
    }

    async fn resume_operations(&self) -> BridgeResult<()> {
        self.resume_operations().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_statistics_creation() {
        let stats = BridgeStatistics {
            total_transfers: 1000,
            active_transfers: 50,
            total_volume_24h: Decimal::new(100000000, 2), // $1,000,000
            total_fees_collected: Decimal::new(300000, 2), // $3,000
            total_liquidity: Decimal::new(5000000000, 2), // $50,000,000
            active_pools: 25,
            security_alerts: 2,
            active_relayers: 10,
            active_validators: 15,
            success_rate: 0.995,
            average_completion_time: 5.5,
            supported_chains: 4,
            last_updated: Utc::now(),
        };

        assert_eq!(stats.total_transfers, 1000);
        assert_eq!(stats.active_transfers, 50);
        assert_eq!(stats.success_rate, 0.995);
        assert_eq!(stats.supported_chains, 4);
    }

    #[test]
    fn test_bridge_health_status_creation() {
        let health = BridgeHealthStatus {
            overall_status: "healthy".to_string(),
            transfer_service: "healthy".to_string(),
            liquidity_service: "healthy".to_string(),
            security_service: "healthy".to_string(),
            relayer_service: "healthy".to_string(),
            validator_service: "healthy".to_string(),
            emergency_pause_active: false,
            last_check: Utc::now(),
        };

        assert_eq!(health.overall_status, "healthy");
        assert!(!health.emergency_pause_active);
    }
}
