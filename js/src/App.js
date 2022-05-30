import './App.css';
import { getVaultData, getStakeData, getAndCreateInstruction, Numberu64 } from "./staking.ts";
import { Connection, PublicKey } from "@solana/web3.js";

export default function App() {
	let connection = new Connection("https://api.devnet.solana.com");
	let pk = new PublicKey("6G37aGPKyn1JT4wcqdREVV2hQLUEAcZTtToafjCCkdym");
	getVaultData(connection);
	getStakeData(connection, pk);
	let n64 = new Numberu64(500);
	getAndCreateInstruction(n64, pk);

  return (
    <div className="App">
      <header className="App-header">
      </header>
    </div>
  );
}
