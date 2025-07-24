// =====================================================================================
// File: core-stablecoin/src/service.rs
// Description: High-level service interfaces for stablecoin operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::{StablecoinError, StablecoinResult};
use crate::types::{Stablecoin, StablecoinConfig, IssuanceRequest, RedemptionRequest};

/// Main stablecoin service trait
#[async_trait]
pub trait StablecoinService: Send + Sync {
    /// Create a new stablecoin
    async fn create_stablecoin(
        &self,
        symbol: String,
        name: String,
        stability_mechanism: crate::types::StabilityMechanism,
    ) -> StablecoinResult<Stablecoin>;

    /// Get stablecoin by ID
    async fn get_stablecoin(&self, id: Uuid) -> StablecoinResult<Option<Stablecoin>>;

    /// Issue new tokens
    async fn issue_tokens(&self, request: IssuanceRequest) -> StablecoinResult<String>;

    /// Redeem tokens
    async fn redeem_tokens(&self, request: RedemptionRequest) -> StablecoinResult<String>;

    /// Get stablecoin price
    async fn get_price(&self, stablecoin_id: Uuid) -> StablecoinResult<rust_decimal::Decimal>;
}

/// Default implementation of stablecoin service
pub struct StablecoinServiceImpl {
    config: StablecoinConfig,
    stablecoins: Mutex<HashMap<Uuid, Stablecoin>>,
}

