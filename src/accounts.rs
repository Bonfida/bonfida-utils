pub use bonfida_macros::accounts;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;

#[accounts()]
#[derive(Clone)]
pub struct Accounts<'a, 'b: 'a> {
    #[cons(writable)]
    a: &'a AccountInfo<'b>,
    b: &'a AccountInfo<'b>,
    #[cons(writable)]
    c: &'a [AccountInfo<'b>],
    d: &'a [AccountInfo<'b>],
    #[cons(writable, signer)]
    e: &'a AccountInfo<'b>,
    #[cons(signer)]
    f: &'a AccountInfo<'b>,
    #[cons(writable, signer)]
    g: &'a [AccountInfo<'b>],
    #[cons(signer)]
    h: &'a [AccountInfo<'b>],
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Params {
    pub match_limit: u64,
}

#[cfg(test)]
mod tests {
    use super::get_instruction;
    use super::AccountKeys;
    use super::Params;
    use solana_program::pubkey::Pubkey;
    #[test]
    fn functional_0() {
        let a = AccountKeys {
            a: Pubkey::new_unique(),
            b: Pubkey::new_unique(),
            c: &[Pubkey::new_unique()],
            d: &[Pubkey::new_unique()],
            e: Pubkey::new_unique(),
            f: Pubkey::new_unique(),
            g: &[Pubkey::new_unique()],
            h: &[Pubkey::new_unique()],
        };
        let instruction = get_instruction(0, a.clone(), Params { match_limit: 46 });
        assert_eq!(instruction.accounts[0].is_writable, true);
        assert_eq!(instruction.accounts[0].is_signer, false);
        assert_eq!(instruction.accounts[0].pubkey, a.a);
        assert_eq!(instruction.accounts[1].is_writable, false);
        assert_eq!(instruction.accounts[1].is_signer, false);
        assert_eq!(instruction.accounts[1].pubkey, a.b);
        assert_eq!(instruction.accounts[2].is_writable, true);
        assert_eq!(instruction.accounts[2].is_signer, false);
        assert_eq!(instruction.accounts[2].pubkey, a.c[0]);
        assert_eq!(instruction.accounts[3].is_writable, false);
        assert_eq!(instruction.accounts[3].is_signer, false);
        assert_eq!(instruction.accounts[3].pubkey, a.d[0]);

        assert_eq!(instruction.accounts[4].is_writable, true);
        assert_eq!(instruction.accounts[4].is_signer, true);
        assert_eq!(instruction.accounts[4].pubkey, a.e);
        assert_eq!(instruction.accounts[5].is_writable, false);
        assert_eq!(instruction.accounts[5].is_signer, true);
        assert_eq!(instruction.accounts[5].pubkey, a.f);
        assert_eq!(instruction.accounts[6].is_writable, true);
        assert_eq!(instruction.accounts[6].is_signer, true);
        assert_eq!(instruction.accounts[6].pubkey, a.g[0]);
        assert_eq!(instruction.accounts[7].is_writable, false);
        assert_eq!(instruction.accounts[7].is_signer, true);
        assert_eq!(instruction.accounts[7].pubkey, a.h[0]);
    }
}
