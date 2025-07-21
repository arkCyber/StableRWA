// =====================================================================================
// File: core-compliance/src/service.rs
// Description: Main compliance service orchestrator
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    error::{ComplianceError, ComplianceResult},
    types::{ComplianceLevel, RiskLevel, KycData, JurisdictionCode},
    kyc::{KycService, KycResult},
    aml::{AmlService, AmlScreeningResult, TransactionData},
    reporting::ReportingService,
    jurisdiction::JurisdictionService,
    audit::AuditService,
    ComplianceConfig, ComplianceCheckResult,
};

/// Main compliance service that orchestrates all compliance operations
pub struct ComplianceService {
    config: ComplianceConfig,
    kyc_service: Arc<RwLock<KycService>>,
    aml_service: Arc<RwLock<AmlService>>,
    reporting_service: Arc<ReportingService>,
    jurisdiction_service: Arc<JurisdictionService>,
    audit_service: Arc<AuditService>,
}

impl ComplianceService {
    /// Create new compliance service
    pub fn new(config: ComplianceConfig) -> Self {
        let kyc_service = Arc::new(RwLock::new(KycService::new(config.kyc_config.clone())));
        let aml_service = Arc::new(RwLock::new(AmlService::new(config.aml_config.clone())));
        let reporting_service = Arc::new(ReportingService::new(config.reporting_config.clone()));
        let jurisdiction_service = Arc::new(JurisdictionService::new(
            config.enabled_jurisdictions.clone(),
        ));
        let audit_service = Arc::new(AuditService::new(config.audit_config.clone()));

        Self {
            config,
            kyc_service,
            aml_service,
            reporting_service,
            jurisdiction_service,
            audit_service,
        }
    }

    /// Perform comprehensive compliance check for a user
    pub async fn perform_compliance_check(
        &self,
        user_id: String,
        kyc_data: &KycData,
        required_level: ComplianceLevel,
        jurisdiction: JurisdictionCode,
    ) -> ComplianceResult<ComplianceCheckResult> {
        let mut result = ComplianceCheckResult::new(user_id.clone());

        // Log audit event
        self.audit_service
            .log_compliance_check_started(&user_id, required_level)
            .await?;

        // 1. Jurisdiction compliance check
        let jurisdiction_checks = self
            .jurisdiction_service
            .check_compliance(jurisdiction, required_level)
            .await?;
        result.jurisdiction_checks = jurisdiction_checks;

        // 2. KYC verification
        let kyc_service = self.kyc_service.read().await;
        let kyc_result = kyc_service.verify_user(kyc_data, required_level).await?;
        result.kyc_result = Some(kyc_result);

        // 3. AML screening
        let aml_service = self.aml_service.read().await;
        let aml_result = aml_service.screen_user(kyc_data).await?;
        result.aml_result = Some(aml_result.clone());

        // 4. Determine overall compliance status
        result.status = self.determine_overall_status(&result)?;
        result.level = self.determine_compliance_level(&result)?;
        result.risk_level = self.determine_risk_level(&result)?;

        // 5. Set expiry date
        result.expires_at = Some(self.calculate_expiry_date(&result));

        // Log completion
        self.audit_service
            .log_compliance_check_completed(&user_id, &result)
            .await?;

        Ok(result)
    }

    /// Check if user meets compliance requirements
    pub async fn check_compliance_status(
        &self,
        user_id: &str,
        required_level: ComplianceLevel,
        max_risk: RiskLevel,
    ) -> ComplianceResult<bool> {
        // This would typically fetch from database
        // For now, we'll return a placeholder implementation
        Ok(true)
    }

    /// Monitor transaction for compliance
    pub async fn monitor_transaction(
        &self,
        transaction_data: &TransactionData,
    ) -> ComplianceResult<bool> {
        // Check if transaction requires enhanced screening
        let aml_service = self.aml_service.read().await;
        if aml_service.requires_enhanced_screening(transaction_data.amount) {
            let aml_check = aml_service.monitor_transaction(transaction_data).await?;

            // Log transaction monitoring
            self.audit_service
                .log_transaction_monitored(&transaction_data.transaction_id, &aml_check)
                .await?;

            // Check if transaction should be blocked
            match aml_check.result {
                crate::types::AmlResult::Block => return Ok(false),
                crate::types::AmlResult::Review => {
                    // Flag for manual review
                    self.audit_service
                        .log_manual_review_required(&transaction_data.transaction_id)
                        .await?;
                }
                _ => {}
            }
        }

        Ok(true)
    }

