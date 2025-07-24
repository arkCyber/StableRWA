// =====================================================================================
// File: core-risk-management/src/insurance.rs
// Description: Insurance and risk transfer mechanisms
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    error::{RiskError, RiskResult},
    types::{RiskCategory, InsurancePolicy, InsuranceClaim, ClaimStatus},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug, warn};
use uuid::Uuid;

/// Insurance coverage types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoverageType {
    Property,
    Liability,
    Professional,
    Cyber,
    Political,
    Credit,
    KeyPerson,
    BusinessInterruption,
    Environmental,
    Directors,
}

/// Insurance policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceConfig {
    pub auto_renewal: bool,
    pub coverage_limits: HashMap<CoverageType, f64>,
    pub deductibles: HashMap<CoverageType, f64>,
    pub preferred_providers: Vec<String>,
    pub risk_tolerance: f64,
    pub claims_threshold: f64,
}

impl Default for InsuranceConfig {
    fn default() -> Self {
        let mut coverage_limits = HashMap::new();
        coverage_limits.insert(CoverageType::Property, 10_000_000.0);
        coverage_limits.insert(CoverageType::Liability, 5_000_000.0);
        coverage_limits.insert(CoverageType::Professional, 2_000_000.0);
        coverage_limits.insert(CoverageType::Cyber, 1_000_000.0);

        let mut deductibles = HashMap::new();
        deductibles.insert(CoverageType::Property, 50_000.0);
        deductibles.insert(CoverageType::Liability, 25_000.0);
        deductibles.insert(CoverageType::Professional, 10_000.0);
        deductibles.insert(CoverageType::Cyber, 5_000.0);

        Self {
            auto_renewal: true,
            coverage_limits,
            deductibles,
            preferred_providers: vec![
                "Lloyd's of London".to_string(),
                "AIG".to_string(),
                "Zurich".to_string(),
            ],
            risk_tolerance: 0.05,
            claims_threshold: 100_000.0,
        }
    }
}

/// Insurance quote request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    pub asset_id: Uuid,
    pub coverage_types: Vec<CoverageType>,
    pub coverage_amounts: HashMap<CoverageType, f64>,
    pub policy_term_months: u32,
    pub deductible_preferences: HashMap<CoverageType, f64>,
    pub risk_profile: RiskProfile,
    pub additional_requirements: Vec<String>,
}

/// Risk profile for insurance underwriting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    pub asset_value: f64,
    pub asset_age_years: u32,
    pub location_risk_score: f64,
    pub historical_claims: Vec<HistoricalClaim>,
    pub security_measures: Vec<String>,
    pub compliance_certifications: Vec<String>,
    pub business_continuity_plan: bool,
}

/// Historical claim information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalClaim {
    pub claim_date: DateTime<Utc>,
    pub claim_amount: f64,
    pub claim_type: CoverageType,
    pub resolution_days: u32,
    pub cause: String,
}

/// Insurance quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsuranceQuote {
    pub quote_id: Uuid,
    pub provider: String,
    pub coverage_details: HashMap<CoverageType, CoverageDetail>,
    pub total_premium: f64,
    pub policy_term_months: u32,
    pub quote_valid_until: DateTime<Utc>,
    pub conditions: Vec<String>,
    pub exclusions: Vec<String>,
    pub rating_factors: HashMap<String, f64>,
}

/// Coverage detail information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageDetail {
    pub coverage_limit: f64,
    pub deductible: f64,
    pub premium: f64,
    pub coverage_description: String,
    pub special_conditions: Vec<String>,
}

/// Claim submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub policy_id: Uuid,
    pub incident_date: DateTime<Utc>,
    pub claim_amount: f64,
    pub coverage_type: CoverageType,
    pub description: String,
    pub supporting_documents: Vec<String>,
    pub emergency_contact: Option<String>,
    pub reported_by: String,
}

/// Insurance service trait
#[async_trait]
pub trait InsuranceService: Send + Sync {
    /// Get insurance quotes from multiple providers
    async fn get_quotes(
        &self,
        request: QuoteRequest,
    ) -> RiskResult<Vec<InsuranceQuote>>;

    /// Purchase insurance policy
    async fn purchase_policy(
        &self,
        quote_id: Uuid,
        payment_method: String,
    ) -> RiskResult<InsurancePolicy>;

    /// Submit insurance claim
    async fn submit_claim(
        &self,
        request: ClaimRequest,
    ) -> RiskResult<InsuranceClaim>;

    /// Get claim status
    async fn get_claim_status(
        &self,
        claim_id: Uuid,
    ) -> RiskResult<ClaimStatus>;

    /// Get active policies
    async fn get_active_policies(
        &self,
        asset_id: Uuid,
    ) -> RiskResult<Vec<InsurancePolicy>>;

    /// Renew policy
    async fn renew_policy(
        &self,
        policy_id: Uuid,
        updates: Option<PolicyUpdate>,
    ) -> RiskResult<InsurancePolicy>;

