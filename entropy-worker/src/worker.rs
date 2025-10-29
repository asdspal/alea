use anyhow::Result;
use entropy_types::{CommitmentPayload, StartCommitmentMsg, NodeId, RevealMsg, RevealPayload};
use secp256k1::{SecretKey, PublicKey};
use std::net::TcpStream;
use log::{info, debug};

use crate::crypto::{generate_secret, compute_commitment, generate_keypair, create_commitment_payload};

/// Worker node state and configuration
pub struct Worker {
    /// Unique identifier for this worker node
    pub node_id: NodeId,
    
    /// Secret key for cryptographic operations
    secret_key: SecretKey,
    
    /// Public key for verification
    public_key: PublicKey,
    
    /// Current round ID the worker is participating in
    current_round_id: Option<u64>,
    
    /// The secret generated for the current round (kept in memory for reveal phase)
    current_secret: Option<[u8; 32]>,
    
    /// The commitment for the current round
    current_commitment: Option<[u8; 32]>,
    
    /// Connection to the aggregator
    aggregator_connection: Option<TcpStream>,
}

impl Worker {
    /// Create a new worker instance with generated keypair
    pub fn new(node_id: NodeId) -> Result<Self> {
        let (secret_key, public_key) = generate_keypair()?;
        
        Ok(Worker {
            node_id,
            secret_key,
            public_key,
            current_round_id: None,
            current_secret: None,
            current_commitment: None,
            aggregator_connection: None,
        })
    }
    
    /// Handle the start commitment message from the aggregator
    pub fn handle_start_commitment(&mut self, msg: &StartCommitmentMsg) -> Result<CommitmentPayload> {
        info!("Worker {} received start commitment for round {}", self.node_id, msg.round_id);
        
        // Check if this worker is part of the committee for this round
        if !msg.committee.contains(&self.node_id) {
            return Err(anyhow::Error::msg(format!(
                "Worker {} is not part of the committee for round {}", 
                self.node_id, 
                msg.round_id
            )));
        }
        
        // Generate a new secret for this round
        let secret = generate_secret()?;
        debug!("Generated secret for round {}: {}", msg.round_id, hex::encode(&secret));
        
        // Compute commitment from the secret
        let commitment = compute_commitment(&secret);
        debug!("Computed commitment for round {}: {}", msg.round_id, hex::encode(&commitment));
        
        // Create the commitment payload
        let payload = create_commitment_payload(msg.round_id, &secret, &self.secret_key)?;
        
        // Store state for later use (reveal phase)
        self.current_round_id = Some(msg.round_id);
        self.current_secret = Some(secret);
        self.current_commitment = Some(commitment);
        
        info!("Successfully created commitment payload for round {}", msg.round_id);
        Ok(payload)
    }
    
    /// Get the current secret (for reveal phase)
    pub fn get_current_secret(&self) -> Option<[u8; 32]> {
        self.current_secret
    }
    
    /// Get the current round ID
    pub fn get_current_round_id(&self) -> Option<u64> {
        self.current_round_id
    }
    
    /// Check if the worker is participating in a round
    pub fn is_participating(&self) -> bool {
        self.current_round_id.is_some()
    }
    
    /// Reset the worker's state for a new round
    pub fn reset_state(&mut self) {
        self.current_round_id = None;
        self.current_secret = None;
        self.current_commitment = None;
    }
    
    /// Get the worker's public key
    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
    }
    
    /// Get the worker's node ID
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }
    
    /// Create a reveal message for the current round
    pub fn create_reveal_message(&self) -> Result<RevealMsg> {
        if let (Some(round_id), Some(secret)) = (self.current_round_id, self.current_secret) {
            Ok(RevealMsg {
                round_id,
                payload: RevealPayload {
                    round_id,
                    secret,
                },
                node_id: self.node_id.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
        } else {
            Err(anyhow::Error::msg("Worker is not participating in a round"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entropy_types::StartCommitmentMsg;

    #[test]
    fn test_worker_creation() {
        let worker = Worker::new("test-node-1".to_string()).unwrap();
        
        assert_eq!(worker.get_node_id(), "test-node-1");
        assert!(!worker.is_participating());
        assert!(worker.get_current_secret().is_none());
        assert!(worker.get_current_round_id().is_none());
    }

    #[test]
    fn test_handle_start_commitment() {
        let mut worker = Worker::new("test-node-2".to_string()).unwrap();
        
        let start_msg = StartCommitmentMsg {
            round_id: 1,
            committee: vec!["test-node-2".to_string(), "test-node-3".to_string()],
        };
        
        let payload = worker.handle_start_commitment(&start_msg).unwrap();
        
        assert_eq!(payload.round_id, 1);
        assert_eq!(worker.get_current_round_id(), Some(1));
        assert!(worker.get_current_secret().is_some());
        assert!(worker.is_participating());
    }

    #[test]
    fn test_worker_not_in_committee() {
        let mut worker = Worker::new("test-node-4".to_string()).unwrap();
        
        let start_msg = StartCommitmentMsg {
            round_id: 1,
            committee: vec!["test-node-2".to_string(), "test-node-3".to_string()],
        };
        
        let result = worker.handle_start_commitment(&start_msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_worker_state_reset() {
        let mut worker = Worker::new("test-node-5".to_string()).unwrap();
        
        let start_msg = StartCommitmentMsg {
            round_id: 1,
            committee: vec!["test-node-5".to_string()],
        };
        
        worker.handle_start_commitment(&start_msg).unwrap();
        assert!(worker.is_participating());
        
        worker.reset_state();
        assert!(!worker.is_participating());
        assert!(worker.get_current_secret().is_none());
        assert!(worker.get_current_round_id().is_none());
    }
}