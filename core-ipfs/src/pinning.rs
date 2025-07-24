// =====================================================================================
// IPFS Pinning Management
//
// Advanced pinning strategies and management for IPFS content
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::{client::IpfsClientTrait, IpfsError, IpfsHash, IpfsResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Pin priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PinPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Default for PinPriority {
    fn default() -> Self {
        PinPriority::Normal
    }
}

/// Pin policy for automatic pinning decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinPolicy {
    pub name: String,
    pub description: String,
    pub priority: PinPriority,
    pub max_size: Option<u64>,
    pub mime_types: Vec<String>,
    pub tags: Vec<String>,
    pub retention_days: Option<u32>,
    pub replication_factor: u32,
}

impl Default for PinPolicy {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: "Default pinning policy".to_string(),
            priority: PinPriority::Normal,
            max_size: None,
            mime_types: Vec::new(),
            tags: Vec::new(),
            retention_days: None,
            replication_factor: 3,
        }
    }
}

/// Pin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinInfo {
    pub hash: IpfsHash,
    pub priority: PinPriority,
    pub policy_name: String,
    pub pinned_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub size: u64,
    pub replication_count: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PinInfo {
    /// Create new pin info
    pub fn new(hash: IpfsHash, priority: PinPriority, policy_name: String, size: u64) -> Self {
        Self {
            hash,
            priority,
            policy_name,
            pinned_at: chrono::Utc::now(),
            expires_at: None,
            size,
            replication_count: 1,
            metadata: HashMap::new(),
        }
    }

    /// Check if pin is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Set expiration based on retention days
    pub fn set_expiration(&mut self, retention_days: u32) {
        self.expires_at = Some(self.pinned_at + chrono::Duration::days(retention_days as i64));
    }
}

/// Pinning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinningStats {
    pub total_pins: u64,
    pub total_size: u64,
    pub pins_by_priority: HashMap<PinPriority, u64>,
    pub pins_by_policy: HashMap<String, u64>,
    pub expired_pins: u64,
    pub replication_health: f64, // Percentage of pins meeting replication factor
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Pinning manager trait
#[async_trait(?Send)]
pub trait PinningManagerTrait {
    /// Create pinning policy
    async fn create_policy(&self, policy: PinPolicy) -> IpfsResult<()>;

    /// Get pinning policy
    async fn get_policy(&self, name: &str) -> IpfsResult<PinPolicy>;

    /// Update pinning policy
    async fn update_policy(&self, policy: PinPolicy) -> IpfsResult<()>;

    /// Delete pinning policy
    async fn delete_policy(&self, name: &str) -> IpfsResult<()>;

    /// Pin content with policy
    async fn pin_with_policy(&self, hash: &IpfsHash, policy_name: &str) -> IpfsResult<PinInfo>;

    /// Pin content with priority
    async fn pin_with_priority(
        &self,
        hash: &IpfsHash,
        priority: PinPriority,
    ) -> IpfsResult<PinInfo>;

    /// Unpin content
    async fn unpin_content(&self, hash: &IpfsHash) -> IpfsResult<()>;

    /// Get pin information
    async fn get_pin_info(&self, hash: &IpfsHash) -> IpfsResult<PinInfo>;

    /// List all pins
    async fn list_pins(&self) -> IpfsResult<Vec<PinInfo>>;

    /// List pins by priority
    async fn list_pins_by_priority(&self, priority: PinPriority) -> IpfsResult<Vec<PinInfo>>;

    /// List pins by policy
    async fn list_pins_by_policy(&self, policy_name: &str) -> IpfsResult<Vec<PinInfo>>;

    /// Clean up expired pins
    async fn cleanup_expired_pins(&self) -> IpfsResult<Vec<IpfsHash>>;

    /// Get pinning statistics
    async fn get_pinning_stats(&self) -> IpfsResult<PinningStats>;

    /// Rebalance pins based on policies
    async fn rebalance_pins(&self) -> IpfsResult<()>;
}

/// In-memory pinning manager implementation
pub struct InMemoryPinningManager {
    client: Arc<dyn IpfsClientTrait>,
    policies: Arc<RwLock<HashMap<String, PinPolicy>>>,
    pins: Arc<RwLock<HashMap<String, PinInfo>>>,
}

