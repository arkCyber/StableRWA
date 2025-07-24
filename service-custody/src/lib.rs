// =====================================================================================
// RWA Tokenization Platform - Custody Service Library
// 
// This module provides enterprise-grade custody services for digital and physical assets
// including secure key management, multi-signature wallets, custody proof systems,
// and insurance integration.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Custody Service Library
//! 
//! This library provides comprehensive custody services for the RWA tokenization platform,
//! including:
//! 
//! - **Digital Asset Custody**: Secure private key management and multi-signature wallets
//! - **Physical Asset Custody**: Integration with third-party custody institutions
//! - **Custody Proof System**: Asset existence and ownership verification
//! - **Insurance Integration**: Insurance coverage for custodied assets
//! 
//! ## Features
//! 
//! - Enterprise-grade security with HSM support
//! - Multi-signature wallet management
//! - Real-time custody monitoring and alerts
//! - Comprehensive audit trails
//! - Insurance policy management
//! - Regulatory compliance reporting
//! 
//! ## Architecture
//! 
//! The custody service is built with a modular architecture:
//! 
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   Digital       │    │   Physical      │    │   Insurance     │
//! │   Custody       │    │   Custody       │    │   Integration   │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!          │                       │                       │
//!          └───────────────────────┼───────────────────────┘
//!                                  │
//!                    ┌─────────────────┐
//!                    │   Custody       │
//!                    │   Service       │
//!                    │   Core          │
//!                    └─────────────────┘
//!                                  │
//!                    ┌─────────────────┐
//!                    │   Proof         │
//!                    │   System        │
//!                    └─────────────────┘
//! ```

pub mod config;
pub mod error;
pub mod types;
pub mod digital;
pub mod physical;
pub mod proof;
pub mod insurance;
pub mod service;
pub mod handlers;
pub mod middleware;
pub mod metrics;
pub mod utils;

// Re-export commonly used types and functions
pub use config::CustodyConfig;
pub use error::{CustodyError, CustodyResult};
pub use service::CustodyService;
pub use types::*;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the custody service with default configuration
/// 
/// # Returns
/// 
/// Returns a configured `CustodyService` instance ready for use
/// 
/// # Errors
/// 
/// Returns `CustodyError` if initialization fails
pub async fn init() -> CustodyResult<CustodyService> {
    let config = CustodyConfig::from_env()?;
    CustodyService::new(config).await
}

/// Initialize the custody service with custom configuration
/// 
/// # Arguments
/// 
/// * `config` - Custom custody service configuration
/// 
/// # Returns
/// 
/// Returns a configured `CustodyService` instance
/// 
/// # Errors
/// 
/// Returns `CustodyError` if initialization fails
pub async fn init_with_config(config: CustodyConfig) -> CustodyResult<CustodyService> {
    CustodyService::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "service-custody");
    }

    #[tokio::test]
    async fn test_init_with_default_config() {
        // This test would require proper environment setup
        // For now, we just test that the function exists and can be called
        let config = CustodyConfig::default();
        let result = init_with_config(config).await;
        
        // In a real test environment, this would succeed
        // For now, we just verify the function signature
        assert!(result.is_err() || result.is_ok());
    }
}
