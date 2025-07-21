// =====================================================================================
// File: core-blockchain/src/contracts/erc20.rs
// Description: ERC20 token contract interactions
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{BlockchainError, BlockchainResult};
use crate::types::{Address, TransactionHash};
use async_trait::async_trait;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// ERC20 token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: U256,
    pub chain_id: u64,
}

/// ERC20 transfer event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub transaction_hash: TransactionHash,
    pub block_number: u64,
    pub log_index: u64,
}

/// ERC20 approval event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalEvent {
    pub owner: Address,
    pub spender: Address,
    pub value: U256,
    pub transaction_hash: TransactionHash,
    pub block_number: u64,
    pub log_index: u64,
}

/// ERC20 contract interface
#[async_trait]
pub trait ERC20Contract: Send + Sync {
    /// Get token name
    async fn name(&self) -> BlockchainResult<String>;
    
    /// Get token symbol
    async fn symbol(&self) -> BlockchainResult<String>;
    
    /// Get token decimals
    async fn decimals(&self) -> BlockchainResult<u8>;
    
    /// Get total supply
    async fn total_supply(&self) -> BlockchainResult<U256>;
    
    /// Get balance of an address
    async fn balance_of(&self, owner: Address) -> BlockchainResult<U256>;
    
    /// Get allowance
    async fn allowance(&self, owner: Address, spender: Address) -> BlockchainResult<U256>;
    
    /// Transfer tokens
    async fn transfer(&self, to: Address, amount: U256) -> BlockchainResult<TransactionHash>;
    
    /// Transfer tokens from one address to another
    async fn transfer_from(
        &self,
        from: Address,
        to: Address,
        amount: U256,
    ) -> BlockchainResult<TransactionHash>;
    
    /// Approve spender to spend tokens
    async fn approve(&self, spender: Address, amount: U256) -> BlockchainResult<TransactionHash>;
    
