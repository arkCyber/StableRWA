// =====================================================================================
// File: core-security/src/validation.rs
// Description: Input validation and sanitization utilities
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::SecurityError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Input validator trait
pub trait Validator {
    fn validate(&self, input: &str) -> Result<(), SecurityError>;
}

/// Email validator
pub struct EmailValidator {
    regex: Regex,
}

impl EmailValidator {
    pub fn new() -> Result<Self, SecurityError> {
        let regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|e| SecurityError::ValidationError(format!("Invalid email regex: {}", e)))?;
        Ok(Self { regex })
    }
}

impl Validator for EmailValidator {
    fn validate(&self, input: &str) -> Result<(), SecurityError> {
        if input.is_empty() {
            return Err(SecurityError::ValidationError("Email cannot be empty".to_string()));
        }
        
        if input.len() > 254 {
            return Err(SecurityError::ValidationError("Email too long".to_string()));
        }
        
        if !self.regex.is_match(input) {
            return Err(SecurityError::ValidationError("Invalid email format".to_string()));
        }
        
        Ok(())
    }
}

/// Username validator
pub struct UsernameValidator {
    min_length: usize,
    max_length: usize,
    regex: Regex,
}

impl UsernameValidator {
    pub fn new(min_length: usize, max_length: usize) -> Result<Self, SecurityError> {
        let regex = Regex::new(r"^[a-zA-Z0-9_-]+$")
            .map_err(|e| SecurityError::ValidationError(format!("Invalid username regex: {}", e)))?;
        
        Ok(Self {
            min_length,
            max_length,
            regex,
        })
    }
}

impl Validator for UsernameValidator {
    fn validate(&self, input: &str) -> Result<(), SecurityError> {
        if input.len() < self.min_length {
            return Err(SecurityError::ValidationError(
                format!("Username must be at least {} characters", self.min_length)
            ));
        }
        
        if input.len() > self.max_length {
            return Err(SecurityError::ValidationError(
                format!("Username must be at most {} characters", self.max_length)
            ));
        }
        
        if !self.regex.is_match(input) {
            return Err(SecurityError::ValidationError(
                "Username can only contain letters, numbers, underscores, and hyphens".to_string()
            ));
        }
        
        Ok(())
    }
}

/// URL validator
pub struct UrlValidator {
    allowed_schemes: Vec<String>,
}

impl UrlValidator {
    pub fn new(allowed_schemes: Vec<String>) -> Self {
        Self { allowed_schemes }
    }
    
    pub fn default() -> Self {
        Self {
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
        }
    }
}

impl Validator for UrlValidator {
    fn validate(&self, input: &str) -> Result<(), SecurityError> {
        let url = url::Url::parse(input)
            .map_err(|_| SecurityError::ValidationError("Invalid URL format".to_string()))?;
        
        if !self.allowed_schemes.contains(&url.scheme().to_string()) {
            return Err(SecurityError::ValidationError(
                format!("URL scheme '{}' not allowed", url.scheme())
            ));
        }
        
        // Check for suspicious patterns
        let url_str = url.as_str().to_lowercase();
        if url_str.contains("javascript:") || url_str.contains("data:") {
            return Err(SecurityError::ValidationError("Suspicious URL detected".to_string()));
        }
        
        Ok(())
    }
}

/// Phone number validator
pub struct PhoneValidator {
    regex: Regex,
}

impl PhoneValidator {
    pub fn new() -> Result<Self, SecurityError> {
        // Simple international phone number regex
        let regex = Regex::new(r"^\+?[1-9]\d{1,14}$")
            .map_err(|e| SecurityError::ValidationError(format!("Invalid phone regex: {}", e)))?;
        Ok(Self { regex })
    }
}

impl Validator for PhoneValidator {
    fn validate(&self, input: &str) -> Result<(), SecurityError> {
        let cleaned = input.replace(&[' ', '-', '(', ')', '.'][..], "");
        
        if !self.regex.is_match(&cleaned) {
            return Err(SecurityError::ValidationError("Invalid phone number format".to_string()));
        }
        
        Ok(())
    }
}

/// Input sanitizer
pub struct InputSanitizer;

