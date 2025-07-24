// =====================================================================================
// RWA Tokenization Platform - Custody Service Utilities
// 
// Utility functions and helpers for custody service operations including
// cryptographic utilities, validation helpers, and common data transformations.
//
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{CustodyError, CustodyResult};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

/// Cryptographic utilities for custody operations
pub struct CryptoUtils;

/// Validation utilities for input data
pub struct ValidationUtils;

/// Data transformation utilities
pub struct TransformUtils;

/// Time utilities for custody operations
pub struct TimeUtils;

/// ID generation utilities
pub struct IdUtils;

/// Hash utilities for data integrity
pub struct HashUtils;

/// Pagination helper for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number (1-based)
    pub page: u32,
    /// Number of items per page
    pub limit: u32,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub pages: u32,
    /// Whether there is a next page
    pub has_next: bool,
    /// Whether there is a previous page
    pub has_prev: bool,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Response data
    pub data: Vec<T>,
    /// Pagination information
    pub pagination: Pagination,
    /// Response metadata
    pub metadata: HashMap<String, String>,
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Retry configuration for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries (milliseconds)
    pub initial_delay_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum delay between retries (milliseconds)
    pub max_delay_ms: u64,
    /// Whether to add jitter to delays
    pub jitter: bool,
}

/// Retry operation result
#[derive(Debug, Clone)]
pub enum RetryResult<T> {
    /// Operation succeeded
    Success(T),
    /// Operation failed after all retries
    Failed(String),
    /// Operation was cancelled
    Cancelled,
}

impl CryptoUtils {
    /// Generate a secure random string
    /// 
    /// # Arguments
    /// 
    /// * `length` - Length of the string to generate
    /// 
    /// # Returns
    /// 
    /// Returns a secure random string
    pub fn generate_random_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate a secure random bytes
    /// 
    /// # Arguments
    /// 
    /// * `length` - Number of bytes to generate
    /// 
    /// # Returns
    /// 
    /// Returns secure random bytes
    pub fn generate_random_bytes(length: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..length).map(|_| rng.gen()).collect()
    }

    /// Hash data using SHA-256
    /// 
    /// # Arguments
    /// 
    /// * `data` - Data to hash
    /// 
    /// # Returns
    /// 
    /// Returns SHA-256 hash as hex string
    pub fn sha256_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Generate HMAC-SHA256
    /// 
    /// # Arguments
    /// 
    /// * `key` - HMAC key
    /// * `data` - Data to authenticate
    /// 
    /// # Returns
    /// 
    /// Returns HMAC-SHA256 as hex string
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        format!("{:x}", mac.finalize().into_bytes())
    }

    /// Constant-time string comparison
    /// 
    /// # Arguments
    /// 
    /// * `a` - First string
    /// * `b` - Second string
    /// 
    /// # Returns
    /// 
    /// Returns true if strings are equal
    pub fn constant_time_eq(a: &str, b: &str) -> bool {
        use subtle::ConstantTimeEq;
        a.as_bytes().ct_eq(b.as_bytes()).into()
    }
}

