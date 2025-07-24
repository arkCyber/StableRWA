// =====================================================================================
// File: core-smart-contract/src/types.rs
// Description: Core types for smart contract management operations
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Smart contract representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub id: Uuid,
    pub name: String,
    pub address: String,
    pub network: String,
    pub version: ContractVersion,
    pub metadata: ContractMetadata,
    pub state: ContractState,
    pub abi: serde_json::Value,
    pub bytecode: String,
    pub source_code: Option<String>,
    pub proxy_config: Option<ProxyConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub description: String,
    pub author: String,
    pub license: String,
    pub compiler_version: String,
    pub optimization_enabled: bool,
    pub optimization_runs: u32,
    pub evm_version: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<ContractDependency>,
    pub interfaces: Vec<String>,
}

/// Contract version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

/// Contract dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDependency {
    pub name: String,
    pub version: String,
    pub source: DependencySource,
    pub required: bool,
}

/// Dependency source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencySource {
    OpenZeppelin,
    NPM,
    GitHub,
    Local,
    Registry,
}

/// Contract state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractState {
    Draft,
    Compiled,
    Deployed,
    Verified,
    Audited,
    Deprecated,
    Paused,
    Upgraded,
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub network: String,
    pub deployer_address: String,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
    pub constructor_args: Vec<serde_json::Value>,
    pub libraries: HashMap<String, String>,
    pub proxy_config: Option<ProxyConfig>,
    pub verification_config: Option<VerificationConfig>,
    pub monitoring_config: Option<MonitoringConfig>,
}

/// Upgrade configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeConfig {
    pub strategy: UpgradeStrategy,
    pub new_implementation: String,
    pub migration_data: Option<Vec<u8>>,
    pub timelock_delay: Option<u64>,
    pub multisig_threshold: Option<u32>,
    pub rollback_plan: Option<RollbackPlan>,
}

/// Upgrade strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeStrategy {
    Transparent,
    UUPS,
    Beacon,
    Diamond,
    Create2,
}

/// Rollback plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub previous_implementation: String,
    pub rollback_conditions: Vec<RollbackCondition>,
    pub automatic_rollback: bool,
    pub rollback_timeout: u64,
}

/// Rollback condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackCondition {
    pub condition_type: RollbackConditionType,
    pub threshold: f64,
    pub check_interval: u64,
}

/// Rollback condition types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RollbackConditionType {
    ErrorRate,
    GasUsage,
    TransactionFailure,
    CustomMetric,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub admin_address: String,
    pub implementation_address: String,
    pub initialization_data: Option<Vec<u8>>,
    pub upgrade_delay: Option<u64>,
}

/// Proxy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProxyType {
    Transparent,
    UUPS,
    Beacon,
    Diamond,
    Minimal,
}

/// Verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub enabled: bool,
    pub verifier: ContractVerifier,
    pub api_key: Option<String>,
    pub source_files: Vec<SourceFile>,
    pub compiler_settings: CompilerSettings,
}

/// Contract verifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractVerifier {
    Etherscan,
    Sourcify,
    Blockscout,
    Custom,
}

/// Source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
    pub imports: Vec<String>,
}

/// Compiler settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerSettings {
    pub version: String,
    pub optimization: OptimizationSettings,
    pub evm_version: String,
    pub libraries: HashMap<String, String>,
    pub metadata: CompilerMetadata,
}

/// Optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    pub enabled: bool,
    pub runs: u32,
    pub details: OptimizationDetails,
}

/// Optimization details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationDetails {
    pub peephole: bool,
    pub inliner: bool,
    pub jumpdest_remover: bool,
    pub order_literals: bool,
    pub deduplicate: bool,
    pub cse: bool,
    pub constant_optimizer: bool,
    pub yul: bool,
}

/// Compiler metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerMetadata {
    pub use_literal_content: bool,
    pub bytecode_hash: String,
    pub cb_or_hash: String,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub metrics: Vec<MonitoringMetric>,
    pub alerts: Vec<MonitoringAlert>,
    pub dashboards: Vec<String>,
    pub retention_days: u32,
}

/// Monitoring metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetric {
    pub name: String,
    pub metric_type: MetricType,
    pub collection_interval: u64,
    pub aggregation: AggregationType,
    pub labels: HashMap<String, String>,
}

/// Metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Aggregation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggregationType {
    Sum,
    Average,
    Min,
    Max,
    Count,
    Rate,
}

/// Monitoring alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringAlert {
    pub name: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub notification_channels: Vec<String>,
    pub cooldown_period: u64,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    pub metric: String,
    pub operator: ComparisonOperator,
    pub threshold: f64,
    pub duration: u64,
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub auditor: String,
    pub audit_type: AuditType,
    pub findings: Vec<AuditFinding>,
    pub overall_score: f64,
    pub recommendations: Vec<String>,
    pub status: AuditStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Audit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditType {
    Security,
    Performance,
    Compliance,
    Formal,
    Automated,
}

/// Audit finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub id: Uuid,
    pub severity: FindingSeverity,
    pub category: FindingCategory,
    pub title: String,
    pub description: String,
    pub location: CodeLocation,
    pub recommendation: String,
    pub status: FindingStatus,
}

/// Finding severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Finding category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FindingCategory {
    Security,
    Performance,
    Logic,
    Style,
    Documentation,
    Compliance,
}

/// Code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line_start: u32,
    pub line_end: u32,
    pub column_start: u32,
    pub column_end: u32,
}

/// Finding status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FindingStatus {
    Open,
    Acknowledged,
    Fixed,
    Dismissed,
    FalsePositive,
}

/// Audit status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuditStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Gas optimization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasOptimizationReport {
    pub id: Uuid,
    pub contract_id: Uuid,
    pub original_gas_usage: u64,
    pub optimized_gas_usage: u64,
    pub savings_percentage: f64,
    pub optimizations: Vec<GasOptimization>,
    pub created_at: DateTime<Utc>,
}

/// Gas optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasOptimization {
    pub optimization_type: OptimizationType,
    pub description: String,
    pub gas_saved: u64,
    pub code_changes: Vec<CodeChange>,
}

/// Optimization types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptimizationType {
    StorageOptimization,
    FunctionOptimization,
    LoopOptimization,
    VariableOptimization,
    LibraryOptimization,
    InlineOptimization,
}

/// Code change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub location: CodeLocation,
    pub original_code: String,
    pub optimized_code: String,
    pub explanation: String,
}

impl Default for ContractVersion {
    fn default() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
            pre_release: None,
            build_metadata: None,
        }
    }
}

impl std::fmt::Display for ContractVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build_metadata {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl ContractVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    pub fn with_pre_release(mut self, pre_release: String) -> Self {
        self.pre_release = Some(pre_release);
        self
    }

    pub fn with_build_metadata(mut self, build_metadata: String) -> Self {
        self.build_metadata = Some(build_metadata);
        self
    }

    pub fn is_compatible_with(&self, other: &ContractVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            runs: 200,
            details: OptimizationDetails {
                peephole: true,
                inliner: true,
                jumpdest_remover: true,
                order_literals: true,
                deduplicate: true,
                cse: true,
                constant_optimizer: true,
                yul: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_version_display() {
        let version = ContractVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");

        let version_with_pre = version.with_pre_release("alpha.1".to_string());
        assert_eq!(version_with_pre.to_string(), "1.2.3-alpha.1");

        let version_with_build = version_with_pre.with_build_metadata("20240101".to_string());
        assert_eq!(version_with_build.to_string(), "1.2.3-alpha.1+20240101");
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = ContractVersion::new(1, 0, 0);
        let v1_1_0 = ContractVersion::new(1, 1, 0);
        let v2_0_0 = ContractVersion::new(2, 0, 0);

        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v1_1_0));
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
    }

    #[test]
    fn test_contract_state_transitions() {
        let states = vec![
            ContractState::Draft,
            ContractState::Compiled,
            ContractState::Deployed,
            ContractState::Verified,
            ContractState::Audited,
        ];

        for state in states {
            assert_ne!(state, ContractState::Deprecated);
        }
    }

    #[test]
    fn test_finding_severity_ordering() {
        assert!(FindingSeverity::Info < FindingSeverity::Low);
        assert!(FindingSeverity::Low < FindingSeverity::Medium);
        assert!(FindingSeverity::Medium < FindingSeverity::High);
        assert!(FindingSeverity::High < FindingSeverity::Critical);
    }
}
