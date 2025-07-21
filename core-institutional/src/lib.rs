// =====================================================================================
// File: core-institutional/src/lib.rs
// Description: Institutional services for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Core Institutional Module
//! 
//! This module provides comprehensive institutional services for the StableRWA platform,
//! including custody services, bulk trading, white label solutions, and institutional
//! API gateway functionality.

pub mod error;
pub mod types;
pub mod custody;
pub mod bulk_trading;
pub mod whitelabel;
pub mod api_gateway;
pub mod service;

// Re-export main types and traits
pub use error::{InstitutionalError, InstitutionalResult};
pub use types::{
    Institution, InstitutionType, InstitutionalAccount, InstitutionalUser,
    InstitutionalRole, Permission, AccountType, ComplianceTier, RiskRating,
    ServiceTier, ApiKey, ReportConfig, WhiteLabelConfig
};
pub use custody::{
    CustodyService, CustodyAccount, CustodyAccountType, SegregationType,
    AuthorizedSigner, SigningAuthority, CustodyTransactionRequest, CustodyTransactionResult
};
pub use bulk_trading::{
    BulkTradingService, BulkOrderRequest, BulkOrder, ExecutionStrategy,
    BulkOrderResult, OrderExecutionResult, TradingStatistics
};
pub use whitelabel::{
    WhiteLabelService, WhiteLabelPlatform, BrandingConfig, FeatureConfig,
    CustomizationOptions, WhiteLabelDeployment
};
pub use api_gateway::{
    ApiGatewayService, ApiEndpoint, RateLimitConfig, AuthenticationConfig,
    ApiMetrics, ApiRequest, ApiResponse
};
pub use service::InstitutionalService;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Main institutional service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalServiceConfig {
    /// Custody service configuration
    pub custody_config: custody::CustodyConfig,
    /// Bulk trading configuration
    pub bulk_trading_config: bulk_trading::BulkTradingConfig,
    /// White label service configuration
    pub whitelabel_config: whitelabel::WhiteLabelConfig,
    /// API gateway configuration
    pub api_gateway_config: api_gateway::ApiGatewayConfig,
    /// Global institutional settings
    pub global_settings: GlobalInstitutionalSettings,
}

impl Default for InstitutionalServiceConfig {
    fn default() -> Self {
        Self {
            custody_config: custody::CustodyConfig::default(),
            bulk_trading_config: bulk_trading::BulkTradingConfig::default(),
            whitelabel_config: whitelabel::WhiteLabelConfig::default(),
            api_gateway_config: api_gateway::ApiGatewayConfig::default(),
            global_settings: GlobalInstitutionalSettings::default(),
        }
    }
}

/// Global institutional settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalInstitutionalSettings {
    /// Minimum institution AUM for premium services
    pub min_aum_premium: Decimal,
    /// Maximum number of users per institution
    pub max_users_per_institution: u32,
    /// Maximum number of accounts per institution
    pub max_accounts_per_institution: u32,
    /// Default session timeout in minutes
    pub default_session_timeout_minutes: u32,
    /// Enable multi-factor authentication
    pub require_mfa: bool,
    /// Enable IP whitelisting
    pub enable_ip_whitelisting: bool,
    /// Audit log retention days
    pub audit_retention_days: u32,
    /// Enable real-time notifications
    pub enable_notifications: bool,
    /// Default reporting frequency
    pub default_reporting_frequency: types::ReportingFrequency,
}

impl Default for GlobalInstitutionalSettings {
    fn default() -> Self {
        Self {
            min_aum_premium: Decimal::new(10000000000, 2), // $100M
            max_users_per_institution: 1000,
            max_accounts_per_institution: 100,
            default_session_timeout_minutes: 30,
            require_mfa: true,
            enable_ip_whitelisting: true,
            audit_retention_days: 2555, // 7 years
            enable_notifications: true,
            default_reporting_frequency: types::ReportingFrequency::Daily,
        }
    }
}

/// Institutional service metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalMetrics {
    pub total_institutions: u64,
    pub active_institutions: u64,
    pub total_aum: Decimal,
    pub total_custody_accounts: u64,
    pub total_trading_volume_24h: Decimal,
    pub active_api_keys: u64,
    pub api_requests_24h: u64,
    pub whitelabel_deployments: u64,
    pub average_response_time_ms: f64,
    pub uptime_percentage: Decimal,
    pub last_updated: DateTime<Utc>,
}

/// Institutional service health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalHealthStatus {
    pub overall_status: String,
    pub custody_service: String,
    pub bulk_trading_service: String,
    pub whitelabel_service: String,
    pub api_gateway_service: String,
    pub database_status: String,
    pub external_services_status: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
}

