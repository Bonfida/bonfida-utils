//! Example instruction //TODO

use bonfida_utils::checks::check_account_owner;

use crate::state::{
    example_state_borsh::ExampleStateBorsh, example_state_cast::ExampleStateCast, Tag,
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    /// An example input parameter
    pub example: String,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    #[cons(writable, signer)]
    /// The fee payer account
    pub fee_payer: &'a T,

    #[cons(writable)]
    /// The example state cast account //TODO
    pub example_state_cast: &'a T,

    #[cons(writable)]
    /// The example state cast account //TODO
    pub example_state_borsh: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            fee_payer: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            example_state_cast: next_account_info(accounts_iter)?,
            example_state_borsh: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;

        // Check owners
        check_account_owner(accounts.example_state_cast, program_id)?;
        check_account_owner(accounts.example_state_borsh, program_id)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    // Verify the example state account
    let (example_state_key, _) = ExampleStateCast::find_key(program_id);
    check_account_key(accounts.example_state_cast, &example_state_key)?;

    let mut example_state_cast_guard = accounts.example_state_cast.data.borrow_mut();

    let example_state_cast =
        ExampleStateCast::from_buffer(&mut example_state_cast_guard, Tag::ExampleStateCast)?;

    let mut example_state_borsh_guard = accounts.example_state_borsh.data.borrow_mut();

    let example_state_borsh =
        ExampleStateBorsh::from_buffer(&mut example_state_borsh_guard, Tag::ExampleStateBorsh)?;

    //...

    // Update example state account
    example_state_borsh.save(&mut example_state_borsh_guard);

    Ok(())
}
