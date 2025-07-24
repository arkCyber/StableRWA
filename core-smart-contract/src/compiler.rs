// =====================================================================================
// File: core-smart-contract/src/compiler.rs
// Description: Smart contract compilation module
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    error::{SmartContractError, SmartContractResult},
    service::{
        CompilationError, CompilationResult, CompilationWarning, CompilerConfig, SourceLocation,
    },
    types::{CompilerSettings, OptimizationSettings, SourceFile},
};

/// Smart contract compiler trait
#[async_trait]
pub trait SmartContractCompiler: Send + Sync {
    /// Compile smart contract source code
    async fn compile(
        &self,
        source_code: &str,
        config: &CompilerConfig,
    ) -> SmartContractResult<CompilationResult>;

    /// Compile multiple contracts
    async fn compile_batch(
        &self,
        sources: &[SourceFile],
        config: &CompilerConfig,
    ) -> SmartContractResult<BatchCompilationResult>;

    /// Get available compiler versions
    async fn get_available_versions(&self) -> SmartContractResult<Vec<String>>;

    /// Validate source code syntax
    async fn validate_syntax(
        &self,
        source_code: &str,
    ) -> SmartContractResult<Vec<CompilationWarning>>;
}

/// Solidity compiler implementation
pub struct SolidityCompiler {
    compiler_path: String,
    cache_dir: String,
    temp_dir: String,
}

/// Vyper compiler implementation
pub struct VyperCompiler {
    compiler_path: String,
    cache_dir: String,
}

/// Batch compilation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCompilationResult {
    pub contracts: HashMap<String, CompilationResult>,
    pub overall_success: bool,
    pub compilation_time_ms: u64,
    pub total_warnings: u32,
    pub total_errors: u32,
}

/// Compiler version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerVersion {
    pub version: String,
    pub build: String,
    pub long_version: String,
    pub keccak: String,
    pub urls: Vec<String>,
}

/// Compilation cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationCache {
    pub source_hash: String,
    pub compiler_version: String,
    pub optimization_settings: OptimizationSettings,
    pub result: CompilationResult,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// Dependency resolver
pub struct DependencyResolver {
    npm_registry: String,
    github_token: Option<String>,
    local_paths: Vec<String>,
}

impl SolidityCompiler {
    pub fn new(compiler_path: String, cache_dir: String, temp_dir: String) -> Self {
        Self {
            compiler_path,
            cache_dir,
            temp_dir,
        }
    }

    /// Install specific compiler version
    pub async fn install_version(&self, version: &str) -> SmartContractResult<()> {
        // Mock installation process
        println!("Installing Solidity compiler version: {}", version);
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        Ok(())
    }

    /// Get compiler information
    pub async fn get_compiler_info(&self) -> SmartContractResult<CompilerVersion> {
        Ok(CompilerVersion {
            version: "0.8.19".to_string(),
            build: "commit.7dd6d404".to_string(),
            long_version: "solc, the solidity compiler commandline interface Version: 0.8.19+commit.7dd6d404.Linux.g++".to_string(),
            keccak: "0x7dd6d404".to_string(),
            urls: vec![
                "https://github.com/ethereum/solidity/releases/tag/v0.8.19".to_string(),
            ],
        })
    }

    /// Prepare compilation input
    fn prepare_compilation_input(
        &self,
        source_code: &str,
        config: &CompilerConfig,
    ) -> serde_json::Value {
        serde_json::json!({
            "language": "Solidity",
            "sources": {
                "contract.sol": {
                    "content": source_code
                }
            },
            "settings": {
                "optimizer": {
                    "enabled": config.optimization_enabled,
                    "runs": config.optimization_runs
                },
                "evmVersion": config.evm_version,
                "libraries": config.libraries,
                "outputSelection": {
                    "*": {
                        "*": config.output_selection
                    }
                }
            }
        })
    }

