use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub trait BorshSize: BorshDeserialize + BorshSerialize {
    fn borsh_len(&self) -> usize;
}

impl BorshSize for u8 {
    fn borsh_len(&self) -> usize {
        1
    }
}

impl BorshSize for u16 {
    fn borsh_len(&self) -> usize {
        2
    }
}

impl BorshSize for u32 {
    fn borsh_len(&self) -> usize {
        4
    }
}

impl BorshSize for u64 {
    fn borsh_len(&self) -> usize {
        8
    }
}

impl BorshSize for u128 {
    fn borsh_len(&self) -> usize {
        16
    }
}

impl BorshSize for i8 {
    fn borsh_len(&self) -> usize {
        1
    }
}

impl BorshSize for i16 {
    fn borsh_len(&self) -> usize {
        2
    }
}

impl BorshSize for i32 {
    fn borsh_len(&self) -> usize {
        4
    }
}

impl BorshSize for i64 {
    fn borsh_len(&self) -> usize {
        8
    }
}

impl BorshSize for i128 {
    fn borsh_len(&self) -> usize {
        16
    }
}

impl BorshSize for Pubkey {
    fn borsh_len(&self) -> usize {
        32
    }
}

impl BorshSize for String {
    fn borsh_len(&self) -> usize {
        4 + self.len()
    }
}

impl<T: BorshSize> BorshSize for Vec<T> {
    fn borsh_len(&self) -> usize {
        if self.is_empty() {
            4
        } else {
            4 + self.len() * self[0].borsh_len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BorshSize;
    use bonfida_macros::BorshSize;
    use borsh::{BorshDeserialize, BorshSerialize};
    use solana_program::pubkey::Pubkey;

    #[derive(BorshSerialize, BorshDeserialize, BorshSize)]
    struct TestStruct {
        a: u8,
        b: u16,
        c: u32,
        d: u64,
        e: u128,
        f: Pubkey,
        g: [u8; 32],
        h: [u64; 4],
    }

    #[derive(BorshSerialize, BorshDeserialize, BorshSize)]
    enum TestEnum {
        FirstVariant,
        SecondVariant,
    }

    #[test]
    fn functional() {
        let s = TestStruct {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: Pubkey::new_unique(),
            g: [0; 32],
            h: [0; 4],
        };
        assert_eq!(s.borsh_len(), 1 + 2 + 4 + 8 + 16 + 32 + 32 + 32);

        let v = TestEnum::FirstVariant;
        assert_eq!(v.borsh_len(), 1);
    }
}
