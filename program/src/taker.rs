use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError
};
use crate::utils;
use crate::token;

pub fn bet(
    mut bet_account: utils::BetAcc,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("account being matched now");
    let accounts_iter = &mut accounts.iter();

    let bet = next_account_info(accounts_iter)?;
    let _tok_prog = next_account_info(accounts_iter)?;
    let _source = next_account_info(accounts_iter)?;
    let destination = next_account_info(accounts_iter)?;
    let bettor = next_account_info(accounts_iter)?; //cant use free bet in taker order, so bettor = authority
    if !utils::correct_pool((*destination).key.to_bytes()) {
        msg!("Incorrect pool");
        return Err(ProgramError::InvalidArgument);
    }
    // sport, league, event, period, mkt, player, in instr data need to be equal to those in acc(included for websocket ig although maybe not used)
    if !utils::ids_match(&bet_account, instruction_data){
        msg!("id information of bet and incoming matcher don't match");
        return Err(ProgramError::InvalidInstructionData);
    }
    // side must be correct
    let side = instruction_data[36];
    if side == 0 && !utils::blank_wallet(bet_account.wallet0)
        || side == 1 && !utils::blank_wallet(bet_account.wallet1)
    {
        msg!("trying to match bet on side that has already been matched");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let stake0 = utils::bytes_to_num(&instruction_data, 20, 28);
    let stake1 = utils::bytes_to_num(&instruction_data, 28, 36);
    
    // check odds are right
    if (side == 0 && (stake0 < bet_account.stake0 || stake1 != bet_account.stake1))
        || (side == 1 && (stake1 < bet_account.stake1 || stake0 != bet_account.stake0))
    {
        msg!("this bettor wants odds are too high or profit doesn't equal what has already been bet");
        return Err(ProgramError::InvalidInstructionData);
    }

    if (bet_account.to_aggregate || bet_account.is_free_bet) && !utils::equal_wallets(bettor.key.to_bytes(), utils::PBMM) {
        msg!("not authorized to place a taker order on an existing unmatched free bet or bet marked for aggregation");
        return Err(ProgramError::InvalidAccountData);
    }
    // if all checks pass, write to correct stake and wallet
    let mut stake: u64 = 0;
    if side == 0 {
        bet_account.stake0 = stake0;
        bet_account.wallet0 = bettor.key.to_bytes();
        stake = stake0;
    } else if side == 1 {
        bet_account.stake1 = stake1;
        bet_account.wallet1 = bettor.key.to_bytes();
        stake = stake1;
    }
    // transfer funds to pool
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