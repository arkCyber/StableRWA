// =====================================================================================
// File: core-utils/src/helpers.rs
// Description: Helper functions and utilities for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

// use crate::UtilError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: Option<u64>,
    pub total_pages: Option<u32>,
}

impl Pagination {
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),                // Ensure page is at least 1
            per_page: per_page.clamp(1, 100), // Limit per_page between 1 and 100
            total: None,
            total_pages: None,
        }
    }

    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self.total_pages = Some(((total as f64) / (self.per_page as f64)).ceil() as u32);
        self
    }

    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    pub fn limit(&self) -> u32 {
        self.per_page
    }

    pub fn has_next_page(&self) -> bool {
        if let Some(total_pages) = self.total_pages {
            self.page < total_pages
        } else {
            false
        }
    }

    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(1, 20)
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: Pagination,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, pagination: Pagination) -> Self {
        Self { data, pagination }
    }
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

/// API error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn validation_error(field: &str, message: &str) -> Self {
        let mut details = HashMap::new();
        details.insert(
            "field".to_string(),
            serde_json::Value::String(field.to_string()),
        );

        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: message.to_string(),
            details: Some(details),
        }
    }

    pub fn not_found(resource: &str, id: &str) -> Self {
        let mut details = HashMap::new();
        details.insert(
            "resource".to_string(),
            serde_json::Value::String(resource.to_string()),
        );
        details.insert("id".to_string(), serde_json::Value::String(id.to_string()));

        Self {
            code: "NOT_FOUND".to_string(),
            message: format!("{} with id '{}' not found", resource, id),
            details: Some(details),
        }
    }

    pub fn unauthorized() -> Self {
        Self::new("UNAUTHORIZED", "Authentication required")
    }

    pub fn forbidden() -> Self {
        Self::new("FORBIDDEN", "Access denied")
    }

    pub fn internal_error() -> Self {
        Self::new("INTERNAL_ERROR", "An internal server error occurred")
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// String utilities
pub struct StringUtils;

impl StringUtils {
    /// Convert string to snake_case
    pub fn to_snake_case(input: &str) -> String {
        let mut result = String::new();
        let mut prev_char_was_uppercase = false;

        for (i, ch) in input.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 && !prev_char_was_uppercase {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap());
                prev_char_was_uppercase = true;
            } else {
                result.push(ch);
                prev_char_was_uppercase = false;
            }
        }

        result
    }

    /// Convert string to camelCase
    pub fn to_camel_case(input: &str) -> String {
        let words: Vec<&str> = input.split('_').collect();
        let mut result = String::new();

        for (i, word) in words.iter().enumerate() {
            if i == 0 {
                result.push_str(&word.to_lowercase());
            } else {
                result.push_str(&Self::capitalize_first(word));
            }
        }

        result
    }

    /// Convert string to PascalCase
    pub fn to_pascal_case(input: &str) -> String {
        input
            .split('_')
            .map(|word| Self::capitalize_first(word))
            .collect::<Vec<String>>()
            .join("")
    }

    /// Capitalize first letter of a string
    pub fn capitalize_first(input: &str) -> String {
        let mut chars = input.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Truncate string to specified length with ellipsis
    pub fn truncate(input: &str, max_length: usize) -> String {
        if input.len() <= max_length {
            input.to_string()
        } else {
            format!("{}...", &input[..max_length.saturating_sub(3)])
        }
    }

    /// Generate a slug from a string
    pub fn slugify(input: &str) -> String {
        input
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>()
            .join("-")
    }
}

/// Number utilities
pub struct NumberUtils;

impl NumberUtils {
    /// Format number with thousands separators
    pub fn format_with_commas(number: u64) -> String {
        let num_str = number.to_string();
        let mut result = String::new();

        for (i, digit) in num_str.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(digit);
        }

        result.chars().rev().collect()
    }

    /// Format currency amount
    pub fn format_currency(amount: f64, currency: &str) -> String {
        match currency.to_uppercase().as_str() {
            "USD" => format!("${:.2}", amount),
            "EUR" => format!("€{:.2}", amount),
            "GBP" => format!("£{:.2}", amount),
            "JPY" => format!("¥{:.0}", amount),
            _ => format!("{:.2} {}", amount, currency),
        }
    }

    /// Calculate percentage
    pub fn percentage(part: f64, total: f64) -> f64 {
        if total == 0.0 {
            0.0
        } else {
            (part / total) * 100.0
        }
    }

    /// Round to specified decimal places
    pub fn round_to_places(number: f64, places: u32) -> f64 {
        let multiplier = 10_f64.powi(places as i32);
        (number * multiplier).round() / multiplier
    }
}

/// Date utilities
pub struct DateUtils;

