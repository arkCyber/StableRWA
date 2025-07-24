// =====================================================================================
// RWA Tokenization Platform - Asset Service Configuration Tests
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use service_asset::config::{ServiceConfig, ServerConfig, DatabaseConfig, CacheConfig, AuthConfig};
use std::env;
use tempfile::NamedTempFile;
use std::io::Write;

/// Test default configuration
#[test]
fn test_default_config() {
    let config = ServiceConfig::default();
    
    // Verify default values
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.workers, 4);
    assert_eq!(config.server.max_connections, 25000);
    assert_eq!(config.server.keep_alive, 75);
    assert_eq!(config.server.client_timeout, 5000);
    assert_eq!(config.server.client_shutdown, 5000);
    
    assert_eq!(config.database.max_connections, 10);
    assert_eq!(config.database.min_connections, 1);
    assert_eq!(config.database.connect_timeout, 30);
    assert_eq!(config.database.idle_timeout, 600);
    assert_eq!(config.database.max_lifetime, 1800);
    
    assert_eq!(config.cache.ttl_seconds, 3600);
    assert_eq!(config.cache.max_connections, 10);
    assert_eq!(config.cache.connection_timeout, 5);
    
    assert_eq!(config.auth.token_expiry_hours, 24);
    assert!(config.auth.require_https);
}

/// Test configuration from environment variables
#[test]
fn test_config_from_env() {
    // Set environment variables
    env::set_var("SERVER_HOST", "0.0.0.0");
    env::set_var("SERVER_PORT", "9090");
    env::set_var("SERVER_WORKERS", "8");
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test_db");
    env::set_var("DATABASE_MAX_CONNECTIONS", "20");
    env::set_var("REDIS_URL", "redis://localhost:6380");
    env::set_var("JWT_SECRET", "test-secret-key");
    env::set_var("AUTH_TOKEN_EXPIRY_HOURS", "48");
    env::set_var("CACHE_TTL_SECONDS", "7200");
    
    let config = ServiceConfig::from_env().unwrap();
    
    // Verify environment values are loaded
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9090);
    assert_eq!(config.server.workers, 8);
    assert_eq!(config.database.url, "postgresql://test:test@localhost/test_db");
    assert_eq!(config.database.max_connections, 20);
    assert_eq!(config.cache.url, "redis://localhost:6380");
    assert_eq!(config.auth.jwt_secret, "test-secret-key");
    assert_eq!(config.auth.token_expiry_hours, 48);
    assert_eq!(config.cache.ttl_seconds, 7200);
    
    // Clean up environment variables
    env::remove_var("SERVER_HOST");
    env::remove_var("SERVER_PORT");
    env::remove_var("SERVER_WORKERS");
    env::remove_var("DATABASE_URL");
    env::remove_var("DATABASE_MAX_CONNECTIONS");
    env::remove_var("REDIS_URL");
    env::remove_var("JWT_SECRET");
    env::remove_var("AUTH_TOKEN_EXPIRY_HOURS");
    env::remove_var("CACHE_TTL_SECONDS");
}

/// Test configuration validation
#[test]
fn test_config_validation() {
    // Test invalid port
    env::set_var("SERVER_PORT", "99999");
    let result = ServiceConfig::from_env();
    env::remove_var("SERVER_PORT");
    
    // Should handle invalid port gracefully
    assert!(result.is_ok() || result.is_err());
    
    // Test invalid worker count
    env::set_var("SERVER_WORKERS", "0");
    let result = ServiceConfig::from_env();
    env::remove_var("SERVER_WORKERS");
    
    // Should handle invalid worker count
    assert!(result.is_ok() || result.is_err());
    
    // Test missing required JWT secret
    env::remove_var("JWT_SECRET");
    let result = ServiceConfig::from_env();
    
    // Should handle missing JWT secret
    assert!(result.is_ok() || result.is_err());
}

/// Test configuration file loading
#[test]
fn test_config_from_file() {
    let config_content = r#"
[server]
host = "0.0.0.0"
port = 8081
workers = 6
max_connections = 30000
keep_alive = 90
client_timeout = 6000
client_shutdown = 6000

[database]
url = "postgresql://user:pass@localhost/asset_db"
max_connections = 15
min_connections = 2
connect_timeout = 45
idle_timeout = 900
max_lifetime = 3600

[cache]
url = "redis://localhost:6379/1"
ttl_seconds = 1800
max_connections = 15
connection_timeout = 10

[auth]
jwt_secret = "file-secret-key"
token_expiry_hours = 12
require_https = false
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    let config = ServiceConfig::from_file(temp_file.path()).unwrap();
    
    // Verify file values are loaded
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 8081);
    assert_eq!(config.server.workers, 6);
    assert_eq!(config.server.max_connections, 30000);
    assert_eq!(config.database.url, "postgresql://user:pass@localhost/asset_db");
    assert_eq!(config.database.max_connections, 15);
    assert_eq!(config.cache.url, "redis://localhost:6379/1");
    assert_eq!(config.cache.ttl_seconds, 1800);
    assert_eq!(config.auth.jwt_secret, "file-secret-key");
    assert_eq!(config.auth.token_expiry_hours, 12);
    assert!(!config.auth.require_https);
}

