from typing import List
from borsh_construct import U8, String, CStruct
from solana.transaction import TransactionInstruction, AccountMeta
from solana.publickey import PublicKey


class CreateCollectionInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(
        self,
    ) -> str:
        return self.schema.build(
            {
                "tag": 1,
            }
        )

    def getInstruction(
        self,
        programId: PublicKey,
        collection_mint: PublicKey,
        edition: PublicKey,
        metadata_account: PublicKey,
        central_state: PublicKey,
        central_state_nft_ata: PublicKey,
        fee_payer: PublicKey,
        spl_token_program: PublicKey,
        metadata_program: PublicKey,
        system_program: PublicKey,
        spl_name_service_program: PublicKey,
        ata_program: PublicKey,
        rent_account: PublicKey,
    ) -> TransactionInstruction:
        data = self.serialize()
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(collection_mint, False, True))
        keys.append(AccountMeta(edition, False, True))
        keys.append(AccountMeta(metadata_account, False, True))
        keys.append(AccountMeta(central_state, False, False))
        keys.append(AccountMeta(central_state_nft_ata, False, True))
        keys.append(AccountMeta(fee_payer, False, False))
        keys.append(AccountMeta(spl_token_program, False, False))
        keys.append(AccountMeta(metadata_program, False, False))
        keys.append(AccountMeta(system_program, False, False))
        keys.append(AccountMeta(spl_name_service_program, False, False))
        keys.append(AccountMeta(ata_program, False, False))
        keys.append(AccountMeta(rent_account, False, False))
        return TransactionInstruction(keys, programId, data)


class CreateMintInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(
        self,
    ) -> str:
        return self.schema.build(
            {
                "tag": 0,
            }
        )

    def getInstruction(
        self,
        programId: PublicKey,
        mint: PublicKey,
        name_account: PublicKey,
        central_state: PublicKey,
        spl_token_program: PublicKey,
        system_program: PublicKey,
        rent_account: PublicKey,
        fee_payer: PublicKey,
    ) -> TransactionInstruction:
        data = self.serialize()
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(mint, False, True))
        keys.append(AccountMeta(name_account, False, True))
        keys.append(AccountMeta(central_state, False, False))
        keys.append(AccountMeta(spl_token_program, False, False))
        keys.append(AccountMeta(system_program, False, False))
        keys.append(AccountMeta(rent_account, False, False))
        keys.append(AccountMeta(fee_payer, False, False))
        return TransactionInstruction(keys, programId, data)


class RedeemNftInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(
        self,
    ) -> str:
        return self.schema.build(
            {
                "tag": 3,
            }
        )

    def getInstruction(
        self,
        programId: PublicKey,
        mint: PublicKey,
        nft_source: PublicKey,
        nft_owner: PublicKey,
        nft_record: PublicKey,
        name_account: PublicKey,
        spl_token_program: PublicKey,
        spl_name_service_program: PublicKey,
    ) -> TransactionInstruction:
        data = self.serialize()
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(mint, False, True))
        keys.append(AccountMeta(nft_source, False, True))
        keys.append(AccountMeta(nft_owner, True, True))
        keys.append(AccountMeta(nft_record, False, True))
        keys.append(AccountMeta(name_account, False, True))
        keys.append(AccountMeta(spl_token_program, False, False))
        keys.append(AccountMeta(spl_name_service_program, False, False))
        return TransactionInstruction(keys, programId, data)


class WithdrawTokensInstruction:
    schema = CStruct(
        "tag" / U8,
    )

    def serialize(
        self,
    ) -> str:
        return self.schema.build(
            {
                "tag": 4,
            }
        )

    def getInstruction(
        self,
        programId: PublicKey,
        nft: PublicKey,
        nft_owner: PublicKey,
        nft_record: PublicKey,
        token_destination: PublicKey,
        token_source: PublicKey,
        spl_token_program: PublicKey,
        system_program: PublicKey,
    ) -> TransactionInstruction:
        data = self.serialize()
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(nft, False, True))
        keys.append(AccountMeta(nft_owner, True, True))
        keys.append(AccountMeta(nft_record, False, True))
        keys.append(AccountMeta(token_destination, False, True))
        keys.append(AccountMeta(token_source, False, True))
        keys.append(AccountMeta(spl_token_program, False, False))
        keys.append(AccountMeta(system_program, False, False))
        return TransactionInstruction(keys, programId, data)


class CreateNftInstruction:
    schema = CStruct(
        "tag" / U8,
        "name" / String,
        "uri" / String,
    )

    def serialize(
        self,
        name: str,
        uri: str,
    ) -> str:
        return self.schema.build(
            {
                "tag": 2,
                "name": name,
                "uri": uri,
            }
        )

    def getInstruction(
        self,
        programId: PublicKey,
        mint: PublicKey,
        nft_destination: PublicKey,
        name_account: PublicKey,
        nft_record: PublicKey,
        name_owner: PublicKey,
        metadata_account: PublicKey,
        edition_account: PublicKey,
        collection_metadata: PublicKey,
        collection_mint: PublicKey,
        central_state: PublicKey,
        fee_payer: PublicKey,
        spl_token_program: PublicKey,
        metadata_program: PublicKey,
        system_program: PublicKey,
        spl_name_service_program: PublicKey,
        rent_account: PublicKey,
        metadata_signer: PublicKey,
        name: str,
        uri: str,
    ) -> TransactionInstruction:
        data = self.serialize(
            name,
            uri,
        )
        keys: List[AccountMeta] = []
        keys.append(AccountMeta(mint, False, True))
        keys.append(AccountMeta(nft_destination, False, True))
        keys.append(AccountMeta(name_account, False, True))
        keys.append(AccountMeta(nft_record, False, True))
        keys.append(AccountMeta(name_owner, True, True))
        keys.append(AccountMeta(metadata_account, False, True))
        keys.append(AccountMeta(edition_account, False, False))
        keys.append(AccountMeta(collection_metadata, False, False))
        keys.append(AccountMeta(collection_mint, False, False))
        keys.append(AccountMeta(central_state, False, True))
        keys.append(AccountMeta(fee_payer, True, True))
        keys.append(AccountMeta(spl_token_program, False, False))
        keys.append(AccountMeta(metadata_program, False, False))
        keys.append(AccountMeta(system_program, False, False))
        keys.append(AccountMeta(spl_name_service_program, False, False))
        keys.append(AccountMeta(rent_account, False, False))
        keys.append(AccountMeta(metadata_signer, True, False))
        return TransactionInstruction(keys, programId, data)
