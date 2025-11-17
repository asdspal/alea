use entropy_aggregator::aggregator::{Aggregator, AggregatorConfig};
use entropy_aggregator::linera_client::{LineraClient, LineraConfig};
use entropy_types::{NodeId};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the aggregator
    let config = AggregatorConfig {
        committee_size: 3,
        threshold: 2,
        commitment_timeout: Duration::from_secs(30),
        reveal_timeout: Duration::from_secs(30),
        port: 900,
    };
    
    let mut aggregator = Aggregator::new(config)?;
    
    // Initialize the Linera client with mock configuration
    let linera_config = LineraConfig {
        endpoint: "http://localhost:8080".to_string(),
        aggregator_key_path: "./aggregator.key".to_string(), // This file doesn't need to exist for mock
        chain_id: None,
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };
    
    aggregator.initialize_mock_linera_client(linera_config);
    
    println!("Aggregator initialized with Linera client");
    println!("Current state: {:?}", aggregator.get_state());
    
    // Example: Start a new round
    let committee: Vec<NodeId> = vec!["node1".to_string(), "node2".to_string(), "node3".to_string()];
    let start_msg = aggregator.start_new_round(1, committee).await?;
    
    println!("Started round: {}", start_msg.round_id);
    println!("Current state: {:?}", aggregator.get_state());
    
    // The aggregator would normally receive commitments and reveals here
    // For this example, we'll just show that the Linera client is available
    
    if aggregator.linera_client.is_some() {
        println!("Linera client is properly initialized");
        
        // Example: Get submission count (should be 0 initially)
        let count = aggregator.submissions_count.lock().unwrap();
        println!("Submission count: {}", *count);
        drop(count);
    } else {
        println!("ERROR: Linera client not initialized");
    }
    
    Ok(())
}