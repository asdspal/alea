import { NetworkError, ReconnectionError } from './errors';

export interface ReconnectionOptions {
  maxAttempts?: number;
  baseDelay?: number;
  maxDelay?: number;
  backoffMultiplier?: number;
  jitter?: boolean;
}

export class ReconnectionManager {
  private maxAttempts: number;
  private baseDelay: number;
  private maxDelay: number;
  private backoffMultiplier: number;
  private jitter: boolean;
  private currentAttempt: number = 0;
  private timeoutId: NodeJS.Timeout | null = null;

  constructor(options: ReconnectionOptions = {}) {
    this.maxAttempts = options.maxAttempts ?? 10;
    this.baseDelay = options.baseDelay ?? 1000; // 1 second
    this.maxDelay = options.maxDelay ?? 30000; // 30 seconds
    this.backoffMultiplier = options.backoffMultiplier ?? 2;
    this.jitter = options.jitter ?? true;
  }

  async attemptReconnection(connectFn: () => Promise<void>): Promise<void> {
    if (this.currentAttempt >= this.maxAttempts) {
      throw new ReconnectionError(
        `Failed to reconnect after ${this.maxAttempts} attempts`,
        this.currentAttempt
      );
    }

    const delay = this.calculateDelay();
    this.currentAttempt++;

    return new Promise((resolve, reject) => {
      this.timeoutId = setTimeout(async () => {
        try {
          await connectFn();
          this.currentAttempt = 0; // Reset on successful connection
          resolve();
        } catch (error) {
          if (error instanceof NetworkError) {
            // Try again with exponential backoff
            try {
              await this.attemptReconnection(connectFn);
              resolve();
            } catch (reconnectionError) {
              reject(reconnectionError);
            }
          } else {
            reject(error);
          }
        }
      }, delay);
    });
  }

  private calculateDelay(): number {
    let delay = this.baseDelay * Math.pow(this.backoffMultiplier, this.currentAttempt);

    // Cap the delay at maxDelay
    delay = Math.min(delay, this.maxDelay);

    // Add jitter to prevent thundering herd
    if (this.jitter) {
      const jitter = Math.random() * 0.3; // 30% jitter
      delay *= (1 + jitter);
    }

    return delay;
  }

  reset(): void {
    this.currentAttempt = 0;
    if (this.timeoutId) {
      clearTimeout(this.timeoutId);
      this.timeoutId = null;
    }
  }

  getCurrentAttempt(): number {
    return this.currentAttempt;
  }

  getMaxAttempts(): number {
    return this.maxAttempts;
  }
}