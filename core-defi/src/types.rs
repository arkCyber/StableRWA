// =====================================================================================
// File: core-defi/src/types.rs
// Description: Common types for DeFi services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Token information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain_id: u64,
    pub logo_uri: Option<String>,
    pub is_native: bool,
    pub is_verified: bool,
    pub tags: Vec<String>,
}

impl Token {
    /// Create a new token
    pub fn new(
        address: String,
        symbol: String,
        name: String,
        decimals: u8,
        chain_id: u64,
    ) -> Self {
        Self {
            address,
            symbol,
            name,
            decimals,
            chain_id,
            logo_uri: None,
            is_native: false,
            is_verified: false,
            tags: vec![],
        }
    }

    /// Check if token is stablecoin
    pub fn is_stablecoin(&self) -> bool {
        self.tags.contains(&"stablecoin".to_string()) ||
        matches!(self.symbol.as_str(), "USDC" | "USDT" | "DAI" | "BUSD" | "FRAX")
    }

    /// Check if token is wrapped native token
    pub fn is_wrapped_native(&self) -> bool {
        matches!(self.symbol.as_str(), "WETH" | "WBNB" | "WMATIC" | "WAVAX")
    }
}

/// Token pair for trading
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenPair {
    pub token_a: Token,
    pub token_b: Token,
    pub fee_tier: Option<u32>, // For Uniswap V3 style pools
}

impl TokenPair {
    /// Create a new token pair
    pub fn new(token_a: Token, token_b: Token) -> Self {
        Self {
            token_a,
            token_b,
            fee_tier: None,
        }
    }

    /// Create a new token pair with fee tier
    pub fn with_fee_tier(token_a: Token, token_b: Token, fee_tier: u32) -> Self {
        Self {
            token_a,
            token_b,
            fee_tier: Some(fee_tier),
        }
    }

    /// Get pair symbol
    pub fn symbol(&self) -> String {
        format!("{}/{}", self.token_a.symbol, self.token_b.symbol)
    }

    /// Get reversed pair
    pub fn reversed(&self) -> Self {
        Self {
            token_a: self.token_b.clone(),
            token_b: self.token_a.clone(),
            fee_tier: self.fee_tier,
        }
    }
}

/// Liquidity pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub id: Uuid,
    pub address: String,
    pub protocol: AMMProtocol,
    pub token_pair: TokenPair,
    pub reserve_a: Decimal,
    pub reserve_b: Decimal,
    pub total_supply: Decimal,
    pub fee_rate: Decimal,
    pub volume_24h: Decimal,
    pub fees_24h: Decimal,
    pub apy: Decimal,
    pub tvl: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl LiquidityPool {
    /// Calculate current price of token A in terms of token B
    pub fn price_a_in_b(&self) -> Decimal {
        if self.reserve_a.is_zero() {
            Decimal::ZERO
        } else {
            self.reserve_b / self.reserve_a
        }
    }

    /// Calculate current price of token B in terms of token A
    pub fn price_b_in_a(&self) -> Decimal {
        if self.reserve_b.is_zero() {
            Decimal::ZERO
        } else {
            self.reserve_a / self.reserve_b
        }
    }

    /// Calculate pool utilization
    pub fn utilization(&self) -> Decimal {
        if self.total_supply.is_zero() {
            Decimal::ZERO
        } else {
            (self.reserve_a + self.reserve_b) / self.total_supply
        }
    }
}

/// AMM protocol enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AMMProtocol {
    UniswapV2,
    UniswapV3,
    SushiSwap,
    PancakeSwap,
    Curve,
    Balancer,
    Bancor,
    Kyber,
    OneInch,
    Custom(u32),
}