    /// Cancel policy
    async fn cancel_policy(
        &self,
        policy_id: Uuid,
        reason: String,
    ) -> RiskResult<()>;

    /// Calculate coverage adequacy
    async fn assess_coverage_adequacy(
        &self,
        asset_id: Uuid,
    ) -> RiskResult<CoverageAssessment>;
}

/// Policy update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdate {
    pub coverage_changes: Option<HashMap<CoverageType, f64>>,
    pub deductible_changes: Option<HashMap<CoverageType, f64>>,
    pub beneficiary_changes: Option<Vec<String>>,
    pub additional_coverage: Option<Vec<CoverageType>>,
}

/// Coverage adequacy assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAssessment {
    pub asset_id: Uuid,
    pub total_coverage: f64,
    pub coverage_gaps: Vec<CoverageGap>,
    pub over_insurance: Vec<CoverageType>,
    pub recommendations: Vec<String>,
    pub adequacy_score: f64,
    pub assessment_date: DateTime<Utc>,
}

/// Coverage gap identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    pub coverage_type: CoverageType,
    pub current_coverage: f64,
    pub recommended_coverage: f64,
    pub gap_amount: f64,
    pub risk_exposure: f64,
    pub priority: GapPriority,
}

/// Gap priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GapPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Default insurance service implementation
pub struct DefaultInsuranceService {
    config: InsuranceConfig,
    policies: HashMap<Uuid, InsurancePolicy>,
    claims: HashMap<Uuid, InsuranceClaim>,
}

impl DefaultInsuranceService {
    pub fn new(config: InsuranceConfig) -> Self {
        Self {
            config,
            policies: HashMap::new(),
            claims: HashMap::new(),
        }
    }

    /// Calculate premium based on risk factors
    fn calculate_premium(
        &self,
        coverage_type: &CoverageType,
        coverage_amount: f64,
        risk_profile: &RiskProfile,
    ) -> f64 {
        let base_rate = match coverage_type {
            CoverageType::Property => 0.005,
            CoverageType::Liability => 0.003,
            CoverageType::Professional => 0.008,
            CoverageType::Cyber => 0.012,
            CoverageType::Political => 0.015,
            CoverageType::Credit => 0.006,
            CoverageType::KeyPerson => 0.010,
            CoverageType::BusinessInterruption => 0.007,
            CoverageType::Environmental => 0.009,
            CoverageType::Directors => 0.011,
        };

        let risk_multiplier = 1.0 + risk_profile.location_risk_score;
        let claims_multiplier = if risk_profile.historical_claims.is_empty() {
            0.9 // No claims discount
        } else {
            1.0 + (risk_profile.historical_claims.len() as f64 * 0.1)
        };

        coverage_amount * base_rate * risk_multiplier * claims_multiplier
    }
}

#[async_trait]
impl InsuranceService for DefaultInsuranceService {
    async fn get_quotes(
        &self,
        request: QuoteRequest,
    ) -> RiskResult<Vec<InsuranceQuote>> {
        info!("Getting insurance quotes for asset {}", request.asset_id);

        let mut quotes = Vec::new();

        for provider in &self.config.preferred_providers {
            let mut coverage_details = HashMap::new();
            let mut total_premium = 0.0;

            for coverage_type in &request.coverage_types {
                let coverage_amount = request.coverage_amounts
                    .get(coverage_type)
                    .unwrap_or(&1_000_000.0);

                let premium = self.calculate_premium(
                    coverage_type,
                    *coverage_amount,
                    &request.risk_profile,
                );

                let deductible = request.deductible_preferences
                    .get(coverage_type)
                    .or_else(|| self.config.deductibles.get(coverage_type))
                    .unwrap_or(&10_000.0);

                coverage_details.insert(coverage_type.clone(), CoverageDetail {
                    coverage_limit: *coverage_amount,
                    deductible: *deductible,
                    premium,
                    coverage_description: format!("{:?} coverage", coverage_type),
                    special_conditions: vec![],
                });

                total_premium += premium;
            }

            quotes.push(InsuranceQuote {
                quote_id: Uuid::new_v4(),
                provider: provider.clone(),
                coverage_details,
                total_premium,
                policy_term_months: request.policy_term_months,
                quote_valid_until: Utc::now() + chrono::Duration::days(30),
                conditions: vec![
                    "Subject to underwriting approval".to_string(),
                    "Property inspection may be required".to_string(),
                ],
                exclusions: vec![
                    "Acts of war".to_string(),
                    "Nuclear risks".to_string(),
                ],
                rating_factors: HashMap::new(),
            });
        }

        debug!("Generated {} quotes", quotes.len());
        Ok(quotes)
    }

