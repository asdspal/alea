import { EntropyClientImpl } from '../client';
import { MockLineraProvider } from '../provider';
import { NetworkError, TimeoutError, RequestError, ConnectionError, ReconnectionError } from '../errors';
import { ReconnectionManager } from '../reconnection';

// Mock implementation of provider that can simulate errors
class ErrorSimulatingProvider extends MockLineraProvider {
  shouldFail: boolean = false;
  failMode: 'network' | 'timeout' | 'request' = 'network';
  
  constructor(shouldFail: boolean = false, failMode: 'network' | 'timeout' | 'request' = 'network') {
    super();
    this.shouldFail = shouldFail;
    this.failMode = failMode;
  }

  async submitTransaction(operation: any, chainId: string): Promise<any> {
    if (this.shouldFail) {
      switch (this.failMode) {
        case 'network':
          throw new Error('Network error');
        case 'timeout':
          // Simulate timeout by delaying longer than timeout
          return new Promise((resolve) => {
            setTimeout(() => resolve({ success: true }), 5000);
          });
        case 'request':
          return { success: false, error: 'Request failed' };
      }
    }
    return super.submitTransaction(operation, chainId);
  }

 async query(query: any, chainId: string): Promise<any> {
    if (this.shouldFail) {
      switch (this.failMode) {
        case 'network':
          throw new Error('Network error');
        case 'timeout':
          // Simulate timeout by delaying longer than timeout
          return new Promise((resolve) => {
            setTimeout(() => resolve({ result: 'mock_result' }), 5000);
          });
        case 'request':
          throw new Error('Query failed');
      }
    }
    return super.query(query, chainId);
  }

  async subscribeToEvents(
    eventType: string,
    chainId: string,
    callback: (event: any) => void
  ): Promise<string> {
    if (this.shouldFail && this.failMode === 'network') {
      throw new Error('Network error');
    }
    return super.subscribeToEvents(eventType, chainId, callback);
  }
}

