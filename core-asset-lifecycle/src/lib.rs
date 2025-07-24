// =====================================================================================
// File: core-asset-lifecycle/src/lib.rs
// Description: Asset lifecycle management for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Asset Lifecycle Module
//! 
//! This module provides comprehensive asset lifecycle management functionality for the
//! StableRWA platform, including asset registration, verification, valuation, and
//! maintenance throughout the asset's entire lifecycle.

pub mod registration;
pub mod verification;
pub mod valuation;
pub mod maintenance;
pub mod tokenization;
pub mod custody;
pub mod error;
pub mod types;
pub mod service;

// Re-export main types and traits
pub use error::{AssetError, AssetResult};
pub use types::{
    Asset, AssetStatus, AssetType, AssetCategory, AssetMetadata,
    AssetDocument, AssetValuation, AssetMaintenance, AssetOwnership
};
pub use service::AssetLifecycleService;
pub use registration::{AssetRegistrationService, RegistrationRequest};
pub use verification::{AssetVerificationService, VerificationResult};
pub use valuation::{AssetValuationService, ValuationMethod, ValuationRequest};
pub use maintenance::{MaintenanceService};
pub use tokenization::{TokenizationService, TokenizationConfig};
pub use custody::{CustodyService};
pub use types::{CustodyRecord, MaintenanceRecord};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;

/// Main asset lifecycle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLifecycleConfig {
    /// Registration service configuration
    pub registration_config: registration::RegistrationConfig,
    /// Verification service configuration
    pub verification_config: verification::VerificationConfig,
    /// Valuation service configuration
    pub valuation_config: valuation::ValuationConfig,
    /// Maintenance service configuration
    pub maintenance_enabled: bool,
    /// Tokenization service configuration
    pub tokenization_config: tokenization::TokenizationConfig,
    /// Custody service configuration
    pub custody_config: custody::CustodyConfig,
    /// Default asset retention period in days
    pub default_retention_days: u32,
    /// Enable automatic lifecycle transitions
    pub auto_transitions: bool,
}

impl Default for AssetLifecycleConfig {
    fn default() -> Self {
        Self {
            registration_config: registration::RegistrationConfig::default(),
            verification_config: verification::VerificationConfig::default(),
            valuation_config: valuation::ValuationConfig::default(),
            maintenance_enabled: true,
            tokenization_config: tokenization::TokenizationConfig::default(),
            custody_config: custody::CustodyConfig::default(),
            default_retention_days: 2555, // 7 years
            auto_transitions: true,
        }
    }
}

/// Asset lifecycle stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LifecycleStage {
    /// Asset is being registered
    Registration = 1,
    /// Asset is under verification
    Verification = 2,
    /// Asset is being valued
    Valuation = 3,
    /// Asset is ready for tokenization
    ReadyForTokenization = 4,
    /// Asset is being tokenized
    Tokenization = 5,
    /// Asset is actively trading
    Active = 6,
    /// Asset is under maintenance
    Maintenance = 7,
    /// Asset is being liquidated
    Liquidation = 8,
    /// Asset lifecycle is complete
    Retired = 9,
}

impl LifecycleStage {
    /// Get the next stage in the lifecycle
    pub fn next_stage(&self) -> Option<LifecycleStage> {
        match self {
            LifecycleStage::Registration => Some(LifecycleStage::Verification),
            LifecycleStage::Verification => Some(LifecycleStage::Valuation),
            LifecycleStage::Valuation => Some(LifecycleStage::ReadyForTokenization),
            LifecycleStage::ReadyForTokenization => Some(LifecycleStage::Tokenization),
            LifecycleStage::Tokenization => Some(LifecycleStage::Active),
            LifecycleStage::Active => Some(LifecycleStage::Maintenance),
            LifecycleStage::Maintenance => Some(LifecycleStage::Active),
            LifecycleStage::Liquidation => Some(LifecycleStage::Retired),
            LifecycleStage::Retired => None,
        }
    }

    /// Check if stage allows specific operations
    pub fn allows_trading(&self) -> bool {
        matches!(self, LifecycleStage::Active)
    }

    pub fn allows_valuation_update(&self) -> bool {
        matches!(
            self,
            LifecycleStage::Valuation
                | LifecycleStage::Active
                | LifecycleStage::Maintenance
        )
    }

    pub fn allows_maintenance(&self) -> bool {
        matches!(
            self,
            LifecycleStage::Active | LifecycleStage::Maintenance
        )
    }
}

/// Asset lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub event_type: LifecycleEventType,
    pub from_stage: Option<LifecycleStage>,
    pub to_stage: LifecycleStage,
    pub triggered_by: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub notes: Option<String>,
}

