use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::program::{invoke, invoke_signed};
use solana_program::{
    self,
    account_info::{next_account_info, AccountInfo},
    declare_id, entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};
use spl_associated_token_account;
use spl_token;

const YEAR: u64 = 31_556_926;
const VAULT_SEED: &[u8; 8] = b"___vault";
const ADMIN_PK: &str = "CbXeKZ47sfbTxyiAg5h4GLpdrnmzwVXPPihfkN3GiNKk";
const MINT: &str = "SCYVn1w92poF5VaLf2myVBbTvBf1M8MLqJwpS64Gb9b";

declare_id!("scyURqbFspMW69Pa5hjY74NJyn1Hbx14YEfDMZeJTcB");

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

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

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let instruction: StakeInstruction = try_from_slice_unchecked(instruction_data).unwrap();

    let admin = ADMIN_PK.parse::<Pubkey>().unwrap();
    let mint = MINT.parse::<Pubkey>().unwrap();

    match instruction {
        StakeInstruction::Withdraw { amount } => {
            let admin_info = next_account_info(accounts_iter)?;
            let admin_token_account_info = next_account_info(accounts_iter)?;

            let vault_info = next_account_info(accounts_iter)?;
            let vault_token_account_info = next_account_info(accounts_iter)?;

            let token_info = next_account_info(accounts_iter)?;
            let mint_info = next_account_info(accounts_iter)?;

            if *admin_info.key != admin || !admin_info.is_signer {
                //unauthorized access
                return Err(ProgramError::Custom(0x231));
            }

            let (vault_address, vault_bump) =
                Pubkey::find_program_address(&[VAULT_SEED], &program_id);
            let admin_token_account =
                spl_associated_token_account::get_associated_token_address(admin_info.key, &mint);
            let vault_token_account =
                spl_associated_token_account::get_associated_token_address(vault_info.key, &mint);

            let vault_data = VaultData::try_from_slice(&vault_info.data.borrow())?;
            let vault_token_account_data = spl_token::state::Account::unpack_from_slice(
                &vault_token_account_info.data.borrow(),
            )?;
            if amount
                > vault_token_account_data.amount
                    - (vault_data.total_obligations + vault_data.total_staked)
            {
                return Err(ProgramError::InsufficientFunds);
            }

            if vault_address != *vault_info.key {
                //wrong stake_info
                return Err(ProgramError::Custom(0x261));
            }

            if admin_token_account != *admin_token_account_info.key {
                //wrong payer_reward_holder_info
                return Err(ProgramError::Custom(0x262));
            }

            if vault_token_account != *vault_token_account_info.key {
                //wrong vault_reward_holder_info
                return Err(ProgramError::Custom(0x263));
            }

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_info.key,
                    vault_token_account_info.key,
                    admin_token_account_info.key,
                    vault_info.key,
                    &[],
                    amount,
                )?,
                &[
                    vault_token_account_info.clone(),
                    admin_token_account_info.clone(),
                    vault_info.clone(),
                    token_info.clone(),
                ],
                &[&[VAULT_SEED, &[vault_bump]]],
            )?;
        }
        StakeInstruction::Unstake => {
            let staker = next_account_info(accounts_iter)?;
            let stake_info = next_account_info(accounts_iter)?;
            let staker_token_account_info = next_account_info(accounts_iter)?;

            let vault_info = next_account_info(accounts_iter)?;
            let vault_token_account_info = next_account_info(accounts_iter)?;

            let token_info = next_account_info(accounts_iter)?;
            let mint_info = next_account_info(accounts_iter)?;

            let clock = Clock::get()?;

            let (stake_address, _stake_bump) =
                Pubkey::find_program_address(&[&staker.key.to_bytes()], &program_id);
            let (vault_address, vault_bump) =
                Pubkey::find_program_address(&[VAULT_SEED], &program_id);
            let staker_token_account =
                spl_associated_token_account::get_associated_token_address(staker.key, &mint);
            let vault_token_account =
                spl_associated_token_account::get_associated_token_address(vault_info.key, &mint);

            if !staker.is_signer {
                //unauthorized access
                return Err(ProgramError::Custom(0x11));
            }

            if *token_info.key != spl_token::id() {
                //wrong token_info
                return Err(ProgramError::Custom(0x345));
            }

            if stake_address != *stake_info.key {
                //wrong stake_info
                return Err(ProgramError::Custom(0x60));
            }

            if vault_address != *vault_info.key {
                //wrong stake_info
                return Err(ProgramError::Custom(0x61));
            }

            if staker_token_account != *staker_token_account_info.key {
                //wrong payer_reward_holder_info
                return Err(ProgramError::Custom(0x62));
            }

            if vault_token_account != *vault_token_account_info.key {
                //wrong vault_reward_holder_info
                return Err(ProgramError::Custom(0x63));
            }

            if mint != *mint_info.key {
                //wrong reward_mint_info
                return Err(ProgramError::Custom(0x67));
            }

            let mut vault_data =
                if let Ok(data) = VaultData::try_from_slice(&vault_info.data.borrow()) {
                    data
                } else {
                    // can't deserialize vault data
                    return Err(ProgramError::Custom(0x912));
                };

            let mut stake_data =
                if let Ok(data) = StakeData::try_from_slice(&stake_info.data.borrow()) {
                    data
                } else {
                    // can't deserialize stake data
                    return Err(ProgramError::Custom(0x913));
                };

            if !stake_data.active {
                //staking is inactive
                return Err(ProgramError::Custom(0x107));
            }

            if stake_data.staker != *staker.key {
                //unauthorized access
                return Err(ProgramError::Custom(0x108));
            }

            let elapsed_duration = (clock.unix_timestamp as u64)
                .checked_sub(stake_data.timestamp)
                .unwrap();
            let n_elapsed_rewards = elapsed_duration
                .checked_div(vault_data.reward_period)
                .unwrap();

            let reward_per_period = stake_data
                .max_reward
                .checked_div(YEAR.checked_div(vault_data.reward_period).unwrap())
                .unwrap();
            //CHECK
            let mut reward = n_elapsed_rewards.checked_mul(reward_per_period).unwrap();
            let mut withdrawal_amount = reward.checked_add(stake_data.staked_amount).unwrap();

            let total_withdrawal = if elapsed_duration < vault_data.min_period {
                withdrawal_amount = withdrawal_amount
                    .checked_sub(withdrawal_amount.checked_div(20).unwrap())
                    .unwrap();
                withdrawal_amount
            } else {
                withdrawal_amount
            };

            msg!("periods passed {:?}", n_elapsed_rewards);
            msg!("reward {:?}", spl_token::amount_to_ui_amount(reward, 9));
            //msg!("Already harvested {:?}", stake_data.harvested);
            msg!(
                "max reward {:?}",
                spl_token::amount_to_ui_amount(stake_data.max_reward, 9)
            );
            msg!("already withdrawn {:?}", stake_data.withdrawn);
            msg!(
                "rinal reward {:?}",
                spl_token::amount_to_ui_amount(reward, 9)
            );

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_info.key,
                    vault_token_account_info.key,
                    staker_token_account_info.key,
                    vault_info.key,
                    &[],
                    total_withdrawal,
                )?,
                &[
                    vault_token_account_info.clone(),
                    staker_token_account_info.clone(),
                    vault_info.clone(),
                    token_info.clone(),
                ],
                &[&[VAULT_SEED, &[vault_bump]]],
            )?;

            vault_data.total_obligations -= stake_data.max_reward;
            vault_data.total_staked -= stake_data.staked_amount;
            vault_data.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;

            stake_data.active = false;
            stake_data.harvested += reward;
            stake_data.withdrawn += reward + stake_data.staked_amount;
            stake_data.staked_amount = 0;
            stake_data.max_reward = 0;
            stake_data.serialize(&mut &mut stake_info.data.borrow_mut()[..])?;
        }

        StakeInstruction::Stake { amount } => {
            let staker = next_account_info(accounts_iter)?;
            let stake_data_info = next_account_info(accounts_iter)?;
            let staker_token_account_info = next_account_info(accounts_iter)?;

            let vault_info = next_account_info(accounts_iter)?;
            let vault_token_account_info = next_account_info(accounts_iter)?;

            let mint_info = next_account_info(accounts_iter)?;

            let token_program = next_account_info(accounts_iter)?;
            let token_assoc = next_account_info(accounts_iter)?;
            let sys_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;

            let clock = Clock::get()?;
            let rent = &Rent::from_account_info(rent_info)?;

            let (stake_data, stake_data_bump) =
                Pubkey::find_program_address(&[&staker.key.to_bytes()], &program_id);
            let (vault_address, _vault_bump) =
                Pubkey::find_program_address(&[VAULT_SEED], &program_id);
            let staker_token_account =
                spl_associated_token_account::get_associated_token_address(staker.key, &mint);
            let vault_token_account =
                spl_associated_token_account::get_associated_token_address(vault_info.key, &mint);

            if *token_program.key != spl_token::id() {
                //wrong token_info
                return Err(ProgramError::Custom(0x345));
            }

            if !staker.is_signer {
                //unauthorized access
                return Err(ProgramError::Custom(0x11));
            }

            if stake_data != *stake_data_info.key {
                //msg!("invalid stake_data account!");
                return Err(ProgramError::Custom(0x10));
            }
            msg!("Stake Safety Checks OK.");

            if stake_data_info.try_data_is_empty()? {
                msg!("No staking account found, creating...");
                let size: u64 = 8 + 32 + 32 + 1 + 8 + 8 + 8 + 8;
                if stake_data_info.owner != program_id {
                    let required_lamports = rent
                        .minimum_balance(size as usize)
                        .max(1)
                        .saturating_sub(stake_data_info.lamports());
                    invoke(
                        &system_instruction::transfer(staker.key, &stake_data, required_lamports),
                        &[staker.clone(), stake_data_info.clone(), sys_info.clone()],
                    )?;
                    invoke_signed(
                        &system_instruction::allocate(&stake_data, size),
                        &[stake_data_info.clone(), sys_info.clone()],
                        &[&[&staker.key.to_bytes(), &[stake_data_bump]]],
                    )?;

                    invoke_signed(
                        &system_instruction::assign(&stake_data, program_id),
                        &[stake_data_info.clone(), sys_info.clone()],
                        &[&[&staker.key.to_bytes(), &[stake_data_bump]]],
                    )?;
                }
                let stake_data = StakeData::try_from_slice(&stake_data_info.data.borrow());
                let mut vault_data =
                    if let Ok(data) = VaultData::try_from_slice(&vault_info.data.borrow()) {
                        data
                    } else {
                        return Err(ProgramError::InvalidAccountData);
                    };

                let total_staker_reward = amount.checked_mul(vault_data.rate).unwrap();

                let harvested = if let Ok(data) = &stake_data {
                    data.harvested
                } else {
                    0
                };

                let staked_amount = if let Ok(data) = &stake_data {
                    data.staked_amount
                } else {
                    amount
                };

                let total_staked = vault_data.total_staked + amount;
                let total_obligations = vault_data.total_obligations + total_staker_reward;

                if spl_token::state::Account::unpack_from_slice(
                    &vault_token_account_info.data.borrow(),
                )?
                .amount
                    < (total_obligations + total_staked)
                {
                    return Err(ProgramError::InsufficientFunds);
                }

                let stake_struct = StakeData {
                    timestamp: clock.unix_timestamp as u64,
                    staker: *staker.key,
                    harvested,
                    active: true,
                    withdrawn: 0,
                    mint,
                    staked_amount: amount,
                    max_reward: amount.checked_mul(2).unwrap(), // change to rate
                };

                vault_data.total_staked = total_staked;
                vault_data.total_obligations = total_obligations;

                stake_struct.serialize(&mut &mut stake_data_info.data.borrow_mut()[..])?;
                vault_data.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;
                msg!("Stake Data Initialized");

                let (vault, vault_bump) = Pubkey::find_program_address(&[VAULT_SEED], &program_id);
                if vault != *vault_info.key {
                    //msg!("Wrong vault");
                    return Err(ProgramError::Custom(0x07));
                }
            } else {
                msg!("Previous staking account found, rolling-over...");
                let mut vault_data =
                    if let Ok(data) = VaultData::try_from_slice(&vault_info.data.borrow()) {
                        data
                    } else {
                        // can't deserialize vault data
                        return Err(ProgramError::Custom(0x912));
                    };

                let mut stake_data =
                    if let Ok(data) = StakeData::try_from_slice(&stake_data_info.data.borrow()) {
                        data
                    } else {
                        // can't deserialize stake data
                        return Err(ProgramError::Custom(0x913));
                    };

                if stake_data.staker != *staker.key {
                    //unauthorized access
                    return Err(ProgramError::Custom(0x108));
                }

                let elapsed_duration = (clock.unix_timestamp as u64)
                    .checked_sub(stake_data.timestamp)
                    .unwrap();
                let n_elapsed_rewards = elapsed_duration
                    .checked_div(vault_data.reward_period)
                    .unwrap();

                let reward_per_period = stake_data
                    .max_reward
                    .checked_div(YEAR.checked_div(vault_data.reward_period).unwrap())
                    .unwrap();
                //CHECK
                let mut reward = n_elapsed_rewards.checked_mul(reward_per_period).unwrap();

                vault_data.total_staked += reward;
                vault_data.total_obligations -= stake_data.max_reward;

                stake_data.active = true;
                stake_data.staked_amount += amount + reward;
                stake_data.max_reward = stake_data.staked_amount.checked_mul(2).unwrap();
                stake_data.timestamp = clock.unix_timestamp as u64;

                vault_data.total_staked += amount;
                vault_data.total_obligations += stake_data.max_reward;

                if spl_token::state::Account::unpack_from_slice(
                    &vault_token_account_info.data.borrow(),
                )?
                .amount
                    < (vault_data.total_obligations + vault_data.total_staked)
                {
                    return Err(ProgramError::InsufficientFunds);
                }

                msg!("periods passed {:?}", n_elapsed_rewards);
                msg!("reward {:?}", spl_token::amount_to_ui_amount(reward, 9));
                msg!("Already harvested {:?}", stake_data.harvested);
                msg!(
                    "Max reward {:?}",
                    spl_token::amount_to_ui_amount(stake_data.max_reward, 9)
                );
                msg!("Already withdrawn {:?}", stake_data.withdrawn);
                msg!(
                    "final reward {:?}",
                    spl_token::amount_to_ui_amount(reward, 9)
                );
                stake_data.serialize(&mut &mut stake_data_info.data.borrow_mut()[..])?;
                vault_data.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;
            }

            if &spl_associated_token_account::get_associated_token_address(
                staker.key,
                mint_info.key,
            ) != staker_token_account_info.key
            {
                // msg!("Wrong source");
                return Err(ProgramError::Custom(0x08));
            }

            if &spl_associated_token_account::get_associated_token_address(
                &vault_info.key,
                mint_info.key,
            ) != vault_token_account_info.key
            {
                //msg!("Wrong destination");
                return Err(ProgramError::Custom(0x09));
            }

            if vault_token_account_info.owner != token_program.key {
                invoke(
                    &spl_associated_token_account::create_associated_token_account(
                        staker.key,
                        vault_info.key,
                        mint_info.key,
                    ),
                    &[
                        staker.clone(),
                        vault_token_account_info.clone(),
                        vault_info.clone(),
                        mint_info.clone(),
                        sys_info.clone(),
                        token_program.clone(),
                        rent_info.clone(),
                        token_assoc.clone(),
                    ],
                )?;
            }
            invoke(
                &spl_token::instruction::transfer(
                    token_program.key,
                    staker_token_account_info.key,
                    vault_token_account_info.key,
                    staker.key,
                    &[],
                    amount,
                )?,
                &[
                    staker_token_account_info.clone(),
                    vault_token_account_info.clone(),
                    staker.clone(),
                    token_program.clone(),
                ],
            )?;
        }

        StakeInstruction::GenerateVault {
            min_period,
            reward_period,
            rate,
            early_withdrawal_fee,
        } => {
            let payer = next_account_info(accounts_iter)?;
            let pda = next_account_info(accounts_iter)?;
            let vault_token_account_info = next_account_info(accounts_iter)?;
            let mint = next_account_info(accounts_iter)?;

            let token_program = next_account_info(accounts_iter)?;
            let atoken_program = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;

            let rent = &Rent::from_account_info(rent_info)?;

            let (vault_pda, vault_bump_seed) =
                Pubkey::find_program_address(&[VAULT_SEED], &program_id);
            let vault_token_account =
                spl_associated_token_account::get_associated_token_address(&pda.key, mint.key);

            if pda.key != &vault_pda {
                //msg!("Wrong account generated by client");
                return Err(ProgramError::Custom(0x00));
            }

            if vault_token_account != *vault_token_account_info.key {
                return Err(ProgramError::InvalidAccountData);
            }

            if pda.owner != program_id {
                let size = 32 + 8 + 8 + 8 + 8 + 8 + 8;

                let required_lamports = rent
                    .minimum_balance(size as usize)
                    .max(1)
                    .saturating_sub(pda.lamports());

                invoke(
                    &system_instruction::transfer(payer.key, &vault_pda, required_lamports),
                    &[payer.clone(), pda.clone(), system_program.clone()],
                )?;

                invoke_signed(
                    &system_instruction::allocate(&vault_pda, size),
                    &[pda.clone(), system_program.clone()],
                    &[&[VAULT_SEED, &[vault_bump_seed]]],
                )?;

                invoke_signed(
                    &system_instruction::assign(&vault_pda, program_id),
                    &[pda.clone(), system_program.clone()],
                    &[&[VAULT_SEED, &[vault_bump_seed]]],
                )?;
            }

            invoke(
                &spl_associated_token_account::create_associated_token_account(
                    payer.key, &vault_pda, mint.key,
                ),
                &[
                    payer.clone(),
                    vault_token_account_info.clone(),
                    pda.clone(),
                    mint.clone(),
                    system_program.clone(),
                    token_program.clone(),
                    rent_info.clone(),
                    atoken_program.clone(),
                ],
            )?;

            if *payer.key != admin || !payer.is_signer {
                //unauthorized access
                return Err(ProgramError::Custom(0x02));
            }

            let contract_data = VaultData {
                mint: *mint.key,
                min_period,
                reward_period,
                rate,
                early_withdrawal_fee,
                total_obligations: 0,
                total_staked: 0,
            };
            contract_data.serialize(&mut &mut pda.data.borrow_mut()[..])?;
        }
    };

    Ok(())
}
