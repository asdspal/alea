#!/bin/bash

# Alea E2E Test Environment Setup Script
# Sets up the test environment for end-to-end testing

set -e

echo "Setting up Alea E2E test environment..."

# Create necessary directories
mkdir -p alea/tests/e2e/logs
mkdir -p alea/tests/e2e/data

# Set environment variables for testing
export ALEA_TEST_MODE=true
export ALEA_LOG_LEVEL=info
export ALEA_NETWORK_MODE=testing

# Build the aggregator if not already built
if [ ! -f "alea/target/debug/entropy-aggregator" ]; then
    echo "Building entropy-aggregator..."
    cd alea/entropy-aggregator
    cargo build --bin entropy-aggregator
    cd ../..
fi

# Build the worker if not already built
if [ ! -f "alea/target/debug/entropy-worker" ]; then
    echo "Building entropy-worker..."
    cd alea/entropy-worker
    cargo build --bin entropy-worker
    cd ../..
fi

# Set up mock SGX environment for testing
export SGX_MODE=SW
export ALEA_USE_MOCK_SGX=true

# Configure test parameters
export ALEA_TEST_AGGREGATOR_PORT=8080
export ALEA_TEST_WORKER_PORT=8081
export ALEA_TEST_TIMEOUT=30

echo "Test environment setup complete!"