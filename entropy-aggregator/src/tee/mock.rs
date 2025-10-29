use anyhow::Result;
use rand::RngCore;
use sha2::{Sha256, Digest};

use super::{Attestation, TEEEnclave};

/// Mock TEE enclave that simulates SGX behavior without requiring actual hardware
pub struct MockTeeEnclave {
    /// Simulated private key for signing attestations
    private_key: [u8; 32],
    /// Simulated public key for verification
    public_key: [u8; 32],
}

impl MockTeeEnclave {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut private_key = [0u8; 32];
        let mut public_key = [0u8; 32];
        
        rng.fill_bytes(&mut private_key);
        // In a real system, public key would be derived from private key
        // For the mock, we'll just hash the private key to get a deterministic public key
        let mut hasher = Sha256::new();
        hasher.update(&private_key);
        public_key.copy_from_slice(&hasher.finalize());

        Self {
            private_key,
            public_key,
        }
    }

    /// Deterministically generate attestation report for given data
    fn generate_attestation_report(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(b"mock_attestation_report");
        hasher.update(data);
        hasher.update(&self.public_key);
        hasher.finalize().to_vec()
    }

    /// Simulate signing with the private key (in reality this would be inside the TEE)
    fn sign_data(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.update(&self.private_key);
        hasher.finalize().to_vec()
    }
}

impl TEEEnclave for MockTeeEnclave {
    fn generate_nonce(&self) -> Result<[u8; 32]> {
        let mut rng = rand::thread_rng();
        let mut nonce = [0u8; 32];
        rng.fill_bytes(&mut nonce);
        Ok(nonce)
    }

    fn aggregate(&self, secrets: &[Vec<u8>]) -> Result<(Vec<u8>, Attestation)> {
        // Aggregate secrets using XOR (in a real implementation, this would be more sophisticated)
        let mut aggregated = vec![0u8; 32];
        
        for secret in secrets {
            for (i, byte) in secret.iter().enumerate() {
                if i < aggregated.len() {
                    aggregated[i] ^= byte;
                }
            }
        }

        // Create a deterministic attestation based on the aggregated data
        let report = self.generate_attestation_report(&aggregated);
        let signature = self.sign_data(&aggregated);

        let attestation = Attestation {
            report,
            public_key: self.public_key.to_vec(),
            signature,
            tee_type: "mock".to_string(),
        };

        Ok((aggregated, attestation))
    }

    fn verify_attestation(&self, attestation: &Attestation, data: &[u8]) -> Result<bool> {
        // Verify that the attestation matches the data
        let expected_report = self.generate_attestation_report(data);
        let expected_signature = self.sign_data(data);

        let report_matches = attestation.report == expected_report;
        let signature_matches = attestation.signature == expected_signature;
        let public_key_matches = attestation.public_key == self.public_key.to_vec();

        Ok(report_matches && signature_matches && public_key_matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tee_nonce_generation() {
        let tee = MockTeeEnclave::new();
        let nonce1 = tee.generate_nonce().unwrap();
        let nonce2 = tee.generate_nonce().unwrap();
        
        // Nonces should be different (with high probability)
        assert_ne!(nonce1, nonce2);
        
        // Nonces should be 32 bytes
        assert_eq!(nonce1.len(), 32);
        assert_eq!(nonce2.len(), 32);
    }

    #[test]
    fn test_mock_tee_aggregate() {
        let tee = MockTeeEnclave::new();
        
        let secrets = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
        ];
        
        let (aggregated, attestation) = tee.aggregate(&secrets).unwrap();
        
        // Check that we got a result
        assert!(!aggregated.is_empty());
        
        // Check that we got an attestation
        assert!(!attestation.report.is_empty());
        assert!(!attestation.public_key.is_empty());
        assert!(!attestation.signature.is_empty());
        assert_eq!(attestation.tee_type, "mock");
    }

    #[test]
    fn test_mock_tee_attestation_verification() {
        let tee = MockTeeEnclave::new();
        
        let secrets = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let (aggregated, attestation) = tee.aggregate(&secrets).unwrap();
        
        // Verify the attestation
        let is_valid = tee.verify_attestation(&attestation, &aggregated).unwrap();
        assert!(is_valid);
        
        // Verify with wrong data should fail
        let wrong_data = vec![0u8; 32];
        let is_valid_wrong = tee.verify_attestation(&attestation, &wrong_data).unwrap();
        assert!(!is_valid_wrong);
    }
}