impl AMMProtocol {
    /// Get protocol name
    pub fn name(&self) -> &'static str {
        match self {
            AMMProtocol::UniswapV2 => "Uniswap V2",
            AMMProtocol::UniswapV3 => "Uniswap V3",
            AMMProtocol::SushiSwap => "SushiSwap",
            AMMProtocol::PancakeSwap => "PancakeSwap",
            AMMProtocol::Curve => "Curve",
            AMMProtocol::Balancer => "Balancer",
            AMMProtocol::Bancor => "Bancor",
            AMMProtocol::Kyber => "Kyber",
            AMMProtocol::OneInch => "1inch",
            AMMProtocol::Custom(_) => "Custom",
        }
    }

    /// Check if protocol supports concentrated liquidity
    pub fn supports_concentrated_liquidity(&self) -> bool {
        matches!(self, AMMProtocol::UniswapV3)
    }

    /// Check if protocol supports multiple fee tiers
    pub fn supports_fee_tiers(&self) -> bool {
        matches!(self, AMMProtocol::UniswapV3 | AMMProtocol::Balancer)
    }
}

/// Lending protocol enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LendingProtocol {
    Compound,
    Aave,
    MakerDAO,
    Venus,
    Cream,
    Benqi,
    Radiant,
    Custom(u32),
}

impl LendingProtocol {
    /// Get protocol name
    pub fn name(&self) -> &'static str {
        match self {
            LendingProtocol::Compound => "Compound",
            LendingProtocol::Aave => "Aave",
            LendingProtocol::MakerDAO => "MakerDAO",
            LendingProtocol::Venus => "Venus",
            LendingProtocol::Cream => "Cream",
            LendingProtocol::Benqi => "Benqi",
            LendingProtocol::Radiant => "Radiant",
            LendingProtocol::Custom(_) => "Custom",
        }
    }

    /// Check if protocol supports flash loans
    pub fn supports_flash_loans(&self) -> bool {
        matches!(
            self,
            LendingProtocol::Aave | LendingProtocol::Compound | LendingProtocol::Radiant
        )
    }

    /// Check if protocol supports variable rates
    pub fn supports_variable_rates(&self) -> bool {
        matches!(
            self,
            LendingProtocol::Aave | LendingProtocol::Compound | LendingProtocol::Venus
        )
    }
}

/// Staking protocol enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StakingProtocol {
    Ethereum2,
    Lido,
    RocketPool,
    Frax,
    Ankr,
    Binance,
    Coinbase,
    Custom(u32),
}

impl StakingProtocol {
    /// Get protocol name
    pub fn name(&self) -> &'static str {
        match self {
            StakingProtocol::Ethereum2 => "Ethereum 2.0",
            StakingProtocol::Lido => "Lido",
            StakingProtocol::RocketPool => "Rocket Pool",
            StakingProtocol::Frax => "Frax",
            StakingProtocol::Ankr => "Ankr",
            StakingProtocol::Binance => "Binance Staking",
            StakingProtocol::Coinbase => "Coinbase Staking",
            StakingProtocol::Custom(_) => "Custom",
        }
    }

    /// Check if protocol supports liquid staking
    pub fn supports_liquid_staking(&self) -> bool {
        matches!(
            self,
            StakingProtocol::Lido | StakingProtocol::RocketPool | StakingProtocol::Frax
        )
    }
}

/// Yield farming protocol enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YieldFarmingProtocol {
    Convex,
    Yearn,
    Harvest,
    Beefy,
    Autofarm,
    PancakeBunny,
    Alpaca,
    Custom(u32),
}

impl YieldFarmingProtocol {
    /// Get protocol name
    pub fn name(&self) -> &'static str {
        match self {
            YieldFarmingProtocol::Convex => "Convex",
            YieldFarmingProtocol::Yearn => "Yearn",
            YieldFarmingProtocol::Harvest => "Harvest",
            YieldFarmingProtocol::Beefy => "Beefy",
            YieldFarmingProtocol::Autofarm => "Autofarm",
            YieldFarmingProtocol::PancakeBunny => "PancakeBunny",
            YieldFarmingProtocol::Alpaca => "Alpaca",
            YieldFarmingProtocol::Custom(_) => "Custom",
        }
    }

    /// Check if protocol supports auto-compounding
    pub fn supports_auto_compounding(&self) -> bool {
        matches!(
            self,
            YieldFarmingProtocol::Convex
                | YieldFarmingProtocol::Yearn
                | YieldFarmingProtocol::Beefy
                | YieldFarmingProtocol::Autofarm
        )
    }
}

/// Position in a DeFi protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub user_id: String,
    pub protocol: String,
    pub position_type: PositionType,
    pub token_address: String,
    pub amount: Decimal,
    pub shares: Decimal,
    pub entry_price: Decimal,
    pub current_price: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub fees_paid: Decimal,
    pub rewards_earned: Decimal,
    pub opened_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
}

