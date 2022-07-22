use async_trait::async_trait;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_program_test::ProgramTestContext;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::{error::TestError, utils::sign_send_instructions};

#[async_trait]
pub trait ProgramTestContextExt {
    async fn mint_tokens(
        &mut self,
        mint_authority: &Keypair,
        mint_pubkey: &Pubkey,
        token_account: &Pubkey,
        amount: u64,
    ) -> Result<(), TestError>;

    async fn get_token_account(
        &mut self,
        key: Pubkey,
    ) -> Result<spl_token::state::Account, TestError>;
}

#[async_trait]
impl ProgramTestContextExt for ProgramTestContext {
    async fn mint_tokens(
        &mut self,
        mint_authority: &Keypair,
        mint_pubkey: &Pubkey,
        token_account: &Pubkey,
        amount: u64,
    ) -> Result<(), TestError> {
        let mint_instruction = spl_token::instruction::mint_to(
            &spl_token::ID,
            mint_pubkey,
            token_account,
            &mint_authority.pubkey(),
            &[],
            amount,
        )?;
        sign_send_instructions(self, vec![mint_instruction], &[mint_authority]).await?;
        Ok(())
    }
    async fn get_token_account(
        &mut self,
        key: Pubkey,
    ) -> Result<spl_token::state::Account, TestError> {
        let raw_account = self
            .banks_client
            .get_account(key)
            .await?
            .ok_or(TestError::AccountDoesNotExist)?;
        if raw_account.owner != spl_token::ID {
            return Err(TestError::InvalidTokenAccount);
        }
        Ok(spl_token::state::Account::unpack(&raw_account.data)?)
    }
}
