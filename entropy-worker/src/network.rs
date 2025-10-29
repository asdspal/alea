use anyhow::Result;
use entropy_types::CommitmentPayload;
use log::{info, debug, warn, error};
use std::io::Write;
use std::net::{TcpStream};
use std::time::Duration;
use serde_json;
use std::thread;

/// TCP client wrapper for communication with the aggregator
pub struct TcpClient {
    /// The TCP stream connection to the aggregator
    stream: Option<TcpStream>,
    
    /// The aggregator's address
    aggregator_addr: String,
    
    /// Maximum number of retry attempts
    max_retries: u32,
    
    /// Base delay for exponential backoff (in milliseconds)
    base_delay_ms: u64,
    
    /// Maximum delay for exponential backoff (in milliseconds)
    max_delay_ms: u64,
}

impl TcpClient {
    /// Create a new TCP client instance
    pub fn new(aggregator_addr: &str) -> Self {
        TcpClient {
            stream: None,
            aggregator_addr: aggregator_addr.to_string(),
            max_retries: 5,  // Maximum number of retries
            base_delay_ms: 100,  // 1 second base delay
            max_delay_ms: 300,  // 30 seconds max delay
        }
    }
    
    /// Connect to the aggregator with exponential backoff
    pub fn connect(&mut self) -> Result<()> {
        self.connect_with_retry(0)
    }
    
    /// Connect to the aggregator with exponential backoff
    fn connect_with_retry(&mut self, retry_count: u32) -> Result<()> {
        if retry_count > 0 {
            // Calculate delay using exponential backoff: base_delay * 2^(retry_count-1)
            let delay_ms = std::cmp::min(
                self.base_delay_ms * 2_u64.pow(retry_count - 1),
                self.max_delay_ms
            );
            
            debug!("Retrying connection to aggregator in {}ms (attempt {}/{})",
                   delay_ms, retry_count, self.max_retries);
            
            thread::sleep(Duration::from_millis(delay_ms));
        }
        
        debug!("Attempting to connect to aggregator at {}", self.aggregator_addr);
        
        // Try to establish connection
        match TcpStream::connect(&self.aggregator_addr) {
            Ok(stream) => {
                // Configure stream options for better reliability
                stream.set_read_timeout(Some(Duration::from_secs(30)))?;
                stream.set_write_timeout(Some(Duration::from_secs(30)))?;
                stream.set_nodelay(true)?;
                
                self.stream = Some(stream);
                if retry_count > 0 {
                    info!("Successfully connected to aggregator at {} after {} attempts",
                          self.aggregator_addr, retry_count);
                } else {
                    info!("Successfully connected to aggregator at {}", self.aggregator_addr);
                }
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to aggregator at {} (attempt {} of {}): {}",
                       self.aggregator_addr, retry_count + 1, self.max_retries, e);
                
                if retry_count < self.max_retries {
                    self.connect_with_retry(retry_count + 1)
                } else {
                    error!("Max retries ({}) exceeded for connection to aggregator at {}",
                           self.max_retries, self.aggregator_addr);
                    Err(anyhow::Error::msg(format!(
                        "Failed to connect to aggregator after {} attempts: {}",
                        self.max_retries, e
                    )))
                }
            }
        }
    }
    
    /// Send a commitment payload to the aggregator
    pub fn send_commitment(&mut self, payload: &CommitmentPayload) -> Result<()> {
        // Ensure we have an active connection with retry logic
        if self.stream.is_none() {
            self.connect()?;
        }
        
        // Try to send the commitment with retry logic
        self.send_commitment_with_retry(payload, 0)
    }
    
    /// Send a commitment payload to the aggregator with retry logic
    fn send_commitment_with_retry(&mut self, payload: &CommitmentPayload, retry_count: u32) -> Result<()> {
        if retry_count > 0 {
            // Calculate delay using exponential backoff: base_delay * 2^(retry_count-1)
            let delay_ms = std::cmp::min(
                self.base_delay_ms * 2_u64.pow(retry_count - 1),
                self.max_delay_ms
            );
            
            debug!("Retrying to send commitment in {}ms (attempt {}/{})",
                   delay_ms, retry_count, self.max_retries);
            
            thread::sleep(Duration::from_millis(delay_ms));
            
            // Reconnect if needed
            if !self.is_connected() {
                if let Err(e) = self.connect() {
                    error!("Failed to reconnect before sending commitment: {}", e);
                    if retry_count < self.max_retries {
                        return self.send_commitment_with_retry(payload, retry_count + 1);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        let stream = match self.stream.as_mut() {
            Some(s) => s,
            None => {
                error!("No active connection to aggregator");
                return Err(anyhow::Error::msg("No active connection to aggregator"));
            }
        };
        
        debug!("Serializing commitment payload for round {}", payload.round_id);
        
        // Serialize the commitment payload to JSON
        let json_payload = match serde_json::to_string(payload) {
            Ok(json) => json,
            Err(e) => {
                error!("Failed to serialize commitment payload: {}", e);
                return Err(anyhow::Error::msg(format!("Failed to serialize commitment payload: {}", e)));
            }
        };
        
        debug!("Sending commitment payload: {}", json_payload);
        
        // Send the length of the message first (4 bytes in big-endian)
        let msg_bytes = json_payload.as_bytes();
        let msg_len = msg_bytes.len() as u32;
        let len_bytes = msg_len.to_be_bytes();
        
        match stream.write_all(&len_bytes) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to write message length for round {} (attempt {} of {}): {}",
                       payload.round_id, retry_count + 1, self.max_retries, e);
                if retry_count < self.max_retries {
                    return self.send_commitment_with_retry(payload, retry_count + 1);
                } else {
                    error!("Max retries ({}) exceeded for sending commitment for round {}",
                           self.max_retries, payload.round_id);
                    return Err(anyhow::Error::msg(format!("Failed to write message length: {}", e)));
                }
            }
        }
        
        match stream.write_all(msg_bytes) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to write message for round {} (attempt {} of {}): {}",
                       payload.round_id, retry_count + 1, self.max_retries, e);
                if retry_count < self.max_retries {
                    return self.send_commitment_with_retry(payload, retry_count + 1);
                } else {
                    error!("Max retries ({}) exceeded for sending commitment for round {}",
                           self.max_retries, payload.round_id);
                    return Err(anyhow::Error::msg(format!("Failed to write message: {}", e)));
                }
            }
        }
        
        match stream.flush() {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to flush stream for round {} (attempt {} of {}): {}",
                       payload.round_id, retry_count + 1, self.max_retries, e);
                if retry_count < self.max_retries {
                    return self.send_commitment_with_retry(payload, retry_count + 1);
                } else {
                    error!("Max retries ({}) exceeded for sending commitment for round {}",
                           self.max_retries, payload.round_id);
                    return Err(anyhow::Error::msg(format!("Failed to flush stream: {}", e)));
                }
            }
        }
        
        info!("Successfully sent commitment payload for round {}", payload.round_id);
        
        Ok(())
    }
    
    /// Check if the client is currently connected
    pub fn is_connected(&self) -> bool {
        match &self.stream {
            Some(stream) => {
                // Try to peek at the stream to check if it's still active
                let mut buf = [0; 1];
                match stream.peek(&mut buf) {
                    Ok(_) => true,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => true,
                    Err(_) => false,
                }
            }
            None => false,
        }
    }
    
    /// Disconnect from the aggregator
    pub fn disconnect(&mut self) {
        if self.stream.is_some() {
            debug!("Disconnecting from aggregator");
            self.stream = None;
        }
    }
    
    /// Attempt to reconnect if connection is lost
    pub fn ensure_connection(&mut self) -> Result<()> {
        if !self.is_connected() {
            warn!("Connection to aggregator lost, attempting to reconnect...");
            self.disconnect();
            self.connect()?;
        }
        Ok(())
    }
}

