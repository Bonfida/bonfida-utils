pub use crate::processor::example_instr;
use {
    bonfida_utils::InstructionsAccount,
    borsh::{BorshDeserialize, BorshSerialize},
    num_derive::FromPrimitive,
    solana_program::{instruction::Instruction, pubkey::Pubkey},
};
#[allow(missing_docs)]
#[derive(BorshDeserialize, BorshSerialize, FromPrimitive)]
pub enum ProgramInstruction {
    /// An example instruction //TODO
    ///
    /// | Index | Writable | Signer | Description                   |
    /// | --------------------------------------------------------- |
    /// | 0     | ❌        | ❌      | The system program account    |
    /// | 1     | ❌        | ❌      | The SPL token program account |
    /// | 2     | ✅        | ✅      | Fee payer account             |
    ExampleInstr,
}
#[allow(missing_docs)]
pub fn create_mint(
    accounts: example_instr::Accounts<Pubkey>,
    params: example_instr::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, ProgramInstruction::ExampleInstr as u8, params)
}
