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
}