use solana_program::program_error::ProgramError;
use solana_program_test::BanksClientError;

pub enum TestError {
    BanksClientError(BanksClientError),
    ProgramError(ProgramError),
    AccountDoesNotExist,
    InvalidTokenAccount,
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
