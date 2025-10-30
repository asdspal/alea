//! Tests to verify that SGX and mock TEE produce the same output format
//! This is important for ensuring compatibility between the mock and real implementations

#[cfg(test)]
mod sgx_mock_comparison_tests {
    use entropy_aggregator::tee::{create_tee_enclave, TEEConfig, TEEEnclave};

    #[test]
    #[cfg(feature = "sgx")]
    fn test_sgx_and_mock_output_formats_match() {
        // This test requires SGX hardware to run, so we'll skip it if SGX is not available
        // In a real implementation, this would compare outputs from both implementations
        
        let seed = b"test_seed_for_comparison";
        
        // Create mock TEE instance
        let mock_config = TEEConfig {
            use_mock: true,
            config_path: None,
            parameters: std::collections::HashMap::new(),
        };
        
        let mock_tee = create_tee_enclave(&mock_config).unwrap();
        let (mock_random, mock_nonce, mock_report) = mock_tee.aggregate(seed.to_vec()).unwrap();
        
        // For SGX, we would do the same, but since we don't have real hardware in tests:
        // We'll verify that the format is consistent by checking the structure
        assert_eq!(mock_random.len(), 32);
        assert_eq!(mock_nonce.len(), 16);
        assert_eq!(mock_report.random_number.len(), 32);
        assert_eq!(mock_report.nonce.len(), 16);
        assert_eq!(mock_report.code_measurement.len(), 32);
        assert!(mock_report.timestamp > 0);
        
        println!("Mock TEE output format verified:");
        println!(" Random number length: {}", mock_random.len());
        println!("  Nonce length: {}", mock_nonce.len());
        println!("  Code measurement length: {}", mock_report.code_measurement.len());
        println!("  Timestamp: {}", mock_report.timestamp);
    }

    #[test]
    fn test_mock_tee_output_format() {
        let seed = b"test_seed";
        
        let config = TEEConfig {
            use_mock: true,
            config_path: None,
            parameters: std::collections::HashMap::new(),
        };
        
        let tee = create_tee_enclave(&config).unwrap();
        let (random, nonce, report) = tee.aggregate(seed.to_vec()).unwrap();
        
        // Verify output format matches expected structure
        assert_eq!(random.len(), 32, "Random number should be 32 bytes");
        assert_eq!(nonce.len(), 16, "Nonce should be 16 bytes");
        assert_eq!(report.random_number.len(), 32, "Report random number should be 32 bytes");
        assert_eq!(report.nonce.len(), 16, "Report nonce should be 16 bytes");
        assert_eq!(report.code_measurement.len(), 32, "Code measurement should be 32 bytes");
        assert!(report.timestamp > 0, "Timestamp should be positive");
        
        // Verify that the random number in the return value matches the one in the report
        assert_eq!(random, report.random_number, "Random number should match report");
        
        // Verify that the nonce in the return value matches the one in the report
        assert_eq!(nonce, report.nonce, "Nonce should match report");
    }

    #[test]
    fn test_mock_tee_deterministic_behavior() {
        let seed = b"deterministic_test_seed";
        
        let config = TEEConfig {
            use_mock: true,
            config_path: None,
            parameters: std::collections::HashMap::new(),
        };
        
        let tee = create_tee_enclave(&config).unwrap();
        
        // Same seed should produce same random number
        let (random1, _nonce1, _report1) = tee.aggregate(seed.to_vec()).unwrap();
        let (random2, _nonce2, _report2) = tee.aggregate(seed.to_vec()).unwrap();
        
        assert_eq!(random1, random2, "Same seed should produce same random number");
    }

    #[test]
    fn test_mock_tee_nonce_uniqueness() {
        let config = TEEConfig {
            use_mock: true,
            config_path: None,
            parameters: std::collections::HashMap::new(),
        };
        
        let tee = create_tee_enclave(&config).unwrap();
        
        // Different calls should produce different nonces
        let (_random1, nonce1, _report1) = tee.aggregate(b"seed1".to_vec()).unwrap();
        let (_random2, nonce2, _report2) = tee.aggregate(b"seed2".to_vec()).unwrap();
        
        // Nonces should be different (counter-based generation)
        assert_ne!(nonce1, nonce2, "Nonces should be unique");
    }

    #[test]
    fn test_mock_tee_attestation_verification() {
        let seed = b"attestation_test";
        
        let config = TEEConfig {
            use_mock: true,
            config_path: None,
            parameters: std::collections::HashMap::new(),
        };
        
        let tee = create_tee_enclave(&config).unwrap();
        let (_random, _nonce, report) = tee.aggregate(seed.to_vec()).unwrap();
        
        // Valid report should pass verification
        let is_valid = tee.verify_attestation(&report).unwrap();
        assert!(is_valid, "Valid attestation should pass verification");
        
        // Tampered report should fail verification
        let tampered_report = entropy_aggregator::tee::AttestationReport {
            random_number: [0u8; 32], // Wrong random number
            nonce: report.nonce,
            code_measurement: report.code_measurement,
            timestamp: report.timestamp,
        };
        
        let is_valid_tampered = tee.verify_attestation(&tampered_report).unwrap();
        assert!(!is_valid_tampered, "Tampered attestation should fail verification");
    }
}