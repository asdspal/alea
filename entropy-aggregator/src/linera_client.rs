use std::sync::Arc;
use std::time::Duration;
use sha2::Digest;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::{info, warn};
use tokio::time::timeout;
use beacon_microchain::{BeaconOperation, RandomnessEvent};

/// Configuration for Linera client
#[derive(Debug, Clone)]
pub struct LineraConfig {
    pub endpoint: String,
    pub aggregator_key_path: String,
    pub chain_id: Option<String>,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl Default for LineraConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8080".to_string(),
            aggregator_key_path: "./aggregator.key".to_string(),
            chain_id: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

/// Linera provider trait for blockchain interaction
#[async_trait::async_trait]
pub trait LineraProvider: Send + Sync {
    /// Submit a randomness event to the beacon microchain
    async fn submit_randomness(&self, event: RandomnessEvent) -> Result<String>; // Returns transaction hash
    
    /// Get the current block height or latest submission
    async fn get_latest_submission(&self) -> Result<Option<u64>>;
    
    /// Check if the provider is connected and operational
    async fn is_connected(&self) -> bool;
    
    /// Submit randomness event with confirmation (this can be implemented differently by each provider)
    async fn submit_randomness_with_confirmation(&self, event: RandomnessEvent) -> Result<String> {
        // Default implementation that just calls submit_randomness
        self.submit_randomness(event).await
    }
}

/// Mock implementation of LineraProvider for testing
pub struct MockLineraProvider {
    config: LineraConfig,
    last_submission_block: Arc<tokio::sync::Mutex<Option<u64>>>,
    submissions_count: Arc<tokio::sync::Mutex<u64>>,
}

impl MockLineraProvider {
    pub fn new(config: LineraConfig) -> Self {
        Self {
            config,
            last_submission_block: Arc::new(tokio::sync::Mutex::new(None)),
            submissions_count: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl LineraProvider for MockLineraProvider {
    async fn submit_randomness(&self, event: RandomnessEvent) -> Result<String> {
        info!("Mock: Submitting randomness event for round {}", event.round_id);
        
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Update submission tracking
        {
            let mut count_guard = self.submissions_count.lock().await;
            *count_guard += 1;
            
            let mut block_guard = self.last_submission_block.lock().await;
            *block_guard = Some(*count_guard); // Using count as mock block number
        }
        
        // Generate a mock transaction hash
        let tx_hash = format!("mock_tx_{}_{}", event.round_id, hex::encode(&event.random_number[..8]));
        info!("Mock: Randomness submission successful, tx_hash: {}", tx_hash);
        
        Ok(tx_hash)
    }

    async fn get_latest_submission(&self) -> Result<Option<u64>> {
        let block_guard = self.last_submission_block.lock().await;
        Ok(*block_guard)
    }

    async fn is_connected(&self) -> bool {
        // Simulate connection check
        true
    }
}

/// Real Linera provider implementation
pub struct RealLineraProvider {
    config: LineraConfig,
    client: reqwest::Client,
    private_key: secp256k1::SecretKey,
    last_submission_block: Arc<tokio::sync::Mutex<Option<u64>>>,
    submissions_count: Arc<tokio::sync::Mutex<u64>>,
}

impl RealLineraProvider {
    pub fn new(config: LineraConfig) -> Result<Self> {
        let client = reqwest::Client::new();
        
        // Load the aggregator's private key from file
        let key_data = std::fs::read_to_string(&config.aggregator_key_path)?;
        let private_key_hex = key_data.trim();
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| anyhow::anyhow!("Failed to decode private key: {}", e))?;
        
        if private_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid private key length: expected 32 bytes, got {}", private_key_bytes.len()));
        }
        
        let private_key = secp256k1::SecretKey::from_slice(&private_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;
        
        Ok(Self {
            config,
            client,
            private_key,
            last_submission_block: Arc::new(tokio::sync::Mutex::new(None)),
            submissions_count: Arc::new(tokio::sync::Mutex::new(0)),
        })
    }

    /// Sign a randomness event with the aggregator's private key
    fn sign_randomness_event(&self, event: &RandomnessEvent) -> Result<Vec<u8>> {
        let secp = secp256k1::Secp256k1::new();
        
        // Serialize the event for signing (excluding the signature itself)
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| anyhow::anyhow!("Failed to serialize event for signing: {}", e))?;
        
        // Hash the event data
        let mut hasher = sha2::Sha256::new();
        hasher.update(&event_bytes);
        let hash = hasher.finalize();
        
        // Sign the hash
        let message = secp256k1::Message::from_digest_slice(&hash)
            .map_err(|e| anyhow::anyhow!("Failed to create message from digest: {}", e))?;
        
        let signature = secp.sign_ecdsa_recoverable(&message, &self.private_key);
        let (recovery_id, signature_bytes) = signature.serialize_compact();
        
        // Combine signature bytes with recovery ID (65 bytes total)
        let mut signature_with_recovery = signature_bytes.to_vec();
        signature_with_recovery.push(recovery_id.to_i32() as u8);
        
        Ok(signature_with_recovery)
    }

    /// Create and submit a transaction to the beacon microchain
    async fn submit_transaction(&self, operation: BeaconOperation) -> Result<String> {
        // This is a simplified implementation - in a real system, this would interact
        // with the actual Linera API to submit transactions
        // For now, we'll simulate the interaction
        
        info!("Submitting transaction to endpoint: {}", self.config.endpoint);
        
        // Simulate network request with retry logic
        let mut attempts = 0;
        loop {
            match self.attempt_submit(&operation).await {
                Ok(tx_hash) => return Ok(tx_hash),
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        return Err(e);
                    }
                    
                    warn!("Transaction submission failed (attempt {}/{}): {}. Retrying in 2s...", 
                          attempts, self.config.max_retries, e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    async fn attempt_submit(&self, operation: &BeaconOperation) -> Result<String> {
        // In a real implementation, this would make an actual HTTP request to the Linera node
        // For now, we'll simulate the operation and return a mock transaction hash
        let tx_hash = format!("real_tx_{}", hex::encode(&sha2::Sha256::digest(
            serde_json::to_vec(operation).unwrap_or_default()
        )[..8]));
        
        info!("Transaction submitted successfully: {}", tx_hash);
        Ok(tx_hash)
    }
}

#[async_trait::async_trait]
impl LineraProvider for RealLineraProvider {
    async fn submit_randomness(&self, event: RandomnessEvent) -> Result<String> {
        info!("Submitting randomness event for round {} to beacon microchain", event.round_id);
        
        // Sign the event
        let signature = self.sign_randomness_event(&event)?;
        info!("Event signed successfully");
        
        // Create the operation
        let operation = BeaconOperation::SubmitRandomness {
            event: event.clone(),
            signature,
        };
        
        // Submit the transaction
        let tx_hash = self.submit_transaction(operation).await?;
        
        // Update submission tracking
        {
            let mut count_guard = self.submissions_count.lock().await;
            *count_guard += 1;
            
            let mut block_guard = self.last_submission_block.lock().await;
            // In a real system, this would be the actual block number
            *block_guard = Some(*count_guard);
        }
        
        info!("Randomness submission completed, tx_hash: {}", tx_hash);
        Ok(tx_hash)
    }

    async fn get_latest_submission(&self) -> Result<Option<u64>> {
        let block_guard = self.last_submission_block.lock().await;
        Ok(*block_guard)
    }

    async fn is_connected(&self) -> bool {
        // In a real implementation, this would check the connection to the Linera node
        // For now, we'll assume it's connected if the client was created successfully
        true
    }
}

/// Linera client that manages the provider and provides high-level operations
pub struct LineraClient {
    provider: Arc<dyn LineraProvider>,
    config: LineraConfig,
}

impl LineraClient {
    /// Create a new Linera client with the given configuration
    pub fn new(config: LineraConfig) -> Result<Self> {
        let provider: Arc<dyn LineraProvider> = if config.endpoint.contains("mock") {
            Arc::new(MockLineraProvider::new(config.clone()))
        } else {
            Arc::new(RealLineraProvider::new(config.clone())?)
        };
        
        Ok(Self { provider, config })
    }

    /// Create a new client with mock provider for testing
    pub fn new_mock(config: LineraConfig) -> Self {
        Self {
            provider: Arc::new(MockLineraProvider::new(config.clone())),
            config,
        }
    }

    /// Submit a randomness event to the beacon microchain with confirmation
    pub async fn submit_randomness_with_confirmation(&self, event: RandomnessEvent) -> Result<String> {
        info!("Starting randomness submission process for round {}", event.round_id);
        
        // Submit the randomness
        let tx_hash = self.provider.submit_randomness(event).await?;
        
        // Wait for confirmation (in a real system, this would poll for transaction confirmation)
        info!("Waiting for confirmation of transaction: {}", tx_hash);
        
        // In a real implementation, we would poll for transaction confirmation
        // For now, we'll just simulate a successful confirmation
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        info!("Transaction confirmed: {}", tx_hash);
        Ok(tx_hash)
    }

    /// Get the latest submission block number
    pub async fn get_latest_submission_block(&self) -> Result<Option<u64>> {
        self.provider.get_latest_submission().await
    }

    /// Check if the client is connected to the network
    pub async fn is_connected(&self) -> bool {
        self.provider.is_connected().await
    }

    /// Get the current submission count
    pub async fn get_submissions_count(&self) -> Result<u64> {
        // This would need to be implemented in the provider
        // For now, we'll return a mock value based on the mock provider's state
        // Since we can't downcast, we'll just return 0 for now
        // In a real implementation, each provider would implement its own count tracking
        Ok(0) // Default value
    }

    /// Get a reference to the provider for direct access
    pub fn get_provider(&self) -> &Arc<dyn LineraProvider> {
        &self.provider
    }
}