use anyhow::Result;
use entropy_types::CommitmentPayload;
use getrandom::getrandom;
use ring::{rand, digest};
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
use sha2::{Sha256, Digest};

/// Generate a cryptographically secure random 32-byte secret using OS RNG
pub fn generate_secret() -> Result<[u8; 32]> {
    let mut secret = [0u8; 32];
    getrandom(&mut secret).map_err(|e| anyhow::Error::msg(format!("Failed to generate random secret: {}", e)))?;
    Ok(secret)
}

/// Compute SHA256 hash of the secret to create commitment
pub fn compute_commitment(secret: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret);
    let result = hasher.finalize();
    let mut commitment = [0u8; 32];
    commitment.copy_from_slice(&result);
    commitment
}

/// Sign the commitment with the node's secp256k1 private key
pub fn sign_commitment(secret_key: &SecretKey, commitment: &[u8; 32]) -> Result<Vec<u8>> {
    let secp = Secp256k1::new();
    
    // Hash the commitment to create a message digest
    let mut hasher = Sha256::new();
    hasher.update(commitment);
    let hash_bytes = hasher.finalize();
    
    // Create a message from the hash
    let message = Message::from_digest_slice(&hash_bytes)?;
    
    // Sign the message
    let signature = secp.sign_ecdsa_recoverable(&message, secret_key);
    let (recovery_id, signature_bytes) = signature.serialize_compact();
    
    // Combine signature and recovery ID into a single vector
    let mut signature_data = vec![0u8; 65]; // 64 bytes for signature + 1 byte for recovery ID
    signature_data[0..64].copy_from_slice(&signature_bytes);
    signature_data[64] = recovery_id.to_i32() as u8;
    
    Ok(signature_data)
}

/// Generate a new secp256k1 key pair for the worker node
pub fn generate_keypair() -> Result<(SecretKey, PublicKey)> {
    let secp = Secp256k1::new();
    
    // Generate a random secret key using the OS RNG
    let mut secret_bytes = [0u8; 32];
    getrandom(&mut secret_bytes)?;
    
    // Ensure the secret key is valid for secp256k1
    let secret_key = SecretKey::from_slice(&secret_bytes)?;
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    
    Ok((secret_key, public_key))
}

/// Create a commitment payload with the secret, commitment hash, and signature
pub fn create_commitment_payload(
    round_id: u64,
    secret: &[u8; 32],
    secret_key: &SecretKey,
) -> Result<CommitmentPayload> {
    let commitment = compute_commitment(secret);
    let signature = sign_commitment(secret_key, &commitment)?;
    
    Ok(CommitmentPayload {
        round_id,
        commitment,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_generate_secret() {
        let secret1 = generate_secret().unwrap();
        let secret2 = generate_secret().unwrap();
        
        // Verify both secrets are 32 bytes
        assert_eq!(secret1.len(), 32);
        assert_eq!(secret2.len(), 32);
        
        // Verify they are different (highly likely with proper randomness)
        assert_ne!(secret1, secret2);
    }

    #[test]
    fn test_compute_commitment() {
        let secret = [1u8; 32];
        let commitment = compute_commitment(&secret);
        
        // Verify commitment is 32 bytes
        assert_eq!(commitment.len(), 32);
        
        // Verify deterministic behavior - same input produces same output
        let commitment2 = compute_commitment(&secret);
        assert_eq!(commitment, commitment2);
        
        // Verify different inputs produce different outputs
        let secret2 = [2u8; 32];
        let commitment3 = compute_commitment(&secret2);
        assert_ne!(commitment, commitment3);
    }

    #[test]
    fn test_generate_keypair() {
        let (secret_key, public_key) = generate_keypair().unwrap();
        
        // Verify we can get the public key from the secret key
        let secp = Secp256k1::new();
        let expected_public_key = PublicKey::from_secret_key(&secp, &secret_key);
        assert_eq!(public_key, expected_public_key);
    }

    #[test]
    fn test_sign_commitment() {
        let (secret_key, _) = generate_keypair().unwrap();
        let commitment = [1u8; 32];
        
        let signature = sign_commitment(&secret_key, &commitment).unwrap();
        
        // Verify signature is 65 bytes (64 bytes for signature + 1 byte for recovery ID)
        assert_eq!(signature.len(), 65);
    }

    #[test]
    fn test_create_commitment_payload() {
        let secret = [1u8; 32];
        let (secret_key, _) = generate_keypair().unwrap();
        let round_id = 123;
        
        let payload = create_commitment_payload(round_id, &secret, &secret_key).unwrap();
        
        // Verify the payload has correct round ID
        assert_eq!(payload.round_id, round_id);
        
        // Verify commitment matches expected value
        let expected_commitment = compute_commitment(&secret);
        assert_eq!(payload.commitment, expected_commitment);
        
        // Verify signature is valid (length check)
        assert_eq!(payload.signature.len(), 65);
    }

    #[test]
    fn test_end_to_end_crypto() {
        // Generate a secret
        let secret = generate_secret().unwrap();
        
        // Compute commitment
        let commitment = compute_commitment(&secret);
        
        // Generate keypair
        let (secret_key, public_key) = generate_keypair().unwrap();
        
        // Sign the commitment
        let signature = sign_commitment(&secret_key, &commitment).unwrap();
        
        // Verify all components work together
        assert_eq!(commitment.len(), 32);
        assert_eq!(signature.len(), 65);
        
        // Create commitment payload
        let payload = create_commitment_payload(42, &secret, &secret_key).unwrap();
        assert_eq!(payload.round_id, 42);
        assert_eq!(payload.commitment, compute_commitment(&secret));
        assert_eq!(payload.signature.len(), 65);
    }
}