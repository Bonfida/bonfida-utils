use std::convert::TryInto;

use pyth_client::{cast, Price, PriceStatus};
use solana_program::{msg, program_error::ProgramError};

use crate::fp_math::safe_downcast;

pub fn get_oracle_price_fp32(
    account_data: &[u8],
    base_decimals: u8,
    quote_decimals: u8,
) -> Result<u64, ProgramError> {
    let price_account = cast::<Price>(account_data);

    if matches!(price_account.agg.status, PriceStatus::Trading) {
        msg!("Pyth price account is not trading. Please retry");
        return Err(ProgramError::InvalidAccountData);
    }

    let price = ((price_account.agg.price as u128) << 32)
        .checked_div(10u128.pow(price_account.expo.abs().try_into().unwrap()))
        .unwrap();

    let corrected_price = (price * 10u128.pow(quote_decimals as u32))
        .checked_div(10u128.pow(base_decimals as u32))
        .unwrap();

    let final_price = safe_downcast(corrected_price).unwrap();

    msg!("Pyth FP32 price value: {:?}", final_price);

    Ok(final_price)
}
