// =====================================================================================
// File: core-security/src/auth.rs
// Description: Authentication and authorization implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{Permission, Role, SecurityError, Session, UserClaims};
use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;
use uuid::Uuid;

/// Authentication service trait
#[async_trait]
pub trait AuthenticationService: Send + Sync {
    /// Authenticate user with email and password
    async fn authenticate(&self, email: &str, password: &str) -> Result<UserClaims, SecurityError>;

    /// Validate user session
    async fn validate_session(&self, session_id: &str) -> Result<Session, SecurityError>;

    /// Create new session for user
    async fn create_session(
        &self,
        user_id: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session, SecurityError>;

    /// Invalidate session
    async fn invalidate_session(&self, session_id: &str) -> Result<(), SecurityError>;

    /// Refresh session
    async fn refresh_session(&self, session_id: &str) -> Result<Session, SecurityError>;
}

/// Authorization service trait
#[async_trait]
pub trait AuthorizationService: Send + Sync {
    /// Check if user has specific permission
    async fn has_permission(
        &self,
        user_id: &str,
        permission: &Permission,
    ) -> Result<bool, SecurityError>;

    /// Check if user has specific role
    async fn has_role(&self, user_id: &str, role: &Role) -> Result<bool, SecurityError>;

    /// Get user permissions
    async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<Permission>, SecurityError>;

    /// Get user roles
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>, SecurityError>;

    /// Assign role to user
    async fn assign_role(&self, user_id: &str, role: &Role) -> Result<(), SecurityError>;

    /// Remove role from user
    async fn remove_role(&self, user_id: &str, role: &Role) -> Result<(), SecurityError>;
}

/// User authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuth {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub failed_login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Password policy configuration
#[derive(Debug, Clone)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_age_days: Option<u32>,
    pub prevent_reuse_count: Option<u32>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_age_days: Some(90),
            prevent_reuse_count: Some(5),
        }
    }
}

/// Password utilities
pub struct PasswordUtils;

impl PasswordUtils {
    /// Hash password using bcrypt
    pub fn hash_password(password: &str) -> Result<String, SecurityError> {
        hash(password, DEFAULT_COST)
            .map_err(|e| SecurityError::EncryptionError(format!("Failed to hash password: {}", e)))
    }

