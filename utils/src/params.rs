pub trait InstructionParams<'a> {
    fn write_instruction_data(&self, buffer: &mut Vec<u8>);
    fn size(&self) -> usize;
    fn parse_instruction_data(buffer: &'a [u8]) -> Self;
}

#[cfg(test)]
pub mod tests {

    use super::InstructionParams;
    use bonfida_macros::InstructionParams;

    #[derive(InstructionParams, PartialEq, Debug)]
    pub struct TestStruct<'a> {
        pub a: &'a u64,
        pub b: &'a [u128],
    }

    #[test]
    pub fn test_0() {
        let a = &42;
        let b = &[198, 2987239847, 234820357, 45];
        let o = TestStruct { a, b };
        let mut buf = Vec::with_capacity(o.size());
        o.write_instruction_data(&mut buf);
        assert_eq!(buf.len(), o.size());
        let o2 = TestStruct::parse_instruction_data(&buf);
        assert_eq!(o2, o)
    }
}
