from typing import List
from borsh_construct import U8, String, CStruct
from solana.transaction import TransactionInstruction, AccountMeta
from solana.publickey import PublicKey


class ExampleInstrInstruction:
    schema = CStruct(
        "tag" / U8,
        "example" / String,
    )

    def serialize(
        self,
        example: str,
    ) -> str:
        return self.schema.build(
            {
                "tag": 0,
                "example": example,
            }
        )

    def getInstruction(
        self,
        programId: PublicKey,
        system_program: PublicKey,
        spl_token_program: PublicKey,
        fee_payer: PublicKey,
        example_state: PublicKey,
        example: str,
    ) -> TransactionInstruction:
        data = self.serialize(
            example,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(system_program, False, False))
        keys.append(AccountMeta(spl_token_program, False, False))
        keys.append(AccountMeta(fee_payer, True, True))
        keys.append(AccountMeta(example_state, False, True))
        return TransactionInstruction(keys, programId, data)
