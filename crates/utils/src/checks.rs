use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Account;
use std::panic::Location;

// Safety verification functions
#[track_caller]
pub fn check_account_key(account: &AccountInfo, key: &Pubkey) -> ProgramResult {
    if account.key != key {
        let caller = Location::caller();
        msg!(
            "Wrong account key: {} should be {} - File: {} - Line: {}",
            account.key,
            key,
            caller.file(),
            caller.line()
        );
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

#[track_caller]
pub fn check_account_owner(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        let caller = Location::caller();
        msg!(
            "Wrong account owner: {} should be {} - File: {} - Line: {}",
            account.owner,
            owner,
            caller.file(),
            caller.line()
        );
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

#[track_caller]
pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    if !(account.is_signer) {
        let caller = Location::caller();
        msg!(
            "Missing signature for: {} - File: {} - Line: {}",
            account.key,
            caller.file(),
            caller.line()
        );
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

#[track_caller]
pub fn check_account_derivation(
    account: &AccountInfo,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> Result<u8, ProgramError> {
    let (key, nonce) = Pubkey::find_program_address(seeds, program_id);
    check_account_key(account, &key)?;
    Ok(nonce)
}

pub fn check_rent_exempt(account: &AccountInfo) -> ProgramResult {
    let rent = Rent::get()?;
    if !rent.is_exempt(account.lamports(), account.data_len()) {
        return Err(ProgramError::AccountNotRentExempt);
    }
    Ok(())
}

#[track_caller]
pub fn check_token_account_owner(
    account: &AccountInfo,
    owner: &Pubkey,
) -> Result<Account, ProgramError> {
    check_account_owner(account, &spl_token::ID)?;
    let token_account = Account::unpack_from_slice(&account.data.borrow())?;
    if token_account.owner != *owner {
        let caller = Location::caller();
        msg!(
            "Wrong account owner: {} should be {} - File: {} - Line: {}",
            token_account.owner,
            owner,
            caller.file(),
            caller.line()
        );
        return Err(ProgramError::InvalidArgument);
    }
    Ok(token_account)
}
