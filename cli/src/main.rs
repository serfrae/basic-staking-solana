use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand,
};
#[allow(unused_imports)]
use solana_clap_utils::{
    input_parsers::{pubkey_of, value_of},
    input_validators::{is_amount, is_keypair, is_parsable, is_pubkey, is_url},
    keypair::{DefaultSigner, SignerFromPathConfig},
};
#[allow(unused_imports)]
use solana_client::rpc_client::RpcClient;
#[allow(unused_imports)]
use solana_sdk::borsh::try_from_slice_unchecked;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Signer};
#[allow(unused_imports)]
use solana_sdk::signer::keypair::Keypair;
#[allow(unused_imports)]
use solana_sdk::signer::signers::Signers;
use solana_sdk::system_program;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account;
use spl_token;

const PROGRAM_ID: &str = "scyURqbFspMW69Pa5hjY74NJyn1Hbx14YEfDMZeJTcB";
const VAULT_SEED: &[u8; 8] = b"___vault";
const MINT: &str = "SCYVn1w92poF5VaLf2myVBbTvBf1M8MLqJwpS64Gb9b";
#[allow(dead_code)]
const ADMIN_PK: &str = "CbXeKZ47sfbTxyiAg5h4GLpdrnmzwVXPPihfkN3GiNKk";

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
enum StakeInstruction {
    GenerateVault {
        #[allow(dead_code)]
        min_period: u64,
        #[allow(dead_code)]
        reward_period: u64,
        #[allow(dead_code)]
        rate: u64,
        #[allow(dead_code)]
        early_withdrawal_fee: u64,
    },
    Stake {
        #[allow(dead_code)]
        amount: u64,
    },
    Unstake,
    Withdraw {
        #[allow(dead_code)]
        amount: u64,
    },
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct StakeData {
    timestamp: u64,
    staker: Pubkey,
    mint: Pubkey,
    active: bool,
    withdrawn: u64,
    harvested: u64,
    staked_amount: u64,
    max_reward: u64,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
struct VaultData {
    mint: Pubkey,
    min_period: u64,
    reward_period: u64,
    rate: u64,
    early_withdrawal_fee: u64,
    total_obligations: u64,
    total_staked: u64,
}

fn main() {
    let matches = app_from_crate!()
        .subcommand(
            SubCommand::with_name("generate_vault_address")
                .arg(
                    Arg::with_name("sign")
                        .short("s")
                        .long("sign")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("env")
                        .short("e")
                        .long("env")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("min_period")
                        .short("m")
                        .long("min_period")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("reward_period")
                        .short("p")
                        .long("reward_period")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("rate")
                        .short("r")
                        .long("rate")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("early_withdrawal_fee")
                        .short("w")
                        .long("early_withdrawal_fee")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("stake")
                .arg(
                    Arg::with_name("sign")
                        .short("s")
                        .long("sign")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("env")
                        .short("e")
                        .long("env")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("amount")
                        .short("a")
                        .validator(is_amount)
                        .long("amount")
                        .value_name("AMOUNT")
                        .allow_hyphen_values(true)
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("unstake")
                .arg(
                    Arg::with_name("sign")
                        .short("s")
                        .long("sign")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("env")
                        .short("e")
                        .long("env")
                        .required(false)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("withdraw")
                .arg(
                    Arg::with_name("sign")
                        .short("s")
                        .long("sign")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("env")
                        .short("e")
                        .long("env")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("amount")
                        .short("a")
                        .validator(is_amount)
                        .allow_hyphen_values(true)
                        .long("amount")
                        .value_name("AMOUNT")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("vault").arg(
                Arg::with_name("env")
                    .short("e")
                    .long("env")
                    .required(false)
                    .takes_value(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("stake-data")
                .arg(
                    Arg::with_name("env")
                        .short("e")
                        .long("env")
                        .required(false)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("address")
                        .short("p")
                        .long("pubkey")
                        .required(true)
                        .takes_value(true)
                        .index(1),
                ),
        )
        .get_matches();

    let program_id = PROGRAM_ID.parse::<Pubkey>().unwrap();
    let mint_pk = MINT.parse::<Pubkey>().unwrap();

    if let Some(matches) = matches.subcommand_matches("withdraw") {
        let url = match matches.value_of("env") {
            Some("dev") => "https://api.devnet.solana.com",
            _ => "https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());

        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let amount = spl_token::ui_amount_to_amount(
            matches.value_of("amount").unwrap().parse::<u64>().unwrap() as f64,
            9,
        );
        println!("{}", amount);

        let (vault, _vault_bump) = Pubkey::find_program_address(&[VAULT_SEED], &program_id);
        let staker_token_account =
            spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &mint_pk);
        let vault_token_account =
            spl_associated_token_account::get_associated_token_address(&vault, &mint_pk);
        let (stake_info, _stake_bump) =
            Pubkey::find_program_address(&[&wallet_pubkey.to_bytes()], &program_id);

        let instructions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Withdraw { amount },
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(stake_info, false),
                AccountMeta::new(staker_token_account, false),
                AccountMeta::new_readonly(vault, false),
                AccountMeta::new(vault_token_account, false),
                AccountMeta::new_readonly(mint_pk, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(
                    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
                        .parse::<Pubkey>()
                        .unwrap(),
                    false,
                ),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(
                    "SysvarRent111111111111111111111111111111111"
                        .parse::<Pubkey>()
                        .unwrap(),
                    false,
                ),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instructions, Some(&wallet_pubkey));
        let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("unstake") {
        let url = match matches.value_of("env") {
            Some("dev") => "https://api.devnet.solana.com",
            _ => "https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());

        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let (vault, _vault_bump) = Pubkey::find_program_address(&[VAULT_SEED], &program_id);
        let staker_token_account =
            spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &mint_pk);
        let vault_token_account =
            spl_associated_token_account::get_associated_token_address(&vault, &mint_pk);
        let (stake_data, _) =
            Pubkey::find_program_address(&[&wallet_pubkey.to_bytes()], &program_id);

        let accounts = vec![
            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new(stake_data, false),
            AccountMeta::new(staker_token_account, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(vault_token_account, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(mint_pk, false),
        ];
        // println!("{:#?}", accounts);
        let instructions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Unstake,
            accounts,
        )];
        let mut tx = Transaction::new_with_payer(&instructions, Some(&wallet_pubkey));
        let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("stake") {
        let url = match matches.value_of("env") {
            Some("dev") => "https://api.devnet.solana.com",
            _ => "https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());

        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let (vault, _vault_bump) = Pubkey::find_program_address(&[VAULT_SEED], &program_id);
        let staker_token_account =
            spl_associated_token_account::get_associated_token_address(&wallet_pubkey, &mint_pk);
        let vault_token_account =
            spl_associated_token_account::get_associated_token_address(&vault, &mint_pk);
        println!("Vault Token Account: {}", vault_token_account);
        let (stake_data, _) =
            Pubkey::find_program_address(&[&wallet_pubkey.to_bytes()], &program_id);
        let amount = spl_token::ui_amount_to_amount(
            matches.value_of("amount").unwrap().parse::<f64>().unwrap(),
            9,
        );
        println!("Amount: {}", amount);

        let instructions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::Stake { amount },
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(stake_data, false),
                AccountMeta::new(staker_token_account, false),
                AccountMeta::new(vault, false),
                AccountMeta::new(vault_token_account, false),
                AccountMeta::new_readonly(MINT.parse::<Pubkey>().unwrap(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(
                    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
                        .parse::<Pubkey>()
                        .unwrap(),
                    false,
                ),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(
                    "SysvarRent111111111111111111111111111111111"
                        .parse::<Pubkey>()
                        .unwrap(),
                    false,
                ),
            ],
        )];
        let mut tx = Transaction::new_with_payer(&instructions, Some(&wallet_pubkey));
        let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("generate_vault_address") {
        let url = match matches.value_of("env") {
            Some("dev") => "https://api.devnet.solana.com",
            _ => "https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());

        let wallet_path = matches.value_of("sign").unwrap();
        let wallet_keypair = read_keypair_file(wallet_path).expect("Can't open file-wallet");
        let wallet_pubkey = wallet_keypair.pubkey();

        let min_period = matches
            .value_of("min_period")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let reward_period = matches
            .value_of("reward_period")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let rate = matches.value_of("rate").unwrap().parse::<u64>().unwrap();
        let early_withdrawal_fee = matches
            .value_of("early_withdrawal_fee")
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let mint = MINT.parse::<Pubkey>().unwrap();
        let (vault_pda, _) = Pubkey::find_program_address(&[VAULT_SEED], &program_id);
        let vault_token_address =
            spl_associated_token_account::get_associated_token_address(&vault_pda, &mint);

        let instructions = vec![Instruction::new_with_borsh(
            program_id,
            &StakeInstruction::GenerateVault {
                min_period,
                reward_period,
                rate,
                early_withdrawal_fee,
            },
            vec![
                AccountMeta::new(wallet_pubkey, true),
                AccountMeta::new(vault_pda, false),
                AccountMeta::new(vault_token_address, false),
                AccountMeta::new_readonly(mint, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(
                    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
                        .parse::<Pubkey>()
                        .unwrap(),
                    false,
                ),
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new_readonly(
                    "SysvarRent111111111111111111111111111111111"
                        .parse::<Pubkey>()
                        .unwrap(),
                    false,
                ),
            ],
        )];

        let mut tx = Transaction::new_with_payer(&instructions, Some(&wallet_pubkey));
        let recent_blockhash = client.get_latest_blockhash().expect("Can't get blockhash");
        tx.sign(&vec![&wallet_keypair], recent_blockhash);
        let id = client.send_transaction(&tx).expect("Transaction failed.");
        println!("vault account generated: {:?}", vault_pda);
        println!("tx id: {:?}", id);
    }

    if let Some(matches) = matches.subcommand_matches("vault") {
        let url = match matches.value_of("env") {
            Some("dev") => "https://api.devnet.solana.com",
            _ => "https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());
        let (vault_data_pk, _vault_data_bump) =
            Pubkey::find_program_address(&[VAULT_SEED], &program_id);
        let raw_vault_data = client.get_account_data(&vault_data_pk).unwrap().clone();
        let vault_data = VaultData::try_from_slice(&raw_vault_data[..]).unwrap();

        println!("Vault Mint: {}", vault_data.mint);
        println!("Minimum Staking Period: {}", vault_data.min_period);
        println!("Reward Period: {}", vault_data.reward_period);
        println!("APR: {}", vault_data.rate);
        println!("Early Withdrawal Fee: {}", vault_data.early_withdrawal_fee);
        println!(
            "Total Obligations: {}",
            spl_token::amount_to_ui_amount(vault_data.total_obligations, 9)
        );
        println!(
            "Total Staked: {}",
            spl_token::amount_to_ui_amount(vault_data.total_staked, 9)
        );
    }

    if let Some(matches) = matches.subcommand_matches("stake-data") {
        let url = match matches.value_of("env") {
            Some("dev") => "https://api.devnet.solana.com",
            _ => "https://api.mainnet-beta.solana.com",
        };
        let client = RpcClient::new_with_commitment(url.to_string(), CommitmentConfig::confirmed());
        let address = matches
            .value_of("address")
            .unwrap()
            .parse::<Pubkey>()
            .unwrap();

        let (stake_data_pk, _stake_data_bump) = Pubkey::find_program_address(
            &[&address.to_bytes()],
            &PROGRAM_ID.parse::<Pubkey>().unwrap(),
        );

        let raw_stake_data = client.get_account_data(&stake_data_pk).unwrap().clone();
        let stake_data = StakeData::try_from_slice(&raw_stake_data[..]).unwrap();

        println!("Started Staking: {}", stake_data.timestamp);
        println!("Staker Address: {}", stake_data.staker);
        println!("Mint of Staked Token: {}", stake_data.mint);
        println!("Staking Active: {}", stake_data.active);
        println!(
            "Amount Withdrawn: {}",
            spl_token::amount_to_ui_amount(stake_data.withdrawn, 9)
        );
        println!(
            "Amount Harvested: {}",
            spl_token::amount_to_ui_amount(stake_data.harvested, 9)
        );
        println!(
            "Staked Amount: {}",
            spl_token::amount_to_ui_amount(stake_data.staked_amount, 9)
        );
        println!(
            "Maximum Reward: {}",
            spl_token::amount_to_ui_amount(stake_data.max_reward, 9)
        );
    }
}