impl InputSanitizer {
    /// Remove HTML tags and encode special characters
    pub fn sanitize_html(input: &str) -> String {
        // Remove HTML tags
        let no_tags = Regex::new(r"<[^>]*>").unwrap().replace_all(input, "");
        
        // Encode special characters
        no_tags
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;")
    }
    
    /// Remove SQL injection patterns
    pub fn sanitize_sql(input: &str) -> String {
        let dangerous_patterns = [
            "';", "--", "/*", "*/", "xp_", "sp_", "exec", "execute",
            "select", "insert", "update", "delete", "drop", "create",
            "alter", "union", "script", "javascript", "vbscript"
        ];
        
        let mut sanitized = input.to_lowercase();
        for pattern in &dangerous_patterns {
            sanitized = sanitized.replace(pattern, "");
        }
        
        sanitized
    }
    
    /// Remove XSS patterns
    pub fn sanitize_xss(input: &str) -> String {
        let xss_patterns = [
            "javascript:", "vbscript:", "onload=", "onerror=", "onclick=",
            "onmouseover=", "onfocus=", "onblur=", "onchange=", "onsubmit=",
            "<script", "</script>", "eval(", "expression(", "url(javascript:",
            "url(data:", "url(vbscript:"
        ];
        
        let mut sanitized = input.to_string();
        for pattern in &xss_patterns {
            sanitized = sanitized.replace(pattern, "");
        }
        
        sanitized
    }
    
    /// Normalize whitespace
    pub fn normalize_whitespace(input: &str) -> String {
        Regex::new(r"\s+").unwrap().replace_all(input.trim(), " ").to_string()
    }
    
    /// Remove control characters
    pub fn remove_control_chars(input: &str) -> String {
        input.chars().filter(|c| !c.is_control() || *c == '\n' || *c == '\t').collect()
    }
}

