use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
pub mod cancel;
pub mod maker;
pub mod partial_taker;
pub mod taker;
pub mod token;
pub mod utils;

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Iterating accounts is safer than indexing
    let accounts_iter = &mut accounts.iter();

    // call it bet even though it could be the cancelation delay account
    let bet = next_account_info(accounts_iter)?;
    // The account must be owned by the program in order to modify its data
    if bet.owner != program_id {
        msg!("bet doesn't belong to this program id");
        return Err(ProgramError::IncorrectProgramId);
    }
    if accounts.len() == 2 {
        let delay_acc = utils::CancelDelay::try_from_slice(&bet.data.borrow())?;
        return cancel::set_delay(delay_acc, accounts, instruction_data);
    }

    let bet_account = utils::BetAcc::try_from_slice(&bet.data.borrow())?;
    let wallet0_is_blank = utils::blank_wallet(bet_account.wallet0);
    let wallet1_is_blank = utils::blank_wallet(bet_account.wallet1);
    let mut result = Ok(());

    if wallet0_is_blank && wallet1_is_blank {
        msg!("Thank you for betting with Purebet!");
        //start bet
        result = maker::bet(bet_account, accounts, instruction_data);
    } else if wallet0_is_blank || wallet1_is_blank {
        if instruction_data.len() == 37 {
            // match bet or partial match (differentiate by num of accs)
            msg!("Thank you for betting with Purebet");
            if accounts.len() == 5 {
                result = taker::bet(bet_account, accounts, instruction_data);
            } else if accounts.len() == 7 {
                msg!("got to call to partial taker without issues");
                result = partial_taker::bet(bet_account, accounts, instruction_data, program_id);
            }
        } else {
            if accounts.len() == 7 || accounts[7].data_len() == 2 {
                result = cancel::bet(bet_account, accounts, instruction_data, program_id);
            } else if accounts.len() == 8 {
                result = Ok(());
            } else {
                result = Err(ProgramError::InvalidArgument);
            }
        }
    // or cancel or refund (differentiate by num of accs)
    //need to remember to check bettor assoc tok belongs to bettor
    } else {
        //grade or push (differentiate by number of accs)
        //need to remember to check bettor assoc tok belongs to bettor
        result = Ok(());
    }

    result
}
