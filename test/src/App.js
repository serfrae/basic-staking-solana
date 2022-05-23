import './App.css';
import { getVaultData, getStakeData, createStakeInstruction, Numberu64 } from "./staking.ts";
import { Connection, PublicKey } from "@solana/web3.js";

export default function App() {
	let connection = new Connection("https://api.devnet.solana.com");
	let pk = new PublicKey("FvSz7PMcxzySrgRehc7EYdiSjfVtshKg9vaeHN1EkB8v");
	getVaultData(connection);
	getStakeData(connection, pk);
	let n64 = new Numberu64(500);
	let sIx = new createStakeInstruction({amount: n64});
	console.log(`stakeIx ${sIx}`);
	sIx.getInstruction();

  return (
    <div className="App">
      <header className="App-header">
      </header>
    </div>
  );
}
