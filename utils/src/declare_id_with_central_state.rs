#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use solana_program::pubkey::Pubkey;

    mod test_context {
        use bonfida_macros::declare_id_with_central_state;
        declare_id_with_central_state!("perpke6JybKfRDitCmnazpCrGN5JRApxxukhA9Js6E6");
    }

    #[test]
    pub fn functional() {
        let program_id = Pubkey::from_str("perpke6JybKfRDitCmnazpCrGN5JRApxxukhA9Js6E6").unwrap();
        let (central_state, central_state_nonce) =
            Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);
        assert_eq!(central_state, test_context::central_state::KEY);
        assert_eq!(central_state_nonce, test_context::central_state::NONCE);
        assert_eq!(program_id, test_context::ID);
    }
}
