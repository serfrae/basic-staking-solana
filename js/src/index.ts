import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
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
import * as borsh from "borsh";
import Wallet from "@project-serum/sol-wallet-adapter";

//---------------Pubkeys + Seeds-----------------------------------------======

export const VAULT_SEED = Buffer.from("___vault");

export const ASSOCIATED_TOKEN_PROGRAM_ID: PublicKey = new PublicKey(
	"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

export const SCY_STAKING_PROGRAM_ID: PublicKey = new PublicKey(
	"SCYV7PXsvGy4PKZLrZCZPaVDccNSNEBKCdJ6etycwEF"
);

export const SCY_MINT: PublicKey = new PublicKey(
	"SCYVn1w92poF5VaLf2myVBbTvBf1M8MLqJwpS64Gb9b"
);

export const SCY_STAKING_VAULT_INFO: PublicKey = new PublicKey(
	findVaultInfoAddress()
);

export const SCY_STAKING_VAULT_TOKEN_ADDRESS: PublicKey = new PublicKey(
	findAssociatedTokenAddress(SCY_STAKING_VAULT_INFO, SCY_MINT)
);

export const RENT_SYSVAR: PublicKey = new PublicKey(
	"SysvarRent111111111111111111111111111111111"
);

export const SYSTEM_PROGRAM_ID: PublicKey = new PublicKey(
	"11111111111111111111111111111111"
);


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

	constructor(
		timestamp?: Numberu64,
		staker?: PublicKey,
		mint?: PublicKey,
		active?: boolean,
		withdraw?: Numberu64,
		harvested?: Numberu64,
		stakedAmount?: Numberu64,
		maxReward?: Numberu64,
	) {
		this.timestamp = timestamp;
		this.staker = staker;
		this.mint = mint;
		this.active = active;
		this.withdraw = withdraw;
		this.harvested = harvested;
		this.stakedAmount = stakedAmount;
		this.maxReward = maxReward;
	}

	static schema = new Map([[
		StakeSchema, { 
			kind: "struct", 
			fields: [
				['timestamp', 'u64'],
				['staker', 'Pubkey'],
				['mint', 'Pubkey'],
				['active', 'bool'],
				['withdrawn', 'u64'],
				['harvested', 'u64'],
				['staked_amount', 'u64'],
				['max_reward', 'u64'],
			]
		}
	]])

	serialize(): Uint8Array {
		return serialize(StakeSchema.schema, this);
	}
}

class Assignable {
	constructor(properties) {
        Object.keys(properties).map((key) => {
            this[key] = properties[key];
        });
    }
}
class Stake extends Assignable { }
class Vault extends Assignable {}

const SCY_SCHEMA: Map<Function, any> = new Map();

class VaultSchema {
	mint: PublicKey = SCY_MINT
	minPeriod: Numberu64 = new Numberu64(3)
	rewardPeriod: Numberu64 = new Numberu64(1)
	rate: Numberu64 = new Numberu64(2)
	earlyWithdrawalFee: Numberu64 = new Numberu64(5)
	totalObligations: Numberu64 = new Numberu64(0)
	totalStaked: Numberu64 = new Numberu64(0)

	constructor(fields?: {
		mint: PublicKey,
		minPeriod: Numberu64,
		rewardPeriod: Numberu64,
		rate: Numberu64,
		earlyWithdrawalFee: Numberu64, 
		totalObligations: Numberu64,
		totalStaked: Numberu64,
	}
	){
		if (fields) {
		this.mint = fields.mint;
		this.minPeriod = fields.minPeriod;
		this.rewardPeriod = fields.rewardPeriod;
		this.rate = fields.rate;
		this.earlyWithdrawalFee = fields.earlyWithdrawalFee;
		this.totalObligations = fields.totalObligations;
		this.totalStaked = fields.totalStaked;
		}
	}

	static schema = new Map([[
		VaultSchema, {
			kind: "struct",
			fields: [ 
				['mint', 'u64'],
				['min_period', 'u64'],
				['reward_period', 'u64'],
				['rate', 'u64'],
				['early_withdrawal_fee', 'u64'],
				['total_obligations', 'u64'],
				['total_staked', 'u64']
			]
		}
	]])

	serialize(): Uint8Array {
		return borsh.serialize(VaultSchema.schema, this);
	}

	static async deserialize(connection: Connection) {
		connection.getAccountInfo(SCY_STAKING_VAULT_INFO).then(
			r => console.log(`${borsh.deserialize(
				SCY_SCHEMA,
				VaultSchema.schema,
				r.data,
			)}`),
			error => alert(error)

		);
	}
}

//------------------------Get Account Data-----------------------------------

export function getStakeData(connection: Connection, staker: PublicKey) {
	connection.getAccountInfo(staker).then(
		r => console.log(`${borsh.deserialize(
			Stake,
			StakeSchema.schema,
			r.data,
		)}`),
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
		//assert(b.length < 8, "Numberu64 too large");

		const zeroPad = Buffer.alloc(8);
		b.copy(zeroPad);
		return zeroPad;
	}

	/**
	 * Construct a Numberu64 from Buffer representation
	 */
	static fromBuffer(buffer): any {
		//assert(buffer.length === 8, `Invalid buffer length: ${buffer.length}`);
		return new BN(
			[...buffer]
			.reverse()
			.map((i) => `00${i.toString(16)}`.slice(-2))
			.join(""),
			16
		);
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
