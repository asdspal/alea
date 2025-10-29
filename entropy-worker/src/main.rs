use log::{info, debug, error};
use env_logger::Env;
use entropy_types::StartCommitmentMsg;
use std::env;
use tokio::signal;

mod crypto;
mod worker;
mod network;

use crate::worker::Worker;
use crate::network::TcpClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    info!("Entropy Worker Node starting...");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let offline_mode = args.iter().any(|arg| arg == "--mode=offline" || arg == "--offline");
    
    // Initialize worker with a default node ID
    let node_id = format!("worker-{}", rand::random::<u64>());
    let mut worker = Worker::new(node_id)?;
    
    if offline_mode {
        info!("Running in offline mode - generating commitment without network connection");
        
        // Simulate receiving a start commitment message
        let start_msg = StartCommitmentMsg {
            round_id: 1,
            committee: vec![worker.get_node_id().to_string()],
        };
        
        // Generate commitment payload
        let payload = worker.handle_start_commitment(&start_msg)?;
        
        info!("Generated commitment payload for round {}: {:?}", payload.round_id, hex::encode(&payload.commitment));
        info!("Signature: {}", hex::encode(&payload.signature));
        
        // In offline mode, just display the results and exit
        println!("Commitment generated successfully:");
        println!("  Round ID: {}", payload.round_id);
        println!("  Commitment: {}", hex::encode(&payload.commitment));
        println!("  Signature: {}", hex::encode(&payload.signature));
        
        return Ok(());
    }
    
    // Normal mode - connect to aggregator and participate in protocol
    debug!("Worker node initialized with ID: {}", worker.get_node_id());
    
    // Initialize TCP client to connect to aggregator
    let mut tcp_client = TcpClient::new("localhost:900");
    
    // Attempt to connect to aggregator
    match tcp_client.connect() {
        Ok(()) => info!("Successfully connected to aggregator"),
        Err(e) => {
            error!("Failed to connect to aggregator: {}", e);
            return Err(e.into());
        }
    }
    
    // Simulate receiving a start commitment message
    // In a real implementation, this would come from the aggregator via network
    let start_msg = StartCommitmentMsg {
        round_id: 1,
        committee: vec![worker.get_node_id().to_string()],
    };
    
    // Handle the start commitment message and generate payload
    let payload = worker.handle_start_commitment(&start_msg)?;
    
    // Send the commitment to the aggregator
    match tcp_client.send_commitment(&payload) {
        Ok(()) => info!("Successfully sent commitment to aggregator for round {}", payload.round_id),
        Err(e) => {
            error!("Failed to send commitment to aggregator: {}", e);
            return Err(e.into());
        }
    }
    
    info!("Commitment phase completed successfully");
    
    // Wait for shutdown signal
    info!("Press Ctrl+C to shutdown gracefully...");
    signal::ctrl_c().await?;
    info!("Received shutdown signal, cleaning up...");
    
    // Perform cleanup - reset worker state
    worker.reset_state();
    
    info!("Worker shutdown complete");
    
    Ok(())
}

// Helper function to generate a mock start commitment message for testing
#[cfg(test)]
fn create_mock_start_commitment_msg(round_id: u64, node_id: &str) -> StartCommitmentMsg {
    StartCommitmentMsg {
        round_id,
        committee: vec![node_id.to_string()],
    }
}