// =====================================================================================
// File: core-stablecoin/src/lib.rs
// Description: Enterprise-grade stablecoin management system for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Stablecoin Module
//! 
//! This module provides comprehensive stablecoin management functionality for the StableRWA platform,
//! including multiple stability mechanisms, issuance/redemption systems, and RWA-backed stablecoins.
//!
//! ## Key Features
//!
//! - **Multiple Stability Mechanisms**: Fiat-collateralized, crypto-collateralized, algorithmic, and RWA-backed
//! - **Enterprise-grade Security**: Multi-signature wallets, time-locks, and emergency controls
//! - **Regulatory Compliance**: KYC/AML integration, reporting, and audit trails
//! - **Risk Management**: Real-time monitoring, collateral management, and liquidation mechanisms
//! - **Cross-chain Support**: Multi-blockchain deployment and bridging capabilities
//!
//! ## Architecture
//!
//! The module is organized into several key components:
//!
//! - `types`: Core data structures and enums
//! - `mechanisms`: Different stability mechanism implementations
//! - `issuance`: Token minting and burning logic
//! - `redemption`: Token redemption and collateral release
//! - `stability`: Price stability maintenance systems
//! - `governance`: Decentralized governance mechanisms
//! - `compliance`: Regulatory compliance and reporting
//! - `risk_management`: Risk assessment and mitigation
//! - `service`: High-level service interfaces
//!
//! ## Usage Example
//!
//! ```rust
//! use core_stablecoin::{StablecoinService, StablecoinConfig, StabilityMechanism};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = StablecoinConfig::default();
//!     let service = StablecoinService::new(config).await?;
//!     
//!     // Create RWA-backed stablecoin
//!     let stablecoin = service.create_stablecoin(
//!         "RWAUSD".to_string(),
//!         "RWA USD Stablecoin".to_string(),
//!         StabilityMechanism::RWABacked,
//!     ).await?;
//!     
//!     println!("Created stablecoin: {}", stablecoin.symbol);
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;
pub mod mechanisms;
pub mod issuance;
pub mod redemption;
pub mod stability;
pub mod governance;
pub mod compliance;
pub mod monitoring;
pub mod risk_management;
pub mod oracle;
pub mod liquidity;
pub mod service;

// Re-export main types and traits
pub use error::{StablecoinError, StablecoinResult};
pub use types::{
    Stablecoin, StablecoinConfig, StabilityMechanism, CollateralType, 
    IssuanceRequest, RedemptionRequest, StabilityParameters
};
pub use mechanisms::{
    FiatCollateralized, CryptoCollateralized, Algorithmic, RWABacked,
    StabilityMechanismTrait
};
pub use issuance::{IssuanceService, IssuanceManager};
pub use redemption::{RedemptionService, RedemptionManager};
pub use stability::{StabilityService, EnterprisePriceStabilizer, EnterpriseArbitrageBot};
pub use governance::{GovernanceService, ProposalManager};
pub use compliance::{
    StablecoinComplianceService, EnterpriseStablecoinCompliance,
    StablecoinComplianceConfig, StablecoinTransactionData, StablecoinTransactionType,
    KycComplianceResult, AmlComplianceResult, UserComplianceStatus,
    ComplianceCheckRequest, ComplianceCheckResult, StablecoinComplianceReport,
    SuspiciousActivity, RiskScore, ComplianceMetrics
};
pub use monitoring::{
    MonitoringService, EnterpriseMonitoringService, MonitoringConfig,
    SystemHealth, StablecoinMetrics, Alert, AlertHandler,
    EmailAlertHandler, SlackAlertHandler, PagerDutyAlertHandler
};
pub use risk_management::{RiskManager, CollateralManager};
pub use oracle::{PriceOracle, OracleAggregator, PriceData, OracleConfig};
pub use liquidity::{LiquidityManager, EnterpriseLiquidityManager, LiquidityPosition, PoolInfo};
pub use service::{StablecoinService, StablecoinServiceImpl};

/// Current version of the core-stablecoin module
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration for stablecoin operations
pub fn default_config() -> StablecoinConfig {
    StablecoinConfig::default()
}

/// Initialize the stablecoin module with logging
pub fn init() {
    tracing::info!("Initializing core-stablecoin module v{}", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_initialization() {
        init();
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert!(config.enable_governance);
        assert!(config.enable_compliance);
    }
}
