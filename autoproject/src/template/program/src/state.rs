use crate::error::TOBEREPLACEDBY_PASCALError;

use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

pub mod example_state_borsh;
pub mod example_state_cast;

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    ExampleStateCast,
    ExampleStateBorsh,
}
