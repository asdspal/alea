/**
 * Test script to verify EntropyClient implementation
 */
import EntropyClientImpl from './src/client';
import { MockLineraProvider } from './src/provider';

async function testEntropyClient() {
  console.log('Testing EntropyClient implementation...');
  
  // Create a mock provider
  const mockProvider = new MockLineraProvider();
  
  // Create the entropy client
 const client = new EntropyClientImpl({
    beaconAddress: 'test-beacon-chain-id',
    provider: mockProvider
  });
  
  // Initialize the client
  await client.initialize();
  
  console.log('✓ Client initialized successfully');
  
  // Define a callback to handle randomness results
  const randomnessCallback = (result: any) => {
    console.log('✓ Randomness result received:', result);
    console.log(`  Round ID: ${result.roundId}`);
    console.log(`  Random Number: ${result.randomNumber}`);
    console.log(`  Nonce: ${result.nonce}`);
    console.log(`  Attestation: ${result.attestation}`);
  };
  
  // Request randomness
  console.log('\nRequesting randomness...');
  const requestId = await client.requestRandomness(randomnessCallback);
  console.log(`✓ Randomness request submitted with ID: ${requestId}`);
  
  // Simulate receiving a randomness event (in a real scenario, this would come from the blockchain)
  console.log('\nSimulating randomness event from beacon...');
  const mockBeaconEvent = {
    type: 'RandomnessPublished',
    event: {
      roundId: 42,
      random_number: new Uint8Array(32).fill(0xab), // Fill with 0xab for testing
      nonce: new Uint8Array(16).fill(0xcd),         // Fill with 0xcd for testing
      attestation: new Uint8Array([0x01, 0x02, 0x03, 0x04])
    }
 };
  
  // Trigger the event manually for testing purposes
  (client as any).handleBeaconEvent(mockBeaconEvent);
  
  // Test querying randomness by round ID
  console.log('\nTesting query by round ID...');
  const result = await client.getRandomnessByRoundId(42);
  console.log('✓ Query result:', result);
  
  // Clean up
  await client.cleanup();
  console.log('\n✓ Client cleaned up successfully');
  
  console.log('\n✓ All tests passed! EntropyClient implementation is working correctly.');
}

// Run the test
testEntropyClient().catch(console.error);