// =====================================================================================
// File: core-defi/src/amm.rs
// Description: Automated Market Maker (AMM) implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::{
    error::{DeFiError, DeFiResult},
    types::{Token, TokenPair, LiquidityPool, AMMProtocol, Price},
};

/// AMM service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMConfig {
    /// Default slippage tolerance
    pub default_slippage_tolerance: Decimal,
    /// Maximum slippage tolerance
    pub max_slippage_tolerance: Decimal,
    /// Minimum liquidity for new pools
    pub min_liquidity_threshold: Decimal,
    /// Maximum price impact allowed
    pub max_price_impact: Decimal,
    /// Enable MEV protection
    pub enable_mev_protection: bool,
    /// Enable sandwich attack protection
    pub enable_sandwich_protection: bool,
    /// Gas optimization level
    pub gas_optimization_level: u8,
    /// Route optimization enabled
    pub enable_route_optimization: bool,
    /// Maximum number of hops in routing
    pub max_routing_hops: u8,
}

impl Default for AMMConfig {
    fn default() -> Self {
        Self {
            default_slippage_tolerance: Decimal::new(50, 4), // 0.5%
            max_slippage_tolerance: Decimal::new(1000, 4), // 10%
            min_liquidity_threshold: Decimal::new(100000, 2), // $1,000
            max_price_impact: Decimal::new(500, 4), // 5%
            enable_mev_protection: true,
            enable_sandwich_protection: true,
            gas_optimization_level: 2,
            enable_route_optimization: true,
            max_routing_hops: 3,
        }
    }
}

/// Swap request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    pub id: Uuid,
    pub user_id: String,
    pub token_in: Token,
    pub token_out: Token,
    pub amount_in: Decimal,
    pub min_amount_out: Decimal,
    pub slippage_tolerance: Decimal,
    pub deadline: DateTime<Utc>,
    pub recipient: Option<String>,
    pub route: Option<Vec<String>>, // Pool addresses for routing
    pub gas_price: Option<u64>,
    pub created_at: DateTime<Utc>,
}

impl SwapRequest {
    /// Create a new swap request
    pub fn new(
        user_id: String,
        token_in: Token,
        token_out: Token,
        amount_in: Decimal,
        slippage_tolerance: Decimal,
    ) -> Self {
        let id = Uuid::new_v4();
        let deadline = Utc::now() + chrono::Duration::minutes(20);
        
        Self {
            id,
            user_id,
            token_in,
            token_out,
            amount_in,
            min_amount_out: Decimal::ZERO, // Will be calculated
            slippage_tolerance,
            deadline,
            recipient: None,
            route: None,
            gas_price: None,
            created_at: Utc::now(),
        }
    }

    /// Validate swap request
    pub fn validate(&self, config: &AMMConfig) -> DeFiResult<()> {
        if self.amount_in <= Decimal::ZERO {
            return Err(DeFiError::validation_error("amount_in", "Amount must be positive"));
        }

        if self.slippage_tolerance > config.max_slippage_tolerance {
            return Err(DeFiError::validation_error(
                "slippage_tolerance",
                "Slippage tolerance exceeds maximum allowed",
            ));
        }

        if self.deadline <= Utc::now() {
            return Err(DeFiError::validation_error("deadline", "Deadline must be in the future"));
        }

        if self.token_in.address == self.token_out.address {
            return Err(DeFiError::validation_error("tokens", "Cannot swap same token"));
        }

        Ok(())
    }
}

/// Swap result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    pub request_id: Uuid,
    pub transaction_hash: String,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub price_impact: Decimal,
    pub gas_used: u64,
    pub gas_price: u64,
    pub route: Vec<String>,
    pub executed_at: DateTime<Utc>,
}

/// Liquidity request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityRequest {
    pub id: Uuid,
    pub user_id: String,
    pub token_a: Token,
    pub token_b: Token,
    pub amount_a: Decimal,
    pub amount_b: Decimal,
    pub min_amount_a: Decimal,
    pub min_amount_b: Decimal,
    pub deadline: DateTime<Utc>,
    pub fee_tier: Option<u32>, // For Uniswap V3
    pub tick_lower: Option<i32>, // For concentrated liquidity
    pub tick_upper: Option<i32>, // For concentrated liquidity
    pub created_at: DateTime<Utc>,
}

/// Liquidity result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityResult {
    pub request_id: Uuid,
    pub transaction_hash: String,
    pub liquidity_tokens: Decimal,
    pub amount_a: Decimal,
    pub amount_b: Decimal,
    pub pool_address: String,
    pub executed_at: DateTime<Utc>,
}

