#!/bin/bash

# Alea E2E Test Runner
# Executes the complete end-to-end test suite

set -e

echo "Starting Alea End-to-End Test Suite..."

# Setup the test environment
echo "Setting up test environment..."
bash ./setup.sh

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing test dependencies..."
    npm install
fi

# Run the full test suite
echo "Running full flow tests..."
npx jest full-flow.test.ts --verbose

echo "Running performance tests..."
npx jest performance.test.ts --verbose

echo "Running error scenario tests..."
npx jest error-scenarios.test.ts --verbose

echo "Running all tests together..."
npx jest --verbose

echo "E2E Test Suite completed!"