// =====================================================================================
// File: core-monitoring/src/logging.rs
// Description: Log aggregation and analysis module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{MonitoringError, MonitoringResult},
    service::LogQuery,
    types::{LogConfig, LogEntry, LogLevel},
};

/// Log aggregator trait
#[async_trait]
pub trait LogAggregator: Send + Sync {
    /// Store log entry
    async fn store_log(&self, log_entry: &LogEntry) -> MonitoringResult<()>;

    /// Query logs
    async fn query_logs(&self, query: &LogQuery) -> MonitoringResult<Vec<LogEntry>>;

    /// Get log statistics
    async fn get_log_stats(
        &self,
        time_range: &crate::types::TimeRange,
    ) -> MonitoringResult<LogStatistics>;

    /// Search logs by text
    async fn search_logs(
        &self,
        search_term: &str,
        limit: Option<u32>,
    ) -> MonitoringResult<Vec<LogEntry>>;

    /// Get log levels distribution
    async fn get_log_levels_distribution(
        &self,
        time_range: &crate::types::TimeRange,
    ) -> MonitoringResult<HashMap<LogLevel, u64>>;
}

/// Log statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStatistics {
    pub total_logs: u64,
    pub logs_by_level: HashMap<LogLevel, u64>,
    pub logs_by_source: HashMap<String, u64>,
    pub error_rate: f64,
    pub warning_rate: f64,
    pub top_sources: Vec<LogSourceStats>,
    pub time_range: crate::types::TimeRange,
}

/// Log source statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSourceStats {
    pub source: String,
    pub count: u64,
    pub error_count: u64,
    pub warning_count: u64,
}

/// Log parser for structured logging
pub struct LogParser {
    patterns: Vec<LogPattern>,
}

/// Log pattern for parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPattern {
    pub name: String,
    pub pattern: String,
    pub fields: Vec<String>,
}

/// Parsed log data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLogData {
    pub structured_fields: HashMap<String, serde_json::Value>,
    pub extracted_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub extracted_level: Option<LogLevel>,
    pub extracted_message: Option<String>,
}

/// Log aggregator implementation
pub struct LogAggregatorImpl {
    config: LogConfig,
    logs: Vec<LogEntry>,
    parser: LogParser,
}

/// Log indexer for fast searching
pub struct LogIndexer {
    text_index: HashMap<String, Vec<Uuid>>,
    level_index: HashMap<LogLevel, Vec<Uuid>>,
    source_index: HashMap<String, Vec<Uuid>>,
}

impl LogParser {
    pub fn new() -> Self {
        let patterns = vec![
            LogPattern {
                name: "nginx_access".to_string(),
                pattern:
                    r#"(\S+) \S+ \S+ \[([\w:/]+\s[+\-]\d{4})\] "(\S+) (\S+) (\S+)" (\d{3}) (\d+)"#
                        .to_string(),
                fields: vec![
                    "remote_addr".to_string(),
                    "time_local".to_string(),
                    "method".to_string(),
                    "uri".to_string(),
                    "protocol".to_string(),
                    "status".to_string(),
                    "body_bytes_sent".to_string(),
                ],
            },
            LogPattern {
                name: "application_log".to_string(),
                pattern: r#"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}) \[(\w+)\] (.+)"#.to_string(),
                fields: vec![
                    "timestamp".to_string(),
                    "level".to_string(),
                    "message".to_string(),
                ],
            },
        ];

        Self { patterns }
    }

