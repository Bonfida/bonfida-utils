use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use crate::error::Error;

#[derive(Clone, Copy, Zeroable, Pod)]
#[allow(missing_docs)]
#[repr(C)]
pub struct ExampleStateCast {
    /// Nonce
    pub nonce: u8,
}

impl ExampleStateCast {
    pub const LEN: usize = std::mem::size_of::<Self>();
}

/// An example PDA state, serialized using Borsh //TODO
#[allow(missing_docs)]
impl ExampleStateCast {
    pub const SEED: &'static [u8; 12] = b"example_seed";

    pub fn initialize(buffer: &mut [u8]) -> Result<(), Error> {
        let (tag, _) = buffer.split_at_mut(8);
        let tag: &mut u64 = bytemuck::from_bytes_mut(tag);
        if *tag != super::Tag::Uninitialized as u64 {
            return Err(Error::DataTypeMismatch.into());
        }
        *tag = super::Tag::ExampleStateCast as u64;
        Ok(())
    }

    pub fn from_buffer(buffer: &mut [u8], expected_tag: super::Tag) -> Result<&mut Self, Error> {
        let (tag, buffer) = buffer.split_at_mut(8);
        if *bytemuck::from_bytes_mut::<u64>(tag) != expected_tag as u64 {
            return Err(Error::DataTypeMismatch.into());
        }
        Ok(bytemuck::from_bytes_mut(buffer))
    }

    pub fn find_key(program_id: &Pubkey) -> (Pubkey, u8) {
        let seeds: &[&[u8]] = &[Self::SEED];
        Pubkey::find_program_address(seeds, program_id)
    }
}
