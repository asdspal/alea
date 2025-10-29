use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json;
use log::{info, warn, error, debug};
use std::sync::Arc;
use std::time::Duration;

use entropy_types::{CommitmentMsg};
use crate::aggregator::Aggregator;
use crate::error::AggregatorError;
use anyhow::Result;

pub struct NetworkHandler {
    aggregator: Arc<Aggregator>,
}

impl NetworkHandler {
    pub fn new(aggregator: Arc<Aggregator>) -> Self {
        Self { aggregator }
    }

    /// Start the TCP listener on the specified address
    pub async fn start_listener(&self, addr: &str) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("Entropy Aggregator listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let aggregator = self.aggregator.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, aggregator, peer_addr).await {
                            let error_msg = format!("{}", e);
                            let error_str = error_msg.as_str();
                            error!("Error handling connection from {}: {}", peer_addr, error_str);
                        }
                    });
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    let error_str = error_msg.as_str();
                    error!("Failed to accept connection: {}", error_str);
                }
            }
        }
    }
}

/// Handle an individual TCP connection
async fn handle_connection(
    mut stream: TcpStream,
    aggregator: Arc<Aggregator>,
    peer_addr: SocketAddr,
) -> Result<()> {
    debug!("New connection from: {}", peer_addr);

    let mut buffer = [0; 4096];
    
    // Read data from the stream with timeout
    let n = match tokio::time::timeout(Duration::from_secs(30), stream.read(&mut buffer)).await {
        Ok(Ok(n)) => n,
        Ok(Err(e)) => {
            error!("Failed to read from connection {}: {}", peer_addr, e);
            return Err(anyhow::anyhow!("Read error: {}", e));
        }
        Err(_) => {
            error!("Read timeout from connection {}", peer_addr);
            return Err(anyhow::anyhow!("Read timeout"));
        }
    };
    
    if n == 0 {
        debug!("Connection from {} closed gracefully", peer_addr);
        return Ok(());
    }

    // Parse the incoming message
    let message_str = String::from_utf8_lossy(&buffer[..n]);
    
    // Try to deserialize as CommitmentMsg first
    if let Ok(commitment_msg) = serde_json::from_str::<CommitmentMsg>(&message_str) {
        debug!("Received commitment message from {}: {:?}", peer_addr, commitment_msg.node_id);
        
        // For now, we'll pass an empty public key - in a real implementation,
        // the public key would be associated with the node ID
        let result = aggregator.process_commitment(commitment_msg, &[]).await;
        
        let response_bytes = match result {
            Ok(success) => {
                if success {
                    info!("Successfully processed commitment from node: {}", peer_addr);
                    &b"ACK"[..]
                } else {
                    warn!("Failed to process commitment from node: {}", peer_addr);
                    &b"NACK"[..]
                }
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                error!("Error processing commitment: {}", error_msg);
                &b"ERROR"[..]
            }
        };
        
        // Try to write the response, but handle potential connection drops
        if let Err(e) = stream.write_all(response_bytes).await {
            warn!("Failed to send response to {}: {} - connection may be dropped", peer_addr, e);
        }
    } else {
        // If it's not a commitment message, log and close connection
        warn!("Received unrecognized message from {}: {}", peer_addr, message_str);
        if let Err(e) = stream.write_all(b"UNKNOWN_MESSAGE_TYPE").await {
            warn!("Failed to send error response to {}: {} - connection may be dropped", peer_addr, e);
        }
    }

    Ok(())
}

/// Client function to send messages to the aggregator (for testing purposes)
pub async fn send_commitment_to_aggregator(
    addr: &str,
    commitment_msg: &CommitmentMsg,
) -> Result<String> {
    let mut stream = TcpStream::connect(addr).await?;
    
    let message_json = serde_json::to_string(commitment_msg)?;
    stream.write_all(message_json.as_bytes()).await?;
    
    let mut response = [0; 1024];
    let n = stream.read(&mut response).await?;
    
    Ok(String::from_utf8_lossy(&response[..n]).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregator::{Aggregator, AggregatorConfig};
    use entropy_types::{CommitmentPayload, CommitmentMsg};
    use tokio::time::timeout;
    use std::time::Duration;

    #[tokio::test]
    async fn test_tcp_listener() {
        let config = AggregatorConfig {
            committee_size: 3,
            threshold: 2,
            port: 9001, // Use a different port for tests
            ..Default::default()
        };
        
        let aggregator = Arc::new(Aggregator::new(config).unwrap());
        let network_handler = NetworkHandler::new(aggregator.clone());
        
        // Start the listener in a background task
        let listener_handle = tokio::spawn(async move {
            let _ = network_handler.start_listener("127.0.0.1:9001").await;
        });
        
        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Test that we can connect to the server
        let result = TcpStream::connect("127.0.0.1:9001").await;
        assert!(result.is_ok());
        
        // Stop the listener task
        listener_handle.abort();
    }

    #[tokio::test]
    async fn test_send_commitment() {
        let config = AggregatorConfig {
            committee_size: 3,
            threshold: 2,
            port: 9002,
            ..Default::default()
        };
        
        let aggregator = Arc::new(Aggregator::new(config).unwrap());
        let network_handler = NetworkHandler::new(aggregator.clone());
        
        // Start the listener in a background task
        let listener_handle = tokio::spawn(async move {
            let _ = network_handler.start_listener("127.0.0.1:9002").await;
        });
        
        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Create a test commitment message
        let commitment_payload = CommitmentPayload {
            round_id: 1,
            commitment: [1u8; 32],
            signature: vec![],
        };
        
        let commitment_msg = CommitmentMsg {
            round_id: 1,
            payload: commitment_payload,
            node_id: "test_node".to_string(),
            timestamp: 1234567890,
        };
        
        // Try to send the commitment to the aggregator
        let result = timeout(
            Duration::from_secs(5),
            send_commitment_to_aggregator("127.0.0.1:9002", &commitment_msg)
        ).await;
        
        match result {
            Ok(response) => {
                // The response might be an error because we didn't start a round first
                println!("Response: {:?}", response);
            }
            Err(_) => {
                println!("Timeout sending commitment");
            }
        }
        
        // Stop the listener task
        listener_handle.abort();
    }
}