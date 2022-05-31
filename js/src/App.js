import './App.css';
import { getVaultData, getStakeData, getAndCreateInstruction, Numberu64 } from "./staking.ts";
import { Connection, PublicKey } from "@solana/web3.js";

export default function App() {
	let connection = new Connection("https://api.devnet.solana.com");
	let pk = new PublicKey("a7vp4mvuhSkf2FTq2TLSrzwaogVKdd2Gf6BiXXLZZk9");
	//getVaultData(connection);
	//getStakeData(connection, pk);
	let n64 = new Numberu64(500);
	getAndCreateInstruction(n64, pk, connection);

  return (
    <div className="App">
      <header className="App-header">
      </header>
    </div>
  );
}
