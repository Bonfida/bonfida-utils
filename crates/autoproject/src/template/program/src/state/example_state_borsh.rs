use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::error::Error;

#[derive(BorshSerialize, BorshDeserialize)]
#[allow(missing_docs)]
#[repr(C)]
pub struct ExampleStateBorsh {
    /// Nonce
    pub nonce: u8,
}

impl ExampleStateBorsh {
    pub const LEN: usize = std::mem::size_of::<Self>();
}

/// An example PDA state, serialized using Borsh //TODO
#[allow(missing_docs)]
impl ExampleStateBorsh {
    pub const SEED: &'static [u8; 12] = b"example_seed";

    pub fn from_buffer(buffer: &[u8], expected_tag: super::Tag) -> Result<Self, ProgramError> {
        let (tag, mut buffer) = buffer.split_at(8);
        if *bytemuck::from_bytes::<u64>(tag) != expected_tag as u64 {
            return Err(Error::DataTypeMismatch.into());
        }
        Ok(Self::deserialize(&mut buffer)?)
    }

    pub fn find_key(program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[Self::SEED];
        Pubkey::find_program_address(seeds, program_id)
    }

    pub fn save(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        self.serialize(&mut (&mut dst[8..]))?;
        Ok(())
    }
}
