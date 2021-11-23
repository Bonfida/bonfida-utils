use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::Instruction;

use crate::borsh_size::BorshSize;

pub trait InstructionsAccount {
    fn get_instruction<P: BorshDeserialize + BorshSerialize + BorshSize>(
        &self,
        instruction_id: u8,
        params: P,
    ) -> Instruction;
}

#[cfg(test)]
mod tests {
    use super::InstructionsAccount;
    use crate::borsh_size::BorshSize;
    use bonfida_macros::{BorshSize, InstructionsAccount};
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::pubkey::Pubkey;
    #[test]
    fn functional_0() {
        #[derive(InstructionsAccount, Clone)]
        pub struct Accounts<'a, T> {
            #[cons(writable)]
            a: &'a T,
            b: &'a T,
            #[cons(writable)]
            c: &'a [T],
            d: &'a [T],
            #[cons(writable, signer)]
            e: &'a T,
            #[cons(signer)]
            f: &'a T,
            #[cons(writable, signer)]
            g: &'a [T],
            #[cons(signer)]
            h: &'a [T],
        }

        #[derive(BorshDeserialize, BorshSerialize, BorshSize, Clone)]
        pub struct Params {
            pub match_limit: u64,
        }
        let a = Accounts {
            a: &Pubkey::new_unique(),
            b: &Pubkey::new_unique(),
            c: &[Pubkey::new_unique()],
            d: &[Pubkey::new_unique()],
            e: &Pubkey::new_unique(),
            f: &Pubkey::new_unique(),
            g: &[Pubkey::new_unique()],
            h: &[Pubkey::new_unique()],
        };
        let params = Params { match_limit: 46 };
        let instruction = a.get_instruction(0, params.clone());
        assert_eq!(instruction.accounts[0].is_writable, true);
        assert_eq!(instruction.accounts[0].is_signer, false);
        assert_eq!(instruction.accounts[0].pubkey, *a.a);
        assert_eq!(instruction.accounts[1].is_writable, false);
        assert_eq!(instruction.accounts[1].is_signer, false);
        assert_eq!(instruction.accounts[1].pubkey, *a.b);
        assert_eq!(instruction.accounts[2].is_writable, true);
        assert_eq!(instruction.accounts[2].is_signer, false);
        assert_eq!(instruction.accounts[2].pubkey, a.c[0]);
        assert_eq!(instruction.accounts[3].is_writable, false);
        assert_eq!(instruction.accounts[3].is_signer, false);
        assert_eq!(instruction.accounts[3].pubkey, a.d[0]);

        assert_eq!(instruction.accounts[4].is_writable, true);
        assert_eq!(instruction.accounts[4].is_signer, true);
        assert_eq!(instruction.accounts[4].pubkey, *a.e);
        assert_eq!(instruction.accounts[5].is_writable, false);
        assert_eq!(instruction.accounts[5].is_signer, true);
        assert_eq!(instruction.accounts[5].pubkey, *a.f);
        assert_eq!(instruction.accounts[6].is_writable, true);
        assert_eq!(instruction.accounts[6].is_signer, true);
        assert_eq!(instruction.accounts[6].pubkey, a.g[0]);
        assert_eq!(instruction.accounts[7].is_writable, false);
        assert_eq!(instruction.accounts[7].is_signer, true);
        assert_eq!(instruction.accounts[7].pubkey, a.h[0]);

        let mut instruction_data = vec![0];
        instruction_data.extend(&params.try_to_vec().unwrap());

        assert_eq!(instruction_data, instruction.data);
    }
}
