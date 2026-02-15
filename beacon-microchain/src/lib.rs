use linera_sdk::linera_base_types::{ContractAbi, ServiceAbi};
use serde::{Deserialize, Serialize};

/// ABI for Beacon Microchain Application
pub struct BeaconAbi;

impl ContractAbi for BeaconAbi {
    type Operation = crate::abi::BeaconOperation;
    type Response = ();
}

impl ServiceAbi for BeaconAbi {
    type Query = crate::abi::BeaconQuery;
    type QueryResponse = crate::abi::BeaconQueryResponse;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RandomnessEvent {
    pub round_id: u64,
    pub random_number: [u8; 32],
    pub nonce: [u8; 16],
    pub attestation: Vec<u8>,
}

pub mod abi {
    use serde::{Deserialize, Serialize};

    use super::RandomnessEvent;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum BeaconOperation {
        SubmitRandomness {
            event: RandomnessEvent,
            signature: Vec<u8>,
        },
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum BeaconMessage {
        SubmitRandomness {
            event: RandomnessEvent,
            signature: Vec<u8>,
        },
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum BeaconQuery {
        GetRandomness { round_id: u64 },
    }

    pub type BeaconQueryResponse = Option<RandomnessEvent>;

    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct BeaconParameters;

    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct BeaconInstantiationArgument {
        pub admin_public_key: Option<String>,
    }
}