    /// Parse compilation output
    fn parse_compilation_output(&self, output: &str) -> SmartContractResult<CompilationResult> {
        // Mock parsing - in reality, this would parse the actual solc output
        let output_json: serde_json::Value = serde_json::from_str(output).map_err(|e| {
            SmartContractError::compilation_error(format!("Failed to parse compiler output: {}", e))
        })?;

        let has_errors = output_json
            .get("errors")
            .and_then(|e| e.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        if has_errors {
            let errors = self.parse_errors(&output_json)?;
            return Ok(CompilationResult {
                success: false,
                bytecode: String::new(),
                abi: serde_json::Value::Null,
                metadata: serde_json::Value::Null,
                warnings: Vec::new(),
                errors,
                gas_estimates: HashMap::new(),
            });
        }

        // Mock successful compilation result
        Ok(CompilationResult {
            success: true,
            bytecode: "0x608060405234801561001057600080fd5b50336000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550610233806100606000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c8063893d20e81461003b578063a6f9dae114610059575b600080fd5b610043610075565b60405161005091906101a7565b60405180910390f35b610073600480360381019061006e919061014b565b61009e565b005b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b6000809054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff16146100fc57600080fd5b806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050565b60008135905061014581610209565b92915050565b60006020828403121561015d57600080fd5b600061016b84828501610136565b91505092915050565b61017d816101c2565b82525050565b600061018e826101b8565b6101988185610209565b93506101a88185602086016101d4565b6101b181610207565b840191505092915050565b600081519050919050565b60006101d2826101b8565b9150919050565b60005b838110156101f25780820151818401526020810190506101d7565b83811115610201576000848401525b50505050565b6000601f19601f8301169050919050565b610221816101c2565b811461022c57600080fd5b5056fea2646970667358221220".to_string(),
            abi: serde_json::json!([
                {
                    "inputs": [],
                    "name": "getOwner",
                    "outputs": [{"internalType": "address", "name": "", "type": "address"}],
                    "stateMutability": "view",
                    "type": "function"
                },
                {
                    "inputs": [{"internalType": "address", "name": "newOwner", "type": "address"}],
                    "name": "changeOwner",
                    "outputs": [],
                    "stateMutability": "nonpayable",
                    "type": "function"
                }
            ]),
            metadata: serde_json::json!({
                "compiler": {"version": "0.8.19+commit.7dd6d404"},
                "language": "Solidity",
                "output": {
                    "abi": [],
                    "devdoc": {"kind": "dev", "methods": {}, "version": 1},
                    "userdoc": {"kind": "user", "methods": {}, "version": 1}
                },
                "settings": {
                    "compilationTarget": {"contract.sol": "SimpleContract"},
                    "evmVersion": "london",
                    "libraries": {},
                    "metadata": {"bytecodeHash": "ipfs"},
                    "optimizer": {"enabled": true, "runs": 200},
                    "remappings": []
                },
                "sources": {
                    "contract.sol": {"keccak256": "0x123..."}
                },
                "version": 1
            }),
            warnings: Vec::new(),
            errors: Vec::new(),
            gas_estimates: {
                let mut estimates = HashMap::new();
                estimates.insert("creation".to_string(), 200000);
                estimates.insert("external".to_string(), 21000);
                estimates.insert("internal".to_string(), 5000);
                estimates
            },
        })
    }

    /// Parse compilation errors
    fn parse_errors(
        &self,
        output: &serde_json::Value,
    ) -> SmartContractResult<Vec<CompilationError>> {
        let mut errors = Vec::new();

        if let Some(error_array) = output.get("errors").and_then(|e| e.as_array()) {
            for error in error_array {
                if let Some(severity) = error.get("severity").and_then(|s| s.as_str()) {
                    if severity == "error" {
                        let message = error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Unknown error")
                            .to_string();

                        let error_type = error
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("CompilerError")
                            .to_string();

                        let source_location = error
                            .get("sourceLocation")
                            .and_then(|loc| self.parse_source_location(loc));

                        errors.push(CompilationError {
                            message,
                            error_type,
                            source_location,
                        });
                    }
                }
            }
        }

        Ok(errors)
    }

    /// Parse source location
    fn parse_source_location(&self, location: &serde_json::Value) -> Option<SourceLocation> {
        Some(SourceLocation {
            file: location.get("file")?.as_str()?.to_string(),
            start: location.get("start")?.as_u64()? as u32,
            end: location.get("end")?.as_u64()? as u32,
        })
    }

    /// Check if source code has syntax errors
    async fn check_syntax(
        &self,
        source_code: &str,
    ) -> SmartContractResult<Vec<CompilationWarning>> {
        // Mock syntax checking
        let mut warnings = Vec::new();

        // Check for common issues
        if source_code.contains("pragma solidity") && !source_code.contains("pragma solidity ^") {
            warnings.push(CompilationWarning {
                message: "Consider using caret (^) in pragma directive for better compatibility"
                    .to_string(),
                severity: "warning".to_string(),
                source_location: Some(SourceLocation {
                    file: "contract.sol".to_string(),
                    start: 0,
                    end: 20,
                }),
            });
        }

        if source_code.contains("tx.origin") {
            warnings.push(CompilationWarning {
                message: "Avoid using tx.origin for authorization, use msg.sender instead"
                    .to_string(),
                severity: "warning".to_string(),
                source_location: None,
            });
        }

        Ok(warnings)
    }

    /// Extract contract name from source code
    fn extract_contract_name(&self, source_code: &str) -> String {
        for line in source_code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("contract ") {
                if let Some(name) = trimmed.split_whitespace().nth(1) {
                    return name.trim_end_matches('{').to_string();
                }
            }
        }
        "UnknownContract".to_string()
    }

