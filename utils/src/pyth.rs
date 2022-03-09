use pyth_client::{load_price, PriceConf, Product};
use solana_program::{msg, program_error::ProgramError};

use crate::fp_math::safe_downcast;

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
pub fn test() {
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
