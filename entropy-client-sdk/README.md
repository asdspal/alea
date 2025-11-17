# @alea/entropy-client-sdk

TypeScript client SDK for Project Entropy - Secure Randomness Generation for Decentralized Applications

## Overview

The Alea Entropy Client SDK provides a secure, verifiable randomness generation service for decentralized applications. It leverages Trusted Execution Environments (TEEs) and secure multi-party protocols to generate unpredictable, bias-resistant randomness suitable for blockchain applications, gaming, lotteries, and other use cases requiring high-integrity random values.

## Features

- **Secure Randomness**: Unpredictable, bias-resistant randomness generated through TEE-verified processes
- **Verifiable**: All randomness comes with cryptographic proofs and attestation reports
- **Easy Integration**: Simple API for requesting randomness from dApps
- **TypeScript Support**: Full type safety with comprehensive type definitions
- **Event-Driven**: Real-time updates when randomness is generated
- **Cross-Platform**: Works in Node.js and modern browsers

## Installation

```bash
npm install @alea/entropy-client-sdk
```

## Quick Start

```typescript
import { EntropyClient } from '@alea/entropy-client-sdk';

// Initialize the client with your beacon address
const client = new EntropyClient('your-beacon-address');

// Request randomness
const requestId = await client.requestRandomness((result) => {
  console.log('Randomness received:', result.randomNumber);
  console.log('Attestation:', result.attestation);
});

console.log('Request ID:', requestId);
```

## API Reference

### `EntropyClient`

The main class for interacting with the Alea Entropy service.

#### Constructor

```typescript
new EntropyClient(beaconAddress: string, options?: ClientOptions)
```

- `beaconAddress`: The address of the entropy beacon microchain
- `options`: Optional configuration for the client

#### Methods

##### `requestRandomness(callback: (result: RandomnessResult) => void): Promise<string>`

Requests randomness from the Alea Entropy network.

- Returns: A promise that resolves to a request ID
- The callback is called when randomness is available

##### `queryRandomness(roundId: number): Promise<RandomnessResult | null>`

Queries a specific round of randomness by its ID.

## Examples

### Basic Usage

```typescript
import { EntropyClient } from '@alea/entropy-client-sdk';

async function getRandomness() {
  const client = new EntropyClient('your-beacon-address');
  
  try {
    const requestId = await client.requestRandomness((result) => {
      console.log(`Randomness for round ${result.roundId}:`, result.randomNumber);
      console.log('Attestation proof:', result.attestation);
    });
    
    console.log('Requested randomness with ID:', requestId);
  } catch (error) {
    console.error('Error requesting randomness:', error);
  }
}

getRandomness();
```

### Advanced Usage with Error Handling

```typescript
import { EntropyClient } from '@alea/entropy-client-sdk';

async function robustRandomnessRequest() {
  const client = new EntropyClient('your-beacon-address');
  
  try {
    const requestId = await client.requestRandomness(
      (result) => {
        // Handle successful randomness result
        console.log('Randomness result:', result);
      },
      (error) => {
        // Handle errors in callback execution
        console.error('Error in randomness callback:', error);
      }
    );
    
    console.log('Request submitted with ID:', requestId);
  } catch (error) {
    console.error('Failed to request randomness:', error);
  }
}

robustRandomnessRequest();
```

## Development

### Building

```bash
npm run build
```

### Testing

```bash
npm test
```

### Contributing

We welcome contributions to the Alea Entropy Client SDK. Please see our [CONTRIBUTING.md](../../CONTRIBUTING.md) for more details.

## License

MIT License - see [LICENSE](./LICENSE) file for details.