// =====================================================================================
// File: auth-service/src/lib.rs
// Description: Authentication microservice for StableRWA. Handles user authentication,
//              JWT, and session management for StableRWA platform.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use log::{info, error};

/// Logs the import of this module with a timestamp.
pub fn log_import() {
    info!("[{}] auth-service module imported", Utc::now());
}

/// Custom error type for authentication operations.
#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials(String),
    TokenExpired(String),
}

/// Example function: Authenticate a user.
/// Returns Ok(token) on success, or AuthError on failure.
pub fn authenticate_user(username: &str, password: &str) -> Result<String, AuthError> {
    log_import();
    if username == "admin" && password == "password" {
        info!("[{}] User '{}' authenticated", Utc::now(), username);
        Ok("token123".to_string())
    } else {
        error!("[{}] Invalid credentials for user '{}'", Utc::now(), username);
        Err(AuthError::InvalidCredentials(username.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_authenticate_user_success() {
        let result = authenticate_user("admin", "password");
        assert!(result.is_ok());
    }
    #[test]
    fn test_authenticate_user_failure() {
        let result = authenticate_user("user", "wrong");
        assert!(matches!(result, Err(AuthError::InvalidCredentials(_))));
    }
}
