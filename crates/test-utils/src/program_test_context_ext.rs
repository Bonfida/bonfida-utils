use async_trait::async_trait;
use solana_program::{
    clock::Clock, instruction::Instruction, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    system_instruction::create_account,
};
use solana_program_test::ProgramTestContext;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

use crate::error::TestError;

const NANOSECONDS_IN_SECOND: u128 = 1_000_000_000;

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

    async fn sign_send_instructions(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<(), TestError>;

    async fn warp_to_timestamp(&mut self, timestamp: i64) -> Result<(), TestError>;
    async fn initialize_token_accounts(
        &mut self,
        mint: Pubkey,
        owners: &[Pubkey],
    ) -> Result<Vec<Pubkey>, TestError>;

    async fn get_current_timestamp(&mut self) -> Result<i64, TestError>;

    async fn initialize_new_account(
        &mut self,
        space: usize,
        program_id: Pubkey,
    ) -> Result<Pubkey, TestError>;
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
        self.sign_send_instructions(&[mint_instruction], &[mint_authority])
            .await?;
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
    async fn sign_send_instructions(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<(), TestError> {
        let mut transaction = Transaction::new_with_payer(instructions, Some(&self.payer.pubkey()));
        let mut payer_signers = Vec::with_capacity(1 + signers.len());
        payer_signers.push(&self.payer);
        for s in signers {
            payer_signers.push(s);
        }
        transaction.partial_sign(&payer_signers, self.last_blockhash);
        self.banks_client.process_transaction(transaction).await?;
        Ok(())
    }

    async fn warp_to_timestamp(&mut self, timestamp: i64) -> Result<(), TestError> {
        let mut clock = self.banks_client.get_sysvar::<Clock>().await?;
        if clock.unix_timestamp > timestamp {
            return Err(TestError::InvalidTimestampForWarp);
        }
        let ns_per_slot = self.genesis_config().ns_per_slot();
        let time_delta_ns =
            (timestamp - clock.unix_timestamp).unsigned_abs() as u128 * NANOSECONDS_IN_SECOND;
        let number_of_slots_to_warp: u64 = (time_delta_ns / ns_per_slot).try_into().unwrap();

        clock.unix_timestamp = timestamp;
        self.set_sysvar(&clock);
        self.warp_to_slot(clock.slot + number_of_slots_to_warp)?;
        Ok(())
    }

    async fn initialize_token_accounts(
        &mut self,
        mint: Pubkey,
        owners: &[Pubkey],
    ) -> Result<Vec<Pubkey>, TestError> {
        let mut instructions = Vec::with_capacity(owners.len());
        let mut account_keys = Vec::with_capacity(owners.len());
        for o in owners {
            let i = spl_associated_token_account::instruction::create_associated_token_account(
                &self.payer.pubkey(),
                o,
                &mint,
                &spl_token::ID,
            );
            account_keys.push(i.accounts[1].pubkey);
            instructions.push(i);
        }
        for c in instructions.chunks(10) {
            self.sign_send_instructions(c, &[]).await?;
        }
        Ok(account_keys)
    }

    async fn get_current_timestamp(&mut self) -> Result<i64, TestError> {
        let clock = self.banks_client.get_sysvar::<Clock>().await?;
        Ok(clock.unix_timestamp)
    }

    async fn initialize_new_account(
        &mut self,
        space: usize,
        program_id: Pubkey,
    ) -> Result<Pubkey, TestError> {
        let account_keypair = Keypair::new();
        let rent = self.banks_client.get_sysvar::<Rent>().await.unwrap();
        let lamports = rent.minimum_balance(space);
        let ix = create_account(
            &self.payer.pubkey(),
            &account_keypair.pubkey(),
            lamports,
            space as u64,
            &program_id,
        );
        self.sign_send_instructions(&[ix], &[&account_keypair])
            .await?;
        Ok(account_keypair.pubkey())
    }
}
