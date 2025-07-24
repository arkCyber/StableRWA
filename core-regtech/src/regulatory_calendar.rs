// =====================================================================================
// File: core-regtech/src/regulatory_calendar.rs
// Description: Regulatory calendar and deadline management module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{RegTechError, RegTechResult},
    types::{Jurisdiction, ReportType},
};

/// Calendar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarConfig {
    pub jurisdictions: Vec<Jurisdiction>,
    pub auto_notifications: bool,
    pub notification_days_before: Vec<u32>,
    pub timezone: String,
}

/// Regulatory calendar trait
#[async_trait]
pub trait RegulatoryCalendar: Send + Sync {
    /// Get upcoming deadlines
    async fn get_upcoming_deadlines(
        &self,
        days_ahead: u32,
    ) -> RegTechResult<Vec<ComplianceDeadline>>;

    /// Add regulatory event
    async fn add_event(&self, event: &RegulatoryEvent) -> RegTechResult<Uuid>;

    /// Get regulatory updates
    async fn get_updates(&self, jurisdiction: Jurisdiction)
        -> RegTechResult<Vec<RegulatoryUpdate>>;
}

/// Regulatory event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryEvent {
    pub event_id: Uuid,
    pub event_type: EventType,
    pub title: String,
    pub description: String,
    pub jurisdiction: Jurisdiction,
    pub due_date: DateTime<Utc>,
    pub priority: Priority,
    pub status: EventStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    ReportingDeadline,
    RegulatoryUpdate,
    ComplianceReview,
    LicenseRenewal,
    AuditDeadline,
    TrainingRequirement,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Event status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventStatus {
    Scheduled,
    InProgress,
    Completed,
    Overdue,
    Cancelled,
}

/// Compliance deadline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceDeadline {
    pub deadline_id: Uuid,
    pub deadline_type: DeadlineType,
    pub title: String,
    pub description: String,
    pub jurisdiction: Jurisdiction,
    pub due_date: DateTime<Utc>,
    pub days_remaining: i64,
    pub priority: Priority,
    pub requirements: Vec<String>,
    pub related_reports: Vec<ReportType>,
}

/// Deadline types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeadlineType {
    ReportSubmission,
    LicenseRenewal,
    ComplianceReview,
    AuditSubmission,
    TrainingCompletion,
    PolicyUpdate,
}

/// Regulatory update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryUpdate {
    pub update_id: Uuid,
    pub title: String,
    pub description: String,
    pub jurisdiction: Jurisdiction,
    pub update_type: UpdateType,
    pub effective_date: DateTime<Utc>,
    pub impact_level: ImpactLevel,
    pub affected_areas: Vec<String>,
    pub action_required: bool,
    pub published_at: DateTime<Utc>,
}

/// Update types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpdateType {
    NewRegulation,
    Amendment,
    Guidance,
    Interpretation,
    Enforcement,
    Consultation,
}

/// Impact levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Regulatory calendar implementation
pub struct RegulatoryCalendarImpl {
    config: CalendarConfig,
    events: HashMap<Uuid, RegulatoryEvent>,
    deadlines: HashMap<Uuid, ComplianceDeadline>,
    updates: HashMap<Uuid, RegulatoryUpdate>,
}

impl RegulatoryCalendarImpl {
    pub fn new(config: CalendarConfig) -> Self {
        let mut calendar = Self {
            config,
            events: HashMap::new(),
            deadlines: HashMap::new(),
            updates: HashMap::new(),
        };

        calendar.initialize_default_events();
        calendar
    }

