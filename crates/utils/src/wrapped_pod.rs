pub trait WrappedPod<'a> {
    fn export(&self, buffer: &mut Vec<u8>);
    fn size(&self) -> usize;
    fn from_bytes(buffer: &'a [u8]) -> Self;
}

pub trait WrappedPodMut<'a> {
    fn export(&self, buffer: &mut Vec<u8>);
    fn size(&self) -> usize;
    fn from_bytes(buffer: &'a mut [u8]) -> Self;
}

#[cfg(test)]
pub mod tests {

    use super::{WrappedPod, WrappedPodMut};
    use bonfida_macros::{WrappedPod, WrappedPodMut};

    #[derive(WrappedPodMut, PartialEq, Debug)]
    pub struct TestStructMut<'a> {
        pub a: &'a mut u64,
        pub b: &'a mut [u128],
    }

    #[derive(WrappedPod, PartialEq, Debug)]
    pub struct TestStruct<'a> {
        pub a: &'a u64,
        pub b: &'a [u128],
    }

    #[test]
    pub fn test_mut() {
        let a = &mut 42;
        let b = &mut [198, 2987239847, 234820357, 45];
        let o = TestStructMut { a, b };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());
        let o2 = TestStructMut::from_bytes(&mut buf);
        assert_eq!(o2, o)
    }

    #[test]
    pub fn test() {
        let a = &mut 42;
        let b = &mut [198, 2987239847, 234820357, 45];
        let o = TestStruct { a, b };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());
        let o2 = TestStruct::from_bytes(&buf);
        assert_eq!(o2, o)
    }
}