/// Uniswap V2 style pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniswapV2Pool {
    pub address: String,
    pub token_pair: TokenPair,
    pub reserve_a: Decimal,
    pub reserve_b: Decimal,
    pub total_supply: Decimal,
    pub fee_rate: Decimal, // 0.3% = 0.003
    pub last_updated: DateTime<Utc>,
}

impl UniswapV2Pool {
    /// Calculate output amount for a given input
    pub fn get_amount_out(&self, amount_in: Decimal, token_in: &Token) -> DeFiResult<Decimal> {
        if amount_in <= Decimal::ZERO {
            return Err(DeFiError::validation_error("amount_in", "Amount must be positive"));
        }

        let (reserve_in, reserve_out) = if token_in.address == self.token_pair.token_a.address {
            (self.reserve_a, self.reserve_b)
        } else if token_in.address == self.token_pair.token_b.address {
            (self.reserve_b, self.reserve_a)
        } else {
            return Err(DeFiError::validation_error("token_in", "Token not in pool"));
        };

        if reserve_in <= Decimal::ZERO || reserve_out <= Decimal::ZERO {
            return Err(DeFiError::insufficient_liquidity("Pool has no liquidity"));
        }

        // Calculate amount out using constant product formula
        // amount_out = (amount_in * fee_multiplier * reserve_out) / (reserve_in + amount_in * fee_multiplier)
        let fee_multiplier = Decimal::ONE - self.fee_rate;
        let amount_in_with_fee = amount_in * fee_multiplier;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in + amount_in_with_fee;
        
        Ok(numerator / denominator)
    }

    /// Calculate price impact for a swap
    pub fn calculate_price_impact(&self, amount_in: Decimal, token_in: &Token) -> DeFiResult<Decimal> {
        let amount_out = self.get_amount_out(amount_in, token_in)?;
        
        let (reserve_in, reserve_out) = if token_in.address == self.token_pair.token_a.address {
            (self.reserve_a, self.reserve_b)
        } else {
            (self.reserve_b, self.reserve_a)
        };

        // Current price
        let current_price = reserve_out / reserve_in;
        
        // Execution price
        let execution_price = amount_out / amount_in;
        
        // Price impact = (current_price - execution_price) / current_price
        let price_impact = (current_price - execution_price) / current_price;
        
        Ok(price_impact.abs())
    }
}

