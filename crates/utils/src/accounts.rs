use crate::{borsh_size::BorshSize, WrappedPod};
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{bytes_of, NoUninit};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub trait InstructionsAccount {
    fn get_accounts_vec(&self) -> Vec<AccountMeta>;

    fn get_instruction<P: BorshDeserialize + BorshSerialize + BorshSize>(
        &self,
        program_id: Pubkey,
        instruction_id: u8,
        params: P,
    ) -> Instruction {
        let cap = 1 + params.borsh_len();
        let mut data = Vec::with_capacity(cap);
        #[allow(clippy::uninit_vec)]
        unsafe {
            data.set_len(cap);
        }
        data[0] = instruction_id;
        let mut data_pointer = &mut data[1..];
        params.serialize(&mut data_pointer).unwrap();
        // We check that we have written to the whole buffer to ensure that no undefined bytes remain at the end of data.
        if !data_pointer.is_empty() {
            panic!()
        }

        let accounts_vec = self.get_accounts_vec();
        Instruction {
            program_id,
            accounts: accounts_vec,
            data,
        }
    }
    fn get_instruction_cast<P: NoUninit>(
        &self,
        program_id: Pubkey,
        instruction_id: u8,
        params: P,
    ) -> Instruction {
        let cap = 8 + std::mem::size_of::<P>();
        let mut data = Vec::with_capacity(cap);
        data.push(instruction_id);
        data.extend([0; 7].iter());
        data.extend(bytes_of(&params));

        Instruction {
            program_id,
            accounts: self.get_accounts_vec(),
            data,
        }
    }
    fn get_instruction_wrapped_pod<'a, P: WrappedPod<'a>>(
        &self,
        program_id: Pubkey,
        instruction_id: u8,
        params: P,
    ) -> Instruction {
        let cap = 8 + params.size();
        let mut data = Vec::with_capacity(cap);
        data.push(instruction_id);
        data.extend([0; 7].iter());
        params.export(&mut data);

        Instruction {
            program_id,
            accounts: self.get_accounts_vec(),
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::InstructionsAccount;
    use crate::borsh_size::BorshSize;
    use bonfida_macros::{BorshSize, InstructionsAccount};
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::pubkey::Pubkey;

    #[cfg(feature = "instruction_params_casting")]
    use bytemuck::{Pod, Zeroable};

    mod accounts {
        use super::*;
        #[derive(InstructionsAccount, Clone)]
        pub struct Accounts<'a, T> {
            #[cons(writable)]
            pub a: &'a T,
            pub b: &'a T,
            #[cons(writable)]
            pub c: &'a [T],
            pub d: &'a [T],
            #[cons(writable, signer)]
            pub e: &'a T,
            #[cons(signer)]
            pub f: &'a T,
            #[cons(writable, signer)]
            pub g: &'a [T],
            #[cons(signer)]
            pub h: &'a [T],
            #[cons(signer)]
            pub i: Option<&'a T>,
        }
    }
    #[derive(BorshDeserialize, BorshSerialize, BorshSize, Clone, Copy)]
    #[cfg_attr(feature = "instruction_params_casting", derive(Zeroable, Pod))]
    #[repr(C)]
    pub struct Params {
        pub match_limit: u64,
    }

    mod accounts_with_vec {
        use super::*;
        #[derive(InstructionsAccount, Clone)]
        pub struct Accounts<'a, T> {
            #[cons(writable)]
            pub a: &'a T,
            pub b: &'a T,
            #[cons(writable)]
            pub c: &'a [T],
            pub d: &'a [T],
            #[cons(writable, signer)]
            pub e: &'a T,
            #[cons(signer)]
            pub f: &'a T,
            #[cons(writable, signer)]
            pub g: &'a [T],
            #[cons(signer)]
            pub h: &'a [T],
            #[cons(signer)]
            pub i: Vec<&'a T>,
        }
    }

    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn functional_0() {
        let i = Pubkey::new_unique();
        let a = accounts::Accounts {
            a: &Pubkey::new_unique(),
            b: &Pubkey::new_unique(),
            c: &[Pubkey::new_unique()],
            d: &[Pubkey::new_unique()],
            e: &Pubkey::new_unique(),
            f: &Pubkey::new_unique(),
            g: &[Pubkey::new_unique()],
            h: &[Pubkey::new_unique()],
            i: Some(&i),
        };
        let params = Params { match_limit: 46 };
        let instruction = a.get_instruction(crate::ID, 0, params);
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
        assert_eq!(instruction.accounts[8].is_writable, false);
        assert_eq!(instruction.accounts[8].is_signer, true);
        assert_eq!(instruction.accounts[8].pubkey, *a.i.unwrap());
        let mut instruction_data = vec![0];
        instruction_data.extend(&params.try_to_vec().unwrap());

        assert_eq!(instruction_data, instruction.data);

        #[cfg(feature = "instruction_params_casting")]
        {
            let instruction = a.get_instruction_cast(crate::ID, 0, params);
            let mut instruction_data = [0; 8].to_vec();
            instruction_data.extend(bytemuck::bytes_of(&params));

            assert_eq!(instruction_data, instruction.data);
        }
    }

    #[test]
    #[allow(clippy::bool_assert_comparison)]
    fn functional_1() {
        let i = (0..4).map(|_| Pubkey::new_unique()).collect::<Vec<_>>();
        let a = accounts_with_vec::Accounts {
            a: &Pubkey::new_unique(),
            b: &Pubkey::new_unique(),
            c: &[Pubkey::new_unique()],
            d: &[Pubkey::new_unique()],
            e: &Pubkey::new_unique(),
            f: &Pubkey::new_unique(),
            g: &[Pubkey::new_unique()],
            h: &[Pubkey::new_unique()],
            i: vec![&i[0], &i[1], &i[2], &i[3]],
        };
        let params = Params { match_limit: 46 };
        let instruction = a.get_instruction(crate::ID, 0, params);
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
        assert_eq!(instruction.accounts[8].is_writable, false);
        assert_eq!(instruction.accounts[8].is_signer, true);
        assert_eq!(instruction.accounts[8].pubkey, *a.i[0]);
        assert_eq!(instruction.accounts[9].is_writable, false);
        assert_eq!(instruction.accounts[9].is_signer, true);
        assert_eq!(instruction.accounts[9].pubkey, *a.i[1]);
        assert_eq!(instruction.accounts[10].is_writable, false);
        assert_eq!(instruction.accounts[10].is_signer, true);
        assert_eq!(instruction.accounts[10].pubkey, *a.i[2]);
        let mut instruction_data = vec![0];
        instruction_data.extend(&params.try_to_vec().unwrap());

        assert_eq!(instruction_data, instruction.data);

        #[cfg(feature = "instruction_params_casting")]
        {
            let instruction = a.get_instruction_cast(crate::ID, 0, params);
            let mut instruction_data = [0; 8].to_vec();
            instruction_data.extend(bytemuck::bytes_of(&params));

            assert_eq!(instruction_data, instruction.data);
        }
    }
}
