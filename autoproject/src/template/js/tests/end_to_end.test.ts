import { beforeAll, expect, jest, test } from "@jest/globals";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import { signAndSendTransactionInstructions, sleep } from "./utils";
import {
  Token,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { TokenMint } from "./utils";
import {  NAME_TOKENIZER_ID_DEVNET } from "../src/bindings";


// Global state initialized once in test startup and cleaned up at test
// teardown.
let connection: Connection;
let feePayer: Keypair;
let programId: PublicKey;

beforeAll(async () => {
  connection = new Connection(
    "https://explorer-api.devnet.solana.com/ ",
    "confirmed"
  );
  feePayer = Keypair.generate();
  const tx = await connection.requestAirdrop(
    feePayer.publicKey,
    LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(tx, "confirmed");
  console.log(`Fee payer airdropped tx ${tx}`);
  programId = NAME_TOKENIZER_ID_DEVNET;
});

jest.setTimeout(1_500_000);

/**
 * Test scenario
 *
 * ...
 * 
 */

test("End to end test", async () => {
  /**
   * Test variables
   */
  const decimals = Math.pow(10, 6);
  const token = await TokenMint.init(connection, feePayer);
  const alice = Keypair.generate();
  const bob = Keypair.generate();

  // Expected balances
  const bobExpectedBalance = { sol: 0, token: 0 };
  const aliceExpectedBalance = { sol: 0, token: 0 };

  /**
   * Create token ATA for Alice and Bob
   */

  const aliceTokenAtaKey = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    token.token.publicKey,
    alice.publicKey
  );
  const bobTokenAtaKey = await Token.getAssociatedTokenAddress(
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    token.token.publicKey,
    bob.publicKey
  );
  let ix = [
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      token.token.publicKey,
      aliceTokenAtaKey,
      alice.publicKey,
      feePayer.publicKey
    ),
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      token.token.publicKey,
      bobTokenAtaKey,
      bob.publicKey,
      feePayer.publicKey
    ),
  ];
  let tx = await signAndSendTransactionInstructions(
    connection,
    [],
    feePayer,
    ix
  );

  /**
   * Airdrop Alice
   */
  tx = await connection.requestAirdrop(alice.publicKey, LAMPORTS_PER_SOL);
  await connection.confirmTransaction(tx, "confirmed");
  aliceExpectedBalance.sol += LAMPORTS_PER_SOL;
});
