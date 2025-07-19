// =====================================================================================
// File: core-blockchain/src/ethereum.rs
// Description: Production-grade Ethereum blockchain adapter
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::traits::*;
use crate::types::*;
use crate::BlockchainError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ethers::{
    core::types::{Address as EthAddress, H256, U256},
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider, ProviderError},
    signers::{LocalWallet, Signer},
    types::{BlockNumber, TransactionRequest},
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Production Ethereum adapter
pub struct EthereumAdapter {
    provider: Arc<Provider<Http>>,
    network: BlockchainNetwork,
    chain_id: u64,
    signer: Option<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
}

impl EthereumAdapter {
    /// Create a new Ethereum adapter
    pub async fn new(
        rpc_url: &str,
        network: BlockchainNetwork,
        private_key: Option<&str>,
    ) -> Result<Self, BlockchainError> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| BlockchainError::ConnectionError(format!("Failed to create provider: {}", e)))?;
        
        let provider = Arc::new(provider);
        
        // Get chain ID
        let chain_id = provider
            .get_chainid()
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get chain ID: {}", e)))?
            .as_u64();
        
        // Setup signer if private key is provided
        let signer = if let Some(pk) = private_key {
            let wallet = LocalWallet::from_str(pk)
                .map_err(|e| BlockchainError::ConfigurationError(format!("Invalid private key: {}", e)))?
                .with_chain_id(chain_id);
            
            let signer = SignerMiddleware::new(provider.clone(), wallet);
            Some(Arc::new(signer))
        } else {
            None
        };
        
        info!("Ethereum adapter created for network: {:?}, chain_id: {}", network, chain_id);
        
        Ok(Self {
            provider,
            network,
            chain_id,
            signer,
        })
    }
    
    /// Convert ethers Address to our Address type
    fn to_address(&self, eth_addr: EthAddress) -> Address {
        Address::new(format!("{:?}", eth_addr), self.network.clone())
    }
    
    /// Convert our Address to ethers Address
    fn from_address(&self, addr: &Address) -> Result<EthAddress, BlockchainError> {
        EthAddress::from_str(&addr.value)
            .map_err(|e| BlockchainError::ParsingError(format!("Invalid Ethereum address: {}", e)))
    }
    
    /// Convert wei to our standard unit (assuming 18 decimals)
    fn wei_to_standard(&self, wei: U256) -> u64 {
        // Convert wei to standard units (this is a simplified conversion)
        // In production, you'd want proper decimal handling
        (wei / U256::from(10u64.pow(18))).as_u64()
    }
    
    /// Convert standard unit to wei
    fn standard_to_wei(&self, amount: u64) -> U256 {
        U256::from(amount) * U256::from(10u64.pow(18))
    }
}

#[async_trait]
impl BlockchainAdapter for EthereumAdapter {
    async fn connect(&self) -> Result<(), BlockchainError> {
        // Test connection by getting latest block number
        self.provider
            .get_block_number()
            .await
            .map_err(|e| BlockchainError::ConnectionError(format!("Connection test failed: {}", e)))?;
        
        info!("Successfully connected to Ethereum network");
        Ok(())
    }
    
    async fn disconnect(&self) -> Result<(), BlockchainError> {
        // HTTP provider doesn't need explicit disconnection
        info!("Disconnected from Ethereum network");
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        self.provider.get_block_number().await.is_ok()
    }
    
    fn network(&self) -> BlockchainNetwork {
        self.network.clone()
    }
    
    async fn get_network_stats(&self) -> Result<NetworkStats, BlockchainError> {
        let latest_block = self.provider
            .get_block_number()
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get latest block: {}", e)))?
            .as_u64();
        
        let gas_price = self.provider
            .get_gas_price()
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get gas price: {}", e)))?
            .as_u64();
        
        Ok(NetworkStats {
            network: self.network.clone(),
            latest_block,
            average_block_time: 12.0, // Ethereum average block time
            gas_price: Some(gas_price),
            total_transactions: 0, // Would need additional API calls
            active_addresses: 0,   // Would need additional API calls
            last_updated: Utc::now(),
        })
    }
}

