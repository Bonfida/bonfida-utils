use crate::error::TOBEREPLACEDBY_PASCALError;

use {
    bonfida_utils::BorshSize,
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey},
};

#[derive(BorshSerialize, BorshDeserialize, BorshSize, PartialEq)]
#[allow(missing_docs)]
pub enum Tag {
    Uninitialized,
    Initialized,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
#[allow(missing_docs)]
pub struct ExampleState {
    /// Tag
    pub tag: Tag,

    /// Nonce
    pub nonce: u8,
    //...
}

/// An example PDA state, serialized using Borsh //TODO
#[allow(missing_docs)]
impl ExampleState {
    pub const SEED: &'static [u8; 12] = b"example_seed";

    pub fn new(nonce: u8) -> Self {
        Self {
            tag: Tag::Initialized,
            nonce,
        }
    }

    pub fn find_key(program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[ExampleState::SEED];
        Pubkey::find_program_address(seeds, program_id)
    }

    pub fn save(&self, mut dst: &mut [u8]) {
        self.serialize(&mut dst).unwrap()
    }

    pub fn from_account_info(a: &AccountInfo, tag: Tag) -> Result<ExampleState, ProgramError> {
        let mut data = &a.data.borrow() as &[u8];
        if data[0] != tag as u8 && data[0] != Tag::Uninitialized as u8 {
            return Err(TOBEREPLACEDBY_PASCALError::DataTypeMismatch.into());
        }
        let result = ExampleState::deserialize(&mut data)?;
        Ok(result)
    }

    pub fn is_initialized(&self) -> bool {
        self.tag == Tag::Initialized
    }
}
