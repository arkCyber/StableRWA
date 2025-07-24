// =====================================================================================
// File: core-layer2/src/state_sync.rs
// Description: State synchronization between L1 and L2 (placeholder)
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// State sync service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSyncService {
    pub enabled: bool,
    pub sync_interval_seconds: u64,
}

/// State root
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRoot {
    pub root_hash: String,
    pub block_number: u64,
    pub timestamp: DateTime<Utc>,
}

/// State merkle tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMerkleTree {
    pub root: String,
    pub depth: u32,
    pub leaf_count: u64,
}

/// Checkpoint manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointManager {
    pub contract_address: String,
    pub checkpoint_interval: u64,
    pub last_checkpoint: u64,
}

/// State proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProof {
    pub id: Uuid,
    pub merkle_proof: Vec<String>,
    pub leaf_index: u64,
    pub root_hash: String,
}

impl Default for StateSyncService {
    fn default() -> Self {
        Self {
            enabled: true,
            sync_interval_seconds: 300, // 5 minutes
        }
    }
}
