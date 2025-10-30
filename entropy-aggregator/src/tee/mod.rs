use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod mock;

/// Random number type - 32 bytes
pub type RandomNumber = [u8; 32];

/// Nonce type - 16 bytes
pub type Nonce = [u8; 16];

/// Attestation report from the TEE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationReport {
    pub random_number: RandomNumber,
    pub nonce: Nonce,
    pub code_measurement: [u8; 32], // SHA256 of enclave code
    pub timestamp: u64,
}

/// Trait that abstracts TEE enclave operations for the entropy aggregator
pub trait TEEEnclave: Send + Sync {
    /// Aggregate entropy secrets within the TEE and produce an attestation
    /// Returns the random number, nonce, and attestation report
    fn aggregate(&self, seed: Vec<u8>) -> Result<(RandomNumber, Nonce, AttestationReport)>;

    /// Verify an attestation report produced by a TEE
    fn verify_attestation(&self, report: &AttestationReport) -> Result<bool>;
}

// Note: The old Attestation struct has been replaced by AttestationReport
// which contains the specific fields required for TEE attestation

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
        #[cfg(feature = "sgx")]
        {
            println!("Using SGX TEE");
            Ok(Box::new(sgx::SgxTeeEnclave::new()?))
        }
        #[cfg(not(feature = "sgx"))]
        {
            // If SGX feature is not enabled, fall back to mock
            println!("SGX feature not enabled, using mock TEE");
            Ok(Box::new(mock::MockTeeEnclave::new()))
        }
    }
}