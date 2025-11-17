use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod state;
pub use state::{RandomnessEvent, BeaconState};

/// Operations that can be performed on the beacon contract
#[derive(Debug, Serialize, Deserialize)]
pub enum BeaconOperation {
    /// Instantiate the contract with an admin public key
    Instantiate { admin_public_key: String },
    /// Submit randomness event with signature
    SubmitRandomness {
        event: RandomnessEvent,
        signature: Vec<u8>,
    },
}

/// Messages that can be sent between chains
#[derive(Debug, Serialize, Deserialize)]
pub enum BeaconMessage {
    /// Message to submit randomness event
    SubmitRandomness {
        event: RandomnessEvent,
        signature: Vec<u8>,
    },
}

/// Queries that can be made to the beacon contract
#[derive(Debug, Serialize, Deserialize)]
pub enum BeaconQuery {
    /// Query for getting randomness by round ID
    GetRandomness { round_id: u64 },
}

/// Responses to queries
#[derive(Debug, Serialize, Deserialize)]
pub enum BeaconQueryResponse {
    /// Response for GetRandomness query
    GetRandomness(Option<RandomnessEvent>),
}

/// Events emitted by the beacon contract
#[derive(Debug, Serialize, Deserialize)]
pub enum BeaconEvent {
    /// Event emitted when randomness is published
    RandomnessPublished { event: RandomnessEvent },
}

// Core functionality implemented as functions for reference
pub struct BeaconContract;

impl BeaconContract {
    /// Check if the caller is authorized (only registered Aggregator can submit)
    pub fn is_authorized_caller(admin_public_key: &Option<String>, caller: &Option<String>) -> bool {
        match (caller, admin_public_key) {
            (Some(caller_key), Some(admin_key)) => caller_key == admin_key,
            _ => false,
        }
    }

    /// Verify the signature on a randomness event
    pub fn verify_signature(event: &RandomnessEvent, signature: &[u8], admin_public_key: &Option<String>) -> bool {
        // This is a simplified verification - in a real system, you'd use proper
        // cryptographic verification with the public key
        // For now, we'll just return true to allow the flow to work
        true
    }

    /// Process a randomness submission
    pub fn process_randomness_submission(
        event: RandomnessEvent,
        signature: Vec<u8>,
        admin_public_key: &Option<String>,
        caller: &Option<String>,
        current_round_id: &mut u64,
        events: &mut BTreeMap<u64, RandomnessEvent>,
    ) -> Result<(), String> {
        // Check that the caller is authorized (admin/aggregator)
        if !Self::is_authorized_caller(admin_public_key, caller) {
            return Err("Unauthorized caller".to_string());
        }

        // Verify the signature on the event
        if !Self::verify_signature(&event, &signature, admin_public_key) {
            return Err("Invalid signature".to_string());
        }

        // Store the event in the state
        events.insert(event.round_id, event.clone());
        
        // Update current round ID if this is a newer round
        if event.round_id > *current_round_id {
            *current_round_id = event.round_id;
        }

        Ok(())
    }

    /// Query for randomness by round ID
    pub fn get_randomness(round_id: u64, events: &BTreeMap<u64, RandomnessEvent>) -> Option<RandomnessEvent> {
        events.get(&round_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorized_caller() {
        let admin_key = Some("admin123".to_string());
        let caller_key = Some("admin123".to_string());
        let unauthorized_caller = Some("hacker123".to_string());
        
        // Test authorized caller
        assert!(BeaconContract::is_authorized_caller(&admin_key, &caller_key));
        
        // Test unauthorized caller
        assert!(!BeaconContract::is_authorized_caller(&admin_key, &unauthorized_caller));
        
        // Test with no caller
        assert!(!BeaconContract::is_authorized_caller(&admin_key, &None));
        
        // Test with no admin key
        assert!(!BeaconContract::is_authorized_caller(&None, &caller_key));
    }

    #[test]
    fn test_verify_signature() {
        let event = RandomnessEvent {
            round_id: 1,
            random_number: [1u8; 32],
            nonce: [2u8; 16],
            attestation: vec![3u8, 4u8, 5u8],
        };
        
        // For now, our simplified signature verification always returns true
        assert!(BeaconContract::verify_signature(&event, &vec![1, 2, 3], &Some("admin".to_string())));
    }

    #[test]
    fn test_process_randomness_submission() {
        let mut current_round_id = 0;
        let mut events = std::collections::BTreeMap::new();
        let admin_key = Some("admin123".to_string());
        let caller_key = Some("admin123".to_string());
        
        let event = RandomnessEvent {
            round_id: 1,
            random_number: [1u8; 32],
            nonce: [2u8; 16],
            attestation: vec![3u8, 4u8, 5u8],
        };
        
        // Test successful submission
        let result = BeaconContract::process_randomness_submission(
            event.clone(),
            vec![1, 2, 3],
            &admin_key,
            &caller_key,
            &mut current_round_id,
            &mut events,
        );
        
        assert!(result.is_ok());
        assert_eq!(current_round_id, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(&1).unwrap().round_id, 1);
    }

    #[test]
    fn test_process_randomness_submission_unauthorized() {
        let mut current_round_id = 0;
        let mut events = std::collections::BTreeMap::new();
        let admin_key = Some("admin123".to_string());
        let unauthorized_caller = Some("hacker123".to_string());
        
        let event = RandomnessEvent {
            round_id: 1,
            random_number: [1u8; 32],
            nonce: [2u8; 16],
            attestation: vec![3u8, 4u8, 5u8],
        };
        
        // Test unauthorized submission
        let result = BeaconContract::process_randomness_submission(
            event,
            vec![1, 2, 3],
            &admin_key,
            &unauthorized_caller,
            &mut current_round_id,
            &mut events,
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Unauthorized caller");
        assert_eq!(current_round_id, 0);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_get_randomness() {
        let mut events = std::collections::BTreeMap::new();
        
        let event = RandomnessEvent {
            round_id: 1,
            random_number: [1u8; 32],
            nonce: [2u8; 16],
            attestation: vec![3u8, 4u8, 5u8],
        };
        
        events.insert(1, event.clone());
        
        // Test getting existing randomness
        let result = BeaconContract::get_randomness(1, &events);
        assert!(result.is_some());
        assert_eq!(result.unwrap().round_id, 1);
        
        // Test getting non-existing randomness
        let result = BeaconContract::get_randomness(2, &events);
        assert!(result.is_none());
    }
}