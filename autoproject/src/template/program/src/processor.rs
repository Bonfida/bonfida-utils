use {
    borsh::BorshDeserialize,
    num_traits::FromPrimitive,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

use crate::instruction::ProgramInstruction;

pub mod example_instr;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[1..];
        msg!("Instruction unpacked");

        match instruction {
            ProgramInstruction::CreateMint => {
                msg!("Instruction: Create mint");
                create_mint::process(program_id, accounts)?;
            }
        }

        Ok(())
    }
}
