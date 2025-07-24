// =====================================================================================
// RWA Tokenization Platform - Custody Service Configuration
// 
// Configuration management for custody service including database connections,
// security settings, provider integrations, and operational parameters.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{CustodyError, CustodyResult};
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main configuration structure for the custody service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyConfig {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Redis cache configuration
    pub redis: RedisConfig,
    /// Security and cryptography settings
    pub security: SecurityConfig,
    /// Digital asset custody configuration
    pub digital_custody: DigitalCustodyConfig,
    /// Physical asset custody configuration
    pub physical_custody: PhysicalCustodyConfig,
    /// Insurance integration configuration
    pub insurance: InsuranceConfig,
    /// Monitoring and metrics configuration
    pub monitoring: MonitoringConfig,
    /// External service integrations
    pub integrations: IntegrationsConfig,
}

/// Server configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port number
    pub port: u16,
    /// Maximum number of worker threads
    pub workers: Option<usize>,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
    /// Enable CORS
    pub cors_enabled: bool,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
}

/// Database configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Query timeout in seconds
    pub query_timeout: u64,
    /// Enable database migrations
    pub auto_migrate: bool,
}

/// Redis cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,
    /// Connection pool size
    pub pool_size: u32,
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Default TTL for cached items in seconds
    pub default_ttl: u64,
}

/// Security and cryptography configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// JWT secret key for authentication
    pub jwt_secret: String,
    /// JWT token expiration time in seconds
    pub jwt_expiration: u64,
    /// Encryption key for sensitive data
    pub encryption_key: String,
    /// HSM configuration (if enabled)
    pub hsm: Option<HsmConfig>,
    /// Key derivation settings
    pub key_derivation: KeyDerivationConfig,
    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,
}

/// Hardware Security Module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    /// HSM provider type
    pub provider: String,
    /// HSM connection URL or path
    pub connection: String,
    /// HSM authentication credentials
    pub credentials: HashMap<String, String>,
    /// HSM slot or partition identifier
    pub slot_id: Option<u32>,
    /// HSM PIN or password
    pub pin: Option<String>,
}

/// Key derivation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    /// Key derivation algorithm (e.g., "PBKDF2", "Argon2")
    pub algorithm: String,
    /// Number of iterations for key derivation
    pub iterations: u32,
    /// Salt length in bytes
    pub salt_length: usize,
    /// Derived key length in bytes
    pub key_length: usize,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per minute limit
    pub requests_per_minute: u32,
    /// Burst capacity
    pub burst_capacity: u32,
    /// Rate limit window in seconds
    pub window_seconds: u64,
}

/// Digital asset custody configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalCustodyConfig {
    /// Supported blockchain networks
    pub networks: HashMap<String, NetworkConfig>,
    /// Multi-signature wallet settings
    pub multisig: MultiSigConfig,
    /// Hot wallet configuration
    pub hot_wallet: WalletConfig,
    /// Cold wallet configuration
    pub cold_wallet: WalletConfig,
}

/// Blockchain network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network RPC endpoint
    pub rpc_url: String,
    /// WebSocket endpoint (if available)
    pub ws_url: Option<String>,
    /// Network chain ID
    pub chain_id: u64,
    /// Block confirmation requirements
    pub confirmations: u32,
    /// Gas price settings
    pub gas_settings: GasSettings,
}

/// Gas price configuration for blockchain transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasSettings {
    /// Default gas price in wei
    pub default_gas_price: u64,
    /// Maximum gas price in wei
    pub max_gas_price: u64,
    /// Gas price multiplier for priority transactions
    pub priority_multiplier: f64,
}

/// Multi-signature wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigConfig {
    /// Default required signatures
    pub default_required: u32,
    /// Default total signers
    pub default_total: u32,
    /// Signer key management
    pub signer_management: SignerManagementConfig,
}

/// Signer management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerManagementConfig {
    /// Key rotation interval in days
    pub rotation_interval_days: u32,
    /// Backup key storage location
    pub backup_location: String,
    /// Key recovery settings
    pub recovery_settings: KeyRecoveryConfig,
}

