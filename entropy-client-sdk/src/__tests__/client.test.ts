import EntropyClientImpl from '../client';
import { RandomnessResult } from '../types';
import { MockLineraProvider } from '../provider';
import { BeaconEvent } from '../events';

describe('EntropyClientImpl', () => {
  let client: EntropyClientImpl;
 let mockProvider: MockLineraProvider;
  const mockBeaconAddress = 'test-beacon-address';

 beforeEach(async () => {
    mockProvider = new MockLineraProvider();
    client = new EntropyClientImpl({
      beaconAddress: mockBeaconAddress,
      provider: mockProvider
    });
    await client.initialize();
  });

  afterEach(async () => {
    await client.cleanup();
  });

  it('should be instantiated with beacon address and provider', () => {
    expect(client).toBeInstanceOf(EntropyClientImpl);
  });

  it('should return a request ID when requesting randomness', async () => {
    const callback = jest.fn();
    const requestId = await client.requestRandomness(callback);
    
    expect(requestId).toBeDefined();
    expect(typeof requestId).toBe('string');
    expect(requestId.startsWith('req_')).toBe(true);
  });

  it('should call the callback with a RandomnessResult when randomness is received', (done) => {
    const callback = (result: RandomnessResult) => {
      expect(result).toBeDefined();
      expect(typeof result.roundId).toBe('number');
      expect(typeof result.randomNumber).toBe('string');
      expect(typeof result.nonce).toBe('string');
      expect(typeof result.attestation).toBe('string');
      expect(result.randomNumber.startsWith('0x')).toBe(true);
      expect(result.nonce.startsWith('0x')).toBe(true);
      expect(result.attestation.startsWith('0x')).toBe(true);
      done();
    };

    // Call requestRandomness
    client.requestRandomness(callback);

    // Simulate receiving a beacon event
    const mockBeaconEvent: BeaconEvent = {
      type: 'RandomnessPublished',
      event: {
        roundId: 1,
        random_number: new Uint8Array(32).fill(1),
        nonce: new Uint8Array(16).fill(2),
        attestation: new Uint8Array([3, 4, 5])
      }
    };

    // For testing purposes, directly call the handleBeaconEvent method
    // In a real scenario, this would be triggered by the provider subscription
    (client as any).handleBeaconEvent(mockBeaconEvent);
  });

  it('should handle errors in randomness callback gracefully', async () => {
    const failingCallback = () => {
      throw new Error('Test error in callback');
    };
    
    const successCallback = (result: RandomnessResult) => {
      expect(result).toBeDefined();
    };
    
    // Register both callbacks
    await client.requestRandomness(failingCallback);
    await client.requestRandomness(successCallback);
    
    // Simulate receiving a beacon event
    const mockBeaconEvent: BeaconEvent = {
      type: 'RandomnessPublished',
      event: {
        roundId: 1,
        random_number: new Uint8Array(32).fill(1),
        nonce: new Uint8Array(16).fill(2),
        attestation: new Uint8Array([3, 4, 5])
      }
    };

    // This should handle the error in the first callback but still execute the second
    expect(() => {
      (client as any).handleBeaconEvent(mockBeaconEvent);
    }).not.toThrow();
  });

  it('should query randomness by round ID', async () => {
    const result = await client.getRandomnessByRoundId(1);
    // This will depend on the mock provider implementation
    expect(result).toBeDefined();
  });

  it('should clean up resources properly', async () => {
    // This test verifies that cleanup doesn't throw errors
    await expect(client.cleanup()).resolves.not.toThrow();
  });
});