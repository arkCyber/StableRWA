// =====================================================================================
// File: user-service/src/lib.rs
// Description: User management microservice for StableRWA. Handles user registration,
//              profile, and management for StableRWA platform.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use log::{info, error};
use polkadot_integration::{self, connect_to_polkadot_rpc, PolkadotError};

/// Logs the import of this module with a timestamp.
pub fn log_import() {
    info!("[{}] user-service module imported", Utc::now());
}

/// Custom error type for user management operations.
#[derive(Debug)]
pub enum UserError {
    UserNotFound(String),
    InvalidUserData(String),
    Blockchain(PolkadotError),
}

/// Example function: Register a new user and connect to Polkadot RPC.
/// Returns Ok(user_id) on success, or UserError on failure.
pub fn register_user(username: &str, polkadot_rpc_url: &str) -> Result<String, UserError> {
    log_import();
    if username.is_empty() {
        error!("[{}] Username is empty", Utc::now());
        return Err(UserError::InvalidUserData("Username is empty".to_string()));
    }
    // Example: Connect to Polkadot RPC before registering user
    connect_to_polkadot_rpc(polkadot_rpc_url).map_err(UserError::Blockchain)?;
    info!("[{}] Registering user: {}", Utc::now(), username);
    Ok(format!("user_{}", username))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_register_user_success() {
        let result = register_user("alice", "https://rpc.polkadot.io");
        assert!(result.is_ok());
    }
    #[test]
    fn test_register_user_empty() {
        let result = register_user("", "https://rpc.polkadot.io");
        assert!(matches!(result, Err(UserError::InvalidUserData(_))));
    }
    #[test]
    fn test_register_user_invalid_rpc() {
        let result = register_user("alice", "");
        assert!(matches!(result, Err(UserError::Blockchain(PolkadotError::RpcError(_)))));
    }
}
