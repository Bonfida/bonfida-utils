use crate::{checks::check_account_owner, tokens};
use borsh::BorshDeserialize;
use pyth_sdk_solana::{
    state::{
        load_mapping_account, load_price_account, load_product_account, CorpAction, PriceStatus,
        PriceType,
    },
    Price,
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use solana_program::pubkey;
use solana_program::{
    account_info::AccountInfo, clock::Clock, msg, program_error::ProgramError, pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::convert::TryInto;

pub const DEFAULT_PYTH_PUSH: Pubkey = pubkey!("pythWSnswVUd12oZpeFP8e9CVaEqJg25g1Vtc2biRsT");
pub const PRICE_FEED_DISCRIMATOR: [u8; 8] = [34, 241, 35, 99, 157, 126, 244, 205];

pub fn check_price_acc_key(
    mapping_acc_data: &[u8],
    product_acc_key: &Pubkey,
    product_acc_data: &[u8],
    price_acc_key: &Pubkey,
) -> Result<(), ProgramError> {
    // Only checking the first mapping account
    let map_acct = load_mapping_account(mapping_acc_data).unwrap();

    // Get and print each Product in Mapping directory
    for prod_akey in &map_acct.products {
        let prod_key = Pubkey::new(&prod_akey.val);

        if *product_acc_key != prod_key {
            continue;
        }
        msg!("Found product in mapping.");

        let prod_acc = load_product_account(product_acc_data).unwrap();

        if !prod_acc.px_acc.is_valid() {
            msg!("Price account is invalid.");
            break;
        }

        // Check only the first price account
        let px_key = Pubkey::new(&prod_acc.px_acc.val);

        if *price_acc_key == px_key {
            msg!("Found correct price account in product.");
            return Ok(());
        }
    }

    msg!("Could not find product in mapping.");
    Err(ProgramError::InvalidArgument)
}

pub fn get_oracle_price_fp32(
    account_data: &[u8],
    base_decimals: u8,
    quote_decimals: u8,
) -> Result<u64, ProgramError> {
    #[cfg(feature = "mock-oracle")]
    {
        // Mock testing oracle
        if account_data.len() == 8 {
            return Ok(u64::from_le_bytes(account_data[0..8].try_into().unwrap()));
        }
    };

    // Pyth Oracle
    let price_account = load_price_account(account_data)?;
    let Price { price, expo, .. } = price_account
        .to_price_feed(&Pubkey::default())
        .get_current_price()
        .ok_or_else(|| {
            msg!("Cannot parse pyth price, information unavailable.");
            ProgramError::InvalidAccountData
        })?;
    let price = if expo > 0 {
        ((price as u128) << 32) * 10u128.pow(expo as u32)
    } else {
        ((price as u128) << 32) / 10u128.pow((-expo) as u32)
    };

    let corrected_price =
        (price * 10u128.pow(quote_decimals as u32)) / 10u128.pow(base_decimals as u32);

    let final_price = corrected_price.try_into().unwrap();

    msg!("Pyth FP32 price value: {:?}", final_price);

    Ok(final_price)
}

pub fn get_oracle_ema_price_fp32(
    account_data: &[u8],
    base_decimals: u8,
    quote_decimals: u8,
) -> Result<u64, ProgramError> {
    #[cfg(feature = "mock-oracle")]
    {
        // Mock testing oracle
        if account_data.len() == 8 {
            return Ok(u64::from_le_bytes(account_data[0..8].try_into().unwrap()));
        }
    };

    // Pyth Oracle
    let price_account = load_price_account(account_data)?;
    let Price { price, expo, .. } = price_account
        .to_price_feed(&Pubkey::default())
        .get_ema_price()
        .ok_or_else(|| {
            msg!("Cannot parse pyth ema price, information unavailable.");
            ProgramError::InvalidAccountData
        })?;
    let price = if expo > 0 {
        ((price as u128) << 32) * 10u128.pow(expo as u32)
    } else {
        ((price as u128) << 32) / 10u128.pow((-expo) as u32)
    };

    let corrected_price =
        (price * 10u128.pow(quote_decimals as u32)) / 10u128.pow(base_decimals as u32);

    let final_price = corrected_price.try_into().unwrap();

    msg!("Pyth FP32 price value: {:?}", final_price);

    Ok(final_price)
}

pub fn get_oracle_price_or_ema_fp32(
    account_data: &[u8],
    base_decimals: u8,
    quote_decimals: u8,
) -> Result<u64, ProgramError> {
    #[cfg(feature = "mock-oracle")]
    {
        // Mock testing oracle
        if account_data.len() == 8 {
            return Ok(u64::from_le_bytes(account_data[0..8].try_into().unwrap()));
        }
    };

    // Pyth Oracle
    let price_account = load_price_account(account_data)?;
    let price_feed = price_account.to_price_feed(&Pubkey::default());
    let Price { price, expo, .. } = price_feed
        .get_current_price()
        .or_else(|| {
            msg!("Cannot parse pyth price, information unavailable. Fallback on EMA");
            price_feed.get_ema_price()
        })
        .unwrap();
    let price = if expo > 0 {
        ((price as u128) << 32) * 10u128.pow(expo as u32)
    } else {
        ((price as u128) << 32) / 10u128.pow((-expo) as u32)
    };

    let corrected_price =
        (price * 10u128.pow(quote_decimals as u32)) / 10u128.pow(base_decimals as u32);

    let final_price = corrected_price.try_into().unwrap();

    msg!("Pyth FP32 price value: {:?}", final_price);

    Ok(final_price)
}

pub fn get_feed_id_from_mint(mint: &Pubkey) -> Result<[u8; 32], ProgramError> {
    match *mint {
        tokens::bat::MINT => Ok(tokens::bat::PRICE_FEED),
        tokens::bonk::MINT => Ok(tokens::bonk::PRICE_FEED),
        tokens::bsol::MINT => Ok(tokens::bsol::PRICE_FEED),
        tokens::fida::MINT => Ok(tokens::fida::PRICE_FEED),
        tokens::inj::MINT => Ok(tokens::inj::PRICE_FEED),
        tokens::msol::MINT => Ok(tokens::msol::PRICE_FEED),
        tokens::pyth::MINT => Ok(tokens::pyth::PRICE_FEED),
        tokens::sol::MINT => Ok(tokens::sol::PRICE_FEED),
        tokens::usdc::MINT => Ok(tokens::usdc::PRICE_FEED),
        tokens::usdt::MINT => Ok(tokens::usdt::PRICE_FEED),
        _ => Err(ProgramError::InvalidArgument),
    }
}

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

    let feed_id = get_feed_id_from_mint(token_mint).unwrap();

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

pub fn get_pyth_feed_account_key(shard: u16, price_feed: &[u8]) -> Pubkey {
    let seeds = &[&shard.to_le_bytes() as &[u8], &price_feed];
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
    pub fn test_sol() {
        // use pyth_sdk_solana::lo;
        use solana_client::rpc_client::RpcClient;
        use solana_program::pubkey;

        let pyth_sol_prod_acc = pubkey!("ALP8SdU9oARYVLgLR7LrqMNCYBnhtnQz1cj6bwgwQmgj");
        let pyth_sol_price_acc = pubkey!("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
        let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

        let prod_data = rpc_client.get_account_data(&pyth_sol_prod_acc).unwrap();
        let symbol = get_market_symbol(&prod_data).unwrap();
        let price_data = rpc_client.get_account_data(&pyth_sol_price_acc).unwrap();
        let price = get_oracle_price_fp32(&price_data, 6, 6).unwrap();
        println!("Found: '{}' FP32 Price: {}", symbol, price);
        let ema_price = get_oracle_ema_price_fp32(&price_data, 6, 6).unwrap();
        println!("Found: '{}' FP32 EMA Price: {}", symbol, ema_price);
    }

    #[test]
    fn print_pyth_oracles() {
        // use pyth_client::{load_mapping, load_price, load_product};
        use solana_client::rpc_client::RpcClient;
        use solana_program::pubkey;
        use solana_program::pubkey::Pubkey;

        let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let mut pyth_mapping_account = pubkey!("AHtgzX45WTKfkPG53L6WYhGEXwQkN1BVknET3sVsLL8J");

        loop {
            // Get Mapping account from key
            let map_data = rpc_client.get_account_data(&pyth_mapping_account).unwrap();
            let map_acct = load_mapping_account(&map_data).unwrap();

            // Get and print each Product in Mapping directory
            let mut i = 0;
            for prod_akey in &map_acct.products {
                let prod_pkey = Pubkey::new(&prod_akey.val);
                let prod_data = rpc_client.get_account_data(&prod_pkey).unwrap();
                let prod_acc = load_product_account(&prod_data).unwrap();

                // print key and reference data for this Product
                println!("product_account .. {:?}", prod_pkey);
                for (k, v) in prod_acc.iter() {
                    if !k.is_empty() || !v.is_empty() {
                        println!("{} {}", k, v);
                    }
                }

                // print all Prices that correspond to this Product
                if prod_acc.px_acc.is_valid() {
                    let mut px_pkey = Pubkey::new(&prod_acc.px_acc.val);
                    loop {
                        let pd = rpc_client.get_account_data(&px_pkey).unwrap();
                        let pa = load_price_account(&pd).unwrap();
                        println!("  price_account .. {:?}", px_pkey);
                        println!("    price_type ... {}", get_price_type(&pa.ptype));
                        println!("    exponent ..... {}", pa.expo);
                        println!("    status ....... {}", get_status(&pa.agg.status));
                        println!("    corp_act ..... {}", get_corp_act(&pa.agg.corp_act));
                        println!("    price ........ {}", pa.agg.price);
                        println!("    conf ......... {}", pa.agg.conf);
                        println!("    valid_slot ... {}", pa.valid_slot);
                        println!("    publish_slot . {}", pa.agg.pub_slot);

                        // go to next price account in list
                        if pa.next.is_valid() {
                            px_pkey = Pubkey::new(&pa.next.val);
                        } else {
                            break;
                        }
                    }
                }
                // go to next product
                i += 1;
                if i == map_acct.num {
                    break;
                }
            }

            // go to next Mapping account in list
            if !map_acct.next.is_valid() {
                break;
            }
            pyth_mapping_account = Pubkey::new(&map_acct.next.val);
        }
    }

    #[test]
    fn test_price_v2() {
        let feed = get_feed_id_from_mint(&tokens::sol::MINT).unwrap();
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
        let price_fp32 =
            get_oracle_price_fp32_v2(&tokens::sol::MINT, &account_info, 9, 6, &clock, 2 * 60)
                .unwrap();

        assert_eq!(565179032, price_fp32);
    }
}
