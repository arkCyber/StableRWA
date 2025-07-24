// =====================================================================================
// File: core-monitoring/src/health_checks.rs
// Description: Health check and service monitoring module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{MonitoringError, MonitoringResult},
    types::{HealthCheck, HealthCheckConfig, HealthCheckDefinition, HealthCheckType, HealthStatus},
};

/// Health check service trait
#[async_trait]
pub trait HealthCheckService: Send + Sync {
    /// Register a new health check
    async fn register_check(&self, definition: &HealthCheckDefinition) -> MonitoringResult<Uuid>;

    /// Unregister a health check
    async fn unregister_check(&self, check_id: &Uuid) -> MonitoringResult<()>;

    /// Perform a single health check
    async fn perform_check(&self, check_id: &Uuid) -> MonitoringResult<HealthCheck>;

    /// Perform all registered health checks
    async fn perform_all_checks(&self) -> MonitoringResult<Vec<HealthCheck>>;

    /// Get health check history
    async fn get_check_history(
        &self,
        check_id: &Uuid,
        limit: Option<u32>,
    ) -> MonitoringResult<Vec<HealthCheck>>;

    /// Get overall system health status
    async fn get_system_health(&self) -> MonitoringResult<SystemHealthStatus>;
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub overall_status: HealthStatus,
    pub healthy_checks: u32,
    pub unhealthy_checks: u32,
    pub degraded_checks: u32,
    pub unknown_checks: u32,
    pub total_checks: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub details: Vec<HealthCheckSummary>,
}

/// Health check summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSummary {
    pub check_id: Uuid,
    pub name: String,
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time: chrono::Duration,
    pub uptime_percentage: f64,
}

/// HTTP health checker
pub struct HTTPHealthChecker {
    client: reqwest::Client,
}

/// TCP health checker
pub struct TCPHealthChecker;

/// Database health checker
pub struct DatabaseHealthChecker;

/// Custom health checker
pub struct CustomHealthChecker;

/// Health check service implementation
pub struct HealthCheckServiceImpl {
    config: HealthCheckConfig,
    definitions: HashMap<Uuid, HealthCheckDefinition>,
    check_history: HashMap<Uuid, Vec<HealthCheck>>,
    http_checker: HTTPHealthChecker,
    tcp_checker: TCPHealthChecker,
    database_checker: DatabaseHealthChecker,
    custom_checker: CustomHealthChecker,
}

impl HTTPHealthChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn check(
        &self,
        target: &str,
        timeout: chrono::Duration,
    ) -> MonitoringResult<HealthCheck> {
        let start_time = std::time::Instant::now();

        // Mock HTTP health check
        let status = if target.contains("unhealthy") {
            HealthStatus::Unhealthy
        } else if target.contains("degraded") {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let response_time = chrono::Duration::from_std(start_time.elapsed())
            .unwrap_or(chrono::Duration::milliseconds(0));

        let message = match status {
            HealthStatus::Healthy => "HTTP endpoint is responding normally".to_string(),
            HealthStatus::Unhealthy => "HTTP endpoint is not responding".to_string(),
            HealthStatus::Degraded => "HTTP endpoint is responding slowly".to_string(),
            HealthStatus::Unknown => "Unable to determine HTTP endpoint status".to_string(),
        };

        Ok(HealthCheck {
            id: Uuid::new_v4(),
            definition_id: Uuid::new_v4(), // This would be set by the caller
            status,
            response_time,
            message,
            checked_at: Utc::now(),
        })
    }
}

impl TCPHealthChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check(
        &self,
        target: &str,
        timeout: chrono::Duration,
    ) -> MonitoringResult<HealthCheck> {
        let start_time = std::time::Instant::now();

        // Mock TCP health check
        let status = if target.contains("closed") {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Healthy
        };

        let response_time = chrono::Duration::from_std(start_time.elapsed())
            .unwrap_or(chrono::Duration::milliseconds(0));

        let message = match status {
            HealthStatus::Healthy => "TCP port is open and accepting connections".to_string(),
            HealthStatus::Unhealthy => "TCP port is closed or not responding".to_string(),
            _ => "TCP port status unknown".to_string(),
        };

        Ok(HealthCheck {
            id: Uuid::new_v4(),
            definition_id: Uuid::new_v4(),
            status,
            response_time,
            message,
            checked_at: Utc::now(),
        })
    }
}

