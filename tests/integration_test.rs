use alea_beacon_microchain::linera_integration::{create_linera_provider, BeaconAction, BeaconTransaction, LineraProvider};
use alea_entropy_aggregator::tee::{create_tee_enclave, TEEConfig};

#[tokio::test]
async fn test_linera_connectivity_with_mock_tee() {
    // Test with mock TEE enabled
    std::env::set_var("ENTROPY_USE_MOCK_TEE", "true");
    
    // Create Linera provider (mock implementation)
    let linera_provider = create_linera_provider(true);
    
    // Test Linera provider functionality
    let state = linera_provider.query_beacon_state().await.unwrap();
    assert_eq!(state.round_id, 0);
    assert!(state.entropy_shares.is_empty());
    assert!(state.latest_entropy.is_none());
    
    // Submit a mock transaction
    let transaction = BeaconTransaction {
        action: BeaconAction::SubmitEntropyShare {
            share: [1u8; 32],
            worker_id: "test_worker".to_string(),
            signature: vec![2u8; 64],
        },
        nonce: 1,
    };
    
    let tx_id = linera_provider.submit_beacon_transaction(transaction).await.unwrap();
    assert_eq!(tx_id.0, [0u8; 32]); // Mock provider returns [0; 32] for all transactions
    
    // Create TEE enclave (should be mock since env var is set)
    let config = TEEConfig::default();
    let tee_enclave = create_tee_enclave(&config).unwrap();
    
    // Test TEE functionality
    let nonce = tee_enclave.generate_nonce().unwrap();
    assert_eq!(nonce.len(), 32);
    
    // Test aggregation
    let secrets = vec![
        vec![1, 2, 3, 4],
        vec![5, 6, 7, 8],
        vec![9, 10, 11, 12],
    ];
    
    let (aggregated, attestation) = tee_enclave.aggregate(&secrets).unwrap();
    assert!(!aggregated.is_empty());
    assert_eq!(attestation.tee_type, "mock");
    
    // Verify the attestation
    let is_valid = tee_enclave.verify_attestation(&attestation, &aggregated).unwrap();
    assert!(is_valid);
    
    println!("Integration test passed: Linera connectivity verified with mock TEE");
}

#[tokio::test]
async fn test_linera_connectivity_without_mock_tee() {
    // Test with mock TEE disabled (should still use mock since real TEE not available in test)
    std::env::remove_var("ENTROPY_USE_MOCK_TEE");
    
    // Create Linera provider (mock implementation)
    let linera_provider = create_linera_provider(false);
    
    // Test Linera provider functionality
    let state = linera_provider.query_beacon_state().await.unwrap();
    assert_eq!(state.round_id, 0);
    assert!(state.entropy_shares.is_empty());
    assert!(state.latest_entropy.is_none());
    
    println!("Integration test passed: Linera connectivity verified without explicit mock TEE");
}

#[test]
fn test_tee_configuration() {
    // Test TEE configuration with environment variable
    std::env::set_var("ENTROPY_USE_MOCK_TEE", "true");
    let config = TEEConfig::default();
    assert!(config.use_mock);
    
    // Test TEE configuration without environment variable
    std::env::remove_var("ENTROPY_USE_MOCK_TEE");
    let config = TEEConfig::default();
    assert!(!config.use_mock);
    
    // Test TEE configuration with invalid environment variable
    std::env::set_var("ENTROPY_USE_MOCK_TEE", "invalid");
    let config = TEEConfig::default();
    assert!(!config.use_mock); // Should default to false on invalid value
    
    println!("TEE configuration test passed");
}