// =====================================================================================
// File: core-asset-lifecycle/src/maintenance.rs
// Description: Asset maintenance and lifecycle management
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    error::{AssetError, AssetResult},
    types::{Asset, AssetType, MaintenanceRecord, MaintenanceType, MaintenanceStatus},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug};
use uuid::Uuid;

/// Maintenance schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceSchedule {
    pub asset_id: Uuid,
    pub maintenance_type: MaintenanceType,
    pub frequency_days: u32,
    pub next_due_date: DateTime<Utc>,
    pub priority: MaintenancePriority,
    pub estimated_cost: Option<f64>,
    pub estimated_duration_hours: Option<u32>,
}

/// Maintenance priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaintenancePriority {
    Low,
    Medium,
    High,
    Critical,
    Emergency,
}

/// Maintenance request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRequest {
    pub asset_id: Uuid,
    pub maintenance_type: MaintenanceType,
    pub priority: MaintenancePriority,
    pub description: String,
    pub requested_by: String,
    pub requested_date: DateTime<Utc>,
    pub preferred_date: Option<DateTime<Utc>>,
    pub estimated_cost: Option<f64>,
    pub vendor_id: Option<String>,
}

/// Asset maintenance service trait
#[async_trait]
pub trait MaintenanceService: Send + Sync {
    /// Schedule maintenance for an asset
    async fn schedule_maintenance(
        &self,
        request: MaintenanceRequest,
    ) -> AssetResult<MaintenanceRecord>;

