// =====================================================================================
// File: core-regtech/src/audit_trail.rs
// Description: Audit trail and compliance logging module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{RegTechError, RegTechResult};

/// Audit trail configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub retention_days: u32,
    pub encryption_enabled: bool,
    pub real_time_logging: bool,
    pub log_levels: Vec<LogLevel>,
}

/// Audit trail service trait
#[async_trait]
pub trait AuditTrail: Send + Sync {
    /// Log audit event
    async fn log_event(&self, event: &AuditEvent) -> RegTechResult<Uuid>;

    /// Query audit logs
    async fn query_logs(&self, query: &AuditQuery) -> RegTechResult<Vec<AuditEvent>>;

    /// Generate audit report
    async fn generate_report(&self, period: &AuditPeriod) -> RegTechResult<AuditReport>;
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub entity_id: String,
    pub user_id: String,
    pub action: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub log_level: LogLevel,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    ComplianceCheck,
    KYCVerification,
    SanctionsScreening,
    AMLAlert,
    ReportGeneration,
    DocumentVerification,
    RiskAssessment,
    UserAction,
    SystemEvent,
    DataAccess,
    ConfigurationChange,
    SecurityEvent,
}

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub event_types: Option<Vec<EventType>>,
    pub entity_id: Option<String>,
    pub user_id: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub log_levels: Option<Vec<LogLevel>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Audit period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub period_type: PeriodType,
}

/// Period types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeriodType {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
    Custom,
}

/// Audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_id: Uuid,
    pub period: AuditPeriod,
    pub total_events: u32,
    pub events_by_type: HashMap<EventType, u32>,
    pub events_by_level: HashMap<LogLevel, u32>,
    pub top_users: Vec<UserActivity>,
    pub top_entities: Vec<EntityActivity>,
    pub security_events: u32,
    pub compliance_events: u32,
    pub generated_at: DateTime<Utc>,
}

/// User activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub user_id: String,
    pub event_count: u32,
    pub last_activity: DateTime<Utc>,
}

/// Entity activity summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityActivity {
    pub entity_id: String,
    pub event_count: u32,
    pub last_activity: DateTime<Utc>,
}

/// Audit trail implementation
pub struct AuditTrailImpl {
    config: AuditConfig,
    events: Vec<AuditEvent>,
}

impl AuditTrailImpl {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
        }
    }

    fn matches_query(&self, event: &AuditEvent, query: &AuditQuery) -> bool {
        // Check event types
        if let Some(ref event_types) = query.event_types {
            if !event_types.contains(&event.event_type) {
                return false;
            }
        }

        // Check entity ID
        if let Some(ref entity_id) = query.entity_id {
            if event.entity_id != *entity_id {
                return false;
            }
        }

        // Check user ID
        if let Some(ref user_id) = query.user_id {
            if event.user_id != *user_id {
                return false;
            }
        }

        // Check date range
        if let Some(start_date) = query.start_date {
            if event.timestamp < start_date {
                return false;
            }
        }

        if let Some(end_date) = query.end_date {
            if event.timestamp > end_date {
                return false;
            }
        }

        // Check log levels
        if let Some(ref log_levels) = query.log_levels {
            if !log_levels.contains(&event.log_level) {
                return false;
            }
        }

        true
    }

    fn generate_user_activities(&self, events: &[AuditEvent]) -> Vec<UserActivity> {
        let mut user_map: HashMap<String, (u32, DateTime<Utc>)> = HashMap::new();

        for event in events {
            let entry = user_map
                .entry(event.user_id.clone())
                .or_insert((0, event.timestamp));
            entry.0 += 1;
            if event.timestamp > entry.1 {
                entry.1 = event.timestamp;
            }
        }

        let mut activities: Vec<UserActivity> = user_map
            .into_iter()
            .map(|(user_id, (count, last_activity))| UserActivity {
                user_id,
                event_count: count,
                last_activity,
            })
            .collect();

        activities.sort_by(|a, b| b.event_count.cmp(&a.event_count));
        activities.truncate(10); // Top 10 users

        activities
    }

    fn generate_entity_activities(&self, events: &[AuditEvent]) -> Vec<EntityActivity> {
        let mut entity_map: HashMap<String, (u32, DateTime<Utc>)> = HashMap::new();

        for event in events {
            let entry = entity_map
                .entry(event.entity_id.clone())
                .or_insert((0, event.timestamp));
            entry.0 += 1;
            if event.timestamp > entry.1 {
                entry.1 = event.timestamp;
            }
        }

        let mut activities: Vec<EntityActivity> = entity_map
            .into_iter()
            .map(|(entity_id, (count, last_activity))| EntityActivity {
                entity_id,
                event_count: count,
                last_activity,
            })
            .collect();

        activities.sort_by(|a, b| b.event_count.cmp(&a.event_count));
        activities.truncate(10); // Top 10 entities

        activities
    }
}