/// Institutional onboarding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalOnboardingRequest {
    pub id: Uuid,
    pub institution_info: Institution,
    pub primary_contact: types::Contact,
    pub compliance_documents: Vec<ComplianceDocument>,
    pub requested_services: Vec<RequestedService>,
    pub estimated_aum: Decimal,
    pub estimated_monthly_volume: Decimal,
    pub regulatory_requirements: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub status: OnboardingStatus,
}

/// Compliance document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceDocument {
    pub document_type: DocumentType,
    pub document_url: String,
    pub expiry_date: Option<DateTime<Utc>>,
    pub verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<Uuid>,
}

/// Document type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    CertificateOfIncorporation,
    RegulatoryLicense,
    AuditedFinancials,
    CompliancePolicy,
    RiskManagementPolicy,
    InsurancePolicy,
    BoardResolution,
    PowerOfAttorney,
    Other,
}

/// Requested service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestedService {
    Custody,
    BulkTrading,
    WhiteLabel,
    PrimeServices,
    ReportingServices,
    ComplianceServices,
    RiskManagement,
}

/// Onboarding status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnboardingStatus {
    Submitted,
    UnderReview,
    DocumentsRequested,
    ComplianceReview,
    RiskAssessment,
    Approved,
    Rejected,
    OnHold,
}

/// Service level agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevelAgreement {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub service_tier: ServiceTier,
    pub uptime_guarantee: Decimal,
    pub response_time_guarantee_ms: u64,
    pub throughput_guarantee_tps: u32,
    pub support_level: SupportLevel,
    pub data_retention_days: u32,
    pub backup_frequency_hours: u32,
    pub disaster_recovery_rto_hours: u32,
    pub disaster_recovery_rpo_hours: u32,
    pub effective_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub penalties: Vec<SLAPenalty>,
}

/// Support level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportLevel {
    Basic,      // Business hours, email
    Standard,   // Extended hours, phone + email
    Premium,    // 24/7, dedicated support manager
    Enterprise, // 24/7, dedicated team + escalation
}

/// SLA penalty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLAPenalty {
    pub metric: String,
    pub threshold: Decimal,
    pub penalty_percentage: Decimal,
    pub max_penalty: Decimal,
}

/// Institutional audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: String,
    pub user_agent: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_institutional_service_config_default() {
        let config = InstitutionalServiceConfig::default();
        assert!(config.global_settings.require_mfa);
        assert!(config.global_settings.enable_ip_whitelisting);
        assert_eq!(config.global_settings.max_users_per_institution, 1000);
        assert_eq!(config.global_settings.audit_retention_days, 2555);
    }

    #[test]
    fn test_global_institutional_settings() {
        let settings = GlobalInstitutionalSettings::default();
        assert_eq!(settings.min_aum_premium, Decimal::new(10000000000, 2));
        assert_eq!(settings.default_session_timeout_minutes, 30);
        assert!(settings.require_mfa);
        assert!(settings.enable_notifications);
    }

    #[test]
    fn test_onboarding_request_creation() {
        let institution = Institution {
            id: Uuid::new_v4(),
            name: "Test Institution".to_string(),
            institution_type: InstitutionType::Bank,
            legal_entity_identifier: Some("123456789012345678".to_string()),
            jurisdiction: "US".to_string(),
            regulatory_licenses: vec!["FDIC".to_string()],
            contact_info: types::ContactInfo {
                primary_contact: types::Contact {
                    name: "John Doe".to_string(),
                    title: "CEO".to_string(),
                    email: "john@testbank.com".to_string(),
                    phone: "+1-555-0123".to_string(),
                },
                compliance_contact: None,
                technical_contact: None,
                address: types::Address {
                    street: "123 Main St".to_string(),
                    city: "New York".to_string(),
                    state_province: "NY".to_string(),
                    postal_code: "10001".to_string(),
                    country: "US".to_string(),
                },
            },
            compliance_tier: ComplianceTier::Enhanced,
            risk_rating: RiskRating::Low,
            aum: Some(Decimal::new(100000000000, 2)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        };

        let onboarding_request = InstitutionalOnboardingRequest {
            id: Uuid::new_v4(),
            institution_info: institution,
            primary_contact: types::Contact {
                name: "Jane Smith".to_string(),
                title: "COO".to_string(),
                email: "jane@testbank.com".to_string(),
                phone: "+1-555-0124".to_string(),
            },
            compliance_documents: vec![],
            requested_services: vec![RequestedService::Custody, RequestedService::BulkTrading],
            estimated_aum: Decimal::new(100000000000, 2),
            estimated_monthly_volume: Decimal::new(1000000000, 2),
            regulatory_requirements: vec!["SOX".to_string(), "GDPR".to_string()],
            created_at: Utc::now(),
            status: OnboardingStatus::Submitted,
        };

        assert_eq!(onboarding_request.status, OnboardingStatus::Submitted);
        assert_eq!(onboarding_request.requested_services.len(), 2);
        assert!(onboarding_request.requested_services.contains(&RequestedService::Custody));
    }
}
