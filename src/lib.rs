use solana_program::declare_id;

declare_id!("perpke6JybKfRDitCmnazpCrGN5JRApxxukhA9Js6E6");
mod accounts;
pub mod fp_math;
mod pubkey;
pub use bonfida_macros::{accounts, pubkey};
