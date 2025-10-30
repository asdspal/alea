//! Untrusted wrapper for SGX enclave interaction
//! This module contains the untrusted code that loads and calls the enclave

#[cfg(feature = "sgx")]
use anyhow::Result;
#[cfg(feature = "sgx")]
use sgx_types::*;
#[cfg(feature = "sgx")]
use sgx_urts::SgxEnclave;

#[cfg(feature = "sgx")]
pub struct SgxEnclaveWrapper {
    enclave: SgxEnclave,
}

#[cfg(feature = "sgx")]
impl SgxEnclaveWrapper {
    /// Create a new enclave wrapper by loading the enclave
    pub fn new(enclave_file: &str) -> Result<Self> {
        let enclave = Self::load_enclave(enclave_file)?;
        Ok(Self { enclave })
    }

    /// Load the SGX enclave from file
    fn load_enclave(enclave_file: &str) -> Result<SgxEnclave> {
        let launch_token = [0; 1024];
        let mut launch_token_updated = 0;
        let mut misc_attr = sgx_misc_attribute_t { 
            secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 }, 
            misc_select: 0 
        };

        let enclave = SgxEnclave::create(
            enclave_file,
            sgx_debug_flag_t::SGX_DEBUG_FLAG_UNPRIVILEGED,
            &mut launch_token,
            &mut launch_token_updated,
            &mut misc_attr,
        )?;

        Ok(enclave)
    }

    /// Call the enclave to perform aggregation
    pub fn aggregate(&self, seed: &[u8]) -> Result<(super::RandomNumber, super::Nonce, super::AttestationReport)> {
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

        if result != sgx_status_t::SGX_SUCCESS || return_val != sgx_status_t::SGX_SUCCESS {
            return Err(anyhow::anyhow!("SGX enclave call failed with status: {:?}, return: {:?}", result, return_val));
        }

        let attestation_report = super::AttestationReport {
            random_number,
            nonce,
            code_measurement,
            timestamp,
        };

        Ok((random_number, nonce, attestation_report))
    }

    /// Generate an attestation report from the enclave
    pub fn get_attestation_report(&self) -> Result<sgx_report_t> {
        // Create report data with some identifying information
        let mut report_data = sgx_report_data_t::default();
        // Fill report_data with some meaningful data (in practice, this would be application-specific)
        for i in 0..32 {
            report_data.d[i] = i as u8;
        }

        let mut target_info = sgx_target_info_t::default();
        let mut report = sgx_report_t::default();

        let mut return_val: sgx_status_t = sgx_status_t::SGX_SUCCESS;
        
        let result = unsafe {
            crate::tee::sgx::enclave::ecall_get_attestation_report(
                self.enclave.geteid(),
                &mut return_val,
                &report_data as *const sgx_report_data_t,
                std::ptr::null_mut(), // Use default target info
                &mut report,
            )
        };

        if result != sgx_status_t::SGX_SUCCESS {
            return Err(anyhow::anyhow!("Failed to get attestation report from enclave"));
        }

        Ok(report)
    }

    /// Extract SGX quote from the report (for remote attestation)
    pub fn generate_quote(&self) -> Result<Vec<u8>> {
        // In a real implementation, this would generate a proper SGX quote
        // using the attestation service. This is a simplified version.
        
        // First get the report
        let report = self.get_attestation_report()?;
        
        // In a real implementation, we would send the report to the Quoting Enclave
        // to get a quote. For now, we'll simulate this with the report data.
        
        // This is a simplified approach - in reality, you'd use Intel's attestation services
        let mut quote = Vec::new();
        quote.extend_from_slice(&report.body.mr_enclave.m);
        quote.extend_from_slice(&report.body.mr_signer.m);
        quote.extend_from_slice(&report.body.isv_prod_id.to_le_bytes());
        quote.extend_from_slice(&report.body.isv_svn.to_le_bytes());
        
        Ok(quote)
    }

    /// Get the underlying enclave handle
    pub fn get_enclave(&self) -> &SgxEnclave {
        &self.enclave
    }
}

// Mock implementation when SGX feature is not enabled
#[cfg(not(feature = "sgx"))]
pub struct SgxEnclaveWrapper;

#[cfg(not(feature = "sgx"))]
impl SgxEnclaveWrapper {
    pub fn new(_enclave_file: &str) -> Result<Self> {
        anyhow::bail!("SGX feature not enabled")
    }
}