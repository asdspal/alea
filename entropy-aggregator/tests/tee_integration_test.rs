use entropy_aggregator::tee::{create_tee_enclave, TEEConfig};

#[test]
fn test_full_tee_aggregation_round() {
    // Create a mock TEE enclave
    let config = TEEConfig {
        use_mock: true,
        config_path: None,
        parameters: std::collections::HashMap::new(),
    };
    
    let tee = create_tee_enclave(&config).unwrap();
    
    // Test the aggregate function
    let seed = b"integration_test_seed".to_vec();
    let (random_number, nonce, attestation_report) = tee.aggregate(seed.clone()).unwrap();
    
    // Verify the results
    assert_eq!(random_number.len(), 32); // RandomNumber is [u8; 32]
    assert_eq!(nonce.len(), 16);        // Nonce is [u8; 16]
    assert_eq!(attestation_report.code_measurement.len(), 32); // code_measurement is [u8; 32]
    
    // Verify the attestation
    let is_valid = tee.verify_attestation(&attestation_report).unwrap();
    assert!(is_valid);
    
    // Test with different seed to ensure deterministic behavior
    let seed2 = b"integration_test_seed_2".to_vec();
    let (random_number2, nonce2, attestation_report2) = tee.aggregate(seed2).unwrap();
    
    // Different seeds should produce different random numbers
    assert_ne!(random_number, random_number2);
    
    // Nonces should be different (counter-based)
    assert_ne!(nonce, nonce2);
    
    // But code measurements should be the same
    assert_eq!(attestation_report.code_measurement, attestation_report2.code_measurement);
}

#[test]
fn test_tee_with_empty_seed() {
    let config = TEEConfig {
        use_mock: true,
        config_path: None,
        parameters: std::collections::HashMap::new(),
    };
    
    let tee = create_tee_enclave(&config).unwrap();
    
    // Test with empty seed
    let seed = vec![];
    let (random_number, nonce, attestation_report) = tee.aggregate(seed).unwrap();
    
    // Verify the results
    assert_eq!(random_number.len(), 32);
    assert_eq!(nonce.len(), 16);
    
    // Verify the attestation
    let is_valid = tee.verify_attestation(&attestation_report).unwrap();
    assert!(is_valid);
}

#[test]
fn test_multiple_aggregations_for_nonce_uniqueness() {
    let config = TEEConfig {
        use_mock: true,
        config_path: None,
        parameters: std::collections::HashMap::new(),
    };
    
    let tee = create_tee_enclave(&config).unwrap();
    
    // Generate multiple aggregations and ensure nonces are unique
    let mut nonces = std::collections::HashSet::new();
    for i in 0..50 {
        let seed = format!("test_seed_{}", i).into_bytes();
        let (_random, nonce, _report) = tee.aggregate(seed).unwrap();
        assert!(nonces.insert(nonce)); // insert returns false if already present
    }
}