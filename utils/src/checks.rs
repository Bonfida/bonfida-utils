use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

// Safety verification functions
pub fn check_account_key(account: &AccountInfo, key: &Pubkey) -> ProgramResult {
    if account.key != key {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_account_owner(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    if !(account.is_signer) {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

pub fn check_account_derivation(
    account: &AccountInfo,
    seeds: &[&[u8]],
    program_id: &Pubkey,
) -> ProgramResult {
    let (key, _) = Pubkey::find_program_address(seeds, program_id);
    check_account_key(account, &key)?;
    Ok(())
}
