// =====================================================================================
// File: core-nft/src/service.rs
// Description: Main NFT service implementation
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{
    error::NFTResult,
    types::*,
    erc721::ERC721Service,
    mock_services::MockERC721Service,
    NFTServiceConfig,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

/// Main NFT service that orchestrates all NFT-related operations
pub struct NFTService {
    config: Arc<RwLock<NFTServiceConfig>>,
    erc721_service: Arc<dyn ERC721Service>,
}

impl NFTService {
    /// Create a new NFT service instance
    pub fn new(config: NFTServiceConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            erc721_service: Arc::new(MockERC721Service),
        }
    }

    /// Create NFT service with shared config
    pub fn with_config(config: Arc<RwLock<NFTServiceConfig>>) -> Self {
        Self {
            config: config.clone(),
            erc721_service: Arc::new(MockERC721Service),
        }
    }

    /// Get current configuration
    pub async fn get_config(&self) -> NFTServiceConfig {
        self.config.read().await.clone()
    }

    /// Get a specific NFT token
    pub async fn get_token(&self, contract_address: &str, token_id: &str) -> NFTResult<NFT> {
        debug!("Getting token {} from contract {}", token_id, contract_address);

        // Use ERC721 service for now
        self.erc721_service.get_token(contract_address, token_id).await
    }

    /// Get tokens by owner
    pub async fn get_tokens_by_owner(&self, owner: &str) -> NFTResult<Vec<NFT>> {
        debug!("Getting tokens for owner {}", owner);

        // Use ERC721 service for now
        self.erc721_service.get_tokens_by_owner(owner).await
    }

    /// Get tokens by collection
    pub async fn get_tokens_by_collection(&self, contract_address: &str) -> NFTResult<Vec<NFT>> {
        debug!("Getting tokens for collection {}", contract_address);

        // Use ERC721 service for now
        self.erc721_service.get_tokens_by_collection(contract_address).await
    }

    /// Transfer token
    pub async fn transfer_token(&self, request: TransferRequest) -> NFTResult<String> {
        info!("Transferring token {} from {} to {}",
              request.token_id, request.from, request.to);

        // Use ERC721 service for now
        self.erc721_service.transfer_token(request).await
    }

    /// Approve token
    pub async fn approve_token(&self, request: ApprovalRequest) -> NFTResult<String> {
        info!("Approving token {} for {}", request.token_id, request.approved);

        // Use ERC721 service for now
        self.erc721_service.approve_token(request).await
    }

    /// Health check
    pub async fn health_check(&self) -> NFTResult<()> {
        debug!("Performing health check");

        // Check ERC721 service
        self.erc721_service.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nft_service_creation() {
        let config = NFTServiceConfig::default();
        let service = NFTService::new(config);

        // Test health check
        assert!(service.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_get_token() {
        let config = NFTServiceConfig::default();
        let service = NFTService::new(config);

        let result = service.get_token("0x1234567890123456789012345678901234567890", "1").await;
        assert!(result.is_ok());

        let token = result.unwrap();
        assert_eq!(token.token_id, "1");
        assert_eq!(token.contract_address, "0x1234567890123456789012345678901234567890");
        assert_eq!(token.standard, NFTStandard::ERC721);
    }

    #[tokio::test]
    async fn test_get_tokens_by_owner() {
        let config = NFTServiceConfig::default();
        let service = NFTService::new(config);

        let result = service.get_tokens_by_owner("0x1234567890123456789012345678901234567890").await;
        assert!(result.is_ok());

        let tokens = result.unwrap();
        assert_eq!(tokens.len(), 0); // Mock service returns empty list
    }

    #[tokio::test]
    async fn test_transfer_token() {
        let config = NFTServiceConfig::default();
        let service = NFTService::new(config);

        let request = TransferRequest {
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "1".to_string(),
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: Some(1),
        };

        let result = service.transfer_token(request).await;
        assert!(result.is_ok());

        let tx_hash = result.unwrap();
        assert!(tx_hash.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_approve_token() {
        let config = NFTServiceConfig::default();
        let service = NFTService::new(config);

        let request = ApprovalRequest {
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "1".to_string(),
            owner: "0x1111111111111111111111111111111111111111".to_string(),
            approved: "0x2222222222222222222222222222222222222222".to_string(),
        };

        let result = service.approve_token(request).await;
        assert!(result.is_ok());

        let tx_hash = result.unwrap();
        assert!(tx_hash.starts_with("0x"));
    }

    #[tokio::test]
    async fn test_nft_service_integration() {
        let config = NFTServiceConfig::default();
        let service = NFTService::new(config);

        // Test health check
        assert!(service.health_check().await.is_ok());

        // Test getting a token
        let token_result = service.get_token("0x1234567890123456789012345678901234567890", "1").await;
        assert!(token_result.is_ok());

        let token = token_result.unwrap();
        assert_eq!(token.standard, NFTStandard::ERC721);
        assert_eq!(token.token_id, "1");
        assert!(!token.is_burned);
        assert!(token.is_transferable);

        // Test getting tokens by owner
        let owner_tokens = service.get_tokens_by_owner("0x1234567890123456789012345678901234567890").await;
        assert!(owner_tokens.is_ok());

        // Test getting tokens by collection
        let collection_tokens = service.get_tokens_by_collection("0x1234567890123456789012345678901234567890").await;
        assert!(collection_tokens.is_ok());

        // Test transfer
        let transfer_request = TransferRequest {
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "1".to_string(),
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: None,
        };

        let transfer_result = service.transfer_token(transfer_request).await;
        assert!(transfer_result.is_ok());

        // Test approval
        let approval_request = ApprovalRequest {
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
            token_id: "1".to_string(),
            owner: "0x1111111111111111111111111111111111111111".to_string(),
            approved: "0x2222222222222222222222222222222222222222".to_string(),
        };

        let approval_result = service.approve_token(approval_request).await;
        assert!(approval_result.is_ok());
    }

    #[tokio::test]
    async fn test_nft_service_config() {
        let mut config = NFTServiceConfig::default();
        config.network = "polygon".to_string();
        config.chain_id = 137;
        config.max_file_size_mb = 50;

        let service = NFTService::new(config.clone());
        let retrieved_config = service.get_config().await;

        assert_eq!(retrieved_config.network, "polygon");
        assert_eq!(retrieved_config.chain_id, 137);
        assert_eq!(retrieved_config.max_file_size_mb, 50);
    }
}