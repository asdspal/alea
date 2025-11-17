import React, { useState, useEffect } from 'react';
import { useEntropy } from '../hooks/useEntropy';
import { verifyFairness } from '../utils/verification';
import './SlotMachine.css';

interface Symbol {
  id: number;
  name: string;
  icon: string;
}

const symbols: Symbol[] = [
  { id: 1, name: 'Cherry', icon: '🍒' },
  { id: 2, name: 'Lemon', icon: '🍋' },
 { id: 3, name: 'Orange', icon: '🍊' },
  { id: 4, name: 'Plum', icon: '🍑' },
 { id: 5, name: 'Grapes', icon: '🍇' },
  { id: 6, name: 'Seven', icon: '7️⃣' },
  { id: 7, name: 'Diamond', icon: '💎' },
  { id: 8, name: 'Crown', icon: '👑' },
];

const SlotMachine: React.FC = () => {
  const [reels, setReels] = useState<number[]>([0, 0, 0]);
  const [isSpinning, setIsSpinning] = useState(false);
  const [result, setResult] = useState<number | null>(null);
  const [fairnessReport, setFairnessReport] = useState<any>(null);
  const { requestEntropy, entropyResult, isLoading, error } = useEntropy();

  const spin = async () => {
    if (isSpinning) return;
    
    setIsSpinning(true);
    setResult(null);
    setFairnessReport(null);
    
    // Request entropy from the Alea system
    await requestEntropy();
  };

  // Handle entropy result when it arrives
  useEffect(() => {
    if (entropyResult && isSpinning) {
      // Generate random symbols based on entropy
      const newReels = Array(3).fill(0).map(() => {
        // Use entropy value to determine symbol (0-7 for our 8 symbols)
        const entropyValue = parseInt(entropyResult.randomNumber.substring(0, 2), 16);
        return entropyValue % symbols.length;
      });
      
      setReels(newReels);
      
      // Check for winning combination
      if (newReels[0] === newReels[1] && newReels[1] === newReels[2]) {
        setResult(1); // Win
      } else {
        setResult(0); // Lose
      }
      
      // Verify fairness of the result
      verifyFairness(entropyResult.randomNumber, entropyResult.attestation, entropyResult.roundId)
        .then(fairnessResult => {
          setFairnessReport(fairnessResult);
        })
        .catch(err => {
          console.error('Fairness verification error:', err);
        });
      
      setIsSpinning(false);
    }
  }, [entropyResult, isSpinning]);

  return (
    <div className="slot-machine">
      <div className="reels-container">
        {reels.map((symbolIndex, index) => (
          <div key={index} className={`reel ${isSpinning ? 'spinning' : ''}`}>
            <div className="symbol">{symbols[symbolIndex]?.icon || '?'}</div>
          </div>
        ))}
      </div>
      
      <div className="controls">
        <button onClick={spin} disabled={isSpinning || isLoading}>
          {isSpinning ? 'Spinning...' : 'Spin'}
        </button>
      </div>
      
      <div className="status">
        {isLoading && <p>Requesting entropy from Alea...</p>}
        {error && <p className="error">Error: {error}</p>}
        {entropyResult && (
          <div className="entropy-data">
            <h3>Entropy Data</h3>
            <p><strong>Round ID:</strong> {entropyResult.roundId}</p>
            <p><strong>Random Number:</strong> {entropyResult.randomNumber}</p>
            <p><strong>Attestation:</strong> {JSON.stringify(entropyResult.attestation)}</p>
          </div>
        )}
        {fairnessReport && (
          <div className="fairness-report">
            <h3>Fairness Verification</h3>
            <p><strong>Attestation Valid:</strong> {fairnessReport.attestationValid ? '✅' : '❌'}</p>
            <p><strong>Commitment Valid:</strong> {fairnessReport.commitmentValid ? '✅' : '❌'}</p>
            <p><strong>Overall Fair:</strong> {fairnessReport.overallFair ? '✅' : '❌'}</p>
            <details>
              <summary>Verification Details</summary>
              <pre>{JSON.stringify(fairnessReport.details, null, 2)}</pre>
            </details>
          </div>
        )}
        {result !== null && (
          <div className={`result ${result === 1 ? 'win' : 'lose'}`}>
            {result === 1 ? '🎉 You Win! 🎉' : 'Try Again!'}
          </div>
        )}
      </div>
    </div>
  );
};

export default SlotMachine;