// =====================================================================================
// File: core-observability/src/tracing_setup.rs
// Description: Distributed tracing setup and utilities
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{ObservabilityError, TraceContext};
use crate::logging::TracingConfig;
use std::collections::HashMap;
use tracing::{span, Instrument, Level, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

/// Tracing setup and configuration
pub struct TracingSetup;

impl TracingSetup {
    /// Initialize distributed tracing
    pub fn init(config: &TracingConfig) -> Result<(), ObservabilityError> {
        // Parse log level
        let level = match config.level.to_lowercase().as_str() {
            "error" => Level::ERROR,
            "warn" => Level::WARN,
            "info" => Level::INFO,
            "debug" => Level::DEBUG,
            "trace" => Level::TRACE,
            _ => Level::INFO,
        };

        let registry = tracing_subscriber::registry();

        // Add console layer
        let console_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true);

        let registry = registry.with(console_layer);

        // Add OpenTelemetry layer if Jaeger endpoint is configured
        #[cfg(feature = "jaeger")]
        let registry = if let Some(jaeger_endpoint) = &config.jaeger_endpoint {
            use opentelemetry::trace::TracerProvider;
            use opentelemetry_jaeger::JaegerTraceExporter;
            use tracing_opentelemetry::OpenTelemetryLayer;

            let tracer = opentelemetry_jaeger::new_agent_pipeline()
                .with_service_name(&config.service_name)
                .with_endpoint(jaeger_endpoint)
                .install_simple()
                .map_err(|e| {
                    ObservabilityError::Tracing(format!("Failed to init Jaeger: {}", e))
                })?;

            registry.with(OpenTelemetryLayer::new(tracer))
        } else {
            registry
        };

        // Initialize the subscriber
        registry
            .try_init()
            .map_err(|e| ObservabilityError::Tracing(format!("Failed to init tracing: {}", e)))?;

        tracing::info!(
            service = %config.service_name,
            level = %config.level,
            jaeger_endpoint = ?config.jaeger_endpoint,
            "Distributed tracing initialized"
        );

        Ok(())
    }

    /// Create a new trace context
    pub fn new_trace_context() -> TraceContext {
        TraceContext::new()
    }

    /// Create a child trace context
    pub fn child_trace_context(parent: &TraceContext) -> TraceContext {
        parent.child_span()
    }
}

/// Span utilities for creating and managing spans
pub struct SpanUtils;

impl SpanUtils {
    /// Create a new span with trace context
    pub fn create_span(
        name: &str,
        trace_context: &TraceContext,
        fields: Option<HashMap<String, String>>,
    ) -> Span {
        let mut span = span!(Level::INFO, "operation", operation = name);

        // Add trace context fields
        span.record("trace_id", &trace_context.trace_id.as_str());
        span.record("span_id", &trace_context.span_id.as_str());

        if let Some(parent_span_id) = &trace_context.parent_span_id {
            span.record("parent_span_id", parent_span_id.as_str());
        }

        // Add custom fields
        if let Some(fields) = fields {
            for (key, value) in fields {
                span.record(key.as_str(), value.as_str());
            }
        }

        span
    }

    /// Create HTTP request span
    pub fn create_http_span(method: &str, path: &str, trace_context: &TraceContext) -> Span {
        let mut fields = HashMap::new();
        fields.insert("http.method".to_string(), method.to_string());
        fields.insert("http.path".to_string(), path.to_string());
        fields.insert("span.kind".to_string(), "server".to_string());

        Self::create_span("http_request", trace_context, Some(fields))
    }

    /// Create database operation span
    pub fn create_db_span(operation: &str, table: &str, trace_context: &TraceContext) -> Span {
        let mut fields = HashMap::new();
        fields.insert("db.operation".to_string(), operation.to_string());
        fields.insert("db.table".to_string(), table.to_string());
        fields.insert("span.kind".to_string(), "client".to_string());

        Self::create_span("db_operation", trace_context, Some(fields))
    }

    /// Create blockchain operation span
    pub fn create_blockchain_span(
        chain: &str,
        operation: &str,
        trace_context: &TraceContext,
    ) -> Span {
        let mut fields = HashMap::new();
        fields.insert("blockchain.chain".to_string(), chain.to_string());
        fields.insert("blockchain.operation".to_string(), operation.to_string());
        fields.insert("span.kind".to_string(), "client".to_string());

        Self::create_span("blockchain_operation", trace_context, Some(fields))
    }

    /// Create external service call span
    pub fn create_external_call_span(
        service: &str,
        endpoint: &str,
        trace_context: &TraceContext,
    ) -> Span {
        let mut fields = HashMap::new();
        fields.insert("external.service".to_string(), service.to_string());
        fields.insert("external.endpoint".to_string(), endpoint.to_string());
        fields.insert("span.kind".to_string(), "client".to_string());

        Self::create_span("external_call", trace_context, Some(fields))
    }

    /// Add error information to span
    pub fn record_error<E: std::fmt::Display>(span: &Span, error: &E) {
        span.record("error", &true);
        span.record("error.message", &error.to_string());
        span.record("error.type", &std::any::type_name::<E>());
    }

    /// Add success information to span
    pub fn record_success(span: &Span) {
        span.record("success", &true);
        span.record("error", &false);
    }
}

/// Tracing middleware for automatic span creation
pub struct TracingMiddleware {
    service_name: String,
}

