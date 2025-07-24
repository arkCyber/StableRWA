// =====================================================================================
// File: core-stablecoin/src/issuance.rs
// Description: Token issuance and minting functionality
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

use crate::{StablecoinResult, StablecoinError};
use crate::types::{IssuanceRequest, Stablecoin, CollateralPosition, CollateralStatus};

/// Issuance transaction record
#[derive(Debug, Clone)]
pub struct IssuanceTransaction {
    pub id: Uuid,
    pub request_id: Uuid,
    pub tx_hash: String,
    pub amount: Decimal,
    pub fees: Decimal,
    pub collateral_locked: Vec<CollateralPosition>,
    pub status: IssuanceStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Issuance status
#[derive(Debug, Clone, PartialEq)]
pub enum IssuanceStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Issuance service trait
#[async_trait]
pub trait IssuanceService: Send + Sync {
    /// Process issuance request
    async fn process_issuance(&self, request: IssuanceRequest) -> StablecoinResult<IssuanceTransaction>;

    /// Validate issuance request
    async fn validate_issuance(&self, request: &IssuanceRequest) -> StablecoinResult<bool>;

    /// Calculate issuance fees
    async fn calculate_fees(&self, amount: Decimal) -> StablecoinResult<Decimal>;

    /// Get issuance transaction by ID
    async fn get_transaction(&self, tx_id: Uuid) -> StablecoinResult<Option<IssuanceTransaction>>;

    /// Get user's issuance history
    async fn get_user_history(&self, user_id: &str, limit: Option<usize>) -> StablecoinResult<Vec<IssuanceTransaction>>;

    /// Cancel pending issuance
    async fn cancel_issuance(&self, tx_id: Uuid) -> StablecoinResult<bool>;

