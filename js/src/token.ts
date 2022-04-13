import { Keypair, PublicKey, Connection } from "@solana/web3.js";
import {
  mintTo,
  createMint,
  createAssociatedTokenAccount,
  getAssociatedTokenAddress,
} from "@solana/spl-token";
import { checkAccountExist } from "./check_account_exist";

/**
 * A `TokenMint` object can be used to easily create and mint tokens in dev environment
 */
export class TokenMint {
  constructor(
    public token: PublicKey,
    public signer: Keypair,
    public connection: Connection
  ) {}

  /**
   * Initialize a `TokenMint` object
   * @param connection The solana connection object to the RPC node
   * @param feePayer The fee payer used to create the mint
   * @param mintAuthority The mint authority
   * @returns
   */
  static async init(
    connection: Connection,
    feePayer: Keypair,
    mintAuthority: PublicKey | null = null
  ): Promise<TokenMint> {
    let signer = new Keypair();
    let token = await createMint(
      connection,
      feePayer,
      mintAuthority || signer.publicKey,
      null,
      6
    );
    return new TokenMint(token, signer, connection);
  }

  /**
   * Get or create the associated token account for the specified `wallet`
   * @param wallet The wallet to get the ATA for
   * @param allowOffcurse Allow the owner account to be a PDA
   * @returns
   */
  async getAssociatedTokenAccount(
    wallet: PublicKey,
    allowOffcurse: boolean = false
  ): Promise<PublicKey> {
    const ata = await getAssociatedTokenAddress(
      this.token,
      wallet,
      allowOffcurse
    );
    const exist = await checkAccountExist(this.connection, ata);

    if (exist) {
      return ata;
    }

    await createAssociatedTokenAccount(
      this.connection,
      this.signer,
      this.token,
      wallet
    );
    return ata;
  }

  /**
   * Mint `amount` tokens into `tokenAccount`
   * @param tokenAccount The token account to mint into
   * @param amount The amount ot mint
   * @returns
   */
  async mintInto(tokenAccount: PublicKey, amount: number): Promise<string> {
    return await mintTo(
      this.connection,
      this.signer,
      this.token,
      tokenAccount,
      this.signer,
      amount
    );
  }
}
