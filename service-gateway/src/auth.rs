// =====================================================================================
// File: service-gateway/src/auth.rs
// Description: Authentication and authorization framework for StableRWA API Gateway
// Author: arkSong (arksong2018@gmail.com)
// Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
// =====================================================================================

use crate::GatewayError;
use actix_web::{dev::ServiceRequest, web, HttpMessage, HttpRequest};
use core_security::{jwt::JwtManager, SecurityError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, error, warn};

/// User claims structure extracted from JWT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

/// Authentication service component for the framework gateway
pub struct AuthService {
    jwt_manager: JwtManager,
    public_routes: HashSet<String>,
}

impl AuthService {
    pub fn new(jwt_manager: JwtManager) -> Self {
        let mut public_routes = HashSet::new();
        
        // Add public routes that don't require authentication
        public_routes.insert("/health".to_string());
        public_routes.insert("/health/ready".to_string());
        public_routes.insert("/health/live".to_string());
        public_routes.insert("/metrics".to_string());
        public_routes.insert("/auth/login".to_string());
        public_routes.insert("/auth/register".to_string());
        public_routes.insert("/auth/refresh".to_string());
        public_routes.insert("/docs".to_string());
        public_routes.insert("/swagger-ui".to_string());

        Self {
            jwt_manager,
            public_routes,
        }
    }

    /// Check if a route is public (doesn't require authentication)
    pub fn is_public_route(&self, path: &str) -> bool {
        self.public_routes.contains(path) || 
        path.starts_with("/docs/") ||
        path.starts_with("/swagger-ui/") ||
        path.starts_with("/static/")
    }

    /// Extract and validate JWT token from request
    pub async fn authenticate_request(&self, req: &ServiceRequest) -> Result<UserClaims, GatewayError> {
        let path = req.path();
        
        // Skip authentication for public routes
        if self.is_public_route(path) {
            debug!("Skipping authentication for public route: {}", path);
            return Err(GatewayError::AuthenticationFailed("Public route".to_string()));
        }

        // Extract token from Authorization header
        let token = self.extract_token_from_request(req)?;
        
        // Validate and decode token
        match self.jwt_manager.validate_token(&token).await {
            Ok(claims) => {
                debug!("Successfully authenticated user: {}", claims.get("user_id").unwrap_or(&serde_json::Value::Null));
                
                // Convert claims to UserClaims
                let user_claims = self.claims_to_user_claims(claims)?;
                
                // Store user claims in request extensions for downstream use
                req.extensions_mut().insert(user_claims.clone());
                
                Ok(user_claims)
            }
            Err(SecurityError::TokenExpired) => {
                warn!("Token expired for request to: {}", path);
                Err(GatewayError::AuthenticationFailed("Token expired".to_string()))
            }
            Err(SecurityError::InvalidToken(msg)) => {
                warn!("Invalid token for request to {}: {}", path, msg);
                Err(GatewayError::AuthenticationFailed(format!("Invalid token: {}", msg)))
            }
            Err(e) => {
                error!("Authentication error for request to {}: {}", path, e);
                Err(GatewayError::AuthenticationFailed(e.to_string()))
            }
        }
    }

    /// Extract JWT token from Authorization header
    fn extract_token_from_request(&self, req: &ServiceRequest) -> Result<String, GatewayError> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .ok_or_else(|| GatewayError::AuthenticationFailed("Missing Authorization header".to_string()))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| GatewayError::AuthenticationFailed("Invalid Authorization header format".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(GatewayError::AuthenticationFailed("Invalid Authorization header format".to_string()));
        }

        let token = auth_str.trim_start_matches("Bearer ").trim();
        if token.is_empty() {
            return Err(GatewayError::AuthenticationFailed("Empty token".to_string()));
        }

