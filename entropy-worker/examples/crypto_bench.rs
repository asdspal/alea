use entropy_worker::crypto;
use std::time::Instant;

fn main() {
    // Profile crypto operations
    let start = Instant::now();
    for _ in 0..1000 {
        let secret = crypto::generate_secret().unwrap();
        let commitment = crypto::compute_commitment(&secret);
        let keypair = crypto::generate_keypair().unwrap();
        let signature = crypto::sign_commitment(&keypair.0, &commitment).unwrap();
    }
    println!("Crypto operations took: {:?}", start.elapsed());
}