/// Position type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionType {
    LiquidityProvider,
    Lender,
    Borrower,
    Staker,
    YieldFarmer,
    LeveragedPosition,
    ShortPosition,
    OptionPosition,
    FuturePosition,
}

/// Strategy for automated DeFi operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub strategy_type: StrategyType,
    pub protocols: Vec<String>,
    pub tokens: Vec<Token>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub risk_level: RiskLevel,
    pub target_apy: Decimal,
    pub max_drawdown: Decimal,
    pub min_investment: Decimal,
    pub max_investment: Option<Decimal>,
    pub auto_rebalance: bool,
    pub rebalance_threshold: Decimal,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Strategy type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    YieldFarming,
    LiquidityMining,
    Arbitrage,
    DeltaNeutral,
    LeveragedYield,
    StablecoinYield,
    LiquidityProvision,
    Staking,
    LendingOptimization,
    CrossProtocolYield,
}

/// Risk level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl RiskLevel {
    /// Get risk score (0.0 to 1.0)
    pub fn score(&self) -> f64 {
        match self {
            RiskLevel::VeryLow => 0.1,
            RiskLevel::Low => 0.3,
            RiskLevel::Medium => 0.5,
            RiskLevel::High => 0.7,
            RiskLevel::VeryHigh => 0.9,
        }
    }

    /// Get maximum leverage for risk level
    pub fn max_leverage(&self) -> Decimal {
        match self {
            RiskLevel::VeryLow => Decimal::new(110, 2), // 1.1x
            RiskLevel::Low => Decimal::new(150, 2), // 1.5x
            RiskLevel::Medium => Decimal::new(200, 2), // 2.0x
            RiskLevel::High => Decimal::new(300, 2), // 3.0x
            RiskLevel::VeryHigh => Decimal::new(500, 2), // 5.0x
        }
    }
}

/// Price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub token_address: String,
    pub price_usd: Decimal,
    pub price_change_24h: Decimal,
    pub volume_24h: Decimal,
    pub market_cap: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// Gas price information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrice {
    pub slow: u64,
    pub standard: u64,
    pub fast: u64,
    pub fastest: u64,
    pub timestamp: DateTime<Utc>,
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub explorer_url: String,
    pub native_token: Token,
    pub block_time: u64,
    pub finality_blocks: u64,
    pub is_testnet: bool,
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new(
            "0xA0b86a33E6441b8435b662f0E2d0B8A0E6E6E6E6".to_string(),
            "USDC".to_string(),
            "USD Coin".to_string(),
            6,
            1,
        );

        assert_eq!(token.symbol, "USDC");
        assert_eq!(token.decimals, 6);
        assert!(token.is_stablecoin());
        assert!(!token.is_wrapped_native());
    }

    #[test]
    fn test_token_pair() {
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

        let pair = TokenPair::with_fee_tier(usdc, weth, 3000);
        assert_eq!(pair.symbol(), "USDC/WETH");
        assert_eq!(pair.fee_tier, Some(3000));

        let reversed = pair.reversed();
        assert_eq!(reversed.symbol(), "WETH/USDC");
    }

    #[test]
    fn test_protocol_features() {
        assert!(AMMProtocol::UniswapV3.supports_concentrated_liquidity());
        assert!(!AMMProtocol::UniswapV2.supports_concentrated_liquidity());

        assert!(LendingProtocol::Aave.supports_flash_loans());
        assert!(!LendingProtocol::MakerDAO.supports_flash_loans());

        assert!(StakingProtocol::Lido.supports_liquid_staking());
        assert!(!StakingProtocol::Ethereum2.supports_liquid_staking());
    }

    #[test]
    fn test_risk_level() {
        assert_eq!(RiskLevel::Low.score(), 0.3);
        assert_eq!(RiskLevel::High.score(), 0.7);
        
        assert_eq!(RiskLevel::Medium.max_leverage(), Decimal::new(200, 2));
        assert_eq!(RiskLevel::VeryHigh.max_leverage(), Decimal::new(500, 2));
    }
}