    /// Get total issued amount for a stablecoin
    async fn get_total_issued(&self, stablecoin_id: Uuid) -> StablecoinResult<Decimal>;
}

/// Issuance manager implementation
pub struct IssuanceManager {
    fee_rate: Decimal,
    min_issuance_amount: Decimal,
    max_issuance_amount: Decimal,
    transactions: Arc<Mutex<HashMap<Uuid, IssuanceTransaction>>>,
    user_transactions: Arc<Mutex<HashMap<String, Vec<Uuid>>>>,
    stablecoin_totals: Arc<Mutex<HashMap<Uuid, Decimal>>>,
}

impl IssuanceManager {
    pub fn new() -> Self {
        Self {
            fee_rate: Decimal::new(10, 4), // 0.1%
            min_issuance_amount: Decimal::new(1, 0), // 1 token minimum
            max_issuance_amount: Decimal::new(1_000_000, 0), // 1M token maximum
            transactions: Arc::new(Mutex::new(HashMap::new())),
            user_transactions: Arc::new(Mutex::new(HashMap::new())),
            stablecoin_totals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_config(
        fee_rate: Decimal,
        min_amount: Decimal,
        max_amount: Decimal,
    ) -> Self {
        Self {
            fee_rate,
            min_issuance_amount: min_amount,
            max_issuance_amount: max_amount,
            transactions: Arc::new(Mutex::new(HashMap::new())),
            user_transactions: Arc::new(Mutex::new(HashMap::new())),
            stablecoin_totals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Validate collateral requirements
    async fn validate_collateral(&self, request: &IssuanceRequest) -> StablecoinResult<bool> {
        if request.collateral.is_empty() {
            return Err(StablecoinError::InsufficientCollateral);
        }

        let total_collateral_value: Decimal = request.collateral
            .iter()
            .filter(|c| c.status == CollateralStatus::Active)
            .map(|c| c.value_usd)
            .sum();

        // Require 150% collateralization ratio
        let required_collateral = request.amount * Decimal::new(15, 1); // 1.5x

        if total_collateral_value < required_collateral {
            return Err(StablecoinError::InsufficientCollateral);
        }

        Ok(true)
    }

    /// Lock collateral for issuance
    async fn lock_collateral(&self, collateral: &mut Vec<CollateralPosition>) -> StablecoinResult<()> {
        for position in collateral.iter_mut() {
            if position.status == CollateralStatus::Active {
                position.status = CollateralStatus::Locked;
                position.locked_until = Some(Utc::now() + chrono::Duration::hours(24));
            }
        }
        Ok(())
    }
}

impl Default for IssuanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IssuanceService for IssuanceManager {
    async fn process_issuance(&self, mut request: IssuanceRequest) -> StablecoinResult<IssuanceTransaction> {
        // Validate the request
        self.validate_issuance(&request).await?;

        // Validate and lock collateral
        self.validate_collateral(&request).await?;
        self.lock_collateral(&mut request.collateral).await?;

        // Calculate fees
        let fees = self.calculate_fees(request.amount).await?;

        // Create transaction record
        let tx_id = Uuid::new_v4();
        let tx_hash = format!("issuance_tx_{}", tx_id.simple());

        let transaction = IssuanceTransaction {
            id: tx_id,
            request_id: request.id,
            tx_hash: tx_hash.clone(),
            amount: request.amount,
            fees,
            collateral_locked: request.collateral.clone(),
            status: IssuanceStatus::Processing,
            created_at: Utc::now(),
            completed_at: None,
        };

        // Store transaction
        {
            let mut transactions = self.transactions.lock().await;
            transactions.insert(tx_id, transaction.clone());
        }

        // Update user transaction history
        {
            let mut user_txs = self.user_transactions.lock().await;
            user_txs.entry(request.user_id.clone())
                .or_insert_with(Vec::new)
                .push(tx_id);
        }

        // Update total issued amount
        {
            let mut totals = self.stablecoin_totals.lock().await;
            let current_total = totals.get(&request.stablecoin_id).unwrap_or(&Decimal::ZERO).clone();
            totals.insert(request.stablecoin_id, current_total + request.amount);
        }

        // Simulate processing completion
        let mut completed_tx = transaction.clone();
        completed_tx.status = IssuanceStatus::Completed;
        completed_tx.completed_at = Some(Utc::now());

        // Update stored transaction
        {
            let mut transactions = self.transactions.lock().await;
            transactions.insert(tx_id, completed_tx.clone());
        }

        Ok(completed_tx)
    }

    async fn validate_issuance(&self, request: &IssuanceRequest) -> StablecoinResult<bool> {
        // Check amount bounds
        if request.amount < self.min_issuance_amount {
            return Err(StablecoinError::InvalidAmount("Amount below minimum".to_string()));
        }

        if request.amount > self.max_issuance_amount {
            return Err(StablecoinError::InvalidAmount("Amount exceeds maximum".to_string()));
        }

        // Check if user ID is valid
        if request.user_id.is_empty() {
            return Err(StablecoinError::InvalidRequest("User ID cannot be empty".to_string()));
        }

        // Check if recipient address is valid (basic check)
        if request.recipient_address.is_empty() || !request.recipient_address.starts_with("0x") {
            return Err(StablecoinError::InvalidRequest("Invalid recipient address".to_string()));
        }

        Ok(true)
    }

    async fn calculate_fees(&self, amount: Decimal) -> StablecoinResult<Decimal> {
        Ok(amount * self.fee_rate)
    }

    async fn get_transaction(&self, tx_id: Uuid) -> StablecoinResult<Option<IssuanceTransaction>> {
        let transactions = self.transactions.lock().await;
        Ok(transactions.get(&tx_id).cloned())
    }

    async fn get_user_history(&self, user_id: &str, limit: Option<usize>) -> StablecoinResult<Vec<IssuanceTransaction>> {
        let user_txs = self.user_transactions.lock().await;
        let transactions = self.transactions.lock().await;

        if let Some(tx_ids) = user_txs.get(user_id) {
            let mut user_transactions: Vec<IssuanceTransaction> = tx_ids
                .iter()
                .filter_map(|id| transactions.get(id).cloned())
                .collect();

            // Sort by creation time (newest first)
            user_transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

            // Apply limit if specified
            if let Some(limit) = limit {
                user_transactions.truncate(limit);
            }

            Ok(user_transactions)
        } else {
            Ok(vec![])
        }
    }

    async fn cancel_issuance(&self, tx_id: Uuid) -> StablecoinResult<bool> {
        let mut transactions = self.transactions.lock().await;

        if let Some(transaction) = transactions.get_mut(&tx_id) {
            match transaction.status {
                IssuanceStatus::Pending | IssuanceStatus::Processing => {
                    transaction.status = IssuanceStatus::Cancelled;

                    // Update total issued amount (subtract back)
                    let mut totals = self.stablecoin_totals.lock().await;
                    if let Some(stablecoin_id) = transactions.values()
                        .find(|tx| tx.id == tx_id)
                        .map(|tx| tx.request_id) // This is a simplification
                    {
                        // In a real implementation, we'd need to track stablecoin_id in the transaction
                        // For now, we'll just mark as cancelled
                    }

                    Ok(true)
                }
                _ => Err(StablecoinError::InvalidRequest("Cannot cancel completed or failed transaction".to_string()))
            }
        } else {
            Err(StablecoinError::TransactionNotFound)
        }
    }

    async fn get_total_issued(&self, stablecoin_id: Uuid) -> StablecoinResult<Decimal> {
        let totals = self.stablecoin_totals.lock().await;
        Ok(totals.get(&stablecoin_id).unwrap_or(&Decimal::ZERO).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::types::{CollateralPosition, CollateralType, CollateralStatus};

    #[tokio::test]
    async fn test_issuance_manager_creation() {
        let manager = IssuanceManager::new();

        // Test default values
        assert_eq!(manager.fee_rate, Decimal::new(10, 4)); // 0.1%
        assert_eq!(manager.min_issuance_amount, Decimal::new(1, 0));
        assert_eq!(manager.max_issuance_amount, Decimal::new(1_000_000, 0));
    }

    #[tokio::test]
    async fn test_issuance_manager_with_config() {
        let manager = IssuanceManager::with_config(
            Decimal::new(5, 4), // 0.05%
            Decimal::new(10, 0), // 10 minimum
            Decimal::new(500_000, 0), // 500k maximum
        );

        assert_eq!(manager.fee_rate, Decimal::new(5, 4));
        assert_eq!(manager.min_issuance_amount, Decimal::new(10, 0));
        assert_eq!(manager.max_issuance_amount, Decimal::new(500_000, 0));
    }

    #[tokio::test]
    async fn test_issuance_manager_default() {
        let manager = IssuanceManager::default();

        // Test that default() creates the same as new()
        let new_manager = IssuanceManager::new();
        assert_eq!(manager.fee_rate, new_manager.fee_rate);
        assert_eq!(manager.min_issuance_amount, new_manager.min_issuance_amount);
        assert_eq!(manager.max_issuance_amount, new_manager.max_issuance_amount);
    }

    #[tokio::test]
    async fn test_process_issuance_basic() {
        let manager = IssuanceManager::new();

        let collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(1500, 0), // $1500 collateral
            value_usd: Decimal::new(1500, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(1000, 0), // $1000 issuance
            collateral: vec![collateral],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.process_issuance(request).await.unwrap();
        assert_eq!(result.status, IssuanceStatus::Completed);
        assert_eq!(result.amount, Decimal::new(1000, 0));
        assert!(!result.tx_hash.is_empty());
        assert!(result.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_process_issuance_with_collateral() {
        let manager = IssuanceManager::new();

        let collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(1000, 0),
            value_usd: Decimal::new(1000, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user456".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(500, 0),
            collateral: vec![collateral],
            network_id: 137, // Polygon
            recipient_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.process_issuance(request).await.unwrap();
        assert!(!result.tx_hash.is_empty());
    }

    #[tokio::test]
    async fn test_validate_issuance() {
        let manager = IssuanceManager::new();

        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user789".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(2000, 0),
            collateral: vec![],
            network_id: 56, // BSC
            recipient_address: "0x789".to_string(),
            created_at: Utc::now(),
        };

        let is_valid = manager.validate_issuance(&request).await.unwrap();
        assert!(is_valid); // Mock implementation always returns true
    }

    #[tokio::test]
    async fn test_calculate_fees_various_amounts() {
        let manager = IssuanceManager::new();

        // Test different amounts
        let test_cases = vec![
            (Decimal::new(1000, 0), Decimal::new(1, 0)),     // 1000 * 0.1% = 1
            (Decimal::new(10000, 0), Decimal::new(10, 0)),   // 10000 * 0.1% = 10
            (Decimal::new(100, 0), Decimal::new(1, 1)),      // 100 * 0.1% = 0.1
            (Decimal::new(1, 0), Decimal::new(1, 3)),        // 1 * 0.1% = 0.001
            (Decimal::ZERO, Decimal::ZERO),                  // 0 * 0.1% = 0
        ];

        for (amount, expected_fee) in test_cases {
            let fee = manager.calculate_fees(amount).await.unwrap();
            assert_eq!(fee, expected_fee, "Fee calculation failed for amount {}", amount);
        }
    }

    #[tokio::test]
    async fn test_calculate_fees_precision() {
        let manager = IssuanceManager::new();

        // Test precision with decimal amounts
        let amount = Decimal::new(123456, 2); // 1234.56
        let fee = manager.calculate_fees(amount).await.unwrap();
        let expected = amount * manager.fee_rate;
        assert_eq!(fee, expected);
    }

    #[tokio::test]
    async fn test_multiple_concurrent_issuances() {
        let manager = std::sync::Arc::new(IssuanceManager::new());
        let mut handles = vec![];

        // Create 5 concurrent issuance requests
        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let collateral = CollateralPosition {
                    id: Uuid::new_v4(),
                    collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
                    amount: Decimal::new(2000 + i as i64 * 200, 0), // Sufficient collateral
                    value_usd: Decimal::new(2000 + i as i64 * 200, 0),
                    locked_until: None,
                    status: CollateralStatus::Active,
                };

                let request = IssuanceRequest {
                    id: Uuid::new_v4(),
                    user_id: format!("user{}", i),
                    stablecoin_id: Uuid::new_v4(),
                    amount: Decimal::new(1000 + i as i64 * 100, 0),
                    collateral: vec![collateral],
                    network_id: 1,
                    recipient_address: format!("0x{:x}", i),
                    created_at: Utc::now(),
                };

                manager_clone.process_issuance(request).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // Verify all succeeded
        for result in results {
            let transaction = result.unwrap().unwrap();
            assert!(!transaction.tx_hash.is_empty());
        }
    }

    #[tokio::test]
    async fn test_issuance_request_validation_edge_cases() {
        let manager = IssuanceManager::new();

        // Test with very large amount (should fail validation)
        let large_request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "whale_user".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(1_000_000_000, 0), // 1 billion
            collateral: vec![],
            network_id: 1,
            recipient_address: "0xwhale".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_issuance(&large_request).await;
        assert!(result.is_err());

        // Test with very small amount (should fail validation)
        let small_request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "small_user".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(1, 6), // 0.000001
            collateral: vec![],
            network_id: 1,
            recipient_address: "0xsmall".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_issuance(&small_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_issuance_with_different_networks() {
        let manager = IssuanceManager::new();

        let networks = vec![1, 137, 56, 42161]; // Ethereum, Polygon, BSC, Arbitrum

        for network_id in networks {
            let collateral = CollateralPosition {
                id: Uuid::new_v4(),
                collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
                amount: Decimal::new(1500, 0), // $1500 collateral for $1000 issuance
                value_usd: Decimal::new(1500, 0),
                locked_until: None,
                status: CollateralStatus::Active,
            };

            let request = IssuanceRequest {
                id: Uuid::new_v4(),
                user_id: format!("user_network_{}", network_id),
                stablecoin_id: Uuid::new_v4(),
                amount: Decimal::new(1000, 0),
                collateral: vec![collateral],
                network_id,
                recipient_address: format!("0x{:x}", network_id),
                created_at: Utc::now(),
            };

            let result = manager.process_issuance(request).await.unwrap();
            assert!(!result.tx_hash.is_empty());
        }
    }

    #[tokio::test]
    async fn test_issuance_with_multiple_collateral_types() {
        let manager = IssuanceManager::new();

        let collaterals = vec![
            CollateralPosition {
                id: Uuid::new_v4(),
                collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
                amount: Decimal::new(1000, 0), // Increased to $1000
                value_usd: Decimal::new(1000, 0),
                locked_until: None,
                status: CollateralStatus::Active,
            },
            CollateralPosition {
                id: Uuid::new_v4(),
                collateral_type: CollateralType::Crypto {
                    token_address: "0xA0b86a33E6441e6e80D0c4C34F0b0B2e0c2D0e0f".to_string(),
                    symbol: "ETH".to_string()
                },
                amount: Decimal::new(1, 0), // 1 ETH
                value_usd: Decimal::new(2500, 0), // Increased to $2500
                locked_until: None,
                status: CollateralStatus::Active,
            },
        ];

        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "multi_collateral_user".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(2000, 0), // Total collateral: $3500, required: $3000 (150%)
            collateral: collaterals,
            network_id: 1,
            recipient_address: "0xmulti".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.process_issuance(request).await.unwrap();
        assert!(!result.tx_hash.is_empty());
    }

    #[tokio::test]
    async fn test_fee_calculation_consistency() {
        let manager = IssuanceManager::new();

        // Test that fee calculation is consistent across multiple calls
        let amount = Decimal::new(1000, 0);

        let fee1 = manager.calculate_fees(amount).await.unwrap();
        let fee2 = manager.calculate_fees(amount).await.unwrap();
        let fee3 = manager.calculate_fees(amount).await.unwrap();

        assert_eq!(fee1, fee2);
        assert_eq!(fee2, fee3);
        assert_eq!(fee1, Decimal::new(1, 0)); // 0.1% of 1000
    }

    #[tokio::test]
    async fn test_insufficient_collateral() {
        let manager = IssuanceManager::new();

        let insufficient_collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(100, 0), // Only $100 collateral
            value_usd: Decimal::new(100, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(1000, 0), // $1000 issuance (needs $1500 collateral)
            collateral: vec![insufficient_collateral],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.process_issuance(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StablecoinError::InsufficientCollateral));
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let manager = IssuanceManager::new();

        // Test amount too small
        let small_request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(5, 1), // 0.5 (below minimum of 1)
            collateral: vec![],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_issuance(&small_request).await;
        assert!(result.is_err());

        // Test amount too large
        let large_request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(2_000_000, 0), // 2M (above maximum of 1M)
            collateral: vec![],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_issuance(&large_request).await;
        assert!(result.is_err());

        // Test empty user ID
        let empty_user_request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(100, 0),
            collateral: vec![],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_issuance(&empty_user_request).await;
        assert!(result.is_err());

        // Test invalid recipient address
        let invalid_address_request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(100, 0),
            collateral: vec![],
            network_id: 1,
            recipient_address: "invalid_address".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_issuance(&invalid_address_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transaction_tracking() {
        let manager = IssuanceManager::new();

        let collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(1500, 0),
            value_usd: Decimal::new(1500, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        let stablecoin_id = Uuid::new_v4();
        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id,
            amount: Decimal::new(1000, 0),
            collateral: vec![collateral],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        // Process issuance
        let transaction = manager.process_issuance(request).await.unwrap();

        // Test getting transaction by ID
        let retrieved_tx = manager.get_transaction(transaction.id).await.unwrap();
        assert!(retrieved_tx.is_some());
        assert_eq!(retrieved_tx.unwrap().id, transaction.id);

        // Test user history
        let history = manager.get_user_history("user123", None).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, transaction.id);

        // Test total issued amount
        let total = manager.get_total_issued(stablecoin_id).await.unwrap();
        assert_eq!(total, Decimal::new(1000, 0));
    }

    #[tokio::test]
    async fn test_collateral_locking() {
        let manager = IssuanceManager::new();

        let mut collateral = vec![CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Crypto {
                token_address: "0xA0b86a33E6441e6e80D0c4C34F0b0B2e0c2D0e0f".to_string(),
                symbol: "ETH".to_string()
            },
            amount: Decimal::new(1, 0),
            value_usd: Decimal::new(2000, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        }];

        // Test collateral locking
        manager.lock_collateral(&mut collateral).await.unwrap();

        assert_eq!(collateral[0].status, CollateralStatus::Locked);
        assert!(collateral[0].locked_until.is_some());
    }
}