    /// Generate mock bytecode based on source complexity
    fn generate_mock_bytecode(&self, source_code: &str) -> String {
        let base = "608060405234801561001057600080fd5b50";
        let complexity = source_code.len() / 50; // Simple complexity measure
        let additional_bytes = "00".repeat(complexity.min(200));
        format!("0x{}{}", base, additional_bytes)
    }

    /// Generate mock ABI based on source analysis
    fn generate_mock_abi(&self, source_code: &str) -> serde_json::Value {
        let mut abi = Vec::new();

        // Look for constructor
        if source_code.contains("constructor") {
            abi.push(serde_json::json!({
                "type": "constructor",
                "inputs": [],
                "stateMutability": "nonpayable"
            }));
        }

        // Look for functions
        for line in source_code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("function ") {
                if let Some(func_part) = trimmed.split("function ").nth(1) {
                    if let Some(func_name) = func_part.split('(').next() {
                        let visibility = if trimmed.contains("public") { "public" }
                                       else if trimmed.contains("external") { "external" }
                                       else if trimmed.contains("internal") { "internal" }
                                       else { "private" };

                        let state_mutability = if trimmed.contains("view") { "view" }
                                             else if trimmed.contains("pure") { "pure" }
                                             else if trimmed.contains("payable") { "payable" }
                                             else { "nonpayable" };

                        abi.push(serde_json::json!({
                            "type": "function",
                            "name": func_name.trim(),
                            "inputs": [],
                            "outputs": [],
                            "stateMutability": state_mutability,
                            "visibility": visibility
                        }));
                    }
                }
            }
        }

        serde_json::Value::Array(abi)
    }

    /// Generate mock metadata
    fn generate_mock_metadata(&self, contract_name: &str) -> serde_json::Value {
        serde_json::json!({
            "compiler": {"version": "0.8.19+commit.7dd6d404"},
            "language": "Solidity",
            "output": {
                "abi": [],
                "devdoc": {"kind": "dev", "methods": {}, "version": 1},
                "userdoc": {"kind": "user", "methods": {}, "version": 1}
            },
            "settings": {
                "compilationTarget": {
                    "contract.sol": contract_name
                },
                "evmVersion": "london",
                "libraries": {},
                "metadata": {"bytecodeHash": "ipfs"},
                "optimizer": {"enabled": true, "runs": 200},
                "remappings": []
            },
            "sources": {
                "contract.sol": {
                    "keccak256": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                }
            },
            "version": 1
        })
    }
}