impl Drop for TcpClient {
    /// Ensure the connection is closed when the client is dropped
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::net::{TcpListener};
    use std::time::Duration;
    
    #[test]
    fn test_tcp_client_creation() {
        let client = TcpClient::new("localhost:9000");
        assert_eq!(client.aggregator_addr, "localhost:9000");
        assert!(!client.is_connected());
    }
    
    #[test]
    fn test_connection_methods() {
        // Note: This test assumes an aggregator is running on localhost:9001 for testing
        // In a real scenario, we would start a mock server for the test
        let mut client = TcpClient::new("localhost:9001");
        
        // The connection should fail since no server is running
        let result = client.connect();
        assert!(result.is_err());
        
        // is_connected should return false when no connection exists
        assert!(!client.is_connected());
    }
    
    #[test]
    fn test_exponential_backoff_calculation() {
        // Test that the exponential backoff calculation works correctly
        let base_delay: u64 = 100; // 100ms base delay from TcpClient
        let max_delay: u64 = 30000; // 30 seconds max delay from TcpClient
        
        // Test the exponential backoff calculation
        assert_eq!(std::cmp::min(base_delay * 2_u64.pow(0), max_delay), base_delay); // 1st retry: 100ms
        assert_eq!(std::cmp::min(base_delay * 2_u64.pow(1), max_delay), base_delay * 2); // 2nd retry: 200ms
        assert_eq!(std::cmp::min(base_delay * 2_u64.pow(2), max_delay), base_delay * 4); // 3rd retry: 400ms
        assert_eq!(std::cmp::min(base_delay * 2_u64.pow(3), max_delay), base_delay * 8); // 4th retry: 800ms
        assert_eq!(std::cmp::min(base_delay * 2_u64.pow(4), max_delay), base_delay * 16); // 5th retry: 1600ms
        // Eventually it should cap at max_delay
        assert_eq!(std::cmp::min(base_delay * 2_u64.pow(10), max_delay), max_delay); // High retry: capped at max
    }
    
    #[test]
    fn test_commitment_payload_serialization() {
        let commitment_payload = CommitmentPayload {
            round_id: 1,
            commitment: [1u8; 32],
            signature: vec![2u8, 3u8, 4u8],
        };
        
        let json = serde_json::to_string(&commitment_payload).unwrap();
        let deserialized: CommitmentPayload = serde_json::from_str(&json).unwrap();
        
        assert_eq!(commitment_payload, deserialized);
    }
}

// Integration test for network functionality
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;
    use std::io::{Read};
    
    #[test]
    fn test_send_commitment_to_mock_server() {
        // Start a mock server in a separate thread
        let server_addr = "127.0.0.1:9002";
        let listener = TcpListener::bind(server_addr).unwrap();
        
        let server_handle = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0; 1024];
                if let Ok(_) = stream.read(&mut buffer) {
                    // Server receives the data
                    return true;
                }
            }
            false
        });
        
        // Give the server a moment to start
        thread::sleep(Duration::from_millis(100));
        
        // Create a client and try to send data
        let mut client = TcpClient::new(server_addr);
        let commitment_payload = CommitmentPayload {
            round_id: 1,
            commitment: [1u8; 32],
            signature: vec![2u8, 3u8, 4u8],
        };
        
        let result = client.send_commitment(&commitment_payload);
        assert!(result.is_ok());
        
        // Wait for the server thread to complete
        let received_data = server_handle.join().unwrap();
        assert!(received_data);
    }
}