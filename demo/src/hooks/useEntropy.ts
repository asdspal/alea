import { useState, useEffect } from 'react';
import { EntropyClientImpl, RandomnessResult, LineraProvider } from '@alea/entropy-client-sdk';

// Mock implementation of LineraProvider for demo purposes
class MockLineraProvider implements LineraProvider {
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

export const useEntropy = () => {
   // Define a type that matches what the UI expects
   interface FormattedRandomnessResult {
     roundId: number;
     randomNumber: string;
     attestation: {
       report: string;
       signature: string;
       signingCert: string;
       teeType: string;
     };
   }
   
   const [entropyResult, setEntropyResult] = useState<FormattedRandomnessResult | RandomnessResult | null>(null);
   const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

   // Initialize the entropy client with a mock provider for demo purposes
   const provider = new MockLineraProvider();
   const entropyClient = new EntropyClientImpl({
     beaconAddress: 'test-beacon-address',
     provider: provider,
     timeout: 30000,
   });

   const requestEntropy = async () => {
     setIsLoading(true);
     setError(null);

     try {
       // Request randomness from the entropy beacon
       const requestId = await entropyClient.requestRandomness((rawResult: RandomnessResult) => {
         // Format the result to match what the UI expects
         // Note: rawResult.attestation is a string, but UI expects an object with report, signature, etc.
         const formattedResult = {
           roundId: rawResult.roundId,
           randomNumber: rawResult.randomNumber,
           attestation: {
             report: rawResult.attestation || '', // Using the attestation field as the report
             signature: '', // Placeholder - in real implementation would extract from full attestation
             signingCert: '', // Placeholder - in real implementation would extract from full attestation
             teeType: 'SGX' // Placeholder - in real implementation would extract from attestation
           }
         };
         setEntropyResult(formattedResult);
         setIsLoading(false);
       });
       
       console.log('Randomness request submitted with ID:', requestId);
     } catch (err) {
       setError(err instanceof Error ? err.message : 'Unknown error occurred');
       setIsLoading(false);
     }
   };

   // Initialize the entropy client
   useEffect(() => {
     const initClient = async () => {
       try {
         await entropyClient.initialize();
       } catch (err) {
         setError(err instanceof Error ? err.message : 'Failed to initialize entropy client');
       }
     };

     initClient();

     // Cleanup function
     return () => {
       entropyClient.cleanup();
     };
   }, []);

   return {
     requestEntropy,
     entropyResult,
     isLoading,
     error,
   };
};