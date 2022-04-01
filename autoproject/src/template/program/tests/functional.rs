use TOBEREPLACEDBY_LOWERSNAKE::entrypoint::process_instruction;

use {
    solana_program_test::{processor, ProgramTest},
    solana_sdk::{
        account::Account,
        signer::{keypair::Keypair, Signer},
    },
};

pub mod common;

#[tokio::test]
async fn test_offer() {
    // Create program and test environment
    let alice = Keypair::new();
    let bob = Keypair::new();

    let mut program_test = ProgramTest::new(
        "TOBEREPLACEDBY_LOWERSNAKE",
        TOBEREPLACEDBY_LOWERSNAKE::ID,
        processor!(process_instruction),
    );

    // program_test.add_program("example_dependency", example_dependency::ID, None);

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

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;
}