impl ValidationUtils {
    /// Validate email address format
    /// 
    /// # Arguments
    /// 
    /// * `email` - Email address to validate
    /// 
    /// # Returns
    /// 
    /// Returns true if email format is valid
    pub fn is_valid_email(email: &str) -> bool {
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).unwrap();
        email_regex.is_match(email)
    }

    /// Validate UUID format
    /// 
    /// # Arguments
    /// 
    /// * `uuid_str` - UUID string to validate
    /// 
    /// # Returns
    /// 
    /// Returns true if UUID format is valid
    pub fn is_valid_uuid(uuid_str: &str) -> bool {
        Uuid::parse_str(uuid_str).is_ok()
    }

    /// Validate URL format
    /// 
    /// # Arguments
    /// 
    /// * `url` - URL to validate
    /// 
    /// # Returns
    /// 
    /// Returns true if URL format is valid
    pub fn is_valid_url(url: &str) -> bool {
        url::Url::parse(url).is_ok()
    }

    /// Validate phone number format (basic)
    /// 
    /// # Arguments
    /// 
    /// * `phone` - Phone number to validate
    /// 
    /// # Returns
    /// 
    /// Returns true if phone format is valid
    pub fn is_valid_phone(phone: &str) -> bool {
        let phone_regex = regex::Regex::new(
            r"^\+?[1-9]\d{1,14}$"
        ).unwrap();
        phone_regex.is_match(phone)
    }

    /// Validate string length
    /// 
    /// # Arguments
    /// 
    /// * `value` - String to validate
    /// * `min_length` - Minimum length
    /// * `max_length` - Maximum length
    /// 
    /// # Returns
    /// 
    /// Returns validation result
    pub fn validate_string_length(value: &str, min_length: usize, max_length: usize) -> CustodyResult<()> {
        if value.len() < min_length {
            return Err(CustodyError::validation(
                "string_length",
                format!("String too short, minimum length is {}", min_length),
            ));
        }
        
        if value.len() > max_length {
            return Err(CustodyError::validation(
                "string_length",
                format!("String too long, maximum length is {}", max_length),
            ));
        }
        
        Ok(())
    }

    /// Validate required fields in a map
    /// 
    /// # Arguments
    /// 
    /// * `data` - Data map to validate
    /// * `required_fields` - List of required field names
    /// 
    /// # Returns
    /// 
    /// Returns validation result
    pub fn validate_required_fields(
        data: &HashMap<String, String>,
        required_fields: &[&str],
    ) -> ValidationResult {
        let mut errors = Vec::new();
        
        for field in required_fields {
            if !data.contains_key(*field) || data[*field].is_empty() {
                errors.push(format!("Required field '{}' is missing or empty", field));
            }
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings: Vec::new(),
        }
    }
}

impl TransformUtils {
    /// Convert string to title case
    /// 
    /// # Arguments
    /// 
    /// * `input` - Input string
    /// 
    /// # Returns
    /// 
    /// Returns title case string
    pub fn to_title_case(input: &str) -> String {
        input
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Sanitize string for safe storage
    /// 
    /// # Arguments
    /// 
    /// * `input` - Input string to sanitize
    /// 
    /// # Returns
    /// 
    /// Returns sanitized string
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.,!?".contains(*c))
            .collect()
    }

    /// Truncate string to maximum length
    /// 
    /// # Arguments
    /// 
    /// * `input` - Input string
    /// * `max_length` - Maximum length
    /// 
    /// # Returns
    /// 
    /// Returns truncated string
    pub fn truncate_string(input: &str, max_length: usize) -> String {
        if input.len() <= max_length {
            input.to_string()
        } else {
            format!("{}...", &input[..max_length.saturating_sub(3)])
        }
    }

    /// Convert bytes to human-readable format
    /// 
    /// # Arguments
    /// 
    /// * `bytes` - Number of bytes
    /// 
    /// # Returns
    /// 
    /// Returns human-readable string
    pub fn bytes_to_human_readable(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
}

impl TimeUtils {
    /// Get current UTC timestamp
    /// 
    /// # Returns
    /// 
    /// Returns current UTC timestamp
    pub fn now_utc() -> DateTime<Utc> {
        Utc::now()
    }

    /// Parse ISO 8601 timestamp
    /// 
    /// # Arguments
    /// 
    /// * `timestamp` - ISO 8601 timestamp string
    /// 
    /// # Returns
    /// 
    /// Returns parsed DateTime or error
    pub fn parse_iso8601(timestamp: &str) -> CustodyResult<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(timestamp)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| CustodyError::validation("timestamp", &e.to_string()))
    }

    /// Format timestamp as ISO 8601
    /// 
    /// # Arguments
    /// 
    /// * `datetime` - DateTime to format
    /// 
    /// # Returns
    /// 
    /// Returns ISO 8601 formatted string
    pub fn format_iso8601(datetime: &DateTime<Utc>) -> String {
        datetime.to_rfc3339()
    }

    /// Check if timestamp is within a time window
    /// 
    /// # Arguments
    /// 
    /// * `timestamp` - Timestamp to check
    /// * `window_seconds` - Time window in seconds
    /// 
    /// # Returns
    /// 
    /// Returns true if timestamp is within the window
    pub fn is_within_time_window(timestamp: &DateTime<Utc>, window_seconds: i64) -> bool {
        let now = Utc::now();
        let diff = (now - *timestamp).num_seconds().abs();
        diff <= window_seconds
    }
}

