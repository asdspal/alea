#!/bin/bash

# Deployment script for Alea Beacon contract to Linera testnet
# This script handles the complete deployment process of the beacon contract to Linera

set -e

# Default values
NETWORK="testnet"
CONTRACT_PATH="./alea/beacon-microchain"
CHAIN_ID=""
VERBOSE=false

# Function to display usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --network=NETWORK    Network to deploy to (default: testnet)"
    echo "  --chain-id=CHAIN_ID  Chain ID to deploy to (optional, will create new if not provided)"
    echo "  --verbose            Enable verbose output"
    echo "  --help               Display this help message"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --network=*)
            NETWORK="${1#*=}"
            shift
            ;;
        --chain-id=*)
            CHAIN_ID="${1#*=}"
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Function for logging
log() {
    if [ "$VERBOSE" = true ]; then
        echo "[INFO] $1"
    fi
}

# Check if linera is installed
if ! command -v linera &> /dev/null; then
    echo "Error: linera is not installed. Please install Linera SDK first."
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust toolchain first."
    exit 1
fi

echo "Starting deployment of Alea Beacon contract to $NETWORK..."

# Build the contract
log "Building beacon contract..."
cd $CONTRACT_PATH
cargo build --release --target wasm32-unknown-unknown
cd - > /dev/null

log "Contract built successfully"

# Deploy the contract
log "Deploying contract to $NETWORK..."

if [ -z "$CHAIN_ID" ]; then
    log "No chain ID provided, creating a new microchain..."
    # Create a new chain for the beacon contract
    CHAIN_ID=$(linera --network $NETWORK create-chain --initial-funding 1 | head -n 1 | cut -d' ' -f1)
    log "Created new chain with ID: $CHAIN_ID"
else
    log "Using provided chain ID: $CHAIN_ID"
fi

# Publish the contract to the chain
log "Publishing contract to chain..."
PUBLICATION_RESULT=$(linera --network $NETWORK --chain $CHAIN_ID publish-bytecode \
    $CONTRACT_PATH/target/wasm32-unknown/release/beacon_microchain.wasm)

CONTRACT_ID=$(echo $PUBLICATION_RESULT | cut -d' ' -f1)
log "Contract published with ID: $CONTRACT_ID"

# Instantiate the contract
log "Instantiating contract..."
INSTANTIATION_RESULT=$(linera --network $NETWORK --chain $CHAIN_ID create-application $CONTRACT_ID \
    --json-argument '"test_admin_key"' --json-parameters '{}')

INSTANCE_ID=$(echo $INSTANTIATION_RESULT | cut -d' ' -f1)
log "Contract instantiated with instance ID: $INSTANCE_ID"

# Output deployment information
cat << EOF

Deployment completed successfully!

Deployment Information:
- Network: $NETWORK
- Chain ID: $CHAIN_ID
- Contract ID: $CONTRACT_ID
- Instance ID: $INSTANCE_ID

Beacon microchain is now deployed and ready to receive randomness submissions.

To interact with the contract, you can use:
linera --network $NETWORK --chain $CHAIN_ID query-service --application-id $INSTANCE_ID --json-query '{"GetRandomness": {"round_id": 1}}'
EOF

# Create a deployment manifest file for future reference
DEPLOYMENT_MANIFEST="alea/beacon-microchain/deployment-manifest.json"
cat > $DEPLOYMENT_MANIFEST << MANIFEST
{
  "network": "$NETWORK",
  "chain_id": "$CHAIN_ID",
  "contract_id": "$CONTRACT_ID",
  "instance_id": "$INSTANCE_ID",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "admin_public_key": "test_admin_key"
}
MANIFEST

echo "Deployment manifest saved to: $DEPLOYMENT_MANIFEST"

echo "Deployment script completed successfully!"