#[async_trait]
impl AuditTrail for AuditTrailImpl {
    async fn log_event(&self, event: &AuditEvent) -> RegTechResult<Uuid> {
        // Mock event logging - in reality, this would persist to database
        Ok(event.event_id)
    }

    async fn query_logs(&self, query: &AuditQuery) -> RegTechResult<Vec<AuditEvent>> {
        let mut matching_events: Vec<AuditEvent> = self
            .events
            .iter()
            .filter(|event| self.matches_query(event, query))
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        matching_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(100) as usize;

        if offset < matching_events.len() {
            let end = (offset + limit).min(matching_events.len());
            matching_events = matching_events[offset..end].to_vec();
        } else {
            matching_events.clear();
        }

        Ok(matching_events)
    }

    async fn generate_report(&self, period: &AuditPeriod) -> RegTechResult<AuditReport> {
        let period_events: Vec<AuditEvent> = self
            .events
            .iter()
            .filter(|event| {
                event.timestamp >= period.start_date && event.timestamp <= period.end_date
            })
            .cloned()
            .collect();

        let total_events = period_events.len() as u32;

        // Count events by type
        let mut events_by_type: HashMap<EventType, u32> = HashMap::new();
        for event in &period_events {
            *events_by_type.entry(event.event_type).or_insert(0) += 1;
        }

        // Count events by level
        let mut events_by_level: HashMap<LogLevel, u32> = HashMap::new();
        for event in &period_events {
            *events_by_level.entry(event.log_level).or_insert(0) += 1;
        }

        let top_users = self.generate_user_activities(&period_events);
        let top_entities = self.generate_entity_activities(&period_events);

        let security_events = period_events
            .iter()
            .filter(|event| event.event_type == EventType::SecurityEvent)
            .count() as u32;

        let compliance_events = period_events
            .iter()
            .filter(|event| {
                matches!(
                    event.event_type,
                    EventType::ComplianceCheck
                        | EventType::KYCVerification
                        | EventType::SanctionsScreening
                )
            })
            .count() as u32;

        Ok(AuditReport {
            report_id: Uuid::new_v4(),
            period: period.clone(),
            total_events,
            events_by_type,
            events_by_level,
            top_users,
            top_entities,
            security_events,
            compliance_events,
            generated_at: Utc::now(),
        })
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            retention_days: 2555, // 7 years
            encryption_enabled: true,
            real_time_logging: true,
            log_levels: vec![
                LogLevel::Info,
                LogLevel::Warning,
                LogLevel::Error,
                LogLevel::Critical,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_event() {
        let config = AuditConfig::default();
        let audit_trail = AuditTrailImpl::new(config);

        let event = AuditEvent {
            event_id: Uuid::new_v4(),
            event_type: EventType::ComplianceCheck,
            entity_id: "entity_123".to_string(),
            user_id: "user_456".to_string(),
            action: "perform_compliance_check".to_string(),
            description: "Performed compliance check for entity".to_string(),
            metadata: HashMap::new(),
            log_level: LogLevel::Info,
            timestamp: Utc::now(),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("RegTech-Client/1.0".to_string()),
        };

        let result = audit_trail.log_event(&event).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), event.event_id);
    }

    #[tokio::test]
    async fn test_query_logs() {
        let config = AuditConfig::default();
        let audit_trail = AuditTrailImpl::new(config);

        let query = AuditQuery {
            event_types: Some(vec![EventType::ComplianceCheck]),
            entity_id: Some("entity_123".to_string()),
            user_id: None,
            start_date: Some(Utc::now() - chrono::Duration::days(7)),
            end_date: Some(Utc::now()),
            log_levels: None,
            limit: Some(10),
            offset: Some(0),
        };

        let result = audit_trail.query_logs(&query).await;
        assert!(result.is_ok());

        let logs = result.unwrap();
        // Since we haven't added any events to the mock implementation,
        // the result should be empty
        assert!(logs.is_empty());
    }

    #[tokio::test]
    async fn test_generate_report() {
        let config = AuditConfig::default();
        let audit_trail = AuditTrailImpl::new(config);

        let period = AuditPeriod {
            start_date: Utc::now() - chrono::Duration::days(30),
            end_date: Utc::now(),
            period_type: PeriodType::Monthly,
        };

        let result = audit_trail.generate_report(&period).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.period.period_type, PeriodType::Monthly);
        assert_eq!(report.total_events, 0); // No events in mock implementation
    }
}
