// =====================================================================================
// File: service-asset/src/lib.rs
// Description: Asset Service library for RWA Platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod handlers;
pub mod models;
pub mod service;

use core_utils::config::Config;
use core_asset_lifecycle::AssetManager;
use service::AssetService;
use std::sync::Arc;
use thiserror::Error;

/// Asset service specific errors
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    #[error("Invalid asset type: {0}")]
    InvalidAssetType(String),
    #[error("Asset already tokenized")]
    AssetAlreadyTokenized,
    #[error("Invalid asset state: {0}")]
    InvalidAssetState(String),
    #[error("Blockchain error: {0}")]
    BlockchainError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sqlx::Error> for AssetError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AssetError::AssetNotFound("Asset not found".to_string()),
            _ => AssetError::DatabaseError(err.to_string()),
        }
    }
}

impl From<core_blockchain::BlockchainError> for AssetError {
    fn from(err: core_blockchain::BlockchainError) -> Self {
        AssetError::BlockchainError(err.to_string())
    }
}

/// Application state for the asset service
pub struct AppState {
    pub config: Config,
    pub asset_service: Arc<AssetService>,
    pub asset_manager: Arc<AssetManager>,
}

/// Asset service result type
pub type AssetResult<T> = Result<T, AssetError>;

/// Asset type definitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetType {
    RealEstate,
    Commodity,
    Artwork,
    Collectible,
    Vehicle,
    Equipment,
    Intellectual,
    Other,
}

impl AssetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::RealEstate => "real_estate",
            AssetType::Commodity => "commodity",
            AssetType::Artwork => "artwork",
            AssetType::Collectible => "collectible",
            AssetType::Vehicle => "vehicle",
            AssetType::Equipment => "equipment",
            AssetType::Intellectual => "intellectual",
            AssetType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "real_estate" => Some(AssetType::RealEstate),
            "commodity" => Some(AssetType::Commodity),
            "artwork" => Some(AssetType::Artwork),
            "collectible" => Some(AssetType::Collectible),
            "vehicle" => Some(AssetType::Vehicle),
            "equipment" => Some(AssetType::Equipment),
            "intellectual" => Some(AssetType::Intellectual),
            "other" => Some(AssetType::Other),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            AssetType::RealEstate,
            AssetType::Commodity,
            AssetType::Artwork,
            AssetType::Collectible,
            AssetType::Vehicle,
            AssetType::Equipment,
            AssetType::Intellectual,
            AssetType::Other,
        ]
    }
}

/// Validation functions
pub fn validate_asset_name(name: &str) -> AssetResult<()> {
    if name.trim().is_empty() {
        return Err(AssetError::ValidationError("Asset name cannot be empty".to_string()));
    }

    if name.len() > 200 {
        return Err(AssetError::ValidationError("Asset name is too long (max 200 characters)".to_string()));
    }

    Ok(())
}

pub fn validate_asset_description(description: &str) -> AssetResult<()> {
    if description.trim().is_empty() {
        return Err(AssetError::ValidationError("Asset description cannot be empty".to_string()));
    }

    if description.len() > 2000 {
        return Err(AssetError::ValidationError("Asset description is too long (max 2000 characters)".to_string()));
    }

    Ok(())
}

pub fn validate_asset_type(asset_type: &str) -> AssetResult<()> {
    if AssetType::from_str(asset_type).is_none() {
        return Err(AssetError::InvalidAssetType(asset_type.to_string()));
    }

    Ok(())
}

pub fn validate_asset_value(value: rust_decimal::Decimal) -> AssetResult<()> {
    if value <= rust_decimal::Decimal::ZERO {
        return Err(AssetError::ValidationError("Asset value must be positive".to_string()));
    }

    if value > rust_decimal::Decimal::from(1_000_000_000_000i64) {
        return Err(AssetError::ValidationError("Asset value is too large".to_string()));
    }

    Ok(())
}

pub fn validate_currency(currency: &str) -> AssetResult<()> {
    let valid_currencies = ["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "BTC", "ETH"];

    if !valid_currencies.contains(&currency.to_uppercase().as_str()) {
        return Err(AssetError::ValidationError(format!("Unsupported currency: {}", currency)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_conversion() {
        assert_eq!(AssetType::RealEstate.as_str(), "real_estate");
        assert_eq!(AssetType::from_str("real_estate"), Some(AssetType::RealEstate));
        assert_eq!(AssetType::from_str("invalid"), None);
    }

    #[test]
    fn test_asset_name_validation() {
        assert!(validate_asset_name("Valid Asset Name").is_ok());
        assert!(validate_asset_name("").is_err());
        assert!(validate_asset_name("   ").is_err());
        assert!(validate_asset_name(&"a".repeat(201)).is_err());
    }

    #[test]
    fn test_asset_value_validation() {
        use rust_decimal_macros::dec;

        assert!(validate_asset_value(dec!(100.00)).is_ok());
        assert!(validate_asset_value(dec!(0.01)).is_ok());
        assert!(validate_asset_value(dec!(0.00)).is_err());
        assert!(validate_asset_value(dec!(-100.00)).is_err());
    }

    #[test]
    fn test_currency_validation() {
        assert!(validate_currency("USD").is_ok());
        assert!(validate_currency("usd").is_ok());
        assert!(validate_currency("EUR").is_ok());
        assert!(validate_currency("BTC").is_ok());
        assert!(validate_currency("INVALID").is_err());
    }
}
