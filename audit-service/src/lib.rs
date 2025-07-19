// =====================================================================================
// File: audit-service/src/lib.rs
// Description: Core audit logic for enterprise-grade audit microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{Utc, DateTime};
use log::{info, error};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: u64,
    pub event_type: String,
    pub actor: String,
    pub target: String,
    pub description: String,
    pub timestamp: String,
    pub compliance_checked: bool,
    pub compliant: bool,
}

/// Audit error type
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Event type is empty")]
    EmptyEventType,
    #[error("Actor is empty")]
    EmptyActor,
    #[error("Target is empty")]
    EmptyTarget,
    #[error("Audit event not found")]
    NotFound,
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Audit event store (in-memory, for demo; replace with DB in production)
#[derive(Default, Clone)]
pub struct AuditStore {
    pub events: Arc<Mutex<VecDeque<AuditEvent>>>,
    pub next_id: Arc<Mutex<u64>>,
}

impl AuditStore {
    /// Creates a new audit store
    pub fn new() -> Self {
        Self::default()
    }

    /// Logs a new audit event
    pub fn log_event(&self, event_type: String, actor: String, target: String, description: String) -> Result<AuditEvent, AuditError> {
        let now = Utc::now();
        if event_type.trim().is_empty() {
            error!("{} - [AuditStore] Log event failed: event_type empty", now);
            return Err(AuditError::EmptyEventType);
        }
        if actor.trim().is_empty() {
            error!("{} - [AuditStore] Log event failed: actor empty", now);
            return Err(AuditError::EmptyActor);
        }
        if target.trim().is_empty() {
            error!("{} - [AuditStore] Log event failed: target empty", now);
            return Err(AuditError::EmptyTarget);
        }
        let mut id_lock = self.next_id.lock().unwrap();
        let id = *id_lock;
        *id_lock += 1;
        let event = AuditEvent {
            id,
            event_type: event_type.clone(),
            actor: actor.clone(),
            target: target.clone(),
            description: description.clone(),
            timestamp: now.to_rfc3339(),
            compliance_checked: false,
            compliant: false,
        };
        self.events.lock().unwrap().push_back(event.clone());
        info!("{} - [AuditStore] Logged event id={} type={}", now, id, event_type);
        Ok(event)
    }

    /// Lists all audit events
    pub fn list_events(&self) -> Vec<AuditEvent> {
        self.events.lock().unwrap().iter().cloned().collect()
    }

    /// Runs a compliance check on an event by id
    pub fn compliance_check(&self, id: u64, rule: fn(&AuditEvent) -> bool) -> Result<AuditEvent, AuditError> {
        let now = Utc::now();
        let mut events = self.events.lock().unwrap();
        let event = events.iter_mut().find(|e| e.id == id).ok_or(AuditError::NotFound)?;
        event.compliance_checked = true;
        event.compliant = rule(event);
        info!("{} - [AuditStore] Compliance checked id={} result={}", now, id, event.compliant);
        Ok(event.clone())
    }

    /// Generates a simple audit report (all events, compliance summary)
    pub fn generate_report(&self) -> String {
        let events = self.events.lock().unwrap();
        let total = events.len();
        let compliant = events.iter().filter(|e| e.compliance_checked && e.compliant).count();
        let non_compliant = events.iter().filter(|e| e.compliance_checked && !e.compliant).count();
        format!(
            "Audit Report\nTotal Events: {}\nCompliant: {}\nNon-compliant: {}\n",
            total, compliant, non_compliant
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Info).try_init();
    }

    #[test]
    fn test_log_and_list_event() {
        init_logger();
        let store = AuditStore::new();
        let e = store.log_event("login".to_string(), "user1".to_string(), "system".to_string(), "User login").unwrap();
        let list = store.list_events();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, e.id);
    }

    #[test]
    fn test_compliance_and_report() {
        init_logger();
        let store = AuditStore::new();
        let e = store.log_event("transfer".to_string(), "user2".to_string(), "asset1".to_string(), "Transfer asset").unwrap();
        let rule = |event: &AuditEvent| event.event_type == "transfer" && event.actor == "user2";
        let checked = store.compliance_check(e.id, rule).unwrap();
        assert!(checked.compliance_checked);
        assert!(checked.compliant);
        let report = store.generate_report();
        assert!(report.contains("Compliant: 1"));
    }
} 