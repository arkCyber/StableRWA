// =====================================================================================
// File: core-observability/src/logging.rs
// Description: Structured logging implementation for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{LogEntry, LogLevel, ErrorInfo, ObservabilityError};
use chrono::{DateTime, Utc};
use core_config::TracingConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::Writer, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use uuid::Uuid;

/// Initialize structured logging
pub fn init_logging(config: &TracingConfig) -> Result<(), ObservabilityError> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.level))
        .map_err(|e| ObservabilityError::Tracing(format!("Failed to create env filter: {}", e)))?;

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .event_format(JsonFormatter::new(&config.service_name))
        .with_filter(env_filter);

    let registry = tracing_subscriber::registry().with(fmt_layer);

    // Add Jaeger tracing if configured
    #[cfg(feature = "jaeger")]
    let registry = if let Some(jaeger_endpoint) = &config.jaeger_endpoint {
        use opentelemetry::trace::TracerProvider;
        use opentelemetry_jaeger::JaegerTraceExporter;
        use tracing_opentelemetry::OpenTelemetryLayer;

        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name(&config.service_name)
            .with_endpoint(jaeger_endpoint)
            .install_simple()
            .map_err(|e| ObservabilityError::Tracing(format!("Failed to init Jaeger: {}", e)))?;

        registry.with(OpenTelemetryLayer::new(tracer))
    } else {
        registry
    };

    registry
        .try_init()
        .map_err(|e| ObservabilityError::Tracing(format!("Failed to init tracing: {}", e)))?;

    tracing::info!("Structured logging initialized for service: {}", config.service_name);
    Ok(())
}

/// JSON formatter for structured logs
pub struct JsonFormatter {
    service_name: String,
}

impl JsonFormatter {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }
}

impl<S, N> FormatEvent<S, N> for JsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();
        
        // Extract span context
        let (trace_id, span_id) = if let Some(span) = ctx.lookup_current() {
            let extensions = span.extensions();
            // In a real implementation, you'd extract actual trace/span IDs from OpenTelemetry
            (Some(Uuid::new_v4().to_string()), Some(Uuid::new_v4().to_string()))
        } else {
            (None, None)
        };

        // Convert tracing level to our LogLevel
        let level = match *metadata.level() {
            tracing::Level::ERROR => LogLevel::Error,
            tracing::Level::WARN => LogLevel::Warn,
            tracing::Level::INFO => LogLevel::Info,
            tracing::Level::DEBUG => LogLevel::Debug,
            tracing::Level::TRACE => LogLevel::Trace,
        };

        // Extract fields
        let mut fields = HashMap::new();
        let mut visitor = FieldVisitor::new(&mut fields);
        event.record(&mut visitor);

        // Extract message
        let message = fields.remove("message")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "".to_string());

        // Check for error information
        let error = if level == LogLevel::Error {
            fields.get("error").and_then(|v| v.as_str()).map(|error_msg| {
                ErrorInfo {
                    error_type: fields.get("error_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("UnknownError")
                        .to_string(),
                    error_message: error_msg.to_string(),
                    stack_trace: fields.get("stack_trace")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                }
            })
        } else {
            None
        };

        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level,
            service: self.service_name.clone(),
            trace_id,
            span_id,
            message,
            fields,
            error,
        };

        let json = serde_json::to_string(&log_entry)
            .map_err(|_| std::fmt::Error)?;

        writeln!(writer, "{}", json)
    }
}

/// Field visitor for extracting structured data from tracing events
struct FieldVisitor<'a> {
    fields: &'a mut HashMap<String, Value>,
}

impl<'a> FieldVisitor<'a> {
    fn new(fields: &'a mut HashMap<String, Value>) -> Self {
        Self { fields }
    }
}

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            Value::String(format!("{:?}", value)),
        );
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.fields.insert(
            field.name().to_string(),
            Value::String(value.to_string()),
        );
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(
            field.name().to_string(),
            Value::Number(serde_json::Number::from(value)),
        );
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(
            field.name().to_string(),
            Value::Number(serde_json::Number::from(value)),
        );
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.insert(
            field.name().to_string(),
            Value::Bool(value),
        );
    }
}