impl IdUtils {
    /// Generate a new UUID v4
    /// 
    /// # Returns
    /// 
    /// Returns new UUID
    pub fn generate_uuid() -> Uuid {
        Uuid::new_v4()
    }

    /// Generate a short ID (8 characters)
    /// 
    /// # Returns
    /// 
    /// Returns short ID string
    pub fn generate_short_id() -> String {
        CryptoUtils::generate_random_string(8)
    }

    /// Generate a request ID for tracing
    /// 
    /// # Returns
    /// 
    /// Returns request ID string
    pub fn generate_request_id() -> String {
        format!("req_{}", Self::generate_short_id())
    }

    /// Generate a transaction ID
    /// 
    /// # Returns
    /// 
    /// Returns transaction ID string
    pub fn generate_transaction_id() -> String {
        format!("tx_{}", Self::generate_uuid().simple())
    }
}

impl HashUtils {
    /// Create a content hash for data integrity
    /// 
    /// # Arguments
    /// 
    /// * `data` - Data to hash
    /// 
    /// # Returns
    /// 
    /// Returns content hash
    pub fn content_hash(data: &[u8]) -> String {
        CryptoUtils::sha256_hash(data)
    }

    /// Create a fingerprint for an object
    /// 
    /// # Arguments
    /// 
    /// * `object` - Serializable object
    /// 
    /// # Returns
    /// 
    /// Returns object fingerprint or error
    pub fn object_fingerprint<T: Serialize>(object: &T) -> CustodyResult<String> {
        let json = serde_json::to_vec(object)
            .map_err(|e| CustodyError::internal(&e.to_string()))?;
        Ok(Self::content_hash(&json))
    }
}

impl Pagination {
    /// Create new pagination info
    /// 
    /// # Arguments
    /// 
    /// * `page` - Current page number
    /// * `limit` - Items per page
    /// * `total` - Total number of items
    /// 
    /// # Returns
    /// 
    /// Returns pagination info
    pub fn new(page: u32, limit: u32, total: u64) -> Self {
        let pages = ((total as f64) / (limit as f64)).ceil() as u32;
        let has_next = page < pages;
        let has_prev = page > 1;

        Self {
            page,
            limit,
            total,
            pages,
            has_next,
            has_prev,
        }
    }

    /// Calculate offset for database queries
    /// 
    /// # Returns
    /// 
    /// Returns offset value
    pub fn offset(&self) -> u64 {
        ((self.page - 1) * self.limit) as u64
    }
}

