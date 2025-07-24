// =====================================================================================
// RWA Tokenization Platform - Oracle Service Configuration
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{OracleError, OracleResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Oracle service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub providers: ProvidersConfig,
    pub aggregation: AggregationConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub client_shutdown: u64,
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
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub command_timeout: u64,
    pub retry_attempts: u32,
}

/// Price providers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub coingecko: ProviderConfig,
    pub coinmarketcap: ProviderConfig,
    pub binance: ProviderConfig,
    pub chainlink: ProviderConfig,
    pub custom_providers: HashMap<String, ProviderConfig>,
}

/// Individual provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub rate_limit_per_minute: u32,
    pub weight: Decimal,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

/// Price aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationConfig {
    pub default_method: String,
    pub min_sources: usize,
    pub max_deviation_percent: Decimal,
    pub confidence_threshold: Decimal,
    pub outlier_detection: bool,
    pub cache_ttl_seconds: u64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub health_check_interval: u64,
    pub alert_thresholds: AlertThresholds,
}

/// Alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub price_deviation_percent: Decimal,
    pub provider_failure_count: u32,
    pub response_time_ms: u64,
    pub error_rate_percent: Decimal,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub api_key_required: bool,
    pub rate_limiting: RateLimitConfig,
    pub cors_origins: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub cleanup_interval: u64,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            providers: ProvidersConfig::default(),
            aggregation: AggregationConfig::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8081,
            workers: 4,
            max_connections: 25000,
            keep_alive: 75,
            client_timeout: 5000,
            client_shutdown: 5000,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost:5432/oracle_service".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            command_timeout: 5,
            retry_attempts: 3,
        }
    }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            coingecko: ProviderConfig {
                enabled: true,
                api_key: None,
                base_url: "https://api.coingecko.com/api/v3".to_string(),
                timeout_seconds: 10,
                rate_limit_per_minute: 50,
                weight: Decimal::new(100, 2), // 1.00
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            coinmarketcap: ProviderConfig {
                enabled: false,
                api_key: None,
                base_url: "https://pro-api.coinmarketcap.com/v1".to_string(),
                timeout_seconds: 10,
                rate_limit_per_minute: 333,
                weight: Decimal::new(120, 2), // 1.20
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            binance: ProviderConfig {
                enabled: true,
                api_key: None,
                base_url: "https://api.binance.com/api/v3".to_string(),
                timeout_seconds: 5,
                rate_limit_per_minute: 1200,
                weight: Decimal::new(110, 2), // 1.10
                retry_attempts: 2,
                retry_delay_ms: 500,
            },
            chainlink: ProviderConfig {
                enabled: false,
                api_key: None,
                base_url: "https://api.chain.link".to_string(),
                timeout_seconds: 15,
                rate_limit_per_minute: 100,
                weight: Decimal::new(150, 2), // 1.50
                retry_attempts: 3,
                retry_delay_ms: 2000,
            },
            custom_providers: HashMap::new(),
        }
    }
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            default_method: "weighted_average".to_string(),
            min_sources: 2,
            max_deviation_percent: Decimal::new(10, 0), // 10%
            confidence_threshold: Decimal::new(80, 2), // 0.80
            outlier_detection: true,
            cache_ttl_seconds: 60,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            health_check_interval: 30,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            price_deviation_percent: Decimal::new(5, 0), // 5%
            provider_failure_count: 3,
            response_time_ms: 5000,
            error_rate_percent: Decimal::new(10, 0), // 10%
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "change-me-in-production".to_string(),
            api_key_required: false,
            rate_limiting: RateLimitConfig::default(),
            cors_origins: vec!["*".to_string()],
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 1000,
            burst_size: 100,
            cleanup_interval: 60,
        }
    }
}

impl OracleConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> OracleResult<Self> {
        let mut config = Self::default();

