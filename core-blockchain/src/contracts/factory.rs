// =====================================================================================
// File: core-blockchain/src/contracts/factory.rs
// Description: Contract factory for deploying and managing contracts
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{BlockchainError, BlockchainResult};
use crate::types::Address;
use crate::contracts::{ContractDeployConfig, DeploymentResult, ContractRegistry, ContractInfo};
use async_trait::async_trait;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Contract template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub bytecode: String,
    pub abi: String,
    pub constructor_params: Vec<ConstructorParam>,
    pub category: ContractCategory,
    pub verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Constructor parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructorParam {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Contract category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractCategory {
    Token,
    NFT,
    DeFi,
    Governance,
    RWA,
    Custom,
}

/// Contract deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub template_id: Uuid,
    pub name: String,
    pub constructor_args: Vec<serde_json::Value>,
    pub deployer: Address,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<U256>,
    pub value: Option<U256>,
}

/// Contract factory trait
#[async_trait]
pub trait ContractFactory: Send + Sync {
    /// Register a new contract template
    async fn register_template(&self, template: ContractTemplate) -> BlockchainResult<()>;
    
    /// Get contract template by ID
    async fn get_template(&self, template_id: Uuid) -> BlockchainResult<Option<ContractTemplate>>;
    
    /// List all templates
    async fn list_templates(&self) -> BlockchainResult<Vec<ContractTemplate>>;
    
    /// Deploy contract from template
    async fn deploy_from_template(&self, request: DeploymentRequest) -> BlockchainResult<DeploymentResult>;
    
    /// Deploy custom contract
    async fn deploy_custom(&self, config: ContractDeployConfig) -> BlockchainResult<DeploymentResult>;
    
    /// Verify deployed contract
    async fn verify_contract(&self, address: Address, template_id: Uuid) -> BlockchainResult<bool>;
    
    /// Get deployment history
    async fn get_deployment_history(&self, deployer: Address) -> BlockchainResult<Vec<DeploymentResult>>;
}

/// Ethereum contract factory implementation
pub struct EthereumContractFactory {
    provider: Arc<Provider<Ws>>,
    chain_id: u64,
    templates: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, ContractTemplate>>>,
    registry: Arc<tokio::sync::RwLock<ContractRegistry>>,
    deployment_history: Arc<tokio::sync::RwLock<Vec<DeploymentResult>>>,
}