#[async_trait]
impl WalletOperations for EthereumAdapter {
    async fn get_balance(&self, address: &Address) -> Result<Balance, BlockchainError> {
        let eth_addr = self.from_address(address)?;
        
        let balance_wei = self.provider
            .get_balance(eth_addr, None)
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get balance: {}", e)))?;
        
        let balance_standard = self.wei_to_standard(balance_wei);
        
        Ok(Balance {
            address: address.clone(),
            amount: balance_standard,
            token_balances: HashMap::new(), // TODO: Implement token balance fetching
            last_updated: Utc::now(),
        })
    }
    
    async fn get_balances(&self, addresses: &[Address]) -> Result<Vec<Balance>, BlockchainError> {
        let mut balances = Vec::new();
        
        for address in addresses {
            let balance = self.get_balance(address).await?;
            balances.push(balance);
        }
        
        Ok(balances)
    }
    
    async fn get_wallet_info(&self, address: &Address) -> Result<Wallet, BlockchainError> {
        let balance = self.get_balance(address).await?;
        let eth_addr = self.from_address(address)?;
        
        let nonce = self.provider
            .get_transaction_count(eth_addr, None)
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get nonce: {}", e)))?
            .as_u64();
        
        let code = self.provider
            .get_code(eth_addr, None)
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get code: {}", e)))?;
        
        let is_contract = !code.is_empty();
        
        Ok(Wallet {
            address: address.clone(),
            balance,
            nonce: Some(nonce),
            is_contract,
            last_activity: None, // Would need transaction history analysis
        })
    }
    
    fn validate_address(&self, address: &str) -> bool {
        EthAddress::from_str(address).is_ok()
    }
    
    async fn generate_address(&self) -> Result<Address, BlockchainError> {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        Ok(self.to_address(wallet.address()))
    }
}

#[async_trait]
impl TransactionOperations for EthereumAdapter {
    async fn send_transaction(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
        options: Option<TransactionOptions>,
    ) -> Result<TransactionHash, BlockchainError> {
        let signer = self.signer.as_ref()
            .ok_or_else(|| BlockchainError::ConfigurationError("No signer configured".to_string()))?;
        
        let to_addr = self.from_address(to)?;
        let amount_wei = self.standard_to_wei(amount);
        
        let mut tx = TransactionRequest::new()
            .to(to_addr)
            .value(amount_wei);
        
        if let Some(opts) = options {
            if let Some(gas_limit) = opts.gas_limit {
                tx = tx.gas(gas_limit);
            }
            if let Some(gas_price) = opts.gas_price {
                tx = tx.gas_price(gas_price);
            }
        }
        
        let pending_tx = signer
            .send_transaction(tx, None)
            .await
            .map_err(|e| BlockchainError::TransactionFailed(format!("Failed to send transaction: {}", e)))?;
        
        let tx_hash = TransactionHash {
            value: format!("{:?}", pending_tx.tx_hash()),
            network: self.network.clone(),
        };
        
        info!("Transaction sent: {}", tx_hash.value);
        Ok(tx_hash)
    }
    
