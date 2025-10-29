use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod mock;

/// Trait that abstracts TEE enclave operations for the entropy aggregator
pub trait TEEEnclave: Send + Sync {
    /// Generate a cryptographically secure nonce within the TEE
    fn generate_nonce(&self) -> Result<[u8; 32]>;

    /// Aggregate entropy secrets within the TEE and produce an attestation
    /// Returns the aggregated output and a cryptographic attestation
    fn aggregate(&self, secrets: &[Vec<u8>]) -> Result<(Vec<u8>, Attestation)>;

    /// Verify an attestation produced by a TEE
    fn verify_attestation(&self, attestation: &Attestation, data: &[u8]) -> Result<bool>;
}

/// Represents an attestation from a TEE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// The attestation report from the TEE
    pub report: Vec<u8>,
    /// Public key used to verify the attestation
    pub public_key: Vec<u8>,
    /// Signature of the data + report
    pub signature: Vec<u8>,
    /// Type of TEE (e.g., "sgx", "sev", "mock")
    pub tee_type: String,
}

/// Configuration for TEE operations
#[derive(Debug, Clone)]
pub struct TEEConfig {
    /// Whether to use mock TEE (for local development)
    pub use_mock: bool,
    /// Path to TEE configuration files
    pub config_path: Option<String>,
    /// Additional parameters for TEE initialization
    pub parameters: std::collections::HashMap<String, String>,
}

impl Default for TEEConfig {
    fn default() -> Self {
        Self {
            use_mock: std::env::var("ENTROPY_USE_MOCK_TEE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            config_path: None,
            parameters: std::collections::HashMap::new(),
        }
    }
}

/// Factory function to create the appropriate TEEEnclave based on configuration
pub fn create_tee_enclave(config: &TEEConfig) -> Result<Box<dyn TEEEnclave>> {
    if config.use_mock {
        println!("Using mock TEE for local development");
        Ok(Box::new(mock::MockTeeEnclave::new()))
    } else {
        // In a real implementation, this would initialize the actual TEE
        // For now, we'll default to mock since actual TEE setup is complex
        println!("Using mock TEE (real TEE not implemented yet)");
        Ok(Box::new(mock::MockTeeEnclave::new()))
    }
}