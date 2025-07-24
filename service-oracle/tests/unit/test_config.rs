// =====================================================================================
// RWA Tokenization Platform - Oracle Service Config Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use service_oracle::config::*;
use std::env;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_oracle_config_default() {
    let config = OracleConfig::default();
    
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8081);
    assert_eq!(config.server.workers, 4);
    assert_eq!(config.database.max_connections, 10);
    assert_eq!(config.redis.max_connections, 10);
    assert_eq!(config.aggregation.default_method, AggregationMethod::WeightedAverage);
    assert_eq!(config.aggregation.min_sources, 2);
    assert!(config.aggregation.deviation_threshold > rust_decimal::Decimal::ZERO);
}

#[test]
fn test_server_config_validation() {
    let mut config = ServerConfig::default();
    
    // Test valid configuration
    assert!(config.validate().is_ok());
    
    // Test invalid port
    config.port = 0;
    assert!(config.validate().is_err());
    
    config.port = 65536;
    assert!(config.validate().is_err());
    
    // Test invalid workers
    config.port = 8080;
    config.workers = 0;
    assert!(config.validate().is_err());
    
    config.workers = 1000;
    assert!(config.validate().is_err());
}

#[test]
fn test_database_config_validation() {
    let mut config = DatabaseConfig::default();
    
    // Test valid configuration
    assert!(config.validate().is_ok());
    
    // Test invalid URL
    config.url = "invalid-url".to_string();
    assert!(config.validate().is_err());
    
    // Test invalid connection counts
    config.url = "postgresql://user:pass@localhost/db".to_string();
    config.max_connections = 0;
    assert!(config.validate().is_err());
    
    config.max_connections = 10;
    config.min_connections = 20; // min > max
    assert!(config.validate().is_err());
}

#[test]
fn test_redis_config_validation() {
    let mut config = RedisConfig::default();
    
    // Test valid configuration
    assert!(config.validate().is_ok());
    
    // Test invalid URL
    config.url = "invalid-url".to_string();
    assert!(config.validate().is_err());
    
    // Test invalid timeouts
    config.url = "redis://localhost:6379".to_string();
    config.connection_timeout = 0;
    assert!(config.validate().is_err());
    
    config.connection_timeout = 5;
    config.command_timeout = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_aggregation_config_validation() {
    let mut config = AggregationConfig::default();
    
    // Test valid configuration
    assert!(config.validate().is_ok());
    
    // Test invalid min_sources
    config.min_sources = 0;
    assert!(config.validate().is_err());
    
    config.min_sources = 100;
    assert!(config.validate().is_err());
    
    // Test invalid deviation_threshold
    config.min_sources = 2;
    config.deviation_threshold = rust_decimal::Decimal::ZERO;
    assert!(config.validate().is_err());
    
    config.deviation_threshold = rust_decimal_macros::dec!(200.0);
    assert!(config.validate().is_err());
    
    // Test invalid confidence_threshold
    config.deviation_threshold = rust_decimal_macros::dec!(10.0);
    config.confidence_threshold = rust_decimal::Decimal::ZERO;
    assert!(config.validate().is_err());
    
    config.confidence_threshold = rust_decimal_macros::dec!(2.0);
    assert!(config.validate().is_err());
}

#[test]
fn test_provider_config_validation() {
    let mut config = ProviderConfig::default();
    
    // Test valid configuration
    assert!(config.validate().is_ok());
    
    // Test invalid weight
    config.weight = rust_decimal::Decimal::ZERO;
    assert!(config.validate().is_err());
    
    config.weight = rust_decimal_macros::dec!(-1.0);
    assert!(config.validate().is_err());
    
    // Test invalid rate_limit
    config.weight = rust_decimal_macros::dec!(1.0);
    config.rate_limit_per_minute = 0;
    assert!(config.validate().is_err());
    
    // Test invalid timeout
    config.rate_limit_per_minute = 60;
    config.timeout_seconds = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_from_env() {
    // Set environment variables
    env::set_var("SERVER_HOST", "0.0.0.0");
    env::set_var("SERVER_PORT", "9090");
    env::set_var("SERVER_WORKERS", "8");
    env::set_var("DATABASE_MAX_CONNECTIONS", "20");
    env::set_var("REDIS_MAX_CONNECTIONS", "15");
    
    let config = OracleConfig::from_env().expect("Failed to load config from env");
    
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9090);
    assert_eq!(config.server.workers, 8);
    assert_eq!(config.database.max_connections, 20);
    assert_eq!(config.redis.max_connections, 15);
    
    // Clean up
    env::remove_var("SERVER_HOST");
    env::remove_var("SERVER_PORT");
    env::remove_var("SERVER_WORKERS");
    env::remove_var("DATABASE_MAX_CONNECTIONS");
    env::remove_var("REDIS_MAX_CONNECTIONS");
}

#[test]
fn test_config_from_file() {
    let config_content = r#"
[server]
host = "0.0.0.0"
port = 9090
workers = 8

[database]
url = "postgresql://test:test@localhost/test"
max_connections = 20
min_connections = 2

[redis]
url = "redis://localhost:6379"
max_connections = 15

[aggregation]
default_method = "median"
min_sources = 3
deviation_threshold = 15.0
confidence_threshold = 0.85
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(config_content.as_bytes()).expect("Failed to write config");
    
    let config = OracleConfig::from_file(temp_file.path())
        .expect("Failed to load config from file");
    
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9090);
    assert_eq!(config.server.workers, 8);
    assert_eq!(config.database.max_connections, 20);
    assert_eq!(config.database.min_connections, 2);
    assert_eq!(config.redis.max_connections, 15);
    assert_eq!(config.aggregation.default_method, AggregationMethod::Median);
    assert_eq!(config.aggregation.min_sources, 3);
}

