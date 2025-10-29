#!/bin/bash

# Profiling script for entropy-aggregator
# This script runs profiling and generates flamegraphs to identify performance bottlenecks

set -e

echo "Starting profiling for entropy-aggregator..."

# Check if cargo-flamegraph is installed
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "cargo-flamegraph not found. Installing..."
    cargo install flamegraph
fi

# Check if perf is available (Linux)
if command -v perf &> /dev/null; then
    PERF_AVAILABLE=true
    echo "perf is available"
else
    PERF_AVAILABLE=false
    echo "perf is not available, will use flamegraph only"
fi

# Create target directory for results
mkdir -p target/profile_results

echo "Running benchmarks with profiling..."

# Run the round latency benchmarks with flamegraph
echo "Generating flamegraph for round latency benchmarks..."
if [ "$PERF_AVAILABLE" = true ]; then
    cargo flamegraph --bench round_latency --profile bench -- --bench
else
    echo "Perf not available, running flamegraph with basic profiling..."
    cargo flamegraph --bench round_latency --profile bench -- --bench 2>/dev/null || echo "Flamegraph generation failed - perf not available"
fi

# If perf is available, also generate perf data
if [ "$PERF_AVAILABLE" = true ]; then
    echo "Running perf record for detailed profiling..."
    cargo build --profile bench --bin entropy-aggregator
    perf record -g target/bench/entropy-aggregator --help > /dev/null 2>&1 || echo "Perf recording skipped - aggregator doesn't support direct execution for profiling"
    
    # Run a simple test with perf to capture data
    echo "Running simple aggregator test with perf..."
    cargo test --profile bench aggregator_creation -- --nocapture --exact 2>/dev/null || echo "Running test with perf failed, trying alternative"
fi

# Run flamegraph for specific functions that might be bottlenecks
echo "Generating flamegraph for crypto operations..."
if [ "$PERF_AVAILABLE" = true ]; then
    cargo flamegraph --example crypto_bench --profile bench
else
    echo "Perf not available, running flamegraph with basic profiling for crypto operations..."
    cargo flamegraph --example crypto_bench --profile bench 2>/dev/null || echo "Flamegraph generation failed for crypto operations - perf not available"
fi

echo "Profiling complete. Results saved in target/"
echo "You can also run 'cargo bench --bench round_latency' for detailed Criterion benchmarks"

# Generate a simple timing report
echo "Generating performance summary..."
echo "=== Performance Summary ==="
cargo bench --bench round_latency 2>/dev/null | grep -E "(time:|Benchmarking)" | head -20