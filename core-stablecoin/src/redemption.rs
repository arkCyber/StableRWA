// =====================================================================================
// File: core-stablecoin/src/redemption.rs
// Description: Token redemption and burning functionality
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
use crate::types::{RedemptionRequest, CollateralPosition, CollateralType, CollateralStatus};

/// Redemption transaction record
#[derive(Debug, Clone)]
pub struct RedemptionTransaction {
    pub id: Uuid,
    pub request_id: Uuid,
    pub tx_hash: String,
    pub amount: Decimal,
    pub fees: Decimal,
    pub collateral_released: Vec<CollateralPosition>,
    pub status: RedemptionStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Redemption status
#[derive(Debug, Clone, PartialEq)]
pub enum RedemptionStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Redemption service trait
#[async_trait]
pub trait RedemptionService: Send + Sync {
    /// Process redemption request
    async fn process_redemption(&self, request: RedemptionRequest) -> StablecoinResult<RedemptionTransaction>;

    /// Validate redemption request
    async fn validate_redemption(&self, request: &RedemptionRequest) -> StablecoinResult<bool>;

    /// Calculate redemption fees
    async fn calculate_fees(&self, amount: Decimal) -> StablecoinResult<Decimal>;

    /// Get redemption transaction by ID
    async fn get_transaction(&self, tx_id: Uuid) -> StablecoinResult<Option<RedemptionTransaction>>;

    /// Get user's redemption history
    async fn get_user_history(&self, user_id: &str, limit: Option<usize>) -> StablecoinResult<Vec<RedemptionTransaction>>;

    /// Cancel pending redemption
    async fn cancel_redemption(&self, tx_id: Uuid) -> StablecoinResult<bool>;

