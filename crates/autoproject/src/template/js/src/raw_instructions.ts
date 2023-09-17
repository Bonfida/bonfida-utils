// This file is auto-generated. DO NOT EDIT
import BN from "bn.js";
import { Schema, serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}
export class exampleInstruction {
  tag: number;
  static schema: Schema = new Map([
    [
      exampleInstruction,
      {
        kind: "struct",
        fields: [["tag", "u8"]],
      },
    ],
  ]);
  constructor() {
    this.tag = 0;
  }
  serialize(): Uint8Array {
    return serialize(exampleInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    tokenSource: PublicKey,
    splTokenProgram: PublicKey,
    systemProgram: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: tokenSource,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: splTokenProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: systemProgram,
      isSigner: false,
      isWritable: false,
    });
    return new TransactionInstruction({
      keys,
      programId,
      data,
    });
  }
}