/// Uniswap V3 style pool with concentrated liquidity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniswapV3Pool {
    pub address: String,
    pub token_pair: TokenPair,
    pub fee_tier: u32,
    pub tick_spacing: i32,
    pub current_tick: i32,
    pub sqrt_price_x96: String, // Q64.96 format
    pub liquidity: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Curve pool for stablecoins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurvePool {
    pub address: String,
    pub tokens: Vec<Token>,
    pub balances: Vec<Decimal>,
    pub amplification_parameter: u64,
    pub fee_rate: Decimal,
    pub admin_fee_rate: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Balancer weighted pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancerPool {
    pub address: String,
    pub tokens: Vec<Token>,
    pub balances: Vec<Decimal>,
    pub weights: Vec<Decimal>,
    pub swap_fee: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Route for multi-hop swaps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRoute {
    pub pools: Vec<String>,
    pub tokens: Vec<Token>,
    pub expected_amount_out: Decimal,
    pub price_impact: Decimal,
    pub gas_estimate: u64,
}

/// AMM service trait
#[async_trait]
pub trait AMMService: Send + Sync {
    /// Get quote for a swap
    async fn get_quote(
        &self,
        token_in: &Token,
        token_out: &Token,
        amount_in: Decimal,
    ) -> DeFiResult<SwapQuote>;
    
    /// Execute a swap
    async fn swap(&self, request: SwapRequest) -> DeFiResult<SwapResult>;
    
    /// Add liquidity to a pool
    async fn add_liquidity(&self, request: LiquidityRequest) -> DeFiResult<LiquidityResult>;
    
    /// Remove liquidity from a pool
    async fn remove_liquidity(&self, request: RemoveLiquidityRequest) -> DeFiResult<LiquidityResult>;
    
    /// Get pool information
    async fn get_pool(&self, pool_address: &str) -> DeFiResult<Option<LiquidityPool>>;
    
    /// Get all pools for a token pair
    async fn get_pools_for_pair(&self, token_a: &Token, token_b: &Token) -> DeFiResult<Vec<LiquidityPool>>;
    
    /// Find best route for a swap
    async fn find_best_route(
        &self,
        token_in: &Token,
        token_out: &Token,
        amount_in: Decimal,
    ) -> DeFiResult<SwapRoute>;
    
    /// Get supported protocols
    async fn get_supported_protocols(&self) -> DeFiResult<Vec<AMMProtocol>>;
    
    /// Health check
    async fn health_check(&self) -> DeFiResult<AMMHealthStatus>;
}

/// Swap quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub token_in: Token,
    pub token_out: Token,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub price_impact: Decimal,
    pub gas_estimate: u64,
    pub route: SwapRoute,
    pub valid_until: DateTime<Utc>,
}

/// Remove liquidity request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveLiquidityRequest {
    pub id: Uuid,
    pub user_id: String,
    pub pool_address: String,
    pub liquidity_tokens: Decimal,
    pub min_amount_a: Decimal,
    pub min_amount_b: Decimal,
    pub deadline: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// AMM health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMMHealthStatus {
    pub status: String,
    pub total_pools: u64,
    pub active_pools: u64,
    pub total_tvl: Decimal,
    pub volume_24h: Decimal,
    pub protocol_statuses: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tokens() -> (Token, Token) {
        let usdc = Token::new(
            "0xA0b86a33E6441b8435b662f0E2d0B8A0E6E6E6E6".to_string(),
            "USDC".to_string(),
            "USD Coin".to_string(),
            6,
            1,
        );

        let weth = Token::new(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            "WETH".to_string(),
            "Wrapped Ether".to_string(),
            18,
            1,
        );

        (usdc, weth)
    }

    #[test]
    fn test_swap_request_creation() {
        let (usdc, weth) = create_test_tokens();
        let request = SwapRequest::new(
            "user123".to_string(),
            usdc,
            weth,
            Decimal::new(1000, 6), // 1000 USDC
            Decimal::new(50, 4), // 0.5% slippage
        );

        assert_eq!(request.amount_in, Decimal::new(1000, 6));
        assert_eq!(request.slippage_tolerance, Decimal::new(50, 4));
        assert!(request.deadline > Utc::now());
    }

    #[test]
    fn test_swap_request_validation() {
        let (usdc, weth) = create_test_tokens();
        let config = AMMConfig::default();

        // Valid request
        let valid_request = SwapRequest::new(
            "user123".to_string(),
            usdc.clone(),
            weth.clone(),
            Decimal::new(1000, 6),
            Decimal::new(50, 4),
        );
        assert!(valid_request.validate(&config).is_ok());

        // Invalid amount
        let mut invalid_request = valid_request.clone();
        invalid_request.amount_in = Decimal::ZERO;
        assert!(invalid_request.validate(&config).is_err());

        // Invalid slippage
        let mut invalid_request = valid_request.clone();
        invalid_request.slippage_tolerance = Decimal::new(2000, 4); // 20%
        assert!(invalid_request.validate(&config).is_err());

        // Same token swap
        let mut invalid_request = valid_request.clone();
        invalid_request.token_out = usdc.clone();
        assert!(invalid_request.validate(&config).is_err());
    }

    #[test]
    fn test_uniswap_v2_pool_calculations() {
        let (usdc, weth) = create_test_tokens();
        let token_pair = TokenPair::new(usdc.clone(), weth.clone());
        
        let pool = UniswapV2Pool {
            address: "0x123...".to_string(),
            token_pair,
            reserve_a: Decimal::new(1000000, 6), // 1M USDC
            reserve_b: Decimal::new(500, 18), // 500 ETH
            total_supply: Decimal::new(22360679, 18), // sqrt(1M * 500)
            fee_rate: Decimal::new(3, 3), // 0.3%
            last_updated: Utc::now(),
        };

        // Test amount out calculation
        let amount_in = Decimal::new(1000, 6); // 1000 USDC
        let amount_out = pool.get_amount_out(amount_in, &usdc).unwrap();
        assert!(amount_out > Decimal::ZERO);

        // Test price impact calculation
        let price_impact = pool.calculate_price_impact(amount_in, &usdc).unwrap();
        assert!(price_impact >= Decimal::ZERO);
        assert!(price_impact < Decimal::new(100, 2)); // Less than 100%
    }

    #[test]
    fn test_amm_config_default() {
        let config = AMMConfig::default();
        assert_eq!(config.default_slippage_tolerance, Decimal::new(50, 4));
        assert_eq!(config.max_slippage_tolerance, Decimal::new(1000, 4));
        assert!(config.enable_mev_protection);
        assert!(config.enable_route_optimization);
        assert_eq!(config.max_routing_hops, 3);
    }
}
