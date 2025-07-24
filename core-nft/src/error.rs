// =====================================================================================
// File: core-nft/src/error.rs
// Description: Error types for NFT operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use thiserror::Error;
use std::fmt;

/// Result type for NFT operations
pub type NFTResult<T> = Result<T, NFTError>;

/// Comprehensive error types for NFT operations
#[derive(Error, Debug, Clone)]
pub enum NFTError {
    // Validation errors
    #[error("Invalid token ID: {0}")]
    InvalidTokenId(String),
    
    #[error("Invalid contract address: {0}")]
    InvalidContractAddress(String),
    
    #[error("Invalid metadata: {reason}")]
    InvalidMetadata { reason: String },
    
    #[error("Invalid image format: expected {expected}, got {actual}")]
    InvalidImageFormat { expected: String, actual: String },
    
    #[error("File too large: {size} bytes, maximum allowed: {max_size} bytes")]
    FileTooLarge { size: u64, max_size: u64 },
    
    #[error("Invalid royalty percentage: {percentage}%, must be between 0% and 100%")]
    InvalidRoyaltyPercentage { percentage: f64 },
    
    // Ownership and permission errors
    #[error("Not authorized: {action} requires {required_role}")]
    NotAuthorized { action: String, required_role: String },
    
    #[error("Token not owned by {owner}")]
    NotOwned { owner: String },
    
    #[error("Token not approved for transfer")]
    NotApproved,
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    
    // Token state errors
    #[error("Token {token_id} not found")]
    TokenNotFound { token_id: String },
    
    #[error("Token {token_id} already exists")]
    TokenAlreadyExists { token_id: String },
    
    #[error("Token {token_id} is not transferable")]
    TokenNotTransferable { token_id: String },
    
    #[error("Token {token_id} is burned")]
    TokenBurned { token_id: String },
    
    // Collection errors
    #[error("Collection {collection_id} not found")]
    CollectionNotFound { collection_id: String },
    
    #[error("Collection {collection_id} is not verified")]
    CollectionNotVerified { collection_id: String },
    
    #[error("Collection limit exceeded: {current}/{max}")]
    CollectionLimitExceeded { current: u64, max: u64 },
    
    // Marketplace errors
    #[error("Listing {listing_id} not found")]
    ListingNotFound { listing_id: String },
    
    #[error("Listing {listing_id} has expired")]
    ListingExpired { listing_id: String },
    
    #[error("Offer {offer_id} not found")]
    OfferNotFound { offer_id: String },
    
    #[error("Insufficient funds for purchase: required {required}, available {available}")]
    InsufficientFunds { required: String, available: String },
    
    #[error("Price too low: minimum {minimum}, offered {offered}")]
    PriceTooLow { minimum: String, offered: String },
    
    // Storage errors
    #[error("IPFS error: {message}")]
    IPFSError { message: String },
    
    #[error("Storage provider {provider} unavailable")]
    StorageUnavailable { provider: String },
    
    #[error("Failed to upload to storage: {reason}")]
    UploadFailed { reason: String },
    
    #[error("Failed to retrieve from storage: {hash}")]
    RetrievalFailed { hash: String },
    
    // Blockchain errors
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
    
    #[error("Gas estimation failed: {reason}")]
    GasEstimationFailed { reason: String },
    
    #[error("Contract call failed: {method} on {contract}")]
    ContractCallFailed { method: String, contract: String },
    
    #[error("Network error: {network} is unavailable")]
    NetworkUnavailable { network: String },
    
    // Metadata errors
    #[error("Metadata validation failed: {field} is {issue}")]
    MetadataValidationFailed { field: String, issue: String },
    
    #[error("Metadata standard {standard} not supported")]
    UnsupportedMetadataStandard { standard: String },
    
    #[error("Failed to parse metadata: {reason}")]
    MetadataParsingFailed { reason: String },
    
    // Minting errors
    #[error("Minting not allowed: {reason}")]
    MintingNotAllowed { reason: String },
    
    #[error("Mint limit exceeded: {current}/{max}")]
    MintLimitExceeded { current: u64, max: u64 },
    
    #[error("Minting phase {phase} not active")]
    MintingPhaseNotActive { phase: String },
    
    // Trading errors
    #[error("Trading not allowed for token {token_id}: {reason}")]
    TradingNotAllowed { token_id: String, reason: String },
    
    #[error("Order {order_id} not found")]
    OrderNotFound { order_id: String },
    
