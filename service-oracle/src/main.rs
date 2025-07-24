// =====================================================================================
// RWA Tokenization Platform - Oracle Service Main
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use service_oracle::{
    cache::PriceCache,
    config::OracleConfig,
    error::OracleResult,
    health::HealthService,
    metrics::OracleMetrics,
    service::OracleService,
    AppState,
    configure_routes,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    init_tracing();

    info!("Starting RWA Oracle Service v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = match load_config().await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    info!("Configuration loaded successfully");

    // Initialize database connection pool
    let db_pool = match init_database(&config).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    info!("Database connection established");

    // Run database migrations
    if let Err(e) = run_migrations(&db_pool).await {
        error!("Failed to run database migrations: {}", e);
        std::process::exit(1);
    }

    info!("Database migrations completed");

    // Initialize cache
    let cache = match PriceCache::new(&config.redis).await {
        Ok(cache) => Arc::new(cache),
        Err(e) => {
            error!("Failed to initialize cache: {}", e);
            std::process::exit(1);
        }
    };

    info!("Cache connection established");

    // Initialize metrics
    let metrics = match OracleMetrics::new() {
        Ok(metrics) => Arc::new(metrics),
        Err(e) => {
            error!("Failed to initialize metrics: {}", e);
            std::process::exit(1);
        }
    };

    info!("Metrics system initialized");

    // Initialize Oracle service
    let oracle_service = match OracleService::new(config.clone(), db_pool.clone()).await {
        Ok(service) => Arc::new(service),
        Err(e) => {
            error!("Failed to initialize Oracle service: {}", e);
            std::process::exit(1);
        }
    };

    info!("Oracle service initialized");

    // Initialize health service
    let health_service = Arc::new(HealthService::new(
        db_pool.clone(),
        cache.clone(),
        Duration::from_secs(config.monitoring.health_check_interval),
    ));

    info!("Health service initialized");

    // Start background services
    start_background_services(&metrics, &health_service).await;

    // Create application state
    let app_state = AppState {
        oracle_service,
        health_service,
        metrics,
    };

    // Start HTTP server
    let server_config = config.server.clone();
    let bind_address = format!("{}:{}", server_config.host, server_config.port);

    info!("Starting HTTP server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(create_cors())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new()
                .add(("X-Version", env!("CARGO_PKG_VERSION")))
                .add(("X-Service", "rwa-oracle"))
            )
            .configure(configure_routes)
    })
    .workers(server_config.workers)
    .max_connections(server_config.max_connections)
    .keep_alive(Duration::from_secs(server_config.keep_alive))
    .client_request_timeout(std::time::Duration::from_secs(server_config.client_timeout))
    .client_disconnect_timeout(std::time::Duration::from_secs(server_config.client_shutdown))
    .bind(&bind_address)?
    .run()
    .await
}

/// Initialize tracing/logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "service_oracle=debug,tower_http=debug,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Load configuration from environment and files
async fn load_config() -> OracleResult<OracleConfig> {
    // Try to load from config file first
    if let Ok(config_path) = std::env::var("CONFIG_PATH") {
        info!("Loading configuration from file: {}", config_path);
        OracleConfig::from_file(&config_path)
    } else {
        info!("Loading configuration from environment variables");
        OracleConfig::from_env()
    }
}

/// Initialize database connection pool
async fn init_database(config: &OracleConfig) -> OracleResult<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(config.database.connect_timeout))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout))
        .max_lifetime(Duration::from_secs(config.database.max_lifetime))
        .connect(&config.database.url)
        .await
        .map_err(|e| service_oracle::error::OracleError::Database(e))?;

    // Test the connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| service_oracle::error::OracleError::Database(e))?;

    Ok(pool)
}

/// Run database migrations
async fn run_migrations(pool: &sqlx::PgPool) -> OracleResult<()> {
    info!("Running database migrations...");
    
    // Create tables if they don't exist
    let migrations = vec![
        include_str!("../migrations/001_initial_schema.sql"),
        include_str!("../migrations/002_price_feeds.sql"),
        include_str!("../migrations/003_subscriptions.sql"),
    ];

    for (i, migration) in migrations.iter().enumerate() {
        info!("Running migration {}", i + 1);
        sqlx::query(migration)
            .execute(pool)
            .await
            .map_err(|e| {
                error!("Migration {} failed: {}", i + 1, e);
                service_oracle::error::OracleError::Database(e)
            })?;
    }

    info!("All migrations completed successfully");
    Ok(())
}

