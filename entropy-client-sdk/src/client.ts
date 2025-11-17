import { EntropyClient, RandomnessResult, BeaconOperation, BeaconQuery, BeaconEvent } from './types';
import { LineraProvider } from './provider';
import { EventManager, convertBeaconEventToResult } from './events';
import { NetworkError, TimeoutError, RequestError, ConnectionError, ReconnectionError } from './errors';
import { ReconnectionManager, ReconnectionOptions } from './reconnection';

export interface EntropyClientConfig {
  beaconAddress: string;
  provider: LineraProvider;
  timeout?: number; // Request timeout in milliseconds
  reconnectionOptions?: ReconnectionOptions;
}

/**
 * EntropyClient class implementation
 * Provides interface to interact with Project Entropy randomness beacon
 */
export class EntropyClientImpl implements EntropyClient {
  private beaconAddress: string;
  private provider: LineraProvider;
  private eventManager: EventManager;
  private randomnessCallbacks: Map<string, (result: RandomnessResult) => void> = new Map();
 private subscriptionId: string | null = null;
  private timeout: number;
  private reconnectionManager: ReconnectionManager;
  private isReconnecting: boolean = false;
 private isInitialized: boolean = false;
 private eventSubscriptionRetryCount: number = 0;
  private maxEventSubscriptionRetries: number = 5;
  private eventHandlers: Map<string, (event: any) => void> = new Map(); // Store event handlers for recovery

  constructor(config: EntropyClientConfig) {
    this.beaconAddress = config.beaconAddress;
    this.provider = config.provider;
    this.eventManager = new EventManager();
    this.timeout = config.timeout ?? 30000; // Default 30 seconds timeout
    this.reconnectionManager = new ReconnectionManager(config.reconnectionOptions);
    // Register the main event handler
    this.eventHandlers.set('RandomnessPublished', this.handleBeaconEvent.bind(this));
  }

  /**
   * Initialize the client and set up event subscriptions
   */
  async initialize(): Promise<void> {
    try {
      // Subscribe to randomness events from the beacon microchain
      this.subscriptionId = await this.subscribeToEventsWithRetry();
      this.isInitialized = true;
    } catch (error) {
      if (error instanceof NetworkError) {
        // Start reconnection process if initialization fails due to network issues
        await this.handleConnectionError(error);
      } else {
        throw error;
      }
    }
  }

  /**
   * Request randomness from the entropy beacon
   * @param callback - Function to handle the randomness result
   * @returns Promise resolving to a request ID string
   */
 async requestRandomness(callback: (result: RandomnessResult) => void): Promise<string> {
    if (!this.isInitialized) {
      throw new ConnectionError('Client not initialized. Call initialize() first.');
    }

    try {
      // Generate a unique request ID
      const requestId = `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      // Register the callback with the request ID
      this.randomnessCallbacks.set(requestId, callback);

      // Create the operation to submit to the beacon
      const operation: BeaconOperation = {
        type: 'RequestRandomness',
        data: {
          requestId,
          timestamp: Date.now(),
        }
      };

      // Submit the transaction to the beacon microchain with timeout
      const result = await this.withTimeout(
        this.provider.submitTransaction(operation, this.beaconAddress),
        this.timeout,
        `RequestRandomness operation timed out after ${this.timeout}ms`
      );
      
      if (!result.success) {
        throw new RequestError(
          `Failed to submit randomness request: ${result.error || 'Unknown error'}`,
          operation,
          result
        );
      }

      console.log(`Randomness request submitted with ID: ${requestId} to beacon at: ${this.beaconAddress}`);
      
      return requestId;
    } catch (error) {
      if (error instanceof TimeoutError || error instanceof NetworkError) {
        await this.handleConnectionError(error);
      }
      console.error('Error requesting randomness:', error);
      throw error;
    }
  }

  /**
   * Get randomness by round ID
   * @param roundId - The round ID to query
   * @returns Promise resolving to RandomnessResult or null
   */
  async getRandomnessByRoundId(roundId: number): Promise<RandomnessResult | null> {
    if (!this.isInitialized) {
      throw new ConnectionError('Client not initialized. Call initialize() first.');
    }

    try {
      const query: BeaconQuery = {
        type: 'GetRandomness',
        data: {
          roundId
        }
      };

      const result = await this.withTimeout(
        this.provider.query(query, this.beaconAddress),
        this.timeout,
        `GetRandomness query timed out after ${this.timeout}ms`
      );
      
      if (result && result.result) {
        // Convert the result to RandomnessResult format
        return result.result as RandomnessResult;
      }
      
      return null;
    } catch (error) {
      if (error instanceof TimeoutError || error instanceof NetworkError) {
        await this.handleConnectionError(error);
      }
      console.error('Error querying randomness by round ID:', error);
      throw error;
    }
  }
 
 /**
  * Clean up resources and unsubscribe from events
  */
 async cleanup(): Promise<void> {
   if (this.subscriptionId) {
     await this.provider.unsubscribeFromEvents(this.subscriptionId);
     this.subscriptionId = null;
   }
   
   this.randomnessCallbacks.clear();
   this.reconnectionManager.reset();
   this.isInitialized = false;
 }

 /**
  * Handle connection errors and initiate reconnection
  */
 private async handleConnectionError(error: Error): Promise<void> {
   if (this.isReconnecting) {
     return; // Already reconnecting, avoid multiple attempts
   }

   this.isReconnecting = true;
   try {
     console.log(`Attempting to reconnect... Attempt #${this.reconnectionManager.getCurrentAttempt() + 1}`);
     await this.reconnectionManager.attemptReconnection(async () => {
       // Reinitialize the client after reconnection
       await this.recoverEventSubscriptions();
       this.isInitialized = true;
       this.isReconnecting = false;
     });
   } catch (reconnectionError) {
     this.isReconnecting = false;
     if (reconnectionError instanceof ReconnectionError) {
       console.error('Max reconnection attempts reached:', reconnectionError);
       throw reconnectionError;
     }
     throw reconnectionError;
   }

 }