impl TracingMiddleware {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// Create trace context from HTTP headers
    pub fn extract_trace_context(&self, headers: &HashMap<String, String>) -> TraceContext {
        // Extract trace context from headers (simplified implementation)
        let trace_id = headers
            .get("x-trace-id")
            .cloned()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let parent_span_id = headers.get("x-span-id").cloned();

        TraceContext {
            trace_id,
            span_id: Uuid::new_v4().to_string(),
            parent_span_id,
            baggage: HashMap::new(),
        }
    }

    /// Inject trace context into HTTP headers
    pub fn inject_trace_context(
        &self,
        trace_context: &TraceContext,
        headers: &mut HashMap<String, String>,
    ) {
        headers.insert("x-trace-id".to_string(), trace_context.trace_id.clone());
        headers.insert("x-span-id".to_string(), trace_context.span_id.clone());

        if let Some(parent_span_id) = &trace_context.parent_span_id {
            headers.insert("x-parent-span-id".to_string(), parent_span_id.clone());
        }

        // Add baggage
        for (key, value) in &trace_context.baggage {
            headers.insert(format!("x-baggage-{}", key), value.clone());
        }
    }
}

/// Async function instrumentation helper
pub struct AsyncInstrumentation;

impl AsyncInstrumentation {
    /// Instrument an async function with tracing
    pub async fn instrument<F, T>(
        operation_name: &str,
        trace_context: &TraceContext,
        future: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let span = SpanUtils::create_span(operation_name, trace_context, None);
        future.instrument(span).await
    }

    /// Instrument an async function with custom fields
    pub async fn instrument_with_fields<F, T>(
        operation_name: &str,
        trace_context: &TraceContext,
        fields: HashMap<String, String>,
        future: F,
    ) -> T
    where
        F: std::future::Future<Output = T>,
    {
        let span = SpanUtils::create_span(operation_name, trace_context, Some(fields));
        future.instrument(span).await
    }

    /// Instrument an async function and handle errors
    pub async fn instrument_with_error_handling<F, T, E>(
        operation_name: &str,
        trace_context: &TraceContext,
        future: F,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let span = SpanUtils::create_span(operation_name, trace_context, None);

        let result = future.instrument(span.clone()).await;

        match &result {
            Ok(_) => SpanUtils::record_success(&span),
            Err(e) => SpanUtils::record_error(&span, e),
        }

        result
    }
}

/// Trace sampling configuration
#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub sample_rate: f64,
    pub max_traces_per_second: Option<u32>,
    pub service_specific_rates: HashMap<String, f64>,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            sample_rate: 0.1, // 10% sampling by default
            max_traces_per_second: Some(100),
            service_specific_rates: HashMap::new(),
        }
    }
}

impl SamplingConfig {
    /// Check if a trace should be sampled
    pub fn should_sample(&self, service: &str, trace_id: &str) -> bool {
        let rate = self
            .service_specific_rates
            .get(service)
            .copied()
            .unwrap_or(self.sample_rate);

        // Simple hash-based sampling
        let hash = self.hash_trace_id(trace_id);
        (hash as f64 / u64::MAX as f64) < rate
    }

    fn hash_trace_id(&self, trace_id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        trace_id.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_creation() {
        let context = TracingSetup::new_trace_context();
        assert!(!context.trace_id.is_empty());
        assert!(!context.span_id.is_empty());
        assert!(context.parent_span_id.is_none());
    }

    #[test]
    fn test_child_trace_context() {
        let parent = TracingSetup::new_trace_context();
        let child = TracingSetup::child_trace_context(&parent);

        assert_eq!(child.trace_id, parent.trace_id);
        assert_ne!(child.span_id, parent.span_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_span_creation() {
        let context = TracingSetup::new_trace_context();
        let span = SpanUtils::create_span("test_operation", &context, None);

        // Span should be created successfully
        assert_eq!(span.metadata().unwrap().name(), "operation");
    }

    #[test]
    fn test_tracing_middleware() {
        let middleware = TracingMiddleware::new("test-service".to_string());

        let mut headers = HashMap::new();
        headers.insert("x-trace-id".to_string(), "test-trace-123".to_string());

        let context = middleware.extract_trace_context(&headers);
        assert_eq!(context.trace_id, "test-trace-123");

        let mut new_headers = HashMap::new();
        middleware.inject_trace_context(&context, &mut new_headers);
        assert_eq!(
            new_headers.get("x-trace-id"),
            Some(&"test-trace-123".to_string())
        );
    }

    #[test]
    fn test_sampling_config() {
        let mut config = SamplingConfig::default();
        config.sample_rate = 0.5; // 50% sampling

        // Test should be deterministic for same trace ID
        let trace_id = "test-trace-123";
        let result1 = config.should_sample("test-service", trace_id);
        let result2 = config.should_sample("test-service", trace_id);
        assert_eq!(result1, result2);

        // Test service-specific rates
        config
            .service_specific_rates
            .insert("special-service".to_string(), 1.0);
        assert!(config.should_sample("special-service", trace_id));
    }

    #[tokio::test]
    async fn test_async_instrumentation() {
        let context = TracingSetup::new_trace_context();

        let result = AsyncInstrumentation::instrument("test_async_operation", &context, async {
            "test_result"
        })
        .await;

        assert_eq!(result, "test_result");
    }

    #[tokio::test]
    async fn test_async_instrumentation_with_error() {
        let context = TracingSetup::new_trace_context();

        let result: Result<&str, &str> = AsyncInstrumentation::instrument_with_error_handling(
            "test_async_operation",
            &context,
            async { Err("test_error") },
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "test_error");
    }
}