impl DateUtils {
    /// Get current timestamp in milliseconds
    pub fn now_millis() -> u64 {
        Utc::now().timestamp_millis() as u64
    }

    /// Get current timestamp in seconds
    pub fn now_seconds() -> u64 {
        Utc::now().timestamp() as u64
    }

    /// Format date for display
    pub fn format_date(date: &DateTime<Utc>) -> String {
        date.format("%Y-%m-%d").to_string()
    }

    /// Format datetime for display
    pub fn format_datetime(date: &DateTime<Utc>) -> String {
        date.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// Get start of day
    pub fn start_of_day(date: &DateTime<Utc>) -> DateTime<Utc> {
        date.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()
    }

    /// Get end of day
    pub fn end_of_day(date: &DateTime<Utc>) -> DateTime<Utc> {
        date.date_naive().and_hms_opt(23, 59, 59).unwrap().and_utc()
    }

    /// Check if date is today
    pub fn is_today(date: &DateTime<Utc>) -> bool {
        let now = Utc::now();
        date.date_naive() == now.date_naive()
    }

    /// Get days between two dates
    pub fn days_between(start: &DateTime<Utc>, end: &DateTime<Utc>) -> i64 {
        (end.date_naive() - start.date_naive()).num_days()
    }
}

/// Hash utilities
pub struct HashUtils;

impl HashUtils {
    /// Generate a simple hash of a string
    pub fn simple_hash(input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    /// Generate a short hash (8 characters)
    pub fn short_hash(input: &str) -> String {
        format!("{:x}", Self::simple_hash(input))[..8].to_string()
    }
}

/// Environment utilities
pub struct EnvUtils;

impl EnvUtils {
    /// Get environment variable with default
    pub fn get_env_or_default(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Check if running in development mode
    pub fn is_development() -> bool {
        Self::get_env_or_default("ENVIRONMENT", "development") == "development"
    }

    /// Check if running in production mode
    pub fn is_production() -> bool {
        Self::get_env_or_default("ENVIRONMENT", "development") == "production"
    }

    /// Check if running in test mode
    pub fn is_test() -> bool {
        Self::get_env_or_default("ENVIRONMENT", "development") == "test"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 10).with_total(95);

        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.offset(), 10);
        assert_eq!(pagination.limit(), 10);
        assert_eq!(pagination.total, Some(95));
        assert_eq!(pagination.total_pages, Some(10));
        assert!(pagination.has_next_page());
        assert!(pagination.has_prev_page());
    }

    #[test]
    fn test_api_response() {
        let success_response = ApiResponse::success("test data");
        assert!(success_response.success);
        assert_eq!(success_response.data, Some("test data"));
        assert!(success_response.error.is_none());

        let error_response = ApiResponse::<String>::error(ApiError::not_found("user", "123"));
        assert!(!error_response.success);
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());
    }

    #[test]
    fn test_string_utils() {
        assert_eq!(StringUtils::to_snake_case("CamelCase"), "camel_case");
        assert_eq!(StringUtils::to_camel_case("snake_case"), "snakeCase");
        assert_eq!(StringUtils::to_pascal_case("snake_case"), "SnakeCase");
        assert_eq!(StringUtils::capitalize_first("hello"), "Hello");
        assert_eq!(StringUtils::truncate("Hello, World!", 8), "Hello...");
        assert_eq!(StringUtils::slugify("Hello World!"), "hello-world");
    }

    #[test]
    fn test_number_utils() {
        assert_eq!(NumberUtils::format_with_commas(1234567), "1,234,567");
        assert_eq!(NumberUtils::format_currency(123.45, "USD"), "$123.45");
        assert_eq!(NumberUtils::percentage(25.0, 100.0), 25.0);
        assert_eq!(NumberUtils::round_to_places(3.14159, 2), 3.14);
    }

    #[test]
    fn test_date_utils() {
        let now = Utc::now();
        assert!(DateUtils::is_today(&now));

        let formatted = DateUtils::format_date(&now);
        assert!(formatted.contains('-'));

        let start = DateUtils::start_of_day(&now);
        assert_eq!(start.hour(), 0);
        assert_eq!(start.minute(), 0);
        assert_eq!(start.second(), 0);
    }

    #[test]
    fn test_hash_utils() {
        let hash1 = HashUtils::simple_hash("test");
        let hash2 = HashUtils::simple_hash("test");
        let hash3 = HashUtils::simple_hash("different");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);

        let short_hash = HashUtils::short_hash("test");
        assert_eq!(short_hash.len(), 8);
    }

    #[test]
    fn test_env_utils() {
        // These tests depend on environment variables, so they're basic
        let env_value = EnvUtils::get_env_or_default("NONEXISTENT_VAR", "default");
        assert_eq!(env_value, "default");
    }
}