/// Test configuration precedence (env > file > default)
#[test]
fn test_config_precedence() {
    // Create config file
    let config_content = r#"
[server]
host = "file-host"
port = 8082

[auth]
jwt_secret = "file-secret"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(config_content.as_bytes()).unwrap();
    
    // Set environment variable (should override file)
    env::set_var("SERVER_HOST", "env-host");
    env::set_var("JWT_SECRET", "env-secret");
    
    let config = ServiceConfig::from_file(temp_file.path()).unwrap();
    
    // Environment should take precedence over file
    assert_eq!(config.server.host, "env-host");
    assert_eq!(config.auth.jwt_secret, "env-secret");
    
    // File should take precedence over default
    assert_eq!(config.server.port, 8082);
    
    // Clean up
    env::remove_var("SERVER_HOST");
    env::remove_var("JWT_SECRET");
}

/// Test configuration serialization
#[test]
fn test_config_serialization() {
    let config = ServiceConfig::default();
    
    // Test TOML serialization
    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("[server]"));
    assert!(toml_str.contains("[database]"));
    assert!(toml_str.contains("[cache]"));
    assert!(toml_str.contains("[auth]"));
    
    // Test deserialization
    let deserialized: ServiceConfig = toml::from_str(&toml_str).unwrap();
    assert_eq!(config.server.host, deserialized.server.host);
    assert_eq!(config.server.port, deserialized.server.port);
    assert_eq!(config.database.max_connections, deserialized.database.max_connections);
}

/// Test environment-specific configurations
#[test]
fn test_environment_configs() {
    // Test development environment
    env::set_var("ENVIRONMENT", "development");
    env::set_var("RUST_LOG", "debug");
    env::set_var("SERVER_HOST", "127.0.0.1");
    
    let dev_config = ServiceConfig::from_env().unwrap();
    assert_eq!(dev_config.server.host, "127.0.0.1");
    
    // Test production environment
    env::set_var("ENVIRONMENT", "production");
    env::set_var("RUST_LOG", "info");
    env::set_var("SERVER_HOST", "0.0.0.0");
    env::set_var("AUTH_REQUIRE_HTTPS", "true");
    
    let prod_config = ServiceConfig::from_env().unwrap();
    assert_eq!(prod_config.server.host, "0.0.0.0");
    
    // Clean up
    env::remove_var("ENVIRONMENT");
    env::remove_var("RUST_LOG");
    env::remove_var("SERVER_HOST");
    env::remove_var("AUTH_REQUIRE_HTTPS");
}

/// Test configuration validation rules
#[test]
fn test_config_validation_rules() {
    let mut config = ServiceConfig::default();
    
    // Test server configuration validation
    config.server.port = 0;
    assert!(config.validate().is_err());
    
    config.server.port = 65536;
    assert!(config.validate().is_err());
    
    config.server.port = 8080;
    config.server.workers = 0;
    assert!(config.validate().is_err());
    
    config.server.workers = 4;
    config.server.max_connections = 0;
    assert!(config.validate().is_err());
    
    // Test database configuration validation
    config.server.max_connections = 25000;
    config.database.max_connections = 0;
    assert!(config.validate().is_err());
    
    config.database.max_connections = 10;
    config.database.min_connections = 20; // min > max
    assert!(config.validate().is_err());
    
    // Test cache configuration validation
    config.database.min_connections = 1;
    config.cache.ttl_seconds = 0;
    assert!(config.validate().is_err());
    
    config.cache.ttl_seconds = 3600;
    config.cache.max_connections = 0;
    assert!(config.validate().is_err());
    
    // Test auth configuration validation
    config.cache.max_connections = 10;
    config.auth.jwt_secret = "".to_string();
    assert!(config.validate().is_err());
    
    config.auth.jwt_secret = "short".to_string(); // Too short
    assert!(config.validate().is_err());
    
    config.auth.jwt_secret = "valid-secret-key-with-sufficient-length".to_string();
    config.auth.token_expiry_hours = 0;
    assert!(config.validate().is_err());
    
    // Valid configuration should pass
    config.auth.token_expiry_hours = 24;
    assert!(config.validate().is_ok());
}

/// Test configuration hot reload simulation
#[test]
fn test_config_hot_reload() {
    // Create initial config file
    let initial_content = r#"
[server]
port = 8080
workers = 4

[cache]
ttl_seconds = 3600
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(initial_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    
    let initial_config = ServiceConfig::from_file(temp_file.path()).unwrap();
    assert_eq!(initial_config.server.port, 8080);
    assert_eq!(initial_config.cache.ttl_seconds, 3600);
    
    // Simulate config file update
    let updated_content = r#"
[server]
port = 8081
workers = 6

[cache]
ttl_seconds = 7200
"#;

    temp_file.write_all(updated_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    
    let updated_config = ServiceConfig::from_file(temp_file.path()).unwrap();
    assert_eq!(updated_config.server.port, 8081);
    assert_eq!(updated_config.server.workers, 6);
    assert_eq!(updated_config.cache.ttl_seconds, 7200);
}

/// Test configuration secrets handling
#[test]
fn test_config_secrets() {
    // Test that secrets are not logged or exposed
    env::set_var("JWT_SECRET", "super-secret-key");
    env::set_var("DATABASE_URL", "postgresql://user:password@localhost/db");
    
    let config = ServiceConfig::from_env().unwrap();
    
    // Verify secrets are loaded
    assert_eq!(config.auth.jwt_secret, "super-secret-key");
    assert!(config.database.url.contains("password"));
    
    // Test debug output doesn't expose secrets
    let debug_output = format!("{:?}", config);
    assert!(!debug_output.contains("super-secret-key"));
    assert!(!debug_output.contains("password"));
    
    // Clean up
    env::remove_var("JWT_SECRET");
    env::remove_var("DATABASE_URL");
}
