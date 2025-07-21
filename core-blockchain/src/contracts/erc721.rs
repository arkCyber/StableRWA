// =====================================================================================
// File: core-blockchain/src/contracts/erc721.rs
// Description: ERC721 NFT contract interactions
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::error::{BlockchainError, BlockchainResult};
use crate::types::{Address, TransactionHash};
use async_trait::async_trait;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// NFT metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub image: String,
    pub external_url: Option<String>,
    pub attributes: Vec<NFTAttribute>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

/// NFT attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTAttribute {
    pub trait_type: String,
    pub value: serde_json::Value,
    pub display_type: Option<String>,
    pub max_value: Option<u64>,
}

/// NFT information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTInfo {
    pub token_id: U256,
    pub contract_address: Address,
    pub owner: Address,
    pub approved: Option<Address>,
    pub token_uri: String,
    pub metadata: Option<NFTMetadata>,
}

/// NFT transfer event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTTransferEvent {
    pub from: Address,
    pub to: Address,
    pub token_id: U256,
    pub transaction_hash: TransactionHash,
    pub block_number: u64,
    pub log_index: u64,
}

/// NFT approval event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTApprovalEvent {
    pub owner: Address,
    pub approved: Address,
    pub token_id: U256,
    pub transaction_hash: TransactionHash,
    pub block_number: u64,
    pub log_index: u64,
}

/// ERC721 contract interface
#[async_trait]
pub trait ERC721Contract: Send + Sync {
    /// Get contract name
    async fn name(&self) -> BlockchainResult<String>;
    
    /// Get contract symbol
    async fn symbol(&self) -> BlockchainResult<String>;
    
    /// Get total supply
    async fn total_supply(&self) -> BlockchainResult<U256>;
    
    /// Get token URI
    async fn token_uri(&self, token_id: U256) -> BlockchainResult<String>;
    
    /// Get owner of token
    async fn owner_of(&self, token_id: U256) -> BlockchainResult<Address>;
    
    /// Get balance of owner
    async fn balance_of(&self, owner: Address) -> BlockchainResult<U256>;
    
    /// Get approved address for token
    async fn get_approved(&self, token_id: U256) -> BlockchainResult<Address>;
    
    /// Check if operator is approved for all tokens of owner
    async fn is_approved_for_all(&self, owner: Address, operator: Address) -> BlockchainResult<bool>;
    
    /// Approve address to transfer token
    async fn approve(&self, to: Address, token_id: U256) -> BlockchainResult<TransactionHash>;
    
    /// Set approval for all tokens
    async fn set_approval_for_all(&self, operator: Address, approved: bool) -> BlockchainResult<TransactionHash>;
    
    /// Transfer token
    async fn transfer_from(
        &self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> BlockchainResult<TransactionHash>;
    
    /// Safe transfer token
    async fn safe_transfer_from(
        &self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> BlockchainResult<TransactionHash>;
    
    /// Safe transfer token with data
    async fn safe_transfer_from_with_data(
        &self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Vec<u8>,
    ) -> BlockchainResult<TransactionHash>;
    
    /// Get transfer events
    async fn get_transfer_events(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> BlockchainResult<Vec<NFTTransferEvent>>;
    
    /// Get approval events
    async fn get_approval_events(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> BlockchainResult<Vec<NFTApprovalEvent>>;
}

/// Ethereum ERC721 contract implementation
pub struct EthereumERC721Contract {
    contract: Contract<Provider<Ws>>,
    address: Address,
    chain_id: u64,
}

impl EthereumERC721Contract {
    /// Create new ERC721 contract instance
    pub fn new(
        address: Address,
        provider: Arc<Provider<Ws>>,
        chain_id: u64,
    ) -> BlockchainResult<Self> {
        let abi = Self::get_erc721_abi();
        let contract = Contract::new(address, abi, provider);
        
        Ok(Self {
            contract,
            address,
            chain_id,
        })
    }
    
    /// Get standard ERC721 ABI
    fn get_erc721_abi() -> Abi {
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
                "name": "totalSupply",
                "outputs": [{"name": "", "type": "uint256"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [{"name": "tokenId", "type": "uint256"}],
                "name": "tokenURI",
                "outputs": [{"name": "", "type": "string"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [{"name": "tokenId", "type": "uint256"}],
                "name": "ownerOf",
                "outputs": [{"name": "", "type": "address"}],
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
                "inputs": [{"name": "tokenId", "type": "uint256"}],
                "name": "getApproved",
                "outputs": [{"name": "", "type": "address"}],
                "type": "function"
            },
            {
                "constant": true,
                "inputs": [
                    {"name": "owner", "type": "address"},
                    {"name": "operator", "type": "address"}
                ],
                "name": "isApprovedForAll",
                "outputs": [{"name": "", "type": "bool"}],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "tokenId", "type": "uint256"}
                ],
                "name": "approve",
                "outputs": [],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "operator", "type": "address"},
                    {"name": "approved", "type": "bool"}
                ],
                "name": "setApprovalForAll",
                "outputs": [],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "from", "type": "address"},
                    {"name": "to", "type": "address"},
                    {"name": "tokenId", "type": "uint256"}
                ],
                "name": "transferFrom",
                "outputs": [],
                "type": "function"
            },
            {
                "constant": false,
                "inputs": [
                    {"name": "from", "type": "address"},
                    {"name": "to", "type": "address"},
                    {"name": "tokenId", "type": "uint256"}
                ],
                "name": "safeTransferFrom",
                "outputs": [],
                "type": "function"
            },
            {
                "anonymous": false,
                "inputs": [
                    {"indexed": true, "name": "from", "type": "address"},
                    {"indexed": true, "name": "to", "type": "address"},
                    {"indexed": true, "name": "tokenId", "type": "uint256"}
                ],
                "name": "Transfer",
                "type": "event"
            },
            {
                "anonymous": false,
                "inputs": [
                    {"indexed": true, "name": "owner", "type": "address"},
                    {"indexed": true, "name": "approved", "type": "address"},
                    {"indexed": true, "name": "tokenId", "type": "uint256"}
                ],
                "name": "Approval",
                "type": "event"
            }
        ]"#).unwrap()
    }
    
