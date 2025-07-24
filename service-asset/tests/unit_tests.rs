// =====================================================================================
// RWA Tokenization Platform - Asset Service Unit Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use service_asset::{
    models::*,
    service::{AssetService, AssetServiceTrait, CreateAssetRequest, UpdateAssetRequest},
    AssetError,
};
use std::str::FromStr;
use uuid::Uuid;

/// Test asset creation and basic operations
#[tokio::test]
async fn test_asset_creation() {
    let repository = InMemoryAssetRepository::new();
    let cache = service_asset::cache::InMemoryCache::new();
    let service = AssetService::new(Box::new(repository), Box::new(cache));

    let request = CreateAssetRequest {
        name: "Test Property".to_string(),
        description: "A test real estate property".to_string(),
        asset_type: AssetType::RealEstate,
        total_value: dec!(500000.00),
        owner_id: Uuid::new_v4().to_string(),
        location: Some("New York, NY".to_string()),
    };

    let result = service.create_asset(request).await;
    assert!(result.is_ok());

    let asset = result.unwrap();
    assert_eq!(asset.name, "Test Property");
    assert_eq!(asset.asset_type, AssetType::RealEstate);
    assert_eq!(asset.total_value, dec!(500000.00));
    assert_eq!(asset.status, AssetStatus::Active);
    assert!(!asset.is_tokenized);
}

/// Test asset retrieval
#[tokio::test]
async fn test_asset_retrieval() {
    let repository = InMemoryAssetRepository::new();
    let cache = service_asset::cache::InMemoryCache::new();
    let service = AssetService::new(Box::new(repository), Box::new(cache));

    // Create an asset first
    let request = CreateAssetRequest {
        name: "Test Asset".to_string(),
        description: "Test description".to_string(),
        asset_type: AssetType::Commodity,
        total_value: dec!(100000.00),
        owner_id: Uuid::new_v4().to_string(),
        location: None,
    };

    let created_asset = service.create_asset(request).await.unwrap();

    // Retrieve the asset
    let retrieved = service.get_asset(&created_asset.id.to_string()).await;
    assert!(retrieved.is_ok());
    
    let asset = retrieved.unwrap();
    assert!(asset.is_some());
    
    let asset = asset.unwrap();
    assert_eq!(asset.id, created_asset.id);
    assert_eq!(asset.name, "Test Asset");
}

/// Test asset update
#[tokio::test]
async fn test_asset_update() {
    let repository = InMemoryAssetRepository::new();
    let cache = service_asset::cache::InMemoryCache::new();
    let service = AssetService::new(Box::new(repository), Box::new(cache));

    // Create an asset first
    let request = CreateAssetRequest {
        name: "Original Name".to_string(),
        description: "Original description".to_string(),
        asset_type: AssetType::Artwork,
        total_value: dec!(50000.00),
        owner_id: Uuid::new_v4().to_string(),
        location: None,
    };

    let created_asset = service.create_asset(request).await.unwrap();

    // Update the asset
    let update_request = UpdateAssetRequest {
        name: Some("Updated Name".to_string()),
        description: Some("Updated description".to_string()),
        total_value: Some(dec!(75000.00)),
        status: Some(AssetStatus::UnderReview),
    };

    let result = service.update_asset(&created_asset.id.to_string(), update_request).await;
    assert!(result.is_ok());

    let updated_asset = result.unwrap();
    assert_eq!(updated_asset.name, "Updated Name");
    assert_eq!(updated_asset.total_value, dec!(75000.00));
    assert_eq!(updated_asset.status, AssetStatus::UnderReview);
}

