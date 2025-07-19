// =====================================================================================
// File: service-user/src/lib.rs
// Description: User Service library for RWA Platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

pub mod handlers;
pub mod models;
pub mod service;

use core_config::AppConfig;
use core_database::DatabaseManager;
use core_observability::BusinessMetrics;
use service::UserService;
use std::sync::Arc;
use thiserror::Error;

/// User service specific errors
#[derive(Error, Debug)]
pub enum UserError {
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Email already exists: {0}")]
    EmailAlreadyExists(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account not activated")]
    AccountNotActivated,
    #[error("Account suspended")]
    AccountSuspended,
    #[error("Invalid email format: {0}")]
    InvalidEmailFormat(String),
    #[error("Password too weak: {0}")]
    WeakPassword(String),
    #[error("Email verification required")]
    EmailVerificationRequired,
    #[error("Invalid verification token")]
    InvalidVerificationToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Too many login attempts")]
    TooManyLoginAttempts,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl From<sqlx::Error> for UserError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => UserError::UserNotFound("User not found".to_string()),
            _ => UserError::DatabaseError(err.to_string()),
        }
    }
}

impl From<core_security::SecurityError> for UserError {
    fn from(err: core_security::SecurityError) -> Self {
        match err {
            core_security::SecurityError::TokenExpired => UserError::TokenExpired,
            core_security::SecurityError::InvalidToken(msg) => UserError::InvalidVerificationToken,
            core_security::SecurityError::AuthenticationFailed(msg) => UserError::InvalidCredentials,
            core_security::SecurityError::RateLimitExceeded => UserError::RateLimitExceeded,
            _ => UserError::ValidationError(err.to_string()),
        }
    }
}

/// Application state for the user service
pub struct AppState {
    pub config: AppConfig,
    pub user_service: Arc<UserService>,
    pub metrics: Arc<BusinessMetrics>,
    pub database: DatabaseManager,
}

/// User service result type
pub type UserResult<T> = Result<T, UserError>;

/// Password validation rules
pub struct PasswordPolicy {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_length: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_length: 128,
        }
    }
}

impl PasswordPolicy {
    /// Validate password against policy
    pub fn validate(&self, password: &str) -> UserResult<()> {
        if password.len() < self.min_length {
            return Err(UserError::WeakPassword(
                format!("Password must be at least {} characters long", self.min_length)
            ));
        }

        if password.len() > self.max_length {
            return Err(UserError::WeakPassword(
                format!("Password must be no more than {} characters long", self.max_length)
            ));
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(UserError::WeakPassword(
                "Password must contain at least one uppercase letter".to_string()
            ));
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(UserError::WeakPassword(
                "Password must contain at least one lowercase letter".to_string()
            ));
        }

        if self.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            return Err(UserError::WeakPassword(
                "Password must contain at least one number".to_string()
            ));
        }

        if self.require_special_chars && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(UserError::WeakPassword(
                "Password must contain at least one special character".to_string()
            ));
        }

        Ok(())
    }
}

/// Email validation
pub fn validate_email(email: &str) -> UserResult<()> {
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|e| UserError::ValidationError(format!("Regex error: {}", e)))?;

    if !email_regex.is_match(email) {
        return Err(UserError::InvalidEmailFormat(email.to_string()));
    }

    if email.len() > 254 {
        return Err(UserError::InvalidEmailFormat(
            "Email address too long".to_string()
        ));
    }

    Ok(())
}

/// Phone number validation
pub fn validate_phone(phone: &str) -> UserResult<()> {
    let phone_regex = regex::Regex::new(r"^\+?[1-9]\d{1,14}$")
        .map_err(|e| UserError::ValidationError(format!("Regex error: {}", e)))?;

    if !phone_regex.is_match(phone) {
        return Err(UserError::ValidationError(
            "Invalid phone number format".to_string()
        ));
    }

    Ok(())
}

/// Name validation
pub fn validate_name(name: &str, field_name: &str) -> UserResult<()> {
    if name.trim().is_empty() {
        return Err(UserError::ValidationError(
            format!("{} cannot be empty", field_name)
        ));
    }

    if name.len() > 100 {
        return Err(UserError::ValidationError(
            format!("{} is too long (max 100 characters)", field_name)
        ));
    }

    // Check for valid characters (letters, spaces, hyphens, apostrophes)
    let name_regex = regex::Regex::new(r"^[a-zA-Z\s\-']+$")
        .map_err(|e| UserError::ValidationError(format!("Regex error: {}", e)))?;

    if !name_regex.is_match(name) {
        return Err(UserError::ValidationError(
            format!("{} contains invalid characters", field_name)
        ));
    }

    Ok(())
}

/// User role definitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    User,
    Moderator,
    Viewer,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::User => "user",
            UserRole::Moderator => "moderator",
            UserRole::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "user" => Some(UserRole::User),
            "moderator" => Some(UserRole::Moderator),
            "viewer" => Some(UserRole::Viewer),
            _ => None,
        }
    }

    pub fn permissions(&self) -> Vec<&'static str> {
        match self {
            UserRole::Admin => vec![
                "users:read", "users:create", "users:update", "users:delete",
                "assets:read", "assets:create", "assets:update", "assets:delete",
                "payments:read", "payments:create", "payments:update",
                "blockchain:read", "blockchain:write",
                "admin:access"
            ],
            UserRole::Moderator => vec![
                "users:read", "users:update",
                "assets:read", "assets:update",
                "payments:read",
                "blockchain:read"
            ],
            UserRole::User => vec![
                "users:read_own", "users:update_own",
                "assets:read", "assets:create_own", "assets:update_own",
                "payments:read_own", "payments:create",
                "blockchain:read"
            ],
            UserRole::Viewer => vec![
                "users:read_own",
                "assets:read",
                "payments:read_own",
                "blockchain:read"
            ],
        }
    }
}

/// User status definitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
    Deleted,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Suspended => "suspended",
            UserStatus::PendingVerification => "pending_verification",
            UserStatus::Deleted => "deleted",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "active" => Some(UserStatus::Active),
            "inactive" => Some(UserStatus::Inactive),
            "suspended" => Some(UserStatus::Suspended),
            "pending_verification" => Some(UserStatus::PendingVerification),
            "deleted" => Some(UserStatus::Deleted),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_policy_validation() {
        let policy = PasswordPolicy::default();
        
        // Valid password
        assert!(policy.validate("StrongPass123!").is_ok());
        
        // Too short
        assert!(policy.validate("Short1!").is_err());
        
        // No uppercase
        assert!(policy.validate("lowercase123!").is_err());
        
        // No lowercase
        assert!(policy.validate("UPPERCASE123!").is_err());
        
        // No numbers
        assert!(policy.validate("NoNumbers!").is_err());
        
        // No special characters
        assert!(policy.validate("NoSpecialChars123").is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk").is_ok());
        
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_phone_validation() {
        assert!(validate_phone("+1234567890").is_ok());
        assert!(validate_phone("1234567890").is_ok());
        
        assert!(validate_phone("123").is_err());
        assert!(validate_phone("abc123").is_err());
        assert!(validate_phone("+0123456789").is_err()); // starts with 0
    }

    #[test]
    fn test_user_role_permissions() {
        let admin = UserRole::Admin;
        let user = UserRole::User;
        
        assert!(admin.permissions().contains(&"admin:access"));
        assert!(!user.permissions().contains(&"admin:access"));
        assert!(user.permissions().contains(&"users:read_own"));
    }
}
