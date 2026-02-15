use beacon_microchain::RandomnessEvent;
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};

#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct BeaconState {
    /// Latest round id recorded
    pub current_round_id: RegisterView<u64>,
    /// Stored events keyed by round id
    #[view(map)]
    pub events: MapView<u64, RandomnessEvent>,
    /// Authorized aggregator/admin key
    pub admin_public_key: RegisterView<Option<String>>,
}

impl BeaconState {
    /// Check if the caller is authorized (only registered Aggregator can submit)
    pub fn is_authorized_caller(&self, caller: &Option<String>) -> bool {
        match (self.admin_public_key.get().as_ref(), caller.as_ref()) {
            (Some(admin_key), Some(caller_key)) => admin_key == caller_key,
            _ => false,
        }
    }

    /// Get randomness by round ID
    pub async fn get_randomness(&self, round_id: u64) -> Option<RandomnessEvent> {
        self.events.get(&round_id).await.expect("read view")
    }
}