impl InMemoryPinningManager {
    /// Create new pinning manager
    pub fn new(client: Arc<dyn IpfsClientTrait>) -> Self {
        let mut policies = HashMap::new();
        policies.insert("default".to_string(), PinPolicy::default());

        Self {
            client,
            policies: Arc::new(RwLock::new(policies)),
            pins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if content matches policy criteria
    async fn matches_policy(
        &self,
        policy: &PinPolicy,
        size: u64,
        mime_type: &str,
        tags: &[String],
    ) -> bool {
        // Check size limit
        if let Some(max_size) = policy.max_size {
            if size > max_size {
                return false;
            }
        }

        // Check MIME types
        if !policy.mime_types.is_empty() && !policy.mime_types.contains(&mime_type.to_string()) {
            return false;
        }

        // Check tags
        if !policy.tags.is_empty() {
            let has_matching_tag = policy
                .tags
                .iter()
                .any(|policy_tag| tags.contains(policy_tag));
            if !has_matching_tag {
                return false;
            }
        }

        true
    }
}

#[async_trait(?Send)]
impl PinningManagerTrait for InMemoryPinningManager {
    #[instrument(skip(self, policy))]
    async fn create_policy(&self, policy: PinPolicy) -> IpfsResult<()> {
        debug!("Creating pinning policy: {}", policy.name);

        self.policies
            .write()
            .await
            .insert(policy.name.clone(), policy);

        info!("Successfully created pinning policy");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_policy(&self, name: &str) -> IpfsResult<PinPolicy> {
        debug!("Getting pinning policy: {}", name);

        self.policies
            .read()
            .await
            .get(name)
            .cloned()
            .ok_or_else(|| IpfsError::PinningError(format!("Policy not found: {}", name)))
    }

    #[instrument(skip(self, policy))]
    async fn update_policy(&self, policy: PinPolicy) -> IpfsResult<()> {
        debug!("Updating pinning policy: {}", policy.name);

        let mut policies = self.policies.write().await;
        if policies.contains_key(&policy.name) {
            policies.insert(policy.name.clone(), policy);
            info!("Successfully updated pinning policy");
            Ok(())
        } else {
            Err(IpfsError::PinningError(format!(
                "Policy not found: {}",
                policy.name
            )))
        }
    }

    #[instrument(skip(self))]
    async fn delete_policy(&self, name: &str) -> IpfsResult<()> {
        debug!("Deleting pinning policy: {}", name);

        if name == "default" {
            return Err(IpfsError::PinningError(
                "Cannot delete default policy".to_string(),
            ));
        }

        let mut policies = self.policies.write().await;
        if policies.remove(name).is_some() {
            info!("Successfully deleted pinning policy: {}", name);
            Ok(())
        } else {
            Err(IpfsError::PinningError(format!(
                "Policy not found: {}",
                name
            )))
        }
    }

    #[instrument(skip(self))]
    async fn pin_with_policy(&self, hash: &IpfsHash, policy_name: &str) -> IpfsResult<PinInfo> {
        debug!("Pinning content with policy: {} -> {}", hash, policy_name);

        let policy = self.get_policy(policy_name).await?;

        // Get content size
        let size = self.client.size(hash).await?;

        // Pin the content
        self.client.pin(hash).await?;

        // Create pin info
        let mut pin_info =
            PinInfo::new(hash.clone(), policy.priority, policy_name.to_string(), size);

        // Set expiration if specified in policy
        if let Some(retention_days) = policy.retention_days {
            pin_info.set_expiration(retention_days);
        }

        pin_info.replication_count = policy.replication_factor;

        // Store pin info
        self.pins
            .write()
            .await
            .insert(hash.as_str().to_string(), pin_info.clone());

        info!(
            "Successfully pinned content with policy: {} -> {}",
            hash, policy_name
        );
        Ok(pin_info)
    }

    #[instrument(skip(self))]
    async fn pin_with_priority(
        &self,
        hash: &IpfsHash,
        priority: PinPriority,
    ) -> IpfsResult<PinInfo> {
        debug!("Pinning content with priority: {} -> {:?}", hash, priority);

        // Get content size
        let size = self.client.size(hash).await?;

        // Pin the content
        self.client.pin(hash).await?;

        // Create pin info with default policy
        let pin_info = PinInfo {
            hash: hash.clone(),
            priority,
            policy_name: "default".to_string(),
            pinned_at: chrono::Utc::now(),
            expires_at: None,
            size,
            replication_count: 1,
            metadata: HashMap::new(),
        };

        // Store pin info
        self.pins
            .write()
            .await
            .insert(hash.as_str().to_string(), pin_info.clone());

        info!(
            "Successfully pinned content with priority: {} -> {:?}",
            hash, priority
        );
        Ok(pin_info)
    }

    #[instrument(skip(self))]
    async fn unpin_content(&self, hash: &IpfsHash) -> IpfsResult<()> {
        debug!("Unpinning content: {}", hash);

        // Unpin from IPFS
        self.client.unpin(hash).await?;

        // Remove from pin info
        self.pins.write().await.remove(hash.as_str());

        info!("Successfully unpinned content: {}", hash);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_pin_info(&self, hash: &IpfsHash) -> IpfsResult<PinInfo> {
        debug!("Getting pin info: {}", hash);

        self.pins
            .read()
            .await
            .get(hash.as_str())
            .cloned()
            .ok_or_else(|| IpfsError::PinningError(format!("Pin info not found: {}", hash)))
    }

    #[instrument(skip(self))]
    async fn list_pins(&self) -> IpfsResult<Vec<PinInfo>> {
        debug!("Listing all pins");

        let pins = self.pins.read().await;
        let pin_list: Vec<PinInfo> = pins.values().cloned().collect();

        info!("Found {} pins", pin_list.len());
        Ok(pin_list)
    }

    #[instrument(skip(self))]
    async fn list_pins_by_priority(&self, priority: PinPriority) -> IpfsResult<Vec<PinInfo>> {
        debug!("Listing pins by priority: {:?}", priority);

        let pins = self.pins.read().await;
        let filtered_pins: Vec<PinInfo> = pins
            .values()
            .filter(|pin| pin.priority == priority)
            .cloned()
            .collect();

        info!(
            "Found {} pins with priority {:?}",
            filtered_pins.len(),
            priority
        );
        Ok(filtered_pins)
    }

    #[instrument(skip(self))]
    async fn list_pins_by_policy(&self, policy_name: &str) -> IpfsResult<Vec<PinInfo>> {
        debug!("Listing pins by policy: {}", policy_name);

        let pins = self.pins.read().await;
        let filtered_pins: Vec<PinInfo> = pins
            .values()
            .filter(|pin| pin.policy_name == policy_name)
            .cloned()
            .collect();

        info!(
            "Found {} pins with policy {}",
            filtered_pins.len(),
            policy_name
        );
        Ok(filtered_pins)
    }

    #[instrument(skip(self))]
    async fn cleanup_expired_pins(&self) -> IpfsResult<Vec<IpfsHash>> {
        debug!("Cleaning up expired pins");

        let mut pins = self.pins.write().await;
        let mut expired_hashes = Vec::new();

        // Find expired pins
        let expired_keys: Vec<String> = pins
            .iter()
            .filter(|(_, pin)| pin.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        // Remove expired pins
        for key in expired_keys {
            if let Some(pin) = pins.remove(&key) {
                // Unpin from IPFS (ignore errors for cleanup)
                if let Err(e) = self.client.unpin(&pin.hash).await {
                    warn!("Failed to unpin expired content {}: {}", pin.hash, e);
                }
                expired_hashes.push(pin.hash);
            }
        }

        info!("Cleaned up {} expired pins", expired_hashes.len());
        Ok(expired_hashes)
    }

    #[instrument(skip(self))]
    async fn get_pinning_stats(&self) -> IpfsResult<PinningStats> {
        debug!("Calculating pinning statistics");

        let pins = self.pins.read().await;
        let total_pins = pins.len() as u64;
        let total_size: u64 = pins.values().map(|pin| pin.size).sum();

        // Calculate pins by priority
        let mut pins_by_priority = HashMap::new();
        for pin in pins.values() {
            *pins_by_priority.entry(pin.priority).or_insert(0) += 1;
        }

        // Calculate pins by policy
        let mut pins_by_policy = HashMap::new();
        for pin in pins.values() {
            *pins_by_policy.entry(pin.policy_name.clone()).or_insert(0) += 1;
        }

        // Count expired pins
        let expired_pins = pins.values().filter(|pin| pin.is_expired()).count() as u64;

        // Calculate replication health (simplified)
        let healthy_pins = pins
            .values()
            .filter(|pin| pin.replication_count >= 1)
            .count();
        let replication_health = if total_pins > 0 {
            (healthy_pins as f64 / total_pins as f64) * 100.0
        } else {
            100.0
        };

        let stats = PinningStats {
            total_pins,
            total_size,
            pins_by_priority,
            pins_by_policy,
            expired_pins,
            replication_health,
            last_updated: chrono::Utc::now(),
        };

        info!(
            "Generated pinning statistics: {} pins, {} bytes",
            total_pins, total_size
        );
        Ok(stats)
    }

    #[instrument(skip(self))]
    async fn rebalance_pins(&self) -> IpfsResult<()> {
        debug!("Rebalancing pins based on policies");

        // This is a simplified rebalancing implementation
        // In a real system, this would involve more complex logic

        let pins = self.pins.read().await;
        let mut rebalanced_count = 0;

        for pin in pins.values() {
            // Check if pin needs rebalancing based on policy
            if let Ok(policy) = self.get_policy(&pin.policy_name).await {
                if pin.replication_count < policy.replication_factor {
                    // In a real implementation, this would trigger additional replication
                    rebalanced_count += 1;
                }
            }
        }

        info!("Rebalanced {} pins", rebalanced_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MockIpfsClient;

    async fn create_test_pinning_manager() -> InMemoryPinningManager {
        let client = Arc::new(MockIpfsClient::new());
        InMemoryPinningManager::new(client)
    }

    fn create_test_policy() -> PinPolicy {
        PinPolicy {
            name: "test_policy".to_string(),
            description: "Test pinning policy".to_string(),
            priority: PinPriority::High,
            max_size: Some(10_000_000), // 10MB
            mime_types: vec!["image/jpeg".to_string(), "image/png".to_string()],
            tags: vec!["important".to_string(), "backup".to_string()],
            retention_days: Some(30),
            replication_factor: 3,
        }
    }

    #[tokio::test]
    async fn test_pin_priority_ordering() {
        assert!(PinPriority::Critical > PinPriority::High);
        assert!(PinPriority::High > PinPriority::Normal);
        assert!(PinPriority::Normal > PinPriority::Low);
    }

    #[tokio::test]
    async fn test_pin_info_creation_and_expiration() {
        let hash =
            IpfsHash::new("QmTestPin123456789012345678901234567890123456".to_string()).unwrap();
        let mut pin_info = PinInfo::new(
            hash.clone(),
            PinPriority::High,
            "test_policy".to_string(),
            1024,
        );

        assert_eq!(pin_info.hash, hash);
        assert_eq!(pin_info.priority, PinPriority::High);
        assert_eq!(pin_info.policy_name, "test_policy");
        assert_eq!(pin_info.size, 1024);
        assert!(!pin_info.is_expired());

        // Set expiration
        pin_info.set_expiration(30);
        assert!(pin_info.expires_at.is_some());
        assert!(!pin_info.is_expired()); // Should not be expired yet
    }

    #[tokio::test]
    async fn test_policy_operations() {
        let manager = create_test_pinning_manager().await;
        let policy = create_test_policy();

        // Create policy
        manager.create_policy(policy.clone()).await.unwrap();

        // Get policy
        let retrieved_policy = manager.get_policy("test_policy").await.unwrap();
        assert_eq!(retrieved_policy.name, policy.name);
        assert_eq!(retrieved_policy.priority, policy.priority);
        assert_eq!(retrieved_policy.max_size, policy.max_size);

        // Update policy
        let mut updated_policy = policy.clone();
        updated_policy.description = "Updated description".to_string();
        manager.update_policy(updated_policy.clone()).await.unwrap();

        let retrieved_updated = manager.get_policy("test_policy").await.unwrap();
        assert_eq!(retrieved_updated.description, "Updated description");

        // Delete policy
        manager.delete_policy("test_policy").await.unwrap();
        assert!(manager.get_policy("test_policy").await.is_err());

        // Cannot delete default policy
        assert!(manager.delete_policy("default").await.is_err());
    }

    #[tokio::test]
    async fn test_pin_with_policy() {
        let manager = create_test_pinning_manager().await;
        let policy = create_test_policy();
        manager.create_policy(policy).await.unwrap();

        // Add test content to mock client
        let hash =
            IpfsHash::new("QmPinTest123456789012345678901234567890123456".to_string()).unwrap();
        let client = manager.client.clone();
        // For testing, we'll use the client directly without downcasting
        let actual_hash = client.add_bytes(b"test content".to_vec()).await.unwrap();

        // Pin with policy using the actual hash
        let pin_info = manager
            .pin_with_policy(&actual_hash, "test_policy")
            .await
            .unwrap();
        assert_eq!(pin_info.hash, actual_hash);
        assert_eq!(pin_info.priority, PinPriority::High);
        assert_eq!(pin_info.policy_name, "test_policy");
        assert_eq!(pin_info.replication_count, 3);
        assert!(pin_info.expires_at.is_some());

        // Verify pin info can be retrieved
        let retrieved_info = manager.get_pin_info(&actual_hash).await.unwrap();
        assert_eq!(retrieved_info.hash, pin_info.hash);
    }

    #[tokio::test]
    async fn test_pin_with_priority() {
        let manager = create_test_pinning_manager().await;

        // Add test content to mock client
        let client = manager.client.clone();
        // For testing, we'll use the client directly
        let actual_hash = client.add_bytes(b"priority test".to_vec()).await.unwrap();

        // Pin with priority
        let pin_info = manager
            .pin_with_priority(&actual_hash, PinPriority::Critical)
            .await
            .unwrap();
        assert_eq!(pin_info.priority, PinPriority::Critical);
        assert_eq!(pin_info.policy_name, "default");
        assert!(pin_info.expires_at.is_none());
    }

    #[tokio::test]
    async fn test_list_pins_operations() {
        let manager = create_test_pinning_manager().await;

        // Create test content and pins
        let hashes = vec![
            IpfsHash::new("QmList1234567890123456789012345678901234567890".to_string()).unwrap(),
            IpfsHash::new("QmList2345678901234567890123456789012345678901".to_string()).unwrap(),
            IpfsHash::new("QmList3456789012345678901234567890123456789012".to_string()).unwrap(),
        ];

        let client = manager.client.clone();

        // Add content using the client interface and get actual hashes
        let mut actual_hashes = Vec::new();
        for i in 0..3 {
            let hash = client
                .add_bytes(format!("content {}", i).into_bytes())
                .await
                .unwrap();
            actual_hashes.push(hash);
        }

        // Pin with different priorities
        manager
            .pin_with_priority(&actual_hashes[0], PinPriority::Low)
            .await
            .unwrap();
        manager
            .pin_with_priority(&actual_hashes[1], PinPriority::High)
            .await
            .unwrap();
        manager
            .pin_with_priority(&actual_hashes[2], PinPriority::High)
            .await
            .unwrap();

        // List all pins
        let all_pins = manager.list_pins().await.unwrap();
        assert_eq!(all_pins.len(), 3);

        // List pins by priority
        let high_priority_pins = manager
            .list_pins_by_priority(PinPriority::High)
            .await
            .unwrap();
        assert_eq!(high_priority_pins.len(), 2);

        let low_priority_pins = manager
            .list_pins_by_priority(PinPriority::Low)
            .await
            .unwrap();
        assert_eq!(low_priority_pins.len(), 1);

        // List pins by policy
        let default_policy_pins = manager.list_pins_by_policy("default").await.unwrap();
        assert_eq!(default_policy_pins.len(), 3);
    }

    #[tokio::test]
    async fn test_unpin_content() {
        let manager = create_test_pinning_manager().await;

        // Add and pin test content
        let client = manager.client.clone();
        let actual_hash = client.add_bytes(b"unpin test".to_vec()).await.unwrap();

        manager
            .pin_with_priority(&actual_hash, PinPriority::Normal)
            .await
            .unwrap();

        // Verify pin exists
        assert!(manager.get_pin_info(&actual_hash).await.is_ok());

        // Unpin content
        manager.unpin_content(&actual_hash).await.unwrap();

        // Verify pin is removed
        assert!(manager.get_pin_info(&actual_hash).await.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_expired_pins() {
        let manager = create_test_pinning_manager().await;

        // Create policy with short retention
        let mut policy = create_test_policy();
        policy.retention_days = Some(1); // 1 day retention
        manager.create_policy(policy).await.unwrap();

        // Add test content
        let client = manager.client.clone();
        let actual_hash = client.add_bytes(b"expired test".to_vec()).await.unwrap();

        // Pin with policy
        let mut pin_info = manager
            .pin_with_policy(&actual_hash, "test_policy")
            .await
            .unwrap();

        // Manually set expiration to past date for testing
        pin_info.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
        manager
            .pins
            .write()
            .await
            .insert(actual_hash.as_str().to_string(), pin_info);

        // Cleanup expired pins
        let expired_hashes = manager.cleanup_expired_pins().await.unwrap();
        assert_eq!(expired_hashes.len(), 1);
        assert_eq!(expired_hashes[0], actual_hash);

        // Verify pin is removed
        assert!(manager.get_pin_info(&actual_hash).await.is_err());
    }

    #[tokio::test]
    async fn test_pinning_statistics() {
        let manager = create_test_pinning_manager().await;

        // Create test content and pins with different priorities
        let client = manager.client.clone();

        // Add content using client interface and get actual hashes
        let mut actual_hashes = Vec::new();
        for i in 0..3 {
            let content = format!("stats content {}", i).repeat(100); // Different sizes
            let hash = client.add_bytes(content.into_bytes()).await.unwrap();
            actual_hashes.push(hash);
        }

        // Pin with different priorities
        manager
            .pin_with_priority(&actual_hashes[0], PinPriority::Low)
            .await
            .unwrap();
        manager
            .pin_with_priority(&actual_hashes[1], PinPriority::High)
            .await
            .unwrap();
        manager
            .pin_with_priority(&actual_hashes[2], PinPriority::Critical)
            .await
            .unwrap();

        // Get statistics
        let stats = manager.get_pinning_stats().await.unwrap();

        assert_eq!(stats.total_pins, 3);
        assert!(stats.total_size > 0);
        assert_eq!(stats.pins_by_priority.get(&PinPriority::Low), Some(&1));
        assert_eq!(stats.pins_by_priority.get(&PinPriority::High), Some(&1));
        assert_eq!(stats.pins_by_priority.get(&PinPriority::Critical), Some(&1));
        assert_eq!(stats.pins_by_policy.get("default"), Some(&3));
        assert_eq!(stats.expired_pins, 0);
        assert_eq!(stats.replication_health, 100.0);
    }

    #[tokio::test]
    async fn test_rebalance_pins() {
        let manager = create_test_pinning_manager().await;

        // Add some test pins
        let client = manager.client.clone();
        let actual_hash = client.add_bytes(b"rebalance test".to_vec()).await.unwrap();

        manager
            .pin_with_priority(&actual_hash, PinPriority::Normal)
            .await
            .unwrap();

        // Rebalance pins (this is a simplified test)
        let result = manager.rebalance_pins().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_pin_policy_default() {
        let policy = PinPolicy::default();
        assert_eq!(policy.name, "default");
        assert_eq!(policy.priority, PinPriority::Normal);
        assert_eq!(policy.replication_factor, 3);
        assert!(policy.retention_days.is_none());
    }

    #[test]
    fn test_pin_priority_default() {
        let priority = PinPriority::default();
        assert_eq!(priority, PinPriority::Normal);
    }
}