/// Key recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRecoveryConfig {
    /// Enable key recovery
    pub enabled: bool,
    /// Recovery threshold (number of shares needed)
    pub threshold: u32,
    /// Total number of recovery shares
    pub total_shares: u32,
}

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Wallet type (hot/cold)
    pub wallet_type: String,
    /// Storage location or HSM reference
    pub storage: String,
    /// Encryption settings
    pub encryption: EncryptionConfig,
    /// Access control settings
    pub access_control: AccessControlConfig,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption algorithm
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
    /// Initialization vector size
    pub iv_size: u32,
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Required authentication methods
    pub auth_methods: Vec<String>,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Maximum failed attempts
    pub max_failed_attempts: u32,
}

/// Physical asset custody configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalCustodyConfig {
    /// Custody provider integrations
    pub providers: HashMap<String, ProviderConfig>,
    /// Asset verification settings
    pub verification: VerificationConfig,
    /// Storage facility requirements
    pub storage_requirements: StorageRequirements,
}

/// Custody provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider API endpoint
    pub api_url: String,
    /// API authentication credentials
    pub credentials: HashMap<String, String>,
    /// Provider-specific settings
    pub settings: HashMap<String, String>,
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Retry configuration
    pub retry: RetryConfig,
}

/// Retry configuration for external services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,
}

/// Asset verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Enable automated verification
    pub automated: bool,
    /// Verification interval in hours
    pub interval_hours: u32,
    /// Required verification methods
    pub methods: Vec<String>,
    /// Verification timeout in seconds
    pub timeout: u64,
}

/// Storage facility requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirements {
    /// Required security certifications
    pub certifications: Vec<String>,
    /// Minimum insurance coverage
    pub min_insurance_coverage: u64,
    /// Environmental requirements
    pub environmental: EnvironmentalRequirements,
    /// Access control requirements
    pub access_control: Vec<String>,
}

/// Environmental storage requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalRequirements {
    /// Temperature range (min, max) in Celsius
    pub temperature_range: (f32, f32),
    /// Humidity range (min, max) in percentage
    pub humidity_range: (f32, f32),
    /// Fire suppression system required
    pub fire_suppression: bool,
    /// Climate control required
    pub climate_control: bool,
}

/// Insurance integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceConfig {
    /// Insurance provider integrations
    pub providers: HashMap<String, ProviderConfig>,
    /// Default coverage settings
    pub default_coverage: CoverageConfig,
    /// Policy management settings
    pub policy_management: PolicyManagementConfig,
}

/// Insurance coverage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageConfig {
    /// Default coverage percentage
    pub coverage_percentage: f64,
    /// Minimum coverage amount
    pub min_coverage_amount: u64,
    /// Maximum coverage amount
    pub max_coverage_amount: u64,
    /// Coverage types
    pub coverage_types: Vec<String>,
}

/// Policy management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyManagementConfig {
    /// Automatic policy renewal
    pub auto_renewal: bool,
    /// Renewal notice period in days
    pub renewal_notice_days: u32,
    /// Policy review interval in days
    pub review_interval_days: u32,
}

/// Monitoring and metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Metrics endpoint path
    pub metrics_path: String,
    /// Health check endpoint path
    pub health_path: String,
    /// Log level
    pub log_level: String,
    /// Log format
    pub log_format: String,
}

/// External service integrations configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    /// Blockchain explorer APIs
    pub explorers: HashMap<String, String>,
    /// Price feed APIs
    pub price_feeds: HashMap<String, String>,
    /// Notification services
    pub notifications: HashMap<String, String>,
    /// Audit services
    pub audit_services: HashMap<String, String>,
}

