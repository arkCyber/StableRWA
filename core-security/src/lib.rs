// =====================================================================================
// File: core-security/src/lib.rs
// Description: Security utilities and authentication for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod auth;
pub mod crypto;
pub mod jwt;
pub mod rate_limit;
pub mod validation;

pub use auth::*;
pub use crypto::*;
pub use jwt::*;
pub use rate_limit::*;
pub use validation::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Security-related errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid credentials")]
    InvalidCredentials,
}

/// User claims for JWT tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,              // Subject (user ID)
    pub email: String,            // User email
    pub roles: Vec<String>,       // User roles
    pub permissions: Vec<String>, // User permissions
    pub exp: i64,                 // Expiration time
    pub iat: i64,                 // Issued at
    pub jti: String,              // JWT ID
}

impl UserClaims {
    pub fn new(
        user_id: String,
        email: String,
        roles: Vec<String>,
        permissions: Vec<String>,
        expiration_hours: i64,
    ) -> Self {
        let now = Utc::now();
        let exp = now + chrono::Duration::hours(expiration_hours);

        Self {
            sub: user_id,
            email,
            roles,
            permissions,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

impl Session {
    pub fn new(user_id: String, duration_hours: i64) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(duration_hours);

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            created_at: now,
            expires_at,
            ip_address: None,
            user_agent: None,
            is_active: true,
            metadata: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    pub fn extend(&mut self, hours: i64) {
        self.expires_at = Utc::now() + chrono::Duration::hours(hours);
    }

    pub fn invalidate(&mut self) {
        self.is_active = false;
    }
}

/// Permission system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    // Asset permissions
    AssetRead,
    AssetWrite,
    AssetDelete,
    AssetManage,

    // User permissions
    UserRead,
    UserWrite,
    UserDelete,
    UserManage,

    // Payment permissions
    PaymentRead,
    PaymentWrite,
    PaymentProcess,
    PaymentManage,

    // System permissions
    SystemAdmin,
    SystemMonitor,
    SystemConfig,

    // Custom permission
    Custom(String),
}

impl Permission {
    pub fn as_string(&self) -> String {
        match self {
            Permission::AssetRead => "asset:read".to_string(),
            Permission::AssetWrite => "asset:write".to_string(),
            Permission::AssetDelete => "asset:delete".to_string(),
            Permission::AssetManage => "asset:manage".to_string(),
            Permission::UserRead => "user:read".to_string(),
            Permission::UserWrite => "user:write".to_string(),
            Permission::UserDelete => "user:delete".to_string(),
            Permission::UserManage => "user:manage".to_string(),
            Permission::PaymentRead => "payment:read".to_string(),
            Permission::PaymentWrite => "payment:write".to_string(),
            Permission::PaymentProcess => "payment:process".to_string(),
            Permission::PaymentManage => "payment:manage".to_string(),
            Permission::SystemAdmin => "system:admin".to_string(),
            Permission::SystemMonitor => "system:monitor".to_string(),
            Permission::SystemConfig => "system:config".to_string(),
            Permission::Custom(perm) => perm.clone(),
        }
    }
}

/// Role system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    SuperAdmin,
    Admin,
    AssetManager,
    PaymentProcessor,
    User,
    ReadOnly,
    Custom(String),
}

impl Role {
    pub fn as_string(&self) -> String {
        match self {
            Role::SuperAdmin => "super_admin".to_string(),
            Role::Admin => "admin".to_string(),
            Role::AssetManager => "asset_manager".to_string(),
            Role::PaymentProcessor => "payment_processor".to_string(),
            Role::User => "user".to_string(),
            Role::ReadOnly => "read_only".to_string(),
            Role::Custom(role) => role.clone(),
        }
    }

    pub fn permissions(&self) -> Vec<Permission> {
        match self {
            Role::SuperAdmin => vec![
                Permission::SystemAdmin,
                Permission::SystemMonitor,
                Permission::SystemConfig,
                Permission::AssetManage,
                Permission::UserManage,
                Permission::PaymentManage,
            ],
            Role::Admin => vec![
                Permission::AssetManage,
                Permission::UserManage,
                Permission::PaymentRead,
                Permission::SystemMonitor,
            ],
            Role::AssetManager => vec![
                Permission::AssetRead,
                Permission::AssetWrite,
                Permission::AssetDelete,
            ],
            Role::PaymentProcessor => vec![
                Permission::PaymentRead,
                Permission::PaymentWrite,
                Permission::PaymentProcess,
            ],
            Role::User => vec![
                Permission::AssetRead,
                Permission::UserRead,
                Permission::PaymentRead,
            ],
            Role::ReadOnly => vec![
                Permission::AssetRead,
                Permission::UserRead,
                Permission::PaymentRead,
            ],
            Role::Custom(_) => vec![], // Custom roles need explicit permission assignment
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_claims() {
        let claims = UserClaims::new(
            "user123".to_string(),
            "user@example.com".to_string(),
            vec!["admin".to_string()],
            vec!["asset:read".to_string()],
            24,
        );

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "user@example.com");
        assert!(claims.has_role("admin"));
        assert!(claims.has_permission("asset:read"));
        assert!(!claims.has_role("user"));
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_session() {
        let mut session = Session::new("user123".to_string(), 24);

        assert_eq!(session.user_id, "user123");
        assert!(session.is_valid());
        assert!(!session.is_expired());

        session.invalidate();
        assert!(!session.is_valid());
    }

    #[test]
    fn test_permissions() {
        assert_eq!(Permission::AssetRead.as_string(), "asset:read");
        assert_eq!(
            Permission::Custom("custom:perm".to_string()).as_string(),
            "custom:perm"
        );
    }

    #[test]
    fn test_roles() {
        let admin_role = Role::Admin;
        assert_eq!(admin_role.as_string(), "admin");

        let permissions = admin_role.permissions();
        assert!(permissions.len() > 0);
        assert!(permissions
            .iter()
            .any(|p| matches!(p, Permission::AssetManage)));
    }
}