    /// Get maintenance schedule for an asset
    async fn get_maintenance_schedule(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<MaintenanceSchedule>>;

    /// Update maintenance record
    async fn update_maintenance_record(
        &self,
        record_id: Uuid,
        status: MaintenanceStatus,
        notes: Option<String>,
    ) -> AssetResult<MaintenanceRecord>;

    /// Get maintenance history for an asset
    async fn get_maintenance_history(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<MaintenanceRecord>>;

    /// Get overdue maintenance items
    async fn get_overdue_maintenance(&self) -> AssetResult<Vec<MaintenanceSchedule>>;

    /// Calculate maintenance costs
    async fn calculate_maintenance_costs(
        &self,
        asset_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> AssetResult<f64>;
}

/// Default maintenance service implementation
pub struct DefaultMaintenanceService {
    schedules: HashMap<Uuid, Vec<MaintenanceSchedule>>,
    records: HashMap<Uuid, MaintenanceRecord>,
}

impl DefaultMaintenanceService {
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
            records: HashMap::new(),
        }
    }

    /// Generate maintenance schedule based on asset type
    fn generate_default_schedule(&self, asset: &Asset) -> Vec<MaintenanceSchedule> {
        let mut schedules = Vec::new();
        let now = Utc::now();

        match asset.asset_type {
            AssetType::RealEstate => {
                schedules.push(MaintenanceSchedule {
                    asset_id: asset.id,
                    maintenance_type: MaintenanceType::Inspection,
                    frequency_days: 365, // Annual inspection
                    next_due_date: now + chrono::Duration::days(365),
                    priority: MaintenancePriority::Medium,
                    estimated_cost: Some(5000.0),
                    estimated_duration_hours: Some(8),
                });
            }
            AssetType::Equipment => {
                schedules.push(MaintenanceSchedule {
                    asset_id: asset.id,
                    maintenance_type: MaintenanceType::Preventive,
                    frequency_days: 90, // Quarterly maintenance
                    next_due_date: now + chrono::Duration::days(90),
                    priority: MaintenancePriority::High,
                    estimated_cost: Some(2000.0),
                    estimated_duration_hours: Some(4),
                });
            }
            AssetType::Vehicles => {
                schedules.push(MaintenanceSchedule {
                    asset_id: asset.id,
                    maintenance_type: MaintenanceType::Preventive,
                    frequency_days: 180, // Semi-annual maintenance
                    next_due_date: now + chrono::Duration::days(180),
                    priority: MaintenancePriority::High,
                    estimated_cost: Some(1500.0),
                    estimated_duration_hours: Some(6),
                });
            }
            _ => {
                // Default schedule for other asset types
                schedules.push(MaintenanceSchedule {
                    asset_id: asset.id,
                    maintenance_type: MaintenanceType::Inspection,
                    frequency_days: 365,
                    next_due_date: now + chrono::Duration::days(365),
                    priority: MaintenancePriority::Low,
                    estimated_cost: Some(1000.0),
                    estimated_duration_hours: Some(2),
                });
            }
        }

        schedules
    }
}

#[async_trait]
impl MaintenanceService for DefaultMaintenanceService {
    async fn schedule_maintenance(
        &self,
        request: MaintenanceRequest,
    ) -> AssetResult<MaintenanceRecord> {
        info!("Scheduling maintenance for asset {}", request.asset_id);

        let record = MaintenanceRecord {
            id: Uuid::new_v4(),
            asset_id: request.asset_id,
            maintenance_type: request.maintenance_type,
            status: MaintenanceStatus::Scheduled,
            scheduled_date: request.preferred_date.unwrap_or_else(|| Utc::now() + chrono::Duration::days(7)),
            completed_date: None,
            description: request.description,
            performed_by: None,
            cost: request.estimated_cost.map(|c| rust_decimal::Decimal::from_f64_retain(c).unwrap_or_default()),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        debug!("Created maintenance record: {:?}", record);
        Ok(record)
    }

    async fn get_maintenance_schedule(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<MaintenanceSchedule>> {
        debug!("Getting maintenance schedule for asset {}", asset_id);
        
        // Mock schedule
        Ok(vec![MaintenanceSchedule {
            asset_id,
            maintenance_type: MaintenanceType::Preventive,
            frequency_days: 90,
            next_due_date: Utc::now() + chrono::Duration::days(30),
            priority: MaintenancePriority::Medium,
            estimated_cost: Some(2500.0),
            estimated_duration_hours: Some(4),
        }])
    }

    async fn update_maintenance_record(
        &self,
        record_id: Uuid,
        status: MaintenanceStatus,
        notes: Option<String>,
    ) -> AssetResult<MaintenanceRecord> {
        info!("Updating maintenance record {} to status {:?}", record_id, status);

        // Mock updated record
        Ok(MaintenanceRecord {
            id: record_id,
            asset_id: Uuid::new_v4(),
            maintenance_type: MaintenanceType::Preventive,
            status,
            scheduled_date: Utc::now(),
            completed_date: if matches!(status, MaintenanceStatus::Completed) {
                Some(Utc::now())
            } else {
                None
            },
            description: "Updated maintenance record".to_string(),
            performed_by: Some("Maintenance Team".to_string()),
            cost: Some(rust_decimal::Decimal::from_f64_retain(2500.0).unwrap_or_default()),
            notes,
            created_at: Utc::now() - chrono::Duration::days(1),
            updated_at: Utc::now(),
        })
    }

    async fn get_maintenance_history(
        &self,
        asset_id: Uuid,
    ) -> AssetResult<Vec<MaintenanceRecord>> {
        debug!("Getting maintenance history for asset {}", asset_id);
        
        // Mock history
        Ok(vec![
            MaintenanceRecord {
                id: Uuid::new_v4(),
                asset_id,
                maintenance_type: MaintenanceType::Preventive,
                status: MaintenanceStatus::Completed,
                scheduled_date: Utc::now() - chrono::Duration::days(90),
                completed_date: Some(Utc::now() - chrono::Duration::days(88)),
                description: "Quarterly preventive maintenance".to_string(),
                performed_by: Some("Maintenance Team A".to_string()),
                cost: Some(rust_decimal::Decimal::from_f64_retain(2200.0).unwrap_or_default()),
                notes: Some("All systems checked and serviced".to_string()),
                created_at: Utc::now() - chrono::Duration::days(95),
                updated_at: Utc::now() - chrono::Duration::days(88),
            }
        ])
    }

    async fn get_overdue_maintenance(&self) -> AssetResult<Vec<MaintenanceSchedule>> {
        debug!("Getting overdue maintenance items");
        
        // Mock overdue items
        Ok(vec![])
    }

    async fn calculate_maintenance_costs(
        &self,
        asset_id: Uuid,
        _period_start: DateTime<Utc>,
        _period_end: DateTime<Utc>,
    ) -> AssetResult<f64> {
        debug!("Calculating maintenance costs for asset {}", asset_id);
        
        // Mock calculation
        Ok(15000.0)
    }
}

impl Default for DefaultMaintenanceService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schedule_maintenance() {
        let service = DefaultMaintenanceService::new();
        let request = MaintenanceRequest {
            asset_id: Uuid::new_v4(),
            maintenance_type: MaintenanceType::Preventive,
            priority: MaintenancePriority::High,
            description: "Test maintenance".to_string(),
            requested_by: "test_user".to_string(),
            requested_date: Utc::now(),
            preferred_date: None,
            estimated_cost: Some(1000.0),
            vendor_id: None,
        };

        let result = service.schedule_maintenance(request).await;
        assert!(result.is_ok());
        
        let record = result.unwrap();
        assert_eq!(record.status, MaintenanceStatus::Scheduled);
    }

    #[tokio::test]
    async fn test_get_maintenance_schedule() {
        let service = DefaultMaintenanceService::new();
        let asset_id = Uuid::new_v4();
        
        let result = service.get_maintenance_schedule(asset_id).await;
        assert!(result.is_ok());
        
        let schedules = result.unwrap();
        assert!(!schedules.is_empty());
    }
}