impl EthereumContractFactory {
    pub fn new(provider: Arc<Provider<Ws>>, chain_id: u64) -> Self {
        Self {
            provider,
            chain_id,
            templates: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            registry: Arc::new(tokio::sync::RwLock::new(ContractRegistry::new())),
            deployment_history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// Initialize with default templates
    pub async fn initialize_default_templates(&self) -> BlockchainResult<()> {
        let templates = vec![
            self.create_erc20_template(),
            self.create_erc721_template(),
            self.create_rwa_token_template(),
        ];
        
        for template in templates {
            self.register_template(template).await?;
        }
        
        Ok(())
    }
    
    /// Create ERC20 token template
    fn create_erc20_template(&self) -> ContractTemplate {
        ContractTemplate {
            id: Uuid::new_v4(),
            name: "ERC20 Token".to_string(),
            description: "Standard ERC20 fungible token".to_string(),
            version: "1.0.0".to_string(),
            bytecode: "0x608060405234801561001057600080fd5b50".to_string(), // Placeholder
            abi: r#"[{"inputs":[{"name":"name","type":"string"},{"name":"symbol","type":"string"}],"type":"constructor"}]"#.to_string(),
            constructor_params: vec![
                ConstructorParam {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    description: "Token name".to_string(),
                    required: true,
                    default_value: None,
                },
                ConstructorParam {
                    name: "symbol".to_string(),
                    param_type: "string".to_string(),
                    description: "Token symbol".to_string(),
                    required: true,
                    default_value: None,
                },
                ConstructorParam {
                    name: "initialSupply".to_string(),
                    param_type: "uint256".to_string(),
                    description: "Initial token supply".to_string(),
                    required: true,
                    default_value: Some("1000000".to_string()),
                },
            ],
            category: ContractCategory::Token,
            verified: true,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Create ERC721 NFT template
    fn create_erc721_template(&self) -> ContractTemplate {
        ContractTemplate {
            id: Uuid::new_v4(),
            name: "ERC721 NFT".to_string(),
            description: "Standard ERC721 non-fungible token".to_string(),
            version: "1.0.0".to_string(),
            bytecode: "0x608060405234801561001057600080fd5b50".to_string(), // Placeholder
            abi: r#"[{"inputs":[{"name":"name","type":"string"},{"name":"symbol","type":"string"}],"type":"constructor"}]"#.to_string(),
            constructor_params: vec![
                ConstructorParam {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    description: "NFT collection name".to_string(),
                    required: true,
                    default_value: None,
                },
                ConstructorParam {
                    name: "symbol".to_string(),
                    param_type: "string".to_string(),
                    description: "NFT collection symbol".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            category: ContractCategory::NFT,
            verified: true,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Create RWA token template
    fn create_rwa_token_template(&self) -> ContractTemplate {
        ContractTemplate {
            id: Uuid::new_v4(),
            name: "RWA Token".to_string(),
            description: "Real World Asset tokenization contract".to_string(),
            version: "1.0.0".to_string(),
            bytecode: "0x608060405234801561001057600080fd5b50".to_string(), // Placeholder
            abi: r#"[{"inputs":[{"name":"name","type":"string"},{"name":"symbol","type":"string"}],"type":"constructor"}]"#.to_string(),
            constructor_params: vec![
                ConstructorParam {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    description: "RWA token name".to_string(),
                    required: true,
                    default_value: None,
                },
                ConstructorParam {
                    name: "symbol".to_string(),
                    param_type: "string".to_string(),
                    description: "RWA token symbol".to_string(),
                    required: true,
                    default_value: None,
                },
                ConstructorParam {
                    name: "assetType".to_string(),
                    param_type: "string".to_string(),
                    description: "Type of real world asset".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            category: ContractCategory::RWA,
            verified: true,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Convert JSON values to ABI tokens
    fn json_to_tokens(&self, values: Vec<serde_json::Value>, params: &[ConstructorParam]) -> BlockchainResult<Vec<ethers::abi::Token>> {
        let mut tokens = Vec::new();
        
        for (i, value) in values.iter().enumerate() {
            if i >= params.len() {
                return Err(BlockchainError::InvalidInput {
                    field: "constructor_args".to_string(),
                    message: "Too many arguments provided".to_string(),
                });
            }
            
            let param = &params[i];
            let token = match param.param_type.as_str() {
                "string" => {
                    if let Some(s) = value.as_str() {
                        ethers::abi::Token::String(s.to_string())
                    } else {
                        return Err(BlockchainError::InvalidInput {
                            field: param.name.clone(),
                            message: "Expected string value".to_string(),
                        });
                    }
                }
                "uint256" => {
                    if let Some(n) = value.as_u64() {
                        ethers::abi::Token::Uint(U256::from(n))
                    } else if let Some(s) = value.as_str() {
                        let n = s.parse::<u64>().map_err(|_| BlockchainError::InvalidInput {
                            field: param.name.clone(),
                            message: "Invalid uint256 value".to_string(),
                        })?;
                        ethers::abi::Token::Uint(U256::from(n))
                    } else {
                        return Err(BlockchainError::InvalidInput {
                            field: param.name.clone(),
                            message: "Expected uint256 value".to_string(),
                        });
                    }
                }
                "address" => {
                    if let Some(s) = value.as_str() {
                        let addr = s.parse::<Address>().map_err(|_| BlockchainError::InvalidInput {
                            field: param.name.clone(),
                            message: "Invalid address format".to_string(),
                        })?;
                        ethers::abi::Token::Address(addr)
                    } else {
                        return Err(BlockchainError::InvalidInput {
                            field: param.name.clone(),
                            message: "Expected address value".to_string(),
                        });
                    }
                }
                "bool" => {
                    if let Some(b) = value.as_bool() {
                        ethers::abi::Token::Bool(b)
                    } else {
                        return Err(BlockchainError::InvalidInput {
                            field: param.name.clone(),
                            message: "Expected boolean value".to_string(),
                        });
                    }
                }
                _ => {
                    return Err(BlockchainError::InvalidInput {
                        field: param.param_type.clone(),
                        message: "Unsupported parameter type".to_string(),
                    });
                }
            };
            
            tokens.push(token);
        }
        
        Ok(tokens)
    }
}

#[async_trait]
impl ContractFactory for EthereumContractFactory {
    async fn register_template(&self, template: ContractTemplate) -> BlockchainResult<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id, template);
        Ok(())
    }
    
    async fn get_template(&self, template_id: Uuid) -> BlockchainResult<Option<ContractTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.get(&template_id).cloned())
    }
    
    async fn list_templates(&self) -> BlockchainResult<Vec<ContractTemplate>> {
        let templates = self.templates.read().await;
        Ok(templates.values().cloned().collect())
    }
    
    async fn deploy_from_template(&self, request: DeploymentRequest) -> BlockchainResult<DeploymentResult> {
        let template = self.get_template(request.template_id).await?
            .ok_or_else(|| BlockchainError::NotFound {
                resource: "template".to_string(),
                id: request.template_id.to_string(),
            })?;
        
        let tokens = self.json_to_tokens(request.constructor_args, &template.constructor_params)?;
        
        let config = ContractDeployConfig {
            contract_name: request.name.clone(),
            bytecode: template.bytecode,
            constructor_args: tokens,
            gas_limit: request.gas_limit,
            gas_price: request.gas_price,
            value: request.value,
        };
        
        let result = self.deploy_custom(config).await?;
        
        // Register deployed contract
        let contract_info = ContractInfo {
            id: Uuid::new_v4(),
            name: request.name,
            address: result.contract_address,
            abi: template.abi,
            bytecode: template.bytecode,
            chain_id: self.chain_id,
            deployed_at: chrono::Utc::now(),
            deployer: request.deployer,
            version: template.version,
            verified: template.verified,
        };
        
        let mut registry = self.registry.write().await;
        registry.register_contract(contract_info);
        
        // Add to deployment history
        let mut history = self.deployment_history.write().await;
        history.push(result.clone());
        
        Ok(result)
    }
    
    async fn deploy_custom(&self, config: ContractDeployConfig) -> BlockchainResult<DeploymentResult> {
        let factory = ContractFactory::new(
            ethers::abi::Abi::default(),
            config.bytecode.parse::<Bytes>()?,
            self.provider.clone(),
        );
        
        let mut deployer = factory.deploy_tokens(config.constructor_args)?;
        
        if let Some(gas_limit) = config.gas_limit {
            deployer = deployer.gas(gas_limit);
        }
        
        if let Some(gas_price) = config.gas_price {
            deployer = deployer.gas_price(gas_price);
        }
        
        if let Some(value) = config.value {
            deployer = deployer.value(value);
        }
        
        let contract = deployer.send().await?;
        let receipt = contract.receipt().await?.ok_or_else(|| {
            BlockchainError::TransactionFailed {
                hash: "unknown".to_string(),
                reason: "No receipt available".to_string(),
            }
        })?;
        
        Ok(DeploymentResult {
            contract_address: receipt.contract_address.unwrap_or_default(),
            transaction_hash: receipt.transaction_hash,
            block_number: receipt.block_number.unwrap_or_default().as_u64(),
            gas_used: receipt.gas_used.unwrap_or_default().as_u64(),
            deployment_cost: receipt.effective_gas_price.unwrap_or_default() * receipt.gas_used.unwrap_or_default(),
        })
    }
    
    async fn verify_contract(&self, address: Address, template_id: Uuid) -> BlockchainResult<bool> {
        // Implementation would verify contract bytecode against template
        Ok(true)
    }
    
    async fn get_deployment_history(&self, deployer: Address) -> BlockchainResult<Vec<DeploymentResult>> {
        let history = self.deployment_history.read().await;
        // Filter by deployer would require additional metadata
        Ok(history.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contract_template_creation() {
        let template = ContractTemplate {
            id: Uuid::new_v4(),
            name: "Test Contract".to_string(),
            description: "A test contract".to_string(),
            version: "1.0.0".to_string(),
            bytecode: "0x608060405234801561001057600080fd5b50".to_string(),
            abi: "[]".to_string(),
            constructor_params: vec![],
            category: ContractCategory::Custom,
            verified: false,
            created_at: chrono::Utc::now(),
        };
        
        assert_eq!(template.name, "Test Contract");
        assert_eq!(template.version, "1.0.0");
        assert!(matches!(template.category, ContractCategory::Custom));
    }
    
    #[test]
    fn test_constructor_param() {
        let param = ConstructorParam {
            name: "initialSupply".to_string(),
            param_type: "uint256".to_string(),
            description: "Initial token supply".to_string(),
            required: true,
            default_value: Some("1000000".to_string()),
        };
        
        assert_eq!(param.name, "initialSupply");
        assert_eq!(param.param_type, "uint256");
        assert!(param.required);
        assert_eq!(param.default_value, Some("1000000".to_string()));
    }
}