/// Validation rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub email: EmailRules,
    pub username: UsernameRules,
    pub password: PasswordRules,
    pub phone: PhoneRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRules {
    pub max_length: usize,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameRules {
    pub min_length: usize,
    pub max_length: usize,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRules {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneRules {
    pub required: bool,
    pub allow_international: bool,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            email: EmailRules {
                max_length: 254,
                required: true,
            },
            username: UsernameRules {
                min_length: 3,
                max_length: 30,
                required: true,
            },
            password: PasswordRules {
                min_length: 8,
                max_length: 128,
                require_uppercase: true,
                require_lowercase: true,
                require_numbers: true,
                require_special_chars: true,
            },
            phone: PhoneRules {
                required: false,
                allow_international: true,
            },
        }
    }
}

/// Comprehensive input validator
pub struct InputValidator {
    rules: ValidationRules,
    email_validator: EmailValidator,
    username_validator: UsernameValidator,
    url_validator: UrlValidator,
    phone_validator: PhoneValidator,
}

impl InputValidator {
    pub fn new(rules: ValidationRules) -> Result<Self, SecurityError> {
        Ok(Self {
            email_validator: EmailValidator::new()?,
            username_validator: UsernameValidator::new(rules.username.min_length, rules.username.max_length)?,
            url_validator: UrlValidator::default(),
            phone_validator: PhoneValidator::new()?,
            rules,
        })
    }
    
    pub fn validate_email(&self, email: &str) -> Result<(), SecurityError> {
        if email.is_empty() && self.rules.email.required {
            return Err(SecurityError::ValidationError("Email is required".to_string()));
        }
        
        if !email.is_empty() {
            self.email_validator.validate(email)?;
        }
        
        Ok(())
    }
    
    pub fn validate_username(&self, username: &str) -> Result<(), SecurityError> {
        if username.is_empty() && self.rules.username.required {
            return Err(SecurityError::ValidationError("Username is required".to_string()));
        }
        
        if !username.is_empty() {
            self.username_validator.validate(username)?;
        }
        
        Ok(())
    }
    
    pub fn validate_url(&self, url: &str) -> Result<(), SecurityError> {
        if !url.is_empty() {
            self.url_validator.validate(url)?;
        }
        Ok(())
    }
    
    pub fn validate_phone(&self, phone: &str) -> Result<(), SecurityError> {
        if phone.is_empty() && self.rules.phone.required {
            return Err(SecurityError::ValidationError("Phone number is required".to_string()));
        }
        
        if !phone.is_empty() {
            self.phone_validator.validate(phone)?;
        }
        
        Ok(())
    }
    
    /// Validate and sanitize user input
    pub fn validate_and_sanitize(&self, input: &str, field_type: &str) -> Result<String, SecurityError> {
        // First sanitize
        let sanitized = match field_type {
            "html" => InputSanitizer::sanitize_html(input),
            "sql" => InputSanitizer::sanitize_sql(input),
            "xss" => InputSanitizer::sanitize_xss(input),
            _ => InputSanitizer::normalize_whitespace(input),
        };
        
        // Then validate based on type
        match field_type {
            "email" => self.validate_email(&sanitized)?,
            "username" => self.validate_username(&sanitized)?,
            "url" => self.validate_url(&sanitized)?,
            "phone" => self.validate_phone(&sanitized)?,
            _ => {}
        }
        
        Ok(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_email_validator() {
        let validator = EmailValidator::new().unwrap();
        
        assert!(validator.validate("user@example.com").is_ok());
        assert!(validator.validate("test.email+tag@domain.co.uk").is_ok());
        
        assert!(validator.validate("").is_err());
        assert!(validator.validate("invalid-email").is_err());
        assert!(validator.validate("@domain.com").is_err());
        assert!(validator.validate("user@").is_err());
    }
    
    #[test]
    fn test_username_validator() {
        let validator = UsernameValidator::new(3, 20).unwrap();
        
        assert!(validator.validate("user123").is_ok());
        assert!(validator.validate("test_user").is_ok());
        assert!(validator.validate("user-name").is_ok());
        
        assert!(validator.validate("ab").is_err()); // Too short
        assert!(validator.validate("a".repeat(21).as_str()).is_err()); // Too long
        assert!(validator.validate("user@name").is_err()); // Invalid character
        assert!(validator.validate("user name").is_err()); // Space not allowed
    }
    
    #[test]
    fn test_url_validator() {
        let validator = UrlValidator::default();
        
        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("http://localhost:8080/path").is_ok());
        
        assert!(validator.validate("javascript:alert('xss')").is_err());
        assert!(validator.validate("data:text/html,<script>alert('xss')</script>").is_err());
        assert!(validator.validate("ftp://example.com").is_err()); // Not in allowed schemes
    }
    
    #[test]
    fn test_phone_validator() {
        let validator = PhoneValidator::new().unwrap();
        
        assert!(validator.validate("+1234567890").is_ok());
        assert!(validator.validate("1234567890").is_ok());
        assert!(validator.validate("+44 20 7946 0958").is_ok());
        
        assert!(validator.validate("").is_err());
        assert!(validator.validate("123").is_err()); // Too short
        assert!(validator.validate("abc123").is_err()); // Contains letters
    }
    
    #[test]
    fn test_input_sanitizer() {
        assert_eq!(
            InputSanitizer::sanitize_html("<script>alert('xss')</script>Hello"),
            "alert(&#x27;xss&#x27;)Hello"
        );
        
        assert_eq!(
            InputSanitizer::sanitize_xss("javascript:alert('xss')"),
            "alert('xss')"
        );
        
        assert_eq!(
            InputSanitizer::normalize_whitespace("  hello   world  "),
            "hello world"
        );
        
        let sql_input = "'; DROP TABLE users; --";
        let sanitized = InputSanitizer::sanitize_sql(sql_input);
        assert!(!sanitized.contains("DROP"));
        assert!(!sanitized.contains("--"));
    }
    
    #[test]
    fn test_input_validator() {
        let rules = ValidationRules::default();
        let validator = InputValidator::new(rules).unwrap();
        
        assert!(validator.validate_email("user@example.com").is_ok());
        assert!(validator.validate_username("testuser").is_ok());
        assert!(validator.validate_url("https://example.com").is_ok());
        assert!(validator.validate_phone("+1234567890").is_ok());
        
        // Test sanitization
        let result = validator.validate_and_sanitize("<script>alert('xss')</script>", "html").unwrap();
        assert!(!result.contains("<script>"));
    }
}
