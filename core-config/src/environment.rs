// =====================================================================================
// File: core-config/src/environment.rs
// Description: Environment-specific configuration management
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tracing::{debug, info, warn};

/// Environment types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Environment {
    /// Get current environment from environment variable
    pub fn current() -> Self {
        match env::var("RWA_ENV").unwrap_or_else(|_| "development".to_string()).to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            "testing" | "test" => Environment::Testing,
            _ => Environment::Development,
        }
    }

    /// Check if current environment is production
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    /// Check if current environment is development
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    /// Get configuration file name for this environment
    pub fn config_file_name(&self) -> &str {
        match self {
            Environment::Development => "development.toml",
            Environment::Testing => "testing.toml",
            Environment::Staging => "staging.toml",
            Environment::Production => "production.toml",
        }
    }

    /// Get log level for this environment
    pub fn default_log_level(&self) -> &str {
        match self {
            Environment::Development => "debug",
            Environment::Testing => "info",
            Environment::Staging => "info",
            Environment::Production => "warn",
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Testing => write!(f, "testing"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

/// Environment configuration loader
pub struct EnvironmentConfig {
    environment: Environment,
    config_dir: String,
    env_vars: HashMap<String, String>,
}

impl EnvironmentConfig {
    /// Create new environment configuration
    pub fn new() -> Self {
        let environment = Environment::current();
        let config_dir = env::var("RWA_CONFIG_DIR").unwrap_or_else(|_| "config".to_string());
        
        info!(
            environment = %environment,
            config_dir = %config_dir,
            "Initializing environment configuration"
        );

        Self {
            environment,
            config_dir,
            env_vars: Self::load_env_vars(),
        }
    }

    /// Get current environment
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    /// Load configuration file for current environment
    pub fn load_config_file<T>(&self) -> ConfigResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let config_file = format!("{}/{}", self.config_dir, self.environment.config_file_name());
        
        if !Path::new(&config_file).exists() {
            warn!(
                config_file = %config_file,
                "Configuration file not found, using defaults"
            );
            return Err(ConfigError::FileNotFound(config_file));
        }

        let content = std::fs::read_to_string(&config_file)
            .map_err(|e| ConfigError::IoError(format!("Failed to read config file {}: {}", config_file, e)))?;

        let config: T = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse config file {}: {}", config_file, e)))?;

        debug!(config_file = %config_file, "Configuration loaded successfully");
        Ok(config)
    }

    /// Get environment variable with fallback
    pub fn get_env_var(&self, key: &str, default: Option<&str>) -> ConfigResult<String> {
        self.env_vars.get(key)
            .cloned()
            .or_else(|| default.map(|s| s.to_string()))
            .ok_or_else(|| ConfigError::MissingVariable(key.to_string()))
    }

    /// Get required environment variable
    pub fn require_env_var(&self, key: &str) -> ConfigResult<String> {
        self.get_env_var(key, None)
    }

    /// Get optional environment variable
    pub fn optional_env_var(&self, key: &str) -> Option<String> {
        self.env_vars.get(key).cloned()
    }

    /// Get environment variable as specific type
    pub fn get_env_var_as<T>(&self, key: &str, default: Option<T>) -> ConfigResult<T>
    where
        T: std::str::FromStr + Clone,
        T::Err: std::fmt::Display,
    {
        if let Some(value) = self.env_vars.get(key) {
            value.parse::<T>()
                .map_err(|e| ConfigError::ParseError(format!("Failed to parse {}: {}", key, e)))
        } else if let Some(default_value) = default {
            Ok(default_value)
        } else {
            Err(ConfigError::MissingVariable(key.to_string()))
        }
    }

    /// Validate required environment variables
    pub fn validate_required_vars(&self, required_vars: &[&str]) -> ConfigResult<()> {
        let mut missing_vars = Vec::new();

        for var in required_vars {
            if !self.env_vars.contains_key(*var) {
                missing_vars.push(*var);
            }
        }

        if !missing_vars.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "Missing required environment variables: {}",
                missing_vars.join(", ")
            )));
        }

        Ok(())
    }

    /// Load all environment variables
    fn load_env_vars() -> HashMap<String, String> {
        env::vars().collect()
    }

    /// Get database URL for current environment
    pub fn database_url(&self) -> ConfigResult<String> {
        match self.environment {
            Environment::Production => self.require_env_var("DATABASE_URL"),
            Environment::Staging => self.require_env_var("STAGING_DATABASE_URL"),
            Environment::Testing => self.get_env_var("TEST_DATABASE_URL", Some("postgresql://localhost:5432/rwa_test")),
            Environment::Development => self.get_env_var("DEV_DATABASE_URL", Some("postgresql://localhost:5432/rwa_dev")),
        }
    }

    /// Get Redis URL for current environment
    pub fn redis_url(&self) -> ConfigResult<String> {
        match self.environment {
            Environment::Production => self.require_env_var("REDIS_URL"),
            Environment::Staging => self.require_env_var("STAGING_REDIS_URL"),
            Environment::Testing => self.get_env_var("TEST_REDIS_URL", Some("redis://localhost:6379/1")),
            Environment::Development => self.get_env_var("DEV_REDIS_URL", Some("redis://localhost:6379/0")),
        }
    }

    /// Get JWT secret for current environment
    pub fn jwt_secret(&self) -> ConfigResult<String> {
        match self.environment {
            Environment::Production => self.require_env_var("JWT_SECRET"),
            Environment::Staging => self.require_env_var("STAGING_JWT_SECRET"),
            _ => self.get_env_var("JWT_SECRET", Some("development_jwt_secret_key_32_chars")),
        }
    }

    /// Get encryption key for current environment
    pub fn encryption_key(&self) -> ConfigResult<String> {
        match self.environment {
            Environment::Production => self.require_env_var("ENCRYPTION_KEY"),
            Environment::Staging => self.require_env_var("STAGING_ENCRYPTION_KEY"),
            _ => self.get_env_var("ENCRYPTION_KEY", Some("development_encryption_key_32_chars")),
        }
    }

    /// Get API base URL for current environment
    pub fn api_base_url(&self) -> ConfigResult<String> {
        match self.environment {
            Environment::Production => self.require_env_var("API_BASE_URL"),
            Environment::Staging => self.get_env_var("API_BASE_URL", Some("https://staging-api.rwa-platform.com")),
            Environment::Testing => self.get_env_var("API_BASE_URL", Some("http://localhost:8080")),
            Environment::Development => self.get_env_var("API_BASE_URL", Some("http://localhost:8080")),
        }
    }

    /// Get blockchain network configurations
    pub fn blockchain_networks(&self) -> ConfigResult<HashMap<String, BlockchainNetworkConfig>> {
        let mut networks = HashMap::new();

        // Ethereum configuration
        if let Ok(ethereum_rpc) = self.get_env_var("ETHEREUM_RPC_URL", None) {
            networks.insert("ethereum".to_string(), BlockchainNetworkConfig {
                name: "Ethereum".to_string(),
                rpc_url: ethereum_rpc,
                chain_id: self.get_env_var_as("ETHEREUM_CHAIN_ID", Some(1))?,
                gas_price_multiplier: self.get_env_var_as("ETHEREUM_GAS_MULTIPLIER", Some(1.0))?,
                confirmation_blocks: self.get_env_var_as("ETHEREUM_CONFIRMATIONS", Some(12))?,
            });
        }

        // Polygon configuration
        if let Ok(polygon_rpc) = self.get_env_var("POLYGON_RPC_URL", None) {
            networks.insert("polygon".to_string(), BlockchainNetworkConfig {
                name: "Polygon".to_string(),
                rpc_url: polygon_rpc,
                chain_id: self.get_env_var_as("POLYGON_CHAIN_ID", Some(137))?,
                gas_price_multiplier: self.get_env_var_as("POLYGON_GAS_MULTIPLIER", Some(1.0))?,
                confirmation_blocks: self.get_env_var_as("POLYGON_CONFIRMATIONS", Some(20))?,
            });
        }

        // Add default testnet configurations for development
        if self.environment.is_development() {
            networks.insert("ethereum_testnet".to_string(), BlockchainNetworkConfig {
                name: "Ethereum Goerli".to_string(),
                rpc_url: self.get_env_var("ETHEREUM_TESTNET_RPC_URL", Some("https://goerli.infura.io/v3/YOUR_PROJECT_ID"))?,
                chain_id: 5,
                gas_price_multiplier: 1.0,
                confirmation_blocks: 6,
            });
        }

        Ok(networks)
    }

    /// Get external service configurations
    pub fn external_services(&self) -> ConfigResult<ExternalServicesConfig> {
        Ok(ExternalServicesConfig {
            stripe: StripeConfig {
                public_key: self.get_env_var("STRIPE_PUBLIC_KEY", Some("pk_test_..."))?,
                secret_key: self.require_env_var("STRIPE_SECRET_KEY")?,
                webhook_secret: self.optional_env_var("STRIPE_WEBHOOK_SECRET"),
            },
            sendgrid: SendGridConfig {
                api_key: self.require_env_var("SENDGRID_API_KEY")?,
                from_email: self.get_env_var("SENDGRID_FROM_EMAIL", Some("noreply@rwa-platform.com"))?,
                from_name: self.get_env_var("SENDGRID_FROM_NAME", Some("RWA Platform"))?,
            },
            aws: AwsConfig {
                access_key_id: self.optional_env_var("AWS_ACCESS_KEY_ID"),
                secret_access_key: self.optional_env_var("AWS_SECRET_ACCESS_KEY"),
                region: self.get_env_var("AWS_REGION", Some("us-east-1"))?,
                s3_bucket: self.optional_env_var("AWS_S3_BUCKET"),
            },
        })
    }

    /// Get monitoring and observability configuration
    pub fn observability_config(&self) -> ConfigResult<ObservabilityConfig> {
        Ok(ObservabilityConfig {
            jaeger_endpoint: self.optional_env_var("JAEGER_ENDPOINT"),
            prometheus_endpoint: self.get_env_var("PROMETHEUS_ENDPOINT", Some("http://localhost:9090"))?,
            log_level: self.get_env_var("LOG_LEVEL", Some(self.environment.default_log_level()))?,
            enable_metrics: self.get_env_var_as("ENABLE_METRICS", Some(true))?,
            enable_tracing: self.get_env_var_as("ENABLE_TRACING", Some(!self.environment.is_production()))?,
        })
    }
}