impl StablecoinServiceImpl {
    /// Create new service instance
    pub async fn new(config: StablecoinConfig) -> StablecoinResult<Self> {
        Ok(Self {
            config,
            stablecoins: Mutex::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl StablecoinService for StablecoinServiceImpl {
    async fn create_stablecoin(
        &self,
        symbol: String,
        name: String,
        stability_mechanism: crate::types::StabilityMechanism,
    ) -> StablecoinResult<Stablecoin> {
        let id = Uuid::new_v4();
        let stablecoin = Stablecoin {
            id,
            symbol,
            name,
            decimals: 18,
            stability_mechanism,
            target_price: rust_decimal::Decimal::ONE,
            current_price: rust_decimal::Decimal::ONE,
            total_supply: rust_decimal::Decimal::ZERO,
            total_collateral_value: rust_decimal::Decimal::ZERO,
            collateral_ratio: rust_decimal::Decimal::ZERO,
            supported_collateral: vec![],
            contract_addresses: HashMap::new(),
            stability_parameters: crate::types::StabilityParameters::default(),
            status: crate::types::StablecoinStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Store the stablecoin in the internal map
        {
            let mut stablecoins = self.stablecoins.lock().unwrap();
            stablecoins.insert(id, stablecoin.clone());
        }

        Ok(stablecoin)
    }

    async fn get_stablecoin(&self, id: Uuid) -> StablecoinResult<Option<Stablecoin>> {
        let stablecoins = self.stablecoins.lock().unwrap();
        Ok(stablecoins.get(&id).cloned())
    }

    async fn issue_tokens(&self, _request: IssuanceRequest) -> StablecoinResult<String> {
        // Mock implementation
        Ok("tx_hash_123".to_string())
    }

    async fn redeem_tokens(&self, _request: RedemptionRequest) -> StablecoinResult<String> {
        // Mock implementation
        Ok("tx_hash_456".to_string())
    }

    async fn get_price(&self, _stablecoin_id: Uuid) -> StablecoinResult<rust_decimal::Decimal> {
        // Mock implementation - return $1.00
        Ok(rust_decimal::Decimal::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_create_stablecoin_service() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        let stablecoin = service.create_stablecoin(
            "RWAUSD".to_string(),
            "RWA USD Stablecoin".to_string(),
            crate::types::StabilityMechanism::RWABacked,
        ).await.unwrap();

        assert_eq!(stablecoin.symbol, "RWAUSD");
        assert_eq!(stablecoin.name, "RWA USD Stablecoin");
        assert_eq!(stablecoin.stability_mechanism, crate::types::StabilityMechanism::RWABacked);
        assert_eq!(stablecoin.decimals, 18);
        assert_eq!(stablecoin.target_price, rust_decimal::Decimal::ONE);
        assert_eq!(stablecoin.current_price, rust_decimal::Decimal::ONE);
        assert_eq!(stablecoin.total_supply, rust_decimal::Decimal::ZERO);
        assert_eq!(stablecoin.status, crate::types::StablecoinStatus::Active);
        assert!(!stablecoin.id.is_nil());
    }

    #[tokio::test]
    async fn test_create_multiple_stablecoins() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        // Create first stablecoin
        let stablecoin1 = service.create_stablecoin(
            "RWAUSD".to_string(),
            "RWA USD Stablecoin".to_string(),
            crate::types::StabilityMechanism::RWABacked,
        ).await.unwrap();

        // Create second stablecoin
        let stablecoin2 = service.create_stablecoin(
            "RWAEUR".to_string(),
            "RWA EUR Stablecoin".to_string(),
            crate::types::StabilityMechanism::Fiat,
        ).await.unwrap();

        // Verify they are different
        assert_ne!(stablecoin1.id, stablecoin2.id);
        assert_ne!(stablecoin1.symbol, stablecoin2.symbol);
        assert_ne!(stablecoin1.stability_mechanism, stablecoin2.stability_mechanism);
    }

    #[tokio::test]
    async fn test_get_stablecoin() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        // Create a stablecoin
        let created = service.create_stablecoin(
            "RWAUSD".to_string(),
            "RWA USD Stablecoin".to_string(),
            crate::types::StabilityMechanism::RWABacked,
        ).await.unwrap();

        // Retrieve it
        let retrieved = service.get_stablecoin(created.id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.symbol, created.symbol);
        assert_eq!(retrieved.name, created.name);
    }

    #[tokio::test]
    async fn test_get_nonexistent_stablecoin() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        let nonexistent_id = Uuid::new_v4();
        let result = service.get_stablecoin(nonexistent_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_issue_tokens() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: rust_decimal::Decimal::new(1000, 0),
            collateral: vec![],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        let tx_hash = service.issue_tokens(request).await.unwrap();
        assert!(!tx_hash.is_empty());
        assert_eq!(tx_hash, "tx_hash_123"); // Mock implementation returns this
    }

    #[tokio::test]
    async fn test_redeem_tokens() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        let request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user456".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: rust_decimal::Decimal::new(500, 0),
            preferred_collateral: None,
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let tx_hash = service.redeem_tokens(request).await.unwrap();
        assert!(!tx_hash.is_empty());
        assert_eq!(tx_hash, "tx_hash_456"); // Mock implementation returns this
    }

    #[tokio::test]
    async fn test_get_price() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        let stablecoin_id = Uuid::new_v4();
        let price = service.get_price(stablecoin_id).await.unwrap();
        assert_eq!(price, rust_decimal::Decimal::ONE); // Mock returns $1.00
    }

    #[tokio::test]
    async fn test_service_creation_with_custom_config() {
        let mut config = StablecoinConfig::default();
        config.enable_governance = false;
        config.enable_compliance = false;
        config.default_stability_mechanism = crate::types::StabilityMechanism::Crypto;

        let service = StablecoinServiceImpl::new(config.clone()).await.unwrap();

        // Verify the service was created with the custom config
        // Note: In a real implementation, we might expose config getters
        let stablecoin = service.create_stablecoin(
            "TESTCOIN".to_string(),
            "Test Coin".to_string(),
            config.default_stability_mechanism,
        ).await.unwrap();

        assert_eq!(stablecoin.stability_mechanism, crate::types::StabilityMechanism::Crypto);
    }

    #[tokio::test]
    async fn test_all_stability_mechanisms() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        let mechanisms = vec![
            crate::types::StabilityMechanism::Fiat,
            crate::types::StabilityMechanism::Crypto,
            crate::types::StabilityMechanism::Algorithmic,
            crate::types::StabilityMechanism::Hybrid,
            crate::types::StabilityMechanism::RWABacked,
        ];

        for (i, mechanism) in mechanisms.iter().enumerate() {
            let stablecoin = service.create_stablecoin(
                format!("TEST{}", i),
                format!("Test Coin {}", i),
                *mechanism,
            ).await.unwrap();

            assert_eq!(stablecoin.stability_mechanism, *mechanism);
            assert_eq!(stablecoin.symbol, format!("TEST{}", i));
        }
    }

    #[tokio::test]
    async fn test_concurrent_stablecoin_creation() {
        let config = StablecoinConfig::default();
        let service = std::sync::Arc::new(StablecoinServiceImpl::new(config).await.unwrap());

        let mut handles = vec![];

        // Create 10 stablecoins concurrently
        for i in 0..10 {
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                service_clone.create_stablecoin(
                    format!("CONCURRENT{}", i),
                    format!("Concurrent Coin {}", i),
                    crate::types::StabilityMechanism::RWABacked,
                ).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // Verify all succeeded and have unique IDs
        let mut ids = std::collections::HashSet::new();
        for result in results {
            let stablecoin = result.unwrap().unwrap();
            assert!(ids.insert(stablecoin.id), "Duplicate ID found: {}", stablecoin.id);
        }

        assert_eq!(ids.len(), 10);
    }

    #[tokio::test]
    async fn test_stablecoin_default_values() {
        let config = StablecoinConfig::default();
        let service = StablecoinServiceImpl::new(config).await.unwrap();

        let stablecoin = service.create_stablecoin(
            "DEFAULT".to_string(),
            "Default Coin".to_string(),
            crate::types::StabilityMechanism::RWABacked,
        ).await.unwrap();

        // Test all default values
        assert_eq!(stablecoin.decimals, 18);
        assert_eq!(stablecoin.target_price, rust_decimal::Decimal::ONE);
        assert_eq!(stablecoin.current_price, rust_decimal::Decimal::ONE);
        assert_eq!(stablecoin.total_supply, rust_decimal::Decimal::ZERO);
        assert_eq!(stablecoin.total_collateral_value, rust_decimal::Decimal::ZERO);
        assert_eq!(stablecoin.collateral_ratio, rust_decimal::Decimal::ZERO);
        assert!(stablecoin.supported_collateral.is_empty());
        assert!(stablecoin.contract_addresses.is_empty());
        assert_eq!(stablecoin.status, crate::types::StablecoinStatus::Active);

        // Verify timestamps are recent (within last minute)
        let now = Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);
        assert!(stablecoin.created_at > one_minute_ago);
        assert!(stablecoin.updated_at > one_minute_ago);
        assert!(stablecoin.created_at <= now);
        assert!(stablecoin.updated_at <= now);
    }
}
