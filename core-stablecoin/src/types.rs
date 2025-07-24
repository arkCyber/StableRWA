// =====================================================================================
// File: core-stablecoin/src/types.rs
// Description: Core data structures and types for stablecoin operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Stablecoin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StablecoinConfig {
    /// Enable governance features
    pub enable_governance: bool,
    /// Enable compliance checks
    pub enable_compliance: bool,
    /// Enable risk management
    pub enable_risk_management: bool,
    /// Enable cross-chain bridging
    pub enable_cross_chain: bool,
    /// Default stability mechanism
    pub default_stability_mechanism: StabilityMechanism,
    /// Minimum collateral ratio (as percentage)
    pub min_collateral_ratio: Decimal,
    /// Maximum issuance per transaction
    pub max_issuance_per_tx: Decimal,
    /// Maximum redemption per transaction
    pub max_redemption_per_tx: Decimal,
    /// Price deviation threshold for stability actions
    pub price_deviation_threshold: Decimal,
    /// Emergency pause threshold
    pub emergency_pause_threshold: Decimal,
    /// Supported networks
    pub supported_networks: Vec<u64>,
    /// Oracle configuration
    pub oracle_config: OracleConfig,
    /// Fee structure
    pub fee_structure: FeeStructure,
}

impl Default for StablecoinConfig {
    fn default() -> Self {
        Self {
            enable_governance: true,
            enable_compliance: true,
            enable_risk_management: true,
            enable_cross_chain: false,
            default_stability_mechanism: StabilityMechanism::RWABacked,
            min_collateral_ratio: Decimal::new(150, 2), // 150%
            max_issuance_per_tx: Decimal::new(1_000_000, 0), // 1M tokens
            max_redemption_per_tx: Decimal::new(1_000_000, 0), // 1M tokens
            price_deviation_threshold: Decimal::new(5, 2), // 5%
            emergency_pause_threshold: Decimal::new(10, 2), // 10%
            supported_networks: vec![1, 137, 56], // Ethereum, Polygon, BSC
            oracle_config: OracleConfig::default(),
            fee_structure: FeeStructure::default(),
        }
    }
}

/// Stability mechanism types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StabilityMechanism {
    /// Fiat-collateralized (e.g., USDC, USDT)
    Fiat,
    /// Crypto-collateralized (e.g., DAI)
    Crypto,
    /// Algorithmic (e.g., UST, FRAX)
    Algorithmic,
    /// Hybrid mechanism
    Hybrid,
    /// RWA-backed (StableRWA's innovation)
    RWABacked,
}

impl StabilityMechanism {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            StabilityMechanism::Fiat => "Backed by fiat currency reserves",
            StabilityMechanism::Crypto => "Backed by cryptocurrency collateral",
            StabilityMechanism::Algorithmic => "Maintained through algorithmic mechanisms",
            StabilityMechanism::Hybrid => "Combination of collateral and algorithmic mechanisms",
            StabilityMechanism::RWABacked => "Backed by real-world assets",
        }
    }

    /// Check if mechanism requires collateral
    pub fn requires_collateral(&self) -> bool {
        matches!(self, StabilityMechanism::Fiat | StabilityMechanism::Crypto | StabilityMechanism::RWABacked | StabilityMechanism::Hybrid)
    }

    /// Get minimum collateral ratio
    pub fn min_collateral_ratio(&self) -> Decimal {
        match self {
            StabilityMechanism::Fiat => Decimal::new(100, 2), // 100%
            StabilityMechanism::Crypto => Decimal::new(150, 2), // 150%
            StabilityMechanism::Algorithmic => Decimal::ZERO,
            StabilityMechanism::Hybrid => Decimal::new(120, 2), // 120%
            StabilityMechanism::RWABacked => Decimal::new(110, 2), // 110%
        }
    }
}

/// Collateral types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollateralType {
    /// Fiat currency
    Fiat { currency: String },
    /// Cryptocurrency
    Crypto { token_address: String, symbol: String },
    /// Real-world asset
    RWA { asset_id: Uuid, asset_type: String },
    /// Basket of assets
    Basket { assets: Vec<CollateralType> },
}

impl CollateralType {
    /// Get collateral identifier
    pub fn identifier(&self) -> String {
        match self {
            CollateralType::Fiat { currency } => format!("fiat:{}", currency),
            CollateralType::Crypto { symbol, .. } => format!("crypto:{}", symbol),
            CollateralType::RWA { asset_id, .. } => format!("rwa:{}", asset_id),
            CollateralType::Basket { assets } => {
                let ids: Vec<String> = assets.iter().map(|a| a.identifier()).collect();
                format!("basket:{}", ids.join(","))
            }
        }
    }

