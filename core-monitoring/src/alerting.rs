// =====================================================================================
// File: core-monitoring/src/alerting.rs
// Description: Alert management and processing module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{MonitoringError, MonitoringResult},
    types::{
        Alert, AlertChannel, AlertChannelConfig, AlertChannelType, AlertCondition, AlertConfig,
        AlertRule, AlertSeverity, AlertStatus, ComparisonOperator, EscalationRule, SuppressionRule,
    },
};

/// Alert manager trait
#[async_trait]
pub trait AlertManager: Send + Sync {
    /// Process alert rules and generate alerts
    async fn process_alerts(&self) -> MonitoringResult<Vec<Alert>>;

    /// Send alert notifications
    async fn send_alert(&self, alert: &Alert) -> MonitoringResult<()>;

    /// Acknowledge alert
    async fn acknowledge_alert(
        &self,
        alert_id: &Uuid,
        acknowledged_by: &str,
    ) -> MonitoringResult<()>;

    /// Resolve alert
    async fn resolve_alert(&self, alert_id: &Uuid, resolved_by: &str) -> MonitoringResult<()>;

    /// Get active alerts
    async fn get_active_alerts(&self) -> MonitoringResult<Vec<Alert>>;

    /// Add alert rule
    async fn add_alert_rule(&self, rule: &AlertRule) -> MonitoringResult<Uuid>;

    /// Remove alert rule
    async fn remove_alert_rule(&self, rule_id: &Uuid) -> MonitoringResult<()>;
}

/// Alert processor for evaluating conditions
pub struct AlertProcessor {
    config: AlertConfig,
}

/// Alert escalation manager
pub struct AlertEscalation {
    escalation_rules: Vec<EscalationRule>,
}

/// Alert suppression manager
pub struct AlertSuppression {
    suppression_rules: Vec<SuppressionRule>,
}

/// Alert notification dispatcher
pub struct AlertNotificationDispatcher {
    channels: HashMap<Uuid, AlertChannel>,
}

/// Main alert manager implementation
pub struct AlertManagerImpl {
    config: AlertConfig,
    processor: AlertProcessor,
    escalation: AlertEscalation,
    suppression: AlertSuppression,
    dispatcher: AlertNotificationDispatcher,
    active_alerts: HashMap<Uuid, Alert>,
    alert_rules: HashMap<Uuid, AlertRule>,
}

impl AlertProcessor {
    pub fn new(config: AlertConfig) -> Self {
        Self { config }
    }

    pub fn evaluate_condition(&self, condition: &AlertCondition, value: f64) -> bool {
        match condition {
            AlertCondition::Threshold {
                operator,
                value: threshold,
            } => match operator {
                ComparisonOperator::GreaterThan => value > *threshold,
                ComparisonOperator::GreaterThanOrEqual => value >= *threshold,
                ComparisonOperator::LessThan => value < *threshold,
                ComparisonOperator::LessThanOrEqual => value <= *threshold,
                ComparisonOperator::Equal => (value - threshold).abs() < f64::EPSILON,
                ComparisonOperator::NotEqual => (value - threshold).abs() >= f64::EPSILON,
            },
            AlertCondition::Range { min, max } => value >= *min && value <= *max,
            AlertCondition::Anomaly { sensitivity: _ } => {
                // Mock anomaly detection - in reality, this would use ML models
                value > 100.0 // Simple threshold for demo
            }
            AlertCondition::Custom { expression: _ } => {
                // Mock custom expression evaluation
                true
            }
        }
    }

    pub fn should_fire_alert(&self, rule: &AlertRule, current_value: f64) -> bool {
        if !rule.enabled {
            return false;
        }

        self.evaluate_condition(&rule.condition, current_value)
    }
}

impl AlertEscalation {
    pub fn new(escalation_rules: Vec<EscalationRule>) -> Self {
        Self { escalation_rules }
    }

