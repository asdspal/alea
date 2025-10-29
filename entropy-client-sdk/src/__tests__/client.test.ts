import EntropyClientImpl from '../client';
import { RandomnessResult } from '../types';

describe('EntropyClientImpl', () => {
  let client: EntropyClientImpl;
  const mockBeaconAddress = 'test-beacon-address';
 const mockProvider = {};

  beforeEach(() => {
    client = new EntropyClientImpl(mockBeaconAddress, mockProvider);
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

  it('should call the callback with a RandomnessResult', (done) => {
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

    client.requestRandomness(callback);
  });
});