        // Server configuration
        if let Ok(host) = env::var("SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            config.server.port = port.parse().map_err(|_| {
                OracleError::Configuration { message: "Invalid SERVER_PORT".to_string() }
            })?;
        }
        if let Ok(workers) = env::var("SERVER_WORKERS") {
            config.server.workers = workers.parse().map_err(|_| {
                OracleError::Configuration { message: "Invalid SERVER_WORKERS".to_string() }
            })?;
        }

        // Database configuration
        if let Ok(url) = env::var("DATABASE_URL") {
            config.database.url = url;
        }
        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            config.database.max_connections = max_conn.parse().map_err(|_| {
                OracleError::Configuration { message: "Invalid DATABASE_MAX_CONNECTIONS".to_string() }
            })?;
        }

        // Redis configuration
        if let Ok(url) = env::var("REDIS_URL") {
            config.redis.url = url;
        }

        // Security configuration
        if let Ok(secret) = env::var("JWT_SECRET") {
            config.security.jwt_secret = secret;
        }

        // Provider API keys
        if let Ok(key) = env::var("COINGECKO_API_KEY") {
            config.providers.coingecko.api_key = Some(key);
        }
        if let Ok(key) = env::var("COINMARKETCAP_API_KEY") {
            config.providers.coinmarketcap.api_key = Some(key);
            config.providers.coinmarketcap.enabled = true;
        }

        config.validate()?;
        Ok(config)
    }

    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> OracleResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            OracleError::Configuration { message: format!("Failed to read config file: {}", e) }
        })?;

        let mut config: Self = toml::from_str(&content).map_err(|e| {
            OracleError::Configuration { message: format!("Failed to parse config file: {}", e) }
        })?;

        // Override with environment variables
        let env_config = Self::from_env()?;
        config.merge_env(env_config);

        config.validate()?;
        Ok(config)
    }

    /// Merge environment configuration
    fn merge_env(&mut self, env_config: Self) {
        if env_config.server.host != ServerConfig::default().host {
            self.server.host = env_config.server.host;
        }
        if env_config.server.port != ServerConfig::default().port {
            self.server.port = env_config.server.port;
        }
        if env_config.database.url != DatabaseConfig::default().url {
            self.database.url = env_config.database.url;
        }
        if env_config.redis.url != RedisConfig::default().url {
            self.redis.url = env_config.redis.url;
        }
        if env_config.security.jwt_secret != SecurityConfig::default().jwt_secret {
            self.security.jwt_secret = env_config.security.jwt_secret;
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> OracleResult<()> {
        if self.server.port == 0 {
            return Err(OracleError::Configuration {
                message: "Server port must be between 1 and 65535".to_string()
            });
        }

        if self.server.workers == 0 {
            return Err(OracleError::Configuration {
                message: "Server workers must be greater than 0".to_string()
            });
        }

        if self.database.max_connections == 0 {
            return Err(OracleError::Configuration {
                message: "Database max_connections must be greater than 0".to_string()
            });
        }

        if self.database.min_connections > self.database.max_connections {
            return Err(OracleError::Configuration {
                message: "Database min_connections cannot be greater than max_connections".to_string()
            });
        }

        if self.aggregation.min_sources == 0 {
            return Err(OracleError::Configuration {
                message: "Aggregation min_sources must be greater than 0".to_string()
            });
        }

        if self.security.jwt_secret.len() < 32 {
            return Err(OracleError::Configuration {
                message: "JWT secret must be at least 32 characters long".to_string()
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = OracleConfig::default();

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8081);
        assert_eq!(config.server.workers, 4);
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.redis.max_connections, 10);
        assert!(config.providers.coingecko.enabled);
        assert!(!config.providers.coinmarketcap.enabled);
    }

    #[test]
    fn test_config_validation_valid() {
        let config = OracleConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_port() {
        let mut config = OracleConfig::default();
        config.server.port = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("port must be between"));
    }

    #[test]
    fn test_config_validation_invalid_workers() {
        let mut config = OracleConfig::default();
        config.server.workers = 0;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("workers must be greater than 0"));
    }

    #[test]
    fn test_config_validation_invalid_db_connections() {
        let mut config = OracleConfig::default();
        config.database.min_connections = 10;
        config.database.max_connections = 5;

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("min_connections cannot be greater"));
    }

    #[test]
    fn test_config_validation_short_jwt_secret() {
        let mut config = OracleConfig::default();
        config.security.jwt_secret = "short".to_string();

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("JWT secret must be at least 32"));
    }

    #[test]
    fn test_config_from_env() {
        env::set_var("SERVER_HOST", "0.0.0.0");
        env::set_var("SERVER_PORT", "9090");
        env::set_var("SERVER_WORKERS", "8");
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test_db");
        env::set_var("REDIS_URL", "redis://localhost:6380");
        env::set_var("JWT_SECRET", "test-secret-key-with-sufficient-length");

        let config = OracleConfig::from_env().unwrap();

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.workers, 8);
        assert_eq!(config.database.url, "postgresql://test:test@localhost/test_db");
        assert_eq!(config.redis.url, "redis://localhost:6380");
        assert_eq!(config.security.jwt_secret, "test-secret-key-with-sufficient-length");

        // Clean up
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("SERVER_WORKERS");
        env::remove_var("DATABASE_URL");
        env::remove_var("REDIS_URL");
        env::remove_var("JWT_SECRET");
    }

    #[test]
    fn test_config_from_file() {
        let config_content = r#"
[server]
host = "0.0.0.0"
port = 8082
workers = 6

[database]
url = "postgresql://user:pass@localhost/oracle_db"
max_connections = 15

[redis]
url = "redis://localhost:6379/1"

[security]
jwt_secret = "file-secret-key-with-sufficient-length"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = OracleConfig::from_file(temp_file.path()).unwrap();

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8082);
        assert_eq!(config.server.workers, 6);
        assert_eq!(config.database.url, "postgresql://user:pass@localhost/oracle_db");
        assert_eq!(config.database.max_connections, 15);
        assert_eq!(config.redis.url, "redis://localhost:6379/1");
        assert_eq!(config.security.jwt_secret, "file-secret-key-with-sufficient-length");
    }

    #[test]
    fn test_provider_config_defaults() {
        let providers = ProvidersConfig::default();

        assert!(providers.coingecko.enabled);
        assert!(!providers.coinmarketcap.enabled);
        assert!(providers.binance.enabled);
        assert!(!providers.chainlink.enabled);

        assert_eq!(providers.coingecko.rate_limit_per_minute, 50);
        assert_eq!(providers.binance.rate_limit_per_minute, 1200);
        assert_eq!(providers.coingecko.weight, Decimal::new(100, 2));
        assert_eq!(providers.binance.weight, Decimal::new(110, 2));
    }

    #[test]
    fn test_aggregation_config_defaults() {
        let aggregation = AggregationConfig::default();

        assert_eq!(aggregation.default_method, "weighted_average");
        assert_eq!(aggregation.min_sources, 2);
        assert_eq!(aggregation.max_deviation_percent, Decimal::new(10, 0));
        assert_eq!(aggregation.confidence_threshold, Decimal::new(80, 2));
        assert!(aggregation.outlier_detection);
        assert_eq!(aggregation.cache_ttl_seconds, 60);
    }

    #[test]
    fn test_config_serialization() {
        let config = OracleConfig::default();

        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[server]"));
        assert!(toml_str.contains("[database]"));
        assert!(toml_str.contains("[providers]"));

        let deserialized: OracleConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.server.host, deserialized.server.host);
        assert_eq!(config.server.port, deserialized.server.port);
    }

    #[test]
    fn test_invalid_env_values() {
        env::set_var("SERVER_PORT", "invalid");

        let result = OracleConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid SERVER_PORT"));

        env::remove_var("SERVER_PORT");
    }

    #[test]
    fn test_api_key_configuration() {
        env::set_var("COINGECKO_API_KEY", "test-coingecko-key");
        env::set_var("COINMARKETCAP_API_KEY", "test-cmc-key");
        env::set_var("JWT_SECRET", "test-secret-key-with-sufficient-length");

        let config = OracleConfig::from_env().unwrap();

        assert_eq!(config.providers.coingecko.api_key, Some("test-coingecko-key".to_string()));
        assert_eq!(config.providers.coinmarketcap.api_key, Some("test-cmc-key".to_string()));
        assert!(config.providers.coinmarketcap.enabled);

        env::remove_var("COINGECKO_API_KEY");
        env::remove_var("COINMARKETCAP_API_KEY");
        env::remove_var("JWT_SECRET");
    }
}
