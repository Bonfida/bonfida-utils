use std::str::FromStr;

use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::account::Account;
use solana_sdk::signature::Signer;
use solana_sdk::{signature::Keypair, transaction::Transaction};
use spl_token::state::Mint;

/// Functional testing utils

pub async fn sign_send_instructions(
    ctx: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    signers: Vec<&Keypair>,
) -> Result<(), BanksClientError> {
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&ctx.payer.pubkey()));
    let mut payer_signers = vec![&ctx.payer];
    for s in signers {
        payer_signers.push(s);
    }
    transaction.partial_sign(&payer_signers, ctx.last_blockhash);
    ctx.banks_client.process_transaction(transaction).await
}

pub fn mint_bootstrap(
    address: Option<&str>,
    decimals: u8,
    program_test: &mut ProgramTest,
    mint_authority: &Pubkey,
) -> (Pubkey, Mint) {
    let address = address
        .map(|s| Pubkey::from_str(s).unwrap())
        .unwrap_or_else(Pubkey::new_unique);
    let mint_info = Mint {
        mint_authority: Some(*mint_authority).into(),
        supply: u32::MAX.into(),
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };
    let mut data = [0; Mint::LEN];
    mint_info.pack_into_slice(&mut data);
    program_test.add_account(
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

pub fn mint_user() {
    let mint_to_instr = mint_to(
        &spl_token::ID,
        &mint,
        &buyer_token_source,
        &mint_authority.pubkey(),
        &[],
        1_000_000,
    )
    .unwrap();
    sign_send_instructions(&mut ctx, vec![mint_to_instr], vec![&mint_authority])
        .await
        .unwrap();

}

pub async fn update_blockhash(&mut self) -> Result<(), BanksClientError> {
    self.prg_test_ctx.last_blockhash = self
        .prg_test_ctx
        .banks_client
        .get_latest_blockhash()
        .await?;
    Ok(())
}

pub fn create_and_get_associated_token_address(
    ctx: &ProgramTestContext,
    parent_key: &Pubkey,
    mint_key: &Pubkey,
) -> (Transaction, Pubkey) {
    let instruction = create_associated_token_account(&ctx.payer.pubkey(), parent_key, mint_key);
    let asset_key = get_associated_token_address(parent_key, mint_key);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&ctx.payer.pubkey()));
    transaction.partial_sign(&[&ctx.payer], ctx.last_blockhash);
    (transaction, asset_key)
}