impl<T> PaginatedResponse<T> {
    /// Create new paginated response
    /// 
    /// # Arguments
    /// 
    /// * `data` - Response data
    /// * `pagination` - Pagination info
    /// 
    /// # Returns
    /// 
    /// Returns paginated response
    pub fn new(data: Vec<T>, pagination: Pagination) -> Self {
        Self {
            data,
            pagination,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to response
    /// 
    /// # Arguments
    /// 
    /// * `key` - Metadata key
    /// * `value` - Metadata value
    /// 
    /// # Returns
    /// 
    /// Returns self for chaining
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl RetryConfig {
    /// Create default retry configuration
    /// 
    /// # Returns
    /// 
    /// Returns default retry config
    pub fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30000,
            jitter: true,
        }
    }

    /// Calculate delay for retry attempt
    /// 
    /// # Arguments
    /// 
    /// * `attempt` - Current attempt number (0-based)
    /// 
    /// # Returns
    /// 
    /// Returns delay in milliseconds
    pub fn calculate_delay(&self, attempt: u32) -> u64 {
        let base_delay = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let delay = base_delay.min(self.max_delay_ms as f64) as u64;
        
        if self.jitter {
            let jitter_range = delay / 10; // 10% jitter
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(0..=jitter_range);
            delay + jitter
        } else {
            delay
        }
    }
}

/// Retry an operation with exponential backoff
/// 
/// # Arguments
/// 
/// * `config` - Retry configuration
/// * `operation` - Operation to retry
/// 
/// # Returns
/// 
/// Returns retry result
pub async fn retry_operation<T, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
) -> RetryResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    for attempt in 0..config.max_attempts {
        match operation().await {
            Ok(result) => return RetryResult::Success(result),
            Err(error) => {
                if attempt == config.max_attempts - 1 {
                    return RetryResult::Failed(error);
                }
                
                let delay = config.calculate_delay(attempt);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
        }
    }
    
    RetryResult::Failed("Max attempts exceeded".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_crypto_utils_random_string() {
        let random_str = CryptoUtils::generate_random_string(16);
        assert_eq!(random_str.len(), 16);
        assert!(random_str.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_crypto_utils_random_bytes() {
        let random_bytes = CryptoUtils::generate_random_bytes(32);
        assert_eq!(random_bytes.len(), 32);
    }

    #[test]
    fn test_crypto_utils_sha256_hash() {
        let data = b"hello world";
        let hash = CryptoUtils::sha256_hash(data);
        assert_eq!(hash.len(), 64); // SHA-256 produces 32 bytes = 64 hex chars
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_crypto_utils_constant_time_eq() {
        assert!(CryptoUtils::constant_time_eq("hello", "hello"));
        assert!(!CryptoUtils::constant_time_eq("hello", "world"));
    }

    #[test]
    fn test_validation_utils_email() {
        assert!(ValidationUtils::is_valid_email("test@example.com"));
        assert!(ValidationUtils::is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!ValidationUtils::is_valid_email("invalid-email"));
        assert!(!ValidationUtils::is_valid_email("@domain.com"));
        assert!(!ValidationUtils::is_valid_email("user@"));
    }

    #[test]
    fn test_validation_utils_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let invalid_uuid = "not-a-uuid";
        
        assert!(ValidationUtils::is_valid_uuid(valid_uuid));
        assert!(!ValidationUtils::is_valid_uuid(invalid_uuid));
    }

    #[test]
    fn test_validation_utils_url() {
        assert!(ValidationUtils::is_valid_url("https://example.com"));
        assert!(ValidationUtils::is_valid_url("http://localhost:8080/path"));
        assert!(!ValidationUtils::is_valid_url("not-a-url"));
        assert!(!ValidationUtils::is_valid_url("ftp://"));
    }

    #[test]
    fn test_validation_utils_phone() {
        assert!(ValidationUtils::is_valid_phone("+1234567890"));
        assert!(ValidationUtils::is_valid_phone("1234567890"));
        assert!(!ValidationUtils::is_valid_phone("123"));
        assert!(!ValidationUtils::is_valid_phone("abc123"));
    }

    #[test]
    fn test_validation_utils_string_length() {
        assert!(ValidationUtils::validate_string_length("hello", 3, 10).is_ok());
        assert!(ValidationUtils::validate_string_length("hi", 3, 10).is_err());
        assert!(ValidationUtils::validate_string_length("this is too long", 3, 10).is_err());
    }

    #[test]
    fn test_validation_utils_required_fields() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John".to_string());
        data.insert("email".to_string(), "john@example.com".to_string());
        
        let result = ValidationUtils::validate_required_fields(&data, &["name", "email"]);
        assert!(result.is_valid);
        
        let result = ValidationUtils::validate_required_fields(&data, &["name", "email", "phone"]);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_transform_utils_title_case() {
        assert_eq!(TransformUtils::to_title_case("hello world"), "Hello World");
        assert_eq!(TransformUtils::to_title_case("HELLO WORLD"), "Hello World");
        assert_eq!(TransformUtils::to_title_case("hELLo WoRLd"), "Hello World");
    }

    #[test]
    fn test_transform_utils_sanitize_string() {
        let input = "Hello, World! <script>alert('xss')</script>";
        let sanitized = TransformUtils::sanitize_string(input);
        assert_eq!(sanitized, "Hello, World! scriptalert'xss'script");
    }

    #[test]
    fn test_transform_utils_truncate_string() {
        assert_eq!(TransformUtils::truncate_string("hello", 10), "hello");
        assert_eq!(TransformUtils::truncate_string("hello world", 8), "hello...");
        assert_eq!(TransformUtils::truncate_string("hi", 8), "hi");
    }

    #[test]
    fn test_transform_utils_bytes_to_human_readable() {
        assert_eq!(TransformUtils::bytes_to_human_readable(512), "512 B");
        assert_eq!(TransformUtils::bytes_to_human_readable(1024), "1.00 KB");
        assert_eq!(TransformUtils::bytes_to_human_readable(1536), "1.50 KB");
        assert_eq!(TransformUtils::bytes_to_human_readable(1048576), "1.00 MB");
    }

    #[test]
    fn test_time_utils_iso8601() {
        let now = TimeUtils::now_utc();
        let formatted = TimeUtils::format_iso8601(&now);
        let parsed = TimeUtils::parse_iso8601(&formatted).unwrap();
        
        // Allow for small differences due to precision
        let diff = (now - parsed).num_milliseconds().abs();
        assert!(diff < 1000);
    }

    #[test]
    fn test_time_utils_time_window() {
        let now = TimeUtils::now_utc();
        let past = now - chrono::Duration::seconds(30);
        let future = now + chrono::Duration::seconds(30);
        let far_past = now - chrono::Duration::seconds(120);
        
        assert!(TimeUtils::is_within_time_window(&past, 60));
        assert!(TimeUtils::is_within_time_window(&future, 60));
        assert!(!TimeUtils::is_within_time_window(&far_past, 60));
    }

    #[test]
    fn test_id_utils_generation() {
        let uuid = IdUtils::generate_uuid();
        assert_ne!(uuid, Uuid::nil());
        
        let short_id = IdUtils::generate_short_id();
        assert_eq!(short_id.len(), 8);
        
        let request_id = IdUtils::generate_request_id();
        assert!(request_id.starts_with("req_"));
        
        let tx_id = IdUtils::generate_transaction_id();
        assert!(tx_id.starts_with("tx_"));
    }

    #[test]
    fn test_hash_utils_content_hash() {
        let data = b"test data";
        let hash = HashUtils::content_hash(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_utils_object_fingerprint() {
        #[derive(Serialize)]
        struct TestObject {
            name: String,
            value: i32,
        }
        
        let obj = TestObject {
            name: "test".to_string(),
            value: 42,
        };
        
        let fingerprint = HashUtils::object_fingerprint(&obj).unwrap();
        assert_eq!(fingerprint.len(), 64);
        
        // Same object should produce same fingerprint
        let fingerprint2 = HashUtils::object_fingerprint(&obj).unwrap();
        assert_eq!(fingerprint, fingerprint2);
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 10, 25);
        
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.limit, 10);
        assert_eq!(pagination.total, 25);
        assert_eq!(pagination.pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
        assert_eq!(pagination.offset(), 10);
    }

    #[test]
    fn test_paginated_response() {
        let data = vec!["item1", "item2", "item3"];
        let pagination = Pagination::new(1, 10, 3);
        
        let response = PaginatedResponse::new(data, pagination)
            .with_metadata("source".to_string(), "test".to_string());
        
        assert_eq!(response.data.len(), 3);
        assert_eq!(response.pagination.total, 3);
        assert_eq!(response.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.backoff_multiplier, 2.0);
        
        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        
        // Second attempt should have longer delay (with jitter, it's approximate)
        assert!(delay1 >= delay0);
    }

    #[tokio::test]
    async fn test_retry_operation_success() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            backoff_multiplier: 2.0,
            max_delay_ms: 1000,
            jitter: false,
        };
        
        let mut attempt_count = 0;
        let result = retry_operation(&config, || {
            attempt_count += 1;
            async move {
                if attempt_count < 2 {
                    Err("Temporary failure".to_string())
                } else {
                    Ok("Success".to_string())
                }
            }
        }).await;
        
        match result {
            RetryResult::Success(value) => assert_eq!(value, "Success"),
            _ => panic!("Expected success"),
        }
        assert_eq!(attempt_count, 2);
    }

    #[tokio::test]
    async fn test_retry_operation_failure() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            backoff_multiplier: 2.0,
            max_delay_ms: 1000,
            jitter: false,
        };
        
        let mut attempt_count = 0;
        let result = retry_operation(&config, || {
            attempt_count += 1;
            async move { Err("Persistent failure".to_string()) }
        }).await;
        
        match result {
            RetryResult::Failed(error) => assert_eq!(error, "Persistent failure"),
            _ => panic!("Expected failure"),
        }
        assert_eq!(attempt_count, 2);
    }
}
