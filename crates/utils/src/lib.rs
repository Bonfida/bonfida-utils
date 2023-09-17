#[cfg(test)]
use solana_program::declare_id;
#[cfg(test)]
declare_id!("11111111111111111111111111111111");

mod accounts;
mod borsh_size;
pub mod checks;
mod compute_record_hash;
mod declare_id_with_central_state;
mod wrapped_pod;

pub mod fp_math;
pub mod pyth;
pub use accounts::InstructionsAccount;
pub use bonfida_macros::{
    compute_hashv as compute_record_hash, declare_id_with_central_state, BorshSize,
    InstructionsAccount, WrappedPod, WrappedPodMut,
};
pub use borsh_size::BorshSize;
pub use wrapped_pod::{WrappedPod, WrappedPodMut};

#[cfg(feature = "benchmarking")]
pub mod bench;
