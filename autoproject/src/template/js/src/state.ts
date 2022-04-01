import { deserialize, Schema } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";

export enum Tag {
  Uninitialized = 0,
  Initialized = 1,
}

export class ExampleState {
  tag: Tag;
  nonce: number;
  nameAccount: PublicKey;
  owner: PublicKey;
  nftMint: PublicKey;

  static schema: Schema = new Map([
    [
      ExampleState,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["nonce", "u8"],
          ["owner", [32]],
        ],
      },
    ],
  ]);

  constructor(obj: { tag: number; nonce: number; owner: Uint8Array }) {
    this.tag = obj.tag as Tag;
    this.nonce = obj.nonce;
    this.owner = new PublicKey(obj.owner);
  }

  static deserialize(data: Buffer): ExampleState {
    return deserialize(this.schema, ExampleState, data);
  }

  static async retrieve(connection: Connection, key: PublicKey) {
    const accountInfo = await connection.getAccountInfo(key);
    if (!accountInfo || !accountInfo.data) {
      throw new Error("State account not found");
    }
    return this.deserialize(accountInfo.data);
  }
  static async findKey(nameAccount: PublicKey, programId: PublicKey) {
    return await PublicKey.findProgramAddress(
      [nameAccount.toBuffer()],
      programId
    );
  }
}
