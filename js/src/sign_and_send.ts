import {
  Keypair,
  Connection,
  TransactionInstruction,
  Transaction,
  Commitment,
} from "@solana/web3.js";

/**
 * Sign and send transaction instructions
 * @param connection The solana connection object to the RPC node
 * @param signers The signers of the transaction
 * @param feePayer The fee payer of the transaction
 * @param txInstructions The instructions to
 * @param skipPreflight Whether to skip the preflight check or not
 * @param preflightCommitment The commitment to use for the preflight
 * @param confirmCommitment The commitment to use for the transaction confirmation
 * @returns The signature of the transaction as a string
 */
export const signAndSendInstructions = async (
  connection: Connection,
  signers: Array<Keypair>,
  feePayer: Keypair,
  txInstructions: Array<TransactionInstruction>,
  skipPreflight: boolean = false,
  preflightCommitment: Commitment = "confirmed",
  confirmCommitment: Commitment = "confirmed"
): Promise<string> => {
  const tx = new Transaction();

  tx.feePayer = feePayer.publicKey;

  signers.push(feePayer);
  tx.add(...txInstructions);

  const signature = await connection.sendTransaction(tx, signers, {
    skipPreflight,
    preflightCommitment,
  });

  await connection.confirmTransaction(signature, confirmCommitment);
  return signature;
};
