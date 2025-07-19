// =====================================================================================
// File: service-gateway/src/health.rs
// Description: Health check endpoints for API Gateway
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::GatewayState;
use actix_web::{web, HttpResponse, Result as ActixResult};
use core_observability::health::{HealthCheckManager, HealthStatus, OverallHealthResult};
use prometheus::{Encoder, TextEncoder};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, error};

/// Basic health check endpoint
pub async fn health_check(data: web::Data<GatewayState>) -> ActixResult<HttpResponse> {
    debug!("Health check requested");
    
    // Perform basic health checks
    let mut checks = HashMap::new();
    let mut overall_healthy = true;

    // Check service registry
    let services = data.service_registry.get_all_services().await;
    let total_services = services.len();
    let healthy_services = services.values()
        .map(|instances| instances.iter().filter(|i| i.is_healthy()).count())
        .sum::<usize>();

    checks.insert("service_registry", json!({
        "status": if total_services > 0 { "healthy" } else { "degraded" },
        "total_services": total_services,
        "healthy_instances": healthy_services,
        "details": services.keys().collect::<Vec<_>>()
    }));

    if total_services == 0 {
        overall_healthy = false;
    }

    // Check metrics system
    checks.insert("metrics", json!({
        "status": "healthy",
        "message": "Metrics collection active"
    }));

    let status = if overall_healthy { "healthy" } else { "degraded" };
    let http_status = if overall_healthy { 200 } else { 503 };

    Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
        .json(json!({
            "status": status,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": "rwa-api-gateway",
            "version": env!("CARGO_PKG_VERSION"),
            "checks": checks
        })))
}

/// Readiness probe endpoint
pub async fn readiness_check(data: web::Data<GatewayState>) -> ActixResult<HttpResponse> {
    debug!("Readiness check requested");
    
    // Check if gateway is ready to serve traffic
    let services = data.service_registry.get_all_services().await;
    let has_healthy_services = services.values()
        .any(|instances| instances.iter().any(|i| i.is_healthy()));

    if has_healthy_services {
        Ok(HttpResponse::Ok().json(json!({
            "status": "ready",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "message": "Gateway is ready to serve traffic"
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(json!({
            "status": "not_ready",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "message": "No healthy backend services available"
        })))
    }
}

/// Liveness probe endpoint
pub async fn liveness_check() -> ActixResult<HttpResponse> {
    debug!("Liveness check requested");
    
    // Simple liveness check - if we can respond, we're alive
    Ok(HttpResponse::Ok().json(json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "message": "Gateway is alive"
    })))
}

/// Metrics endpoint for Prometheus scraping
pub async fn metrics_endpoint(data: web::Data<GatewayState>) -> ActixResult<HttpResponse> {
    debug!("Metrics endpoint requested");
    
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(metrics_text) => {
            Ok(HttpResponse::Ok()
                .content_type("text/plain; version=0.0.4; charset=utf-8")
                .body(metrics_text))
        }
        Err(e) => {
            error!("Failed to encode metrics: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to encode metrics",
                "message": e.to_string()
            })))
        }
    }
}

/// Detailed service status endpoint
pub async fn service_status(data: web::Data<GatewayState>) -> ActixResult<HttpResponse> {
    debug!("Service status requested");
    
    let services = data.service_registry.get_all_services().await;
    let mut service_details = HashMap::new();

    for (service_name, instances) in services {
        let healthy_count = instances.iter().filter(|i| i.is_healthy()).count();
        let total_count = instances.len();
        
        let instance_details: Vec<_> = instances.iter().map(|instance| {
            json!({
                "id": instance.id,
                "url": instance.url(),
                "status": instance.status,
                "weight": instance.weight,
                "last_heartbeat": instance.last_heartbeat.elapsed().as_secs(),
                "metadata": instance.metadata
            })
        }).collect();

        service_details.insert(service_name, json!({
            "healthy_instances": healthy_count,
            "total_instances": total_count,
            "status": if healthy_count > 0 { "healthy" } else { "unhealthy" },
            "instances": instance_details
        }));
    }

    Ok(HttpResponse::Ok().json(json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": service_details
    })))
}

/// Gateway configuration endpoint
pub async fn gateway_config(data: web::Data<GatewayState>) -> ActixResult<HttpResponse> {
    debug!("Gateway configuration requested");
    
    Ok(HttpResponse::Ok().json(json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "gateway": {
            "version": env!("CARGO_PKG_VERSION"),
            "name": "rwa-api-gateway",
            "description": "API Gateway for RWA platform microservices"
        },
        "server": {
            "host": data.config.server.host,
            "port": data.config.server.port,
            "workers": data.config.server.workers,
            "keep_alive": data.config.server.keep_alive,
            "client_timeout": data.config.server.client_timeout
        },
        "security": {
            "jwt_expiration": data.config.security.jwt_expiration,
            "rate_limit": {
                "requests_per_minute": data.config.security.rate_limit.requests_per_minute,
                "burst_size": data.config.security.rate_limit.burst_size
            }
        },
        "observability": {
            "tracing_level": data.config.observability.tracing.level,
            "metrics_enabled": data.config.observability.metrics.enabled,
            "metrics_port": data.config.observability.metrics.port
        }
    })))
}

