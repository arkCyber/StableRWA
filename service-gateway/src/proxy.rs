// =====================================================================================
// File: service-gateway/src/proxy.rs
// Description: HTTP proxy functionality for routing requests to microservices
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{GatewayError, GatewayState};
use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use reqwest::Client;
use serde_json::Value;
use std::time::Instant;
use tracing::{error, info, warn};

/// HTTP client for proxying requests
pub struct ProxyClient {
    client: Client,
}

impl ProxyClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }
    
    /// Proxy request to a target service
    pub async fn proxy_request(
        &self,
        target_url: &str,
        method: &str,
        headers: &actix_web::http::HeaderMap,
        body: Option<web::Bytes>,
    ) -> Result<HttpResponse, GatewayError> {
        let start_time = Instant::now();
        
        info!("Proxying {} request to {}", method, target_url);
        
        // Build request
        let mut request = match method {
            "GET" => self.client.get(target_url),
            "POST" => self.client.post(target_url),
            "PUT" => self.client.put(target_url),
            "DELETE" => self.client.delete(target_url),
            "PATCH" => self.client.patch(target_url),
            _ => return Err(GatewayError::ProxyError(format!("Unsupported method: {}", method))),
        };
        
        // Copy headers (excluding hop-by-hop headers)
        for (name, value) in headers {
            let header_name = name.as_str();
            if !is_hop_by_hop_header(header_name) {
                if let Ok(header_value) = value.to_str() {
                    request = request.header(header_name, header_value);
                }
            }
        }
        
        // Add body if present
        if let Some(body_bytes) = body {
            request = request.body(body_bytes.to_vec());
        }
        
        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| GatewayError::ProxyError(format!("Request failed: {}", e)))?;
        
        let status = response.status();
        let response_headers = response.headers().clone();
        let response_body = response
            .bytes()
            .await
            .map_err(|e| GatewayError::ProxyError(format!("Failed to read response body: {}", e)))?;
        
        let duration = start_time.elapsed();
        info!("Proxy request completed in {:?} with status {}", duration, status);
        
        // Build response
        let mut http_response = HttpResponse::build(status);
        
        // Copy response headers (excluding hop-by-hop headers)
        for (name, value) in response_headers {
            if let Some(header_name) = name {
                if !is_hop_by_hop_header(header_name.as_str()) {
                    http_response.insert_header((header_name.as_str(), value.as_bytes()));
                }
            }
        }
        
        Ok(http_response.body(response_body))
    }
}

/// Check if a header is hop-by-hop and should not be forwarded
fn is_hop_by_hop_header(header_name: &str) -> bool {
    matches!(header_name.to_lowercase().as_str(),
        "connection" | "keep-alive" | "proxy-authenticate" | "proxy-authorization" |
        "te" | "trailers" | "transfer-encoding" | "upgrade"
    )
}

/// Proxy request to asset service
pub async fn proxy_to_asset_service(
    req: HttpRequest,
    body: web::Bytes,
    data: web::Data<GatewayState>,
) -> ActixResult<HttpResponse> {
    let service_url = data.service_registry
        .get_service_url("asset-service")
        .ok_or_else(|| {
            error!("Asset service not found in registry");
            actix_web::error::ErrorServiceUnavailable("Asset service unavailable")
        })?;
    
    let target_url = format!("{}{}", service_url, req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
    let method = req.method().as_str();
    
    // Update metrics
    data.metrics.http_requests_total
        .with_label_values(&[method, "/assets", "proxied"])
        .inc();
    
    let proxy_client = ProxyClient::new();
    let body_option = if body.is_empty() { None } else { Some(body) };
    
    match proxy_client.proxy_request(&target_url, method, req.headers(), body_option).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed to proxy to asset service: {}", e);
            data.metrics.http_requests_total
                .with_label_values(&[method, "/assets", "error"])
                .inc();
            Err(actix_web::error::ErrorBadGateway(e.to_string()))
        }
    }
}