    /// Check if collateral is volatile
    pub fn is_volatile(&self) -> bool {
        matches!(self, CollateralType::Crypto { .. })
    }
}

/// Stablecoin definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stablecoin {
    /// Unique identifier
    pub id: Uuid,
    /// Token symbol (e.g., "RWAUSD")
    pub symbol: String,
    /// Token name (e.g., "RWA USD Stablecoin")
    pub name: String,
    /// Number of decimal places
    pub decimals: u8,
    /// Stability mechanism
    pub stability_mechanism: StabilityMechanism,
    /// Target price (usually 1.0 for USD-pegged)
    pub target_price: Decimal,
    /// Current price
    pub current_price: Decimal,
    /// Total supply
    pub total_supply: Decimal,
    /// Total collateral value
    pub total_collateral_value: Decimal,
    /// Collateral ratio
    pub collateral_ratio: Decimal,
    /// Supported collateral types
    pub supported_collateral: Vec<CollateralType>,
    /// Contract addresses on different networks
    pub contract_addresses: HashMap<u64, String>,
    /// Stability parameters
    pub stability_parameters: StabilityParameters,
    /// Current status
    pub status: StablecoinStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Stablecoin status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StablecoinStatus {
    /// Active and operational
    Active,
    /// Temporarily paused
    Paused,
    /// Emergency pause (critical issues)
    EmergencyPause,
    /// Deprecated (no new issuance)
    Deprecated,
    /// Sunset (redemption only)
    Sunset,
}

/// Stability parameters for different mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityParameters {
    /// Rebalancing frequency (in seconds)
    pub rebalancing_frequency: u64,
    /// Price band for stability actions (percentage)
    pub price_band: Decimal,
    /// Maximum daily issuance
    pub max_daily_issuance: Decimal,
    /// Maximum daily redemption
    pub max_daily_redemption: Decimal,
    /// Arbitrage incentive rate
    pub arbitrage_incentive_rate: Decimal,
    /// Liquidation threshold
    pub liquidation_threshold: Decimal,
    /// Stability fee (annual percentage)
    pub stability_fee: Decimal,
    /// Reserve ratio for algorithmic mechanisms
    pub reserve_ratio: Option<Decimal>,
}

impl Default for StabilityParameters {
    fn default() -> Self {
        Self {
            rebalancing_frequency: 3600, // 1 hour
            price_band: Decimal::new(1, 2), // 1%
            max_daily_issuance: Decimal::new(10_000_000, 0), // 10M tokens
            max_daily_redemption: Decimal::new(10_000_000, 0), // 10M tokens
            arbitrage_incentive_rate: Decimal::new(5, 3), // 0.5%
            liquidation_threshold: Decimal::new(110, 2), // 110%
            stability_fee: Decimal::new(2, 2), // 2% annual
            reserve_ratio: Some(Decimal::new(20, 2)), // 20%
        }
    }
}

/// Oracle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    /// Primary oracle provider
    pub primary_oracle: String,
    /// Backup oracle providers
    pub backup_oracles: Vec<String>,
    /// Price update frequency (in seconds)
    pub update_frequency: u64,
    /// Maximum price age (in seconds)
    pub max_price_age: u64,
    /// Minimum number of oracle sources
    pub min_oracle_sources: u32,
    /// Price deviation threshold for oracle validation
    pub price_deviation_threshold: Decimal,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            primary_oracle: "chainlink".to_string(),
            backup_oracles: vec!["band".to_string(), "api3".to_string()],
            update_frequency: 300, // 5 minutes
            max_price_age: 900, // 15 minutes
            min_oracle_sources: 3,
            price_deviation_threshold: Decimal::new(2, 2), // 2%
        }
    }
}

/// Fee structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeStructure {
    /// Issuance fee (percentage)
    pub issuance_fee: Decimal,
    /// Redemption fee (percentage)
    pub redemption_fee: Decimal,
    /// Stability fee (annual percentage)
    pub stability_fee: Decimal,
    /// Governance fee (percentage of operations)
    pub governance_fee: Decimal,
    /// Emergency fee (for emergency operations)
    pub emergency_fee: Decimal,
}

