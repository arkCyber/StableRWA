// =====================================================================================
// File: core-config/src/lib.rs
// Description: Centralized configuration management for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn};

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub blockchain: BlockchainConfig,
    pub security: SecurityConfig,
    pub observability: ObservabilityConfig,
    pub external_services: ExternalServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub keep_alive: Option<u64>,
    pub client_timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: u64,
    pub command_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub ethereum: EthereumConfig,
    pub solana: SolanaConfig,
    pub polkadot: PolkadotConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_limit: u64,
    pub gas_price: Option<u64>,
    pub private_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub commitment: String,
    pub keypair_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolkadotConfig {
    pub rpc_url: String,
    pub keypair_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub bcrypt_cost: u32,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub tracing: TracingConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub level: String,
    pub jaeger_endpoint: Option<String>,
    pub service_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    pub openai_api_key: Option<String>,
    pub notification_service_url: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                keep_alive: Some(75),
                client_timeout: Some(5000),
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/rwa_platform".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 1800,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
                connection_timeout: 5,
                command_timeout: 5,
            },
            blockchain: BlockchainConfig {
                ethereum: EthereumConfig {
                    rpc_url: "http://localhost:8545".to_string(),
                    chain_id: 1,
                    gas_limit: 21000,
                    gas_price: None,
                    private_key: None,
                },
                solana: SolanaConfig {
                    rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                    commitment: "confirmed".to_string(),
                    keypair_path: None,
                },
                polkadot: PolkadotConfig {
                    rpc_url: "wss://rpc.polkadot.io".to_string(),
                    keypair_path: None,
                },
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key".to_string(),
                jwt_expiration: 3600,
                bcrypt_cost: 12,
                rate_limit: RateLimitConfig {
                    requests_per_minute: 60,
                    burst_size: 10,
                },
            },
            observability: ObservabilityConfig {
                tracing: TracingConfig {
                    level: "info".to_string(),
                    jaeger_endpoint: None,
                    service_name: "rwa-platform".to_string(),
                },
                metrics: MetricsConfig {
                    enabled: true,
                    endpoint: "/metrics".to_string(),
                    port: 9090,
                },
            },
            external_services: ExternalServicesConfig {
                openai_api_key: None,
                notification_service_url: None,
            },
        }
    }
}

/// Configuration loader with environment override support
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from files and environment variables
    pub fn load() -> Result<AppConfig, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        
        info!("Loading configuration for environment: {}", run_mode);

        let config = Config::builder()
            // Start with default configuration
            .add_source(Config::try_from(&AppConfig::default())?)
            // Load environment-specific config file
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Load local config file (for development overrides)
            .add_source(File::with_name("config/local").required(false))
            // Override with environment variables (with RWA_ prefix)
            .add_source(Environment::with_prefix("RWA").separator("__"))
            .build()?;

        let app_config: AppConfig = config.try_deserialize()?;
        
        // Validate critical configuration
        Self::validate_config(&app_config)?;
        
        info!("Configuration loaded successfully");
        Ok(app_config)
    }

    /// Validate critical configuration values
    fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
        if config.security.jwt_secret == "your-secret-key" {
            warn!("Using default JWT secret - this should be changed in production!");
        }

        if config.database.url.contains("localhost") && env::var("RUN_MODE").unwrap_or_default() == "production" {
            return Err(ConfigError::Message(
                "Database URL should not use localhost in production".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.security.bcrypt_cost, 12);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.server.port, deserialized.server.port);
    }
}