    fn initialize_default_events(&mut self) {
        // Add some default regulatory events
        let sar_deadline = ComplianceDeadline {
            deadline_id: Uuid::new_v4(),
            deadline_type: DeadlineType::ReportSubmission,
            title: "SAR Filing Deadline".to_string(),
            description: "Suspicious Activity Report filing deadline".to_string(),
            jurisdiction: Jurisdiction::US,
            due_date: Utc::now() + chrono::Duration::days(30),
            days_remaining: 30,
            priority: Priority::High,
            requirements: vec![
                "Complete SAR form".to_string(),
                "Gather supporting documentation".to_string(),
                "Review and approve".to_string(),
            ],
            related_reports: vec![ReportType::SAR],
        };

        self.deadlines
            .insert(sar_deadline.deadline_id, sar_deadline);

        // Add regulatory update
        let update = RegulatoryUpdate {
            update_id: Uuid::new_v4(),
            title: "New AML Guidelines".to_string(),
            description: "Updated anti-money laundering guidelines for financial institutions"
                .to_string(),
            jurisdiction: Jurisdiction::US,
            update_type: UpdateType::Guidance,
            effective_date: Utc::now() + chrono::Duration::days(60),
            impact_level: ImpactLevel::High,
            affected_areas: vec![
                "Transaction monitoring".to_string(),
                "Customer due diligence".to_string(),
            ],
            action_required: true,
            published_at: Utc::now(),
        };

        self.updates.insert(update.update_id, update);
    }

    fn calculate_days_remaining(&self, due_date: DateTime<Utc>) -> i64 {
        (due_date - Utc::now()).num_days()
    }
}

#[async_trait]
impl RegulatoryCalendar for RegulatoryCalendarImpl {
    async fn get_upcoming_deadlines(
        &self,
        days_ahead: u32,
    ) -> RegTechResult<Vec<ComplianceDeadline>> {
        let cutoff_date = Utc::now() + chrono::Duration::days(days_ahead as i64);

        let mut upcoming_deadlines: Vec<ComplianceDeadline> = self
            .deadlines
            .values()
            .filter(|deadline| deadline.due_date <= cutoff_date && deadline.due_date >= Utc::now())
            .cloned()
            .collect();

        // Update days remaining
        for deadline in &mut upcoming_deadlines {
            deadline.days_remaining = self.calculate_days_remaining(deadline.due_date);
        }

        // Sort by due date
        upcoming_deadlines.sort_by_key(|d| d.due_date);

        Ok(upcoming_deadlines)
    }

    async fn add_event(&self, event: &RegulatoryEvent) -> RegTechResult<Uuid> {
        // Mock event addition - in reality, this would persist to database
        Ok(event.event_id)
    }

    async fn get_updates(
        &self,
        jurisdiction: Jurisdiction,
    ) -> RegTechResult<Vec<RegulatoryUpdate>> {
        let updates: Vec<RegulatoryUpdate> = self
            .updates
            .values()
            .filter(|update| update.jurisdiction == jurisdiction)
            .cloned()
            .collect();

        Ok(updates)
    }
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            jurisdictions: vec![Jurisdiction::US, Jurisdiction::EU],
            auto_notifications: true,
            notification_days_before: vec![30, 14, 7, 1],
            timezone: "UTC".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upcoming_deadlines() {
        let config = CalendarConfig::default();
        let calendar = RegulatoryCalendarImpl::new(config);

        let deadlines = calendar.get_upcoming_deadlines(60).await.unwrap();
        assert!(!deadlines.is_empty());

        // Check that deadlines are sorted by due date
        for window in deadlines.windows(2) {
            assert!(window[0].due_date <= window[1].due_date);
        }
    }

    #[tokio::test]
    async fn test_regulatory_updates() {
        let config = CalendarConfig::default();
        let calendar = RegulatoryCalendarImpl::new(config);

        let updates = calendar.get_updates(Jurisdiction::US).await.unwrap();
        assert!(!updates.is_empty());

        // Check that all updates are for the requested jurisdiction
        for update in &updates {
            assert_eq!(update.jurisdiction, Jurisdiction::US);
        }
    }

    #[tokio::test]
    async fn test_add_event() {
        let config = CalendarConfig::default();
        let calendar = RegulatoryCalendarImpl::new(config);

        let event = RegulatoryEvent {
            event_id: Uuid::new_v4(),
            event_type: EventType::ReportingDeadline,
            title: "Test Event".to_string(),
            description: "Test regulatory event".to_string(),
            jurisdiction: Jurisdiction::US,
            due_date: Utc::now() + chrono::Duration::days(30),
            priority: Priority::Medium,
            status: EventStatus::Scheduled,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = calendar.add_event(&event).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), event.event_id);
    }
}
