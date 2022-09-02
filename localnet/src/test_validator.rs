use solana_core::validator::Validator;
use solana_program_test::ProgramTestError;
use solana_runtime::bank::Bank;
use solana_sdk::{
    clock::Slot,
    pubkey::Pubkey,
    sysvar::{Sysvar, SysvarId},
};

pub trait TestValidator {
    fn set_sysvar<T: SysvarId + Sysvar>(&self, sysvar: &T);
    fn warp_to_slot(&mut self, warp_slot: Slot) -> Result<(), ProgramTestError>;
}

impl TestValidator for Validator {
    fn set_sysvar<T: SysvarId + Sysvar>(&self, sysvar: &T) {
        let bank_forks = self.bank_forks.read().unwrap();
        let bank = bank_forks.working_bank();
        bank.set_sysvar_for_tests(sysvar);
    }

    fn warp_to_slot(&mut self, warp_slot: Slot) -> Result<(), ProgramTestError> {
        let mut bank_forks = self.bank_forks.write().unwrap();
        let bank = bank_forks.working_bank();

        // Fill ticks until a new blockhash is recorded, otherwise retried transactions will have
        // the same signature
        bank.fill_bank_with_ticks_for_tests();

        // Ensure that we are actually progressing forward
        let working_slot = bank.slot();
        if warp_slot <= working_slot {
            return Err(ProgramTestError::InvalidWarpSlot);
        }

        // Warp ahead to one slot *before* the desired slot because the bank
        // from Bank::warp_from_parent() is frozen. If the desired slot is one
        // slot *after* the working_slot, no need to warp at all.
        let pre_warp_slot = warp_slot - 1;
        let warp_bank = if pre_warp_slot == working_slot {
            bank.freeze();
            bank
        } else {
            bank_forks.insert(Bank::warp_from_parent(
                &bank,
                &Pubkey::default(),
                pre_warp_slot,
            ))
        };
        bank_forks.set_root(
            pre_warp_slot,
            &solana_runtime::accounts_background_service::AbsRequestSender::default(),
            Some(pre_warp_slot),
        );

        // warp_bank is frozen so go forward to get unfrozen bank at warp_slot
        bank_forks.insert(Bank::new_from_parent(
            &warp_bank,
            &Pubkey::default(),
            warp_slot,
        ));

        // // Update block commitment cache, otherwise banks server will poll at
        // // the wrong slot
        // let mut w_block_commitment_cache = self.block_commitment_cache.write().unwrap();
        // // HACK: The root set here should be `pre_warp_slot`, but since we're
        // // in a testing environment, the root bank never updates after a warp.
        // // The ticking thread only updates the working bank, and never the root
        // // bank.
        // w_block_commitment_cache.set_all_slots(warp_slot, warp_slot);
        Ok(())
    }
}