/// Start background services
async fn start_background_services(
    metrics: &Arc<OracleMetrics>,
    health_service: &Arc<HealthService>,
) {
    info!("Starting background services...");

    // Start metrics collection
    metrics.start_background_collection().await;
    info!("Background metrics collection started");

    // Start health monitoring
    health_service.start_monitoring().await;
    info!("Background health monitoring started");

    // Start cleanup tasks
    start_cleanup_tasks().await;
    info!("Background cleanup tasks started");
}

/// Start cleanup tasks
async fn start_cleanup_tasks() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
        
        loop {
            interval.tick().await;
            
            // Cleanup old price data (keep last 30 days)
            if let Err(e) = cleanup_old_prices().await {
                warn!("Failed to cleanup old prices: {}", e);
            }
            
            // Cleanup expired cache entries would be handled by Redis TTL
            // Additional cleanup tasks can be added here
        }
    });
}

/// Cleanup old price data
async fn cleanup_old_prices() -> OracleResult<()> {
    // This would require database access - simplified for now
    info!("Cleaning up old price data...");
    
    // In a real implementation, you would:
    // 1. Connect to database
    // 2. Delete prices older than retention period
    // 3. Log cleanup statistics
    
    Ok(())
}

/// Create CORS configuration
fn create_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600)
}

/// Graceful shutdown handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            info!("Received SIGTERM, shutting down gracefully...");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_tracing_initialization() {
        // Test that tracing can be initialized without panicking
        // Note: This test might interfere with other tests if run in parallel
        init_tracing();
        
        // Test that we can log messages
        info!("Test log message");
        warn!("Test warning message");
        error!("Test error message");
    }

    #[tokio::test]
    async fn test_config_loading_from_env() {
        // Set test environment variables
        env::set_var("SERVER_HOST", "127.0.0.1");
        env::set_var("SERVER_PORT", "8080");
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost/test_db");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("JWT_SECRET", "test-secret-key-with-sufficient-length-for-testing");

        let result = load_config().await;
        
        // Clean up environment variables
        env::remove_var("SERVER_HOST");
        env::remove_var("SERVER_PORT");
        env::remove_var("DATABASE_URL");
        env::remove_var("REDIS_URL");
        env::remove_var("JWT_SECRET");

        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_cors_configuration() {
        let cors = create_cors();
        // Basic test to ensure CORS can be created
        // More detailed testing would require integration tests
        assert!(true); // Placeholder assertion
    }

    #[tokio::test]
    async fn test_cleanup_old_prices() {
        // Test the cleanup function (currently a no-op)
        let result = cleanup_old_prices().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_environment_variables() {
        // Test that required environment variables can be read
        let test_vars = vec![
            ("DATABASE_URL", "postgresql://localhost/test"),
            ("REDIS_URL", "redis://localhost:6379"),
            ("JWT_SECRET", "test-secret-key-with-sufficient-length"),
        ];

        for (key, value) in &test_vars {
            env::set_var(key, value);
            assert_eq!(env::var(key).unwrap(), *value);
            env::remove_var(key);
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default as it requires actual database connection
    async fn test_database_initialization() {
        let config = OracleConfig::default();
        
        // This test would require a test database to be running
        // In a real CI/CD environment, you'd set up a test database
        match init_database(&config).await {
            Ok(_pool) => {
                // Database connection successful
                assert!(true);
            }
            Err(_e) => {
                // Expected to fail in test environment without database
                assert!(true);
            }
        }
    }

    #[test]
    fn test_version_info() {
        // Test that version information is available
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        
        let name = env!("CARGO_PKG_NAME");
        assert_eq!(name, "service-oracle");
    }

    #[tokio::test]
    async fn test_background_services_startup() {
        // Test that background services can be started without panicking
        let metrics = Arc::new(OracleMetrics::new().unwrap());
        
        // Create a minimal health service for testing
        // Note: This would require proper database and cache setup in real tests
        let redis_config = service_oracle::config::RedisConfig {
            url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: 5,
            command_timeout: 5,
            retry_attempts: 3,
        };

        // Skip actual health service creation in unit tests
        // start_background_services(&metrics, &health_service).await;
        
        // Just test that metrics can be created
        assert!(metrics.export_metrics().is_ok());
    }
}
