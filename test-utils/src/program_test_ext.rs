use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::ProgramTest;
use solana_sdk::account::Account;
use spl_token::state::Mint;

pub trait ProgramTestExt {
    fn add_mint(
        &mut self,
        key: Option<Pubkey>,
        decimals: u8,
        mint_authority: &Pubkey,
    ) -> (Pubkey, Mint);

    fn add_account_with_lamports(&mut self, key: Pubkey, lamports: u64);
}

impl ProgramTestExt for ProgramTest {
    fn add_mint(
        &mut self,
        key: Option<Pubkey>,
        decimals: u8,
        mint_authority: &Pubkey,
    ) -> (Pubkey, Mint) {
        let address = key.unwrap_or_else(Pubkey::new_unique);
        let mint_info = Mint {
            mint_authority: Some(*mint_authority).into(),
            supply: u32::MAX.into(),
            decimals,
            is_initialized: true,
            freeze_authority: None.into(),
        };
        let mut data = [0; Mint::LEN];
        mint_info.pack_into_slice(&mut data);
        self.add_account(
            address,
            Account {
                lamports: u32::MAX.into(),
                data: data.into(),
                owner: spl_token::ID,
                executable: false,
                ..Account::default()
            },
        );
        (address, mint_info)
    }

    fn add_account_with_lamports(&mut self, key: Pubkey, lamports: u64) {
        self.add_account(
            key,
            Account {
                lamports,
                ..Default::default()
            },
        );
    }
}
