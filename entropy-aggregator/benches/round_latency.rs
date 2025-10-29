use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use entropy_aggregator::aggregator::{Aggregator, AggregatorConfig};
use entropy_worker::worker::Worker;
use entropy_types::{CommitmentMsg, StartCommitmentMsg};
use std::sync::Arc;

// Benchmark the commitment phase (worker side)
fn bench_commitment_phase(c: &mut Criterion) {
    let mut group = c.benchmark_group("Commitment Phase");
    
    for size in [1, 2, 3, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("process_commitment", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    // Create an aggregator instance
                    let config = AggregatorConfig {
                        committee_size: size,
                        threshold: (size / 2) + 1,
                        commitment_timeout: std::time::Duration::from_secs(30),
                        reveal_timeout: std::time::Duration::from_secs(30),
                        port: 0,
                    };
                    let aggregator = Aggregator::new(config).unwrap();
                    
                    // Start a new round
                    let mut committee = Vec::new();
                    for i in 0..size {
                        committee.push(format!("node{}", i));
                    }
                    let start_msg = tokio::runtime::Runtime::new().unwrap().block_on(aggregator.start_new_round(black_box(1u64), black_box(committee))).unwrap();
                    
                    // Create a worker and generate a commitment
                    let mut worker = Worker::new("node0".to_string()).unwrap();
                    let commitment_payload = worker.handle_start_commitment(&start_msg).unwrap();
                    
                    // Create a commitment message
                    let commitment_msg = CommitmentMsg {
                        round_id: 1,
                        payload: commitment_payload,
                        node_id: "node0".to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    
                    // Process the commitment in the aggregator
                    let public_key_bytes = worker.get_public_key().serialize().to_vec();
                    let _ = aggregator.process_commitment(black_box(commitment_msg), black_box(&public_key_bytes));
                })
            },
        );
    }
    group.finish();
}

// Benchmark the reveal phase (worker and aggregator)
fn bench_reveal_phase(c: &mut Criterion) {
    let mut group = c.benchmark_group("Reveal Phase");
    
    for size in [1, 2, 3, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("process_reveal", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    // Create an aggregator instance
                    let config = AggregatorConfig {
                        committee_size: size,
                        threshold: (size / 2) + 1,
                        commitment_timeout: std::time::Duration::from_secs(30),
                        reveal_timeout: std::time::Duration::from_secs(30),
                        port: 0,
                    };
                    let aggregator = Arc::new(Aggregator::new(config).unwrap());
                    
                    // Start a new round
                    let mut committee = Vec::new();
                    for i in 0..size {
                        committee.push(format!("node{}", i));
                    }
                    let start_msg = tokio::runtime::Runtime::new().unwrap().block_on(aggregator.start_new_round(black_box(1u64), black_box(committee))).unwrap();
                    
                    // Create a worker and generate a commitment first
                    let mut worker = Worker::new("node0".to_string()).unwrap();
                    let commitment_payload = worker.handle_start_commitment(&start_msg).unwrap();
                    
                    // Create a commitment message
                    let commitment_msg = CommitmentMsg {
                        round_id: 1,
                        payload: commitment_payload,
                        node_id: "node0".to_string(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };
                    
                    // Process the commitment in the aggregator to store it
                    let public_key_bytes = worker.get_public_key().serialize().to_vec();
                    let _ = aggregator.process_commitment(black_box(commitment_msg), black_box(&public_key_bytes));
                    
                    // Create and process the reveal
                    let reveal_msg = worker.create_reveal_message().unwrap();
                    let _ = aggregator.process_reveal(black_box(reveal_msg));
                })
            },
        );
    }
    group.finish();
}

