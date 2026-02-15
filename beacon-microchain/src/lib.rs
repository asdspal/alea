use linera_sdk::linera_base_types::{ContractAbi, ServiceAbi};
use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

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

#[derive(Clone, Debug, PartialEq)]
pub struct RandomnessEvent {
    pub round_id: u64,
    pub random_number: String,  // Changed from [u8; 32] to String for WASM compatibility
    pub nonce: String,          // Changed from [u8; 16] to String for WASM compatibility
    pub attestation: String,    // Changed from Vec<u8> to String for WASM compatibility
}

impl Serialize for RandomnessEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(4)?;
        tup.serialize_element(&self.round_id)?;
        tup.serialize_element(&self.random_number)?;
        tup.serialize_element(&self.nonce)?;
        tup.serialize_element(&self.attestation)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for RandomnessEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventVisitor;

        impl<'de> Visitor<'de> for EventVisitor {
            type Value = RandomnessEvent;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("tuple (round_id, random_number, nonce, attestation)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let round_id: u64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let random_number: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let nonce: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let attestation: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                Ok(RandomnessEvent {
                    round_id,
                    random_number,
                    nonce,
                    attestation,
                })
            }
        }

        deserializer.deserialize_tuple(4, EventVisitor)
    }
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