    /// Verify password against hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, SecurityError> {
        verify(password, hash).map_err(|e| {
            SecurityError::EncryptionError(format!("Failed to verify password: {}", e))
        })
    }

    /// Validate password against policy
    pub fn validate_password(password: &str, policy: &PasswordPolicy) -> Result<(), SecurityError> {
        if password.len() < policy.min_length {
            return Err(SecurityError::ValidationError(format!(
                "Password must be at least {} characters long",
                policy.min_length
            )));
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(SecurityError::ValidationError(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(SecurityError::ValidationError(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if policy.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(SecurityError::ValidationError(
                "Password must contain at least one number".to_string(),
            ));
        }

        if policy.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(SecurityError::ValidationError(
                "Password must contain at least one special character".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate secure random password that meets default policy requirements
    pub fn generate_password(length: usize) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Ensure minimum length
        let actual_length = length.max(8);

        // Character sets
        const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        const NUMBERS: &[u8] = b"0123456789";
        const SPECIAL: &[u8] = b"!@#$%^&*";
        const ALL_CHARS: &[u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";

        let mut password = Vec::with_capacity(actual_length);

        // Ensure at least one character from each required set
        password.push(UPPERCASE[rng.gen_range(0..UPPERCASE.len())] as char);
        password.push(LOWERCASE[rng.gen_range(0..LOWERCASE.len())] as char);
        password.push(NUMBERS[rng.gen_range(0..NUMBERS.len())] as char);
        password.push(SPECIAL[rng.gen_range(0..SPECIAL.len())] as char);

        // Fill the rest with random characters
        for _ in 4..actual_length {
            let idx = rng.gen_range(0..ALL_CHARS.len());
            password.push(ALL_CHARS[idx] as char);
        }

        // Shuffle the password to avoid predictable patterns
        use rand::seq::SliceRandom;
        password.shuffle(&mut rng);

        password.into_iter().collect()
    }
}

/// Account lockout manager
pub struct AccountLockoutManager {
    max_attempts: u32,
    lockout_duration_minutes: u32,
}

impl AccountLockoutManager {
    pub fn new(max_attempts: u32, lockout_duration_minutes: u32) -> Self {
        Self {
            max_attempts,
            lockout_duration_minutes,
        }
    }

    /// Check if account is locked
    pub fn is_locked(&self, user_auth: &UserAuth) -> bool {
        if let Some(locked_until) = user_auth.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    /// Record failed login attempt
    pub fn record_failed_attempt(&self, user_auth: &mut UserAuth) {
        user_auth.failed_login_attempts += 1;

        if user_auth.failed_login_attempts >= self.max_attempts {
            user_auth.locked_until =
                Some(Utc::now() + chrono::Duration::minutes(self.lockout_duration_minutes as i64));
            warn!(
                "Account locked for user: {} due to {} failed attempts",
                user_auth.email, user_auth.failed_login_attempts
            );
        }
    }

    /// Reset failed attempts on successful login
    pub fn reset_failed_attempts(&self, user_auth: &mut UserAuth) {
        user_auth.failed_login_attempts = 0;
        user_auth.locked_until = None;
        user_auth.last_login = Some(Utc::now());
    }
}

/// Multi-factor authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    pub enabled: bool,
    pub methods: Vec<MfaMethod>,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod {
    Totp { secret: String },
    Sms { phone_number: String },
    Email { email: String },
}

/// Audit log entry for authentication events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAuditLog {
    pub id: String,
    pub user_id: Option<String>,
    pub event_type: AuthEventType,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEventType {
    Login,
    Logout,
    PasswordChange,
    PasswordReset,
    AccountLocked,
    AccountUnlocked,
    MfaEnabled,
    MfaDisabled,
    RoleAssigned,
    RoleRemoved,
    PermissionGranted,
    PermissionRevoked,
}

impl AuthAuditLog {
    pub fn new(
        event_type: AuthEventType,
        user_id: Option<String>,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            event_type,
            ip_address,
            user_agent,
            success,
            error_message: None,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.error_message = Some(error.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123!";
        let hash = PasswordUtils::hash_password(password).unwrap();

        assert!(PasswordUtils::verify_password(password, &hash).unwrap());
        assert!(!PasswordUtils::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        let policy = PasswordPolicy::default();

        // Valid password
        assert!(PasswordUtils::validate_password("ValidPass123!", &policy).is_ok());

        // Too short
        assert!(PasswordUtils::validate_password("Short1!", &policy).is_err());

        // No uppercase
        assert!(PasswordUtils::validate_password("lowercase123!", &policy).is_err());

        // No numbers
        assert!(PasswordUtils::validate_password("NoNumbers!", &policy).is_err());
    }

    #[test]
    fn test_account_lockout() {
        let lockout_manager = AccountLockoutManager::new(3, 30);
        let mut user_auth = UserAuth {
            user_id: "test_user".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            is_active: true,
            is_verified: true,
            failed_login_attempts: 0,
            locked_until: None,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Not locked initially
        assert!(!lockout_manager.is_locked(&user_auth));

        // Record failed attempts
        lockout_manager.record_failed_attempt(&mut user_auth);
        lockout_manager.record_failed_attempt(&mut user_auth);
        assert!(!lockout_manager.is_locked(&user_auth));

        lockout_manager.record_failed_attempt(&mut user_auth);
        assert!(lockout_manager.is_locked(&user_auth));
    }

    #[test]
    fn test_password_generation() {
        let password = PasswordUtils::generate_password(12);
        assert_eq!(password.len(), 12);

        let policy = PasswordPolicy::default();
        assert!(PasswordUtils::validate_password(&password, &policy).is_ok());
    }
}
