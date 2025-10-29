# Development Setup Guide

This guide provides step-by-step instructions for setting up the Alea development environment on different operating systems.

## Prerequisites

Before starting, ensure you have the following installed on your system:

- Git (version control)
- Rust 1.70+ and Cargo (package manager)
- A C/C++ compiler (GCC on Linux/macOS, MSVC on Windows)
- OpenSSL development libraries (for HTTPS support)
- Docker (optional, for containerized development)

## Installing Rust

### On Linux/macOS:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### On Windows:
1. Download the Rust installer from https://www.rust-lang.org/tools/install
2. Run `rustup-init.exe` and follow the prompts
3. Restart your terminal

Verify the installation:
```bash
rustc --version
cargo --version
```

## Operating System Specific Setup

### Linux (Ubuntu/Debian)

1. Install build tools and dependencies:
```bash
sudo apt update
sudo apt install build-essential cmake pkg-config libssl-dev git clang
```

2. Install additional Rust targets if needed:
```bash
rustup target add wasm32-unknown-unknown
```

3. Clone the repository:
```bash
git clone https://github.com/akindo/linera.git
cd linera/alea
```

### Linux (CentOS/RHEL/Fedora)

1. Install build tools and dependencies:
```bash
# For CentOS/RHEL
sudo yum groupinstall "Development Tools"
sudo yum install cmake pkgconfig openssl-devel git clang

# For Fedora
sudo dnf groupinstall "Development Tools"
sudo dnf install cmake pkgconfig openssl-devel git clang
```

2. Install additional Rust targets if needed:
```bash
rustup target add wasm32-unknown-unknown
```

3. Clone the repository:
```bash
git clone https://github.com/akindo/linera.git
cd linera/alea
```

### macOS

1. Install Xcode command line tools:
```bash
xcode-select --install
```

2. Install additional dependencies using Homebrew:
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
brew install cmake pkg-config openssl git llvm
```

3. Set environment variables for OpenSSL (if needed):
```bash
export PKG_CONFIG_PATH="$(brew --prefix openssl)/lib/pkgconfig"
```

4. Install additional Rust targets if needed:
```bash
rustup target add wasm32-unknown-unknown
```

5. Clone the repository:
```bash
git clone https://github.com/akindo/linera.git
cd linera/alea
```

### Windows

1. Install Visual Studio Build Tools or Visual Studio Community with C++ development tools
2. Install Git for Windows from https://git-scm.com/download/win
3. Install CMake from https://cmake.org/download/
4. Install OpenSSL for Windows (or use vcpkg to manage dependencies)

5. Open "Developer Command Prompt for VS" or "Developer PowerShell for VS" and clone the repository:
```cmd
git clone https://github.com/akindo/linera.git
cd linera/alea
```

6. Install additional Rust targets if needed:
```cmd
rustup target add wasm32-unknown-unknown
```

## Project Setup

### 1. Clone the Repository
```bash
git clone https://github.com/akindo/linera.git
cd linera/alea
```

### 2. Verify Dependencies
Check that all required tools are available:
```bash
# Check Rust version
rustc --version

# Check Cargo version
cargo --version

# Check if required tools are available
which git
which cmake
which pkg-config  # On Windows, you may need to install this separately
```

### 3. Build the Project

To build all crates in the workspace:
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

# Build types library
cargo build -p types
```

### 4. Run Tests

To run all tests in the workspace:
```bash
cargo test --all
```

To run tests for a specific crate:
```bash
cargo test -p beacon-microchain
cargo test -p entropy-worker
cargo test -p entropy-aggregator
cargo test -p types
```

## Development Workflow

### Code Quality Checks

Before committing your code, run these checks:

```bash
# Check for compilation errors without building
cargo check --all

# Check for warnings and style issues
cargo clippy --all -- -D warnings

# Format code according to project standards
cargo fmt --all
```

### Running Components

To run the off-chain components during development:

```bash
# Run entropy worker (in separate terminal)
cargo run --bin entropy-worker

# Run entropy aggregator (in separate terminal)
cargo run --bin entropy-aggregator
```

### Mock TEE Setup

For local development without requiring actual TEE hardware, you can use the mock TEE implementation:

```bash
# Enable mock TEE mode
export ENTROPY_USE_MOCK_TEE=true

# Run the aggregator with mock TEE
cargo run -p entropy-aggregator
```

When using the mock TEE, you'll see a log message: "Using mock TEE for local development".

### Linera Testnet Integration

The system provides abstractions for connecting to the Linera testnet:

- `LineraProvider` trait in `beacon-microchain/src/linera_integration.rs` provides an interface for Linera operations
- Mock implementation available for local testing
- Environment variable `ENTROPY_USE_MOCK_TEE` controls whether to use mock or real implementations

### Integration Testing

Run the full test suite to verify all components work together:

```bash
ENTROPY_USE_MOCK_TEE=true cargo test --all --nocapture
```

## Development Environment Configuration

### VS Code Setup

If using VS Code, install these extensions for the best development experience:

- rust-analyzer
- CodeLLDB (for debugging)
- crates (for managing dependencies)
- GitLens (for enhanced Git features)

Create a `.vscode/settings.json` file in your project root:

```json
{
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.cargo.runBuildScripts": true
}
```

### Environment Variables

The following environment variables can be used to configure the development environment:

- `ENTROPY_USE_MOCK_TEE` - Set to `true` to use mock TEE instead of real hardware
- `RUST_LOG` - Set logging level (e.g., `info`, `debug`, `trace`)
- `LINERA_ENDPOINT` - Endpoint for connecting to Linera testnet (when not using mock)

### Debugging

To enable detailed logging during development:

```bash
# Set log level to debug
RUST_LOG=debug cargo run --bin entropy-aggregator

# Or for even more detail
RUST_LOG=trace cargo run --bin entropy-aggregator
```

## Troubleshooting

### Common Issues

1. **Compilation errors with OpenSSL**:
   - Linux/macOS: Ensure `libssl-dev` or `openssl-devel` is installed
   - Windows: Ensure OpenSSL is properly installed and in your PATH

2. **Missing build tools**:
   - Linux: Install `build-essential` (Ubuntu) or equivalent
   - macOS: Install Xcode command line tools
   - Windows: Install Visual Studio Build Tools

3. **WASM target missing**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

4. **Permission errors**:
   - Ensure you have write permissions to the project directory
   - On Linux/macOS, you may need to fix ownership: `sudo chown -R $USER:$USER .`

### Getting Help

If you encounter issues not covered here:

1. Check the existing GitHub issues
2. Run `cargo metadata` to verify your setup
3. Ask in the project's communication channels
4. File a new issue with detailed information about your environment and the problem