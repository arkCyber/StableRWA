// =====================================================================================
// File: core-nft/examples/basic_usage.rs
// Description: Basic usage example for the core-nft library
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use core_nft::{
    NFTService, NFTServiceConfig, StorageProvider,
    TransferRequest, ApprovalRequest,
};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting NFT service example");

    // Create a custom configuration
    let mut config = NFTServiceConfig::default();
    config.network = "polygon".to_string();
    config.chain_id = 137;
    config.storage_provider = StorageProvider::IPFS;
    config.max_file_size_mb = 50;

    // Initialize the NFT service
    let nft_service = NFTService::new(config);

    // Perform health check
    info!("Performing health check...");
    match nft_service.health_check().await {
        Ok(_) => info!("✅ NFT service is healthy"),
        Err(e) => {
            error!("❌ Health check failed: {:?}", e);
            return Err(e.into());
        }
    }

    // Example contract and token
    let contract_address = "0x1234567890123456789012345678901234567890";
    let token_id = "1";
    let owner_address = "0x1111111111111111111111111111111111111111";
    let recipient_address = "0x2222222222222222222222222222222222222222";

    // Get token information
    info!("Getting token information...");
    match nft_service.get_token(contract_address, token_id).await {
        Ok(token) => {
            info!("✅ Token retrieved successfully:");
            info!("  - Token ID: {}", token.token_id);
            info!("  - Contract: {}", token.contract_address);
            info!("  - Standard: {:?}", token.standard);
            info!("  - Owner: {}", token.owner);
            info!("  - Creator: {}", token.creator);
            info!("  - Name: {}", token.metadata.name);
            info!("  - Is Transferable: {}", token.is_transferable);
            info!("  - Is Burned: {}", token.is_burned);
        }
        Err(e) => {
            error!("❌ Failed to get token: {:?}", e);
        }
    }

    // Get tokens by owner
    info!("Getting tokens by owner...");
    match nft_service.get_tokens_by_owner(owner_address).await {
        Ok(tokens) => {
            info!("✅ Found {} tokens for owner {}", tokens.len(), owner_address);
            for (i, token) in tokens.iter().enumerate() {
                info!("  {}. Token {} - {}", i + 1, token.token_id, token.metadata.name);
            }
        }
        Err(e) => {
            error!("❌ Failed to get tokens by owner: {:?}", e);
        }
    }

    // Get tokens by collection
    info!("Getting tokens by collection...");
    match nft_service.get_tokens_by_collection(contract_address).await {
        Ok(tokens) => {
            info!("✅ Found {} tokens in collection {}", tokens.len(), contract_address);
        }
        Err(e) => {
            error!("❌ Failed to get tokens by collection: {:?}", e);
        }
    }

    // Transfer token example
    info!("Simulating token transfer...");
    let transfer_request = TransferRequest {
        contract_address: contract_address.to_string(),
        token_id: token_id.to_string(),
        from: owner_address.to_string(),
        to: recipient_address.to_string(),
        amount: None, // ERC721 doesn't need amount
    };

    match nft_service.transfer_token(transfer_request).await {
        Ok(tx_hash) => {
            info!("✅ Token transfer initiated successfully");
            info!("  - Transaction Hash: {}", tx_hash);
        }
        Err(e) => {
            error!("❌ Failed to transfer token: {:?}", e);
        }
    }

    // Approve token example
    info!("Simulating token approval...");
    let approval_request = ApprovalRequest {
        contract_address: contract_address.to_string(),
        token_id: token_id.to_string(),
        owner: owner_address.to_string(),
        approved: recipient_address.to_string(),
    };

    match nft_service.approve_token(approval_request).await {
        Ok(tx_hash) => {
            info!("✅ Token approval set successfully");
            info!("  - Transaction Hash: {}", tx_hash);
        }
        Err(e) => {
            error!("❌ Failed to approve token: {:?}", e);
        }
    }

    // Display service configuration
    info!("Current service configuration:");
    let current_config = nft_service.get_config().await;
    info!("  - Network: {}", current_config.network);
    info!("  - Chain ID: {}", current_config.chain_id);
    info!("  - Storage Provider: {:?}", current_config.storage_provider);
    info!("  - Max File Size: {} MB", current_config.max_file_size_mb);
    info!("  - Marketplace Fee: {}%", current_config.marketplace_fee_percentage);
    info!("  - Max Royalty: {}%", current_config.max_royalty_percentage);
    info!("  - Batch Operations: {}", current_config.enable_batch_operations);
    info!("  - Lazy Minting: {}", current_config.enable_lazy_minting);
    info!("  - Gasless Transactions: {}", current_config.enable_gasless_transactions);

    info!("🎉 NFT service example completed successfully!");

    Ok(())
}

// Helper function to demonstrate error handling
async fn demonstrate_error_handling(nft_service: &NFTService) {
    info!("Demonstrating error handling...");

    // Try to get a non-existent token
    match nft_service.get_token("0xinvalid", "999").await {
        Ok(_) => info!("Unexpected success"),
        Err(e) => info!("Expected error caught: {:?}", e),
    }

    // Try invalid transfer
    let invalid_transfer = TransferRequest {
        contract_address: "invalid_address".to_string(),
        token_id: "1".to_string(),
        from: "invalid_from".to_string(),
        to: "invalid_to".to_string(),
        amount: None,
    };

    match nft_service.transfer_token(invalid_transfer).await {
        Ok(_) => info!("Unexpected success"),
        Err(e) => info!("Expected validation error: {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_runs() {
        let config = NFTServiceConfig::default();
        let service = NFTService::new(config);
        
        // Just test that the service can be created and health check passes
        assert!(service.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_custom_config() {
        let mut config = NFTServiceConfig::default();
        config.network = "testnet".to_string();
        config.chain_id = 5; // Goerli
        
        let service = NFTService::new(config);
        let retrieved_config = service.get_config().await;
        
        assert_eq!(retrieved_config.network, "testnet");
        assert_eq!(retrieved_config.chain_id, 5);
    }
}