    pub async fn check_escalation(&self, alert: &Alert) -> MonitoringResult<bool> {
        let alert_age = Utc::now() - alert.starts_at;

        for rule in &self.escalation_rules {
            // Check if escalation conditions are met
            for condition in &rule.conditions {
                match condition {
                    crate::types::EscalationCondition::Severity(severity) => {
                        if alert.severity >= *severity {
                            return Ok(true);
                        }
                    }
                    crate::types::EscalationCondition::Duration(duration) => {
                        if alert_age >= *duration {
                            return Ok(true);
                        }
                    }
                    crate::types::EscalationCondition::UnacknowledgedFor(duration) => {
                        if alert.status != AlertStatus::Acknowledged && alert_age >= *duration {
                            return Ok(true);
                        }
                    }
                    crate::types::EscalationCondition::Label { key, value } => {
                        if alert.labels.get(key) == Some(value) {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}

impl AlertSuppression {
    pub fn new(suppression_rules: Vec<SuppressionRule>) -> Self {
        Self { suppression_rules }
    }

    pub fn should_suppress_alert(&self, alert: &Alert) -> bool {
        for rule in &self.suppression_rules {
            if !rule.enabled {
                continue;
            }

            let mut all_conditions_met = true;

            for condition in &rule.conditions {
                match condition {
                    crate::types::SuppressionCondition::Label { key, value } => {
                        if alert.labels.get(key) != Some(value) {
                            all_conditions_met = false;
                            break;
                        }
                    }
                    crate::types::SuppressionCondition::TimeWindow { start: _, end: _ } => {
                        // Mock time window check
                        // In reality, this would check if current time is within the window
                    }
                    crate::types::SuppressionCondition::Maintenance { system: _ } => {
                        // Mock maintenance window check
                        // In reality, this would check if the system is in maintenance
                    }
                }
            }

            if all_conditions_met {
                return true;
            }
        }

        false
    }
}

impl AlertNotificationDispatcher {
    pub fn new(channels: Vec<AlertChannel>) -> Self {
        let channel_map: HashMap<Uuid, AlertChannel> = channels
            .into_iter()
            .map(|channel| (channel.id, channel))
            .collect();

        Self {
            channels: channel_map,
        }
    }

    pub async fn send_notification(
        &self,
        alert: &Alert,
        channel_id: &Uuid,
    ) -> MonitoringResult<()> {
        let channel = self
            .channels
            .get(channel_id)
            .ok_or_else(|| MonitoringError::alert_error("Alert channel not found"))?;

        if !channel.enabled {
            return Ok(());
        }

        match &channel.config {
            AlertChannelConfig::Email { recipients } => {
                self.send_email_notification(alert, recipients).await
            }
            AlertChannelConfig::Slack {
                webhook_url,
                channel: slack_channel,
            } => {
                self.send_slack_notification(alert, webhook_url, slack_channel)
                    .await
            }
            AlertChannelConfig::Discord { webhook_url } => {
                self.send_discord_notification(alert, webhook_url).await
            }
            AlertChannelConfig::Webhook { url, headers } => {
                self.send_webhook_notification(alert, url, headers).await
            }
            AlertChannelConfig::SMS { phone_numbers } => {
                self.send_sms_notification(alert, phone_numbers).await
            }
            AlertChannelConfig::PagerDuty { integration_key } => {
                self.send_pagerduty_notification(alert, integration_key)
                    .await
            }
            AlertChannelConfig::OpsGenie { api_key, team } => {
                self.send_opsgenie_notification(alert, api_key, team).await
            }
        }
    }

    async fn send_email_notification(
        &self,
        alert: &Alert,
        recipients: &[String],
    ) -> MonitoringResult<()> {
        // Mock email sending
        println!("Sending email alert '{}' to {:?}", alert.name, recipients);
        Ok(())
    }

    async fn send_slack_notification(
        &self,
        alert: &Alert,
        webhook_url: &str,
        channel: &str,
    ) -> MonitoringResult<()> {
        // Mock Slack notification
        println!(
            "Sending Slack alert '{}' to {} via {}",
            alert.name, channel, webhook_url
        );
        Ok(())
    }

    async fn send_discord_notification(
        &self,
        alert: &Alert,
        webhook_url: &str,
    ) -> MonitoringResult<()> {
        // Mock Discord notification
        println!("Sending Discord alert '{}' via {}", alert.name, webhook_url);
        Ok(())
    }

    async fn send_webhook_notification(
        &self,
        alert: &Alert,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> MonitoringResult<()> {
        // Mock webhook notification
        println!(
            "Sending webhook alert '{}' to {} with headers {:?}",
            alert.name, url, headers
        );
        Ok(())
    }

    async fn send_sms_notification(
        &self,
        alert: &Alert,
        phone_numbers: &[String],
    ) -> MonitoringResult<()> {
        // Mock SMS sending
        println!("Sending SMS alert '{}' to {:?}", alert.name, phone_numbers);
        Ok(())
    }

    async fn send_pagerduty_notification(
        &self,
        alert: &Alert,
        integration_key: &str,
    ) -> MonitoringResult<()> {
        // Mock PagerDuty notification
        println!(
            "Sending PagerDuty alert '{}' with key {}",
            alert.name, integration_key
        );
        Ok(())
    }

    async fn send_opsgenie_notification(
        &self,
        alert: &Alert,
        api_key: &str,
        team: &str,
    ) -> MonitoringResult<()> {
        // Mock OpsGenie notification
        println!(
            "Sending OpsGenie alert '{}' to team {} with key {}",
            alert.name, team, api_key
        );
        Ok(())
    }
}

impl AlertManagerImpl {
    pub fn new(config: AlertConfig) -> Self {
        let processor = AlertProcessor::new(config.clone());
        let escalation = AlertEscalation::new(config.escalation_rules.clone());
        let suppression = AlertSuppression::new(config.suppression_rules.clone());
        let dispatcher = AlertNotificationDispatcher::new(config.channels.clone());

        Self {
            config,
            processor,
            escalation,
            suppression,
            dispatcher,
            active_alerts: HashMap::new(),
            alert_rules: HashMap::new(),
        }
    }

    fn create_alert_from_rule(&self, rule: &AlertRule) -> Alert {
        Alert {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            name: rule.name.clone(),
            description: format!("Alert triggered by rule: {}", rule.name),
            severity: rule.severity,
            status: AlertStatus::Firing,
            labels: rule.labels.clone(),
            annotations: rule.annotations.clone(),
            starts_at: Utc::now(),
            ends_at: None,
            generator_url: None,
        }
    }
}

#[async_trait]
impl AlertManager for AlertManagerImpl {
    async fn process_alerts(&self) -> MonitoringResult<Vec<Alert>> {
        let mut new_alerts = Vec::new();

        for rule in self.alert_rules.values() {
            // Mock metric value - in reality, this would query actual metrics
            let current_value = 75.0;

            if self.processor.should_fire_alert(rule, current_value) {
                let alert = self.create_alert_from_rule(rule);

                // Check if alert should be suppressed
                if !self.suppression.should_suppress_alert(&alert) {
                    new_alerts.push(alert);
                }
            }
        }

        Ok(new_alerts)
    }

    async fn send_alert(&self, alert: &Alert) -> MonitoringResult<()> {
        // Send to all configured channels
        for channel in &self.config.channels {
            if let Err(e) = self.dispatcher.send_notification(alert, &channel.id).await {
                eprintln!("Failed to send alert to channel {}: {}", channel.name, e);
            }
        }

        Ok(())
    }

    async fn acknowledge_alert(
        &self,
        alert_id: &Uuid,
        acknowledged_by: &str,
    ) -> MonitoringResult<()> {
        // Mock alert acknowledgment
        println!("Alert {} acknowledged by {}", alert_id, acknowledged_by);
        Ok(())
    }

    async fn resolve_alert(&self, alert_id: &Uuid, resolved_by: &str) -> MonitoringResult<()> {
        // Mock alert resolution
        println!("Alert {} resolved by {}", alert_id, resolved_by);
        Ok(())
    }

    async fn get_active_alerts(&self) -> MonitoringResult<Vec<Alert>> {
        let active_alerts: Vec<Alert> = self
            .active_alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Firing)
            .cloned()
            .collect();

        Ok(active_alerts)
    }

    async fn add_alert_rule(&self, rule: &AlertRule) -> MonitoringResult<Uuid> {
        // Mock rule addition
        Ok(rule.id)
    }

    async fn remove_alert_rule(&self, rule_id: &Uuid) -> MonitoringResult<()> {
        // Mock rule removal
        println!("Removed alert rule {}", rule_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_processor_threshold_condition() {
        let config = AlertConfig {
            enabled: true,
            rules: Vec::new(),
            channels: Vec::new(),
            escalation_rules: Vec::new(),
            suppression_rules: Vec::new(),
        };

        let processor = AlertProcessor::new(config);

        let condition = AlertCondition::Threshold {
            operator: ComparisonOperator::GreaterThan,
            value: 80.0,
        };

        assert!(processor.evaluate_condition(&condition, 90.0));
        assert!(!processor.evaluate_condition(&condition, 70.0));
    }

    #[test]
    fn test_alert_processor_range_condition() {
        let config = AlertConfig {
            enabled: true,
            rules: Vec::new(),
            channels: Vec::new(),
            escalation_rules: Vec::new(),
            suppression_rules: Vec::new(),
        };

        let processor = AlertProcessor::new(config);

        let condition = AlertCondition::Range {
            min: 10.0,
            max: 90.0,
        };

        assert!(processor.evaluate_condition(&condition, 50.0));
        assert!(!processor.evaluate_condition(&condition, 95.0));
        assert!(!processor.evaluate_condition(&condition, 5.0));
    }

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let config = AlertConfig {
            enabled: true,
            rules: Vec::new(),
            channels: Vec::new(),
            escalation_rules: Vec::new(),
            suppression_rules: Vec::new(),
        };

        let alert_manager = AlertManagerImpl::new(config);
        let alerts = alert_manager.process_alerts().await.unwrap();

        // Should be empty since no rules are configured
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_alert_notification_dispatcher() {
        let channel = AlertChannel {
            id: Uuid::new_v4(),
            name: "test-email".to_string(),
            channel_type: AlertChannelType::Email,
            config: AlertChannelConfig::Email {
                recipients: vec!["test@example.com".to_string()],
            },
            enabled: true,
        };

        let dispatcher = AlertNotificationDispatcher::new(vec![channel.clone()]);

        let alert = Alert {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(),
            name: "Test Alert".to_string(),
            description: "Test alert description".to_string(),
            severity: AlertSeverity::Warning,
            status: AlertStatus::Firing,
            labels: HashMap::new(),
            annotations: HashMap::new(),
            starts_at: Utc::now(),
            ends_at: None,
            generator_url: None,
        };

        let result = dispatcher.send_notification(&alert, &channel.id).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_alert_suppression() {
        let suppression_rule = SuppressionRule {
            id: Uuid::new_v4(),
            name: "Maintenance suppression".to_string(),
            conditions: vec![crate::types::SuppressionCondition::Label {
                key: "environment".to_string(),
                value: "maintenance".to_string(),
            }],
            duration: chrono::Duration::hours(1),
            enabled: true,
        };

        let suppression = AlertSuppression::new(vec![suppression_rule]);

        let mut labels = HashMap::new();
        labels.insert("environment".to_string(), "maintenance".to_string());

        let alert = Alert {
            id: Uuid::new_v4(),
            rule_id: Uuid::new_v4(),
            name: "Test Alert".to_string(),
            description: "Test alert description".to_string(),
            severity: AlertSeverity::Warning,
            status: AlertStatus::Firing,
            labels,
            annotations: HashMap::new(),
            starts_at: Utc::now(),
            ends_at: None,
            generator_url: None,
        };

        assert!(suppression.should_suppress_alert(&alert));
    }
}
