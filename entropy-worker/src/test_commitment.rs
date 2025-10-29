#[cfg(test)]
mod commitment_integration_tests {
    use crate::crypto::{generate_secret, compute_commitment, sign_commitment, generate_keypair, create_commitment_payload};
    use crate::worker::Worker;
    use entropy_types::{StartCommitmentMsg, CommitmentPayload};

    #[test]
    fn test_full_commitment_flow() {
        // 1. Generate a secret
        let secret = generate_secret().unwrap();
        assert_eq!(secret.len(), 32);

        // 2. Generate keypair for the worker
        let (secret_key, public_key) = generate_keypair().unwrap();

        // 3. Compute commitment from secret
        let commitment = compute_commitment(&secret);
        assert_eq!(commitment.len(), 32);

        // 4. Sign the commitment
        let signature = sign_commitment(&secret_key, &commitment).unwrap();
        assert_eq!(signature.len(), 65); // 64 bytes for signature + 1 byte for recovery ID

        // 5. Create commitment payload
        let round_id = 42;
        let payload = create_commitment_payload(round_id, &secret, &secret_key).unwrap();
        
        assert_eq!(payload.round_id, round_id);
        assert_eq!(payload.commitment, commitment);
        assert_eq!(payload.signature.len(), 65);

        println!("Full commitment flow test passed:");
        println!("  Secret: {}", hex::encode(&secret));
        println!("  Commitment: {}", hex::encode(&commitment));
        println!(" Signature: {}", hex::encode(&signature));
        println!("  Round ID: {}", payload.round_id);
    }

    #[test]
    fn test_worker_commitment_generation() {
        // Create a worker instance
        let mut worker = Worker::new("test-worker-1".to_string()).unwrap();
        
        // Create a start commitment message
        let start_msg = StartCommitmentMsg {
            round_id: 123,
            committee: vec!["test-worker-1".to_string()],
        };
        
        // Handle the start commitment message
        let payload = worker.handle_start_commitment(&start_msg).unwrap();
        
        // Verify the payload
        assert_eq!(payload.round_id, 123);
        assert_eq!(worker.get_current_round_id(), Some(123));
        assert!(worker.get_current_secret().is_some());
        assert!(worker.is_participating());
        
        println!("Worker commitment generation test passed:");
        println!("  Generated payload for round: {}", payload.round_id);
        println!("  Commitment: {}", hex::encode(&payload.commitment));
        println!("  Signature: {}", hex::encode(&payload.signature));
    }

    #[test]
    fn test_multiple_workers_different_secrets() {
        // Create multiple workers
        let mut worker1 = Worker::new("test-worker-1".to_string()).unwrap();
        let mut worker2 = Worker::new("test-worker-2".to_string()).unwrap();
        
        // Create start commitment messages
        let start_msg = StartCommitmentMsg {
            round_id: 456,
            committee: vec!["test-worker-1".to_string(), "test-worker-2".to_string()],
        };
        
        // Both workers generate commitments
        let payload1 = worker1.handle_start_commitment(&start_msg).unwrap();
        let payload2 = worker2.handle_start_commitment(&start_msg).unwrap();
        
        // Verify both payloads have the same round ID but different commitments
        assert_eq!(payload1.round_id, 456);
        assert_eq!(payload2.round_id, 456);
        assert_ne!(payload1.commitment, payload2.commitment);
        
        println!("Multiple workers test passed:");
        println!("  Worker1 commitment: {}", hex::encode(&payload1.commitment));
        println!("  Worker2 commitment: {}", hex::encode(&payload2.commitment));
    }

    #[test]
    fn test_commitment_with_known_values() {
        // Use a known secret for deterministic testing
        let known_secret = [42u8; 32];
        
        // Generate keypair
        let (secret_key, _) = generate_keypair().unwrap();
        
        // Compute commitment
        let commitment = compute_commitment(&known_secret);
        
        // Expected commitment using a known SHA256 implementation
        let mut hasher = sha2::Sha256::new();
        hasher.update(&known_secret);
        let expected_commitment = hasher.finalize();
        
        assert_eq!(commitment, expected_commitment.as_slice());
        
        // Sign the commitment
        let signature = sign_commitment(&secret_key, &commitment).unwrap();
        assert_eq!(signature.len(), 65);
        
        // Create payload
        let payload = create_commitment_payload(789, &known_secret, &secret_key).unwrap();
        assert_eq!(payload.round_id, 789);
        assert_eq!(payload.commitment, commitment);
        
        println!("Known values test passed:");
        println!("  Known secret: {}", hex::encode(&known_secret));
        println!("  Computed commitment: {}", hex::encode(&commitment));
        println!("  Expected commitment: {}", hex::encode(expected_commitment));
    }

    #[test]
    fn test_end_to_end_commitment_protocol() {
        // Simulate the complete commitment phase of the protocol
        
        // 1. Initialize worker
        let mut worker = Worker::new("end-to-end-worker".to_string()).unwrap();
        
        // 2. Receive start commitment message from aggregator
        let start_msg = StartCommitmentMsg {
            round_id: 999,
            committee: vec!["end-to-end-worker".to_string(), "other-worker-1".to_string(), "other-worker-2".to_string()],
        };
        
        // 3. Process the message and generate commitment
        let payload = worker.handle_start_commitment(&start_msg).unwrap();
        
        // 4. Verify all components of the payload
        assert_eq!(payload.round_id, 999);
        assert_eq!(worker.get_current_round_id(), Some(999));
        assert!(worker.get_current_secret().is_some());
        assert_eq!(payload.commitment, compute_commitment(&worker.get_current_secret().unwrap()));
        
        // 5. Verify signature length
        assert_eq!(payload.signature.len(), 65);
        
        println!("End-to-end commitment protocol test passed:");
        println!("  Round ID: {}", payload.round_id);
        println!("  Commitment: {}", hex::encode(&payload.commitment));
        println!("  Signature: {}", hex::encode(&payload.signature));
        println!("  Worker participating: {}", worker.is_participating());
    }
}