import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { exampleInstruction } from "./raw_instructions";

/**
 * Mainnet program ID
 */
export const NAME_TOKENIZER_ID = new PublicKey(""); //TODO

/**
 * Devnet program ID (might not have the latest version deployed!)
 */
export const NAME_TOKENIZER_ID_DEVNET = new PublicKey(""); //TODO

/**
 * This function can be used as a js binding example.
 * @param feePayer The fee payer of the transaction
 * @param programId The program ID
 * @returns
 */
export const example = async (feePayer: PublicKey, programId: PublicKey) => {
  const ix = new exampleInstruction().getInstruction(
    programId,
    SystemProgram.programId,
    SYSVAR_RENT_PUBKEY,
    feePayer
  );

  return [ix];
};
