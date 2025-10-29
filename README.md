# Alea


## Documentation

- [DEVELOPMENT.md](DEVELOPMENT.md) - Local setup, build, and test instructions
- [CONTRIBUTING.md](CONTRIBUTING.md) - Code style, PR process, and contribution guidelines
- [GLOSSARY.md](GLOSSARY.md) - Terms and definitions

A decentralized randomness beacon system built on the Linera microchain platform.

## Project Structure

- `beacon-microchain`: Linera microchain contract for the randomness beacon
- `entropy-worker`: Off-chain worker node for entropy generation
- `entropy-aggregator`: Off-chain aggregator node for collecting and submitting entropy

## Build Instructions

### Prerequisites
- Rust 1.70+ 
- Cargo

### Building the Project

To build all crates in release mode:

```bash
cargo build --all --release
```

To build individual crates:

```bash
# Build beacon microchain contract
cargo build -p beacon-microchain

# Build entropy worker
cargo build -p entropy-worker

# Build entropy aggregator
cargo build -p entropy-aggregator
```

### Running the Project

After building, you can run the off-chain components:

```bash
# Run entropy worker
cargo run --bin entropy-worker

# Run entropy aggregator
cargo run --bin entropy-aggregator
```

### Development

To check for warnings and style issues:

```bash
cargo clippy --all -- -D warnings
```

To run tests (when available):

```bash
cargo test --all
```

## Core Types

### RandomnessEvent
Represents a randomness event with:
- `round_id`: The round identifier
- `random_number`: 32-byte random number
- `nonce`: 16-byte nonce
- `attestation`: Attestation data

### BeaconState
Maintains the beacon state with:
- `current_round_id`: Current round identifier
- `events`: Map of round IDs to RandomnessEvent

## Local Development Setup

### Using Mock TEE

For local development without requiring actual TEE hardware, you can use the mock TEE implementation:

```bash
# Enable mock TEE mode
export ENTROPY_USE_MOCK_TEE=true

# Run the aggregator with mock TEE
cargo run -p entropy-aggregator
```

When using the mock TEE, you'll see a log message: "Using mock TEE for local development".

### Linera Testnet Connectivity

The system provides abstractions for connecting to the Linera testnet:

- `LinerapProvider` trait in `beacon-microchain/src/linera_integration.rs` provides an interface for Linera operations
- Mock implementation available for local testing
- Environment variable `ENTROPY_USE_MOCK_TEE` controls whether to use mock or real implementations

### Integration Testing

Run the full test suite to verify all components work together:

```bash
ENTROPY_USE_MOCK_TEE=true cargo test --all -- --nocapture
```