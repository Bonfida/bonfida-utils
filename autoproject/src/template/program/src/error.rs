use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum TOBEREPLACEDBY_PASCALError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
}

impl From<TOBEREPLACEDBY_PASCALError> for ProgramError {
    fn from(e: TOBEREPLACEDBY_PASCALError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TOBEREPLACEDBY_PASCALError {
    fn type_of() -> &'static str {
        "TOBEREPLACEDBY_PASCALError"
    }
}
