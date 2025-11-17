# Entropy Client SDK Documentation

Welcome to the documentation for the @alea/entropy-client-sdk. This SDK provides a TypeScript interface to interact with Project Entropy's decentralized randomness beacon system built on Linera.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](api.md)
- [Integration Guide](integration.md)
- [Examples](#examples)

## Installation

```bash
npm install @alea/entropy-client-sdk
```

## Quick Start

```typescript
import { EntropyClientImpl, MockLineraProvider } from '@alea/entropy-client-sdk';

// Initialize a provider (using mock for demonstration)
const provider = new MockLineraProvider();

// Initialize the client with beacon address and provider
const client = new EntropyClientImpl({
  beaconAddress: 'beacon-contract-address',
  provider: provider,
});

// Initialize the client
await client.initialize();

// Request randomness
const requestId = await client.requestRandomness((result) => {
  console.log('Randomness result:', result);
});

console.log('Request ID:', requestId);

// Later, clean up resources
await client.cleanup();
```

## Examples

You can find example applications in the [examples](./examples/) directory:

- [Simple Request Example](./examples/simple-request.ts) - Basic randomness request
- [Game Integration Example](./examples/game-integration.ts) - Game mechanics with randomness