use solana_program::declare_id;

declare_id!("perpke6JybKfRDitCmnazpCrGN5JRApxxukhA9Js6E6");
mod accounts;
mod borsh_size;
pub mod checks;
pub mod fp_math;
mod pubkey;
pub use accounts::InstructionsAccount;
pub use bonfida_macros::{pubkey, BorshSize, InstructionsAccount};
pub use borsh_size::BorshSize;
