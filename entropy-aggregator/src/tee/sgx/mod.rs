//! SGX TEE implementation for the entropy aggregator
//! This module provides integration with Intel SGX for secure aggregation

#[cfg(feature = "sgx")]
pub mod enclave;
#[cfg(feature = "sgx")]
pub mod untrusted;

#[cfg(feature = "sgx")]
use anyhow::Result;
#[cfg(feature = "sgx")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "sgx")]
use super::{AttestationReport, Nonce, RandomNumber, TEEEnclave};

/// SGX TEE enclave implementation
#[cfg(feature = "sgx")]
pub struct SgxTeeEnclave {
    /// Handle to the SGX enclave
    enclave: sgx_urts::SgxEnclave,
}

#[cfg(feature = "sgx")]
impl SgxTeeEnclave {
    /// Create a new SGX TEE enclave instance
    pub fn new() -> Result<Self> {
        // Load the SGX enclave
        let enclave = sgx_urts::SgxEnclave::load(
            "sgx_enclave.signed.so", 
            sgx_urts::SgxEnclaveCreateError::InvalidMetadata
        )?;
        
        Ok(Self { enclave })
    }
    
    /// Perform aggregation within the SGX enclave
    pub fn aggregate_in_enclave(&self, seed: Vec<u8>) -> Result<(RandomNumber, Nonce, AttestationReport)> {
        use sgx_types::*;
        
        let mut return_val: sgx_status_t = sgx_status_t::SGX_SUCCESS;
        let mut random_number: [u8; 32] = [0; 32];
        let mut nonce: [u8; 16] = [0; 16];
        let mut code_measurement: [u8; 32] = [0; 32];
        let mut timestamp: u64 = 0;
        
        let result = unsafe {
            crate::tee::sgx::enclave::ecall_aggregate(
                self.enclave.geteid(),
                &mut return_val,
                seed.as_ptr(),
                seed.len() as u32,
                random_number.as_mut_ptr(),
                nonce.as_mut_ptr(),
                code_measurement.as_mut_ptr(),
                &mut timestamp,
            )
        };
        
        if result != sgx_types::sgx_status_t::SGX_SUCCESS || return_val != sgx_types::sgx_status_t::SGX_SUCCESS {
            return Err(anyhow::anyhow!("SGX enclave call failed"));
        }
        
        let attestation_report = AttestationReport {
            random_number,
            nonce,
            code_measurement,
            timestamp,
        };
        
        Ok((random_number, nonce, attestation_report))
    }
}

#[cfg(feature = "sgx")]
impl TEEEnclave for SgxTeeEnclave {
    fn aggregate(&self, seed: Vec<u8>) -> Result<(RandomNumber, Nonce, AttestationReport)> {
        self.aggregate_in_enclave(seed)
    }

    fn verify_attestation(&self, report: &AttestationReport) -> Result<bool> {
        // In a real implementation, this would verify the SGX quote/attestation
        // For now, we'll implement basic verification
        let code_measurement_valid = report.code_measurement != [0; 32]; // Should not be all zeros
        let random_number_valid = report.random_number.len() == 32;
        let nonce_valid = report.nonce.len() == 16;
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let timestamp_valid = report.timestamp <= current_time + 60; // Allow 1 minute tolerance

        Ok(code_measurement_valid && random_number_valid && nonce_valid && timestamp_valid)
    }
}

/// Fallback implementation when SGX feature is not enabled
#[cfg(not(feature = "sgx"))]
pub struct SgxTeeEnclave;

#[cfg(not(feature = "sgx"))]
impl SgxTeeEnclave {
    pub fn new() -> anyhow::Result<Self> {
        anyhow::bail!("SGX feature not enabled")
    }
}