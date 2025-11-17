/**
 * Game Integration Example
 * 
 * This example demonstrates how to integrate the Entropy Client SDK into a game application
 * to provide verifiable randomness for game mechanics.
 */

import { EntropyClientImpl, MockLineraProvider, type RandomnessResult } from '../src';

class GameWithRandomness {
  private client: EntropyClientImpl;
  private provider: MockLineraProvider;
  private gameStarted: boolean = false;

  constructor(beaconAddress: string) {
    this.provider = new MockLineraProvider();
    this.client = new EntropyClientImpl({
      beaconAddress,
      provider: this.provider,
      timeout: 30000,
    });
  }

  async startGame() {
    console.log('Starting game with entropy integration...\n');
    
    try {
      await this.client.initialize();
      this.gameStarted = true;
      console.log('Game started successfully with entropy client initialized!\n');
    } catch (error) {
      console.error('Failed to start game:', error);
      throw error;
    }
  }

  async rollDice(): Promise<number> {
    if (!this.gameStarted) {
      throw new Error('Game not started. Call startGame() first.');
    }

    console.log('Rolling dice using entropy randomness...');

    return new Promise((resolve, reject) => {
      this.client.requestRandomness((result: RandomnessResult) => {
        try {
          // Convert the randomness to a dice roll (1-6)
          // Take the last few hex digits and convert to a number in range [1, 6]
          const hexValue = result.randomNumber.substring(result.randomNumber.length - 4);
          const numericValue = parseInt(hexValue, 16);
          const diceValue = (numericValue % 6) + 1;
          
          console.log(`Dice roll result: ${diceValue}`);
          console.log(`Randomness details:`, {
            roundId: result.roundId,
            randomNumber: result.randomNumber,
            nonce: result.nonce,
            attestation: result.attestation
          });
          
          resolve(diceValue);
        } catch (error) {
          reject(error);
        }
      }).catch(reject);
    });
  }

  async drawCard(): Promise<{ suit: string; value: string }> {
    if (!this.gameStarted) {
      throw new Error('Game not started. Call startGame() first.');
    }

    console.log('Drawing card using entropy randomness...');

    return new Promise((resolve, reject) => {
      this.client.requestRandomness((result: RandomnessResult) => {
        try {
          // Convert randomness to card selection
          // Use different parts of the random number for suit and value
          const hexValue = result.randomNumber.substring(2); // Remove '0x' prefix
          
          // Extract suit (0-3 for 4 suits)
          const suitHex = hexValue.substring(0, 2);
          const suitIndex = parseInt(suitHex, 16) % 4;
          const suits = ['Hearts', 'Diamonds', 'Clubs', 'Spades'];
          const suit = suits[suitIndex];
          
          // Extract value (0-12 for 13 values: Ace through King)
          const valueHex = hexValue.substring(2, 4);
          const valueIndex = parseInt(valueHex, 16) % 13;
          const values = ['Ace', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'Jack', 'Queen', 'King'];
          const value = values[valueIndex];
          
          console.log(`Card drawn: ${value} of ${suit}`);
          
          resolve({ suit, value });
        } catch (error) {
          reject(error);
        }
      }).catch(reject);
    });
  }

 async endGame() {
    console.log('\nEnding game and cleaning up resources...');
    if (this.gameStarted) {
      await this.client.cleanup();
      this.gameStarted = false;
      console.log('Game ended successfully!');
    }
  }
}

async function gameExample() {
  console.log('=== Game Integration Example ===\n');
  
  const game = new GameWithRandomness('beacon-contract-address');
  
  try {
    // Start the game
    await game.startGame();
    
    // Roll dice multiple times
    console.log('--- Dice Rolling ---');
    for (let i = 0; i < 3; i++) {
      const diceRoll = await game.rollDice();
      console.log(`Roll ${i + 1}: ${diceRoll}\n`);
      // Wait a bit between rolls to see different results
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    
    // Draw cards
    console.log('--- Card Drawing ---');
    for (let i = 0; i < 2; i++) {
      const card = await game.drawCard();
      console.log(`Card ${i + 1}: ${card.value} of ${card.suit}\n`);
      // Wait a bit between draws
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    
  } catch (error) {
    console.error('Game error:', error);
  } finally {
    // End the game
    await game.endGame();
    console.log('\nExample completed!');
  }
}

// Run the example
gameExample().catch(console.error);