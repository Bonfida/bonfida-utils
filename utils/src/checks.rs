use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

// Safety verification functions
pub fn check_account_key(account: &AccountInfo, key: &Pubkey) -> ProgramResult {
    if account.key != key {
        msg!("Wrong account key: {} should be {}", account.key, key);
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_account_owner(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        msg!("Wrong account owner: {} should be {}", account.owner, owner);
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    if !(account.is_signer) {
        msg!("Missing signature for: {}", account.key);
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

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