/// Lifecycle event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleEventType {
    StageTransition,
    StatusUpdate,
    ValuationUpdate,
    MaintenanceScheduled,
    MaintenanceCompleted,
    ComplianceCheck,
    OwnershipTransfer,
    TokenizationInitiated,
    TokenizationCompleted,
    TradingStarted,
    TradingSuspended,
    AssetRetired,
}

/// Asset lifecycle summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleSummary {
    pub asset_id: Uuid,
    pub current_stage: LifecycleStage,
    pub stage_duration_days: u32,
    pub total_lifecycle_days: u32,
    pub completion_percentage: f64,
    pub next_milestone: Option<LifecycleMilestone>,
    pub recent_events: Vec<LifecycleEvent>,
    pub health_score: f64,
}

/// Lifecycle milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleMilestone {
    pub milestone_type: MilestoneType,
    pub target_date: DateTime<Utc>,
    pub description: String,
    pub completion_criteria: Vec<String>,
}

/// Milestone type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneType {
    RegistrationComplete,
    VerificationComplete,
    ValuationComplete,
    TokenizationReady,
    TokenizationComplete,
    TradingLive,
    MaintenanceDue,
    RevaluationDue,
    ComplianceReview,
    LifecycleComplete,
}

/// Asset performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPerformanceMetrics {
    pub asset_id: Uuid,
    pub current_value: Decimal,
    pub initial_value: Decimal,
    pub value_change_percentage: Decimal,
    pub total_return: Decimal,
    pub annualized_return: Decimal,
    pub volatility: Decimal,
    pub liquidity_score: f64,
    pub maintenance_cost_ratio: Decimal,
    pub compliance_score: f64,
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_config_default() {
        let config = AssetLifecycleConfig::default();
        assert_eq!(config.default_retention_days, 2555);
        assert!(config.auto_transitions);
    }

    #[test]
    fn test_lifecycle_stage_progression() {
        let stage = LifecycleStage::Registration;
        assert_eq!(stage.next_stage(), Some(LifecycleStage::Verification));

        let final_stage = LifecycleStage::Retired;
        assert_eq!(final_stage.next_stage(), None);
    }

    #[test]
    fn test_lifecycle_stage_permissions() {
        let active_stage = LifecycleStage::Active;
        assert!(active_stage.allows_trading());
        assert!(active_stage.allows_valuation_update());
        assert!(active_stage.allows_maintenance());

        let registration_stage = LifecycleStage::Registration;
        assert!(!registration_stage.allows_trading());
        assert!(!registration_stage.allows_valuation_update());
        assert!(!registration_stage.allows_maintenance());
    }

    #[test]
    fn test_lifecycle_stage_ordering() {
        assert!(LifecycleStage::Registration < LifecycleStage::Verification);
        assert!(LifecycleStage::Verification < LifecycleStage::Valuation);
        assert!(LifecycleStage::Active > LifecycleStage::Tokenization);
    }

    #[test]
    fn test_lifecycle_event_creation() {
        let event = LifecycleEvent {
            id: Uuid::new_v4(),
            asset_id: Uuid::new_v4(),
            event_type: LifecycleEventType::StageTransition,
            from_stage: Some(LifecycleStage::Registration),
            to_stage: LifecycleStage::Verification,
            triggered_by: "system".to_string(),
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
            notes: None,
        };

        assert_eq!(event.event_type, LifecycleEventType::StageTransition);
        assert_eq!(event.to_stage, LifecycleStage::Verification);
    }

    #[test]
    fn test_lifecycle_summary_creation() {
        let summary = LifecycleSummary {
            asset_id: Uuid::new_v4(),
            current_stage: LifecycleStage::Active,
            stage_duration_days: 30,
            total_lifecycle_days: 180,
            completion_percentage: 75.0,
            next_milestone: None,
            recent_events: vec![],
            health_score: 0.85,
        };

        assert_eq!(summary.current_stage, LifecycleStage::Active);
        assert_eq!(summary.completion_percentage, 75.0);
        assert_eq!(summary.health_score, 0.85);
    }

    #[test]
    fn test_asset_performance_metrics() {
        let metrics = AssetPerformanceMetrics {
            asset_id: Uuid::new_v4(),
            current_value: Decimal::new(110000, 2), // $1,100.00
            initial_value: Decimal::new(100000, 2), // $1,000.00
            value_change_percentage: Decimal::new(1000, 2), // 10.00%
            total_return: Decimal::new(10000, 2), // $100.00
            annualized_return: Decimal::new(1200, 2), // 12.00%
            volatility: Decimal::new(500, 2), // 5.00%
            liquidity_score: 0.75,
            maintenance_cost_ratio: Decimal::new(200, 2), // 2.00%
            compliance_score: 0.95,
            last_updated: Utc::now(),
        };

        assert_eq!(metrics.current_value, Decimal::new(110000, 2));
        assert_eq!(metrics.value_change_percentage, Decimal::new(1000, 2));
        assert_eq!(metrics.liquidity_score, 0.75);
    }
}
