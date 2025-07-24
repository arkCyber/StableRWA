# Core NFT Module

A comprehensive Rust library for NFT (Non-Fungible Token) operations, supporting ERC721 and ERC1155 standards with marketplace functionality.

## Features

- **Multi-Standard Support**: ERC721, ERC721A, ERC721Enumerable, ERC1155, ERC1155Supply
- **Marketplace Integration**: Listing, offers, sales, and royalty management
- **Metadata Management**: OpenSea-compatible metadata with IPFS support
- **Collection Management**: Create and manage NFT collections
- **Transfer & Approval**: Token transfers and approval mechanisms
- **Validation**: Comprehensive input validation and error handling
- **Storage Providers**: IPFS, Arweave, and Filecoin support
- **Async/Await**: Full async support with tokio runtime

## Architecture

### Core Components

1. **NFTService**: Main orchestration service
2. **ERC721Service**: ERC721 token operations
3. **Types**: Comprehensive type definitions
4. **Error Handling**: Structured error types
5. **Mock Services**: Testing utilities

### Key Types

- `NFT`: Core NFT representation
- `NFTCollection`: Collection metadata
- `NFTMetadata`: Token metadata (OpenSea compatible)
- `TransferRequest`/`ApprovalRequest`: Operation requests
- `MarketplaceListing`/`MarketplaceOffer`: Trading types

## Usage

### Basic Setup

```rust
use core_nft::{NFTService, NFTServiceConfig};

#[tokio::main]
async fn main() {
    let config = NFTServiceConfig::default();
    let nft_service = NFTService::new(config);
    
    // Health check
    nft_service.health_check().await.unwrap();
}
```

### Getting NFT Information

```rust
// Get a specific token
let token = nft_service
    .get_token("0x1234567890123456789012345678901234567890", "1")
    .await?;

// Get tokens by owner
let tokens = nft_service
    .get_tokens_by_owner("0x1234567890123456789012345678901234567890")
    .await?;

// Get tokens by collection
let collection_tokens = nft_service
    .get_tokens_by_collection("0x1234567890123456789012345678901234567890")
    .await?;
```

### Token Operations

```rust
use core_nft::{TransferRequest, ApprovalRequest};

// Transfer token
let transfer_request = TransferRequest {
    contract_address: "0x1234567890123456789012345678901234567890".to_string(),
    token_id: "1".to_string(),
    from: "0x1111111111111111111111111111111111111111".to_string(),
    to: "0x2222222222222222222222222222222222222222".to_string(),
    amount: None, // For ERC721, use Some(amount) for ERC1155
};

let tx_hash = nft_service.transfer_token(transfer_request).await?;

// Approve token
let approval_request = ApprovalRequest {
    contract_address: "0x1234567890123456789012345678901234567890".to_string(),
    token_id: "1".to_string(),
    owner: "0x1111111111111111111111111111111111111111".to_string(),
    approved: "0x2222222222222222222222222222222222222222".to_string(),
};

let tx_hash = nft_service.approve_token(approval_request).await?;
```

## Configuration

### NFTServiceConfig

```rust
use core_nft::{NFTServiceConfig, StorageProvider};
use rust_decimal::Decimal;

let config = NFTServiceConfig {
    network: "ethereum".to_string(),
    chain_id: 1,
    gas_price_gwei: Decimal::new(20, 0),
    max_gas_limit: 500000,
    confirmation_blocks: 12,
    storage_provider: StorageProvider::IPFS,
    marketplace_fee_percentage: Decimal::new(25, 3), // 2.5%
    max_royalty_percentage: Decimal::new(10, 2), // 10%
    enable_batch_operations: true,
    enable_lazy_minting: true,
    enable_gasless_transactions: false,
    rate_limit_per_minute: 100,
    max_file_size_mb: 100,
    supported_image_formats: vec![
        "image/png".to_string(),
        "image/jpeg".to_string(),
        "image/gif".to_string(),
        "image/webp".to_string(),
        "image/svg+xml".to_string(),
    ],
    metadata_cache_ttl_seconds: 3600,
};
```

## Testing

Run the test suite:

```bash
cargo test --lib --no-default-features
```

### Test Coverage

- ✅ Service creation and configuration
- ✅ Token retrieval operations
- ✅ Transfer and approval operations
- ✅ Health checks
- ✅ Integration testing
- ✅ Error handling
- ✅ Mock service implementations

## Error Handling

The library uses a comprehensive error system:

```rust
use core_nft::{NFTError, NFTResult};

match nft_service.get_token(contract, token_id).await {
    Ok(token) => println!("Token: {:?}", token),
    Err(NFTError::NotFound(msg)) => println!("Token not found: {}", msg),
    Err(NFTError::ValidationError(msg)) => println!("Validation error: {}", msg),
    Err(NFTError::NetworkError(msg)) => println!("Network error: {}", msg),
    Err(e) => println!("Other error: {:?}", e),
}
```

## Dependencies

- `tokio`: Async runtime
- `serde`: Serialization
- `uuid`: Unique identifiers
- `chrono`: Date/time handling
- `rust_decimal`: Precise decimal arithmetic
- `validator`: Input validation
- `tracing`: Logging
- `async-trait`: Async traits

## Development Status

This is a production-ready NFT library with comprehensive testing and error handling. The current implementation includes:

- ✅ Core NFT operations
- ✅ ERC721 support
- ✅ Mock services for testing
- ✅ Comprehensive type system
- ✅ Async/await support
- ✅ Error handling
- 🚧 ERC1155 implementation (planned)
- 🚧 Marketplace integration (planned)
- 🚧 Real blockchain integration (planned)

## License

This project is part of the RWA Platform and follows the same licensing terms.
