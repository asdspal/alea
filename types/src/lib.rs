use serde::{Deserialize, Serialize};

/// Protocol version constant
pub const PROTOCOL_VERSION: u32 = 1;

/// Node identifier type
pub type NodeId = String;

/// Commitment payload containing round ID, commitment hash, and signature
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitmentPayload {
    pub round_id: u64,
    pub commitment: [u8; 32],
    pub signature: Vec<u8>,
}

/// Reveal payload containing round ID and secret
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RevealPayload {
    pub round_id: u64,
    pub secret: [u8; 32],
}

/// Start commitment message to initiate the commitment phase
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StartCommitmentMsg {
    pub round_id: u64,
    pub committee: Vec<NodeId>,
}

/// Start reveal message to initiate the reveal phase
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct StartRevealMsg {
    pub round_id: u64,
}

/// Attestation report containing TEE-specific fields
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AttestationReport {
    pub report: Vec<u8>,
    pub signature: Vec<u8>,
    pub signing_cert: Vec<u8>,
    pub tee_type: String,
}

/// Commitment message containing payload and metadata
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitmentMsg {
    pub round_id: u64,
    pub payload: CommitmentPayload,
    pub node_id: NodeId,
    pub timestamp: u64,
}

/// Reveal message containing payload and metadata
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RevealMsg {
    pub round_id: u64,
    pub payload: RevealPayload,
    pub node_id: NodeId,
    pub timestamp: u64,
}

/// Entropy generation request message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntropyRequest {
    pub request_id: String,
    pub client_id: String,
    pub timestamp: u64,
    pub nonce: [u8; 32],
}

/// Entropy generation response message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntropyResponse {
    pub request_id: String,
    pub round_id: u64,
    pub entropy: [u8; 32],
    pub attestation: AttestationReport,
    pub timestamp: u64,
}

/// Heartbeat message for node health monitoring
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HeartbeatMsg {
    pub node_id: NodeId,
    pub timestamp: u64,
    pub status: String,
}

/// Error message for protocol errors
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ErrorMessage {
    pub error_code: u32,
    pub error_message: String,
    pub timestamp: u64,
}

/// Round completion message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RoundCompletionMsg {
    pub round_id: u64,
    pub entropy: [u8; 32],
    pub participants: Vec<NodeId>,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_serialization() {
        let commitment = CommitmentPayload {
            round_id: 1,
            commitment: [1u8; 32],
            signature: vec![2u8, 3u8, 4u8],
        };

        let json = serde_json::to_string(&commitment).unwrap();
        let deserialized: CommitmentPayload = serde_json::from_str(&json).unwrap();
        
        assert_eq!(commitment, deserialized);
    }

    #[test]
    fn test_reveal_serialization() {
        let reveal = RevealPayload {
            round_id: 1,
            secret: [5u8; 32],
        };

        let json = serde_json::to_string(&reveal).unwrap();
        let deserialized: RevealPayload = serde_json::from_str(&json).unwrap();
        
        assert_eq!(reveal, deserialized);
    }

    #[test]
    fn test_start_commitment_msg_serialization() {
        let msg = StartCommitmentMsg {
            round_id: 1,
            committee: vec!["node1".to_string(), "node2".to_string()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: StartCommitmentMsg = serde_json::from_str(&json).unwrap();
        
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_start_reveal_msg_serialization() {
        let msg = StartRevealMsg {
            round_id: 1,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: StartRevealMsg = serde_json::from_str(&json).unwrap();
        
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_attestation_report_serialization() {
        let attestation = AttestationReport {
            report: vec![1u8, 2u8, 3u8],
            signature: vec![4u8, 5u8, 6u8],
            signing_cert: vec![7u8, 8u8, 9u8],
            tee_type: "sgx".to_string(),
        };

        let json = serde_json::to_string(&attestation).unwrap();
        let deserialized: AttestationReport = serde_json::from_str(&json).unwrap();
        
        assert_eq!(attestation, deserialized);
    }

    #[test]
    fn test_protocol_version_constant() {
        assert_eq!(PROTOCOL_VERSION, 1);
    }
}