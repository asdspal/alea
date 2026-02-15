#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use beacon_microchain::{
    abi::{BeaconParameters, BeaconQuery, BeaconQueryResponse},
    BeaconAbi,
};
use linera_sdk::{linera_base_types::WithServiceAbi, service, views::View, Service, ServiceRuntime};

use self::state::BeaconState;

service!(BeaconService);

pub struct BeaconService {
    state: BeaconState,
}

impl WithServiceAbi for BeaconService {
    type Abi = BeaconAbi;
}

impl Service for BeaconService {
    type Parameters = BeaconParameters;

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = BeaconState::load(runtime.root_view_storage_context())
            .await
            .expect("load state");
        Self { state }
    }

    async fn handle_query(&self, query: BeaconQuery) -> BeaconQueryResponse {
        match query {
            BeaconQuery::GetRandomness { round_id } => self.state.get_randomness(round_id).await,
        }
    }
}
