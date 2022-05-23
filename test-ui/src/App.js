import logo from './logo.svg';
import './App.css';
import { getVaultData, getStakeData, createStakeInstruction, Numberu64 } from './staking.ts';
import { Connection, PublicKey } from '@solana/web3.js';

function App() {
	console.log("app init");
	//getStakeData(new Connection('https://api.devnet.solana.com', 'confirmed'), new PublicKey('FvSz7PMcxzySrgRehc7EYdiSjfVtshKg9vaeHN1EkB8v'));
	getVaultData(new Connection('https://api.devnet.solana.com', 'confirmed'));
	console.log("get stake data");

	console.log("creating Stake Instruction");
	let stakeIx = new createStakeInstruction(new Numberu64(500));
	console.log(stakeIx);


  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <p>
          Edit <code>src/App.js</code> and save to reload.
        </p>
        <a
          className="App-link"
          href="https://reactjs.org"
          target="_blank"
          rel="noopener noreferrer"
        >
          Learn React
        </a>
      </header>
    </div>
  );
}

export default App;
