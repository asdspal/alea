/**
 * Custom error types for the Alea Entropy Client SDK
 */

export class AleaClientError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AleaClientError';
    Object.setPrototypeOf(this, AleaClientError.prototype);
  }
}

export class NetworkError extends AleaClientError {
  constructor(message: string, public readonly cause?: Error) {
    super(message);
    this.name = 'NetworkError';
    Object.setPrototypeOf(this, NetworkError.prototype);
  }
}

export class TimeoutError extends AleaClientError {
  constructor(message: string = 'Request timed out') {
    super(message);
    this.name = 'TimeoutError';
    Object.setPrototypeOf(this, TimeoutError.prototype);
  }
}

export class ConnectionError extends AleaClientError {
  constructor(message: string, public readonly code?: string) {
    super(message);
    this.name = 'ConnectionError';
    Object.setPrototypeOf(this, ConnectionError.prototype);
  }
}

export class ReconnectionError extends AleaClientError {
  constructor(message: string, public readonly attempts: number) {
    super(message);
    this.name = 'ReconnectionError';
    Object.setPrototypeOf(this, ReconnectionError.prototype);
  }
}

export class SubscriptionError extends AleaClientError {
  constructor(message: string, public readonly subscriptionId?: string) {
    super(message);
    this.name = 'SubscriptionError';
    Object.setPrototypeOf(this, SubscriptionError.prototype);
  }
}

export class RequestError extends AleaClientError {
  constructor(message: string, public readonly request: any, public readonly response?: any) {
    super(message);
    this.name = 'RequestError';
    Object.setPrototypeOf(this, RequestError.prototype);
  }
}