    /// Get total redeemed amount for a stablecoin
    async fn get_total_redeemed(&self, stablecoin_id: Uuid) -> StablecoinResult<Decimal>;
}

/// Redemption manager implementation
pub struct RedemptionManager {
    fee_rate: Decimal,
    min_redemption_amount: Decimal,
    max_redemption_amount: Decimal,
    transactions: Arc<Mutex<HashMap<Uuid, RedemptionTransaction>>>,
    user_transactions: Arc<Mutex<HashMap<String, Vec<Uuid>>>>,
    stablecoin_totals: Arc<Mutex<HashMap<Uuid, Decimal>>>,
    available_collateral: Arc<Mutex<HashMap<Uuid, Vec<CollateralPosition>>>>,
}

impl RedemptionManager {
    pub fn new() -> Self {
        Self {
            fee_rate: Decimal::new(10, 4), // 0.1%
            min_redemption_amount: Decimal::new(1, 0), // 1 token minimum
            max_redemption_amount: Decimal::new(1_000_000, 0), // 1M token maximum
            transactions: Arc::new(Mutex::new(HashMap::new())),
            user_transactions: Arc::new(Mutex::new(HashMap::new())),
            stablecoin_totals: Arc::new(Mutex::new(HashMap::new())),
            available_collateral: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_config(
        fee_rate: Decimal,
        min_amount: Decimal,
        max_amount: Decimal,
    ) -> Self {
        Self {
            fee_rate,
            min_redemption_amount: min_amount,
            max_redemption_amount: max_amount,
            transactions: Arc::new(Mutex::new(HashMap::new())),
            user_transactions: Arc::new(Mutex::new(HashMap::new())),
            stablecoin_totals: Arc::new(Mutex::new(HashMap::new())),
            available_collateral: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add collateral to the pool for redemptions
    pub async fn add_collateral(&self, stablecoin_id: Uuid, collateral: CollateralPosition) -> StablecoinResult<()> {
        let mut available = self.available_collateral.lock().await;
        available.entry(stablecoin_id)
            .or_insert_with(Vec::new)
            .push(collateral);
        Ok(())
    }

    /// Select collateral for redemption based on preference
    async fn select_collateral(
        &self,
        stablecoin_id: Uuid,
        amount: Decimal,
        preferred_type: Option<CollateralType>,
    ) -> StablecoinResult<Vec<CollateralPosition>> {
        let mut available = self.available_collateral.lock().await;
        let collateral_pool = available.get_mut(&stablecoin_id)
            .ok_or(StablecoinError::InsufficientCollateral)?;

        let mut selected = Vec::new();
        let mut remaining_amount = amount;

        // First, try to use preferred collateral type if specified
        if let Some(preferred) = preferred_type {
            for collateral in collateral_pool.iter_mut() {
                if remaining_amount <= Decimal::ZERO {
                    break;
                }

                if std::mem::discriminant(&collateral.collateral_type) == std::mem::discriminant(&preferred)
                    && collateral.status == CollateralStatus::Active {
                    let available_value = collateral.value_usd.min(remaining_amount);
                    if available_value > Decimal::ZERO {
                        let mut selected_collateral = collateral.clone();
                        selected_collateral.value_usd = available_value;
                        selected_collateral.status = CollateralStatus::Locked;
                        selected.push(selected_collateral);

                        collateral.value_usd -= available_value;
                        remaining_amount -= available_value;

                        if collateral.value_usd <= Decimal::ZERO {
                            collateral.status = CollateralStatus::Used;
                        }
                    }
                }
            }
        }

        // If still need more collateral, use any available
        for collateral in collateral_pool.iter_mut() {
            if remaining_amount <= Decimal::ZERO {
                break;
            }

            if collateral.status == CollateralStatus::Active {
                let available_value = collateral.value_usd.min(remaining_amount);
                if available_value > Decimal::ZERO {
                    let mut selected_collateral = collateral.clone();
                    selected_collateral.value_usd = available_value;
                    selected_collateral.status = CollateralStatus::Locked;
                    selected.push(selected_collateral);

                    collateral.value_usd -= available_value;
                    remaining_amount -= available_value;

                    if collateral.value_usd <= Decimal::ZERO {
                        collateral.status = CollateralStatus::Used;
                    }
                }
            }
        }

        if remaining_amount > Decimal::ZERO {
            return Err(StablecoinError::InsufficientCollateral);
        }

        Ok(selected)
    }
}

impl Default for RedemptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RedemptionService for RedemptionManager {
    async fn process_redemption(&self, request: RedemptionRequest) -> StablecoinResult<RedemptionTransaction> {
        // Validate the request
        self.validate_redemption(&request).await?;

        // Calculate fees
        let fees = self.calculate_fees(request.amount).await?;

        // Select collateral for redemption
        let collateral_released = self.select_collateral(
            request.stablecoin_id,
            request.amount,
            request.preferred_collateral.clone(),
        ).await?;

        // Create transaction record
        let tx_id = Uuid::new_v4();
        let tx_hash = format!("redemption_tx_{}", tx_id.simple());

        let transaction = RedemptionTransaction {
            id: tx_id,
            request_id: request.id,
            tx_hash: tx_hash.clone(),
            amount: request.amount,
            fees,
            collateral_released: collateral_released.clone(),
            status: RedemptionStatus::Processing,
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

        // Update total redeemed amount
        {
            let mut totals = self.stablecoin_totals.lock().await;
            let current_total = totals.get(&request.stablecoin_id).unwrap_or(&Decimal::ZERO).clone();
            totals.insert(request.stablecoin_id, current_total + request.amount);
        }

        // Simulate processing completion
        let mut completed_tx = transaction.clone();
        completed_tx.status = RedemptionStatus::Completed;
        completed_tx.completed_at = Some(Utc::now());

        // Update stored transaction
        {
            let mut transactions = self.transactions.lock().await;
            transactions.insert(tx_id, completed_tx.clone());
        }

        Ok(completed_tx)
    }

    async fn validate_redemption(&self, request: &RedemptionRequest) -> StablecoinResult<bool> {
        // Check amount bounds
        if request.amount < self.min_redemption_amount {
            return Err(StablecoinError::InvalidAmount("Amount below minimum".to_string()));
        }

        if request.amount > self.max_redemption_amount {
            return Err(StablecoinError::InvalidAmount("Amount exceeds maximum".to_string()));
        }

        // Check if user ID is valid
        if request.user_id.is_empty() {
            return Err(StablecoinError::InvalidRequest("User ID cannot be empty".to_string()));
        }

        // Check if sender address is valid (basic check)
        if request.sender_address.is_empty() || !request.sender_address.starts_with("0x") {
            return Err(StablecoinError::InvalidRequest("Invalid sender address".to_string()));
        }

        Ok(true)
    }

    async fn calculate_fees(&self, amount: Decimal) -> StablecoinResult<Decimal> {
        Ok(amount * self.fee_rate)
    }

    async fn get_transaction(&self, tx_id: Uuid) -> StablecoinResult<Option<RedemptionTransaction>> {
        let transactions = self.transactions.lock().await;
        Ok(transactions.get(&tx_id).cloned())
    }

    async fn get_user_history(&self, user_id: &str, limit: Option<usize>) -> StablecoinResult<Vec<RedemptionTransaction>> {
        let user_txs = self.user_transactions.lock().await;
        let transactions = self.transactions.lock().await;

        if let Some(tx_ids) = user_txs.get(user_id) {
            let mut user_transactions: Vec<RedemptionTransaction> = tx_ids
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

    async fn cancel_redemption(&self, tx_id: Uuid) -> StablecoinResult<bool> {
        let mut transactions = self.transactions.lock().await;

        if let Some(transaction) = transactions.get_mut(&tx_id) {
            match transaction.status {
                RedemptionStatus::Pending | RedemptionStatus::Processing => {
                    transaction.status = RedemptionStatus::Cancelled;
                    Ok(true)
                }
                _ => Err(StablecoinError::InvalidRequest("Cannot cancel completed or failed transaction".to_string()))
            }
        } else {
            Err(StablecoinError::TransactionNotFound)
        }
    }

    async fn get_total_redeemed(&self, stablecoin_id: Uuid) -> StablecoinResult<Decimal> {
        let totals = self.stablecoin_totals.lock().await;
        Ok(totals.get(&stablecoin_id).unwrap_or(&Decimal::ZERO).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    #[tokio::test]
    async fn test_redemption_manager_creation() {
        let manager = RedemptionManager::new();

        assert_eq!(manager.fee_rate, Decimal::new(10, 4)); // 0.1%
        assert_eq!(manager.min_redemption_amount, Decimal::new(1, 0));
        assert_eq!(manager.max_redemption_amount, Decimal::new(1_000_000, 0));
    }

    #[tokio::test]
    async fn test_redemption_manager_with_config() {
        let manager = RedemptionManager::with_config(
            Decimal::new(5, 4), // 0.05%
            Decimal::new(10, 0), // 10 minimum
            Decimal::new(500_000, 0), // 500k maximum
        );

        assert_eq!(manager.fee_rate, Decimal::new(5, 4));
        assert_eq!(manager.min_redemption_amount, Decimal::new(10, 0));
        assert_eq!(manager.max_redemption_amount, Decimal::new(500_000, 0));
    }

    #[tokio::test]
    async fn test_add_collateral() {
        let manager = RedemptionManager::new();
        let stablecoin_id = Uuid::new_v4();

        let collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(1000, 0),
            value_usd: Decimal::new(1000, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        manager.add_collateral(stablecoin_id, collateral).await.unwrap();

        // Verify collateral was added
        let available = manager.available_collateral.lock().await;
        assert!(available.contains_key(&stablecoin_id));
        assert_eq!(available[&stablecoin_id].len(), 1);
    }

    #[tokio::test]
    async fn test_process_redemption_basic() {
        let manager = RedemptionManager::new();
        let stablecoin_id = Uuid::new_v4();

        // Add collateral first
        let collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(1000, 0),
            value_usd: Decimal::new(1000, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        manager.add_collateral(stablecoin_id, collateral).await.unwrap();

        let request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id,
            amount: Decimal::new(500, 0),
            preferred_collateral: None,
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.process_redemption(request).await.unwrap();
        assert_eq!(result.status, RedemptionStatus::Completed);
        assert_eq!(result.amount, Decimal::new(500, 0));
        assert!(!result.tx_hash.is_empty());
        assert!(result.completed_at.is_some());
        assert!(!result.collateral_released.is_empty());

        let fees = manager.calculate_fees(Decimal::new(500, 0)).await.unwrap();
        assert_eq!(fees, Decimal::new(5, 1)); // 0.1% of 500 = 0.5
    }

    #[tokio::test]
    async fn test_redemption_with_preferred_collateral() {
        let manager = RedemptionManager::new();
        let stablecoin_id = Uuid::new_v4();

        // Add different types of collateral
        let fiat_collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(500, 0),
            value_usd: Decimal::new(500, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        let crypto_collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Crypto {
                token_address: "0xA0b86a33E6441e6e80D0c4C34F0b0B2e0c2D0e0f".to_string(),
                symbol: "ETH".to_string()
            },
            amount: Decimal::new(1, 0),
            value_usd: Decimal::new(2000, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        manager.add_collateral(stablecoin_id, fiat_collateral).await.unwrap();
        manager.add_collateral(stablecoin_id, crypto_collateral).await.unwrap();

        let request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id,
            amount: Decimal::new(300, 0),
            preferred_collateral: Some(CollateralType::Fiat { currency: "USD".to_string() }),
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.process_redemption(request).await.unwrap();
        assert_eq!(result.status, RedemptionStatus::Completed);

        // Should have used fiat collateral first
        let fiat_used = result.collateral_released.iter()
            .any(|c| matches!(c.collateral_type, CollateralType::Fiat { .. }));
        assert!(fiat_used);
    }

    #[tokio::test]
    async fn test_insufficient_collateral() {
        let manager = RedemptionManager::new();
        let stablecoin_id = Uuid::new_v4();

        // Add insufficient collateral
        let collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(100, 0),
            value_usd: Decimal::new(100, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        manager.add_collateral(stablecoin_id, collateral).await.unwrap();

        let request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id,
            amount: Decimal::new(500, 0), // More than available collateral
            preferred_collateral: None,
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.process_redemption(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StablecoinError::InsufficientCollateral));
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let manager = RedemptionManager::new();

        // Test amount too small
        let small_request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(5, 1), // 0.5 (below minimum of 1)
            preferred_collateral: None,
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_redemption(&small_request).await;
        assert!(result.is_err());

        // Test amount too large
        let large_request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(2_000_000, 0), // 2M (above maximum of 1M)
            preferred_collateral: None,
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_redemption(&large_request).await;
        assert!(result.is_err());

        // Test empty user ID
        let empty_user_request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(100, 0),
            preferred_collateral: None,
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_redemption(&empty_user_request).await;
        assert!(result.is_err());

        // Test invalid sender address
        let invalid_address_request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(100, 0),
            preferred_collateral: None,
            network_id: 1,
            sender_address: "invalid_address".to_string(),
            created_at: Utc::now(),
        };

        let result = manager.validate_redemption(&invalid_address_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transaction_tracking() {
        let manager = RedemptionManager::new();
        let stablecoin_id = Uuid::new_v4();

        // Add collateral
        let collateral = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(1000, 0),
            value_usd: Decimal::new(1000, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        manager.add_collateral(stablecoin_id, collateral).await.unwrap();

        let request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id,
            amount: Decimal::new(500, 0),
            preferred_collateral: None,
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        // Process redemption
        let transaction = manager.process_redemption(request).await.unwrap();

        // Test getting transaction by ID
        let retrieved_tx = manager.get_transaction(transaction.id).await.unwrap();
        assert!(retrieved_tx.is_some());
        assert_eq!(retrieved_tx.unwrap().id, transaction.id);

        // Test user history
        let history = manager.get_user_history("user123", None).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, transaction.id);

        // Test total redeemed amount
        let total = manager.get_total_redeemed(stablecoin_id).await.unwrap();
        assert_eq!(total, Decimal::new(500, 0));
    }

    #[tokio::test]
    async fn test_cancel_redemption() {
        let manager = RedemptionManager::new();
        let tx_id = Uuid::new_v4();

        // Create a pending transaction manually for testing
        let transaction = RedemptionTransaction {
            id: tx_id,
            request_id: Uuid::new_v4(),
            tx_hash: "test_tx".to_string(),
            amount: Decimal::new(100, 0),
            fees: Decimal::new(1, 1),
            collateral_released: vec![],
            status: RedemptionStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
        };

        {
            let mut transactions = manager.transactions.lock().await;
            transactions.insert(tx_id, transaction);
        }

        // Test cancellation
        let result = manager.cancel_redemption(tx_id).await.unwrap();
        assert!(result);

        // Verify status changed
        let updated_tx = manager.get_transaction(tx_id).await.unwrap().unwrap();
        assert_eq!(updated_tx.status, RedemptionStatus::Cancelled);
    }
}
