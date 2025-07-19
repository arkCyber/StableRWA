// =====================================================================================
// File: core-observability/src/lib.rs
// Description: Observability utilities for RWA platform - logging, metrics, tracing
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod logging;
pub mod metrics;
pub mod tracing_setup;
pub mod health;

pub use logging::*;
pub use metrics::*;
pub use tracing_setup::*;
pub use health::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Observability errors
#[derive(Error, Debug)]
pub enum ObservabilityError {
    #[error("Metrics error: {0}")]
    Metrics(String),
    #[error("Tracing error: {0}")]
    Tracing(String),
    #[error("Health check error: {0}")]
    HealthCheck(String),
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
    pub error: Option<ErrorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub error_type: String,
    pub error_message: String,
    pub stack_trace: Option<String>,
}

/// Business metrics for RWA platform
#[derive(Debug, Clone)]
pub struct BusinessMetrics {
    // Asset metrics
    pub assets_created: prometheus::Counter,
    pub assets_updated: prometheus::Counter,
    pub assets_deleted: prometheus::Counter,
    pub asset_value_total: prometheus::Gauge,
    
    // User metrics
    pub users_registered: prometheus::Counter,
    pub users_active: prometheus::Gauge,
    pub user_sessions: prometheus::Gauge,
    
    // Payment metrics
    pub payments_initiated: prometheus::Counter,
    pub payments_completed: prometheus::Counter,
    pub payments_failed: prometheus::Counter,
    pub payment_volume: prometheus::Counter,
    
    // Blockchain metrics
    pub blockchain_transactions: prometheus::CounterVec,
    pub blockchain_balance_checks: prometheus::CounterVec,
    pub blockchain_connection_status: prometheus::GaugeVec,
    
    // System metrics
    pub http_requests_total: prometheus::CounterVec,
    pub http_request_duration: prometheus::HistogramVec,
    pub database_connections: prometheus::Gauge,
    pub cache_hits: prometheus::Counter,
    pub cache_misses: prometheus::Counter,
}

impl BusinessMetrics {
    pub fn new() -> Result<Self, ObservabilityError> {
        let registry = prometheus::default_registry();
        
        // Asset metrics
        let assets_created = prometheus::Counter::new("rwa_assets_created_total", "Total number of assets created")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(assets_created.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let assets_updated = prometheus::Counter::new("rwa_assets_updated_total", "Total number of assets updated")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(assets_updated.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let assets_deleted = prometheus::Counter::new("rwa_assets_deleted_total", "Total number of assets deleted")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(assets_deleted.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let asset_value_total = prometheus::Gauge::new("rwa_asset_value_total", "Total value of all assets")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(asset_value_total.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        
        // User metrics
        let users_registered = prometheus::Counter::new("rwa_users_registered_total", "Total number of registered users")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(users_registered.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let users_active = prometheus::Gauge::new("rwa_users_active", "Number of active users")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(users_active.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let user_sessions = prometheus::Gauge::new("rwa_user_sessions", "Number of active user sessions")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(user_sessions.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        
        // Payment metrics
        let payments_initiated = prometheus::Counter::new("rwa_payments_initiated_total", "Total number of payments initiated")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(payments_initiated.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let payments_completed = prometheus::Counter::new("rwa_payments_completed_total", "Total number of payments completed")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(payments_completed.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let payments_failed = prometheus::Counter::new("rwa_payments_failed_total", "Total number of payments failed")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(payments_failed.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let payment_volume = prometheus::Counter::new("rwa_payment_volume_total", "Total payment volume processed")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(payment_volume.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        
        // Blockchain metrics
        let blockchain_transactions = prometheus::CounterVec::new(
            prometheus::Opts::new("rwa_blockchain_transactions_total", "Total blockchain transactions by chain"),
            &["chain", "status"]
        ).map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(blockchain_transactions.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let blockchain_balance_checks = prometheus::CounterVec::new(
            prometheus::Opts::new("rwa_blockchain_balance_checks_total", "Total blockchain balance checks by chain"),
            &["chain"]
        ).map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(blockchain_balance_checks.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let blockchain_connection_status = prometheus::GaugeVec::new(
            prometheus::Opts::new("rwa_blockchain_connection_status", "Blockchain connection status by chain"),
            &["chain"]
        ).map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(blockchain_connection_status.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        
        // System metrics
        let http_requests_total = prometheus::CounterVec::new(
            prometheus::Opts::new("rwa_http_requests_total", "Total HTTP requests"),
            &["method", "endpoint", "status"]
        ).map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(http_requests_total.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let http_request_duration = prometheus::HistogramVec::new(
            prometheus::HistogramOpts::new("rwa_http_request_duration_seconds", "HTTP request duration"),
            &["method", "endpoint"]
        ).map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(http_request_duration.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let database_connections = prometheus::Gauge::new("rwa_database_connections", "Number of active database connections")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(database_connections.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let cache_hits = prometheus::Counter::new("rwa_cache_hits_total", "Total cache hits")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(cache_hits.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
            
        let cache_misses = prometheus::Counter::new("rwa_cache_misses_total", "Total cache misses")
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        registry.register(Box::new(cache_misses.clone()))
            .map_err(|e| ObservabilityError::Metrics(e.to_string()))?;
        
        Ok(Self {
            assets_created,
            assets_updated,
            assets_deleted,
            asset_value_total,
            users_registered,
            users_active,
            user_sessions,
            payments_initiated,
            payments_completed,
            payments_failed,
            payment_volume,
            blockchain_transactions,
            blockchain_balance_checks,
            blockchain_connection_status,
            http_requests_total,
            http_request_duration,
            database_connections,
            cache_hits,
            cache_misses,
        })
    }
}

/// Trace context for distributed tracing
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4().to_string(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            baggage: HashMap::new(),
        }
    }
    
    pub fn child_span(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            baggage: self.baggage.clone(),
        }
    }
}
