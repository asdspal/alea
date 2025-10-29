use std::fmt;

#[derive(Debug)]
pub enum AggregatorError {
    /// Node doesn't send commitment within timeout
    CommitmentTimeout { node_id: String, round_id: u64 },
    /// Node sends invalid commitment (bad signature)
    InvalidCommitmentSignature { node_id: String, round_id: u64 },
    /// Node sends reveal that doesn't match commitment
    InvalidReveal { node_id: String, round_id: u64 },
    /// Network connection error
    NetworkError { node_id: String, message: String },
    /// Invalid round ID in message
    InvalidRoundId { received: u64, expected: u64 },
    /// Node not in committee
    NodeNotInCommittee { node_id: String, round_id: u64 },
    /// Aggregator internal error
    InternalError { message: String },
    /// TEE/Enclave error
    TEEError { message: String },
    /// Configuration error
    ConfigError { message: String },
}

impl fmt::Display for AggregatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregatorError::CommitmentTimeout { node_id, round_id } => {
                write!(f, "Commitment timeout for node {} in round {}", node_id, round_id)
            }
            AggregatorError::InvalidCommitmentSignature { node_id, round_id } => {
                write!(f, "Invalid commitment signature from node {} in round {}", node_id, round_id)
            }
            AggregatorError::InvalidReveal { node_id, round_id } => {
                write!(f, "Invalid reveal from node {} in round {}", node_id, round_id)
            }
            AggregatorError::NetworkError { node_id, message } => {
                write!(f, "Network error with node {}: {}", node_id, message)
            }
            AggregatorError::InvalidRoundId { received, expected } => {
                write!(f, "Invalid round ID: received {}, expected {}", received, expected)
            }
            AggregatorError::NodeNotInCommittee { node_id, round_id } => {
                write!(f, "Node {} not in committee for round {}", node_id, round_id)
            }
            AggregatorError::InternalError { message } => {
                write!(f, "Internal error: {}", message)
            }
            AggregatorError::TEEError { message } => {
                write!(f, "TEE error: {}", message)
            }
            AggregatorError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
        }
    }
}

impl std::error::Error for AggregatorError {}

#[derive(Debug)]
pub enum WorkerError {
    /// Connection to aggregator failed
    ConnectionFailed { address: String, attempts: u32 },
    /// Aggregator rejected commitment
    CommitmentRejected { message: String },
    /// Aggregator rejected reveal
    RevealRejected { message: String },
    /// Network error during communication
    NetworkError { message: String },
    /// Worker internal error
    InternalError { message: String },
    /// Crypto operation failed
    CryptoError { message: String },
    /// Invalid message format
    InvalidMessage { message: String },
}

impl fmt::Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerError::ConnectionFailed { address, attempts } => {
                write!(f, "Failed to connect to aggregator at {} after {} attempts", address, attempts)
            }
            WorkerError::CommitmentRejected { message } => {
                write!(f, "Commitment rejected by aggregator: {}", message)
            }
            WorkerError::RevealRejected { message } => {
                write!(f, "Reveal rejected by aggregator: {}", message)
            }
            WorkerError::NetworkError { message } => {
                write!(f, "Network error: {}", message)
            }
            WorkerError::InternalError { message } => {
                write!(f, "Internal error: {}", message)
            }
            WorkerError::CryptoError { message } => {
                write!(f, "Crypto error: {}", message)
            }
            WorkerError::InvalidMessage { message } => {
                write!(f, "Invalid message: {}", message)
            }
        }
    }
}

impl std::error::Error for WorkerError {}

// Helper trait to convert errors to AggregatorError
pub trait IntoAggregatorError<T> {
    fn into_agg_error(self, error_type: fn(String) -> AggregatorError) -> Result<T, AggregatorError>;
}

impl<T, E: std::fmt::Display> IntoAggregatorError<T> for Result<T, E> {
    fn into_agg_error(self, error_type: fn(String) -> AggregatorError) -> Result<T, AggregatorError> {
        self.map_err(|e| error_type(e.to_string()))
    }
}