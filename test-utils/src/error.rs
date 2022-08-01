use solana_program::program_error::ProgramError;
use solana_program_test::{BanksClientError, ProgramTestError};

#[derive(Debug)]
pub enum TestError {
    BanksClientError(BanksClientError),
    ProgramError(ProgramError),
    ProgramTestError(ProgramTestError),
    AccountDoesNotExist,
    InvalidTokenAccount,
    InvalidTimestampForWarp,
}

impl From<BanksClientError> for TestError {
    fn from(e: BanksClientError) -> Self {
        Self::BanksClientError(e)
    }
}

impl From<ProgramError> for TestError {
    fn from(e: ProgramError) -> Self {
        Self::ProgramError(e)
    }
}

impl From<ProgramTestError> for TestError {
    fn from(e: ProgramTestError) -> Self {
        Self::ProgramTestError(e)
    }
}