#[async_trait]
impl SmartContractCompiler for SolidityCompiler {
    async fn compile(
        &self,
        source_code: &str,
        config: &CompilerConfig,
    ) -> SmartContractResult<CompilationResult> {
        let start_time = std::time::Instant::now();

        // Validate source code first
        let syntax_warnings = self.check_syntax(source_code).await?;

        // Prepare compilation input
        let input = self.prepare_compilation_input(source_code, config);

        // Mock compilation process - in production, this would call actual solc
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Extract contract name for more realistic output
        let contract_name = self.extract_contract_name(source_code);

        // Generate realistic mock output
        let mock_output = serde_json::json!({
            "contracts": {
                "contract.sol": {
                    [contract_name]: {
                        "abi": self.generate_mock_abi(source_code),
                        "evm": {
                            "bytecode": {
                                "object": self.generate_mock_bytecode(source_code)
                            },
                            "gasEstimates": {
                                "creation": {
                                    "codeDepositCost": "200000",
                                    "executionCost": "infinite"
                                },
                                "external": {
                                    "": "21000"
                                }
                            }
                        },
                        "metadata": self.generate_mock_metadata(&contract_name)
                    }
                }
            }
        });

        let mut result = self.parse_compilation_output(&mock_output.to_string())?;
        result.warnings.extend(syntax_warnings);

        Ok(result)
    }

    async fn compile_batch(
        &self,
        sources: &[SourceFile],
        config: &CompilerConfig,
    ) -> SmartContractResult<BatchCompilationResult> {
        let start_time = std::time::Instant::now();
        let mut contracts = HashMap::new();
        let mut total_warnings = 0;
        let mut total_errors = 0;
        let mut overall_success = true;

        for source in sources {
            match self.compile(&source.content, config).await {
                Ok(result) => {
                    total_warnings += result.warnings.len() as u32;
                    total_errors += result.errors.len() as u32;
                    if !result.success {
                        overall_success = false;
                    }
                    contracts.insert(source.path.clone(), result);
                }
                Err(e) => {
                    overall_success = false;
                    total_errors += 1;
                    contracts.insert(
                        source.path.clone(),
                        CompilationResult {
                            success: false,
                            bytecode: String::new(),
                            abi: serde_json::Value::Null,
                            metadata: serde_json::Value::Null,
                            warnings: Vec::new(),
                            errors: vec![CompilationError {
                                message: e.to_string(),
                                error_type: "BatchCompilationError".to_string(),
                                source_location: None,
                            }],
                            gas_estimates: HashMap::new(),
                        },
                    );
                }
            }
        }

        Ok(BatchCompilationResult {
            contracts,
            overall_success,
            compilation_time_ms: start_time.elapsed().as_millis() as u64,
            total_warnings,
            total_errors,
        })
    }

    async fn get_available_versions(&self) -> SmartContractResult<Vec<String>> {
        // Mock available versions
        Ok(vec![
            "0.8.19".to_string(),
            "0.8.18".to_string(),
            "0.8.17".to_string(),
            "0.8.16".to_string(),
            "0.8.15".to_string(),
            "0.7.6".to_string(),
            "0.6.12".to_string(),
        ])
    }

    async fn validate_syntax(
        &self,
        source_code: &str,
    ) -> SmartContractResult<Vec<CompilationWarning>> {
        self.check_syntax(source_code).await
    }
}

impl VyperCompiler {
    pub fn new(compiler_path: String, cache_dir: String) -> Self {
        Self {
            compiler_path,
            cache_dir,
        }
    }

    /// Extract contract name from Vyper source
    fn extract_contract_name(&self, _source_code: &str) -> String {
        // Vyper doesn't have explicit contract names like Solidity
        "VyperContract".to_string()
    }

    /// Generate mock Vyper bytecode
    fn generate_vyper_bytecode(&self, source_code: &str) -> String {
        let base = "6080604052348015600f57600080fd5b50";
        let complexity = source_code.len() / 40;
        let additional = "00".repeat(complexity.min(150));
        format!("0x{}{}", base, additional)
    }

    /// Generate mock Vyper ABI
    fn generate_vyper_abi(&self, source_code: &str) -> serde_json::Value {
        let mut abi = Vec::new();

        // Look for @external functions
        for line in source_code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("def ") && source_code.contains("@external") {
                if let Some(func_part) = trimmed.split("def ").nth(1) {
                    if let Some(func_name) = func_part.split('(').next() {
                        abi.push(serde_json::json!({
                            "type": "function",
                            "name": func_name.trim(),
                            "inputs": [],
                            "outputs": [],
                            "stateMutability": "nonpayable"
                        }));
                    }
                }
            }
        }

