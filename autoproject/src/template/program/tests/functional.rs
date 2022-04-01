use {
    borsh::BorshSerialize,
    mpl_token_metadata::pda::{find_master_edition_account, find_metadata_account},
    name_tokenizer::{
        entrypoint::process_instruction,
        instruction::{create_collection, create_mint, create_nft, redeem_nft, withdraw_tokens},
        state::{
            CentralState, NftRecord, COLLECTION_PREFIX, METADATA_SIGNER, MINT_PREFIX,
            ROOT_DOMAIN_ACCOUNT,
        },
    },
    solana_program::{hash::hashv, pubkey::Pubkey, system_program, sysvar},
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        signer::{keypair::Keypair, Signer},
    },
    spl_associated_token_account::{create_associated_token_account, get_associated_token_address},
    spl_name_service::state::{get_seeds_and_key, HASH_PREFIX},
};

pub mod common;

use crate::common::utils::{mint_bootstrap, sign_send_instructions};

#[tokio::test]
async fn test_offer() {
    // Create program and test environment
    let alice = Keypair::new();
    let bob = Keypair::new();
    let mint_authority = Keypair::new();

    let mut program_test = ProgramTest::new(
        "name_tokenizer",
        name_tokenizer::ID,
        processor!(process_instruction),
    );
    program_test.add_program("spl_name_service", spl_name_service::ID, None);
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

    // Create domain name
    let name = "something_domain_name";
    let hashed_name = hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec();

    let (name_key, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed_name,
        None,
        Some(&ROOT_DOMAIN_ACCOUNT),
    );

    let root_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: ROOT_DOMAIN_ACCOUNT,
        owner: alice.pubkey(),
        class: Pubkey::default(),
    }
    .try_to_vec()
    .unwrap();

    program_test.add_account(
        name_key,
        Account {
            lamports: 1_000_000,
            data: root_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    program_test.add_account(
        alice.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        bob.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );

    //
    // Create mint
    //
    let (usdc_mint, _) = mint_bootstrap(None, 6, &mut program_test, &mint_authority.pubkey());

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    /////
    // Create central state
    ////
    let (central_key, _) = CentralState::find_key(&name_tokenizer::ID);

    ////
    // Create mint
    ////
    let (nft_mint, _) =
        Pubkey::find_program_address(&[MINT_PREFIX, &name_key.to_bytes()], &name_tokenizer::ID);

    let ix = create_mint(
        create_mint::Accounts {
            mint: &nft_mint,
            central_state: &central_key,
            name_account: &name_key,
            spl_token_program: &spl_token::ID,
            rent_account: &sysvar::rent::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            system_program: &system_program::ID,
        },
        create_mint::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    ////
    // Create central state ATA for collection mint
    ////

    let (collection_mint, _) = Pubkey::find_program_address(
        &[COLLECTION_PREFIX, &name_tokenizer::ID.to_bytes()],
        &name_tokenizer::ID,
    );
    let central_state_collection_ata =
        get_associated_token_address(&name_tokenizer::central_state::KEY, &collection_mint);

    ////
    // Create collection
    ////
    let (edition_key, _) = find_master_edition_account(&collection_mint);
    let (collection_metadata_key, _) = find_metadata_account(&collection_mint);
    let ix = create_collection(
        create_collection::Accounts {
            collection_mint: &collection_mint,
            edition: &edition_key,
            metadata_account: &collection_metadata_key,
            central_state: &central_key,
            central_state_nft_ata: &central_state_collection_ata,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            spl_token_program: &spl_token::ID,
            metadata_program: &mpl_token_metadata::ID,
            system_program: &system_program::ID,
            spl_name_service_program: &spl_name_service::ID,
            rent_account: &sysvar::rent::ID,
            ata_program: &spl_associated_token_account::ID,
        },
        create_collection::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    ////
    // Create Alice and Bob ATAs
    ////

    let ix = create_associated_token_account(&alice.pubkey(), &alice.pubkey(), &nft_mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();
    let ix = create_associated_token_account(&bob.pubkey(), &bob.pubkey(), &nft_mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();

    ////
    // Create NFT
    ////

    let alice_nft_ata = get_associated_token_address(&alice.pubkey(), &nft_mint);
    let bob_nft_ata = get_associated_token_address(&bob.pubkey(), &nft_mint);
    let (nft_record, _) = NftRecord::find_key(&name_key, &name_tokenizer::ID);
    let (metadata_key, _) = find_metadata_account(&nft_mint);

    let ix = create_nft(
        create_nft::Accounts {
            mint: &nft_mint,
            nft_destination: &alice_nft_ata,
            name_account: &name_key,
            nft_record: &nft_record,
            name_owner: &alice.pubkey(),
            metadata_account: &metadata_key,
            central_state: &central_key,
            spl_token_program: &spl_token::ID,
            metadata_program: &mpl_token_metadata::ID,
            system_program: &system_program::ID,
            spl_name_service_program: &spl_name_service::ID,
            rent_account: &sysvar::rent::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            edition_account: &edition_key,
            collection_metadata: &collection_metadata_key,
            collection_mint: &collection_mint,
            #[cfg(not(feature = "devnet"))]
            metadata_signer: &METADATA_SIGNER,
        },
        create_nft::Params {
            name: name.to_string(),
            uri: "test".to_string(),
        },
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ////
    // Withdraw NFT
    ////
    let ix = redeem_nft(
        redeem_nft::Accounts {
            mint: &nft_mint,
            nft_source: &alice_nft_ata,
            nft_owner: &alice.pubkey(),
            nft_record: &nft_record,
            name_account: &name_key,
            spl_token_program: &spl_token::ID,
            spl_name_service_program: &spl_name_service::ID,
        },
        redeem_nft::Params {},
    );

    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ////
    // Send tokens
    ////
    let usdc_ata_program = get_associated_token_address(&nft_record, &usdc_mint);
    let usdc_ata_alice = get_associated_token_address(&alice.pubkey(), &usdc_mint);
    let usdc_ata_bob = get_associated_token_address(&bob.pubkey(), &usdc_mint);

    let ix = create_associated_token_account(&prg_test_ctx.payer.pubkey(), &nft_record, &usdc_mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let ix =
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &alice.pubkey(), &usdc_mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();
    let ix =
        create_associated_token_account(&prg_test_ctx.payer.pubkey(), &bob.pubkey(), &usdc_mint);
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &usdc_mint,
        &usdc_ata_program,
        &mint_authority.pubkey(),
        &[],
        10_000_000_000,
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&mint_authority])
        .await
        .unwrap();

    ////
    // Withdraw sent tokens
    ////
    let ix = withdraw_tokens(
        withdraw_tokens::Accounts {
            nft: &alice_nft_ata,
            nft_owner: &alice.pubkey(),
            nft_record: &nft_record,
            token_source: &usdc_ata_program,
            token_destination: &usdc_ata_alice,
            spl_token_program: &spl_token::ID,
            system_program: &system_program::ID,
        },
        withdraw_tokens::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ////
    // Bob tries to withdraw tokens
    ////

    let ix = withdraw_tokens(
        withdraw_tokens::Accounts {
            nft: &alice_nft_ata,
            nft_owner: &bob.pubkey(),
            nft_record: &nft_record,
            token_source: &usdc_ata_program,
            token_destination: &usdc_ata_bob,
            spl_token_program: &spl_token::ID,
            system_program: &system_program::ID,
        },
        withdraw_tokens::Params {},
    );
    let err = sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .is_err();
    assert!(err);

    ////
    // Alice transfer domain to Bob
    ////
    let ix = spl_name_service::instruction::transfer(
        spl_name_service::ID,
        bob.pubkey(),
        name_key,
        alice.pubkey(),
        None,
    )
    .unwrap();
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ////
    // Bob creates NFT again
    ////
    let ix = create_nft(
        create_nft::Accounts {
            mint: &nft_mint,
            nft_destination: &bob_nft_ata,
            name_account: &name_key,
            nft_record: &nft_record,
            name_owner: &bob.pubkey(),
            metadata_account: &metadata_key,
            central_state: &central_key,
            spl_token_program: &spl_token::ID,
            metadata_program: &mpl_token_metadata::ID,
            system_program: &system_program::ID,
            spl_name_service_program: &spl_name_service::ID,
            rent_account: &sysvar::rent::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            edition_account: &edition_key,
            collection_metadata: &collection_metadata_key,
            collection_mint: &collection_mint,
            #[cfg(not(feature = "devnet"))]
            metadata_signer: &METADATA_SIGNER,
        },
        create_nft::Params {
            name: name.to_string(),
            uri: "test".to_string(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
    let ix = withdraw_tokens(
        withdraw_tokens::Accounts {
            nft: &bob_nft_ata,
            nft_owner: &bob.pubkey(),
            nft_record: &nft_record,
            token_source: &usdc_ata_program,
            token_destination: &usdc_ata_bob,
            spl_token_program: &spl_token::ID,
            system_program: &system_program::ID,
        },
        withdraw_tokens::Params {},
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![&bob])
        .await
        .unwrap();
}