    async fn get_transaction(&self, hash: &TransactionHash) -> Result<Option<Transaction>, BlockchainError> {
        let tx_hash = H256::from_str(&hash.value)
            .map_err(|e| BlockchainError::ParsingError(format!("Invalid transaction hash: {}", e)))?;
        
        let tx = self.provider
            .get_transaction(tx_hash)
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get transaction: {}", e)))?;
        
        if let Some(tx) = tx {
            let receipt = self.provider
                .get_transaction_receipt(tx_hash)
                .await
                .map_err(|e| BlockchainError::NetworkError(format!("Failed to get receipt: {}", e)))?;
            
            let status = if let Some(receipt) = &receipt {
                if receipt.status == Some(1.into()) {
                    TransactionStatus::Confirmed
                } else {
                    TransactionStatus::Failed
                }
            } else {
                TransactionStatus::Pending
            };
            
            let transaction = Transaction {
                hash: hash.clone(),
                from: self.to_address(tx.from),
                to: self.to_address(tx.to.unwrap_or_default()),
                amount: self.wei_to_standard(tx.value),
                fee: self.wei_to_standard(tx.gas_price.unwrap_or_default() * tx.gas),
                status,
                block_number: tx.block_number.map(|n| n.as_u64()),
                timestamp: None, // Would need block timestamp
                confirmations: 0, // Would need calculation
                metadata: HashMap::new(),
            };
            
            Ok(Some(transaction))
        } else {
            Ok(None)
        }
    }
    
    async fn get_transaction_status(&self, hash: &TransactionHash) -> Result<TransactionStatus, BlockchainError> {
        if let Some(tx) = self.get_transaction(hash).await? {
            Ok(tx.status)
        } else {
            Ok(TransactionStatus::Dropped)
        }
    }
    
    async fn get_transaction_history(
        &self,
        address: &Address,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Transaction>, BlockchainError> {
        // This would require using an indexing service like Etherscan API
        // For now, return empty vector
        warn!("Transaction history not implemented - requires indexing service");
        Ok(Vec::new())
    }
    
    async fn estimate_fee(
        &self,
        from: &Address,
        to: &Address,
        amount: u64,
    ) -> Result<FeeEstimate, BlockchainError> {
        let gas_price = self.provider
            .get_gas_price()
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get gas price: {}", e)))?
            .as_u64();
        
        let gas_limit = 21000u64; // Standard ETH transfer gas limit
        
        let mut estimated_time = HashMap::new();
        estimated_time.insert("slow".to_string(), 300u64);     // 5 minutes
        estimated_time.insert("standard".to_string(), 180u64); // 3 minutes
        estimated_time.insert("fast".to_string(), 60u64);      // 1 minute
        
        Ok(FeeEstimate {
            network: self.network.clone(),
            slow: gas_price * gas_limit / 2,      // Half gas price
            standard: gas_price * gas_limit,      // Current gas price
            fast: gas_price * gas_limit * 2,      // Double gas price
            estimated_time_seconds: estimated_time,
            last_updated: Utc::now(),
        })
    }
    
    async fn wait_for_confirmation(
        &self,
        hash: &TransactionHash,
        confirmations: u32,
        timeout_seconds: u64,
    ) -> Result<Transaction, BlockchainError> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_seconds);
        
        loop {
            if start_time.elapsed() > timeout {
                return Err(BlockchainError::TimeoutError(
                    "Transaction confirmation timeout".to_string()
                ));
            }
            
            if let Some(tx) = self.get_transaction(hash).await? {
                if tx.confirmations >= confirmations {
                    return Ok(tx);
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ethereum_adapter_creation() {
        // This test would require a test RPC endpoint
        // For now, just test address validation
        let adapter = EthereumAdapter {
            provider: Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap()),
            network: BlockchainNetwork::EthereumTestnet,
            chain_id: 1,
            signer: None,
        };
        
        assert!(adapter.validate_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"));
        assert!(!adapter.validate_address("invalid_address"));
    }
    
    #[test]
    fn test_wei_conversion() {
        let adapter = EthereumAdapter {
            provider: Arc::new(Provider::<Http>::try_from("http://localhost:8545").unwrap()),
            network: BlockchainNetwork::EthereumTestnet,
            chain_id: 1,
            signer: None,
        };
        
        let wei = U256::from(1000000000000000000u64); // 1 ETH in wei
        let standard = adapter.wei_to_standard(wei);
        assert_eq!(standard, 1);
        
        let back_to_wei = adapter.standard_to_wei(1);
        assert_eq!(back_to_wei, wei);
    }
}