        serde_json::Value::Array(abi)
    }
}

#[async_trait]
impl SmartContractCompiler for VyperCompiler {
    async fn compile(
        &self,
        source_code: &str,
        config: &CompilerConfig,
    ) -> SmartContractResult<CompilationResult> {
        let start_time = std::time::Instant::now();

        // Mock Vyper compilation
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        let contract_name = self.extract_contract_name(source_code);

        Ok(CompilationResult {
            success: true,
            bytecode: self.generate_vyper_bytecode(source_code),
            abi: self.generate_vyper_abi(source_code),
            metadata: serde_json::json!({
                "compiler": {"version": "0.3.7"},
                "language": "Vyper"
            }),
            warnings: Vec::new(),
            errors: Vec::new(),
            gas_estimates: {
                let mut estimates = HashMap::new();
                estimates.insert("creation".to_string(), 150000);
                estimates.insert("external".to_string(), 21000);
                estimates
            },
        })
    }

    async fn compile_batch(
        &self,
        sources: &[SourceFile],
        config: &CompilerConfig,
    ) -> SmartContractResult<BatchCompilationResult> {
        let start_time = std::time::Instant::now();
        let mut contracts = HashMap::new();
        let mut total_warnings = 0;
        let mut total_errors = 0;
        let mut overall_success = true;

        for source in sources {
            match self.compile(&source.content, config).await {
                Ok(result) => {
                    total_warnings += result.warnings.len() as u32;
                    total_errors += result.errors.len() as u32;
                    if !result.success {
                        overall_success = false;
                    }
                    contracts.insert(source.path.clone(), result);
                }
                Err(e) => {
                    overall_success = false;
                    total_errors += 1;
                    contracts.insert(
                        source.path.clone(),
                        CompilationResult {
                            success: false,
                            bytecode: String::new(),
                            abi: serde_json::Value::Null,
                            metadata: serde_json::Value::Null,
                            warnings: Vec::new(),
                            errors: vec![CompilationError {
                                message: e.to_string(),
                                error_type: "VyperCompilationError".to_string(),
                                source_location: None,
                            }],
                            gas_estimates: HashMap::new(),
                        },
                    );
                }
            }
        }

        Ok(BatchCompilationResult {
            contracts,
            overall_success,
            compilation_time_ms: start_time.elapsed().as_millis() as u64,
            total_warnings,
            total_errors,
        })
    }

    async fn get_available_versions(&self) -> SmartContractResult<Vec<String>> {
        Ok(vec![
            "0.3.7".to_string(),
            "0.3.6".to_string(),
            "0.3.5".to_string(),
            "0.3.4".to_string(),
            "0.2.16".to_string(),
        ])
    }

    async fn validate_syntax(
        &self,
        source_code: &str,
    ) -> SmartContractResult<Vec<CompilationWarning>> {
        let mut warnings = Vec::new();

        // Basic Vyper syntax checks
        if !source_code.contains("# @version") {
            warnings.push(CompilationWarning {
                message: "Missing version pragma".to_string(),
                severity: "warning".to_string(),
                source_location: None,
            });
        }

        Ok(warnings)
    }
}

#[async_trait]
impl SmartContractCompiler for VyperCompiler {
    async fn compile(
        &self,
        source_code: &str,
        config: &CompilerConfig,
    ) -> SmartContractResult<CompilationResult> {
        // Mock Vyper compilation
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;

        Ok(CompilationResult {
            success: true,
            bytecode: "0x600160005260206000f3".to_string(),
            abi: serde_json::json!([
                {
                    "name": "get_value",
                    "outputs": [{"name": "", "type": "uint256"}],
                    "inputs": [],
                    "stateMutability": "view",
                    "type": "function"
                }
            ]),
            metadata: serde_json::json!({"compiler": "vyper", "version": "0.3.7"}),
            warnings: Vec::new(),
            errors: Vec::new(),
            gas_estimates: {
                let mut estimates = HashMap::new();
                estimates.insert("creation".to_string(), 150000);
                estimates.insert("external".to_string(), 18000);
                estimates
            },
        })
    }