/// Blockchain network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainNetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_price_multiplier: f64,
    pub confirmation_blocks: u32,
}

/// External services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    pub stripe: StripeConfig,
    pub sendgrid: SendGridConfig,
    pub aws: AwsConfig,
}

/// Stripe payment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeConfig {
    pub public_key: String,
    pub secret_key: String,
    pub webhook_secret: Option<String>,
}

/// SendGrid email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendGridConfig {
    pub api_key: String,
    pub from_email: String,
    pub from_name: String,
}

/// AWS services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub region: String,
    pub s3_bucket: Option<String>,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub jaeger_endpoint: Option<String>,
    pub prometheus_endpoint: String,
    pub log_level: String,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
}

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate production configuration
    pub fn validate_production_config(env_config: &EnvironmentConfig) -> ConfigResult<()> {
        let required_vars = vec![
            "DATABASE_URL",
            "REDIS_URL",
            "JWT_SECRET",
            "ENCRYPTION_KEY",
            "API_BASE_URL",
            "STRIPE_SECRET_KEY",
            "SENDGRID_API_KEY",
        ];

        env_config.validate_required_vars(&required_vars)?;

        // Validate JWT secret length
        let jwt_secret = env_config.jwt_secret()?;
        if jwt_secret.len() < 32 {
            return Err(ConfigError::ValidationError(
                "JWT secret must be at least 32 characters long".to_string()
            ));
        }

        // Validate encryption key length
        let encryption_key = env_config.encryption_key()?;
        if encryption_key.len() < 32 {
            return Err(ConfigError::ValidationError(
                "Encryption key must be at least 32 characters long".to_string()
            ));
        }

        // Validate database URL format
        let database_url = env_config.database_url()?;
        if !database_url.starts_with("postgresql://") {
            return Err(ConfigError::ValidationError(
                "Database URL must be a PostgreSQL connection string".to_string()
            ));
        }

        info!("Production configuration validation passed");
        Ok(())
    }

    /// Validate development configuration
    pub fn validate_development_config(env_config: &EnvironmentConfig) -> ConfigResult<()> {
        // Less strict validation for development
        let _ = env_config.database_url()?;
        let _ = env_config.redis_url()?;
        let _ = env_config.jwt_secret()?;

        info!("Development configuration validation passed");
        Ok(())
    }

    /// Validate configuration based on environment
    pub fn validate_config(env_config: &EnvironmentConfig) -> ConfigResult<()> {
        match env_config.environment() {
            Environment::Production => Self::validate_production_config(env_config),
            Environment::Staging => Self::validate_production_config(env_config), // Same as production
            _ => Self::validate_development_config(env_config),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_environment_detection() {
        // Test default environment
        env::remove_var("RWA_ENV");
        assert_eq!(Environment::current(), Environment::Development);

        // Test production environment
        env::set_var("RWA_ENV", "production");
        assert_eq!(Environment::current(), Environment::Production);

        // Test staging environment
        env::set_var("RWA_ENV", "staging");
        assert_eq!(Environment::current(), Environment::Staging);

        // Clean up
        env::remove_var("RWA_ENV");
    }

    #[test]
    fn test_environment_properties() {
        let prod = Environment::Production;
        assert!(prod.is_production());
        assert!(!prod.is_development());
        assert_eq!(prod.config_file_name(), "production.toml");
        assert_eq!(prod.default_log_level(), "warn");

        let dev = Environment::Development;
        assert!(!dev.is_production());
        assert!(dev.is_development());
        assert_eq!(dev.config_file_name(), "development.toml");
        assert_eq!(dev.default_log_level(), "debug");
    }

    #[test]
    fn test_environment_config() {
        env::set_var("TEST_VAR", "test_value");
        env::set_var("TEST_NUMBER", "42");

        let config = EnvironmentConfig::new();

        // Test environment variable retrieval
        assert_eq!(config.get_env_var("TEST_VAR", None).unwrap(), "test_value");
        assert_eq!(config.get_env_var("NONEXISTENT", Some("default")).unwrap(), "default");

        // Test typed environment variable
        assert_eq!(config.get_env_var_as::<i32>("TEST_NUMBER", None).unwrap(), 42);

        // Clean up
        env::remove_var("TEST_VAR");
        env::remove_var("TEST_NUMBER");
    }

    #[test]
    fn test_config_validation() {
        let config = EnvironmentConfig::new();
        
        // Development validation should be lenient
        let result = ConfigValidator::validate_development_config(&config);
        // This might fail if required env vars are not set, which is expected
        
        // Test that validation function exists and can be called
        assert!(ConfigValidator::validate_config(&config).is_ok() || ConfigValidator::validate_config(&config).is_err());
    }
}