impl Default for FeeStructure {
    fn default() -> Self {
        Self {
            issuance_fee: Decimal::new(10, 4), // 0.1%
            redemption_fee: Decimal::new(10, 4), // 0.1%
            stability_fee: Decimal::new(2, 2), // 2% annual
            governance_fee: Decimal::new(5, 4), // 0.05%
            emergency_fee: Decimal::new(50, 4), // 0.5%
        }
    }
}

/// Issuance request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuanceRequest {
    /// Request ID
    pub id: Uuid,
    /// User ID
    pub user_id: String,
    /// Stablecoin ID
    pub stablecoin_id: Uuid,
    /// Amount to issue
    pub amount: Decimal,
    /// Collateral provided
    pub collateral: Vec<CollateralPosition>,
    /// Target network
    pub network_id: u64,
    /// Recipient address
    pub recipient_address: String,
    /// Request timestamp
    pub created_at: DateTime<Utc>,
}

/// Redemption request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedemptionRequest {
    /// Request ID
    pub id: Uuid,
    /// User ID
    pub user_id: String,
    /// Stablecoin ID
    pub stablecoin_id: Uuid,
    /// Amount to redeem
    pub amount: Decimal,
    /// Preferred collateral for redemption
    pub preferred_collateral: Option<CollateralType>,
    /// Source network
    pub network_id: u64,
    /// Sender address
    pub sender_address: String,
    /// Request timestamp
    pub created_at: DateTime<Utc>,
}

/// Collateral position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollateralPosition {
    /// Position ID
    pub id: Uuid,
    /// Collateral type
    pub collateral_type: CollateralType,
    /// Amount of collateral
    pub amount: Decimal,
    /// Current value in USD
    pub value_usd: Decimal,
    /// Lock timestamp (if applicable)
    pub locked_until: Option<DateTime<Utc>>,
    /// Position status
    pub status: CollateralStatus,
}

