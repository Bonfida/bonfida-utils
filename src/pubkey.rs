#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bonfida_macros::pubkey;
    use solana_program::pubkey::Pubkey;
    #[test]
    fn functional_0() {
        let a = Pubkey::from_str("perpke6JybKfRDitCmnazpCrGN5JRApxxukhA9Js6E6").unwrap();
        let b = pubkey!("perpke6JybKfRDitCmnazpCrGN5JRApxxukhA9Js6E6");
        assert_eq!(a, b);
    }
}