    /// Get transfer events
    async fn get_transfer_events(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> BlockchainResult<Vec<TransferEvent>>;
    
    /// Get approval events
    async fn get_approval_events(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> BlockchainResult<Vec<ApprovalEvent>>;
}

/// Ethereum ERC20 contract implementation
pub struct EthereumERC20Contract {
    contract: Contract<Provider<Ws>>,
    address: Address,
    chain_id: u64,
}

impl EthereumERC20Contract {
    /// Create new ERC20 contract instance
    pub fn new(
        address: Address,
        provider: Arc<Provider<Ws>>,
        chain_id: u64,
    ) -> BlockchainResult<Self> {
        let abi = Self::get_erc20_abi();
        let contract = Contract::new(address, abi, provider);
        
        Ok(Self {
            contract,
            address,
            chain_id,
        })
    }
    
    /// Get standard ERC20 ABI
    fn get_erc20_abi() -> Abi {
        // Standard ERC20 ABI
        serde_json::from_str(r#"[
            {
                "constant": true,
                "inputs": [],
                "name": "name",
                "outputs": [{"name": "", "type": "string"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [],
                "name": "symbol",
                "outputs": [{"name": "", "type": "string"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [],
                "name": "decimals",
                "outputs": [{"name": "", "type": "uint8"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [],
                "name": "totalSupply",
                "outputs": [{"name": "", "type": "uint256"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [{"name": "owner", "type": "address"}],
                "name": "balanceOf",
                "outputs": [{"name": "", "type": "uint256"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [
                    {"name": "owner", "type": "address"},
                    {"name": "spender", "type": "address"}
                ],
                "name": "allowance",
                "outputs": [{"name": "", "type": "uint256"}],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "value", "type": "uint256"}
                ],
                "name": "transfer",
                "outputs": [{"name": "", "type": "bool"}],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "from", "type": "address"},
                    {"name": "to", "type": "address"},
                    {"name": "value", "type": "uint256"}
                ],
                "name": "transferFrom",
                "outputs": [{"name": "", "type": "bool"}],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "spender", "type": "address"},
                    {"name": "value", "type": "uint256"}
                ],
                "name": "approve",
                "outputs": [{"name": "", "type": "bool"}],
                "type": "function"
            },
            {
                "anonymous": false,
                "inputs": [
                    {"indexed": true, "name": "from", "type": "address"},
                    {"indexed": true, "name": "to", "type": "address"},
                    {"indexed": false, "name": "value", "type": "uint256"}
                ],
                "name": "Transfer",
                "type": "event"
            },
            {
                "anonymous": false,
                "inputs": [
                    {"indexed": true, "name": "owner", "type": "address"},
                    {"indexed": true, "name": "spender", "type": "address"},
                    {"indexed": false, "name": "value", "type": "uint256"}
                ],
                "name": "Approval",
                "type": "event"
            }
        ]"#).unwrap()
    }
    
    /// Get token information
    pub async fn get_token_info(&self) -> BlockchainResult<TokenInfo> {
        let name = self.name().await?;
        let symbol = self.symbol().await?;
        let decimals = self.decimals().await?;
        let total_supply = self.total_supply().await?;
        
        Ok(TokenInfo {
            address: self.address,
            name,
            symbol,
            decimals,
            total_supply,
            chain_id: self.chain_id,
        })
    }
}

#[async_trait]
impl ERC20Contract for EthereumERC20Contract {
    async fn name(&self) -> BlockchainResult<String> {
        let result: String = self.contract.method("name", ())?.call().await?;
        Ok(result)
    }
    
    async fn symbol(&self) -> BlockchainResult<String> {
        let result: String = self.contract.method("symbol", ())?.call().await?;
        Ok(result)
    }
    
    async fn decimals(&self) -> BlockchainResult<u8> {
        let result: u8 = self.contract.method("decimals", ())?.call().await?;
        Ok(result)
    }
    
    async fn total_supply(&self) -> BlockchainResult<U256> {
        let result: U256 = self.contract.method("totalSupply", ())?.call().await?;
        Ok(result)
    }
    
    async fn balance_of(&self, owner: Address) -> BlockchainResult<U256> {
        let result: U256 = self.contract.method("balanceOf", owner)?.call().await?;
        Ok(result)
    }
    
    async fn allowance(&self, owner: Address, spender: Address) -> BlockchainResult<U256> {
        let result: U256 = self.contract.method("allowance", (owner, spender))?.call().await?;
        Ok(result)
    }
    
    async fn transfer(&self, to: Address, amount: U256) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("transfer", (to, amount))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn transfer_from(
        &self,
        from: Address,
        to: Address,
        amount: U256,
    ) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("transferFrom", (from, to, amount))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn approve(&self, spender: Address, amount: U256) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("approve", (spender, amount))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn get_transfer_events(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> BlockchainResult<Vec<TransferEvent>> {
        let event = self.contract.event::<(Address, Address, U256)>("Transfer")?;
        let mut filter = event.filter();
        
        if let Some(from) = from_block {
            filter = filter.from_block(from);
        }
        
        if let Some(to) = to_block {
            filter = filter.to_block(to);
        }
        
        let logs = filter.query().await?;
        let mut events = Vec::new();
        
        for log in logs {
            let event = TransferEvent {
                from: log.0,
                to: log.1,
                value: log.2,
                transaction_hash: log.meta.transaction_hash,
                block_number: log.meta.block_number.as_u64(),
                log_index: log.meta.log_index.as_u64(),
            };
            events.push(event);
        }
        
        Ok(events)
    }
    
    async fn get_approval_events(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> BlockchainResult<Vec<ApprovalEvent>> {
        let event = self.contract.event::<(Address, Address, U256)>("Approval")?;
        let mut filter = event.filter();
        
        if let Some(from) = from_block {
            filter = filter.from_block(from);
        }
        
        if let Some(to) = to_block {
            filter = filter.to_block(to);
        }
        
        let logs = filter.query().await?;
        let mut events = Vec::new();
        
        for log in logs {
            let event = ApprovalEvent {
                owner: log.0,
                spender: log.1,
                value: log.2,
                transaction_hash: log.meta.transaction_hash,
                block_number: log.meta.block_number.as_u64(),
                log_index: log.meta.log_index.as_u64(),
            };
            events.push(event);
        }
        
        Ok(events)
    }
}

/// ERC20 token manager for handling multiple tokens
pub struct ERC20Manager {
    provider: Arc<Provider<Ws>>,
    chain_id: u64,
}

impl ERC20Manager {
    pub fn new(provider: Arc<Provider<Ws>>, chain_id: u64) -> Self {
        Self { provider, chain_id }
    }
    
    /// Create ERC20 contract instance
    pub fn create_contract(&self, address: Address) -> BlockchainResult<EthereumERC20Contract> {
        EthereumERC20Contract::new(address, self.provider.clone(), self.chain_id)
    }
    
    /// Get token information for multiple addresses
    pub async fn get_tokens_info(&self, addresses: Vec<Address>) -> BlockchainResult<Vec<TokenInfo>> {
        let mut tokens = Vec::new();
        
        for address in addresses {
            let contract = self.create_contract(address)?;
            let info = contract.get_token_info().await?;
            tokens.push(info);
        }
        
        Ok(tokens)
    }
    
    /// Get balances for multiple tokens and addresses
    pub async fn get_balances(
        &self,
        token_addresses: Vec<Address>,
        wallet_addresses: Vec<Address>,
    ) -> BlockchainResult<std::collections::HashMap<(Address, Address), U256>> {
        let mut balances = std::collections::HashMap::new();
        
        for token_address in &token_addresses {
            let contract = self.create_contract(*token_address)?;
            
            for wallet_address in &wallet_addresses {
                let balance = contract.balance_of(*wallet_address).await?;
                balances.insert((*token_address, *wallet_address), balance);
            }
        }
        
        Ok(balances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_info_creation() {
        let token_info = TokenInfo {
            address: Address::zero(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 18,
            total_supply: U256::from(1_000_000_000u64) * U256::exp10(18),
            chain_id: 1,
        };
        
        assert_eq!(token_info.name, "Test Token");
        assert_eq!(token_info.symbol, "TEST");
        assert_eq!(token_info.decimals, 18);
        assert_eq!(token_info.chain_id, 1);
    }
    
    #[test]
    fn test_transfer_event_creation() {
        let event = TransferEvent {
            from: Address::zero(),
            to: Address::from_low_u64_be(1),
            value: U256::from(1000),
            transaction_hash: H256::zero(),
            block_number: 12345,
            log_index: 0,
        };
        
        assert_eq!(event.from, Address::zero());
        assert_eq!(event.to, Address::from_low_u64_be(1));
        assert_eq!(event.value, U256::from(1000));
        assert_eq!(event.block_number, 12345);
    }
    
    #[test]
    fn test_erc20_abi_parsing() {
        let abi = EthereumERC20Contract::get_erc20_abi();
        assert!(!abi.functions.is_empty());
        assert!(!abi.events.is_empty());
        
        // Check for required functions
        assert!(abi.functions.contains_key("name"));
        assert!(abi.functions.contains_key("symbol"));
        assert!(abi.functions.contains_key("decimals"));
        assert!(abi.functions.contains_key("totalSupply"));
        assert!(abi.functions.contains_key("balanceOf"));
        assert!(abi.functions.contains_key("transfer"));
        assert!(abi.functions.contains_key("approve"));
        
        // Check for required events
        assert!(abi.events.contains_key("Transfer"));
        assert!(abi.events.contains_key("Approval"));
    }
}
