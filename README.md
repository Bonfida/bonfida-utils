<h1 align="center">Bonfida utils</h1>
<br />
<p align="center">
<img width="250" src="https://ftx.com/static/media/fida.ce20eedf.svg"/>
</p>
<br />

<br />
<h2 align="center">Table of contents</h2>
<br />

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Used by](#used-by)
4. [Check functions](#check-functions)
5. [FP32 and FP64 math functions](#fp32)
6. [`InstructionsAccount` trait](#instructions-account)
7. [`BorshSize` trait](#borsh-size)
8. [Project structure](#project-structure)
9. [Example](#examples)

<br />
<a name="introduction"></a>
<h2 align="center">Introduction</h2>
<br />

This repo is a collection of different utilities in use across various Bonfida projects. The repository has the following structure:

- `utils` : Main `bonfida-utils` utilities library
- `autobindings` : CLI tool to autogenerate Typescript bindings for smart contracts written in the specific Bonfida style
- `autodoc`: CLI tool to generate a documented `instruction.rs` file
- `macros` : Auxiliary crate containing macros in use by the main `bonfida-utils` library

<br />
<a name="used-by"></a>
<h2 align="center">Used by</h2>
<br />

- [Serum DEX v4](https://github.com/Bonfida/dex-v4)
- [Serum Core (asset agnostic orderbook)](https://github.com/Bonfida/agnostic-orderbook)

<br />
<a name="installation"></a>
<h2 align="center">Installation</h2>
<br />

This repository is [published on crates.io](https://crates.io/crates/bonfida-utils), in order to use it in your Solana programs add this to your `Cargo.toml` file

```
bonfida-utils = "0.1"
```

To automatically generate Javascript instructions install the `autobindings` CLI

```
git clone https://github.com/Bonfida/bonfida-utils.git
cd bonfida-utils/autobindings
cargo install --path .
```

To automatically generate a documentted `instruction.rs` install the `autodoc` CLI:

```
cd bonfida-utils/autodoc
cargo install --path .
```

In order to generate instruction bindings automatically your project needs to follow a certain structure and derive certain traits that are detailed in the following sections.

<br />
<a name="check-functions"></a>
<h2 align="center">Check functions</h2>
<br />

`bonfida-utils` contains safety verification functions:

- `check_account_key`
- `check_account_owner`
- `check_signer`

<br />
<a name="fp32"></a>
<h2 align="center">FP32 and FP64 math functions</h2>
<br />

`bonfida-utils` contains some useful math functions for FP32 and FP64:

- `fp32_div`
- `fp32_mul`
- `ifp32_div`
- `ifp32_mul`
- `fp64_div`
- `fp64_mul`

<br/>
<a name="instructions-account"></a>
<h2 align="center">InstructionsAccount</h2>
<br/>

The `Accounts` struct needs to derive the `InstructionsAccount` trait in order to automatically generate bindings in Rust and JS. In order to know which accounts are writable and/or signer you will have to specify constraints (`cons`) for each account of the struct:

1.  For writable accounts: `#[cons(writable)]`
2.  For signer accounts: `#[cons(signer)]`
3.  For signer and writable accounts: `#[cons(signer, writable)]`

For example

```rust
use bonfida_utils::{InstructionsAccount};

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {

    pub read_only_account: &'a T, // Read only account

    #[cons(writable)] // This specifies that the account is writable
    pub writable_account: &'a T,

    #[cons(signer)] // This specifies that the account is sginer
    pub signer_account: &'a T,

    // Write the d
    #[cons(signer, writable)]  // This specifies that the account is sginer and writable
    pub signer_and_writable_account: &'a T,
}

```

To specify accounts that are optional

```rust
pub struct Accounts<'a, T> {
    // Writable account that is optional
    #[cons(writable)]
    pub referrer_account_opt: Option<&'a T>,
}
```

<br/>
<a name="borsh-size"></a>
<h2 align="center">BorshSize</h2>
<br/>

The struct used for the data of the instruction needs to derive the `BorshSize` trait, for example let's take the following struct

```rust
#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
pub struct Params {
    pub position_type: PositionType,
    pub market_index: u16,
    pub max_base_qty: u64,
    pub max_quote_qty: u64,
    pub limit_price: u64,
    pub match_limit: u64,
    pub self_trade_behavior: u8,
    pub order_type: OrderType,
    pub number_of_markets: u8,
}
```

In the above example, `BorshSize` should be derived for `PositionType` and `OrderType` as well. The derive macro can take care of this
for field-less enums :

```rust
#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
pub enum OrderType {
  Limit,
  ImmediateOrCancel,
  FillOrKill,
  PostOnly
}
```

You might need to implement `BorshSize` yourself for certain types (e.g an `enum` with variants containing fields) :

```rust
#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq, FromPrimitive)]
pub enum ExampleEnum {
    FirstVariant,
    SecondVariant(u128),
}

impl BorshSize for ExampleEnum {
    fn borsh_len(&self) -> usize {
        match self {
          Self::FirstVariant => 1,
          Self::SecondVariant(n) => 1 + n.borsh_len()
        }
    }
}
```

<br />
<a name="project-structure"></a>
<h2 align="center">Project structure</h2>
<br />

🚨 The project structure is important:

1. The Solana program must be in a folder called `program`
2. The JS bindings must be in a folder called `js`
3. The processor folder needs to contain instructions' logic in separate files. The name of each file needs to be snake case and match the name of the associated function in `instructions.rs`
4. The instruction enum of `instructions.rs` needs to have Pascal case names that match the snake case names of the files in `processor`. This is detailed in the example below.
5. The instruction enum of `instructions.rs` needs to be the first enum to be defined in that file.

<br />
<a name="examples"></a>
<h2 align="center">Examples</h2>
<br />

Let's have a look at a real life example.

The project structure is as follow

```
├── program
│   ├── instructions.rs
│   ├── processor
│   │   ├── create_market.rs
├── js
    ├── src
├── python
    ├── src
│...
│... The rest is omitted
```

To simplify we will consider only one instruction `create_market.rs` and only focus on `processor` and `instructions.rs`.

We can see from the project structure that `create_market.rs` is located in the `processor` directory, let's have a look at the contents of the file:

```rust
//! Creates a perp market
// The sentence above will be used to describe the instruction in the auto generated doc ⚠️
use bonfida_utils::{checks::check_signer, BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
// Other imports are omitted

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The market address
    #[cons(writable)] // This specifies that the account is writable
    pub market: &'a T,

    /// The ecosystem address
    #[cons(writable)]
    pub ecosystem: &'a T,

    /// The address of the Serum Core market
    pub aob_orderbook: &'a T,

    /// The address of the Serum Core event queue
    pub aob_event_queue: &'a T,

    /// The address of the Serum Core asks slab
    pub aob_asks: &'a T,

    /// The address of the Serum Core bids slab
    pub aob_bids: &'a T,

    /// The program ID of Serum Core
    pub aob_program: &'a T,

    /// The Pyth oracle address for this market
    pub oracle: &'a T,

    /// The market admin address
    #[cons(signer)] // This specifies that the account is signer
    pub admin: &'a T,

    /// The market vault address
    pub vault: &'a T,
}

// Constraints (cons) can be combined e.g
// #[cons(signer, writable)]
// pub some_account: &'a T
//
// Optional accounts are supported as well
// pub discount_account_opt: Option<&'a T>

// BorshSize might require custom impl e.g for enum
#[derive(BorshSerialize, BorshDeserialize, BorshSize)]
pub struct Params {
    pub market_symbol: String,
    pub signer_nonce: u8,
    pub coin_decimals: u8,
    pub quote_decimals: u8,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let a = Accounts {
            market: next_account_info(accounts_iter)?,
            ecosystem: next_account_info(accounts_iter)?,
            aob_orderbook: next_account_info(accounts_iter)?,
            aob_event_queue: next_account_info(accounts_iter)?,
            aob_asks: next_account_info(accounts_iter)?,
            aob_bids: next_account_info(accounts_iter)?,
            aob_program: next_account_info(accounts_iter)?,
            oracle: next_account_info(accounts_iter)?,
            admin: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
        };

        // Account checks are omitted

        Ok(a)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let accounts = Accounts::parse(accounts)?;

    let Params {
        market_symbol,
        signer_nonce,
        coin_decimals,
        quote_decimals,
    } = params;

    // Instruction logic is omitted

    Ok(())
}

```

We can see that `create_market.rs` contains two important definitions:

1. `Accounts` struct: its `InstructionsAccount` trait implementation should be derived, and the constraints for each account of the struct should be specified
2. `Params` struct: its `BorshSize` trait implementation should be derived

The `instruction.rs` file can be autogenerated using `cargo autodoc`

```rust
use bonfida_utils::InstructionsAccount;
// Other imports are omitted

#[derive(BorshSerialize, BorshDeserialize)]
pub enum PerpInstruction {
    /// Creates a new perps market
    ///
    /// | Index | Writable | Signer | Description                               |
    /// | --------------------------------------------------------------------- |
    /// | 0     | ✅        | ❌      | The market address                        |
    /// | 1     | ✅        | ❌      | The ecosystem address                     |
    /// | 2     | ❌        | ❌      | The address of the Serum Core market      |
    /// | 3     | ❌        | ❌      | The address of the Serum Core event queue |
    /// | 4     | ❌        | ❌      | The address of the Serum Core asks slab   |
    /// | 5     | ❌        | ❌      | The address of the Serum Core bids slab   |
    /// | 6     | ❌        | ❌      | The program ID of Serum Core              |
    /// | 7     | ❌        | ❌      | The Pyth oracle address for this market   |
    /// | 8     | ❌        | ✅      | The market admin address                  |
    /// | 9     | ❌        | ❌      | The market vault address                  |
    CreateMarket,
    ///
    /// ...
}
pub fn create_market(
    accounts: create_market::Accounts<Pubkey>,
    params: create_market::Params,
) -> Instruction {
    accounts.get_instruction(crate::ID, PerpInstruction::CreateMarket as u8, params)
}
```

In order to generate Javascript instruction bindings run

```
cargo autobindings
```

This will generate a file named `raw_instructions.ts` that contains all the instructions of your program

```js
// This file is auto-generated. DO NOT EDIT
import BN from "bn.js";
import { Schema, serialize } from "borsh";
import { PublicKey, TransactionInstruction } from "@solana/web3.js";

export interface AccountKey {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}

export class createMarketInstruction {
  tag: number;
  marketSymbol: string;
  signerNonce: number;
  coinDecimals: number;
  quoteDecimals: number;
  static schema: Schema = new Map([
    [
      createMarketInstruction,
      {
        kind: "struct",
        fields: [
          ["tag", "u8"],
          ["marketSymbol", "string"],
          ["signerNonce", "u8"],
          ["coinDecimals", "u8"],
          ["quoteDecimals", "u8"],
        ],
      },
    ],
  ]);
  constructor(obj: {
    marketSymbol: string,
    signerNonce: number,
    coinDecimals: number,
    quoteDecimals: number,
  }) {
    this.tag = 0;
    this.marketSymbol = obj.marketSymbol;
    this.signerNonce = obj.signerNonce;
    this.coinDecimals = obj.coinDecimals;
    this.quoteDecimals = obj.quoteDecimals;
  }
  serialize(): Uint8Array {
    return serialize(createMarketInstruction.schema, this);
  }
  getInstruction(
    programId: PublicKey,
    market: PublicKey,
    ecosystem: PublicKey,
    aobOrderbook: PublicKey,
    aobEventQueue: PublicKey,
    aobAsks: PublicKey,
    aobBids: PublicKey,
    aobProgram: PublicKey,
    oracle: PublicKey,
    admin: PublicKey,
    vault: PublicKey
  ): TransactionInstruction {
    const data = Buffer.from(this.serialize());
    let keys: AccountKey[] = [];
    keys.push({
      pubkey: market,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: ecosystem,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: aobOrderbook,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: aobEventQueue,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: aobAsks,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: aobBids,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: aobProgram,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: oracle,
      isSigner: false,
      isWritable: false,
    });
    keys.push({
      pubkey: admin,
      isSigner: true,
      isWritable: false,
    });
    keys.push({
      pubkey: vault,
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
```

To generate Python bindings run

```
cargo autobindings --target-language py
```
