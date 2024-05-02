#[cfg(test)]
mod tests {

    use solana_program::hash::hashv;

    use bonfida_macros::compute_hashv;

    #[test]
    pub fn functional() {
        let record_hash = compute_hashv!("ETH");
        let record_hash_ref = hashv(&["SPL Name Service\x01ETH".as_bytes()]).to_bytes();
        assert_eq!(record_hash, record_hash_ref);
    }
}