// Benchmark the full commitment-reveal round
fn bench_full_round(c: &mut Criterion) {
    let mut group = c.benchmark_group("Full Round");
    
    for size in [1, 2, 3, 5, 10] {
        group.bench_with_input(
            BenchmarkId::new("full_round", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    // Create an aggregator instance
                    let config = AggregatorConfig {
                        committee_size: size,
                        threshold: (size / 2) + 1,
                        commitment_timeout: std::time::Duration::from_secs(30),
                        reveal_timeout: std::time::Duration::from_secs(30),
                        port: 0,
                    };
                    let aggregator = Arc::new(Aggregator::new(config).unwrap());
                    
                    // Start a new round
                    let mut committee = Vec::new();
                    for i in 0..size {
                        committee.push(format!("node{}", i));
                    }
                    let start_msg = tokio::runtime::Runtime::new().unwrap().block_on(aggregator.start_new_round(black_box(1u64), black_box(committee))).unwrap();
                    
                    // Process multiple workers (simulating the full round)
                    for node_idx in 0..size {
                        let node_id = format!("node{}", node_idx);
                        let mut worker = Worker::new(node_id.clone()).unwrap();
                        
                        // Generate commitment
                        let commitment_payload = worker.handle_start_commitment(&start_msg).unwrap();
                        
                        // Create and process commitment message
                        let commitment_msg = CommitmentMsg {
                            round_id: 1,
                            payload: commitment_payload,
                            node_id: node_id.clone(),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };
                        
                        let public_key_bytes = worker.get_public_key().serialize().to_vec();
                        let _ = aggregator.process_commitment(black_box(commitment_msg), black_box(&public_key_bytes));
                        
                        // If we have enough commitments, process reveal
                        if node_idx + 1 >= (size / 2) + 1 {
                            let reveal_msg = worker.create_reveal_message().unwrap();
                            let _ = aggregator.process_reveal(black_box(reveal_msg));
                        }
                    }
                })
            },
        );
    }
    group.finish();
}

// Benchmark signature verification specifically
fn bench_signature_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("Signature Verification");
    
    // Create a worker to generate test data
    let mut worker = Worker::new("test_node".to_string()).unwrap();
    let start_msg = StartCommitmentMsg {
        round_id: 1,
        committee: vec!["test_node".to_string()],
    };
    let commitment_payload = worker.handle_start_commitment(&start_msg).unwrap();
    
    let commitment_msg = CommitmentMsg {
        round_id: 1,
        payload: commitment_payload,
        node_id: "test_node".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    let public_key_bytes = worker.get_public_key().serialize().to_vec();
    
    group.bench_function("verify_signature", |b| {
        use secp256k1::{Secp256k1, PublicKey};
        use sha2::{Sha256, Digest};
        
        b.iter(|| {
            let secp = Secp256k1::verification_only();
            
            // Deserialize the public key from bytes
            let public_key = PublicKey::from_slice(&public_key_bytes)
                .unwrap();

            // Deserialize the signature from bytes (secp256k1 signatures are 65 bytes)
            let signature_bytes = &commitment_msg.payload.signature;
            if signature_bytes.len() != 65 {
                return;
            }
            
            let recovery_id_byte = signature_bytes[64];
            let signature_bytes_64: [u8; 64] = signature_bytes[0..64].try_into()
                .unwrap();
            
            let recovery_id = secp256k1::ecdsa::RecoveryId::from_i32(recovery_id_byte as i32)
                .unwrap();
            
            let recoverable_sig = secp256k1::ecdsa::RecoverableSignature::from_compact(&signature_bytes_64, recovery_id)
                .unwrap();

            // Convert to non-recoverable signature for verification
            let signature = recoverable_sig.to_standard();

            // Create message by hashing the round_id and commitment
            let mut hasher = Sha256::new();
            hasher.update(commitment_msg.payload.round_id.to_le_bytes());
            hasher.update(&commitment_msg.payload.commitment);
            let message_hash = hasher.finalize();
            let message = secp256k1::Message::from_digest_slice(&message_hash)
                .unwrap();

            // Verify the signature
            let _ = secp.verify_ecdsa(&message, &signature, &public_key);
        })
    });
    group.finish();
}

// Benchmark commitment computation specifically
fn bench_commitment_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Commitment Computation");
    
    group.bench_function("compute_commitment", |b| {
        b.iter(|| {
            let secret = entropy_worker::crypto::generate_secret().unwrap();
            let _ = entropy_worker::crypto::compute_commitment(black_box(&secret));
        })
    });
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = 
        bench_commitment_phase,
        bench_reveal_phase,
        bench_full_round,
        bench_signature_verification,
        bench_commitment_computation,
);
criterion_main!(benches);