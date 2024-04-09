use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult, 
    msg, 
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    program::invoke_signed,
};

use spl_token::instruction::transfer;
use spl_token::state::Account as TokenAccount;
use crate::utils;

const OFFICIAL_TOK_PROG: [u8; 32] = [
    6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 28, 180, 133, 237,
    95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169,
]; // could also use program id provdied in spl_token crate

pub fn are_paired(auth:[u8;32], tok:&AccountInfo) -> Result<bool, ProgramError>{
    let mut data: &[u8] = &tok.try_borrow_data()?;
    let acc = TokenAccount::unpack(&mut data)?;
    return Ok(utils::equal_wallets(auth, acc.owner.to_bytes()));
}

pub fn send(
    accounts: &[AccountInfo],
    source_ind: usize,
    dest_ind: usize,
    auth_ind: usize,
    tok_prog_ind: usize,
    amnt: u64,
) -> ProgramResult {
    msg!("transferring tokens");
    let source = &accounts[source_ind];
    let destination = &accounts[dest_ind];
    let authority = &accounts[auth_ind];
    let tok_prog = &accounts[tok_prog_ind];

    if !utils::equal_wallets((tok_prog).key.to_bytes(), OFFICIAL_TOK_PROG) {
        msg!("Incorrect token program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let ix = transfer(
        tok_prog.key,
        source.key,
        destination.key,
        authority.key,
        &[&authority.key],
        amnt,
    )?;
    invoke(
        &ix,
        &[
            source.clone(),
            destination.clone(),
            authority.clone(),
            tok_prog.clone(),
        ],
    )?;
    Ok(())
}

// seed and bump key for transfering tokens will be hard coded to program
pub fn send_out(
    accounts: &[AccountInfo],
    source_ind: usize,
    dest_ind: usize,
    auth_ind: usize,
    tok_prog_ind: usize,
    amnt: u64,
) -> ProgramResult {
    let source = &accounts[source_ind];
    let destination = &accounts[dest_ind];
    let authority = &accounts[auth_ind];
    let tok_prog = &accounts[tok_prog_ind];

    if !utils::equal_wallets((tok_prog).key.to_bytes(), OFFICIAL_TOK_PROG) {
        msg!("Incorrect token program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    let ix = transfer(
        tok_prog.key,
        source.key,
        destination.key,
        authority.key,
        &[&authority.key],
        amnt,
    )?;
    invoke_signed(
        &ix,
        &[
            source.clone(), 
            destination.clone(), 
            authority.clone(), 
            tok_prog.clone()
        ],
        &[ &[ b"pool", &[255]]] ,
    )?;
    Ok(())
}