import {
	AccountInfo,
	Connection,
	clusterApiUrl,
	Keypair,
	PublicKey,
	SYSVAR_CLOCK_PUBKEY,
	Transaction,
	TransactionInstruction,
} from "@solana/web3.js";
import BN from "bn.js";
import { deserializeUnchecked, Schema, serialize } from "borsh";
import { Buffer } from 'buffer';

export const VAULT_SEED = Buffer.from("___vault");

export const ASSOCIATED_TOKEN_PROGRAM_ID: PublicKey = new PublicKey(
	"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

export const SCY_STAKING_PROGRAM_ID: PublicKey = new PublicKey(
	"titFt4THm4Yv6XY8BDje4vn3eGtCtZkhCXqQhYyDp7W"
);

export const SCY_MINT: PublicKey = new PublicKey(
	"SCYVn1w92poF5VaLf2myVBbTvBf1M8MLqJwpS64Gb9b"
);
export const SCY_STAKING_VAULT_INFO: PublicKey = new PublicKey(
	"2Yf1SfEZwzT342KUT3UCZgKK5kXnLbfUGM6c7DzQFHsn"
);

export const SCY_STAKING_VAULT_TOKEN_ADDRESS: PublicKey = new PublicKey(
	"6jgtCz9sgtXoKaJEntnfcLDqJNQTKdWBKHdfM4dpzvKd"
);

export const TOKEN_PROGRAM_ID: PublicKey = new PublicKey(
	"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);
export const RENT_SYSVAR: PublicKey = new PublicKey(
	"SysvarRent111111111111111111111111111111111"
);

export const SYSTEM_PROGRAM_ID: PublicKey = new PublicKey(
	"11111111111111111111111111111111"
);

//-----------------------u64 Typscript------------------------------------------
export class Numberu64 extends BN {
	/**
	 * Convert to Buffer representation
	 */
	toBuffer(): Buffer {
		const a = super.toArray().reverse();
		const b = Buffer.from(a);
		if (b.length === 8) {
			return b;
		}

		const zeroPad = Buffer.alloc(8);
		b.copy(zeroPad);
		return zeroPad;
	}

	/**
	 * Construct a Numberu64 from Buffer representation
	 */
	static fromBuffer(buffer: Buffer): any {
		return new BN(
			[...buffer]
			.reverse()
			.map((i) => `00${i.toString(16)}`.slice(-2))
			.join(""),
			16
		);
	}
}

//------------------------Structs----------------------------------------------
export type StakeData = {
	timestamp: Numberu64,
	staker: PublicKey,
	mint: PublicKey,
	active: boolean,
	withdraw: Numberu64,
	harvested: Numberu64,
	stakedAmount: Numberu64,
	maxReward: Numberu64,
}

export type VaultData = {
	mint: PublicKey,
	minPeriod: Numberu64,
	rewardPeriod: Numberu64,
	rate: Numberu64,
	earlyWithdrawalFee: Numberu64,
	totalObligations: Numberu64,
	totalStaked: Numberu64,
}

class StakeSchema {
	timestamp: Numberu64
	staker: PublicKey
	mint: PublicKey
	active: boolean
	withdraw: Numberu64
	harvested: Numberu64
	stakedAmount: Numberu64
	maxReward: Numberu64

	constructor(fields?: {
		timestamp: Numberu64,
		staker: PublicKey,
		mint: PublicKey,
		active: boolean,
		withdraw: Numberu64,
		harvested: Numberu64,
		stakedAmount: Numberu64,
		maxReward: Numberu64,
	}) {
		this.timestamp = fields.timestamp;
		this.staker = fields.staker;
		this.mint = fields.mint;
		this.active = fields.active;
		this.withdraw = fields.withdraw;
		this.harvested = fields.harvested;
		this.stakedAmount = fields.stakedAmount;
		this.maxReward = fields.maxReward;
	}

	static schema = new Map([[
		StakeSchema, { 
			kind: "struct", 
			fields: [
				['timestamp', 'u64'],
				['staker', ['u8', 32]],
				['mint', ['u8', 32]],
				['active', ['u8', 1]],
				['withdrawn', 'u64'],
				['harvested', 'u64'],
				['stakedAmount', 'u64'],
				['maxReward', 'u64'],
			]
		}
	]])

	serialize(): Uint8Array {
		return serialize(StakeSchema.schema, this);
	}
}

class VaultSchema {
	mint: PublicKey
	minPeriod: Numberu64
	rewardPeriod: Numberu64
	rate: Numberu64
	earlyWithdrawalFee: Numberu64
	totalObligations: Numberu64
	totalStaked: Numberu64;

	constructor(fields?: {
		mint: PublicKey,
		minPeriod: Numberu64,
		rewardPeriod: Numberu64,
		rate: Numberu64,
		earlyWithdrawalFee: Numberu64, 
		totalObligations: Numberu64,
		totalStaked: Numberu64,
	}){
		this.mint = fields.mint;
		this.minPeriod = fields.minPeriod;
		this.rewardPeriod = fields.rewardPeriod;
		this.rate = fields.rate;
		this.earlyWithdrawalFee = fields.earlyWithdrawalFee;
		this.totalObligations = fields.totalObligations;
		this.totalStaked = fields.totalStaked;
	}

	static schema = new Map([[
		VaultSchema, {
			kind: "struct",
			fields: [ 
				['mint', ['u8', 32]],
				['minPeriod', 'u64'],
				['rewardPeriod', 'u64'],
				['rate', 'u64'],
				['earlyWithdrawalFee', 'u64'],
				['totalObligations', 'u64'],
				['totalStaked', 'u64']
			]
		}
	]])

	serialize(): Uint8Array {
		return serialize(VaultSchema.schema, this);
	}
}

//------------------------Get Account Data-----------------------------------

export function getVaultData(connection: Connection) {
	console.log("retrieving account data");
	console.log(SCY_STAKING_VAULT_INFO.toBase58());
	connection.getAccountInfo(SCY_STAKING_VAULT_INFO).then(
		r => {
			console.log(r);
			const val = deserializeUnchecked(
				VaultSchema.schema,
				VaultSchema,
				r.data,
			);
			console.log(val)
		},
		error => alert(error)
	);
}

export function getStakeData(connection: Connection, staker: PublicKey) {
	console.log("retrieving stake data");
	console.log(staker.toBase58());
	connection.getAccountInfo(staker).then(
		r => {
			console.log(r);
			const val = deserializeUnchecked(
				StakeSchema.schema,
				StakeSchema,
				r.data,
			);
			console.log(val);
		},
		error => alert(error)
	);
}

//------------------------Instructions-----------------------------------------
export class createStakeInstruction {
	amount: Numberu64;
	static schema: Schema = new Map([[createStakeInstruction, {kind: "struct", fields: [["amount", "u64"]]},],]);

	constructor(obj: {
		amount: Numberu64;
	}) {
		this.amount = obj.amount;
	}

	serialize(): Uint8Array {
		return serialize(createStakeInstruction.schema, this);
	}

	getInstruction(
		programId: PublicKey,
		staker: PublicKey,
		stakerTokenAccount: PublicKey,
		stakeInfo: PublicKey,
	): TransactionInstruction {
		const data = Buffer.from(this.serialize());
		let keys = [
			{
				pubkey: staker,
				isSigner: true,
				isWritable: true,
			},
			{
				pubkey: stakeInfo,
				isSigner: false,
				isWritable: true,
			},		
			{
				pubkey: stakerTokenAccount,
				isSigner: false,
				isWritable: true,
			},

			{
				pubkey: SCY_STAKING_VAULT_INFO,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: SCY_STAKING_VAULT_TOKEN_ADDRESS,
				isSigner: false,
				isWritable: true,
			},
			{
				pubkey: SCY_MINT,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: TOKEN_PROGRAM_ID,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: SYSTEM_PROGRAM_ID,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: RENT_SYSVAR,
				isSigner: false,
				isWritable: false,
			},
		];

		return new TransactionInstruction({
			keys,
			programId: SCY_STAKING_PROGRAM_ID,
			data,
		});
	}
}
export class createUnstakeInstruction {
	static schema: Schema = new Map([[createUnstakeInstruction, {},],]);

	serialize(): Uint8Array {
		return serialize(createUnstakeInstruction.schema, this);
	}

	getInstruction(
		programId: PublicKey,
		staker: PublicKey,
		stakerTokenAccount: PublicKey,
		stakeInfo: PublicKey,
	): TransactionInstruction {
		const data = Buffer.from(this.serialize());
		let keys = [
			{
				pubkey: staker,
				isSigner: true,
				isWritable: true,
			},
			{
				pubkey: stakeInfo,
				isSigner: false,
				isWritable: true,
			},
			{
				pubkey: stakerTokenAccount,
				isSigner: false,
				isWritable: true,
			},
			{
				pubkey: SCY_STAKING_VAULT_INFO,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: SCY_STAKING_VAULT_TOKEN_ADDRESS,
				isSigner: false,
				isWritable: true,
			},
			{
				pubkey: TOKEN_PROGRAM_ID,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: SYSTEM_PROGRAM_ID,
				isSigner: false,
				isWritable: false,
			},
			{
				pubkey: RENT_SYSVAR,
				isSigner: false,
				isWritable: false,
			},
		];

		return new TransactionInstruction({
			keys,
			programId: SCY_STAKING_PROGRAM_ID,
			data,
		});
	}
}



//------------------------------------------------------------------------------
export const signAndSendTransactionInstructions = async (
	// sign and send transaction
	connection: Connection,
	signers: Array<Keypair>,
	feePayer: Keypair,
	txInstructions: Array<TransactionInstruction>
): Promise<string> => {
	const tx = new Transaction();
	tx.feePayer = feePayer.publicKey;
	signers.push(feePayer);
	tx.add(...txInstructions);
	return await connection.sendTransaction(tx, signers, {
		preflightCommitment: "single",
	});
};

export async function findStakeInfoAddress(
	walletAddress: PublicKey,
): Promise<PublicKey> {
	return (
		await PublicKey.findProgramAddress(
			[
				walletAddress.toBuffer(),
			],
			SCY_STAKING_PROGRAM_ID,
		)
	)[0];
}

export async function findVaultInfoAddress(): Promise<PublicKey> {
	return (
		await PublicKey.findProgramAddress(
			[VAULT_SEED], 
			SCY_STAKING_PROGRAM_ID
		)
	)[0];
}

export async function findAssociatedTokenAddress(
	walletAddress: PublicKey,
	tokenMintAddress: PublicKey
): Promise<PublicKey> {
	return (
		await PublicKey.findProgramAddress(
			[
				walletAddress.toBuffer(),
				TOKEN_PROGRAM_ID.toBuffer(),
				tokenMintAddress.toBuffer(),
			],
			ASSOCIATED_TOKEN_PROGRAM_ID
		)
	)[0];
}
