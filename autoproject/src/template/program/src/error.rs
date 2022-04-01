use {
    num_derive::FromPrimitive,
    solana_program::{decode_error::DecodeError, program_error::ProgramError},
    thiserror::Error,
};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum OfferError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("Data type mismatch")]
    DataTypeMismatch,
    #[error("Wrong account owner")]
    WrongOwner,
    #[error("Account is uninitialized")]
    Uninitialized,
}

impl From<OfferError> for ProgramError {
    fn from(e: OfferError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for OfferError {
    fn type_of() -> &'static str {
        "OfferError"
    }
}