        Ok(token.to_string())
    }

    /// Convert JWT claims to UserClaims struct
    fn claims_to_user_claims(&self, claims: serde_json::Map<String, serde_json::Value>) -> Result<UserClaims, GatewayError> {
        let user_id = claims
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GatewayError::AuthenticationFailed("Missing user_id in token".to_string()))?
            .to_string();

        let email = claims
            .get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GatewayError::AuthenticationFailed("Missing email in token".to_string()))?
            .to_string();

        let roles = claims
            .get("roles")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let permissions = claims
            .get("permissions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let exp = claims
            .get("exp")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| GatewayError::AuthenticationFailed("Missing exp in token".to_string()))?;

        let iat = claims
            .get("iat")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| GatewayError::AuthenticationFailed("Missing iat in token".to_string()))?;

        Ok(UserClaims {
            user_id,
            email,
            roles,
            permissions,
            exp,
            iat,
        })
    }

    /// Check if user has required permission
    pub fn check_permission(&self, user_claims: &UserClaims, required_permission: &str) -> bool {
        // Admin role has all permissions
        if user_claims.roles.contains(&"admin".to_string()) {
            return true;
        }

        // Check specific permission
        user_claims.permissions.contains(&required_permission.to_string())
    }

    /// Check if user has required role
    pub fn check_role(&self, user_claims: &UserClaims, required_role: &str) -> bool {
        user_claims.roles.contains(&required_role.to_string())
    }

    /// Get permission requirements for different routes
    pub fn get_route_permissions(&self, method: &str, path: &str) -> Vec<String> {
        let mut permissions = Vec::new();

        match (method, path) {
            // Asset permissions
            ("GET", path) if path.starts_with("/api/v1/assets") => {
                permissions.push("assets:read".to_string());
            }
            ("POST", path) if path.starts_with("/api/v1/assets") => {
                permissions.push("assets:create".to_string());
            }
            ("PUT", path) if path.starts_with("/api/v1/assets") => {
                permissions.push("assets:update".to_string());
            }
            ("DELETE", path) if path.starts_with("/api/v1/assets") => {
                permissions.push("assets:delete".to_string());
            }

            // User permissions
            ("GET", path) if path.starts_with("/api/v1/users") => {
                permissions.push("users:read".to_string());
            }
            ("POST", path) if path.starts_with("/api/v1/users") => {
                permissions.push("users:create".to_string());
            }
            ("PUT", path) if path.starts_with("/api/v1/users") => {
                permissions.push("users:update".to_string());
            }
            ("DELETE", path) if path.starts_with("/api/v1/users") => {
                permissions.push("users:delete".to_string());
            }

            // Payment permissions
            ("GET", path) if path.starts_with("/api/v1/payments") => {
                permissions.push("payments:read".to_string());
            }
            ("POST", path) if path.starts_with("/api/v1/payments") => {
                permissions.push("payments:create".to_string());
            }

            // Blockchain permissions
            ("GET", path) if path.starts_with("/api/v1/blockchain") => {
                permissions.push("blockchain:read".to_string());
            }
            ("POST", path) if path.starts_with("/api/v1/blockchain") => {
                permissions.push("blockchain:write".to_string());
            }

            _ => {
                // Default permission for authenticated routes
                permissions.push("api:access".to_string());
            }
        }

        permissions
    }

    /// Authorize request based on user claims and route requirements
    pub fn authorize_request(&self, user_claims: &UserClaims, method: &str, path: &str) -> Result<(), GatewayError> {
        let required_permissions = self.get_route_permissions(method, path);

        for permission in required_permissions {
            if !self.check_permission(user_claims, &permission) {
                warn!(
                    "User {} lacks permission {} for {} {}",
                    user_claims.user_id, permission, method, path
                );
                return Err(GatewayError::AuthenticationFailed(
                    format!("Insufficient permissions: {}", permission)
                ));
            }
        }

        debug!(
            "User {} authorized for {} {}",
            user_claims.user_id, method, path
        );

        Ok(())
    }
}

/// Extract user claims from request extensions
pub fn get_user_claims(req: &HttpRequest) -> Option<UserClaims> {
    req.extensions().get::<UserClaims>().cloned()
}

/// Check if current user has specific permission
pub fn has_permission(req: &HttpRequest, permission: &str) -> bool {
    if let Some(claims) = get_user_claims(req) {
        claims.permissions.contains(&permission.to_string()) || 
        claims.roles.contains(&"admin".to_string())
    } else {
        false
    }
}

/// Check if current user has specific role
pub fn has_role(req: &HttpRequest, role: &str) -> bool {
    if let Some(claims) = get_user_claims(req) {
        claims.roles.contains(&role.to_string())
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_security::jwt::JwtConfig;

    fn create_test_auth_service() -> AuthService {
        let jwt_config = JwtConfig {
            secret: "test_secret_key_32_characters_long".to_string(),
            expiration: 3600,
            refresh_expiration: 86400,
            issuer: "test".to_string(),
            audience: "test".to_string(),
        };
        let jwt_manager = JwtManager::new(jwt_config);
        AuthService::new(jwt_manager)
    }

    #[test]
    fn test_is_public_route() {
        let auth_service = create_test_auth_service();
        
        assert!(auth_service.is_public_route("/health"));
        assert!(auth_service.is_public_route("/auth/login"));
        assert!(auth_service.is_public_route("/docs/api"));
        assert!(!auth_service.is_public_route("/api/v1/assets"));
    }

    #[test]
    fn test_get_route_permissions() {
        let auth_service = create_test_auth_service();
        
        let permissions = auth_service.get_route_permissions("GET", "/api/v1/assets");
        assert!(permissions.contains(&"assets:read".to_string()));
        
        let permissions = auth_service.get_route_permissions("POST", "/api/v1/payments");
        assert!(permissions.contains(&"payments:create".to_string()));
    }

    #[test]
    fn test_check_permission() {
        let auth_service = create_test_auth_service();
        
        let user_claims = UserClaims {
            user_id: "test_user".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec!["assets:read".to_string()],
            exp: 9999999999,
            iat: 1000000000,
        };
        
        assert!(auth_service.check_permission(&user_claims, "assets:read"));
        assert!(!auth_service.check_permission(&user_claims, "assets:delete"));
        
        // Test admin role
        let admin_claims = UserClaims {
            user_id: "admin_user".to_string(),
            email: "admin@example.com".to_string(),
            roles: vec!["admin".to_string()],
            permissions: vec![],
            exp: 9999999999,
            iat: 1000000000,
        };
        
        assert!(auth_service.check_permission(&admin_claims, "assets:delete"));
    }
}
