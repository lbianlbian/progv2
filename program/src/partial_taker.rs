use borsh::{BorshDeserialize, BorshSerialize};
use crate::token;
use crate::utils;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn bet(
    mut bet_account: utils::BetAcc,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
    program_id: &Pubkey,
) -> ProgramResult {
    //get accounts
    let accounts_iter = &mut accounts.iter();

    let bet = next_account_info(accounts_iter)?;
    let _tok_prog = next_account_info(accounts_iter)?;
    let _source = next_account_info(accounts_iter)?;
    let destination = next_account_info(accounts_iter)?;
    let bettor = next_account_info(accounts_iter)?; //cant use free bet in taker order, so bettor = authority
    let rent_payer = next_account_info(accounts_iter)?;
    let new_bet = next_account_info(accounts_iter)?;

    if new_bet.owner != program_id {
        msg!("new bet doesn't belong to this program id");
        return Err(ProgramError::IncorrectProgramId);
    }
    if !utils::correct_pool((*destination).key.to_bytes()) {
        msg!("Incorrect pool");
        return Err(ProgramError::InvalidArgument);
    }
    //check instruction data for match with original acc
    if !utils::ids_match(&bet_account, instruction_data) {
        msg!("id information of bet and incoming matcher don't match");
        return Err(ProgramError::InvalidInstructionData);
    }
    //check new bet acc for blankness
    let mut new_bet_account = utils::BetAcc::try_from_slice(&new_bet.data.borrow())?;
    if !utils::blank_acc(&new_bet_account) {
        msg!("trying to start bet in non empty bet account");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    //check correct side
    let side = instruction_data[36];
    if (side == 0 && !utils::blank_wallet(bet_account.wallet0))
        || (side == 1 && !utils::blank_wallet(bet_account.wallet1))
    {
        msg!("trying to match bet on side that has already been matched");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if (bet_account.to_aggregate || bet_account.is_free_bet)
        && !utils::equal_wallets(bettor.key.to_bytes(), utils::PBMM)
    {
        msg!("not authorized to place a partial taker order on an existing unmatched free bet or bet marked for aggregation");
        return Err(ProgramError::InvalidAccountData);
    }

    //set id information of new acc based on original acc
    utils::set_bet_ids(&mut new_bet_account, instruction_data);
    //set new acc wallets
    if side == 0 {
        new_bet_account.wallet0 = bettor.key.to_bytes();
        new_bet_account.wallet1 = bet_account.wallet1;
    } else if side == 1 {
        new_bet_account.wallet1 = bettor.key.to_bytes();
        new_bet_account.wallet0 = bet_account.wallet0;
    }
    new_bet_account.rent_payer = rent_payer.key.to_bytes();

    //set new acc time, might be useful
    let clock = Clock::get()?;
    new_bet_account.placed_at = clock.unix_timestamp as u64;

    //handle stakes appropriately.
    let stake0 = utils::bytes_to_num(&instruction_data, 20, 28);
    let stake1 = utils::bytes_to_num(&instruction_data, 28, 36);
    let mut stake: u64 = 0;
    let mut odds: f64 = 0.0;
    let mut original_odds: f64 = 0.0;
    if side == 0 {
        odds = (stake0 + stake1) as f64 / stake0 as f64;
        original_odds =
            (bet_account.stake0 + bet_account.stake1) as f64 / bet_account.stake0 as f64;
        stake = stake0;
    } else if side == 1 {
        odds = (stake0 + stake1) as f64 / stake1 as f64;
        original_odds =
            (bet_account.stake0 + bet_account.stake1) as f64 / bet_account.stake1 as f64;
        stake = stake1;
    }
    if odds - original_odds > 0.01 {
        msg!("this bettor wants odds that are too high");
        return Err(ProgramError::InvalidInstructionData);
    }

    //ratio of original should stay the same,
    //old acc original bettor side decrements stakes by instr data,
    //and open side decreased by same ratio
    if side == 0 {
        let odds1 = (bet_account.stake0 + bet_account.stake1) as f64 / bet_account.stake1 as f64;
        bet_account.stake1 -= stake1;
        bet_account.stake0 = bet_account.stake1 * (odds1 - 1.0) as u64;
    } else if side == 1 {
        let odds0 = (bet_account.stake0 + bet_account.stake1) as f64 / bet_account.stake0 as f64;
        bet_account.stake0 -= stake0;
        bet_account.stake1 = bet_account.stake0 * (odds0 - 1.0) as u64;
    }

    //new acc gets stakes from instr data
    new_bet_account.stake0 = stake0;
    new_bet_account.stake1 = stake1;

    new_bet_account.to_aggregate = bet_account.to_aggregate;
    bet_account.to_aggregate = false;

    //send tokens
    let result = token::send(accounts, 2, 3, 4, 1, stake);
    match result {
        Ok(_result) => {
            bet_account.serialize(&mut *bet.data.borrow_mut())?;
            new_bet_account.serialize(&mut *new_bet.data.borrow_mut())?;
        }
        Err(err) => {
            return Err(err);
        }
    }

    Ok(())
}
