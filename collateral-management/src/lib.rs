// =====================================================================================
// File: collateral-management/src/lib.rs
// Description: Core collateral management logic for enterprise-grade microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Collateral asset structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collateral {
    pub id: u64,
    pub owner: String,
    pub asset_type: String,
    pub value: f64,
    pub registered_at: String,
    pub released: bool,
}

/// Collateral management error type
#[derive(Debug, Error)]
pub enum CollateralError {
    #[error("Owner address is empty")]
    EmptyOwner,
    #[error("Asset type is empty")]
    EmptyAssetType,
    #[error("Collateral not found")]
    NotFound,
    #[error("Collateral already released")]
    AlreadyReleased,
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Collateral storage (in-memory, for demo; replace with DB in production)
#[derive(Default, Clone)]
pub struct CollateralStore {
    pub collaterals: Arc<Mutex<HashMap<u64, Collateral>>>,
    pub next_id: Arc<Mutex<u64>>,
}

impl CollateralStore {
    /// Creates a new collateral store
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new collateral
    pub fn register(&self, owner: String, asset_type: String, value: f64) -> Result<Collateral, CollateralError> {
        let now = Utc::now();
        if owner.trim().is_empty() {
            error!("{} - [CollateralStore] Register failed: owner empty", now);
            return Err(CollateralError::EmptyOwner);
        }
        if asset_type.trim().is_empty() {
            error!("{} - [CollateralStore] Register failed: asset_type empty", now);
            return Err(CollateralError::EmptyAssetType);
        }
        let mut id_lock = self.next_id.lock().unwrap();
        let id = *id_lock;
        *id_lock += 1;
        let collateral = Collateral {
            id,
            owner: owner.clone(),
            asset_type: asset_type.clone(),
            value,
            registered_at: now.to_rfc3339(),
            released: false,
        };
        self.collaterals.lock().unwrap().insert(id, collateral.clone());
        info!("{} - [CollateralStore] Registered collateral id={} owner={}", now, id, owner);
        Ok(collateral)
    }

    /// Updates the value of a collateral
    pub fn update_value(&self, id: u64, value: f64) -> Result<Collateral, CollateralError> {
        let now = Utc::now();
        let mut collaterals = self.collaterals.lock().unwrap();
        let collateral = collaterals.get_mut(&id).ok_or(CollateralError::NotFound)?;
        if collateral.released {
            error!("{} - [CollateralStore] Update failed: already released id={}", now, id);
            return Err(CollateralError::AlreadyReleased);
        }
        collateral.value = value;
        info!("{} - [CollateralStore] Updated value id={} value={}", now, id, value);
        Ok(collateral.clone())
    }

    /// Releases a collateral
    pub fn release(&self, id: u64) -> Result<Collateral, CollateralError> {
        let now = Utc::now();
        let mut collaterals = self.collaterals.lock().unwrap();
        let collateral = collaterals.get_mut(&id).ok_or(CollateralError::NotFound)?;
        if collateral.released {
            error!("{} - [CollateralStore] Release failed: already released id={}", now, id);
            return Err(CollateralError::AlreadyReleased);
        }
        collateral.released = true;
        info!("{} - [CollateralStore] Released collateral id={}", now, id);
        Ok(collateral.clone())
    }

    /// Gets a collateral by id
    pub fn get(&self, id: u64) -> Result<Collateral, CollateralError> {
        let now = Utc::now();
        let collaterals = self.collaterals.lock().unwrap();
        collaterals.get(&id).cloned().ok_or_else(|| {
            error!("{} - [CollateralStore] Collateral not found: {}", now, id);
            CollateralError::NotFound
        })
    }

    /// Lists all collaterals for an owner
    pub fn list_by_owner(&self, owner: &str) -> Vec<Collateral> {
        let collaterals = self.collaterals.lock().unwrap();
        collaterals.values().filter(|c| c.owner == owner).cloned().collect()
    }

    /// Lists all collaterals
    pub fn list_all(&self) -> Vec<Collateral> {
        self.collaterals.lock().unwrap().values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Info).try_init();
    }

    #[test]
    fn test_register_and_get() {
        init_logger();
        let store = CollateralStore::new();
        let c = store.register("0xabc".to_string(), "RealEstate".to_string(), 1000.0).unwrap();
        let fetched = store.get(c.id).unwrap();
        assert_eq!(c.id, fetched.id);
        assert_eq!(c.owner, fetched.owner);
    }

    #[test]
    fn test_update_and_release() {
        init_logger();
        let store = CollateralStore::new();
        let c = store.register("0xabc".to_string(), "Gold".to_string(), 500.0).unwrap();
        let updated = store.update_value(c.id, 600.0).unwrap();
        assert_eq!(updated.value, 600.0);
        let released = store.release(c.id).unwrap();
        assert!(released.released);
        assert!(store.update_value(c.id, 700.0).is_err());
        assert!(store.release(c.id).is_err());
    }

    #[test]
    fn test_list_by_owner() {
        init_logger();
        let store = CollateralStore::new();
        store.register("0xabc".to_string(), "Car".to_string(), 200.0).unwrap();
        store.register("0xabc".to_string(), "Land".to_string(), 300.0).unwrap();
        store.register("0xdef".to_string(), "Art".to_string(), 400.0).unwrap();
        let list = store.list_by_owner("0xabc");
        assert_eq!(list.len(), 2);
    }
} 