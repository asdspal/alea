use anyhow::Result;
use async_trait::async_trait;
use linera_sdk::linera_base_types::{Amount, ChainId, AccountOwner};
use serde::{Deserialize, Serialize};

/// Trait that abstracts Linera operations for the entropy system
#[async_trait]
pub trait LineraProvider: Send + Sync {
    /// Query the current state of the beacon microchain
    async fn query_beacon_state(&self) -> Result<BeaconStateQueryResult>;

    /// Submit a transaction to update the beacon state
    async fn submit_beacon_transaction(&self, transaction: BeaconTransaction) -> Result<TransactionId>;

    /// Get the current chain ID
    async fn get_chain_id(&self) -> Result<ChainId>;

    /// Get the current account balance
    async fn get_balance(&self, owner: AccountOwner) -> Result<Amount>;
}

/// Result of querying beacon state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconStateQueryResult {
    pub entropy_shares: Vec<EntropyShare>,
    pub latest_entropy: Option<[u8; 32]>,
    pub round_id: u64,
}

/// A transaction to be submitted to the beacon microchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconTransaction {
    pub action: BeaconAction,
    pub nonce: u64,
}

/// Actions that can be performed on the beacon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeaconAction {
    SubmitEntropyShare {
        share: [u8; 32],
        worker_id: String,
        signature: Vec<u8>,
    },
    AggregateEntropy {
        aggregated_entropy: [u8; 32],
        attestation: Vec<u8>,
    },
}

/// Mock implementation of LineraProvider for testing
pub struct MockLineraProvider {
    state: BeaconStateQueryResult,
    chain_id: ChainId,
}

impl MockLineraProvider {
    pub fn new(chain_id: ChainId) -> Self {
        Self {
            state: BeaconStateQueryResult {
                entropy_shares: Vec::new(),
                latest_entropy: None,
                round_id: 0,
            },
            chain_id,
        }
    }
}

#[async_trait]
impl LineraProvider for MockLineraProvider {
    async fn query_beacon_state(&self) -> Result<BeaconStateQueryResult> {
        Ok(self.state.clone())
    }

    async fn submit_beacon_transaction(&self, _transaction: BeaconTransaction) -> Result<TransactionId> {
        // In a mock, we just return a dummy transaction ID
        Ok(TransactionId::new([0u8; 32]))
    }

    async fn get_chain_id(&self) -> Result<ChainId> {
        Ok(self.chain_id)
    }

    async fn get_balance(&self, _owner: AccountOwner) -> Result<Amount> {
        Ok(Amount::from_tokens(100))
    }
}

/// Concrete implementation using the actual Linera SDK
pub struct LineraSdkProvider;

#[async_trait]
impl LineraProvider for LineraSdkProvider {
    async fn query_beacon_state(&self) -> Result<BeaconStateQueryResult> {
        // This would interact with the actual Linera chain
        // For now, returning a default state
        Ok(BeaconStateQueryResult {
            entropy_shares: Vec::new(),
            latest_entropy: None,
            round_id: 0,
        })
    }

    async fn submit_beacon_transaction(&self, _transaction: BeaconTransaction) -> Result<TransactionId> {
        // This would submit the actual transaction to Linera
        // For now, returning a dummy transaction ID
        Ok(TransactionId::new([0u8; 32]))
    }

    async fn get_chain_id(&self) -> Result<ChainId> {
        // Get the current chain ID from the runtime
        // For now, returning a default chain ID for the mock implementation
        // In a real implementation, this would call the actual Linera API
        Ok(ChainId::root(0))
    }

    async fn get_balance(&self, _owner: AccountOwner) -> Result<Amount> {
        // Query the balance from the chain
        // This is a simplified implementation
        Ok(Amount::from_tokens(100))
    }
}

/// Represents an entropy share submitted by a worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyShare {
    pub worker_id: String,
    pub share: [u8; 32],
    pub timestamp: u64,
}

/// Represents a unique transaction identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionId([u8; 32]);

impl TransactionId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

/// Factory function to create the appropriate LineraProvider based on configuration
pub fn create_linera_provider(use_mock: bool) -> Box<dyn LineraProvider> {
    if use_mock {
        Box::new(MockLineraProvider::new(ChainId::root(0)))
    } else {
        Box::new(LineraSdkProvider)
    }
}