    pub fn parse_log(&self, raw_message: &str) -> ParsedLogData {
        let mut structured_fields = HashMap::new();
        let mut extracted_timestamp = None;
        let mut extracted_level = None;
        let mut extracted_message = None;

        // Try to match against known patterns
        for pattern in &self.patterns {
            // Mock pattern matching - in reality, this would use regex
            if pattern.name == "application_log" && raw_message.contains("[") {
                // Extract timestamp
                if let Some(timestamp_str) = raw_message.split(' ').next() {
                    if let Ok(timestamp) =
                        chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
                    {
                        extracted_timestamp = Some(timestamp.and_utc());
                    }
                }

                // Extract log level
                if raw_message.contains("[ERROR]") {
                    extracted_level = Some(LogLevel::Error);
                } else if raw_message.contains("[WARN]") {
                    extracted_level = Some(LogLevel::Warn);
                } else if raw_message.contains("[INFO]") {
                    extracted_level = Some(LogLevel::Info);
                } else if raw_message.contains("[DEBUG]") {
                    extracted_level = Some(LogLevel::Debug);
                }

                // Extract message
                if let Some(msg_start) = raw_message.find("] ") {
                    extracted_message = Some(raw_message[msg_start + 2..].to_string());
                }

                structured_fields.insert(
                    "pattern_matched".to_string(),
                    serde_json::Value::String(pattern.name.clone()),
                );
                break;
            }
        }

        ParsedLogData {
            structured_fields,
            extracted_timestamp,
            extracted_level,
            extracted_message,
        }
    }
}

impl LogIndexer {
    pub fn new() -> Self {
        Self {
            text_index: HashMap::new(),
            level_index: HashMap::new(),
            source_index: HashMap::new(),
        }
    }

    pub fn index_log(&mut self, log_entry: &LogEntry) {
        // Index by text content
        let words: Vec<&str> = log_entry.message.split_whitespace().collect();
        for word in words {
            let word_lower = word.to_lowercase();
            self.text_index
                .entry(word_lower)
                .or_default()
                .push(log_entry.id);
        }

        // Index by log level
        self.level_index
            .entry(log_entry.level)
            .or_default()
            .push(log_entry.id);

        // Index by source
        self.source_index
            .entry(log_entry.source.clone())
            .or_default()
            .push(log_entry.id);
    }

    pub fn search_by_text(&self, search_term: &str) -> Vec<Uuid> {
        let search_lower = search_term.to_lowercase();
        self.text_index
            .get(&search_lower)
            .cloned()
            .unwrap_or_default()
    }

    pub fn search_by_level(&self, level: LogLevel) -> Vec<Uuid> {
        self.level_index.get(&level).cloned().unwrap_or_default()
    }

    pub fn search_by_source(&self, source: &str) -> Vec<Uuid> {
        self.source_index.get(source).cloned().unwrap_or_default()
    }
}

impl LogAggregatorImpl {
    pub fn new(config: LogConfig) -> Self {
        Self {
            config,
            logs: Vec::new(),
            parser: LogParser::new(),
        }
    }

    fn should_store_log(&self, log_entry: &LogEntry) -> bool {
        log_entry.level >= self.config.log_level
    }

    fn cleanup_old_logs(&mut self) {
        let retention_cutoff =
            Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        self.logs.retain(|log| log.timestamp > retention_cutoff);
    }

    fn calculate_error_rate(&self, logs: &[LogEntry]) -> f64 {
        if logs.is_empty() {
            return 0.0;
        }

        let error_count = logs
            .iter()
            .filter(|log| log.level >= LogLevel::Error)
            .count();

        (error_count as f64 / logs.len() as f64) * 100.0
    }

    fn calculate_warning_rate(&self, logs: &[LogEntry]) -> f64 {
        if logs.is_empty() {
            return 0.0;
        }

        let warning_count = logs
            .iter()
            .filter(|log| log.level == LogLevel::Warn)
            .count();

        (warning_count as f64 / logs.len() as f64) * 100.0
    }

    fn get_top_sources(&self, logs: &[LogEntry], limit: usize) -> Vec<LogSourceStats> {
        let mut source_stats: HashMap<String, LogSourceStats> = HashMap::new();

        for log in logs {
            let stats = source_stats
                .entry(log.source.clone())
                .or_insert(LogSourceStats {
                    source: log.source.clone(),
                    count: 0,
                    error_count: 0,
                    warning_count: 0,
                });

            stats.count += 1;

            if log.level >= LogLevel::Error {
                stats.error_count += 1;
            } else if log.level == LogLevel::Warn {
                stats.warning_count += 1;
            }
        }

        let mut sorted_stats: Vec<LogSourceStats> = source_stats.into_values().collect();
        sorted_stats.sort_by(|a, b| b.count.cmp(&a.count));
        sorted_stats.truncate(limit);

        sorted_stats
    }
}

