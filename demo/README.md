# Alea Entropy - Provably Fair Slot Machine Demo

This demo showcases the Alea Entropy system for provably fair gaming applications. It demonstrates how to integrate with the Alea Entropy client SDK to generate verifiable randomness for a slot machine game.

## Features

- Provably fair slot machine using Alea Entropy
- Real-time randomness generation with TEE attestation
- Fairness verification of entropy generation
- Transparent randomness display with attestation data

## Architecture

The demo consists of:

1. **Slot Machine UI** - A React-based slot machine interface
2. **Entropy Hook** - Integration with the Alea Entropy client SDK
3. **Verification Utilities** - Attestation and fairness verification logic
4. **Client SDK** - Interface to the Alea Entropy system

## Installation

1. Ensure you have Node.js (v14 or higher) installed
2. Navigate to the demo directory: `cd alea/demo`
3. Install dependencies: `npm install`

## Usage

To run the development server:

```bash
npm run dev
```

To build the application:

```bash
npm run build
```

To run the built application:

```bash
npm run preview
```

## How It Works

1. When you click "Spin", the demo requests entropy from the Alea Entropy system
2. The entropy is generated in a Trusted Execution Environment (TEE) with cryptographic attestation
3. The attestation proves that the randomness was generated fairly
4. The entropy is used to determine the slot machine symbols
5. Fairness verification is performed to ensure the process was legitimate

## Fairness Verification

The demo includes verification of:
- Attestation validity (proving the entropy came from a TEE)
- Commitment verification (ensuring the entropy wasn't manipulated)
- Overall fairness of the process

## Integration with Alea Entropy

The demo integrates with the Alea Entropy system through:

- `useEntropy` hook for requesting randomness
- Attestation verification utilities
- Real-time display of entropy and attestation data

## Security

The system ensures security through:
- Trusted Execution Environment (TEE) for entropy generation
- Cryptographic attestation of the TEE
- Verifiable commitment-reveal scheme
- Client-side verification of all operations