impl Default for CustodyConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                timeout: 30,
                max_body_size: 1024 * 1024, // 1MB
                cors_enabled: true,
                cors_origins: vec!["*".to_string()],
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/custody".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 30,
                query_timeout: 30,
                auto_migrate: true,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
                timeout: 5,
                default_ttl: 3600,
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key".to_string(),
                jwt_expiration: 3600,
                encryption_key: "your-encryption-key".to_string(),
                hsm: None,
                key_derivation: KeyDerivationConfig {
                    algorithm: "PBKDF2".to_string(),
                    iterations: 100000,
                    salt_length: 32,
                    key_length: 32,
                },
                rate_limiting: RateLimitConfig {
                    enabled: true,
                    requests_per_minute: 100,
                    burst_capacity: 10,
                    window_seconds: 60,
                },
            },
            digital_custody: DigitalCustodyConfig {
                networks: HashMap::new(),
                multisig: MultiSigConfig {
                    default_required: 2,
                    default_total: 3,
                    signer_management: SignerManagementConfig {
                        rotation_interval_days: 90,
                        backup_location: "/secure/backup".to_string(),
                        recovery_settings: KeyRecoveryConfig {
                            enabled: true,
                            threshold: 3,
                            total_shares: 5,
                        },
                    },
                },
                hot_wallet: WalletConfig {
                    wallet_type: "hot".to_string(),
                    storage: "database".to_string(),
                    encryption: EncryptionConfig {
                        algorithm: "AES-256-GCM".to_string(),
                        key_size: 256,
                        iv_size: 12,
                    },
                    access_control: AccessControlConfig {
                        auth_methods: vec!["jwt".to_string(), "mfa".to_string()],
                        session_timeout: 1800,
                        max_failed_attempts: 3,
                    },
                },
                cold_wallet: WalletConfig {
                    wallet_type: "cold".to_string(),
                    storage: "hsm".to_string(),
                    encryption: EncryptionConfig {
                        algorithm: "AES-256-GCM".to_string(),
                        key_size: 256,
                        iv_size: 12,
                    },
                    access_control: AccessControlConfig {
                        auth_methods: vec!["jwt".to_string(), "mfa".to_string(), "hardware_token".to_string()],
                        session_timeout: 900,
                        max_failed_attempts: 2,
                    },
                },
            },
            physical_custody: PhysicalCustodyConfig {
                providers: HashMap::new(),
                verification: VerificationConfig {
                    automated: true,
                    interval_hours: 24,
                    methods: vec!["visual".to_string(), "rfid".to_string()],
                    timeout: 300,
                },
                storage_requirements: StorageRequirements {
                    certifications: vec!["ISO27001".to_string(), "SOC2".to_string()],
                    min_insurance_coverage: 1000000,
                    environmental: EnvironmentalRequirements {
                        temperature_range: (18.0, 24.0),
                        humidity_range: (40.0, 60.0),
                        fire_suppression: true,
                        climate_control: true,
                    },
                    access_control: vec!["biometric".to_string(), "card_access".to_string()],
                },
            },
            insurance: InsuranceConfig {
                providers: HashMap::new(),
                default_coverage: CoverageConfig {
                    coverage_percentage: 100.0,
                    min_coverage_amount: 10000,
                    max_coverage_amount: 100000000,
                    coverage_types: vec!["theft".to_string(), "damage".to_string(), "loss".to_string()],
                },
                policy_management: PolicyManagementConfig {
                    auto_renewal: true,
                    renewal_notice_days: 30,
                    review_interval_days: 90,
                },
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_path: "/metrics".to_string(),
                health_path: "/health".to_string(),
                log_level: "info".to_string(),
                log_format: "json".to_string(),
            },
            integrations: IntegrationsConfig {
                explorers: HashMap::new(),
                price_feeds: HashMap::new(),
                notifications: HashMap::new(),
                audit_services: HashMap::new(),
            },
        }
    }
}

impl CustodyConfig {
    /// Load configuration from environment variables and config files
    pub fn from_env() -> CustodyResult<Self> {
        let mut config = Config::builder()
            .add_source(File::with_name("config/custody").required(false))
            .add_source(File::with_name("config/custody.local").required(false))
            .add_source(Environment::with_prefix("CUSTODY").separator("__"));

        if let Ok(config_path) = std::env::var("CUSTODY_CONFIG_PATH") {
            config = config.add_source(File::with_name(&config_path).required(true));
        }

        let config = config.build()?;
        let custody_config: CustodyConfig = config.try_deserialize()?;
        
        custody_config.validate()?;
        Ok(custody_config)
    }

