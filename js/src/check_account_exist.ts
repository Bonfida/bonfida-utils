import { Connection, PublicKey } from "@solana/web3.js";

/**
 * Check if an account exist
 * @param connection The solana connection object to the RPC node
 * @param key The account to check
 * @returns A boolean
 */
export const checkAccountExist = async (
  connection: Connection,
  key: PublicKey
): Promise<boolean> => {
  const info = await connection.getAccountInfo(key);
  if (!info || !info.data) {
    return false;
  }
  return true;
};