/// Structured logging macros
#[macro_export]
macro_rules! log_info {
    ($($field:tt)*) => {
        tracing::info!($($field)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $($field:tt)*) => {
        tracing::error!(
            error = %$error,
            error_type = std::any::type_name_of_val(&$error),
            $($field)*
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($field:tt)*) => {
        tracing::warn!($($field)*)
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($field:tt)*) => {
        tracing::debug!($($field)*)
    };
}

/// Business event logger
pub struct BusinessEventLogger {
    service_name: String,
}

impl BusinessEventLogger {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// Log user action
    pub fn log_user_action(
        &self,
        user_id: &str,
        action: &str,
        resource_type: &str,
        resource_id: Option<&str>,
        success: bool,
        metadata: Option<HashMap<String, Value>>,
    ) {
        let mut fields = HashMap::new();
        fields.insert("event_type".to_string(), Value::String("user_action".to_string()));
        fields.insert("user_id".to_string(), Value::String(user_id.to_string()));
        fields.insert("action".to_string(), Value::String(action.to_string()));
        fields.insert("resource_type".to_string(), Value::String(resource_type.to_string()));
        fields.insert("success".to_string(), Value::Bool(success));

        if let Some(resource_id) = resource_id {
            fields.insert("resource_id".to_string(), Value::String(resource_id.to_string()));
        }

        if let Some(metadata) = metadata {
            for (key, value) in metadata {
                fields.insert(format!("meta_{}", key), value);
            }
        }

        if success {
            tracing::info!(
                user_id = user_id,
                action = action,
                resource_type = resource_type,
                resource_id = resource_id,
                "User action completed successfully"
            );
        } else {
            tracing::warn!(
                user_id = user_id,
                action = action,
                resource_type = resource_type,
                resource_id = resource_id,
                "User action failed"
            );
        }
    }

    /// Log system event
    pub fn log_system_event(
        &self,
        event_type: &str,
        component: &str,
        message: &str,
        metadata: Option<HashMap<String, Value>>,
    ) {
        tracing::info!(
            event_type = event_type,
            component = component,
            service = %self.service_name,
            metadata = ?metadata,
            "{}",
            message
        );
    }

    /// Log security event
    pub fn log_security_event(
        &self,
        event_type: &str,
        user_id: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        success: bool,
        message: &str,
    ) {
        if success {
            tracing::info!(
                event_type = event_type,
                user_id = user_id,
                ip_address = ip_address,
                user_agent = user_agent,
                security_event = true,
                "{}",
                message
            );
        } else {
            tracing::warn!(
                event_type = event_type,
                user_id = user_id,
                ip_address = ip_address,
                user_agent = user_agent,
                security_event = true,
                "Security event failed: {}",
                message
            );
        }
    }

    /// Log performance metrics
    pub fn log_performance(
        &self,
        operation: &str,
        duration_ms: u64,
        success: bool,
        metadata: Option<HashMap<String, Value>>,
    ) {
        tracing::info!(
            operation = operation,
            duration_ms = duration_ms,
            success = success,
            performance_metric = true,
            metadata = ?metadata,
            "Operation completed in {}ms",
            duration_ms
        );
    }
}

/// Log correlation ID for request tracing
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id() {
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();
        
        assert_ne!(id1.as_str(), id2.as_str());
        assert_eq!(id1.as_str().len(), 36); // UUID length
    }

    #[test]
    fn test_business_event_logger() {
        let logger = BusinessEventLogger::new("test-service".to_string());
        
        // Test that methods don't panic
        logger.log_user_action("user123", "create", "asset", Some("asset123"), true, None);
        logger.log_system_event("startup", "main", "Service started", None);
        logger.log_security_event("login", Some("user123"), Some("192.168.1.1"), None, true, "User logged in");
        logger.log_performance("database_query", 150, true, None);
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new("test-service");
        assert_eq!(formatter.service_name, "test-service");
    }
}
