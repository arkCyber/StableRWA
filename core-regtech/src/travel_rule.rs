// =====================================================================================
// File: core-regtech/src/travel_rule.rs
// Description: Travel Rule compliance module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{RegTechError, RegTechResult},
    types::ComplianceFramework,
};

/// Travel Rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelRuleConfig {
    pub threshold_amount: Decimal,
    pub vasp_directory: String,
    pub message_encryption: bool,
    pub compliance_frameworks: Vec<ComplianceFramework>,
}

/// Travel Rule service trait
#[async_trait]
pub trait TravelRuleService: Send + Sync {
    /// Check if transaction requires Travel Rule compliance
    async fn requires_travel_rule(&self, amount: Decimal) -> RegTechResult<bool>;

    /// Create Travel Rule message
    async fn create_message(
        &self,
        transaction_data: &TravelRuleTransaction,
    ) -> RegTechResult<TravelRuleMessage>;
}

/// Travel Rule message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelRuleMessage {
    pub message_id: Uuid,
    pub originator_info: OriginatorInfo,
    pub beneficiary_info: BeneficiaryInfo,
    pub transaction_info: TransactionInfo,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Originator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginatorInfo {
    pub name: String,
    pub address: String,
    pub account_number: String,
}

/// Beneficiary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficiaryInfo {
    pub name: String,
    pub address: String,
    pub account_number: String,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub amount: Decimal,
    pub currency: String,
    pub transaction_id: String,
}

/// Travel Rule transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelRuleTransaction {
    pub transaction_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub originator: OriginatorInfo,
    pub beneficiary: BeneficiaryInfo,
}

/// VASP directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VASPDirectory {
    pub vasps: HashMap<String, VASPInfo>,
}

/// VASP information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VASPInfo {
    pub vasp_id: String,
    pub name: String,
    pub jurisdiction: String,
    pub public_key: String,
}

/// Travel Rule compliance checker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelRuleCompliance {
    pub is_compliant: bool,
    pub required_fields: Vec<String>,
    pub missing_fields: Vec<String>,
}

/// Travel Rule service implementation
pub struct TravelRuleServiceImpl {
    config: TravelRuleConfig,
    vasp_directory: VASPDirectory,
}

impl TravelRuleServiceImpl {
    pub fn new(config: TravelRuleConfig) -> Self {
        Self {
            config,
            vasp_directory: VASPDirectory {
                vasps: HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl TravelRuleService for TravelRuleServiceImpl {
    async fn requires_travel_rule(&self, amount: Decimal) -> RegTechResult<bool> {
        Ok(amount >= self.config.threshold_amount)
    }

    async fn create_message(
        &self,
        transaction_data: &TravelRuleTransaction,
    ) -> RegTechResult<TravelRuleMessage> {
        let message = TravelRuleMessage {
            message_id: Uuid::new_v4(),
            originator_info: transaction_data.originator.clone(),
            beneficiary_info: transaction_data.beneficiary.clone(),
            transaction_info: TransactionInfo {
                amount: transaction_data.amount,
                currency: transaction_data.currency.clone(),
                transaction_id: transaction_data.transaction_id.clone(),
            },
            created_at: chrono::Utc::now(),
        };

        Ok(message)
    }
}

impl Default for TravelRuleConfig {
    fn default() -> Self {
        Self {
            threshold_amount: Decimal::new(300000, 2), // $3000.00
            vasp_directory: "https://vaspdirectory.org".to_string(),
            message_encryption: true,
            compliance_frameworks: vec![ComplianceFramework::FATF],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_travel_rule_threshold() {
        let config = TravelRuleConfig::default();
        let service = TravelRuleServiceImpl::new(config);

        // Test amount above threshold
        let high_amount = Decimal::new(500000, 2); // $5000.00
        let requires = service.requires_travel_rule(high_amount).await.unwrap();
        assert!(requires);

        // Test amount below threshold
        let low_amount = Decimal::new(100000, 2); // $1000.00
        let requires = service.requires_travel_rule(low_amount).await.unwrap();
        assert!(!requires);
    }
}
