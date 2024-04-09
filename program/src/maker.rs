use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    sysvar::Sysvar
};
use crate::utils;
use crate::token;

pub fn bet(
    mut bet_account: utils::BetAcc,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let bet = next_account_info(accounts_iter)?;
    let _tok_prog = next_account_info(accounts_iter)?;
    let _source = next_account_info(accounts_iter)?;
    let destination = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?; // not necessarily the bettor if using free bet
    let bettor = next_account_info(accounts_iter)?;
    let rent_payer = next_account_info(accounts_iter)?;

    if !utils::correct_pool((*destination).key.to_bytes()) {
        msg!("Incorrect pool");
        return Err(ProgramError::InvalidArgument);
    }
    if !utils::blank_acc(&bet_account){
        msg!("trying to start bet in non empty bet account");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    // set account values
    utils::set_bet_ids(&mut bet_account, instruction_data);
    bet_account.stake0 = utils::bytes_to_num(&instruction_data, 20, 28);
    bet_account.stake1 = utils::bytes_to_num(&instruction_data, 28, 36);

    let side = instruction_data[36];
    let stake: u64;
    if side == 0 {
        bet_account.wallet0 = (*bettor).key.to_bytes();
        stake = bet_account.stake0;
    } else {
        bet_account.wallet1 = (*bettor).key.to_bytes();
        stake = bet_account.stake1;
    }
    bet_account.rent_payer = (*rent_payer).key.to_bytes();
    bet_account.is_free_bet = !utils::equal_wallets((*authority).key.to_bytes(), (*bettor).key.to_bytes());
    bet_account.to_aggregate = instruction_data[37] == 1;

    let clock = Clock::get()?;
    bet_account.placed_at = clock.unix_timestamp as u64;
    // call another function to send the correct tokens to the correct address
    // need to check return value of this for error and not run the below line if sending tokens errors
    let result = token::send(accounts, 2, 3, 4, 1, stake);
    match result {
        Ok(_result) => {
            bet_account.serialize(&mut *bet.data.borrow_mut())?;
        }
        Err(err) => {
            return Err(err);
        }
    }

    Ok(())
}