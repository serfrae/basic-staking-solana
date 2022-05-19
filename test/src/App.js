import './App.css';
import { getVaultData, getStakeData } from "./staking.ts";
import { Connection, PublicKey } from "@solana/web3.js";

export default function App() {
	let connection = new Connection("https://api.devnet.solana.com");
	let pk = new PublicKey("FvSz7PMcxzySrgRehc7EYdiSjfVtshKg9vaeHN1EkB8v");
	getVaultData(connection);
	getStakeData(connection, pk);


  return (
    <div className="App">
      <header className="App-header">
      </header>
    </div>
  );
}
