use beacon_microchain::RandomnessEvent;
use linera_sdk::views::{linera_views, MapView, RegisterView, RootView, ViewStorageContext};

#[derive(RootView)]
#[view(context = ViewStorageContext)]
pub struct BeaconState {
    /// Latest round id recorded
    pub current_round_id: RegisterView<u64>,
    /// Stored random_number keyed by round id
    #[view(map)]
    pub event_random: MapView<u64, String>,
    /// Stored nonce keyed by round id
    #[view(map)]
    pub event_nonce: MapView<u64, String>,
    /// Stored attestation keyed by round id
    #[view(map)]
    pub event_attestation: MapView<u64, String>,
    /// Authorized aggregator/admin key
    pub admin_public_key: RegisterView<Option<String>>,
}

impl BeaconState {
    /// Get randomness by round ID
    pub async fn get_randomness(&self, round_id: u64) -> Option<RandomnessEvent> {
        let random = self.event_random.get(&round_id).await.ok().flatten()?;
        let nonce = self.event_nonce.get(&round_id).await.ok().flatten()?;
        let att = self.event_attestation.get(&round_id).await.ok().flatten()?;
        Some(RandomnessEvent {
            round_id,
            random_number: random,
            nonce,
            attestation: att,
        })
    }
}
