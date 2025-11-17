/**
 * Event handling for Entropy Client SDK
 * Manages subscription to beacon microchain events
 */

import { RandomnessResult } from './types';

export interface RandomnessEvent {
  roundId: number;
  random_number: Uint8Array; // [u8; 32] in Rust
  nonce: Uint8Array; // [u8; 16] in Rust
  attestation: Uint8Array; // Vec<u8> in Rust
}

export interface BeaconEvent {
  type: 'RandomnessPublished';
  event: RandomnessEvent;
}

export interface EventSubscription {
  id: string;
  eventType: string;
 callback: (event: BeaconEvent) => void;
}

/**
 * Event manager for handling beacon microchain events
 */
export class EventManager {
  private subscriptions: Map<string, EventSubscription> = new Map();
  private eventListeners: Map<string, Set<(event: any) => void>> = new Map();

  /**
   * Subscribe to a specific event type
   * @param eventType - Type of event to subscribe to
   * @param callback - Function to handle incoming events
   * @returns Subscription ID
   */
  subscribe(eventType: string, callback: (event: BeaconEvent) => void): string {
    const subscriptionId = `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    const subscription: EventSubscription = {
      id: subscriptionId,
      eventType,
      callback
    };

    this.subscriptions.set(subscriptionId, subscription);

    // Add to event listener map
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    this.eventListeners.get(eventType)!.add(callback);

    return subscriptionId;
  }

  /**
   * Unsubscribe from events
   * @param subscriptionId - Subscription ID to remove
   */
  unsubscribe(subscriptionId: string): boolean {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      return false;
    }

    // Remove from event listener map
    const listeners = this.eventListeners.get(subscription.eventType);
    if (listeners) {
      listeners.delete(subscription.callback);
      if (listeners.size === 0) {
        this.eventListeners.delete(subscription.eventType);
      }
    }

    this.subscriptions.delete(subscriptionId);
    return true;
  }

  /**
   * Emit an event to all subscribers
   * @param eventType - Type of event to emit
   * @param event - Event data to emit
   */
  emit(eventType: string, event: BeaconEvent): void {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(event);
        } catch (error) {
          console.error(`Error in event callback for ${eventType}:`, error);
        }
      });
    }
  }

  /**
   * Get all active subscriptions
   */
  getSubscriptions(): EventSubscription[] {
    return Array.from(this.subscriptions.values());
  }
}

/**
 * Convert beacon event to RandomnessResult
 * @param beaconEvent - Raw beacon event
 * @returns RandomnessResult
 */
export function convertBeaconEventToResult(beaconEvent: BeaconEvent): RandomnessResult {
  if (beaconEvent.type !== 'RandomnessPublished' || !beaconEvent.event) {
    throw new Error('Invalid beacon event format');
  }

  const { event } = beaconEvent;
  
  return {
    roundId: event.roundId,
    randomNumber: '0x' + Array.from(event.random_number)
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join(''),
    nonce: '0x' + Array.from(event.nonce)
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join(''),
    attestation: '0x' + Array.from(event.attestation)
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join('')
  };
}