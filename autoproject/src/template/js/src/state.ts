import { deserialize, Schema } from "borsh";
import { Connection, PublicKey } from "@solana/web3.js";
import BN from "bn.js";

export enum Tag {
  Uninitialized = 0,
  Initialized = 1,
}

export class ExampleState {
  static SEED = "example_seed";
  tag: Tag;
  nonce: number;

  static schema: Schema = new Map([
    [
      ExampleState,
      {
        kind: "struct",
        fields: [
          ["tag", "u64"],
          ["nonce", "u8"],
        ],
      },
    ],
  ]);

  constructor(obj: { tag: BN; nonce: number; owner: Uint8Array }) {
    this.tag = obj.tag.toNumber() as Tag;
    this.nonce = obj.nonce;
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
  static async findKey(programId: PublicKey) {
    return await PublicKey.findProgramAddress(
      [Buffer.from(ExampleState.SEED)],
      programId
    );
  }
}
