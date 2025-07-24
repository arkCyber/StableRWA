// =====================================================================================
// File: service-asset/src/config.rs
// Description: Enterprise-grade configuration management for Asset Service
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

/// Main service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Blockchain configuration
    pub blockchain: BlockchainConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Feature flags
    pub features: FeatureFlags,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitingConfig,
    /// Cache configuration
    pub cache: CacheConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: u32,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub client_shutdown: u64,
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_file: String,
    pub key_file: String,
    pub ca_file: Option<String>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub ssl_mode: String,
    pub migration_path: String,
}

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub networks: HashMap<String, NetworkConfig>,
    pub default_network: String,
    pub gas_price_strategy: GasPriceStrategy,
    pub confirmation_blocks: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_limit: u64,
    pub gas_price: Option<u64>,
    pub contracts: HashMap<String, ContractConfig>,
    pub explorer_url: Option<String>,
}

/// Contract configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    pub address: String,
    pub abi_path: String,
    pub deployment_block: Option<u64>,
}

/// Gas price strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GasPriceStrategy {
    Fixed(u64),
    Dynamic,
    Oracle(String),
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub cors_origins: Vec<String>,
    pub api_key_header: String,
    pub encryption_key: String,
    pub password_policy: PasswordPolicy,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u8,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
    pub log_level: String,
    pub jaeger_endpoint: Option<String>,
    pub prometheus_endpoint: Option<String>,
    pub health_check_interval: u64,
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub tokenization_enabled: bool,
    pub valuation_enabled: bool,
    pub metadata_enabled: bool,
    pub audit_logging: bool,
    pub rate_limiting: bool,
    pub caching: bool,
    pub blockchain_integration: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub redis_url: Option<String>,
    pub ttl_seconds: u64,
    pub max_entries: u32,
    pub compression: bool,
}

