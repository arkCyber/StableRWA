// =====================================================================================
// File: core-asset-lifecycle/src/service.rs
// Description: Main asset lifecycle service orchestrator
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    custody::{CustodyService, DefaultCustodyService},
    error::{AssetError, AssetResult},
    maintenance::{MaintenanceService, DefaultMaintenanceService},
    registration::AssetRegistrationService,
    tokenization::{TokenizationService, DefaultTokenizationService},
    types::{Asset, AssetLifecycleEvent, AssetStatus, LifecycleStage},
    valuation::AssetValuationService,
    verification::AssetVerificationService,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, debug, warn};
use uuid::Uuid;

/// Asset lifecycle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConfig {
    pub auto_verification: bool,
    pub auto_valuation: bool,
    pub maintenance_scheduling: bool,
    pub custody_required: bool,
    pub tokenization_enabled: bool,
    pub compliance_checks: bool,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            auto_verification: true,
            auto_valuation: true,
            maintenance_scheduling: true,
            custody_required: false,
            tokenization_enabled: false,
            compliance_checks: true,
        }
    }
}

/// Asset lifecycle orchestrator trait
#[async_trait]
pub trait AssetLifecycleService: Send + Sync {
    /// Initialize asset lifecycle
    async fn initialize_lifecycle(
        &self,
        asset: Asset,
        config: LifecycleConfig,
    ) -> AssetResult<Vec<AssetLifecycleEvent>>;

    /// Progress asset to next lifecycle stage
    async fn progress_lifecycle(
        &self,
        asset_id: Uuid,
        target_stage: LifecycleStage,
    ) -> AssetResult<Vec<AssetLifecycleEvent>>;