#[async_trait]
impl LogAggregator for LogAggregatorImpl {
    async fn store_log(&self, log_entry: &LogEntry) -> MonitoringResult<()> {
        if !self.should_store_log(log_entry) {
            return Ok(());
        }

        // Parse structured data if enabled
        if self.config.structured_logging {
            let _parsed_data = self.parser.parse_log(&log_entry.message);
            // In a real implementation, this would enhance the log entry with parsed data
        }

        // Mock log storage - in reality, this would persist to a database or file
        println!(
            "Storing log: [{}] {} - {}",
            log_entry.level as u8, log_entry.source, log_entry.message
        );

        Ok(())
    }

    async fn query_logs(&self, query: &LogQuery) -> MonitoringResult<Vec<LogEntry>> {
        // Mock log querying - in reality, this would query a database
        let mut mock_logs = vec![
            LogEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now() - chrono::Duration::minutes(5),
                level: LogLevel::Info,
                message: "Application started successfully".to_string(),
                source: "app".to_string(),
                labels: HashMap::new(),
                fields: HashMap::new(),
            },
            LogEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now() - chrono::Duration::minutes(3),
                level: LogLevel::Warn,
                message: "High memory usage detected".to_string(),
                source: "system".to_string(),
                labels: HashMap::new(),
                fields: HashMap::new(),
            },
            LogEntry {
                id: Uuid::new_v4(),
                timestamp: Utc::now() - chrono::Duration::minutes(1),
                level: LogLevel::Error,
                message: "Database connection failed".to_string(),
                source: "database".to_string(),
                labels: HashMap::new(),
                fields: HashMap::new(),
            },
        ];

        // Apply filters
        mock_logs.retain(|log| {
            // Time range filter
            if log.timestamp < query.start_time || log.timestamp > query.end_time {
                return false;
            }

            // Level filter
            if let Some(level) = query.level {
                if log.level != level {
                    return false;
                }
            }

            // Source filter
            if let Some(ref source) = query.source {
                if log.source != *source {
                    return false;
                }
            }

            // Message content filter
            if let Some(ref contains) = query.message_contains {
                if !log.message.contains(contains) {
                    return false;
                }
            }

            // Labels filter
            for (key, value) in &query.labels {
                if log.labels.get(key) != Some(value) {
                    return false;
                }
            }

            true
        });

        // Apply pagination
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(100) as usize;

        if offset < mock_logs.len() {
            let end = (offset + limit).min(mock_logs.len());
            mock_logs = mock_logs[offset..end].to_vec();
        } else {
            mock_logs.clear();
        }

        Ok(mock_logs)
    }

    async fn get_log_stats(
        &self,
        time_range: &crate::types::TimeRange,
    ) -> MonitoringResult<LogStatistics> {
        // Mock log statistics
        let mut logs_by_level = HashMap::new();
        logs_by_level.insert(LogLevel::Info, 1000);
        logs_by_level.insert(LogLevel::Warn, 50);
        logs_by_level.insert(LogLevel::Error, 10);
        logs_by_level.insert(LogLevel::Debug, 500);

        let mut logs_by_source = HashMap::new();
        logs_by_source.insert("app".to_string(), 800);
        logs_by_source.insert("system".to_string(), 400);
        logs_by_source.insert("database".to_string(), 360);

        let total_logs = logs_by_level.values().sum();

        let top_sources = vec![
            LogSourceStats {
                source: "app".to_string(),
                count: 800,
                error_count: 5,
                warning_count: 20,
            },
            LogSourceStats {
                source: "system".to_string(),
                count: 400,
                error_count: 3,
                warning_count: 15,
            },
            LogSourceStats {
                source: "database".to_string(),
                count: 360,
                error_count: 2,
                warning_count: 15,
            },
        ];

        Ok(LogStatistics {
            total_logs,
            logs_by_level,
            logs_by_source,
            error_rate: 0.64,   // (10/1560) * 100
            warning_rate: 3.21, // (50/1560) * 100
            top_sources,
            time_range: time_range.clone(),
        })
    }

    async fn search_logs(
        &self,
        search_term: &str,
        limit: Option<u32>,
    ) -> MonitoringResult<Vec<LogEntry>> {
        // Mock log search
        let mock_results = vec![LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now() - chrono::Duration::minutes(2),
            level: LogLevel::Error,
            message: format!("Error occurred: {}", search_term),
            source: "app".to_string(),
            labels: HashMap::new(),
            fields: HashMap::new(),
        }];

        let limit = limit.unwrap_or(100) as usize;
        Ok(mock_results.into_iter().take(limit).collect())
    }

    async fn get_log_levels_distribution(
        &self,
        time_range: &crate::types::TimeRange,
    ) -> MonitoringResult<HashMap<LogLevel, u64>> {
        let mut distribution = HashMap::new();
        distribution.insert(LogLevel::Trace, 100);
        distribution.insert(LogLevel::Debug, 500);
        distribution.insert(LogLevel::Info, 1000);
        distribution.insert(LogLevel::Warn, 50);
        distribution.insert(LogLevel::Error, 10);
        distribution.insert(LogLevel::Fatal, 1);

        Ok(distribution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_parser() {
        let parser = LogParser::new();

        let raw_log = "2024-01-15 10:30:45 [ERROR] Database connection failed";
        let parsed = parser.parse_log(raw_log);

        assert!(parsed.extracted_timestamp.is_some());
        assert_eq!(parsed.extracted_level, Some(LogLevel::Error));
        assert_eq!(
            parsed.extracted_message,
            Some("Database connection failed".to_string())
        );
    }

    #[test]
    fn test_log_indexer() {
        let mut indexer = LogIndexer::new();

        let log_entry = LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Error,
            message: "Database connection failed".to_string(),
            source: "app".to_string(),
            labels: HashMap::new(),
            fields: HashMap::new(),
        };

        indexer.index_log(&log_entry);

        let search_results = indexer.search_by_text("database");
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0], log_entry.id);

        let level_results = indexer.search_by_level(LogLevel::Error);
        assert_eq!(level_results.len(), 1);
        assert_eq!(level_results[0], log_entry.id);
    }

    #[tokio::test]
    async fn test_log_aggregator() {
        let config = LogConfig {
            enabled: true,
            log_level: LogLevel::Info,
            retention_days: 7,
            structured_logging: true,
        };

        let aggregator = LogAggregatorImpl::new(config);

        let log_entry = LogEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level: LogLevel::Info,
            message: "Test log message".to_string(),
            source: "test".to_string(),
            labels: HashMap::new(),
            fields: HashMap::new(),
        };

        let result = aggregator.store_log(&log_entry).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_query() {
        let config = LogConfig {
            enabled: true,
            log_level: LogLevel::Debug,
            retention_days: 7,
            structured_logging: true,
        };

        let aggregator = LogAggregatorImpl::new(config);

        let query = LogQuery {
            level: Some(LogLevel::Error),
            source: None,
            message_contains: Some("Database".to_string()),
            labels: HashMap::new(),
            start_time: Utc::now() - chrono::Duration::hours(1),
            end_time: Utc::now(),
            limit: Some(10),
            offset: Some(0),
        };

        let results = aggregator.query_logs(&query).await.unwrap();
        assert!(!results.is_empty());

        for log in &results {
            assert_eq!(log.level, LogLevel::Error);
            assert!(log.message.contains("Database"));
        }
    }
}