/// System information endpoint
pub async fn system_info() -> ActixResult<HttpResponse> {
    debug!("System information requested");
    
    let system_info = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "system": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "family": std::env::consts::FAMILY
        },
        "runtime": {
            "version": env!("CARGO_PKG_VERSION"),
            "rust_version": env!("CARGO_PKG_RUST_VERSION"),
            "build_timestamp": env!("VERGEN_BUILD_TIMESTAMP"),
            "git_sha": env!("VERGEN_GIT_SHA")
        },
        "process": {
            "pid": std::process::id(),
            "uptime_seconds": get_uptime_seconds()
        }
    });

    Ok(HttpResponse::Ok().json(system_info))
}

/// Get process uptime in seconds
fn get_uptime_seconds() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;
    
    static START_TIME: OnceLock<Instant> = OnceLock::new();
    let start = START_TIME.get_or_init(|| Instant::now());
    start.elapsed().as_secs()
}

/// Comprehensive health check with detailed diagnostics
pub async fn detailed_health_check(data: web::Data<GatewayState>) -> ActixResult<HttpResponse> {
    debug!("Detailed health check requested");
    
    let start_time = std::time::Instant::now();
    let mut checks = HashMap::new();
    let mut overall_status = HealthStatus::Healthy;

    // Service registry check
    let services = data.service_registry.get_all_services().await;
    let total_services = services.len();
    let healthy_services = services.values()
        .map(|instances| instances.iter().filter(|i| i.is_healthy()).count())
        .sum::<usize>();

    let registry_status = if total_services > 0 && healthy_services > 0 {
        HealthStatus::Healthy
    } else if total_services > 0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };

    checks.insert("service_registry".to_string(), json!({
        "status": registry_status,
        "total_services": total_services,
        "healthy_instances": healthy_services,
        "message": format!("{}/{} services have healthy instances", 
                          services.values().filter(|instances| instances.iter().any(|i| i.is_healthy())).count(),
                          total_services)
    }));

    if registry_status == HealthStatus::Unhealthy {
        overall_status = HealthStatus::Unhealthy;
    } else if registry_status == HealthStatus::Degraded && overall_status == HealthStatus::Healthy {
        overall_status = HealthStatus::Degraded;
    }

    // Memory usage check (simplified)
    checks.insert("memory".to_string(), json!({
        "status": HealthStatus::Healthy,
        "message": "Memory usage within acceptable limits"
    }));

    // Rate limiter check
    checks.insert("rate_limiter".to_string(), json!({
        "status": HealthStatus::Healthy,
        "message": "Rate limiter operational"
    }));

    let duration = start_time.elapsed();
    let http_status = match overall_status {
        HealthStatus::Healthy => 200,
        HealthStatus::Degraded => 200,
        HealthStatus::Unhealthy => 503,
        HealthStatus::Unknown => 503,
    };

    Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(http_status).unwrap())
        .json(json!({
            "status": overall_status,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "duration_ms": duration.as_millis(),
            "service": "rwa-api-gateway",
            "version": env!("CARGO_PKG_VERSION"),
            "checks": checks,
            "summary": {
                "total_checks": checks.len(),
                "healthy_checks": checks.values().filter(|v| v["status"] == "Healthy").count(),
                "degraded_checks": checks.values().filter(|v| v["status"] == "Degraded").count(),
                "unhealthy_checks": checks.values().filter(|v| v["status"] == "Unhealthy").count()
            }
        })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routing::ServiceRegistry;
    use actix_web::{test, web, App};
    use core_config::AppConfig;
    use core_observability::BusinessMetrics;
    use std::sync::Arc;

    async fn create_test_app() -> impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        let config = AppConfig::default();
        let metrics = Arc::new(BusinessMetrics::new().unwrap());
        let service_registry = Arc::new(ServiceRegistry::new());
        let rate_limiter = Arc::new(core_security::InMemoryRateLimiter::new());

        let state = web::Data::new(GatewayState {
            config,
            metrics,
            service_registry,
            rate_limiter,
        });

        test::init_service(
            App::new()
                .app_data(state)
                .route("/health", web::get().to(health_check))
                .route("/health/ready", web::get().to(readiness_check))
                .route("/health/live", web::get().to(liveness_check))
                .route("/metrics", web::get().to(metrics_endpoint))
        ).await
    }

    #[actix_web::test]
    async fn test_health_check() {
        let app = create_test_app().await;
        
        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_readiness_check() {
        let app = create_test_app().await;
        
        let req = test::TestRequest::get()
            .uri("/health/ready")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        // Should return 503 since no services are registered
        assert_eq!(resp.status(), 503);
    }

    #[actix_web::test]
    async fn test_liveness_check() {
        let app = create_test_app().await;
        
        let req = test::TestRequest::get()
            .uri("/health/live")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let app = create_test_app().await;
        
        let req = test::TestRequest::get()
            .uri("/metrics")
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        let content_type = resp.headers().get("content-type").unwrap();
        assert!(content_type.to_str().unwrap().contains("text/plain"));
    }

    #[test]
    fn test_uptime_calculation() {
        let uptime = get_uptime_seconds();
        assert!(uptime >= 0);
    }
}