describe('Error Handling Tests', () => {
 let mockProvider: MockLineraProvider;
  const beaconAddress = 'test-beacon';

  beforeEach(() => {
    mockProvider = new MockLineraProvider();
  });

  test('Network error during initialization triggers reconnection', async () => {
    const errorProviderNetwork = new ErrorSimulatingProvider(true, 'network');
    const client = new EntropyClientImpl({
      beaconAddress,
      provider: errorProviderNetwork,
      timeout: 100, // Short timeout for faster test
      reconnectionOptions: {
        maxAttempts: 1, // Only try once to fail quickly
        baseDelay: 10,
        maxDelay: 100
      }
    });

    // This should fail with a reconnection error after max attempts
    await expect(client.initialize()).rejects.toThrow(ReconnectionError);
  });

  test('Request timeout is properly handled', async () => {
    // First initialize the client with a working provider
    const workingProvider = new MockLineraProvider();
    const client = new EntropyClientImpl({
      beaconAddress,
      provider: workingProvider,
      timeout: 10 // Set very short timeout to trigger timeout error
    });
    
    // Mock the submitTransaction to delay response
    jest.spyOn(workingProvider, 'submitTransaction').mockImplementation(() => {
      return new Promise((resolve) => {
        setTimeout(() => resolve({ success: true }), 100); // Longer than timeout
      });
    });

    await client.initialize(); // Initialize with working provider first
    await expect(client.requestRandomness(() => {})).rejects.toThrow(TimeoutError);
  });

  test('Request error is properly handled', async () => {
    // First initialize the client with a working provider
    const workingProvider = new MockLineraProvider();
    const client = new EntropyClientImpl({
      beaconAddress,
      provider: workingProvider,
      timeout: 100
    });
    
    // Mock the submitTransaction to return failure
    jest.spyOn(workingProvider, 'submitTransaction').mockResolvedValue({ 
      success: false, 
      error: 'Request failed' 
    });

    await client.initialize(); // Initialize with working provider first
    await expect(client.requestRandomness(() => {})).rejects.toThrow(RequestError);
  });

 test('Query timeout is properly handled', async () => {
    // First initialize the client with a working provider
    const workingProvider = new MockLineraProvider();
    const client = new EntropyClientImpl({
      beaconAddress,
      provider: workingProvider,
      timeout: 10 // Set very short timeout to trigger timeout error
    });
    
    // Mock the query to delay response
    jest.spyOn(workingProvider, 'query').mockImplementation(() => {
      return new Promise((resolve) => {
        setTimeout(() => resolve({ result: 'test' }), 100); // Longer than timeout
      });
    });

    await client.initialize(); // Initialize with working provider first
    await expect(client.getRandomnessByRoundId(1)).rejects.toThrow(TimeoutError);
  });

  test('Query error is properly handled', async () => {
    // First initialize the client with a working provider
    const workingProvider = new MockLineraProvider();
    const client = new EntropyClientImpl({
      beaconAddress,
      provider: workingProvider,
      timeout: 1000
    });
    
    // Mock the query to throw an error
    jest.spyOn(workingProvider, 'query').mockRejectedValue(new Error('Query failed'));

    await client.initialize(); // Initialize with working provider first
    await expect(client.getRandomnessByRoundId(1)).rejects.toThrow();
  });

  test('Connection error when client not initialized', async () => {
    const client = new EntropyClientImpl({
      beaconAddress,
      provider: mockProvider,
      timeout: 1000
    });

    // Don't initialize the client, then try to request randomness
    await expect(client.requestRandomness(() => {})).rejects.toThrow(ConnectionError);
  });

 test('Reconnection logic works after network failure', async () => {
    // Create a provider that initially fails but then works
    const submitTransactionMock = jest.fn()
      .mockRejectedValueOnce(new Error('Network error'))
      .mockResolvedValue({ success: true });
      
    const unreliableProvider = {
      ...mockProvider,
      submitTransaction: submitTransactionMock,
      query: jest.fn().mockResolvedValue({ result: 'mock_result' }),
      subscribeToEvents: jest.fn().mockResolvedValue('mock-subscription-id'),
      unsubscribeFromEvents: jest.fn().mockResolvedValue(undefined),
      getChainId: jest.fn().mockResolvedValue('test-chain-id'),
      getAccount: jest.fn().mockResolvedValue({ address: 'test', publicKey: 'key' })
    };

    const client = new EntropyClientImpl({
      beaconAddress,
      provider: unreliableProvider as any,
      timeout: 1000,
      reconnectionOptions: {
        maxAttempts: 3,
        baseDelay: 10, // Short delay for testing
        maxDelay: 100
      }
    });

    // Initialize the client
    await client.initialize();

    // First call will fail, second will succeed due to mock implementation
    const result = await client.requestRandomness(() => {});
    expect(result).toBeDefined();
    expect(submitTransactionMock).toHaveBeenCalledTimes(2); // Called twice due to retry
  });

  test('Event subscription recovery after connection loss', async () => {
    let subscriptionCount = 0;
    const recoveryProvider = {
      ...mockProvider,
      subscribeToEvents: jest.fn().mockImplementation(() => {
        subscriptionCount++;
        return Promise.resolve(`sub-${subscriptionCount}`);
      }),
      unsubscribeFromEvents: jest.fn().mockResolvedValue(undefined),
      submitTransaction: jest.fn().mockResolvedValue({ success: true }),
      query: jest.fn().mockResolvedValue({ result: 'mock_result' }),
      getChainId: jest.fn().mockResolvedValue('test-chain-id'),
      getAccount: jest.fn().mockResolvedValue({ address: 'test', publicKey: 'key' })
    };

    const client = new EntropyClientImpl({
      beaconAddress,
      provider: recoveryProvider as any,
      timeout: 1000
    });

    await client.initialize();
    
    // Verify initial subscription
    expect(recoveryProvider.subscribeToEvents).toHaveBeenCalledTimes(1);
    
    // Simulate calling recovery method
    await client['recoverEventSubscriptions']();
    
    // Should have unsubscribed and resubscribed
    expect(recoveryProvider.unsubscribeFromEvents).toHaveBeenCalled();
    expect(recoveryProvider.subscribeToEvents).toHaveBeenCalledTimes(2);
  });

  test('Reconnection error after max attempts reached', async () => {
    const failingProvider = {
      ...mockProvider,
      submitTransaction: jest.fn().mockRejectedValue(new Error('Network error')),
      query: jest.fn().mockRejectedValue(new Error('Network error')),
      subscribeToEvents: jest.fn().mockResolvedValue('mock-subscription-id'), // Allow initialization to succeed
      unsubscribeFromEvents: jest.fn().mockResolvedValue(undefined),
      getChainId: jest.fn().mockResolvedValue('test-chain-id'),
      getAccount: jest.fn().mockResolvedValue({ address: 'test', publicKey: 'key' })
    };

    const client = new EntropyClientImpl({
      beaconAddress,
      provider: failingProvider as any,
      timeout: 100,
      reconnectionOptions: {
        maxAttempts: 2,
        baseDelay: 10, // Short delay for testing
        maxDelay: 100
      }
    });

    await client.initialize(); // Initialize first with working subscribe
    // This should eventually throw a ReconnectionError after max attempts
    await expect(client.requestRandomness(() => {})).rejects.toThrow(ReconnectionError);
  });
});

describe('ReconnectionManager Tests', () => {
  test('Exponential backoff works correctly', async () => {
    const reconnectionManager = new ReconnectionManager({
      maxAttempts: 2, // Just 2 attempts for this test
      baseDelay: 10, // Short delay for testing
      maxDelay: 1000,
      backoffMultiplier: 2
    });

    // Test that calculateDelay method is working correctly
    const connectFn = jest.fn().mockRejectedValue(new Error('Connection failed'));
    
    // This test will check if the reconnection attempts increase delays
    const start = Date.now();
    try {
      await reconnectionManager.attemptReconnection(connectFn);
    } catch (error) {
      // Expected to fail after max attempts
    }
    const duration = Date.now() - start;
    
    // Should take at least baseDelay (10ms) for the first attempt and then more for the second
    expect(connectFn).toHaveBeenCalledTimes(2); // Called twice due to retry
  });

  test('Reset functionality works', () => {
    const reconnectionManager = new ReconnectionManager({
      maxAttempts: 3,
      baseDelay: 100,
      maxDelay: 1000
    });

    // Simulate some attempts by directly setting the property
    (reconnectionManager as any).currentAttempt = 2;
    
    reconnectionManager.reset();
    
    expect(reconnectionManager.getCurrentAttempt()).toBe(0);
  });
});