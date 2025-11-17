/**
 * Simple Request Example
 *
 * This example demonstrates how to make a basic randomness request using the Entropy Client SDK.
 */

import { EntropyClientImpl } from '../src/client';
import { MockLineraProvider } from '../src/provider';
import { type RandomnessResult } from '../src/types';

async function simpleRequestExample() {
  console.log('Starting simple randomness request example...\n');

  // Create a mock provider for testing
  const provider = new MockLineraProvider();

  // Initialize the client with beacon address and provider
  const client = new EntropyClientImpl({
    beaconAddress: 'test-beacon-contract-address',
    provider: provider,
    timeout: 30000, // 30 second timeout
  });

  try {
    // Initialize the client
    console.log('Initializing client...');
    await client.initialize();
    console.log('Client initialized successfully!\n');

    // Request randomness
    console.log('Requesting randomness...');
    const requestId = await client.requestRandomness((result: RandomnessResult) => {
      console.log('\n--- Randomness Result Received ---');
      console.log('Round ID:', result.roundId);
      console.log('Random Number:', result.randomNumber);
      console.log('Nonce:', result.nonce);
      console.log('Attestation:', result.attestation);
      console.log('----------------------------------\n');
    });

    console.log('Randomness requested with ID:', requestId);

    // Wait a bit to see the result
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Try to get randomness by round ID (this will return null in mock provider)
    console.log('\nTrying to get randomness by round ID...');
    const roundResult = await client.getRandomnessByRoundId(1);
    if (roundResult) {
      console.log('Retrieved randomness by round ID:', roundResult);
    } else {
      console.log('No randomness found for round ID 1 (expected with mock provider)');
    }

  } catch (error) {
    console.error('Error occurred:', error);
  } finally {
    // Clean up resources
    console.log('\nCleaning up resources...');
    await client.cleanup();
    console.log('Resources cleaned up successfully!');
  }
}

// Run the example
simpleRequestExample().catch(console.error);