impl DatabaseHealthChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check(
        &self,
        target: &str,
        timeout: chrono::Duration,
    ) -> MonitoringResult<HealthCheck> {
        let start_time = std::time::Instant::now();

        // Mock database health check
        let status = if target.contains("offline") {
            HealthStatus::Unhealthy
        } else if target.contains("slow") {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        let response_time = chrono::Duration::from_std(start_time.elapsed())
            .unwrap_or(chrono::Duration::milliseconds(0));

        let message = match status {
            HealthStatus::Healthy => "Database is responding to queries normally".to_string(),
            HealthStatus::Unhealthy => "Database is not responding or offline".to_string(),
            HealthStatus::Degraded => "Database is responding but with high latency".to_string(),
            HealthStatus::Unknown => "Unable to determine database status".to_string(),
        };

        Ok(HealthCheck {
            id: Uuid::new_v4(),
            definition_id: Uuid::new_v4(),
            status,
            response_time,
            message,
            checked_at: Utc::now(),
        })
    }
}

impl CustomHealthChecker {
    pub fn new() -> Self {
        Self
    }

    pub async fn check(
        &self,
        target: &str,
        timeout: chrono::Duration,
    ) -> MonitoringResult<HealthCheck> {
        let start_time = std::time::Instant::now();

        // Mock custom health check
        let status = HealthStatus::Healthy;

        let response_time = chrono::Duration::from_std(start_time.elapsed())
            .unwrap_or(chrono::Duration::milliseconds(0));

        Ok(HealthCheck {
            id: Uuid::new_v4(),
            definition_id: Uuid::new_v4(),
            status,
            response_time,
            message: "Custom health check completed successfully".to_string(),
            checked_at: Utc::now(),
        })
    }
}

