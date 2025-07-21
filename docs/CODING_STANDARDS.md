# Coding Standards for StableRWA Platform

This document outlines the coding standards and best practices for the StableRWA platform development.

## Table of Contents

- [General Principles](#general-principles)
- [Rust Coding Standards](#rust-coding-standards)
- [Project Structure](#project-structure)
- [Error Handling](#error-handling)
- [Testing Standards](#testing-standards)
- [Documentation Standards](#documentation-standards)
- [Security Guidelines](#security-guidelines)
- [Performance Guidelines](#performance-guidelines)
- [Code Review Guidelines](#code-review-guidelines)

## General Principles

### 1. Code Quality
- **Readability**: Code should be self-documenting and easy to understand
- **Maintainability**: Code should be easy to modify and extend
- **Reliability**: Code should handle errors gracefully and be robust
- **Performance**: Code should be efficient and scalable

### 2. SOLID Principles
- **Single Responsibility**: Each module/function should have one reason to change
- **Open/Closed**: Open for extension, closed for modification
- **Liskov Substitution**: Subtypes must be substitutable for their base types
- **Interface Segregation**: Clients shouldn't depend on interfaces they don't use
- **Dependency Inversion**: Depend on abstractions, not concretions

### 3. DRY (Don't Repeat Yourself)
- Extract common functionality into reusable components
- Use configuration for environment-specific values
- Create utility functions for repeated operations

## Rust Coding Standards

### 1. Formatting and Style

#### Use `rustfmt` for consistent formatting
```bash
cargo fmt
```

#### Line Length
- Maximum 100 characters per line
- Break long lines at logical points

#### Indentation
- Use 4 spaces (no tabs)
- Align continuation lines appropriately

```rust
// Good
let result = some_long_function_name(
    first_parameter,
    second_parameter,
    third_parameter,
);

// Bad
let result = some_long_function_name(first_parameter, second_parameter, third_parameter);
```

### 2. Naming Conventions

#### Functions and Variables
```rust
// Use snake_case
fn calculate_total_value() -> Decimal { }
let user_account = Account::new();
```

#### Types and Traits
```rust
// Use PascalCase
struct AssetManager;
trait PaymentProcessor;
enum TransactionStatus;
```

#### Constants
```rust
// Use SCREAMING_SNAKE_CASE
const MAX_RETRY_ATTEMPTS: u32 = 3;
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
```

#### Modules
```rust
// Use snake_case
mod asset_management;
mod payment_processing;
```

### 3. Code Organization

#### Module Structure
```rust
// lib.rs or main.rs
pub mod config;
pub mod error;
pub mod types;
pub mod handlers;
pub mod services;
pub mod repositories;
pub mod utils;

// Re-exports
pub use error::{Error, Result};
pub use types::*;
```

#### Import Organization
```rust
// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// External crate imports
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

// Internal imports
use crate::error::{Error, Result};
use crate::types::Asset;
```

### 4. Function Design

#### Function Length
- Keep functions under 50 lines when possible
- Extract complex logic into separate functions
- Use descriptive function names

#### Parameters
- Limit function parameters (max 5-7)
- Use structs for complex parameter sets
- Consider builder pattern for optional parameters

```rust
// Good
#[derive(Debug)]
pub struct CreateAssetRequest {
    pub name: String,
    pub asset_type: AssetType,
    pub owner_id: String,
    pub metadata: AssetMetadata,
}

pub async fn create_asset(request: CreateAssetRequest) -> Result<Asset> {
    // Implementation
}

// Bad
pub async fn create_asset(
    name: String,
    asset_type: AssetType,
    owner_id: String,
    description: String,
    location: String,
    value: Decimal,
    currency: String,
) -> Result<Asset> {
    // Too many parameters
}
```

### 5. Type Definitions

#### Struct Design
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Use private fields with getters when needed
    #[serde(skip)]
    internal_state: InternalState,
}

impl Asset {
    pub fn new(name: String, asset_type: AssetType, owner_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            asset_type,
            owner_id,
            created_at: now,
            updated_at: now,
            internal_state: InternalState::default(),
        }
    }
    
    pub fn internal_state(&self) -> &InternalState {
        &self.internal_state
    }
}
```

#### Enum Design
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    RealEstate,
    Art,
    Commodities,
    IntellectualProperty,
    Equipment,
}

impl AssetType {
    pub fn display_name(&self) -> &'static str {
        match self {
            AssetType::RealEstate => "Real Estate",
            AssetType::Art => "Art & Collectibles",
            AssetType::Commodities => "Commodities",
            AssetType::IntellectualProperty => "Intellectual Property",
            AssetType::Equipment => "Equipment & Machinery",
        }
    }
}
```

## Project Structure

### 1. Workspace Organization
```
rwa-platform/
├── Cargo.toml              # Workspace manifest
├── README.md
├── LICENSE
├── .gitignore
├── docker-compose.yml
├── 
├── core-utils/             # Shared utilities
├── core-security/          # Security components
├── core-blockchain/        # Blockchain integration
├── core-compliance/        # Compliance framework
├── core-asset-lifecycle/   # Asset management
├── core-risk-management/   # Risk assessment
├── core-bridge/           # Cross-chain bridge
├── core-did/              # Decentralized identity
├── core-analytics/        # Data analytics
├── core-trading/          # Trading engine
├── 
├── service-gateway/       # API gateway
├── service-user/          # User management
├── service-asset/         # Asset service
├── service-payment/       # Payment processing
├── 
├── testing/               # Test utilities
├── docs/                  # Documentation
├── scripts/               # Build/deployment scripts
├── k8s/                   # Kubernetes manifests
└── infrastructure/        # Infrastructure as code
```

### 2. Service Structure
```
service-example/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs             # Library root
│   ├── main.rs            # Binary entry point
│   ├── config.rs          # Configuration
│   ├── error.rs           # Error types
│   ├── types.rs           # Domain types
│   ├── handlers/          # HTTP handlers
│   │   ├── mod.rs
│   │   ├── health.rs
│   │   └── api.rs
│   ├── services/          # Business logic
│   │   ├── mod.rs
│   │   └── example_service.rs
│   ├── repositories/      # Data access
│   │   ├── mod.rs
│   │   └── example_repo.rs
│   └── utils/             # Utilities
│       ├── mod.rs
│       └── helpers.rs
├── tests/                 # Integration tests
├── benches/              # Benchmarks
└── examples/             # Usage examples
```

## Error Handling

### 1. Error Types
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {id}")]
    NotFound { id: String },
    
    #[error("Invalid asset data: {field} - {message}")]
    InvalidData { field: String, message: String },
    
    #[error("Permission denied: {action} on asset {id}")]
    PermissionDenied { action: String, id: String },
    
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type AssetResult<T> = Result<T, AssetError>;
```

### 2. Error Handling Patterns
```rust
// Use ? operator for error propagation
pub async fn get_asset(id: &str) -> AssetResult<Asset> {
    let asset = repository.find_by_id(id).await?;
    validate_asset(&asset)?;
    Ok(asset)
}

// Handle specific errors when needed
pub async fn update_asset(id: &str, data: UpdateAssetData) -> AssetResult<Asset> {
    match repository.find_by_id(id).await {
        Ok(asset) => {
            let updated = apply_updates(asset, data)?;
            repository.save(&updated).await?;
            Ok(updated)
        }
        Err(AssetError::NotFound { .. }) => {
            Err(AssetError::NotFound { id: id.to_string() })
        }
        Err(e) => Err(e),
    }
}
```

## Testing Standards

### 1. Test Organization
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    
    #[tokio::test]
    async fn test_create_asset_success() {
        // Arrange
        let request = CreateAssetRequest {
            name: "Test Asset".to_string(),
            asset_type: AssetType::RealEstate,
            owner_id: "user123".to_string(),
            metadata: AssetMetadata::default(),
        };
        
        // Act
        let result = create_asset(request).await;
        
        // Assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.name, "Test Asset");
        assert_eq!(asset.asset_type, AssetType::RealEstate);
    }
    
    #[tokio::test]
    async fn test_create_asset_invalid_name() {
        // Test error cases
        let request = CreateAssetRequest {
            name: "".to_string(), // Invalid empty name
            asset_type: AssetType::RealEstate,
            owner_id: "user123".to_string(),
            metadata: AssetMetadata::default(),
        };
        
        let result = create_asset(request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AssetError::InvalidData { .. }));
    }
}
```

### 2. Test Utilities
```rust
// test_utils.rs
pub fn create_test_asset() -> Asset {
    Asset::new(
        "Test Asset".to_string(),
        AssetType::RealEstate,
        "test_user".to_string(),
    )
}

pub async fn setup_test_db() -> TestDatabase {
    // Setup test database
}

pub fn assert_asset_equals(actual: &Asset, expected: &Asset) {
    assert_eq!(actual.id, expected.id);
    assert_eq!(actual.name, expected.name);
    // ... other assertions
}
```

## Documentation Standards

### 1. Doc Comments
```rust
/// Calculates the total value of an asset portfolio.
///
/// This function aggregates the current market values of all assets
/// in the portfolio, converting to the specified currency if needed.
///
/// # Arguments
///
/// * `assets` - A slice of assets to include in the calculation
/// * `currency` - The target currency for the total value
/// * `exchange_rates` - Current exchange rates for currency conversion
///
/// # Returns
///
/// Returns `Ok(Decimal)` with the total portfolio value, or an error
/// if the calculation fails due to missing exchange rates or invalid data.
///
/// # Errors
///
/// This function will return an error if:
/// - Any asset has an invalid or missing valuation
/// - Exchange rates are missing for required currency conversions
/// - Arithmetic overflow occurs during calculation
///
/// # Examples
///
/// ```
/// use stablerwa::{Asset, calculate_portfolio_value};
/// use rust_decimal::Decimal;
/// use std::collections::HashMap;
///
/// let assets = vec![/* ... */];
/// let mut rates = HashMap::new();
/// rates.insert("USD".to_string(), Decimal::ONE);
/// 
/// let total = calculate_portfolio_value(&assets, "USD", &rates)?;
/// println!("Portfolio value: ${}", total);
/// ```
pub fn calculate_portfolio_value(
    assets: &[Asset],
    currency: &str,
    exchange_rates: &HashMap<String, Decimal>,
) -> Result<Decimal, CalculationError> {
    // Implementation
}
```

### 2. Module Documentation
```rust
//! # Asset Management Module
//!
//! This module provides functionality for managing real-world assets
//! throughout their lifecycle, from registration to tokenization.
//!
//! ## Key Features
//!
//! - Asset registration and metadata management
//! - Valuation tracking and history
//! - Document storage and verification
//! - Compliance monitoring
//!
//! ## Usage
//!
//! ```rust
//! use stablerwa::asset::{AssetManager, CreateAssetRequest};
//!
//! let manager = AssetManager::new(config).await?;
//! let asset = manager.create_asset(request).await?;
//! ```

pub mod asset_manager;
pub mod valuation;
pub mod documents;
```

## Security Guidelines

### 1. Input Validation
```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Validate, Deserialize)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
    
    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,
}

fn validate_phone(phone: &str) -> Result<(), ValidationError> {
    // Custom phone validation logic
    Ok(())
}
```

### 2. Sensitive Data Handling
```rust
use secrecy::{Secret, ExposeSecret};

#[derive(Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub database: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database
        )
    }
}
```

## Performance Guidelines

### 1. Async Best Practices
```rust
// Use async/await for I/O operations
pub async fn fetch_asset_data(id: &str) -> Result<AssetData> {
    let asset = database.get_asset(id).await?;
    let valuation = valuation_service.get_current_value(&asset).await?;
    let documents = document_service.list_documents(&asset.id).await?;
    
    Ok(AssetData {
        asset,
        valuation,
        documents,
    })
}

// Use join! for concurrent operations
use tokio::try_join;

pub async fn fetch_asset_summary(id: &str) -> Result<AssetSummary> {
    let (asset, valuation, documents) = try_join!(
        database.get_asset(id),
        valuation_service.get_current_value(id),
        document_service.count_documents(id)
    )?;
    
    Ok(AssetSummary {
        asset,
        current_value: valuation,
        document_count: documents,
    })
}
```

### 2. Memory Management
```rust
// Use Arc for shared ownership
use std::sync::Arc;

#[derive(Clone)]
pub struct AssetService {
    repository: Arc<dyn AssetRepository>,
    validator: Arc<dyn AssetValidator>,
    cache: Arc<dyn Cache>,
}

// Use Cow for conditional cloning
use std::borrow::Cow;

pub fn format_asset_name(name: &str, prefix: Option<&str>) -> Cow<str> {
    match prefix {
        Some(p) => Cow::Owned(format!("{}: {}", p, name)),
        None => Cow::Borrowed(name),
    }
}
```

## Code Review Guidelines

### 1. Review Checklist

#### Functionality
- [ ] Code solves the intended problem
- [ ] Edge cases are handled
- [ ] Error conditions are properly managed
- [ ] Business logic is correct

#### Code Quality
- [ ] Code is readable and well-structured
- [ ] Functions are appropriately sized
- [ ] Variable names are descriptive
- [ ] Comments explain complex logic

#### Testing
- [ ] Unit tests cover new functionality
- [ ] Integration tests verify interactions
- [ ] Error cases are tested
- [ ] Test data is realistic

#### Security
- [ ] Input validation is present
- [ ] Sensitive data is protected
- [ ] Authentication/authorization is correct
- [ ] No security vulnerabilities introduced

#### Performance
- [ ] No obvious performance issues
- [ ] Database queries are optimized
- [ ] Memory usage is reasonable
- [ ] Async operations are used appropriately

### 2. Review Comments

#### Constructive Feedback
```
// Good
"Consider using a more descriptive variable name here. 
`user_account` would be clearer than `ua`."

"This function is getting quite long. Could we extract 
the validation logic into a separate function?"

// Bad
"This is wrong."
"Bad code."
```

#### Suggestions
```
// Provide specific suggestions
"Instead of unwrap(), consider using match or ? operator 
to handle the error case gracefully."

"We could use the builder pattern here to make the API 
more ergonomic. See the example in the user service."
```

Remember: Code reviews are about improving code quality and sharing knowledge, not criticizing the author.
