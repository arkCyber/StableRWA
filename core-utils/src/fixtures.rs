// =====================================================================================
// File: core-utils/src/fixtures.rs
// Description: Test data fixtures and generators for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use fake::Fake;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// User test fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFixture {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserFixture {
    /// Generate a random user fixture
    pub fn generate() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email: fake::faker::internet::en::SafeEmail().fake(),
            first_name: fake::faker::name::en::FirstName().fake(),
            last_name: fake::faker::name::en::LastName().fake(),
            password_hash: "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uIoO"
                .to_string(), // "password"
            is_active: true,
            is_verified: fake::faker::boolean::en::Boolean(70).fake(), // 70% chance of being verified
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate a user with specific email
    pub fn with_email(email: &str) -> Self {
        let mut user = Self::generate();
        user.email = email.to_string();
        user
    }

    /// Generate an admin user
    pub fn admin() -> Self {
        let mut user = Self::generate();
        user.email = "admin@rwa-platform.com".to_string();
        user.first_name = "Admin".to_string();
        user.last_name = "User".to_string();
        user.is_verified = true;
        user
    }

    /// Generate multiple users
    pub fn generate_many(count: usize) -> Vec<Self> {
        (0..count).map(|_| Self::generate()).collect()
    }
}

/// Asset test fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetFixture {
    pub id: String,
    pub name: String,
    pub description: String,
    pub asset_type: String,
    pub total_value: f64,
    pub currency: String,
    pub owner_id: String,
    pub is_tokenized: bool,
    pub token_address: Option<String>,
    pub blockchain_network: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AssetFixture {
    /// Generate a random asset fixture
    pub fn generate() -> Self {
        let now = Utc::now();
        let asset_types = ["real_estate", "art", "collectible", "commodity", "vehicle"];
        let currencies = ["USD", "EUR", "GBP", "JPY"];
        let networks = ["ethereum", "solana", "polkadot"];

        let mut rng = rand::thread_rng();
        let is_tokenized = rng.gen_bool(0.3); // 30% chance of being tokenized

        let mut metadata = HashMap::new();
        metadata.insert(
            "category".to_string(),
            serde_json::Value::String("investment".to_string()),
        );
        metadata.insert(
            "risk_level".to_string(),
            serde_json::Value::String("medium".to_string()),
        );

        Self {
            id: Uuid::new_v4().to_string(),
            name: fake::faker::company::en::CompanyName().fake(),
            description: fake::faker::lorem::en::Paragraph(3..5).fake(),
            asset_type: asset_types[rng.gen_range(0..asset_types.len())].to_string(),
            total_value: rng.gen_range(10000.0..10000000.0),
            currency: currencies[rng.gen_range(0..currencies.len())].to_string(),
            owner_id: Uuid::new_v4().to_string(),
            is_tokenized,
            token_address: if is_tokenized {
                Some(format!(
                    "0x{}",
                    fake::faker::lorem::en::Word().fake::<String>().repeat(8)
                ))
            } else {
                None
            },
            blockchain_network: if is_tokenized {
                Some(networks[rng.gen_range(0..networks.len())].to_string())
            } else {
                None
            },
            metadata,
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate an asset for a specific owner
    pub fn for_owner(owner_id: &str) -> Self {
        let mut asset = Self::generate();
        asset.owner_id = owner_id.to_string();
        asset
    }

    /// Generate a tokenized asset
    pub fn tokenized() -> Self {
        let mut asset = Self::generate();
        asset.is_tokenized = true;
        asset.token_address = Some(format!(
            "0x{}",
            fake::faker::lorem::en::Word().fake::<String>().repeat(8)
        ));
        asset.blockchain_network = Some("ethereum".to_string());
        asset
    }

    /// Generate multiple assets
    pub fn generate_many(count: usize) -> Vec<Self> {
        (0..count).map(|_| Self::generate()).collect()
    }
}

/// Transaction test fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionFixture {
    pub id: String,
    pub transaction_hash: String,
    pub blockchain_network: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub fee: f64,
    pub status: String,
    pub block_number: Option<u64>,
    pub confirmations: u32,
    pub asset_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TransactionFixture {
    /// Generate a random transaction fixture
    pub fn generate() -> Self {
        let now = Utc::now();
        let networks = ["ethereum", "solana", "polkadot"];
        let statuses = ["pending", "confirmed", "failed"];

        let mut rng = rand::thread_rng();
        let status = statuses[rng.gen_range(0..statuses.len())];

        let mut metadata = HashMap::new();
        metadata.insert(
            "gas_used".to_string(),
            serde_json::Value::Number(serde_json::Number::from(21000)),
        );

        Self {
            id: Uuid::new_v4().to_string(),
            transaction_hash: format!(
                "0x{}",
                fake::faker::lorem::en::Word().fake::<String>().repeat(16)
            ),
            blockchain_network: networks[rng.gen_range(0..networks.len())].to_string(),
            from_address: format!(
                "0x{}",
                fake::faker::lorem::en::Word().fake::<String>().repeat(8)
            ),
            to_address: format!(
                "0x{}",
                fake::faker::lorem::en::Word().fake::<String>().repeat(8)
            ),
            amount: rng.gen_range(0.001..1000.0),
            fee: rng.gen_range(0.0001..0.1),
            status: status.to_string(),
            block_number: if status == "confirmed" {
                Some(rng.gen_range(1000000..2000000))
            } else {
                None
            },
            confirmations: if status == "confirmed" {
                rng.gen_range(1..100)
            } else {
                0
            },
            asset_id: if rng.gen_bool(0.5) {
                Some(Uuid::new_v4().to_string())
            } else {
                None
            },
            user_id: if rng.gen_bool(0.8) {
                Some(Uuid::new_v4().to_string())
            } else {
                None
            },
            metadata,
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate a confirmed transaction
    pub fn confirmed() -> Self {
        let mut tx = Self::generate();
        tx.status = "confirmed".to_string();
        tx.block_number = Some(rand::thread_rng().gen_range(1000000..2000000));
        tx.confirmations = rand::thread_rng().gen_range(6..100);
        tx
    }

    /// Generate a pending transaction
    pub fn pending() -> Self {
        let mut tx = Self::generate();
        tx.status = "pending".to_string();
        tx.block_number = None;
        tx.confirmations = 0;
        tx
    }

    /// Generate multiple transactions
    pub fn generate_many(count: usize) -> Vec<Self> {
        (0..count).map(|_| Self::generate()).collect()
    }
}

/// Payment test fixture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentFixture {
    pub id: String,
    pub user_id: String,
    pub asset_id: String,
    pub amount: f64,
    pub currency: String,
    pub payment_method: String,
    pub status: String,
    pub transaction_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PaymentFixture {
    /// Generate a random payment fixture
    pub fn generate() -> Self {
        let now = Utc::now();
        let currencies = ["USD", "EUR", "GBP"];
        let methods = ["credit_card", "bank_transfer", "crypto"];
        let statuses = ["pending", "completed", "failed", "cancelled"];

        let mut rng = rand::thread_rng();
        let status = statuses[rng.gen_range(0..statuses.len())];

        let mut metadata = HashMap::new();
        metadata.insert(
            "processor".to_string(),
            serde_json::Value::String("stripe".to_string()),
        );

        Self {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            asset_id: Uuid::new_v4().to_string(),
            amount: rng.gen_range(100.0..50000.0),
            currency: currencies[rng.gen_range(0..currencies.len())].to_string(),
            payment_method: methods[rng.gen_range(0..methods.len())].to_string(),
            status: status.to_string(),
            transaction_id: if status == "completed" {
                Some(format!(
                    "tx_{}",
                    fake::faker::lorem::en::Word().fake::<String>()
                ))
            } else {
                None
            },
            metadata,
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate a completed payment
    pub fn completed() -> Self {
        let mut payment = Self::generate();
        payment.status = "completed".to_string();
        payment.transaction_id = Some(format!(
            "tx_{}",
            fake::faker::lorem::en::Word().fake::<String>()
        ));
        payment
    }

    /// Generate multiple payments
    pub fn generate_many(count: usize) -> Vec<Self> {
        (0..count).map(|_| Self::generate()).collect()
    }
}

/// Fixture builder for creating related test data
pub struct FixtureBuilder {
    users: Vec<UserFixture>,
    assets: Vec<AssetFixture>,
    transactions: Vec<TransactionFixture>,
    payments: Vec<PaymentFixture>,
}

impl FixtureBuilder {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            assets: Vec::new(),
            transactions: Vec::new(),
            payments: Vec::new(),
        }
    }

    /// Add users to the fixture set
    pub fn with_users(mut self, count: usize) -> Self {
        self.users = UserFixture::generate_many(count);
        self
    }

    /// Add assets for existing users
    pub fn with_assets_for_users(mut self, assets_per_user: usize) -> Self {
        for user in &self.users {
            for _ in 0..assets_per_user {
                self.assets.push(AssetFixture::for_owner(&user.id));
            }
        }
        self
    }

    /// Add transactions for existing assets
    pub fn with_transactions_for_assets(mut self, transactions_per_asset: usize) -> Self {
        for asset in &self.assets {
            for _ in 0..transactions_per_asset {
                let mut tx = TransactionFixture::generate();
                tx.asset_id = Some(asset.id.clone());
                tx.user_id = Some(asset.owner_id.clone());
                self.transactions.push(tx);
            }
        }
        self
    }

    /// Add payments for existing users and assets
    pub fn with_payments_for_users(mut self, payments_per_user: usize) -> Self {
        for user in &self.users {
            let user_assets: Vec<_> = self
                .assets
                .iter()
                .filter(|a| a.owner_id == user.id)
                .collect();

            for _ in 0..payments_per_user {
                let mut payment = PaymentFixture::generate();
                payment.user_id = user.id.clone();

                if let Some(asset) = user_assets.first() {
                    payment.asset_id = asset.id.clone();
                }

                self.payments.push(payment);
            }
        }
        self
    }

    /// Build the complete fixture set
    pub fn build(self) -> FixtureSet {
        FixtureSet {
            users: self.users,
            assets: self.assets,
            transactions: self.transactions,
            payments: self.payments,
        }
    }
}

/// Complete set of test fixtures
#[derive(Debug, Clone)]
pub struct FixtureSet {
    pub users: Vec<UserFixture>,
    pub assets: Vec<AssetFixture>,
    pub transactions: Vec<TransactionFixture>,
    pub payments: Vec<PaymentFixture>,
}

impl FixtureSet {
    /// Create a simple fixture set with related data
    pub fn simple() -> Self {
        FixtureBuilder::new()
            .with_users(3)
            .with_assets_for_users(2)
            .with_transactions_for_assets(1)
            .with_payments_for_users(1)
            .build()
    }

    /// Create a comprehensive fixture set
    pub fn comprehensive() -> Self {
        FixtureBuilder::new()
            .with_users(10)
            .with_assets_for_users(5)
            .with_transactions_for_assets(3)
            .with_payments_for_users(2)
            .build()
    }

    /// Get total count of all fixtures
    pub fn total_count(&self) -> usize {
        self.users.len() + self.assets.len() + self.transactions.len() + self.payments.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_fixture_generation() {
        let user = UserFixture::generate();
        assert!(!user.id.is_empty());
        assert!(user.email.contains('@'));
        assert!(!user.first_name.is_empty());
        assert!(!user.last_name.is_empty());
    }

    #[test]
    fn test_asset_fixture_generation() {
        let asset = AssetFixture::generate();
        assert!(!asset.id.is_empty());
        assert!(!asset.name.is_empty());
        assert!(asset.total_value > 0.0);
        assert!(!asset.currency.is_empty());
    }

    #[test]
    fn test_transaction_fixture_generation() {
        let tx = TransactionFixture::generate();
        assert!(!tx.id.is_empty());
        assert!(tx.transaction_hash.starts_with("0x"));
        assert!(tx.amount > 0.0);
    }

    #[test]
    fn test_fixture_builder() {
        let fixtures = FixtureBuilder::new()
            .with_users(2)
            .with_assets_for_users(1)
            .build();

        assert_eq!(fixtures.users.len(), 2);
        assert_eq!(fixtures.assets.len(), 2);

        // Check that assets are owned by the generated users
        for asset in &fixtures.assets {
            assert!(fixtures.users.iter().any(|u| u.id == asset.owner_id));
        }
    }

    #[test]
    fn test_fixture_set_simple() {
        let fixtures = FixtureSet::simple();
        assert_eq!(fixtures.users.len(), 3);
        assert_eq!(fixtures.assets.len(), 6); // 3 users * 2 assets
        assert!(fixtures.total_count() > 0);
    }
}
