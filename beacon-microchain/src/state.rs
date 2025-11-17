use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RandomnessEvent {
    pub round_id: u64,
    pub random_number: [u8; 32],
    pub nonce: [u8; 16],
    pub attestation: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BeaconState {
    pub current_round_id: u64,
    pub events: BTreeMap<u64, RandomnessEvent>,
    pub admin_public_key: Option<String>, // Using String as placeholder until we determine the correct type
}

impl BeaconState {
    /// Check if the caller is authorized (only registered Aggregator can submit)
    pub fn is_authorized_caller(&self, caller: &Option<String>) -> bool {
        match (&self.admin_public_key, caller) {
            (Some(admin_key), Some(caller_key)) => admin_key == caller_key,
            _ => false,
        }
    }

    /// Get randomness by round ID
    pub fn get_randomness(&self, round_id: u64) -> Option<RandomnessEvent> {
        self.events.get(&round_id).cloned()
    }
}