impl HealthCheckServiceImpl {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            definitions: HashMap::new(),
            check_history: HashMap::new(),
            http_checker: HTTPHealthChecker::new(),
            tcp_checker: TCPHealthChecker::new(),
            database_checker: DatabaseHealthChecker::new(),
            custom_checker: CustomHealthChecker::new(),
        }
    }

    async fn execute_check(
        &self,
        definition: &HealthCheckDefinition,
    ) -> MonitoringResult<HealthCheck> {
        let mut check = match definition.check_type {
            HealthCheckType::HTTP => {
                self.http_checker
                    .check(&definition.target, definition.timeout)
                    .await?
            }
            HealthCheckType::TCP => {
                self.tcp_checker
                    .check(&definition.target, definition.timeout)
                    .await?
            }
            HealthCheckType::Database => {
                self.database_checker
                    .check(&definition.target, definition.timeout)
                    .await?
            }
            HealthCheckType::Custom => {
                self.custom_checker
                    .check(&definition.target, definition.timeout)
                    .await?
            }
        };

        // Set the correct definition ID
        check.definition_id = definition.id;

        Ok(check)
    }

    fn calculate_uptime_percentage(&self, check_id: &Uuid) -> f64 {
        if let Some(history) = self.check_history.get(check_id) {
            if history.is_empty() {
                return 100.0;
            }

            let healthy_count = history
                .iter()
                .filter(|check| check.status == HealthStatus::Healthy)
                .count();

            (healthy_count as f64 / history.len() as f64) * 100.0
        } else {
            100.0
        }
    }

    fn determine_overall_status(&self, checks: &[HealthCheck]) -> HealthStatus {
        if checks.is_empty() {
            return HealthStatus::Unknown;
        }

        let unhealthy_count = checks
            .iter()
            .filter(|check| check.status == HealthStatus::Unhealthy)
            .count();

        let degraded_count = checks
            .iter()
            .filter(|check| check.status == HealthStatus::Degraded)
            .count();

        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

#[async_trait]
impl HealthCheckService for HealthCheckServiceImpl {
    async fn register_check(&self, definition: &HealthCheckDefinition) -> MonitoringResult<Uuid> {
        // Mock check registration
        Ok(definition.id)
    }

    async fn unregister_check(&self, check_id: &Uuid) -> MonitoringResult<()> {
        // Mock check unregistration
        Ok(())
    }

    async fn perform_check(&self, check_id: &Uuid) -> MonitoringResult<HealthCheck> {
        // Create a mock definition for demo
        let definition = HealthCheckDefinition {
            id: *check_id,
            name: "Mock Health Check".to_string(),
            check_type: HealthCheckType::HTTP,
            target: "http://localhost:8080/health".to_string(),
            timeout: chrono::Duration::seconds(5),
            interval: chrono::Duration::seconds(30),
            retries: 3,
        };

        self.execute_check(&definition).await
    }

    async fn perform_all_checks(&self) -> MonitoringResult<Vec<HealthCheck>> {
        // Mock performing all checks
        let mut checks = Vec::new();

        // Create some sample health checks
        let definitions = vec![
            HealthCheckDefinition {
                id: Uuid::new_v4(),
                name: "API Gateway".to_string(),
                check_type: HealthCheckType::HTTP,
                target: "http://localhost:8080/health".to_string(),
                timeout: chrono::Duration::seconds(5),
                interval: chrono::Duration::seconds(30),
                retries: 3,
            },
            HealthCheckDefinition {
                id: Uuid::new_v4(),
                name: "Database".to_string(),
                check_type: HealthCheckType::Database,
                target: "postgresql://localhost:5432/app".to_string(),
                timeout: chrono::Duration::seconds(10),
                interval: chrono::Duration::seconds(60),
                retries: 2,
            },
            HealthCheckDefinition {
                id: Uuid::new_v4(),
                name: "Redis Cache".to_string(),
                check_type: HealthCheckType::TCP,
                target: "localhost:6379".to_string(),
                timeout: chrono::Duration::seconds(3),
                interval: chrono::Duration::seconds(30),
                retries: 3,
            },
        ];

        for definition in definitions {
            let check = self.execute_check(&definition).await?;
            checks.push(check);
        }

        Ok(checks)
    }

    async fn get_check_history(
        &self,
        check_id: &Uuid,
        limit: Option<u32>,
    ) -> MonitoringResult<Vec<HealthCheck>> {
        // Mock check history retrieval
        let mut history = Vec::new();
        let limit = limit.unwrap_or(100) as usize;

        // Generate some mock history
        for i in 0..limit.min(10) {
            let check = HealthCheck {
                id: Uuid::new_v4(),
                definition_id: *check_id,
                status: if i % 10 == 0 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                },
                response_time: chrono::Duration::milliseconds(50 + (i as i64 * 10)),
                message: format!("Health check #{} completed", i + 1),
                checked_at: Utc::now() - chrono::Duration::minutes(i as i64 * 5),
            };
            history.push(check);
        }

        Ok(history)
    }

    async fn get_system_health(&self) -> MonitoringResult<SystemHealthStatus> {
        let checks = self.perform_all_checks().await?;

        let healthy_checks = checks
            .iter()
            .filter(|check| check.status == HealthStatus::Healthy)
            .count() as u32;

        let unhealthy_checks = checks
            .iter()
            .filter(|check| check.status == HealthStatus::Unhealthy)
            .count() as u32;

        let degraded_checks = checks
            .iter()
            .filter(|check| check.status == HealthStatus::Degraded)
            .count() as u32;

        let unknown_checks = checks
            .iter()
            .filter(|check| check.status == HealthStatus::Unknown)
            .count() as u32;

        let total_checks = checks.len() as u32;
        let overall_status = self.determine_overall_status(&checks);

        let details: Vec<HealthCheckSummary> = checks
            .iter()
            .map(|check| HealthCheckSummary {
                check_id: check.id,
                name: format!("Health Check {}", check.id),
                status: check.status,
                last_check: check.checked_at,
                response_time: check.response_time,
                uptime_percentage: self.calculate_uptime_percentage(&check.definition_id),
            })
            .collect();

        Ok(SystemHealthStatus {
            overall_status,
            healthy_checks,
            unhealthy_checks,
            degraded_checks,
            unknown_checks,
            total_checks,
            last_updated: Utc::now(),
            details,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_health_checker() {
        let checker = HTTPHealthChecker::new();

        let check = checker
            .check("http://localhost:8080/health", chrono::Duration::seconds(5))
            .await
            .unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(!check.message.is_empty());
    }

    #[tokio::test]
    async fn test_tcp_health_checker() {
        let checker = TCPHealthChecker::new();

        let check = checker
            .check("localhost:8080", chrono::Duration::seconds(5))
            .await
            .unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(!check.message.is_empty());
    }

    #[tokio::test]
    async fn test_database_health_checker() {
        let checker = DatabaseHealthChecker::new();

        let check = checker
            .check(
                "postgresql://localhost:5432/test",
                chrono::Duration::seconds(10),
            )
            .await
            .unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(!check.message.is_empty());
    }

    #[tokio::test]
    async fn test_health_check_service() {
        let config = HealthCheckConfig {
            enabled: true,
            check_interval_seconds: 30,
            timeout_seconds: 10,
            checks: Vec::new(),
        };

        let service = HealthCheckServiceImpl::new(config);

        let checks = service.perform_all_checks().await.unwrap();
        assert_eq!(checks.len(), 3);

        let system_health = service.get_system_health().await.unwrap();
        assert_eq!(system_health.total_checks, 3);
        assert!(system_health.healthy_checks > 0);
    }

    #[tokio::test]
    async fn test_unhealthy_endpoint() {
        let checker = HTTPHealthChecker::new();

        let check = checker
            .check(
                "http://localhost:8080/unhealthy",
                chrono::Duration::seconds(5),
            )
            .await
            .unwrap();
        assert_eq!(check.status, HealthStatus::Unhealthy);
        assert!(check.message.contains("not responding"));
    }

    #[tokio::test]
    async fn test_degraded_endpoint() {
        let checker = HTTPHealthChecker::new();

        let check = checker
            .check(
                "http://localhost:8080/degraded",
                chrono::Duration::seconds(5),
            )
            .await
            .unwrap();
        assert_eq!(check.status, HealthStatus::Degraded);
        assert!(check.message.contains("slowly"));
    }
}
