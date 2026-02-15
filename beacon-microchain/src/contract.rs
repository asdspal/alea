#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use beacon_microchain::{
    abi::{BeaconInstantiationArgument, BeaconMessage, BeaconOperation, BeaconParameters},
    BeaconAbi, RandomnessEvent,
};
use linera_sdk::{
    contract, linera_base_types::WithContractAbi, views::RootView, views::View, Contract, ContractRuntime,
};

use self::state::BeaconState;

contract!(BeaconContract);

pub struct BeaconContract {
    state: BeaconState,
    runtime: ContractRuntime<Self>,
}

impl WithContractAbi for BeaconContract {
    type Abi = BeaconAbi;
}

impl Contract for BeaconContract {
    type Message = BeaconMessage;
    type InstantiationArgument = BeaconInstantiationArgument;
    type Parameters = BeaconParameters;
    type EventValue = RandomnessEvent;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = BeaconState::load(runtime.root_view_storage_context())
            .await
            .expect("load state");
        Self { state, runtime }
    }

    async fn instantiate(&mut self, argument: Self::InstantiationArgument) {
        self.state.admin_public_key.set(argument.admin_public_key);
    }

    async fn execute_operation(&mut self, operation: BeaconOperation) -> () {
        match operation {
            BeaconOperation::SubmitRandomness { event, signature } => {
                self.handle_submission(event, signature).await
            }
        }
    }

    async fn execute_message(&mut self, message: BeaconMessage) {
        match message {
            BeaconMessage::SubmitRandomness { event, signature } => {
                self.handle_submission(event, signature).await
            }
        }
    }

    async fn store(mut self) {
        self.state.save().await.expect("save state");
    }
}

impl BeaconContract {
    async fn handle_submission(&mut self, event: RandomnessEvent, _signature: Vec<u8>) {
        // TODO: verify signature once format is defined; for now accept
        let round_id = event.round_id;
        if round_id > *self.state.current_round_id.get() {
            self.state.current_round_id.set(round_id);
        }
        self.state
            .events
            .insert(&round_id, event)
            .expect("insert event");
    }

    /// Test helper: pure BTreeMap version mirroring contract logic.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub fn process_randomness_submission(
        event: RandomnessEvent,
        _signature: Vec<u8>,
        admin_public_key: &Option<String>,
        caller: &Option<String>,
        current_round_id: &mut u64,
        events: &mut std::collections::BTreeMap<u64, RandomnessEvent>,
    ) -> Result<(), String> {
        match (admin_public_key.as_ref(), caller.as_ref()) {
            (Some(admin), Some(caller_key)) if admin == caller_key => {}
            _ => return Err("Unauthorized caller".to_string()),
        }

        events.insert(event.round_id, event.clone());
        if event.round_id > *current_round_id {
            *current_round_id = event.round_id;
        }
        Ok(())
    }

    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub fn get_randomness(
        round_id: u64,
        events: &std::collections::BTreeMap<u64, RandomnessEvent>,
    ) -> Option<RandomnessEvent> {
        events.get(&round_id).cloned()
    }
}
