use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use entropy_types::{CommitmentPayload, NodeId};

/// Aggregator state enum representing different phases of the protocol
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AggregatorState {
    /// Initial state - waiting to start a new round
    Idle,
    /// Collecting commitments from worker nodes
    CollectingCommitments {
        round_id: u64,
        commitments: HashMap<NodeId, (CommitmentPayload, Vec<u8>)>, // (payload, public_key)
        threshold: usize,
    },
    /// Collecting reveals from worker nodes
    CollectingReveals {
        round_id: u64,
        reveals: HashMap<NodeId, Vec<u8>>, // (node_id, reveal_data)
        threshold: usize,
    },
    /// Aggregating the final entropy value in TEE
    Aggregating {
        round_id: u64,
    },
    /// Publishing the final result to the beacon chain
    Publishing {
        round_id: u64,
    },
}

impl AggregatorState {
    /// Check if the current state is Idle
    pub fn is_idle(&self) -> bool {
        matches!(self, AggregatorState::Idle)
    }

    /// Check if the current state is CollectingCommitments
    pub fn is_collecting_commitments(&self) -> bool {
        matches!(self, AggregatorState::CollectingCommitments { .. })
    }

    /// Check if the current state is CollectingReveals
    pub fn is_collecting_reveals(&self) -> bool {
        matches!(self, AggregatorState::CollectingReveals { .. })
    }

    /// Check if the current state is Publishing
    pub fn is_publishing(&self) -> bool {
        matches!(self, AggregatorState::Publishing { .. })
    }

    /// Get the round ID if the state has one
    pub fn get_round_id(&self) -> Option<u64> {
        match self {
            AggregatorState::Idle => None,
            AggregatorState::CollectingCommitments { round_id, .. } => Some(*round_id),
            AggregatorState::CollectingReveals { round_id, .. } => Some(*round_id),
            AggregatorState::Aggregating { round_id } => Some(*round_id),
            AggregatorState::Publishing { round_id } => Some(*round_id),
        }
    }

    /// Check if we have enough commitments to transition to reveal phase
    pub fn has_enough_commitments(&self, threshold: usize) -> bool {
        match self {
            AggregatorState::CollectingCommitments { commitments, .. } => {
                commitments.len() >= threshold
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_enum() {
        let idle_state = AggregatorState::Idle;
        assert!(idle_state.is_idle());

        let collecting_state = AggregatorState::CollectingCommitments {
            round_id: 1,
            commitments: HashMap::new(),
            threshold: 3,
        };
        assert!(collecting_state.is_collecting_commitments());
        assert_eq!(collecting_state.get_round_id(), Some(1));
    }

    #[test]
    fn test_enough_commitments() {
        let mut commitments = HashMap::new();
        commitments.insert("node1".to_string(), (CommitmentPayload {
            round_id: 1,
            commitment: [0u8; 32],
            signature: vec![],
        }, vec![]));
        commitments.insert("node2".to_string(), (CommitmentPayload {
            round_id: 1,
            commitment: [0u8; 32],
            signature: vec![],
        }, vec![]));

        let state = AggregatorState::CollectingCommitments {
            round_id: 1,
            commitments,
            threshold: 3,
        };

        assert!(!state.has_enough_commitments(3));
        assert!(state.has_enough_commitments(2));
    }
}