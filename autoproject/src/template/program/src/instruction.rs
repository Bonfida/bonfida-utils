pub use crate::processor::{
    create_collection, create_mint, create_nft, redeem_nft, withdraw_tokens,
};
use {
    bonfida_utils::InstructionsAccount,
    borsh::{BorshDeserialize, BorshSerialize},
    num_derive::FromPrimitive,
    solana_program::{instruction::Instruction, pubkey::Pubkey},
};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// Create the NFT mint
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The mint of the NFT           |
    /// | 1     | ✅        | ❌      | The domain name account       |
    /// | 2     | ❌        | ❌      | The central state account     |
    /// | 3     | ❌        | ❌      | The SPL token program account |
    /// | 4     | ❌        | ❌      | The system program account    |
    /// | 5     | ❌        | ❌      | Rent sysvar account           |
    /// | 6     | ❌        | ❌      | Fee payer account             |
    CreateMint,
}
#[allow(missing_docs)]
pub fn create_mint(
    accounts: create_mint::Accounts<Pubkey>,
    params: create_mint::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::CreateMint as u8, params)
}
