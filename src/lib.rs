#[cfg(test)]
use solana_program::declare_id;
#[cfg(test)]
declare_id!("LiquidationRecord11111111111111111111111111");

mod accounts;
mod borsh_size;
pub mod checks;
pub mod fp_math;
mod pubkey;
pub use accounts::InstructionsAccount;
pub use bonfida_macros::{pubkey, BorshSize, InstructionsAccount};
pub use borsh_size::BorshSize;
