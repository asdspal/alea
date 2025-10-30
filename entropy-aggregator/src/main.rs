use log::{info, debug, error};
use env_logger::Env;
use std::sync::Arc;
use clap::Parser;
use tokio::signal;

use entropy_aggregator::tee::{create_tee_enclave, TEEConfig};
use entropy_aggregator::aggregator::{Aggregator, AggregatorConfig};
use entropy_aggregator::network::NetworkHandler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Committee size for the entropy generation
    #[arg(long, default_value_t = 3)]
    committee_size: usize,
    
    /// Threshold for the entropy generation
    #[arg(long, default_value_t = 2)]
    threshold: usize,
    
    /// Port to listen on
    #[arg(long, default_value_t = 900)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    
    info!("Entropy Aggregator Node starting...");
    
    let args = Args::parse();
    
    // Create aggregator configuration
    let config = AggregatorConfig {
        committee_size: args.committee_size,
        threshold: args.threshold,
        port: args.port,
        ..Default::default()
    };
    
    // Create the aggregator
    let aggregator = Arc::new(Aggregator::new(config)?);
    
    // Create TEE enclave based on configuration
    let tee_config = TEEConfig::default();
    let tee_enclave = create_tee_enclave(&tee_config)?;
    
    debug!("Aggregator node initialized with TEE configuration");
    
    // Test TEE functionality
    // Example of aggregation (with a test seed for now)
    let seed = b"test_seed_for_main".to_vec();
    let (random_number, nonce, attestation_report) = tee_enclave.aggregate(seed)?;
    info!("TEE generated random number: {:?}", &random_number[..8]); // Log first 8 bytes for brevity
    info!("TEE generated nonce: {:?}", &nonce[..8]); // Log first 8 bytes for brevity
    info!("TEE generated attestation with code measurement: {:?}", &attestation_report.code_measurement[..8]);
    
    // Create network handler and start listening
    let network_handler = NetworkHandler::new(aggregator.clone());
    let addr = format!("0.0.0.0:{}", args.port);
    
    // Start the TCP listener in a background task
    let network_handle = tokio::spawn(async move {
        if let Err(e) = network_handler.start_listener(&addr).await {
            error!("Network error: {}", e);
        }
    });
    
    // Log initial state
    info!("Aggregator initial state: {:?}", aggregator.get_state());
    info!("Listening on port {}", args.port);
    
    // Start the aggregator with timeout handling in a background task
    let aggregator_clone = aggregator.clone();
    let aggregator_handle = tokio::spawn(async move {
        if let Err(e) = aggregator_clone.run_with_timeout().await {
            error!("Aggregator error: {}", e);
        }
    });
    
    // Wait for shutdown signal
    info!("Press Ctrl+C to shutdown gracefully...");
    signal::ctrl_c().await?;
    info!("Received shutdown signal, cleaning up...");
    
    // Perform cleanup - transition aggregator to idle state
    {
        let mut state_guard = aggregator.state.lock().unwrap();
        *state_guard = entropy_aggregator::state_machine::AggregatorState::Idle;
    }
    
    info!("Aggregator shutdown complete");
    
    // Cancel the spawned tasks
    network_handle.abort();
    aggregator_handle.abort();
    
    Ok(())
}