/// Proxy request to user service
pub async fn proxy_to_user_service(
    req: HttpRequest,
    body: web::Bytes,
    data: web::Data<GatewayState>,
) -> ActixResult<HttpResponse> {
    let service_url = data.service_registry
        .get_service_url("user-service")
        .ok_or_else(|| {
            error!("User service not found in registry");
            actix_web::error::ErrorServiceUnavailable("User service unavailable")
        })?;
    
    let target_url = format!("{}{}", service_url, req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
    let method = req.method().as_str();
    
    // Update metrics
    data.metrics.http_requests_total
        .with_label_values(&[method, "/users", "proxied"])
        .inc();
    
    let proxy_client = ProxyClient::new();
    let body_option = if body.is_empty() { None } else { Some(body) };
    
    match proxy_client.proxy_request(&target_url, method, req.headers(), body_option).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed to proxy to user service: {}", e);
            data.metrics.http_requests_total
                .with_label_values(&[method, "/users", "error"])
                .inc();
            Err(actix_web::error::ErrorBadGateway(e.to_string()))
        }
    }
}

/// Proxy request to payment service
pub async fn proxy_to_payment_service(
    req: HttpRequest,
    body: web::Bytes,
    data: web::Data<GatewayState>,
) -> ActixResult<HttpResponse> {
    let service_url = data.service_registry
        .get_service_url("payment-service")
        .ok_or_else(|| {
            error!("Payment service not found in registry");
            actix_web::error::ErrorServiceUnavailable("Payment service unavailable")
        })?;
    
    let target_url = format!("{}{}", service_url, req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
    let method = req.method().as_str();
    
    // Update metrics
    data.metrics.http_requests_total
        .with_label_values(&[method, "/payments", "proxied"])
        .inc();
    
    let proxy_client = ProxyClient::new();
    let body_option = if body.is_empty() { None } else { Some(body) };
    
    match proxy_client.proxy_request(&target_url, method, req.headers(), body_option).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed to proxy to payment service: {}", e);
            data.metrics.http_requests_total
                .with_label_values(&[method, "/payments", "error"])
                .inc();
            Err(actix_web::error::ErrorBadGateway(e.to_string()))
        }
    }
}

/// Proxy request to auth service
pub async fn proxy_to_auth_service(
    req: HttpRequest,
    body: web::Bytes,
    data: web::Data<GatewayState>,
) -> ActixResult<HttpResponse> {
    let service_url = data.service_registry
        .get_service_url("auth-service")
        .ok_or_else(|| {
            error!("Auth service not found in registry");
            actix_web::error::ErrorServiceUnavailable("Auth service unavailable")
        })?;
    
    let target_url = format!("{}{}", service_url, req.uri().path_and_query().map(|x| x.as_str()).unwrap_or(""));
    let method = req.method().as_str();
    
    // Update metrics
    data.metrics.http_requests_total
        .with_label_values(&[method, "/auth", "proxied"])
        .inc();
    
    let proxy_client = ProxyClient::new();
    let body_option = if body.is_empty() { None } else { Some(body) };
    
    match proxy_client.proxy_request(&target_url, method, req.headers(), body_option).await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed to proxy to auth service: {}", e);
            data.metrics.http_requests_total
                .with_label_values(&[method, "/auth", "error"])
                .inc();
            Err(actix_web::error::ErrorBadGateway(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hop_by_hop_headers() {
        assert!(is_hop_by_hop_header("connection"));
        assert!(is_hop_by_hop_header("Connection"));
        assert!(is_hop_by_hop_header("keep-alive"));
        assert!(!is_hop_by_hop_header("content-type"));
        assert!(!is_hop_by_hop_header("authorization"));
    }
    
    #[test]
    fn test_proxy_client_creation() {
        let client = ProxyClient::new();
        // Just test that it can be created without panicking
        assert!(true);
    }
}
