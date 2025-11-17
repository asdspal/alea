use entropy_aggregator::aggregator::{Aggregator, AggregatorConfig};
use entropy_aggregator::linera_client::{LineraClient, LineraConfig};
use std::time::Duration;

#[tokio::test]
async fn test_aggregator_with_linera_client() {
    // Initialize the aggregator
    let config = AggregatorConfig {
        committee_size: 3,
        threshold: 2,
        commitment_timeout: Duration::from_secs(30),
        reveal_timeout: Duration::from_secs(30),
        port: 9000,
    };
    
    let mut aggregator = Aggregator::new(config).unwrap();
    
    // Initialize the Linera client with mock configuration
    let linera_config = LineraConfig {
        endpoint: "http://localhost:8080".to_string(),
        aggregator_key_path: "./aggregator.key".to_string(), // This file doesn't need to exist for mock
        chain_id: None,
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };
    
    aggregator.initialize_mock_linera_client(linera_config);
    
    // Verify that the Linera client is properly initialized
    assert!(aggregator.linera_client.is_some(), "Linera client should be initialized");
    
    println!("✓ Aggregator initialized with Linera client");
    
    // Test that we can get the submission count
    {
        let count = aggregator.submissions_count.lock().unwrap();
        assert_eq!(*count, 0, "Initial submission count should be 0");
        println!("✓ Initial submission count is 0: {}", *count);
    }
    
    // Test that we can get the last submission block
    {
        let block = aggregator.last_submission_block.lock().unwrap();
        assert!(block.is_none(), "Initial last submission block should be None");
        println!("✓ Initial last submission block is None");
    }
    
    // Test entropy aggregation
    let test_round_id = 1;
    let test_entropy = [1u8; 32];
    let test_attestation = vec![1u8, 2u8, 3u8];
    
    // Since there are no reveals, this should fail
    let result = aggregator.aggregate_reveals(test_round_id);
    assert!(result.is_err(), "Should fail when no reveals are available");
    println!("✓ Correctly fails when no reveals are available");
    
    // Test that we can access the reveals map to add some test data
    {
        let reveals = aggregator.reveals.lock().unwrap();
        println!("✓ Can access reveals map (current count: {})", reveals.len());
    }
    
    // Test the state transitions
    let initial_state = aggregator.get_state();
    assert!(initial_state.is_idle(), "Initial state should be Idle");
    println!("✓ Initial state is Idle: {:?}", initial_state);
}

#[tokio::test]
async fn test_randomness_submission() {
    // Initialize the aggregator
    let config = AggregatorConfig {
        committee_size: 1,
        threshold: 1,
        commitment_timeout: Duration::from_secs(30),
        reveal_timeout: Duration::from_secs(30),
        port: 9000,
    };
    
    let mut aggregator = Aggregator::new(config).unwrap();
    
    // Initialize the Linera client with mock configuration
    let linera_config = LineraConfig {
        endpoint: "mock://test".to_string(),
        aggregator_key_path: "./aggregator.key".to_string(),
        chain_id: None,
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };
    
    aggregator.initialize_mock_linera_client(linera_config);
    
    // Test submission without Linera client should fail
    let entropy = [42u8; 32];
    let attestation = vec![1u8, 2u8, 3u8];
    
    // This should succeed now that we have a client
    let result = aggregator.submit_randomness_to_beacon(1, entropy, attestation).await;
    assert!(result.is_ok(), "Submission should succeed with mock client");
    
    println!("✓ Randomness submission succeeds with mock client: {:?}", result.unwrap());
    
    // Check that submission count was incremented
    {
        let count = aggregator.submissions_count.lock().unwrap();
        assert_eq!(*count, 1, "Submission count should be 1 after successful submission");
        println!("✓ Submission count incremented to: {}", *count);
    }
    
    // Check that last submission block was updated
    {
        let block = aggregator.last_submission_block.lock().unwrap();
        assert_eq!(*block, Some(1), "Last submission block should be Some(1)");
        println!("✓ Last submission block updated to: {:?}", *block);
    }
}

#[tokio::test]
async fn test_aggregator_creation() {
    let config = AggregatorConfig::default();
    let aggregator = Aggregator::new(config).unwrap();
    
    // Check that all new fields are properly initialized
    assert!(aggregator.linera_client.is_none(), "Linera client should be None initially");
    {
        let count = aggregator.submissions_count.lock().unwrap();
        assert_eq!(*count, 0, "Initial submissions count should be 0");
    }
    {
        let block = aggregator.last_submission_block.lock().unwrap();
        assert!(block.is_none(), "Initial last submission block should be None");
    }
    
    println!("✓ Aggregator created with all new fields properly initialized");
}