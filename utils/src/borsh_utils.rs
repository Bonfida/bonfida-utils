use borsh::BorshSerialize;

pub fn borsh_save<T: BorshSerialize>(obj: &T, mut dst: &mut [u8]) -> Result<(), std::io::Error> {
    obj.serialize(&mut dst)
}

#[cfg(test)]
mod tests {
    use super::borsh_save;
    use borsh::BorshSerialize;
    #[derive(BorshSerialize)]
    struct TestStruct {
        a: u8,
        b: u64,
    }

    #[test]
    fn serialize() {
        let mut buffer = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        let s = TestStruct { a: 1, b: 18 };
        borsh_save(&s, &mut buffer).unwrap();
        assert_eq!(buffer, [1u8, 18, 0, 0, 0, 0, 0, 0, 0]);
    }
}