    /// Get NFT information
    pub async fn get_nft_info(&self, token_id: U256) -> BlockchainResult<NFTInfo> {
        let owner = self.owner_of(token_id).await?;
        let approved = self.get_approved(token_id).await.ok();
        let token_uri = self.token_uri(token_id).await?;
        
        // Fetch metadata from URI (simplified)
        let metadata = self.fetch_metadata(&token_uri).await.ok();
        
        Ok(NFTInfo {
            token_id,
            contract_address: self.address,
            owner,
            approved,
            token_uri,
            metadata,
        })
    }
    
    /// Fetch metadata from URI
    async fn fetch_metadata(&self, uri: &str) -> BlockchainResult<NFTMetadata> {
        let client = reqwest::Client::new();
        let response = client.get(uri).send().await?;
        let metadata: NFTMetadata = response.json().await?;
        Ok(metadata)
    }
}

#[async_trait]
impl ERC721Contract for EthereumERC721Contract {
    async fn name(&self) -> BlockchainResult<String> {
        let result: String = self.contract.method("name", ())?.call().await?;
        Ok(result)
    }
    
    async fn symbol(&self) -> BlockchainResult<String> {
        let result: String = self.contract.method("symbol", ())?.call().await?;
        Ok(result)
    }
    
    async fn total_supply(&self) -> BlockchainResult<U256> {
        let result: U256 = self.contract.method("totalSupply", ())?.call().await?;
        Ok(result)
    }
    
    async fn token_uri(&self, token_id: U256) -> BlockchainResult<String> {
        let result: String = self.contract.method("tokenURI", token_id)?.call().await?;
        Ok(result)
    }
    
    async fn owner_of(&self, token_id: U256) -> BlockchainResult<Address> {
        let result: Address = self.contract.method("ownerOf", token_id)?.call().await?;
        Ok(result)
    }
    
    async fn balance_of(&self, owner: Address) -> BlockchainResult<U256> {
        let result: U256 = self.contract.method("balanceOf", owner)?.call().await?;
        Ok(result)
    }
    
    async fn get_approved(&self, token_id: U256) -> BlockchainResult<Address> {
        let result: Address = self.contract.method("getApproved", token_id)?.call().await?;
        Ok(result)
    }
    
    async fn is_approved_for_all(&self, owner: Address, operator: Address) -> BlockchainResult<bool> {
        let result: bool = self.contract.method("isApprovedForAll", (owner, operator))?.call().await?;
        Ok(result)
    }
    
    async fn approve(&self, to: Address, token_id: U256) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("approve", (to, token_id))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn set_approval_for_all(&self, operator: Address, approved: bool) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("setApprovalForAll", (operator, approved))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn transfer_from(
        &self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("transferFrom", (from, to, token_id))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn safe_transfer_from(
        &self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("safeTransferFrom", (from, to, token_id))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn safe_transfer_from_with_data(
        &self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Vec<u8>,
    ) -> BlockchainResult<TransactionHash> {
        let tx = self.contract.method("safeTransferFrom", (from, to, token_id, data))?.send().await?;
        Ok(tx.tx_hash())
    }
    
    async fn get_transfer_events(
        &self,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> BlockchainResult<Vec<NFTTransferEvent>> {
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
            let event = NFTTransferEvent {
                from: log.0,
                to: log.1,
                token_id: log.2,
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
    ) -> BlockchainResult<Vec<NFTApprovalEvent>> {
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
            let event = NFTApprovalEvent {
                owner: log.0,
                approved: log.1,
                token_id: log.2,
                transaction_hash: log.meta.transaction_hash,
                block_number: log.meta.block_number.as_u64(),
                log_index: log.meta.log_index.as_u64(),
            };
            events.push(event);
        }
        
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nft_metadata() {
        let metadata = NFTMetadata {
            name: "Test NFT".to_string(),
            description: "A test NFT".to_string(),
            image: "https://example.com/image.png".to_string(),
            external_url: Some("https://example.com".to_string()),
            attributes: vec![
                NFTAttribute {
                    trait_type: "Color".to_string(),
                    value: serde_json::Value::String("Blue".to_string()),
                    display_type: None,
                    max_value: None,
                },
            ],
            background_color: Some("#0000FF".to_string()),
            animation_url: None,
            youtube_url: None,
        };
        
        assert_eq!(metadata.name, "Test NFT");
        assert_eq!(metadata.attributes.len(), 1);
        assert_eq!(metadata.attributes[0].trait_type, "Color");
    }
}