/// Collateral status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollateralStatus {
    /// Active and available
    Active,
    /// Locked for specific operation
    Locked,
    /// Under liquidation
    Liquidating,
    /// Released back to user
    Released,
    /// Used/consumed in transaction
    Used,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stability_mechanism_properties() {
        // Test collateral requirements
        assert!(StabilityMechanism::Fiat.requires_collateral());
        assert!(StabilityMechanism::Crypto.requires_collateral());
        assert!(StabilityMechanism::RWABacked.requires_collateral());
        assert!(StabilityMechanism::Hybrid.requires_collateral());
        assert!(!StabilityMechanism::Algorithmic.requires_collateral());

        // Test minimum collateral ratios
        assert_eq!(StabilityMechanism::Fiat.min_collateral_ratio(), Decimal::new(100, 2));
        assert_eq!(StabilityMechanism::Crypto.min_collateral_ratio(), Decimal::new(150, 2));
        assert_eq!(StabilityMechanism::RWABacked.min_collateral_ratio(), Decimal::new(110, 2));
        assert_eq!(StabilityMechanism::Hybrid.min_collateral_ratio(), Decimal::new(120, 2));
        assert_eq!(StabilityMechanism::Algorithmic.min_collateral_ratio(), Decimal::ZERO);
    }

    #[test]
    fn test_stability_mechanism_descriptions() {
        assert_eq!(StabilityMechanism::Fiat.description(), "Backed by fiat currency reserves");
        assert_eq!(StabilityMechanism::Crypto.description(), "Backed by cryptocurrency collateral");
        assert_eq!(StabilityMechanism::Algorithmic.description(), "Maintained through algorithmic mechanisms");
        assert_eq!(StabilityMechanism::Hybrid.description(), "Combination of collateral and algorithmic mechanisms");
        assert_eq!(StabilityMechanism::RWABacked.description(), "Backed by real-world assets");
    }

    #[test]
    fn test_collateral_type_identifier() {
        let fiat = CollateralType::Fiat { currency: "USD".to_string() };
        assert_eq!(fiat.identifier(), "fiat:USD");

        let crypto = CollateralType::Crypto {
            token_address: "0x123".to_string(),
            symbol: "ETH".to_string()
        };
        assert_eq!(crypto.identifier(), "crypto:ETH");

        let rwa_id = Uuid::new_v4();
        let rwa = CollateralType::RWA {
            asset_id: rwa_id,
            asset_type: "real_estate".to_string()
        };
        assert_eq!(rwa.identifier(), format!("rwa:{}", rwa_id));

        let basket = CollateralType::Basket {
            assets: vec![
                CollateralType::Fiat { currency: "USD".to_string() },
                CollateralType::Crypto { token_address: "0x123".to_string(), symbol: "ETH".to_string() }
            ]
        };
        assert_eq!(basket.identifier(), "basket:fiat:USD,crypto:ETH");
    }

    #[test]
    fn test_collateral_type_volatility() {
        let fiat = CollateralType::Fiat { currency: "USD".to_string() };
        assert!(!fiat.is_volatile());

        let crypto = CollateralType::Crypto {
            token_address: "0x123".to_string(),
            symbol: "ETH".to_string()
        };
        assert!(crypto.is_volatile());

        let rwa = CollateralType::RWA {
            asset_id: Uuid::new_v4(),
            asset_type: "real_estate".to_string()
        };
        assert!(!rwa.is_volatile());

        let basket = CollateralType::Basket { assets: vec![] };
        assert!(!basket.is_volatile());
    }

    #[test]
    fn test_default_config() {
        let config = StablecoinConfig::default();
        assert!(config.enable_governance);
        assert!(config.enable_compliance);
        assert!(config.enable_risk_management);
        assert!(!config.enable_cross_chain);
        assert_eq!(config.default_stability_mechanism, StabilityMechanism::RWABacked);
        assert_eq!(config.min_collateral_ratio, Decimal::new(150, 2));
        assert_eq!(config.max_issuance_per_tx, Decimal::new(1_000_000, 0));
        assert_eq!(config.max_redemption_per_tx, Decimal::new(1_000_000, 0));
        assert_eq!(config.price_deviation_threshold, Decimal::new(5, 2));
        assert_eq!(config.emergency_pause_threshold, Decimal::new(10, 2));
        assert_eq!(config.supported_networks, vec![1, 137, 56]);
    }

    #[test]
    fn test_default_stability_parameters() {
        let params = StabilityParameters::default();
        assert_eq!(params.rebalancing_frequency, 3600);
        assert_eq!(params.price_band, Decimal::new(1, 2));
        assert_eq!(params.max_daily_issuance, Decimal::new(10_000_000, 0));
        assert_eq!(params.max_daily_redemption, Decimal::new(10_000_000, 0));
        assert_eq!(params.arbitrage_incentive_rate, Decimal::new(5, 3));
        assert_eq!(params.liquidation_threshold, Decimal::new(110, 2));
        assert_eq!(params.stability_fee, Decimal::new(2, 2));
        assert_eq!(params.reserve_ratio, Some(Decimal::new(20, 2)));
    }

    #[test]
    fn test_default_oracle_config() {
        let config = OracleConfig::default();
        assert_eq!(config.primary_oracle, "chainlink");
        assert_eq!(config.backup_oracles, vec!["band", "api3"]);
        assert_eq!(config.update_frequency, 300);
        assert_eq!(config.max_price_age, 900);
        assert_eq!(config.min_oracle_sources, 3);
        assert_eq!(config.price_deviation_threshold, Decimal::new(2, 2));
    }

    #[test]
    fn test_default_fee_structure() {
        let fees = FeeStructure::default();
        assert_eq!(fees.issuance_fee, Decimal::new(10, 4));
        assert_eq!(fees.redemption_fee, Decimal::new(10, 4));
        assert_eq!(fees.stability_fee, Decimal::new(2, 2));
        assert_eq!(fees.governance_fee, Decimal::new(5, 4));
        assert_eq!(fees.emergency_fee, Decimal::new(50, 4));
    }

    #[test]
    fn test_stablecoin_status_enum() {
        // Test all variants exist
        let statuses = vec![
            StablecoinStatus::Active,
            StablecoinStatus::Paused,
            StablecoinStatus::EmergencyPause,
            StablecoinStatus::Deprecated,
            StablecoinStatus::Sunset,
        ];

        for status in statuses {
            // Test Debug trait
            let debug_str = format!("{:?}", status);
            assert!(!debug_str.is_empty());

            // Test Clone trait
            let cloned = status.clone();
            assert_eq!(status, cloned);
        }
    }

    #[test]
    fn test_collateral_status_enum() {
        let statuses = vec![
            CollateralStatus::Active,
            CollateralStatus::Locked,
            CollateralStatus::Liquidating,
            CollateralStatus::Released,
        ];

        for status in statuses {
            let debug_str = format!("{:?}", status);
            assert!(!debug_str.is_empty());

            let cloned = status.clone();
            assert_eq!(status, cloned);
        }
    }

    #[test]
    fn test_stablecoin_creation() {
        let stablecoin = Stablecoin {
            id: Uuid::new_v4(),
            symbol: "RWAUSD".to_string(),
            name: "RWA USD Stablecoin".to_string(),
            decimals: 18,
            stability_mechanism: StabilityMechanism::RWABacked,
            target_price: Decimal::ONE,
            current_price: Decimal::ONE,
            total_supply: Decimal::ZERO,
            total_collateral_value: Decimal::ZERO,
            collateral_ratio: Decimal::ZERO,
            supported_collateral: vec![],
            contract_addresses: HashMap::new(),
            stability_parameters: StabilityParameters::default(),
            status: StablecoinStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(stablecoin.symbol, "RWAUSD");
        assert_eq!(stablecoin.name, "RWA USD Stablecoin");
        assert_eq!(stablecoin.decimals, 18);
        assert_eq!(stablecoin.stability_mechanism, StabilityMechanism::RWABacked);
        assert_eq!(stablecoin.status, StablecoinStatus::Active);
    }

    #[test]
    fn test_issuance_request_creation() {
        let request = IssuanceRequest {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(1000, 0),
            collateral: vec![],
            network_id: 1,
            recipient_address: "0x123".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(request.user_id, "user123");
        assert_eq!(request.amount, Decimal::new(1000, 0));
        assert_eq!(request.network_id, 1);
        assert_eq!(request.recipient_address, "0x123");
    }

    #[test]
    fn test_redemption_request_creation() {
        let request = RedemptionRequest {
            id: Uuid::new_v4(),
            user_id: "user456".to_string(),
            stablecoin_id: Uuid::new_v4(),
            amount: Decimal::new(500, 0),
            preferred_collateral: Some(CollateralType::Fiat { currency: "USD".to_string() }),
            network_id: 1,
            sender_address: "0x456".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(request.user_id, "user456");
        assert_eq!(request.amount, Decimal::new(500, 0));
        assert!(request.preferred_collateral.is_some());
        assert_eq!(request.sender_address, "0x456");
    }

    #[test]
    fn test_collateral_position_creation() {
        let position = CollateralPosition {
            id: Uuid::new_v4(),
            collateral_type: CollateralType::Fiat { currency: "USD".to_string() },
            amount: Decimal::new(1000, 0),
            value_usd: Decimal::new(1000, 0),
            locked_until: None,
            status: CollateralStatus::Active,
        };

        assert_eq!(position.amount, Decimal::new(1000, 0));
        assert_eq!(position.value_usd, Decimal::new(1000, 0));
        assert!(position.locked_until.is_none());
        assert_eq!(position.status, CollateralStatus::Active);
    }

    #[test]
    fn test_serialization_deserialization() {
        let config = StablecoinConfig::default();

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.is_empty());

        // Test JSON deserialization
        let deserialized: StablecoinConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.enable_governance, deserialized.enable_governance);
        assert_eq!(config.default_stability_mechanism, deserialized.default_stability_mechanism);
    }

    #[test]
    fn test_stability_mechanism_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(StabilityMechanism::Fiat);
        set.insert(StabilityMechanism::Crypto);
        set.insert(StabilityMechanism::Fiat); // Duplicate

        assert_eq!(set.len(), 2); // Should only contain unique values
        assert!(set.contains(&StabilityMechanism::Fiat));
        assert!(set.contains(&StabilityMechanism::Crypto));
    }

    #[test]
    fn test_collateral_type_equality() {
        let fiat1 = CollateralType::Fiat { currency: "USD".to_string() };
        let fiat2 = CollateralType::Fiat { currency: "USD".to_string() };
        let fiat3 = CollateralType::Fiat { currency: "EUR".to_string() };

        assert_eq!(fiat1, fiat2);
        assert_ne!(fiat1, fiat3);
    }

    #[test]
    fn test_nested_basket_collateral() {
        let inner_basket = CollateralType::Basket {
            assets: vec![
                CollateralType::Fiat { currency: "USD".to_string() },
                CollateralType::Crypto { token_address: "0x123".to_string(), symbol: "ETH".to_string() }
            ]
        };

        let outer_basket = CollateralType::Basket {
            assets: vec![
                inner_basket,
                CollateralType::RWA { asset_id: Uuid::new_v4(), asset_type: "bonds".to_string() }
            ]
        };

        let identifier = outer_basket.identifier();
        assert!(identifier.starts_with("basket:"));
        assert!(identifier.contains("fiat:USD"));
        assert!(identifier.contains("crypto:ETH"));
    }
}