#[test]
fn test_config_merge() {
    let mut base_config = OracleConfig::default();
    base_config.server.port = 8080;
    base_config.database.max_connections = 5;
    
    let mut override_config = OracleConfig::default();
    override_config.server.port = 9090;
    override_config.redis.max_connections = 20;
    
    let merged = base_config.merge(override_config);
    
    assert_eq!(merged.server.port, 9090); // Overridden
    assert_eq!(merged.database.max_connections, 5); // From base
    assert_eq!(merged.redis.max_connections, 20); // Overridden
}

#[test]
fn test_aggregation_method_serialization() {
    use serde_json;
    
    let methods = vec![
        AggregationMethod::Mean,
        AggregationMethod::Median,
        AggregationMethod::WeightedAverage,
        AggregationMethod::VolumeWeighted,
    ];
    
    for method in methods {
        let json = serde_json::to_string(&method).expect("Failed to serialize");
        let deserialized: AggregationMethod = serde_json::from_str(&json)
            .expect("Failed to deserialize");
        assert_eq!(method, deserialized);
    }
}

#[test]
fn test_config_validation_comprehensive() {
    let config = OracleConfig::default();
    
    // Should pass validation
    assert!(config.validate().is_ok());
    
    // Test with invalid nested configs
    let mut invalid_config = config.clone();
    invalid_config.server.port = 0;
    assert!(invalid_config.validate().is_err());
    
    let mut invalid_config = config.clone();
    invalid_config.database.max_connections = 0;
    assert!(invalid_config.validate().is_err());
    
    let mut invalid_config = config.clone();
    invalid_config.redis.connection_timeout = 0;
    assert!(invalid_config.validate().is_err());
    
    let mut invalid_config = config.clone();
    invalid_config.aggregation.min_sources = 0;
    assert!(invalid_config.validate().is_err());
}

#[test]
fn test_config_display() {
    let config = OracleConfig::default();
    let display_str = format!("{}", config);
    
    assert!(display_str.contains("OracleConfig"));
    assert!(display_str.contains("server"));
    assert!(display_str.contains("database"));
    assert!(display_str.contains("redis"));
    assert!(display_str.contains("aggregation"));
}

#[test]
fn test_config_debug() {
    let config = OracleConfig::default();
    let debug_str = format!("{:?}", config);
    
    assert!(debug_str.contains("OracleConfig"));
    assert!(debug_str.contains("ServerConfig"));
    assert!(debug_str.contains("DatabaseConfig"));
    assert!(debug_str.contains("RedisConfig"));
    assert!(debug_str.contains("AggregationConfig"));
}

#[test]
fn test_config_clone() {
    let config = OracleConfig::default();
    let cloned = config.clone();
    
    assert_eq!(config.server.host, cloned.server.host);
    assert_eq!(config.server.port, cloned.server.port);
    assert_eq!(config.database.url, cloned.database.url);
    assert_eq!(config.redis.url, cloned.redis.url);
}

#[test]
fn test_config_partial_eq() {
    let config1 = OracleConfig::default();
    let config2 = OracleConfig::default();
    let mut config3 = OracleConfig::default();
    config3.server.port = 9090;
    
    assert_eq!(config1, config2);
    assert_ne!(config1, config3);
}

#[test]
fn test_provider_config_creation() {
    let config = ProviderConfig::new(
        true,
        Some("test-api-key".to_string()),
        rust_decimal_macros::dec!(1.5),
        120,
        30,
    );
    
    assert!(config.enabled);
    assert_eq!(config.api_key, Some("test-api-key".to_string()));
    assert_eq!(config.weight, rust_decimal_macros::dec!(1.5));
    assert_eq!(config.rate_limit_per_minute, 120);
    assert_eq!(config.timeout_seconds, 30);
}

#[test]
fn test_config_error_handling() {
    // Test invalid file path
    let result = OracleConfig::from_file("/nonexistent/path/config.toml");
    assert!(result.is_err());
    
    // Test invalid TOML content
    let invalid_toml = r#"
[server
host = "invalid
"#;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(invalid_toml.as_bytes()).expect("Failed to write config");
    
    let result = OracleConfig::from_file(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_config_environment_precedence() {
    // Set conflicting environment variables
    env::set_var("SERVER_PORT", "7777");
    
    let config_content = r#"
[server]
port = 8888
"#;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(config_content.as_bytes()).expect("Failed to write config");
    
    // Environment should take precedence
    let config = OracleConfig::from_file_with_env(temp_file.path())
        .expect("Failed to load config");
    
    assert_eq!(config.server.port, 7777);
    
    // Clean up
    env::remove_var("SERVER_PORT");
}