/// Test asset deletion
#[tokio::test]
async fn test_asset_deletion() {
    let repository = InMemoryAssetRepository::new();
    let cache = service_asset::cache::InMemoryCache::new();
    let service = AssetService::new(Box::new(repository), Box::new(cache));

    // Create an asset first
    let request = CreateAssetRequest {
        name: "To Be Deleted".to_string(),
        description: "This asset will be deleted".to_string(),
        asset_type: AssetType::Vehicle,
        total_value: dec!(25000.00),
        owner_id: Uuid::new_v4().to_string(),
        location: None,
    };

    let created_asset = service.create_asset(request).await.unwrap();
    let asset_id = created_asset.id.to_string();

    // Delete the asset
    let result = service.delete_asset(&asset_id).await;
    assert!(result.is_ok());

    // Verify it's deleted
    let retrieved = service.get_asset(&asset_id).await;
    assert!(retrieved.is_ok());
    assert!(retrieved.unwrap().is_none());
}

/// Test asset type parsing
#[test]
fn test_asset_type_parsing() {
    assert_eq!(AssetType::from_str("real_estate").unwrap(), AssetType::RealEstate);
    assert_eq!(AssetType::from_str("commodity").unwrap(), AssetType::Commodity);
    assert_eq!(AssetType::from_str("artwork").unwrap(), AssetType::Artwork);
    assert_eq!(AssetType::from_str("collectible").unwrap(), AssetType::Collectible);
    assert_eq!(AssetType::from_str("intellectual_property").unwrap(), AssetType::IntellectualProperty);
    assert_eq!(AssetType::from_str("equipment").unwrap(), AssetType::Equipment);
    assert_eq!(AssetType::from_str("vehicle").unwrap(), AssetType::Vehicle);
    assert_eq!(AssetType::from_str("other").unwrap(), AssetType::Other);
    
    assert!(AssetType::from_str("invalid").is_err());
}

/// Test asset status parsing
#[test]
fn test_asset_status_parsing() {
    assert_eq!(AssetStatus::from_str("active").unwrap(), AssetStatus::Active);
    assert_eq!(AssetStatus::from_str("inactive").unwrap(), AssetStatus::Inactive);
    assert_eq!(AssetStatus::from_str("under_review").unwrap(), AssetStatus::UnderReview);
    assert_eq!(AssetStatus::from_str("sold").unwrap(), AssetStatus::Sold);
    assert_eq!(AssetStatus::from_str("deprecated").unwrap(), AssetStatus::Deprecated);
    
    assert!(AssetStatus::from_str("invalid").is_err());
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    let repository = InMemoryAssetRepository::new();
    let cache = service_asset::cache::InMemoryCache::new();
    let service = AssetService::new(Box::new(repository), Box::new(cache));

    // Test getting non-existent asset
    let result = service.get_asset("non-existent-id").await;
    assert!(result.is_err());

    // Test updating non-existent asset
    let update_request = UpdateAssetRequest {
        name: Some("Updated".to_string()),
        description: None,
        total_value: None,
        status: None,
    };
    
    let result = service.update_asset("non-existent-id", update_request).await;
    assert!(result.is_err());

    // Test deleting non-existent asset
    let result = service.delete_asset("non-existent-id").await;
    assert!(result.is_err());
}

/// Test asset listing with filters
#[tokio::test]
async fn test_asset_listing_with_filters() {
    let repository = InMemoryAssetRepository::new();
    let cache = service_asset::cache::InMemoryCache::new();
    let service = AssetService::new(Box::new(repository), Box::new(cache));

    let owner_id = Uuid::new_v4();

    // Create multiple assets
    for i in 0..5 {
        let request = CreateAssetRequest {
            name: format!("Asset {}", i),
            description: format!("Description {}", i),
            asset_type: if i % 2 == 0 { AssetType::RealEstate } else { AssetType::Commodity },
            total_value: dec!(100000.00) * Decimal::from(i + 1),
            owner_id: owner_id.to_string(),
            location: None,
        };
        service.create_asset(request).await.unwrap();
    }

    // Test listing with pagination
    let pagination = service_asset::service::Pagination { page: 1, per_page: 3 };
    let filters = service_asset::service::AssetFilters {
        asset_type: None,
        owner_id: Some(owner_id),
        status: None,
        min_value: None,
        max_value: None,
    };

    let result = service.list_assets(pagination, filters).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.data.len(), 3);
    assert_eq!(response.pagination.total_count, 5);
    assert_eq!(response.pagination.total_pages, 2);
}
