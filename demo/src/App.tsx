import React from 'react';
import SlotMachine from './components/SlotMachine.tsx';
import './App.css';

function App() {
  return (
    <div className="App">
      <header className="App-header">
        <h1>Provably Fair Slot Machine</h1>
        <p>Powered by Alea Entropy - Verifiable Randomness for Web3</p>
      </header>
      <main>
        <SlotMachine />
      </main>
    </div>
  );
}

export default App;