    /// Get current lifecycle status
    async fn get_lifecycle_status(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<AssetLifecycleStatus>;

    /// Get lifecycle history
    async fn get_lifecycle_history(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<AssetLifecycleEvent>>;

    /// Update asset status
    async fn update_asset_status(
        &self,
        asset_id: Uuid,
        status: AssetStatus,
        reason: Option<String>,
    ) -> AssetResult<AssetLifecycleEvent>;

    /// Retire asset
    async fn retire_asset(
        &self,
        asset_id: Uuid,
        retirement_reason: String,
    ) -> AssetResult<AssetLifecycleEvent>;

    /// Get lifecycle metrics
    async fn get_lifecycle_metrics(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<LifecycleMetrics>;
}

/// Asset lifecycle status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLifecycleStatus {
    pub asset_id: Uuid,
    pub current_stage: LifecycleStage,
    pub status: AssetStatus,
    pub stage_progress: f64, // 0.0 to 1.0
    pub next_actions: Vec<String>,
    pub blocking_issues: Vec<String>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

/// Lifecycle metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleMetrics {
    pub asset_id: Uuid,
    pub total_lifecycle_days: u32,
    pub current_stage_days: u32,
    pub completion_percentage: f64,
    pub total_costs: f64,
    pub stage_costs: std::collections::HashMap<LifecycleStage, f64>,
    pub efficiency_score: f64, // 0.0 to 1.0
    pub risk_score: f64, // 0.0 to 1.0
}

/// Default asset lifecycle service implementation
pub struct DefaultAssetLifecycleService {
    registration_service: Arc<AssetRegistrationService>,
    verification_service: Arc<AssetVerificationService>,
    valuation_service: Arc<AssetValuationService>,
    custody_service: Arc<dyn CustodyService>,
    maintenance_service: Arc<dyn MaintenanceService>,
    tokenization_service: Arc<dyn TokenizationService>,
}

impl DefaultAssetLifecycleService {
    pub fn new() -> Self {
        Self {
            registration_service: Arc::new(AssetRegistrationService::new(crate::registration::RegistrationConfig::default())),
            verification_service: Arc::new(AssetVerificationService::new(crate::verification::VerificationConfig::default())),
            valuation_service: Arc::new(AssetValuationService::new(crate::valuation::ValuationConfig::default())),
            custody_service: Arc::new(DefaultCustodyService::new()),
            maintenance_service: Arc::new(DefaultMaintenanceService::new()),
            tokenization_service: Arc::new(DefaultTokenizationService::new()),
        }
    }

    pub fn with_services(
        registration_service: Arc<AssetRegistrationService>,
        verification_service: Arc<AssetVerificationService>,
        valuation_service: Arc<AssetValuationService>,
        custody_service: Arc<dyn CustodyService>,
        maintenance_service: Arc<dyn MaintenanceService>,
        tokenization_service: Arc<dyn TokenizationService>,
    ) -> Self {
        Self {
            registration_service,
            verification_service,
            valuation_service,
            custody_service,
            maintenance_service,
            tokenization_service,
        }
    }

    /// Determine next lifecycle stage
    fn determine_next_stage(&self, current_stage: &LifecycleStage) -> Option<LifecycleStage> {
        match current_stage {
            LifecycleStage::Registration => Some(LifecycleStage::Verification),
            LifecycleStage::Verification => Some(LifecycleStage::Valuation),
            LifecycleStage::Valuation => Some(LifecycleStage::Active),
            LifecycleStage::Active => Some(LifecycleStage::Maintenance),
            LifecycleStage::Maintenance => Some(LifecycleStage::Active),
            LifecycleStage::Disposal => None,
            LifecycleStage::Retired => None,
        }
    }

    /// Calculate stage progress
    fn calculate_stage_progress(&self, stage: &LifecycleStage, events: &[AssetLifecycleEvent]) -> f64 {
        // Mock calculation based on completed events in the stage
        let stage_events: Vec<_> = events.iter()
            .filter(|e| &e.lifecycle_stage == stage)
            .collect();

        if stage_events.is_empty() {
            return 0.0;
        }

        // Simple progress calculation
        match stage {
            LifecycleStage::Registration => 1.0, // Assume registration is complete if we have events
            LifecycleStage::Verification => 0.8, // Mock 80% complete
            LifecycleStage::Valuation => 0.6,    // Mock 60% complete
            LifecycleStage::Active => 1.0,       // Active is ongoing
            LifecycleStage::Maintenance => 0.4,  // Mock 40% complete
            LifecycleStage::Disposal => 0.9,     // Mock 90% complete
            LifecycleStage::Retired => 1.0,      // Retired is complete
        }
    }
}

#[async_trait]
impl AssetLifecycleService for DefaultAssetLifecycleService {
    async fn initialize_lifecycle(
        &self,
        asset: Asset,
        config: LifecycleConfig,
    ) -> AssetResult<Vec<AssetLifecycleEvent>> {
        info!("Initializing lifecycle for asset {}", asset.id);

        let mut events = Vec::new();

        // Create initialization event
        events.push(AssetLifecycleEvent {
            id: Uuid::new_v4(),
            asset_id: asset.id,
            event_type: "lifecycle_initialized".to_string(),
            lifecycle_stage: LifecycleStage::Registration,
            description: "Asset lifecycle initialized".to_string(),
            metadata: serde_json::json!({
                "config": config,
                "asset_type": asset.asset_type
            }),
            timestamp: Utc::now(),
            triggered_by: "system".to_string(),
        });

        // Auto-trigger verification if enabled
        if config.auto_verification {
            events.push(AssetLifecycleEvent {
                id: Uuid::new_v4(),
                asset_id: asset.id,
                event_type: "verification_scheduled".to_string(),
                lifecycle_stage: LifecycleStage::Verification,
                description: "Asset verification scheduled".to_string(),
                metadata: serde_json::json!({"auto_triggered": true}),
                timestamp: Utc::now(),
                triggered_by: "system".to_string(),
            });
        }

        // Auto-trigger valuation if enabled
        if config.auto_valuation {
            events.push(AssetLifecycleEvent {
                id: Uuid::new_v4(),
                asset_id: asset.id,
                event_type: "valuation_scheduled".to_string(),
                lifecycle_stage: LifecycleStage::Valuation,
                description: "Asset valuation scheduled".to_string(),
                metadata: serde_json::json!({"auto_triggered": true}),
                timestamp: Utc::now(),
                triggered_by: "system".to_string(),
            });
        }

        debug!("Created {} lifecycle events for asset {}", events.len(), asset.id);
        Ok(events)
    }

    async fn progress_lifecycle(
        &self,
        asset_id: Uuid,
        target_stage: LifecycleStage,
    ) -> AssetResult<Vec<AssetLifecycleEvent>> {
        info!("Progressing asset {} to stage {:?}", asset_id, target_stage);

        let mut events = Vec::new();

        // Create stage transition event
        events.push(AssetLifecycleEvent {
            id: Uuid::new_v4(),
            asset_id,
            event_type: "stage_transition".to_string(),
            lifecycle_stage: target_stage.clone(),
            description: format!("Asset progressed to {:?} stage", target_stage),
            metadata: serde_json::json!({
                "target_stage": target_stage,
                "transition_time": Utc::now()
            }),
            timestamp: Utc::now(),
            triggered_by: "system".to_string(),
        });

        debug!("Created stage transition event for asset {}", asset_id);
        Ok(events)
    }

    async fn get_lifecycle_status(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<AssetLifecycleStatus> {
        debug!("Getting lifecycle status for asset {}", asset_id);

        // Mock status
        Ok(AssetLifecycleStatus {
            asset_id,
            current_stage: LifecycleStage::Active,
            status: AssetStatus::Active,
            stage_progress: 0.75,
            next_actions: vec![
                "Schedule maintenance".to_string(),
                "Update valuation".to_string(),
            ],
            blocking_issues: vec![],
            estimated_completion: Some(Utc::now() + chrono::Duration::days(30)),
            last_updated: Utc::now(),
        })
    }

    async fn get_lifecycle_history(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<AssetLifecycleEvent>> {
        debug!("Getting lifecycle history for asset {}", asset_id);

        // Mock history
        Ok(vec![
            AssetLifecycleEvent {
                id: Uuid::new_v4(),
                asset_id,
                event_type: "lifecycle_initialized".to_string(),
                lifecycle_stage: LifecycleStage::Registration,
                description: "Asset lifecycle initialized".to_string(),
                metadata: serde_json::json!({}),
                timestamp: Utc::now() - chrono::Duration::days(90),
                triggered_by: "system".to_string(),
            },
            AssetLifecycleEvent {
                id: Uuid::new_v4(),
                asset_id,
                event_type: "verification_completed".to_string(),
                lifecycle_stage: LifecycleStage::Verification,
                description: "Asset verification completed successfully".to_string(),
                metadata: serde_json::json!({"outcome": "approved"}),
                timestamp: Utc::now() - chrono::Duration::days(60),
                triggered_by: "verifier".to_string(),
            },
        ])
    }

    async fn update_asset_status(
        &self,
        asset_id: Uuid,
        status: AssetStatus,
        reason: Option<String>,
    ) -> AssetResult<AssetLifecycleEvent> {
        info!("Updating asset {} status to {:?}", asset_id, status);

        let event = AssetLifecycleEvent {
            id: Uuid::new_v4(),
            asset_id,
            event_type: "status_updated".to_string(),
            lifecycle_stage: LifecycleStage::Active, // Assume active stage for status updates
            description: format!("Asset status updated to {:?}", status),
            metadata: serde_json::json!({
                "new_status": status,
                "reason": reason
            }),
            timestamp: Utc::now(),
            triggered_by: "system".to_string(),
        };

        debug!("Created status update event: {:?}", event);
        Ok(event)
    }

    async fn retire_asset(
        &self,
        asset_id: Uuid,
        retirement_reason: String,
    ) -> AssetResult<AssetLifecycleEvent> {
        info!("Retiring asset {} - reason: {}", asset_id, retirement_reason);

        let event = AssetLifecycleEvent {
            id: Uuid::new_v4(),
            asset_id,
            event_type: "asset_retired".to_string(),
            lifecycle_stage: LifecycleStage::Retired,
            description: "Asset retired from active service".to_string(),
            metadata: serde_json::json!({
                "retirement_reason": retirement_reason,
                "retirement_date": Utc::now()
            }),
            timestamp: Utc::now(),
            triggered_by: "system".to_string(),
        };

        debug!("Created retirement event: {:?}", event);
        Ok(event)
    }

    async fn get_lifecycle_metrics(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<LifecycleMetrics> {
        debug!("Calculating lifecycle metrics for asset {}", asset_id);

        // Mock metrics
        let mut stage_costs = std::collections::HashMap::new();
        stage_costs.insert(LifecycleStage::Registration, 5000.0);
        stage_costs.insert(LifecycleStage::Verification, 3000.0);
        stage_costs.insert(LifecycleStage::Valuation, 2000.0);
        stage_costs.insert(LifecycleStage::Active, 15000.0);
        stage_costs.insert(LifecycleStage::Maintenance, 8000.0);

        Ok(LifecycleMetrics {
            asset_id,
            total_lifecycle_days: 365,
            current_stage_days: 90,
            completion_percentage: 75.0,
            total_costs: 33000.0,
            stage_costs,
            efficiency_score: 0.85,
            risk_score: 0.15,
        })
    }
}

impl Default for DefaultAssetLifecycleService {
    fn default() -> Self {
        Self::new()
    }
}
