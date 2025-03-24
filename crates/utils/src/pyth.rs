use crate::{checks::check_account_owner, tokens::SupportedToken};
use borsh::BorshDeserialize;
use pyth_sdk_solana::state::{load_product_account, CorpAction, PriceStatus, PriceType};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use solana_program::pubkey;
use solana_program::{
    account_info::AccountInfo, clock::Clock, msg, program_error::ProgramError, pubkey::Pubkey,
};
use std::convert::TryInto;

pub const DEFAULT_PYTH_PUSH: Pubkey = pubkey!("pythWSnswVUd12oZpeFP8e9CVaEqJg25g1Vtc2biRsT");
pub const PRICE_FEED_DISCRIMATOR: [u8; 8] = [34, 241, 35, 99, 157, 126, 244, 205];

pub fn parse_price_v2(data: &[u8]) -> Result<PriceUpdateV2, ProgramError> {
    let tag = &data[..8];

    if tag != PRICE_FEED_DISCRIMATOR {
        return Err(ProgramError::InvalidAccountData);
    }

    let des = PriceUpdateV2::deserialize(&mut &data[8..]).unwrap();

    Ok(des)
}

// Used for Pyth v2 i.e pull model
pub fn get_oracle_price_fp32_v2(
    token_mint: &Pubkey,
    account: &AccountInfo,
    base_decimals: u8,
    quote_decimals: u8,
    clock: &Clock,
    maximum_age: u64,
) -> Result<u64, ProgramError> {
    check_account_owner(account, &pyth_solana_receiver_sdk::ID)?;

    let data = &account.data.borrow() as &[u8];

    let update = parse_price_v2(data).unwrap();

    let feed_id = SupportedToken::from_mint(token_mint).unwrap().price_feed();

    let pyth_solana_receiver_sdk::price_update::Price {
        price, exponent, ..
    } = update
        .get_price_no_older_than(clock, maximum_age, &feed_id)
        .unwrap();

    let price = if exponent > 0 {
        ((price as u128) << 32) * 10u128.pow(exponent as u32)
    } else {
        ((price as u128) << 32) / 10u128.pow((-exponent) as u32)
    };

    let corrected_price =
        (price * 10u128.pow(quote_decimals as u32)) / 10u128.pow(base_decimals as u32);

    let final_price = corrected_price.try_into().unwrap();

    msg!("Pyth FP32 price value: {:?}", final_price);

    Ok(final_price)
}

// Used for Pyth v2 to allow any feed id without validation, the token/asset validation must be done by the caller program
pub fn get_oracle_price_from_feed_id_fp32(
    feed_id: &[u8; 32],
    account: &AccountInfo,
    base_decimals: u8,
    quote_decimals: u8,
    clock: &Clock,
    maximum_age: u64,
) -> Result<u64, ProgramError> {
    check_account_owner(account, &pyth_solana_receiver_sdk::ID)?;

    let data = &account.data.borrow() as &[u8];

    let update = parse_price_v2(data).unwrap();

    let pyth_solana_receiver_sdk::price_update::Price {
        price, exponent, ..
    } = update
        .get_price_no_older_than(clock, maximum_age, feed_id)
        .unwrap();

    let price = if exponent > 0 {
        ((price as u128) << 32) * 10u128.pow(exponent as u32)
    } else {
        ((price as u128) << 32) / 10u128.pow((-exponent) as u32)
    };

    let corrected_price =
        (price * 10u128.pow(quote_decimals as u32)) / 10u128.pow(base_decimals as u32);

    let final_price = corrected_price.try_into().unwrap();

    msg!("Pyth FP32 price value: {:?}", final_price);

    Ok(final_price)
}

pub fn get_pyth_feed_account_key(shard: u16, price_feed: &[u8]) -> Pubkey {
    let seeds = &[&shard.to_le_bytes() as &[u8], price_feed];
    let (key, _) = Pubkey::find_program_address(seeds, &DEFAULT_PYTH_PUSH);
    key
}

pub fn get_market_symbol(pyth_product_acc_data: &[u8]) -> Result<&str, ProgramError> {
    let pyth_product = load_product_account(pyth_product_acc_data).unwrap();
    for (k, v) in pyth_product.iter() {
        if k == "symbol" {
            return Ok(v);
        }
    }
    msg!("The provided pyth product account has no attribute 'symbol'.");
    Err(ProgramError::InvalidArgument)
}

//Utils

pub fn get_price_type(ptype: &PriceType) -> &'static str {
    match ptype {
        PriceType::Unknown => "unknown",
        PriceType::Price => "price",
        // PriceType::TWAP => "twap",
        // PriceType::Volatility => "volatility",
    }
}

pub fn get_status(st: &PriceStatus) -> &'static str {
    match st {
        PriceStatus::Unknown => "unknown",
        PriceStatus::Trading => "trading",
        PriceStatus::Halted => "halted",
        PriceStatus::Auction => "auction",
        PriceStatus::Ignored => "ignored",
    }
}

pub fn get_corp_act(cact: &CorpAction) -> &'static str {
    match cact {
        CorpAction::NoCorpAct => "nocorpact",
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test_price_v2() {
        let feed = SupportedToken::Sol.price_feed();
        let key = get_pyth_feed_account_key(0, &feed);

        let mut account_data = [
            34, 241, 35, 99, 157, 126, 244, 205, 96, 49, 71, 4, 52, 13, 237, 223, 55, 31, 212, 36,
            114, 20, 143, 36, 142, 157, 26, 109, 26, 94, 178, 172, 58, 205, 139, 127, 213, 214,
            178, 67, 1, 239, 13, 139, 111, 218, 44, 235, 164, 29, 161, 93, 64, 149, 209, 218, 57,
            42, 13, 47, 142, 208, 198, 199, 188, 15, 76, 250, 200, 194, 128, 181, 109, 151, 237,
            87, 16, 3, 0, 0, 0, 135, 164, 49, 1, 0, 0, 0, 0, 248, 255, 255, 255, 103, 85, 30, 102,
            0, 0, 0, 0, 103, 85, 30, 102, 0, 0, 0, 0, 208, 47, 218, 39, 3, 0, 0, 0, 14, 62, 204, 0,
            0, 0, 0, 0, 255, 5, 134, 15, 0, 0, 0, 0, 0,
        ];

        let mut lamports = u64::MAX;
        let account_info = AccountInfo {
            data: Rc::new(RefCell::new(&mut account_data[..])),
            key: &key,
            lamports: Rc::new(RefCell::new(&mut lamports)),
            owner: &pyth_solana_receiver_sdk::ID,
            rent_epoch: u64::MAX,
            is_signer: false,
            is_writable: false,
            executable: false,
        };
        let clock: Clock = Clock {
            ..Default::default()
        };
        let price_fp32 = get_oracle_price_fp32_v2(
            &SupportedToken::Sol.mint(),
            &account_info,
            9,
            6,
            &clock,
            2 * 60,
        )
        .unwrap();

        assert_eq!(565179032, price_fp32);

        let price_fp32 = get_oracle_price_from_feed_id_fp32(
            &SupportedToken::Sol.price_feed(),
            &account_info,
            9,
            6,
            &clock,
            2 * 60,
        )
        .unwrap();
        assert_eq!(565179032, price_fp32);
    }
}