impl ServiceConfig {
    /// Load configuration from environment variables and config files
    pub fn load() -> Result<Self, ConfigError> {
        let mut config = Self::default();
        
        // Load from environment variables
        config.load_from_env()?;
        
        // Load from config file if specified
        if let Ok(config_path) = env::var("CONFIG_PATH") {
            config.load_from_file(&config_path)?;
        }
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load configuration from environment variables
    fn load_from_env(&mut self) -> Result<(), ConfigError> {
        // Server configuration
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            self.server.port = port.parse().map_err(|_| ConfigError::InvalidPort)?;
        }
        
        // Database configuration
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.database.url = db_url;
        }
        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            self.database.max_connections = max_conn.parse().map_err(|_| ConfigError::InvalidDatabaseConfig)?;
        }
        
        // Security configuration
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            self.security.jwt_secret = jwt_secret;
        }
        if let Ok(encryption_key) = env::var("ENCRYPTION_KEY") {
            self.security.encryption_key = encryption_key;
        }
        
        // Blockchain configuration
        if let Ok(eth_rpc) = env::var("ETHEREUM_RPC_URL") {
            if let Some(eth_config) = self.blockchain.networks.get_mut("ethereum") {
                eth_config.rpc_url = eth_rpc;
            }
        }
        
        Ok(())
    }
    
    /// Load configuration from file
    fn load_from_file(&mut self, path: &str) -> Result<(), ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_string()))?;
        
        let file_config: ServiceConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        // Merge configurations (file overrides defaults, env overrides file)
        *self = file_config;
        self.load_from_env()?;
        
        Ok(())
    }
    
    /// Validate configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(ConfigError::InvalidPort);
        }
        
        // Validate database configuration
        if self.database.url.is_empty() {
            return Err(ConfigError::InvalidDatabaseConfig);
        }
        
        // Validate security configuration
        if self.security.jwt_secret.len() < 32 {
            return Err(ConfigError::WeakJwtSecret);
        }
        
        if self.security.encryption_key.len() < 32 {
            return Err(ConfigError::WeakEncryptionKey);
        }
        
        // Validate blockchain configuration
        if self.blockchain.networks.is_empty() {
            return Err(ConfigError::NoBlockchainNetworks);
        }
        
        if !self.blockchain.networks.contains_key(&self.blockchain.default_network) {
            return Err(ConfigError::InvalidDefaultNetwork);
        }
        
        Ok(())
    }
    
    /// Get network configuration by name
    pub fn get_network(&self, name: &str) -> Option<&NetworkConfig> {
        self.blockchain.networks.get(name)
    }
    
    /// Get default network configuration
    pub fn get_default_network(&self) -> Option<&NetworkConfig> {
        self.blockchain.networks.get(&self.blockchain.default_network)
    }
    
    /// Check if feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "tokenization" => self.features.tokenization_enabled,
            "valuation" => self.features.valuation_enabled,
            "metadata" => self.features.metadata_enabled,
            "audit_logging" => self.features.audit_logging,
            "rate_limiting" => self.features.rate_limiting,
            "caching" => self.features.caching,
            "blockchain_integration" => self.features.blockchain_integration,
            _ => false,
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        let mut networks = HashMap::new();
        
        // Ethereum mainnet
        networks.insert("ethereum".to_string(), NetworkConfig {
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: 1,
            gas_limit: 21000,
            gas_price: None,
            contracts: HashMap::new(),
            explorer_url: Some("https://etherscan.io".to_string()),
        });
        
        // Ethereum testnet (Goerli)
        networks.insert("ethereum_testnet".to_string(), NetworkConfig {
            name: "Ethereum Goerli".to_string(),
            rpc_url: "https://goerli.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: 5,
            gas_limit: 21000,
            gas_price: None,
            contracts: HashMap::new(),
            explorer_url: Some("https://goerli.etherscan.io".to_string()),
        });
        
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                max_connections: 1000,
                keep_alive: 75,
                client_timeout: 5000,
                client_shutdown: 5000,
                tls: None,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/rwa_assets".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 1800,
                ssl_mode: "prefer".to_string(),
                migration_path: "./migrations".to_string(),
            },
            blockchain: BlockchainConfig {
                networks,
                default_network: "ethereum".to_string(),
                gas_price_strategy: GasPriceStrategy::Dynamic,
                confirmation_blocks: 12,
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            security: SecurityConfig {
                jwt_secret: "your-super-secret-jwt-key-here-must-be-at-least-32-chars".to_string(),
                jwt_expiration: 3600,
                cors_origins: vec!["*".to_string()],
                api_key_header: "X-API-Key".to_string(),
                encryption_key: "your-super-secret-encryption-key-32-chars".to_string(),
                password_policy: PasswordPolicy {
                    min_length: 8,
                    require_uppercase: true,
                    require_lowercase: true,
                    require_numbers: true,
                    require_symbols: false,
                },
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                tracing_enabled: true,
                log_level: "info".to_string(),
                jaeger_endpoint: None,
                prometheus_endpoint: None,
                health_check_interval: 30,
            },
            features: FeatureFlags {
                tokenization_enabled: true,
                valuation_enabled: true,
                metadata_enabled: true,
                audit_logging: true,
                rate_limiting: true,
                caching: true,
                blockchain_integration: true,
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                requests_per_minute: 100,
                burst_size: 10,
                whitelist: vec![],
                blacklist: vec![],
            },
            cache: CacheConfig {
                enabled: true,
                redis_url: Some("redis://localhost:6379".to_string()),
                ttl_seconds: 300,
                max_entries: 10000,
                compression: true,
            },
        }
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    
    #[error("Invalid port number")]
    InvalidPort,
    
    #[error("Invalid database configuration")]
    InvalidDatabaseConfig,
    
    #[error("JWT secret must be at least 32 characters")]
    WeakJwtSecret,
    
    #[error("Encryption key must be at least 32 characters")]
    WeakEncryptionKey,
    
    #[error("No blockchain networks configured")]
    NoBlockchainNetworks,
    
    #[error("Invalid default network")]
    InvalidDefaultNetwork,
    
    #[error("Environment variable error: {0}")]
    EnvError(String),
}

impl NetworkConfig {
    pub fn ethereum_mainnet() -> Self {
        Self {
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: 1,
            gas_limit: 21000,
            gas_price: None,
            contracts: HashMap::new(),
            explorer_url: Some("https://etherscan.io".to_string()),
        }
    }
    
    pub fn ethereum_testnet() -> Self {
        Self {
            name: "Ethereum Goerli".to_string(),
            rpc_url: "https://goerli.infura.io/v3/YOUR_PROJECT_ID".to_string(),
            chain_id: 5,
            gas_limit: 21000,
            gas_price: None,
            contracts: HashMap::new(),
            explorer_url: Some("https://goerli.etherscan.io".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = ServiceConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.features.tokenization_enabled);
        assert!(config.blockchain.networks.contains_key("ethereum"));
    }

    #[test]
    fn test_config_validation() {
        let mut config = ServiceConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid port should fail
        config.server.port = 0;
        assert!(config.validate().is_err());
        
        // Reset and test weak JWT secret
        config = ServiceConfig::default();
        config.security.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_feature_flags() {
        let config = ServiceConfig::default();
        assert!(config.is_feature_enabled("tokenization"));
        assert!(config.is_feature_enabled("valuation"));
        assert!(!config.is_feature_enabled("unknown_feature"));
    }

    #[test]
    fn test_network_config() {
        let config = ServiceConfig::default();
        let eth_config = config.get_network("ethereum");
        assert!(eth_config.is_some());
        assert_eq!(eth_config.unwrap().chain_id, 1);
        
        let default_network = config.get_default_network();
        assert!(default_network.is_some());
    }
}
