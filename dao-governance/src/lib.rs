// =====================================================================================
// File: dao-governance/src/lib.rs
// Description: Core DAO governance logic for enterprise-grade DAO microservice.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use log::{info, error};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// DAO member structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Member {
    pub address: String,
    pub joined_at: String,
}

/// DAO proposal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: String,
    pub votes_for: u32,
    pub votes_against: u32,
    pub executed: bool,
}

/// DAO vote structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: String,
    pub support: bool,
    pub voted_at: String,
}

/// DAO error type
#[derive(Debug, Error)]
pub enum DaoError {
    #[error("Member not found")]
    MemberNotFound,
    #[error("Proposal not found")]
    ProposalNotFound,
    #[error("Already voted")]
    AlreadyVoted,
    #[error("Proposal already executed")]
    AlreadyExecuted,
    #[error("Not authorized")]
    NotAuthorized,
    #[error("Internal error: {0}")]
    Internal(String),
}

/// DAO governance store (in-memory, for demo; replace with DB in production)
#[derive(Default, Clone)]
pub struct DaoStore {
    pub members: Arc<Mutex<HashSet<Member>>>,
    pub proposals: Arc<Mutex<HashMap<u64, Proposal>>>,
    pub votes: Arc<Mutex<HashMap<(u64, String), Vote>>>,
    pub next_proposal_id: Arc<Mutex<u64>>,
}

impl DaoStore {
    /// Creates a new DAO store
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new member
    pub fn add_member(&self, address: String) -> Result<Member, DaoError> {
        let now = Utc::now();
        let member = Member { address: address.clone(), joined_at: now.to_rfc3339() };
        self.members.lock().unwrap().insert(member.clone());
        info!("{} - [DaoStore] Member added: {}", now, address);
        Ok(member)
    }

    /// Creates a new proposal
    pub fn create_proposal(&self, proposer: String, title: String, description: String) -> Result<Proposal, DaoError> {
        let now = Utc::now();
        if !self.members.lock().unwrap().iter().any(|m| m.address == proposer) {
            error!("{} - [DaoStore] Proposer not a member: {}", now, proposer);
            return Err(DaoError::NotAuthorized);
        }
        let mut id_lock = self.next_proposal_id.lock().unwrap();
        let id = *id_lock;
        *id_lock += 1;
        let proposal = Proposal {
            id,
            title,
            description,
            proposer: proposer.clone(),
            created_at: now.to_rfc3339(),
            votes_for: 0,
            votes_against: 0,
            executed: false,
        };
        self.proposals.lock().unwrap().insert(id, proposal.clone());
        info!("{} - [DaoStore] Proposal created: {} by {}", now, id, proposer);
        Ok(proposal)
    }

    /// Casts a vote on a proposal
    pub fn vote(&self, proposal_id: u64, voter: String, support: bool) -> Result<Vote, DaoError> {
        let now = Utc::now();
        if !self.members.lock().unwrap().iter().any(|m| m.address == voter) {
            error!("{} - [DaoStore] Voter not a member: {}", now, voter);
            return Err(DaoError::NotAuthorized);
        }
        let mut proposals = self.proposals.lock().unwrap();
        let proposal = proposals.get_mut(&proposal_id).ok_or(DaoError::ProposalNotFound)?;
        if proposal.executed {
            error!("{} - [DaoStore] Proposal already executed: {}", now, proposal_id);
            return Err(DaoError::AlreadyExecuted);
        }
        let mut votes = self.votes.lock().unwrap();
        if votes.contains_key(&(proposal_id, voter.clone())) {
            error!("{} - [DaoStore] Already voted: {} on {}", now, voter, proposal_id);
            return Err(DaoError::AlreadyVoted);
        }
        let vote = Vote {
            proposal_id,
            voter: voter.clone(),
            support,
            voted_at: now.to_rfc3339(),
        };
        if support {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }
        votes.insert((proposal_id, voter.clone()), vote.clone());
        info!("{} - [DaoStore] Vote cast: {} on {} support={}", now, voter, proposal_id, support);
        Ok(vote)
    }

    /// Executes a proposal if it has more for than against votes
    pub fn execute_proposal(&self, proposal_id: u64) -> Result<Proposal, DaoError> {
        let now = Utc::now();
        let mut proposals = self.proposals.lock().unwrap();
        let proposal = proposals.get_mut(&proposal_id).ok_or(DaoError::ProposalNotFound)?;
        if proposal.executed {
            error!("{} - [DaoStore] Proposal already executed: {}", now, proposal_id);
            return Err(DaoError::AlreadyExecuted);
        }
        if proposal.votes_for <= proposal.votes_against {
            error!("{} - [DaoStore] Proposal not passed: {}", now, proposal_id);
            return Err(DaoError::Internal("Proposal did not pass".to_string()));
        }
        proposal.executed = true;
        info!("{} - [DaoStore] Proposal executed: {}", now, proposal_id);
        Ok(proposal.clone())
    }

    /// Gets all proposals
    pub fn get_proposals(&self) -> Vec<Proposal> {
        self.proposals.lock().unwrap().values().cloned().collect()
    }

    /// Gets all members
    pub fn get_members(&self) -> Vec<Member> {
        self.members.lock().unwrap().iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).filter_level(LevelFilter::Info).try_init();
    }

    #[test]
    fn test_member_and_proposal_flow() {
        init_logger();
        let store = DaoStore::new();
        let m = store.add_member("0xabc".to_string()).unwrap();
        let p = store.create_proposal(m.address.clone(), "Test".to_string(), "Desc".to_string()).unwrap();
        let v = store.vote(p.id, m.address.clone(), true).unwrap();
        let exec = store.execute_proposal(p.id);
        assert!(exec.is_ok());
    }

    #[test]
    fn test_vote_and_execute_failures() {
        init_logger();
        let store = DaoStore::new();
        let m = store.add_member("0xabc".to_string()).unwrap();
        let p = store.create_proposal(m.address.clone(), "Test".to_string(), "Desc".to_string()).unwrap();
        // Not a member
        assert!(store.vote(p.id, "0xdef".to_string(), true).is_err());
        // Double vote
        store.vote(p.id, m.address.clone(), true).unwrap();
        assert!(store.vote(p.id, m.address.clone(), false).is_err());
        // Execute without enough for votes
        let p2 = store.create_proposal(m.address.clone(), "Test2".to_string(), "Desc2".to_string()).unwrap();
        store.vote(p2.id, m.address.clone(), false).unwrap();
        assert!(store.execute_proposal(p2.id).is_err());
    }
} 