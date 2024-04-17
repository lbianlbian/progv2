use crate::token;
use crate::utils;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    sysvar::Sysvar,
    pubkey::Pubkey,
};
use borsh::BorshDeserialize;

pub fn bet(
    bet_account: utils::BetAcc,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    program_id: &Pubkey,
    is_refund: bool
) -> ProgramResult {
    //get accounts
    let accounts_iter = &mut accounts.iter();

    let bet = next_account_info(accounts_iter)?;
    let _tok_prog = next_account_info(accounts_iter)?;
    let _source = next_account_info(accounts_iter)?;
    let destination = next_account_info(accounts_iter)?;
    let bettorOrRefunder = next_account_info(accounts_iter)?; //cant use free bet in taker order, so bettor = authority
    let rent_payer = next_account_info(accounts_iter)?;
    let _pda = next_account_info(accounts_iter)?;
    
    // check that instruction data (will just be id info with side) matches bet info
    if !utils::ids_match(&bet_account, instruction_data) {
        msg!("id information of bet and instruction data don't match");
        return Err(ProgramError::InvalidInstructionData);
    }
    let side = instruction_data[20];
    if (side == 0 && utils::blank_wallet(bet_account.wallet0))
        || (side == 1 && utils::blank_wallet(bet_account.wallet1))
    {
        msg!("trying to cancel wrong side of bet");
        return Err(ProgramError::InvalidInstructionData);
    }
    // make sure the bettor signed the tx so people can't cancel other people's bets
    if !bettorOrRefunder.is_signer {
        msg!("bettor isn't signing");
        return Err(ProgramError::InvalidArgument);
    }
    if is_refund && !utils::equal_wallets(bettorOrRefunder.key.to_bytes(), utils::ADMIN){
        msg!("refunding must be done by admin");
        return Err(ProgramError::InvalidArgument);
    }
    
    let mut bettor:[u8;32];
    if utils::blank_wallet(bet_account.wallet0){
        bettor = bet_account.wallet1;
    }
    else{
        bettor = bet_account.wallet0;
    }
    if !is_refund && !utils::equal_wallets(bettorOrRefunder.key.to_bytes(), bettor){
        msg!("not correct bettor canceling");
        return Err(ProgramError::InvalidArgument);
    }
    // if it is a free bet, return the usdc to the rent_payer, otherwise bettor
    if !bet_account.is_free_bet{
        
        if !token::are_paired(bettor.key.to_bytes(), destination)?{
            msg!("wrong associated token account");
            return Err(ProgramError::InvalidArgument);
        }
    }   
    if bet_account.is_free_bet{
        if !token::are_paired(rent_payer.key.to_bytes(), destination)?{
            msg!("wrong associated token account");
            return Err(ProgramError::InvalidArgument);
        }
    }
    if bet_account.to_aggregate{ // only appears if canceling a to aggregate account
        //need to also allow 7 acc cancelation instr if the bet isn't to_aggregate
        let delay_storage: &AccountInfo = next_account_info(accounts_iter)?;
        // check that current time is at least delay seconds later than placed_at
        let delay_acc = utils::CancelDelay::try_from_slice(&delay_storage.data.borrow())?;
        if !delay_acc.is_real{
            // fake delay acc
            return Err(ProgramError::InvalidAccountData);
        }
        if delay_storage.owner != program_id{
            // counterfeit delay acc from another program
            return Err(ProgramError::IncorrectProgramId);
        }
        let clock = Clock::get()?;
        let curr_time = clock.unix_timestamp as u64;
        if !is_refund && bet_account.to_aggregate && curr_time - (delay_acc.seconds as u64) < bet_account.placed_at {
            msg!("too early to cancel");
            return Err(ProgramError::InvalidAccountData);
        }
    }
    
    // send stake0 or stake1 usdc depending on side
    let mut stake:u64 = 0;
    if utils::blank_wallet(bet_account.wallet0){
        stake = bet_account.stake1;
    }
    else if utils::blank_wallet(bet_account.wallet1){
        stake = bet_account.stake0;
    }
    
    let result = token::send_out(accounts, 2, 3, 6, 1, stake);
    match result {
        Ok(_result) => {
            // refund lamports to rent payer
            let total_lamports = **bet.try_borrow_lamports()?;
            **bet.try_borrow_mut_lamports()? -= total_lamports;
            **rent_payer.try_borrow_mut_lamports()? += total_lamports;
        }
        Err(err) => {
            return Err(err);
        }
    }
    Ok(())
}

use borsh::BorshSerialize;

pub fn set_delay (
    mut delay_acc: utils::CancelDelay,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    
    let accounts_iter = &mut accounts.iter();

    let delay_storage = next_account_info(accounts_iter)?;
    let admin = next_account_info(accounts_iter)?;

    // check signer
    if !admin.is_signer || !utils::equal_wallets(admin.key.to_bytes(), utils::ADMIN){
        msg!("only the admin key can update the cancelation delay acc");
        return Err(ProgramError::InvalidArgument);
    }

    // if passed, update botha ttrs of delay acc
    delay_acc.is_real = true;
    delay_acc.seconds = instruction_data[0];
    delay_acc.serialize(&mut *delay_storage.data.borrow_mut())?;
    Ok(())
}
