use std::convert::TryInto;

use pyth_client::{cast, Price};
use solana_program::{msg, program_error::ProgramError};

pub fn get_oracle_price_fp32(
    account_data: &[u8],
    base_decimals: u8,
    quote_decimals: u8,
) -> Result<u64, ProgramError> {
    let price_account = cast::<Price>(account_data);
    let price = ((price_account.agg.price as u128) << 32)
        / 10u128.pow(price_account.expo.abs().try_into().unwrap());

    let corrected_price =
        (price * 10u128.pow(quote_decimals as u32)) / 10u128.pow(base_decimals as u32);
    msg!("Oracle value: {:?}", corrected_price >> 32);

    Ok(corrected_price as u64)
}