    async fn compile_batch(
        &self,
        sources: &[SourceFile],
        config: &CompilerConfig,
    ) -> SmartContractResult<BatchCompilationResult> {
        let start_time = std::time::Instant::now();
        let mut contracts = HashMap::new();

        for source in sources {
            let result = self.compile(&source.content, config).await?;
            contracts.insert(source.path.clone(), result);
        }

        Ok(BatchCompilationResult {
            contracts,
            overall_success: true,
            compilation_time_ms: start_time.elapsed().as_millis() as u64,
            total_warnings: 0,
            total_errors: 0,
        })
    }

    async fn get_available_versions(&self) -> SmartContractResult<Vec<String>> {
        Ok(vec![
            "0.3.7".to_string(),
            "0.3.6".to_string(),
            "0.3.5".to_string(),
        ])
    }

    async fn validate_syntax(
        &self,
        source_code: &str,
    ) -> SmartContractResult<Vec<CompilationWarning>> {
        // Mock Vyper syntax validation
        Ok(Vec::new())
    }
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {
            npm_registry: "https://registry.npmjs.org".to_string(),
            github_token: None,
            local_paths: vec!["./node_modules".to_string(), "./lib".to_string()],
        }
    }

    /// Resolve contract dependencies
    pub async fn resolve_dependencies(
        &self,
        dependencies: &[crate::types::ContractDependency],
    ) -> SmartContractResult<HashMap<String, String>> {
        let mut resolved = HashMap::new();

        for dep in dependencies {
            match dep.source {
                crate::types::DependencySource::OpenZeppelin => {
                    resolved.insert(
                        dep.name.clone(),
                        format!("@openzeppelin/contracts/{}", dep.name),
                    );
                }
                crate::types::DependencySource::NPM => {
                    resolved.insert(dep.name.clone(), format!("node_modules/{}", dep.name));
                }
                crate::types::DependencySource::Local => {
                    resolved.insert(dep.name.clone(), format!("./contracts/{}", dep.name));
                }
                _ => {
                    // Handle other sources
                    resolved.insert(dep.name.clone(), dep.name.clone());
                }
            }
        }

        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_solidity_compiler() {
        let compiler = SolidityCompiler::new(
            "solc".to_string(),
            "./cache".to_string(),
            "./temp".to_string(),
        );

        let config = CompilerConfig::default();
        let source_code = r#"
            pragma solidity ^0.8.0;
            contract SimpleContract {
                address public owner;
                constructor() { owner = msg.sender; }
                function getOwner() public view returns (address) { return owner; }
            }
        "#;

        let result = compiler.compile(source_code, &config).await.unwrap();
        assert!(result.success);
        assert!(!result.bytecode.is_empty());
        assert!(!result.abi.is_null());
    }

    #[tokio::test]
    async fn test_vyper_compiler() {
        let compiler = VyperCompiler::new("vyper".to_string(), "./cache".to_string());

        let config = CompilerConfig::default();
        let source_code = r#"
            # @version ^0.3.0
            value: public(uint256)
            
            @external
            def __init__():
                self.value = 42
        "#;

        let result = compiler.compile(source_code, &config).await.unwrap();
        assert!(result.success);
        assert!(!result.bytecode.is_empty());
    }

    #[tokio::test]
    async fn test_syntax_validation() {
        let compiler = SolidityCompiler::new(
            "solc".to_string(),
            "./cache".to_string(),
            "./temp".to_string(),
        );

        let source_with_warning = r#"
            pragma solidity 0.8.0;
            contract Test {
                function auth() public view returns (bool) {
                    return tx.origin == msg.sender;
                }
            }
        "#;

        let warnings = compiler.validate_syntax(source_with_warning).await.unwrap();
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.message.contains("tx.origin")));
    }

    #[tokio::test]
    async fn test_dependency_resolver() {
        let resolver = DependencyResolver::new();

        let dependencies = vec![crate::types::ContractDependency {
            name: "Ownable.sol".to_string(),
            version: "4.8.0".to_string(),
            source: crate::types::DependencySource::OpenZeppelin,
            required: true,
        }];

        let resolved = resolver.resolve_dependencies(&dependencies).await.unwrap();
        assert!(resolved.contains_key("Ownable.sol"));
        assert!(resolved["Ownable.sol"].contains("@openzeppelin"));
    }
}
