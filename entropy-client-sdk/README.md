# @alea/entropy-client-sdk

TypeScript client SDK for Project Entropy - a decentralized randomness beacon system.

## Installation

```bash
npm install @alea/entropy-client-sdk
```

## Usage

```typescript
import EntropyClientImpl from '@alea/entropy-client-sdk';

// Initialize the client with beacon address and provider
const client = new EntropyClientImpl('beacon-contract-address', provider);

// Request randomness
const requestId = await client.requestRandomness((result) => {
  console.log('Randomness result:', result);
  console.log('Round ID:', result.roundId);
  console.log('Random number:', result.randomNumber);
  console.log('Nonce:', result.nonce);
  console.log('Attestation:', result.attestation);
});

console.log('Request ID:', requestId);
```

## API

### `EntropyClientImpl`

#### Constructor
```typescript
new EntropyClientImpl(beaconAddress: string, provider: any)
```

- `beaconAddress`: Address of the entropy beacon contract
- `provider`: Blockchain provider instance

#### Methods

##### `requestRandomness`

```typescript
async requestRandomness(callback: (result: RandomnessResult) => void): Promise<string>
```

Requests randomness from the entropy beacon.

- `callback`: Function called when randomness is available
- Returns: A request ID string

### Types

#### `RandomnessResult`

```typescript
{
  roundId: number;      // The round ID for this randomness
  randomNumber: string; // The random number in hex format
  nonce: string;        // The nonce in hex format
  attestation: string;  // The attestation in hex format
}
```

## Development

```bash
# Install dependencies
npm install

# Build the SDK
npm run build

# Run tests
npm test

# Run tests in watch mode
npm run test:watch
```

## License

MIT