    #[error("Order {order_id} has been filled")]
    OrderAlreadyFilled { order_id: String },
    
    // Rate limiting errors
    #[error("Rate limit exceeded: {action} limited to {limit} per {window}")]
    RateLimitExceeded { action: String, limit: u64, window: String },
    
    // Configuration errors
    #[error("Configuration error: {parameter} is invalid")]
    ConfigurationError { parameter: String },
    
    #[error("Feature {feature} is not enabled")]
    FeatureNotEnabled { feature: String },
    
    // External service errors
    #[error("External service error: {service} returned {status}")]
    ExternalServiceError { service: String, status: String },
    
    #[error("API key invalid for service {service}")]
    InvalidAPIKey { service: String },
    
    // Database errors
    #[error("Database error: {operation} failed")]
    DatabaseError { operation: String },
    
    #[error("Connection pool exhausted")]
    ConnectionPoolExhausted,
    
    // Serialization errors
    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },
    
    #[error("Deserialization error: {reason}")]
    DeserializationError { reason: String },
    
    // Generic errors
    #[error("Internal error: {message}")]
    Internal { message: String },
    
    #[error("Operation timeout: {operation} took longer than {timeout_ms}ms")]
    Timeout { operation: String, timeout_ms: u64 },
    
    #[error("Resource not available: {resource}")]
    ResourceUnavailable { resource: String },
}

impl NFTError {
    /// Create a new validation error
    pub fn validation(reason: &str) -> Self {
        Self::InvalidMetadata {
            reason: reason.to_string(),
        }
    }
    
    /// Create a new authorization error
    pub fn unauthorized(action: &str, required_role: &str) -> Self {
        Self::NotAuthorized {
            action: action.to_string(),
            required_role: required_role.to_string(),
        }
    }
    
    /// Create a new not found error
    pub fn not_found(resource: &str, id: &str) -> Self {
        match resource {
            "token" => Self::TokenNotFound {
                token_id: id.to_string(),
            },
            "collection" => Self::CollectionNotFound {
                collection_id: id.to_string(),
            },
            "listing" => Self::ListingNotFound {
                listing_id: id.to_string(),
            },
            "offer" => Self::OfferNotFound {
                offer_id: id.to_string(),
            },
            "order" => Self::OrderNotFound {
                order_id: id.to_string(),
            },
            _ => Self::ResourceUnavailable {
                resource: format!("{}: {}", resource, id),
            },
        }
    }
    
    /// Create a new internal error
    pub fn internal(message: &str) -> Self {
        Self::Internal {
            message: message.to_string(),
        }
    }
    
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkUnavailable { .. }
                | Self::StorageUnavailable { .. }
                | Self::ExternalServiceError { .. }
                | Self::ConnectionPoolExhausted
                | Self::Timeout { .. }
        )
    }
    
    /// Get error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::InvalidTokenId(_)
            | Self::InvalidContractAddress(_)
            | Self::InvalidMetadata { .. }
            | Self::InvalidImageFormat { .. }
            | Self::FileTooLarge { .. }
            | Self::InvalidRoyaltyPercentage { .. } => "validation",
            
            Self::NotAuthorized { .. }
            | Self::NotOwned { .. }
            | Self::NotApproved => "authorization",
            
            Self::TokenNotFound { .. }
            | Self::TokenAlreadyExists { .. }
            | Self::CollectionNotFound { .. } => "not_found",
            
            Self::IPFSError { .. }
            | Self::StorageUnavailable { .. }
            | Self::UploadFailed { .. }
            | Self::RetrievalFailed { .. } => "storage",
            
            Self::TransactionFailed { .. }
            | Self::GasEstimationFailed { .. }
            | Self::ContractCallFailed { .. }
            | Self::NetworkUnavailable { .. } => "blockchain",
            
            Self::RateLimitExceeded { .. } => "rate_limit",
            
            Self::DatabaseError { .. }
            | Self::ConnectionPoolExhausted => "database",
            
            Self::Timeout { .. } => "timeout",
            
            _ => "other",
        }
    }
}

// Implement conversion from common error types
impl From<serde_json::Error> for NFTError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            reason: err.to_string(),
        }
    }
}

impl From<sqlx::Error> for NFTError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError {
            operation: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for NFTError {
    fn from(err: reqwest::Error) -> Self {
        Self::ExternalServiceError {
            service: "HTTP".to_string(),
            status: err.to_string(),
        }
    }
}

impl From<std::io::Error> for NFTError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal {
            message: format!("IO error: {}", err),
        }
    }
}