    async fn purchase_policy(
        &self,
        quote_id: Uuid,
        _payment_method: String,
    ) -> RiskResult<InsurancePolicy> {
        info!("Purchasing policy for quote {}", quote_id);

        // Mock policy creation
        let policy = crate::types::InsurancePolicy {
            id: Uuid::new_v4(),
            policy_number: format!("POL-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            policy_type: crate::types::PolicyType::PropertyInsurance,
            insurer: "Mock Insurance Co.".to_string(),
            insured_entity: "Asset Owner".to_string(),
            coverage_amount: rust_decimal::Decimal::from_f64_retain(1000000.0).unwrap_or_default(),
            deductible: rust_decimal::Decimal::from_f64_retain(10000.0).unwrap_or_default(),
            premium: rust_decimal::Decimal::from_f64_retain(25000.0).unwrap_or_default(),
            policy_start: Utc::now(),
            policy_end: Utc::now() + chrono::Duration::days(365),
            covered_risks: vec![crate::types::RiskType::Market, crate::types::RiskType::Operational],
            exclusions: vec!["War and terrorism".to_string()],
            claims_history: vec![],
            status: crate::types::PolicyStatus::Active,
        };

        debug!("Policy created: {}", policy.policy_number);
        Ok(policy)
    }

    async fn submit_claim(
        &self,
        request: ClaimRequest,
    ) -> RiskResult<InsuranceClaim> {
        info!("Submitting claim for policy {}", request.policy_id);

        let claim = crate::types::InsuranceClaim {
            id: Uuid::new_v4(),
            policy_id: request.policy_id,
            claim_number: format!("CLM-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            incident_date: request.incident_date,
            reported_date: Utc::now(),
            claim_amount: rust_decimal::Decimal::from_f64_retain(request.claim_amount).unwrap_or_default(),
            claim_type: crate::types::ClaimType::PropertyDamage,
            description: request.description,
            status: ClaimStatus::Submitted,
            adjuster: None,
            settlement_amount: None,
            settlement_date: None,
            documents: request.supporting_documents,
        };

        debug!("Claim submitted: {}", claim.claim_number);
        Ok(claim)
    }

    async fn get_claim_status(
        &self,
        claim_id: Uuid,
    ) -> RiskResult<ClaimStatus> {
        debug!("Getting status for claim {}", claim_id);
        
        // Mock status
        Ok(ClaimStatus::UnderReview)
    }

    async fn get_active_policies(
        &self,
        asset_id: Uuid,
    ) -> RiskResult<Vec<InsurancePolicy>> {
        debug!("Getting active policies for asset {}", asset_id);
        
        // Mock active policies
        Ok(vec![])
    }

    async fn renew_policy(
        &self,
        policy_id: Uuid,
        _updates: Option<PolicyUpdate>,
    ) -> RiskResult<InsurancePolicy> {
        info!("Renewing policy {}", policy_id);
        
        // Mock renewed policy
        let policy = crate::types::InsurancePolicy {
            id: policy_id,
            policy_number: format!("POL-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            policy_type: crate::types::PolicyType::PropertyInsurance,
            insurer: "Mock Insurance Co.".to_string(),
            insured_entity: "Asset Owner".to_string(),
            coverage_amount: rust_decimal::Decimal::from_f64_retain(1100000.0).unwrap_or_default(),
            deductible: rust_decimal::Decimal::from_f64_retain(10000.0).unwrap_or_default(),
            premium: rust_decimal::Decimal::from_f64_retain(26000.0).unwrap_or_default(), // Slight increase
            policy_start: Utc::now(),
            policy_end: Utc::now() + chrono::Duration::days(365),
            covered_risks: vec![crate::types::RiskType::Market],
            exclusions: vec!["Standard exclusions".to_string()],
            claims_history: vec![],
            status: crate::types::PolicyStatus::Active,
        };

        debug!("Policy renewed: {}", policy.policy_number);
        Ok(policy)
    }

    async fn cancel_policy(
        &self,
        policy_id: Uuid,
        reason: String,
    ) -> RiskResult<()> {
        info!("Cancelling policy {} - reason: {}", policy_id, reason);
        
        // Mock cancellation
        debug!("Policy cancelled successfully");
        Ok(())
    }

    async fn assess_coverage_adequacy(
        &self,
        asset_id: Uuid,
    ) -> RiskResult<CoverageAssessment> {
        debug!("Assessing coverage adequacy for asset {}", asset_id);
        
        // Mock assessment
        Ok(CoverageAssessment {
            asset_id,
            total_coverage: 15_000_000.0,
            coverage_gaps: vec![
                CoverageGap {
                    coverage_type: CoverageType::Cyber,
                    current_coverage: 500_000.0,
                    recommended_coverage: 2_000_000.0,
                    gap_amount: 1_500_000.0,
                    risk_exposure: 0.3,
                    priority: GapPriority::High,
                }
            ],
            over_insurance: vec![],
            recommendations: vec![
                "Increase cyber coverage".to_string(),
                "Consider umbrella policy".to_string(),
            ],
            adequacy_score: 0.75,
            assessment_date: Utc::now(),
        })
    }
}

impl Default for DefaultInsuranceService {
    fn default() -> Self {
        Self::new(InsuranceConfig::default())
    }
}
