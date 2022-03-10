use crate::fp_math::safe_downcast;
use pyth_client::{load_price, CorpAction, PriceConf, PriceStatus, PriceType, Product};
use solana_program::{msg, program_error::ProgramError};
#[cfg(feature = "mock-oracle")]
use std::convert::TryInto;

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
    let price_account = load_price(account_data)?;
    let PriceConf {
        price,
        conf: _,
        expo,
    } = price_account.get_current_price().ok_or_else(|| {
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

    let final_price = safe_downcast(corrected_price).unwrap();

    msg!("Pyth FP32 price value: {:?}", final_price);

    Ok(final_price)
}

pub fn get_market_symbol(pyth_product: &Product) -> Result<&str, ProgramError> {
    for (k, v) in pyth_product.iter() {
        if k == "symbol" {
            return Ok(v);
        }
    }
    msg!("The provided pyth product account has no attribute 'symbol'.");
    Err(ProgramError::InvalidArgument)
}

#[test]
pub fn test_sol() {
    use pyth_client::load_product;
    use solana_client::rpc_client::RpcClient;
    use solana_program::pubkey;

    let pyth_sol_prod_acc = pubkey!("ALP8SdU9oARYVLgLR7LrqMNCYBnhtnQz1cj6bwgwQmgj");
    let pyth_sol_price_acc = pubkey!("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    let prod_data = rpc_client.get_account_data(&pyth_sol_prod_acc).unwrap();
    let symbol = get_market_symbol(load_product(&prod_data).unwrap()).unwrap();
    let price_data = rpc_client.get_account_data(&pyth_sol_price_acc).unwrap();
    let price = get_oracle_price_fp32(&price_data, 6, 6).unwrap();
    println!("Found: '{}' FP32 Price: {}", symbol, price);
}

#[test]
fn print_pyth_oracles() {
    use pyth_client::{load_mapping, load_price, load_product};
    use solana_client::rpc_client::RpcClient;
    use solana_program::pubkey;
    use solana_program::pubkey::Pubkey;

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let mut pyth_mapping_account = pubkey!("AHtgzX45WTKfkPG53L6WYhGEXwQkN1BVknET3sVsLL8J");

    loop {
        // Get Mapping account from key
        let map_data = rpc_client.get_account_data(&pyth_mapping_account).unwrap();
        let map_acct = load_mapping(&map_data).unwrap();

        // Get and print each Product in Mapping directory
        let mut i = 0;
        for prod_akey in &map_acct.products {
            let prod_pkey = Pubkey::new(&prod_akey.val);
            let prod_data = rpc_client.get_account_data(&prod_pkey).unwrap();
            let prod_acc = load_product(&prod_data).unwrap();

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
                    let pa = load_price(&pd).unwrap();
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
