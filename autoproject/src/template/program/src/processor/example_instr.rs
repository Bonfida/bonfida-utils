//! Create a verified collection

use crate::{
    cpi::Cpi,
    state::{COLLECTION_NAME, COLLECTION_PREFIX, COLLECTION_URI, META_SYMBOL},
};

use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_token_metadata::{
        instruction::{create_master_edition_v3, create_metadata_accounts_v2},
        pda::{find_master_edition_account, find_metadata_account},
        state::Creator,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        system_program, sysvar,
    },
    spl_associated_token_account::create_associated_token_account,
    spl_token::{
        instruction::{initialize_mint, mint_to},
        state::Mint,
    },
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The mint of the collection
    #[cons(writable)]
    pub collection_mint: &'a T,

    #[cons(writable)]
    pub edition: &'a T,

    /// The metadata account
    #[cons(writable)]
    pub metadata_account: &'a T,

    /// The central state account
    pub central_state: &'a T,

    #[cons(writable)]
    /// Token account of the central state to hold the master edition
    pub central_state_nft_ata: &'a T,

    /// The fee payer account
    pub fee_payer: &'a T,

    /// The SPL token program account
    pub spl_token_program: &'a T,

    /// The metadata program account
    pub metadata_program: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The SPL name service program account
    pub spl_name_service_program: &'a T,

    pub ata_program: &'a T,

    /// Rent sysvar account
    pub rent_account: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            collection_mint: next_account_info(accounts_iter)?,
            edition: next_account_info(accounts_iter)?,
            metadata_account: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            central_state_nft_ata: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            metadata_program: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            spl_name_service_program: next_account_info(accounts_iter)?,
            ata_program: next_account_info(accounts_iter)?,
            rent_account: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        check_account_key(accounts.spl_token_program, &spl_token::ID)?;
        check_account_key(accounts.metadata_program, &mpl_token_metadata::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.spl_name_service_program, &spl_name_service::ID)?;
        check_account_key(accounts.ata_program, &spl_associated_token_account::ID)?;
        check_account_key(accounts.rent_account, &sysvar::rent::ID)?;

        // Check owners
        check_account_owner(accounts.collection_mint, &system_program::ID)?;
        check_account_owner(accounts.edition, &system_program::ID)?;
        check_account_owner(accounts.metadata_account, &system_program::ID)?;
        check_account_owner(accounts.central_state_nft_ata, &system_program::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let (collection_mint, collection_mint_nonce) =
        Pubkey::find_program_address(&[COLLECTION_PREFIX, &program_id.to_bytes()], program_id);
    check_account_key(accounts.collection_mint, &collection_mint)?;

    let (metadata_key, _) = find_metadata_account(&collection_mint);
    check_account_key(accounts.metadata_account, &metadata_key)?;

    let (edition_key, _) = find_master_edition_account(&collection_mint);
    check_account_key(accounts.edition, &edition_key)?;

    // Create mint account
    msg!("+ Creating mint");
    let seeds: &[&[u8]] = &[
        COLLECTION_PREFIX,
        &program_id.to_bytes(),
        &[collection_mint_nonce],
    ];
    Cpi::create_account(
        &spl_token::ID,
        accounts.system_program,
        accounts.fee_payer,
        &accounts.collection_mint.clone(),
        seeds,
        Mint::LEN,
    )?;
    msg!("+ Initialize mint");
    // Initialize mint
    let ix = initialize_mint(
        &spl_token::ID,
        &collection_mint,
        &crate::central_state::KEY,
        Some(&crate::central_state::KEY),
        0,
    )?;
    invoke_signed(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.collection_mint.clone(),
            accounts.rent_account.clone(),
        ],
        &[seeds],
    )?;

    // Create central state ATA
    msg!("+ Creating central state ATA");
    let ix = create_associated_token_account(
        accounts.fee_payer.key,
        &crate::central_state::KEY,
        &collection_mint,
    );
    invoke(
        &ix,
        &[
            accounts.ata_program.clone(),
            accounts.fee_payer.clone(),
            accounts.central_state_nft_ata.clone(),
            accounts.central_state.clone(),
            accounts.collection_mint.clone(),
            accounts.system_program.clone(),
            accounts.spl_token_program.clone(),
            accounts.rent_account.clone(),
        ],
    )?;

    // Mint NFT
    // (because the master edition ix requires mint supply === 1)
    msg!("+ Minting NFT");
    let seeds: &[&[u8]] = &[&program_id.to_bytes(), &[crate::central_state::NONCE]];
    let ix = mint_to(
        &spl_token::ID,
        &collection_mint,
        accounts.central_state_nft_ata.key,
        &crate::central_state::KEY,
        &[],
        1,
    )?;

    invoke_signed(
        &ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.collection_mint.clone(),
            accounts.central_state_nft_ata.clone(),
            accounts.central_state.clone(),
        ],
        &[seeds],
    )?;

    // Create collection
    msg!("+ Creating collection");
    let central_creator = Creator {
        address: crate::central_state::KEY,
        verified: true,
        share: 100,
    };
    let ix = create_metadata_accounts_v2(
        mpl_token_metadata::ID,
        *accounts.metadata_account.key,
        collection_mint,
        crate::central_state::KEY,
        *accounts.fee_payer.key,
        crate::central_state::KEY,
        COLLECTION_NAME.to_string(),
        META_SYMBOL.to_string(),
        COLLECTION_URI.to_string(),
        Some(vec![central_creator]),
        0,
        true,
        true,
        None,
        None,
    );

    invoke_signed(
        &ix,
        &[
            accounts.metadata_program.clone(),
            accounts.metadata_account.clone(),
            accounts.rent_account.clone(),
            accounts.collection_mint.clone(),
            accounts.central_state.clone(),
            accounts.fee_payer.clone(),
        ],
        &[seeds],
    )?;

    // Create master edition
    msg!("+ Creating master edition");
    let ix = create_master_edition_v3(
        mpl_token_metadata::ID,
        edition_key,
        collection_mint,
        crate::central_state::KEY,
        crate::central_state::KEY,
        metadata_key,
        *accounts.fee_payer.key,
        Some(0),
    );
    invoke_signed(
        &ix,
        &[
            accounts.metadata_program.clone(),
            accounts.edition.clone(),
            accounts.collection_mint.clone(),
            accounts.central_state.clone(),
            accounts.central_state.clone(),
            accounts.fee_payer.clone(),
            accounts.metadata_account.clone(),
            accounts.spl_token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent_account.clone(),
        ],
        &[seeds],
    )?;

    Ok(())
}