    /// Load configuration from a specific file
    pub fn from_file<P: AsRef<Path>>(path: P) -> CustodyResult<Self> {
        let config = Config::builder()
            .add_source(File::from(path.as_ref()))
            .build()?;
        
        let custody_config: CustodyConfig = config.try_deserialize()?;
        custody_config.validate()?;
        Ok(custody_config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> CustodyResult<()> {
        // Validate server configuration
        if self.server.port == 0 {
            return Err(CustodyError::configuration("Server port cannot be 0"));
        }

        // Validate database configuration
        if self.database.url.is_empty() {
            return Err(CustodyError::configuration("Database URL cannot be empty"));
        }

        // Validate security configuration
        if self.security.jwt_secret.len() < 32 {
            return Err(CustodyError::configuration("JWT secret must be at least 32 characters"));
        }

        if self.security.encryption_key.len() < 32 {
            return Err(CustodyError::configuration("Encryption key must be at least 32 characters"));
        }

        // Validate multi-signature configuration
        if self.digital_custody.multisig.default_required == 0 {
            return Err(CustodyError::configuration("Multi-sig required signatures cannot be 0"));
        }

        if self.digital_custody.multisig.default_required > self.digital_custody.multisig.default_total {
            return Err(CustodyError::configuration("Multi-sig required cannot exceed total signers"));
        }

        Ok(())
    }

    /// Get network configuration by name
    pub fn get_network_config(&self, network: &str) -> Option<&NetworkConfig> {
        self.digital_custody.networks.get(network)
    }

    /// Get provider configuration by name
    pub fn get_provider_config(&self, provider: &str) -> Option<&ProviderConfig> {
        self.physical_custody.providers.get(provider)
            .or_else(|| self.insurance.providers.get(provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = CustodyConfig::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.security.rate_limiting.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = CustodyConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid port should fail
        config.server.port = 0;
        assert!(config.validate().is_err());
        
        // Reset port and test empty database URL
        config.server.port = 8080;
        config.database.url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_security_validation() {
        let mut config = CustodyConfig::default();
        
        // Short JWT secret should fail
        config.security.jwt_secret = "short".to_string();
        assert!(config.validate().is_err());
        
        // Short encryption key should fail
        config.security.jwt_secret = "a".repeat(32);
        config.security.encryption_key = "short".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_multisig_validation() {
        let mut config = CustodyConfig::default();
        
        // Required > total should fail
        config.digital_custody.multisig.default_required = 5;
        config.digital_custody.multisig.default_total = 3;
        assert!(config.validate().is_err());
        
        // Required = 0 should fail
        config.digital_custody.multisig.default_required = 0;
        config.digital_custody.multisig.default_total = 3;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_network_config_retrieval() {
        let mut config = CustodyConfig::default();
        
        // Add a test network
        let network_config = NetworkConfig {
            rpc_url: "https://mainnet.infura.io".to_string(),
            ws_url: None,
            chain_id: 1,
            confirmations: 12,
            gas_settings: GasSettings {
                default_gas_price: 20000000000,
                max_gas_price: 100000000000,
                priority_multiplier: 1.2,
            },
        };
        
        config.digital_custody.networks.insert("ethereum".to_string(), network_config);
        
        let retrieved = config.get_network_config("ethereum");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().chain_id, 1);
        
        let non_existent = config.get_network_config("bitcoin");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_environmental_requirements() {
        let requirements = EnvironmentalRequirements {
            temperature_range: (18.0, 24.0),
            humidity_range: (40.0, 60.0),
            fire_suppression: true,
            climate_control: true,
        };

        assert_eq!(requirements.temperature_range.0, 18.0);
        assert_eq!(requirements.temperature_range.1, 24.0);
        assert!(requirements.fire_suppression);
        assert!(requirements.climate_control);
    }

    #[test]
    fn test_retry_config() {
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,
        };

        assert_eq!(retry_config.max_attempts, 3);
        assert_eq!(retry_config.backoff_multiplier, 2.0);
    }
}