 /**
  * Recover event subscriptions after reconnection
  */
private async recoverEventSubscriptions(): Promise<void> {
   if (!this.isInitialized) {
     throw new ConnectionError('Client not initialized. Cannot recover subscriptions.');
   }

   try {
     // Unsubscribe from current events if any
     if (this.subscriptionId) {
       await this.provider.unsubscribeFromEvents(this.subscriptionId);
       this.subscriptionId = null;
     }

     // Re-subscribe to all event types
     this.subscriptionId = await this.subscribeToEventsWithRetry();
     console.log('Event subscriptions recovered successfully');
   } catch (error) {
     console.error('Failed to recover event subscriptions:', error);
     throw new NetworkError('Failed to recover event subscriptions after reconnection', error as Error);
   }
}

 /**
  * Execute a promise with a timeout
 */
 private async withTimeout<T>(promise: Promise<T>, timeoutMs: number, errorMessage: string): Promise<T> {
   return Promise.race([
     promise,
     new Promise<T>((_, reject) => {
       setTimeout(() => {
         reject(new TimeoutError(errorMessage));
       }, timeoutMs);
     })
   ]);
 }

 /**
  * Subscribe to events with retry logic in case of failure
  */
 private async subscribeToEventsWithRetry(): Promise<string> {
   let lastError: Error | null = null;

   for (let attempt = 0; attempt < this.maxEventSubscriptionRetries; attempt++) {
     try {
       const subscriptionId = await this.provider.subscribeToEvents(
         'RandomnessPublished',
         this.beaconAddress,
         this.handleBeaconEvent.bind(this)
       );
       
       this.eventSubscriptionRetryCount = 0; // Reset retry count on success
       return subscriptionId;
     } catch (caughtError: any) {
       lastError = caughtError as Error;
       console.warn(`Event subscription attempt ${attempt + 1} failed:`, caughtError);
       
       // Wait before retrying (exponential backoff)
       if (attempt < this.maxEventSubscriptionRetries - 1) {
         const delay = Math.min(1000 * Math.pow(2, attempt), 30000); // Max 30 seconds
         await new Promise(resolve => setTimeout(resolve, delay));
       }
     }
   }

   // If all retries failed, throw the last error
   if (lastError) {
     throw new NetworkError('Failed to subscribe to events after multiple attempts', lastError);
   } else {
     throw new NetworkError('Failed to subscribe to events after multiple attempts');
   }
 }

 /**
  * Handle incoming beacon events
  * @param event - The beacon event received
  */
 private handleBeaconEvent(event: any): void {
   try {
     // Convert the beacon event to RandomnessResult
     const randomnessResult = convertBeaconEventToResult(event as any);
     
     // Call all registered callbacks with the result
     this.randomnessCallbacks.forEach((callback, id) => {
       try {
         callback(randomnessResult);
       } catch (error) {
         console.error(`Error in randomness callback ${id}:`, error);
       }
     });
   } catch (error) {
     console.error('Error processing beacon event:', error);
   }
 }
}


export default EntropyClientImpl;