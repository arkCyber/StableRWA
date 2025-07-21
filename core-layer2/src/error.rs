// =====================================================================================
// File: core-layer2/src/error.rs
// Description: Error types for Layer 2 services
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Result type for Layer 2 operations
pub type Layer2Result<T> = Result<T, Layer2Error>;

/// Layer 2 service error types
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum Layer2Error {
    /// Validation errors
    #[error("Validation error in field '{field}': {message}")]
    ValidationError { field: String, message: String },

    /// Network errors
    #[error("Network error on {network}: {message}")]
    NetworkError { network: String, message: String },

    /// Bridge errors
    #[error("Bridge error: {message}")]
    BridgeError { message: String },

    /// Gas estimation errors
    #[error("Gas estimation failed: {message}")]
    GasEstimationError { message: String },

    /// Transaction errors
    #[error("Transaction failed on {network}: {reason}")]
    TransactionError { network: String, reason: String },

    /// Cross-chain errors
    #[error("Cross-chain operation failed: {message}")]
    CrossChainError { message: String },

    /// State sync errors
    #[error("State sync error: {message}")]
    StateSyncError { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Not found errors
    #[error("Resource not found: {resource_type} with id '{id}'")]
    NotFound { resource_type: String, id: String },

    /// Internal errors
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

impl Layer2Error {
    pub fn validation_error<S: Into<String>>(field: S, message: S) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn network_error<S: Into<String>>(network: S, message: S) -> Self {
        Self::NetworkError {
            network: network.into(),
            message: message.into(),
        }
    }

    pub fn bridge_error<S: Into<String>>(message: S) -> Self {
        Self::BridgeError {
            message: message.into(),
        }
    }

    pub fn not_found<S: Into<String>>(resource_type: S, id: S) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }
}
