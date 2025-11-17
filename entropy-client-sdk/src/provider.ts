/**
 * Linera Provider abstraction for Entropy Client SDK
 * Provides interface to interact with Linera blockchain
 */

export interface LineraProvider {
 /**
   * Submit a transaction to the blockchain
   * @param operation - The operation to submit
   * @param chainId - The target chain ID
   * @returns Transaction result
   */
  submitTransaction: (operation: any, chainId: string) => Promise<any>;

  /**
   * Query the blockchain state
   * @param query - The query to execute
   * @param chainId - The target chain ID
   * @returns Query result
   */
  query: (query: any, chainId: string) => Promise<any>;

  /**
   * Subscribe to events from the blockchain
   * @param eventType - The type of event to subscribe to
   * @param chainId - The target chain ID
   * @param callback - Function to handle incoming events
   * @returns Subscription ID
   */
  subscribeToEvents: (
    eventType: string,
    chainId: string,
    callback: (event: any) => void
  ) => Promise<string>;

  /**
   * Unsubscribe from events
   * @param subscriptionId - The subscription ID to unsubscribe
   */
  unsubscribeFromEvents: (subscriptionId: string) => Promise<void>;

  /**
   * Get current chain ID
   */
  getChainId: () => Promise<string>;

  /**
   * Get account information
   */
  getAccount: () => Promise<any>;
}

/**
 * Mock implementation of LineraProvider for testing purposes
 */
export class MockLineraProvider implements LineraProvider {
  private subscriptions: Map<string, (event: any) => void> = new Map();

  async submitTransaction(operation: any, chainId: string): Promise<any> {
    console.log(`Mock submitting transaction to chain ${chainId}:`, operation);
    // Simulate a successful transaction
    return {
      success: true,
      transactionId: `tx_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
    };
  }

  async query(query: any, chainId: string): Promise<any> {
    console.log(`Mock querying chain ${chainId}:`, query);
    // Simulate returning a query result
    return {
      result: 'mock_query_result',
    };
  }

  async subscribeToEvents(
    eventType: string,
    chainId: string,
    callback: (event: any) => void
  ): Promise<string> {
    const subscriptionId = `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    this.subscriptions.set(subscriptionId, callback);
    console.log(`Mock subscribed to ${eventType} events on chain ${chainId} with ID ${subscriptionId}`);
    return subscriptionId;
  }

  async unsubscribeFromEvents(subscriptionId: string): Promise<void> {
    this.subscriptions.delete(subscriptionId);
    console.log(`Mock unsubscribed from events with ID ${subscriptionId}`);
  }

  async getChainId(): Promise<string> {
    return 'test-chain-id';
  }

  async getAccount(): Promise<any> {
    return {
      address: 'test-account-address',
      publicKey: 'test-public-key',
    };
  }

  // Method to simulate emitting events for testing
  public emitEvent(subscriptionId: string, event: any): void {
    const callback = this.subscriptions.get(subscriptionId);
    if (callback) {
      callback(event);
    }
 }
}