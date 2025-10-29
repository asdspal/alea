/**
 * Type definitions for Project Entropy client SDK
 * Mirroring Rust types from entropy-types crate
 */

export interface CommitmentPayload {
  roundId: number;
  commitment: string; // hex string representing [u8; 32]
  signature: string; // hex string
}

export interface RevealPayload {
  roundId: number;
  secret: string; // hex string representing [u8; 32]
}

export interface StartCommitmentMsg {
  roundId: number;
 committee: string[]; // NodeId[] where NodeId is string
}

export interface StartRevealMsg {
  roundId: number;
}

export interface AttestationReport {
  report: string; // hex string
  signature: string; // hex string
  signingCert: string; // hex string
  teeType: string;
}

export interface CommitmentMsg {
  roundId: number;
 payload: CommitmentPayload;
  nodeId: string; // NodeId
  timestamp: number;
}

export interface RevealMsg {
  roundId: number;
  payload: RevealPayload;
  nodeId: string; // NodeId
  timestamp: number;
}

export interface EntropyRequest {
  requestId: string;
  clientId: string;
  timestamp: number;
 nonce: string; // hex string representing [u8; 32]
}

export interface EntropyResponse {
  requestId: string;
  roundId: number;
  entropy: string; // hex string representing [u8; 32]
  attestation: AttestationReport;
 timestamp: number;
}

export interface HeartbeatMsg {
  nodeId: string; // NodeId
  timestamp: number;
  status: string;
}

export interface ErrorMessage {
  errorCode: number;
  errorMessage: string;
  timestamp: number;
}

export interface RoundCompletionMsg {
  roundId: number;
  entropy: string; // hex string representing [u8; 32]
  participants: string[]; // NodeId[]
  timestamp: number;
}

// Existing types
export interface RandomnessResult {
  roundId: number;
 randomNumber: string; // hex
  nonce: string; // hex
 attestation: string; // hex
}

export interface EntropyClient {
  requestRandomness(callback: (result: RandomnessResult) => void): Promise<string>;
}

// Protocol version constant
export const PROTOCOL_VERSION = 1;