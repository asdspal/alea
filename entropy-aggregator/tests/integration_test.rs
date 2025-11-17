use entropy_aggregator::aggregator::{Aggregator, AggregatorConfig};
use entropy_aggregator::linera_client::{LineraClient, LineraConfig};
use entropy_types::{NodeId};
use std::time::Duration;

#[tokio::test]
async fn test_full_aggregation_and_submission_flow() {
    // Initialize the aggregator
    let config = AggregatorConfig {
        committee_size: 2,
        threshold: 2,
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
    
    println!("✓ Aggregator initialized with Linera client");
    
    // Start a new round
    let committee: Vec<NodeId> = vec!["node1".to_string(), "node2".to_string()];
    let start_msg = aggregator.start_new_round(1, committee).await.unwrap();
    
    println!("✓ Started round: {}", start_msg.round_id);
    
    // Verify we're in the correct state
    let current_state = aggregator.get_state();
    assert!(matches!(current_state, entropy_aggregator::state_machine::AggregatorState::CollectingCommitments { round_id: 1, .. }));
    println!("✓ Correct state: CollectingCommitments");
    
    // The aggregator would normally receive commitments and reveals here
    // For this test, we'll simulate having reveals available and test the submission
    
    // Manually add some reveals to test aggregation
    {
        let reveals = aggregator.reveals.lock().unwrap();
        println!("✓ Reveals map ready, current count: {}", reveals.len());
    }
    
    // Test the aggregation function directly
    let entropy = aggregator.aggregate_reveals(1);
    assert!(entropy.is_err()); // Should fail since no reveals are available
    println!("✓ Aggregation correctly fails when no reveals available");
    
    // Test that we can submit to beacon with mock client
    let entropy = [42u8; 32];
    let attestation = vec![1u8, 2u8, 3u8];
    
    let result = aggregator.submit_randomness_to_beacon(1, entropy, attestation).await;
    assert!(result.is_ok(), "Submission should succeed with mock client");
    
    println!("✓ Randomness submission succeeds: {}", result.unwrap());
    
    // Check that submission tracking was updated
    {
        let count = aggregator.submissions_count.lock().unwrap();
        assert_eq!(*count, 1, "Submission count should be 1 after successful submission");
        println!("✓ Submission count updated to: {}", *count);
    }
    
    {
        let block = aggregator.last_submission_block.lock().unwrap();
        assert_eq!(*block, Some(1), "Last submission block should be Some(1)");
        println!("✓ Last submission block updated to: {:?}", *block);
    }
    
    println!("✓ Full integration test passed");
}