    /// Generate compliance report
    pub async fn generate_compliance_report(
        &self,
        report_type: crate::types::ReportType,
        jurisdiction: JurisdictionCode,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> ComplianceResult<crate::types::ComplianceReport> {
        self.reporting_service
            .generate_report(report_type, jurisdiction, period_start, period_end)
            .await
    }

    /// Update compliance configuration
    pub async fn update_config(&mut self, new_config: ComplianceConfig) -> ComplianceResult<()> {
        // Update services with new configuration
        let mut kyc_service = self.kyc_service.write().await;
        *kyc_service = KycService::new(new_config.kyc_config.clone());

        let mut aml_service = self.aml_service.write().await;
        *aml_service = AmlService::new(new_config.aml_config.clone());

        self.config = new_config;

        // Log configuration change
        self.audit_service.log_config_change().await?;

        Ok(())
    }

    /// Get compliance statistics
    pub async fn get_compliance_statistics(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> ComplianceResult<ComplianceStatistics> {
        // This would typically aggregate data from database
        Ok(ComplianceStatistics {
            total_checks: 0,
            approved_checks: 0,
            rejected_checks: 0,
            pending_checks: 0,
            average_processing_time_seconds: 0.0,
            risk_distribution: std::collections::HashMap::new(),
        })
    }

    // Private helper methods

    fn determine_overall_status(
        &self,
        result: &ComplianceCheckResult,
    ) -> ComplianceResult<crate::types::ComplianceStatus> {
        // Check KYC status
        if let Some(kyc_result) = &result.kyc_result {
            if kyc_result.status != crate::types::VerificationStatus::Verified {
                return Ok(crate::types::ComplianceStatus::Rejected);
            }
        }

        // Check AML status
        if let Some(aml_result) = &result.aml_result {
            match aml_result.overall_result {
                crate::types::AmlResult::Block => {
                    return Ok(crate::types::ComplianceStatus::Rejected);
                }
                crate::types::AmlResult::Review => {
                    return Ok(crate::types::ComplianceStatus::InProgress);
                }
                _ => {}
            }
        }

        // Check jurisdiction compliance
        for jurisdiction_check in &result.jurisdiction_checks {
            if !jurisdiction_check.compliant {
                return Ok(crate::types::ComplianceStatus::Rejected);
            }
        }

        Ok(crate::types::ComplianceStatus::Approved)
    }

    fn determine_compliance_level(
        &self,
        result: &ComplianceCheckResult,
    ) -> ComplianceResult<ComplianceLevel> {
        if let Some(kyc_result) = &result.kyc_result {
            Ok(kyc_result.compliance_level)
        } else {
            Ok(ComplianceLevel::Basic)
        }
    }

    fn determine_risk_level(&self, result: &ComplianceCheckResult) -> ComplianceResult<RiskLevel> {
        if let Some(aml_result) = &result.aml_result {
            Ok(aml_result.risk_level)
        } else {
            Ok(RiskLevel::Unknown)
        }
    }

    fn calculate_expiry_date(&self, result: &ComplianceCheckResult) -> DateTime<Utc> {
        let base_duration = match result.level {
            ComplianceLevel::Basic => chrono::Duration::days(180),   // 6 months
            ComplianceLevel::Standard => chrono::Duration::days(365), // 1 year
            ComplianceLevel::Enhanced => chrono::Duration::days(730), // 2 years
            ComplianceLevel::Premium => chrono::Duration::days(1095), // 3 years
        };

        // Adjust based on risk level
        let adjusted_duration = match result.risk_level {
            RiskLevel::Critical => base_duration / 4,
            RiskLevel::High => base_duration / 2,
            RiskLevel::Medium => base_duration * 3 / 4,
            _ => base_duration,
        };

        Utc::now() + adjusted_duration
    }
}

/// Compliance statistics structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplianceStatistics {
    pub total_checks: u64,
    pub approved_checks: u64,
    pub rejected_checks: u64,
    pub pending_checks: u64,
    pub average_processing_time_seconds: f64,
    pub risk_distribution: std::collections::HashMap<RiskLevel, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, IdentityDocument, DocumentType};

    fn create_test_kyc_data() -> KycData {
        KycData {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            date_of_birth: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            nationality: "US".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "New York".to_string(),
                state_province: Some("NY".to_string()),
                postal_code: "10001".to_string(),
                country: "US".to_string(),
            },
            identity_document: IdentityDocument {
                document_type: DocumentType::Passport,
                document_number: "123456789".to_string(),
                issuing_country: "US".to_string(),
                issue_date: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                expiry_date: Some(chrono::NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()),
            },
            verification_status: crate::types::VerificationStatus::NotStarted,
            verification_date: None,
            expiry_date: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_compliance_service_creation() {
        let config = ComplianceConfig::default();
        let _service = ComplianceService::new(config);
        // Service should be created without panicking
    }

    #[test]
    fn test_expiry_date_calculation() {
        let config = ComplianceConfig::default();
        let service = ComplianceService::new(config);

        let mut result = ComplianceCheckResult::new("user123".to_string());
        result.level = ComplianceLevel::Standard;
        result.risk_level = RiskLevel::Low;

        let expiry = service.calculate_expiry_date(&result);
        assert!(expiry > Utc::now());
    }

    #[test]
    fn test_risk_adjusted_expiry() {
        let config = ComplianceConfig::default();
        let service = ComplianceService::new(config);

        let mut low_risk_result = ComplianceCheckResult::new("user1".to_string());
        low_risk_result.level = ComplianceLevel::Standard;
        low_risk_result.risk_level = RiskLevel::Low;

        let mut high_risk_result = ComplianceCheckResult::new("user2".to_string());
        high_risk_result.level = ComplianceLevel::Standard;
        high_risk_result.risk_level = RiskLevel::High;

        let low_risk_expiry = service.calculate_expiry_date(&low_risk_result);
        let high_risk_expiry = service.calculate_expiry_date(&high_risk_result);

        // High risk should have shorter expiry
        assert!(high_risk_expiry < low_risk_expiry);
    }
}
