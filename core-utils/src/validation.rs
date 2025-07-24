// =====================================================================================
// File: core-utils/src/validation.rs
// Description: Validation utilities for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

// use crate::UtilError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation result
pub type ValidationResult = Result<(), ValidationError>;

/// Validation error with field-specific messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationError {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, field: &str, message: &str) {
        self.errors
            .entry(field.to_string())
            .or_insert_with(Vec::new)
            .push(message.to_string());
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn merge(&mut self, other: ValidationError) {
        for (field, messages) in other.errors {
            for message in messages {
                self.add_error(&field, &message);
            }
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages: Vec<String> = self
            .errors
            .iter()
            .flat_map(|(field, errors)| {
                errors
                    .iter()
                    .map(move |error| format!("{}: {}", field, error))
            })
            .collect();
        write!(f, "{}", messages.join(", "))
    }
}

impl std::error::Error for ValidationError {}

/// Validator trait for custom validation logic
pub trait Validator<T> {
    fn validate(&self, value: &T) -> ValidationResult;
}

/// Common validation functions
pub struct Validate;

impl Validate {
    /// Validate that a string is not empty
    pub fn not_empty(value: &str, field: &str) -> ValidationResult {
        if value.trim().is_empty() {
            let mut error = ValidationError::new();
            error.add_error(field, "cannot be empty");
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Validate string length
    pub fn length(
        value: &str,
        field: &str,
        min: Option<usize>,
        max: Option<usize>,
    ) -> ValidationResult {
        let len = value.len();
        let mut error = ValidationError::new();

        if let Some(min_len) = min {
            if len < min_len {
                error.add_error(
                    field,
                    &format!("must be at least {} characters long", min_len),
                );
            }
        }

        if let Some(max_len) = max {
            if len > max_len {
                error.add_error(
                    field,
                    &format!("must be at most {} characters long", max_len),
                );
            }
        }

        if error.has_errors() {
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Validate email format
    pub fn email(value: &str, field: &str) -> ValidationResult {
        let email_regex =
            regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if !email_regex.is_match(value) {
            let mut error = ValidationError::new();
            error.add_error(field, "must be a valid email address");
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Validate URL format
    pub fn url(value: &str, field: &str) -> ValidationResult {
        match url::Url::parse(value) {
            Ok(_) => Ok(()),
            Err(_) => {
                let mut error = ValidationError::new();
                error.add_error(field, "must be a valid URL");
                Err(error)
            }
        }
    }

    /// Validate numeric range
    pub fn range<T>(value: T, field: &str, min: Option<T>, max: Option<T>) -> ValidationResult
    where
        T: PartialOrd + std::fmt::Display,
    {
        let mut error = ValidationError::new();

        if let Some(min_val) = min {
            if value < min_val {
                error.add_error(field, &format!("must be at least {}", min_val));
            }
        }

        if let Some(max_val) = max {
            if value > max_val {
                error.add_error(field, &format!("must be at most {}", max_val));
            }
        }

        if error.has_errors() {
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Validate that value is in a list of allowed values
    pub fn in_list_str(value: &str, field: &str, allowed: &[&str]) -> ValidationResult {
        if allowed.contains(&value) {
            Ok(())
        } else {
            let mut error = ValidationError::new();
            let allowed_str: Vec<String> = allowed.iter().map(|v| v.to_string()).collect();
            error.add_error(
                field,
                &format!("must be one of: {}", allowed_str.join(", ")),
            );
            Err(error)
        }
    }

    /// Validate UUID format
    pub fn uuid(value: &str, field: &str) -> ValidationResult {
        match uuid::Uuid::parse_str(value) {
            Ok(_) => Ok(()),
            Err(_) => {
                let mut error = ValidationError::new();
                error.add_error(field, "must be a valid UUID");
                Err(error)
            }
        }
    }

    /// Validate phone number format (basic)
    pub fn phone(value: &str, field: &str) -> ValidationResult {
        let phone_regex = regex::Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap();
        let cleaned = value.replace(&[' ', '-', '(', ')', '.'][..], "");

        if !phone_regex.is_match(&cleaned) {
            let mut error = ValidationError::new();
            error.add_error(field, "must be a valid phone number");
            Err(error)
        } else {
            Ok(())
        }
    }

    /// Validate that a value is positive
    pub fn positive<T>(value: T, field: &str) -> ValidationResult
    where
        T: PartialOrd + Default + std::fmt::Display,
    {
        if value > T::default() {
            Ok(())
        } else {
            let mut error = ValidationError::new();
            error.add_error(field, "must be positive");
            Err(error)
        }
    }

    /// Validate that a value is not negative
    pub fn non_negative<T>(value: T, field: &str) -> ValidationResult
    where
        T: PartialOrd + Default + std::fmt::Display,
    {
        if value >= T::default() {
            Ok(())
        } else {
            let mut error = ValidationError::new();
            error.add_error(field, "cannot be negative");
            Err(error)
        }
    }

    /// Validate password strength
    pub fn password_strength(value: &str, field: &str) -> ValidationResult {
        let mut error = ValidationError::new();

        if value.len() < 8 {
            error.add_error(field, "must be at least 8 characters long");
        }

        if !value.chars().any(|c| c.is_uppercase()) {
            error.add_error(field, "must contain at least one uppercase letter");
        }

        if !value.chars().any(|c| c.is_lowercase()) {
            error.add_error(field, "must contain at least one lowercase letter");
        }

        if !value.chars().any(|c| c.is_numeric()) {
            error.add_error(field, "must contain at least one number");
        }

        if !value.chars().any(|c| !c.is_alphanumeric()) {
            error.add_error(field, "must contain at least one special character");
        }

        if error.has_errors() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

/// Validation builder for chaining validations
pub struct ValidationBuilder {
    errors: ValidationError,
}

impl ValidationBuilder {
    pub fn new() -> Self {
        Self {
            errors: ValidationError::new(),
        }
    }

    /// Add a validation result to the builder
    pub fn add_result(mut self, result: ValidationResult) -> Self {
        if let Err(error) = result {
            self.errors.merge(error);
        }
        self
    }

    /// Add a custom validation
    pub fn validate<F>(mut self, validation: F) -> Self
    where
        F: FnOnce() -> ValidationResult,
    {
        if let Err(error) = validation() {
            self.errors.merge(error);
        }
        self
    }

    /// Build the final validation result
    pub fn build(self) -> ValidationResult {
        if self.errors.has_errors() {
            Err(self.errors)
        } else {
            Ok(())
        }
    }
}

/// Macro for easier validation building
#[macro_export]
macro_rules! validate {
    ($($validation:expr),* $(,)?) => {
        {
            let mut builder = $crate::validation::ValidationBuilder::new();
            $(
                builder = builder.add_result($validation);
            )*
            builder.build()
        }
    };
}

/// Common validation patterns for RWA platform
pub struct RwaValidate;

impl RwaValidate {
    /// Validate asset data
    pub fn asset_data(
        name: &str,
        description: &str,
        total_value: f64,
        currency: &str,
    ) -> ValidationResult {
        validate![
            Validate::not_empty(name, "name"),
            Validate::length(name, "name", Some(1), Some(255)),
            Validate::not_empty(description, "description"),
            Validate::length(description, "description", Some(1), Some(1000)),
            Validate::positive(total_value, "total_value"),
            Validate::not_empty(currency, "currency"),
            Validate::length(currency, "currency", Some(3), Some(3)),
        ]
    }

    /// Validate user registration data
    pub fn user_registration(
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
    ) -> ValidationResult {
        validate![
            Validate::not_empty(email, "email"),
            Validate::email(email, "email"),
            Validate::not_empty(password, "password"),
            Validate::password_strength(password, "password"),
            Validate::not_empty(first_name, "first_name"),
            Validate::length(first_name, "first_name", Some(1), Some(50)),
            Validate::not_empty(last_name, "last_name"),
            Validate::length(last_name, "last_name", Some(1), Some(50)),
        ]
    }

    /// Validate payment data
    pub fn payment_data(amount: f64, currency: &str, payment_method: &str) -> ValidationResult {
        let allowed_currencies = ["USD", "EUR", "GBP", "JPY"];
        let allowed_methods = ["credit_card", "bank_transfer", "crypto"];

        validate![
            Validate::positive(amount, "amount"),
            Validate::not_empty(currency, "currency"),
            Validate::in_list_str(currency, "currency", &allowed_currencies),
            Validate::not_empty(payment_method, "payment_method"),
            Validate::in_list_str(payment_method, "payment_method", &allowed_methods),
        ]
    }

    /// Validate blockchain transaction data
    pub fn transaction_data(
        from_address: &str,
        to_address: &str,
        amount: f64,
        blockchain_network: &str,
    ) -> ValidationResult {
        let allowed_networks = ["ethereum", "solana", "polkadot"];

        validate![
            Validate::not_empty(from_address, "from_address"),
            Validate::not_empty(to_address, "to_address"),
            Validate::positive(amount, "amount"),
            Validate::not_empty(blockchain_network, "blockchain_network"),
            Validate::in_list_str(blockchain_network, "blockchain_network", &allowed_networks),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error() {
        let mut error = ValidationError::new();
        assert!(error.is_empty());

        error.add_error("field1", "error message 1");
        error.add_error("field1", "error message 2");
        error.add_error("field2", "error message 3");

        assert!(error.has_errors());
        assert_eq!(error.errors.get("field1").unwrap().len(), 2);
        assert_eq!(error.errors.get("field2").unwrap().len(), 1);
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(Validate::not_empty("hello", "field").is_ok());
        assert!(Validate::not_empty("", "field").is_err());
        assert!(Validate::not_empty("   ", "field").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(Validate::length("hello", "field", Some(3), Some(10)).is_ok());
        assert!(Validate::length("hi", "field", Some(3), Some(10)).is_err());
        assert!(Validate::length("hello world!", "field", Some(3), Some(10)).is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(Validate::email("user@example.com", "email").is_ok());
        assert!(Validate::email("invalid-email", "email").is_err());
        assert!(Validate::email("@example.com", "email").is_err());
    }

    #[test]
    fn test_validate_range() {
        assert!(Validate::range(5, "field", Some(1), Some(10)).is_ok());
        assert!(Validate::range(0, "field", Some(1), Some(10)).is_err());
        assert!(Validate::range(15, "field", Some(1), Some(10)).is_err());
    }

    #[test]
    fn test_validate_in_list() {
        let allowed = ["red", "green", "blue"];
        assert!(Validate::in_list_str("red", "color", &allowed).is_ok());
        assert!(Validate::in_list_str("yellow", "color", &allowed).is_err());
    }

    #[test]
    fn test_validate_uuid() {
        assert!(Validate::uuid("550e8400-e29b-41d4-a716-446655440000", "id").is_ok());
        assert!(Validate::uuid("invalid-uuid", "id").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        assert!(Validate::password_strength("StrongPass123!", "password").is_ok());
        assert!(Validate::password_strength("weak", "password").is_err());
        assert!(Validate::password_strength("nouppercase123!", "password").is_err());
    }

    #[test]
    fn test_validation_builder() {
        let result = ValidationBuilder::new()
            .add_result(Validate::not_empty("hello", "field1"))
            .add_result(Validate::email("invalid", "field2"))
            .build();

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.errors.contains_key("field2"));
        assert!(!error.errors.contains_key("field1"));
    }

    #[test]
    fn test_rwa_validate_asset_data() {
        let result = RwaValidate::asset_data("Test Asset", "A test asset", 1000.0, "USD");
        assert!(result.is_ok());

        let result = RwaValidate::asset_data("", "A test asset", -100.0, "INVALID");
        assert!(result.is_err());
    }

    #[test]
    fn test_rwa_validate_user_registration() {
        let result =
            RwaValidate::user_registration("user@example.com", "StrongPass123!", "John", "Doe");
        assert!(result.is_ok());

        let result = RwaValidate::user_registration("invalid-email", "weak", "", "");
        assert!(result.is_err());
    }
}
