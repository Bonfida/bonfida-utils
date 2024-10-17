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
        pub b: &'a mut [u32],
        pub c: &'a mut [u128],
    }

    #[derive(WrappedPod, PartialEq, Debug)]
    pub struct TestStruct<'a> {
        pub a: &'a u64,
        pub b: &'a [u32],
        pub c: &'a [u128],
    }

    #[derive(WrappedPodMut, PartialEq, Debug)]
    pub struct TestStructMut2<'a> {
        pub a: &'a mut u64,
        pub b: &'a mut str,
        pub c: &'a mut str,
    }

    #[derive(WrappedPod, PartialEq, Debug)]
    pub struct TestStruct2<'a> {
        pub a: &'a u64,
        pub b: &'a str,
        pub c: &'a str,
    }

    #[test]
    pub fn test_mut() {
        let a = &mut rand::random();
        let b = &mut rand::random::<[u32; 4]>();
        let c = &mut rand::random::<[u128; 7]>();
        let o = TestStructMut { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());
        let o2 = TestStructMut::from_bytes(&mut buf);
        assert_eq!(o2, o)
    }

    #[test]
    pub fn test() {
        let a = &rand::random();
        let b = &rand::random::<[u32; 4]>();
        let c = &rand::random::<[u128; 7]>();
        let o = TestStruct { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());
        let o2 = TestStruct::from_bytes(&buf);
        assert_eq!(o2, o)
    }

    #[test]
    pub fn test_mut_2() {
        let a = &mut rand::random();
        let b = &mut (0..10).map(|_| rand::random::<char>()).collect::<String>();
        let c = &mut (0..15).map(|_| rand::random::<char>()).collect::<String>();
        let o = TestStructMut2 { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());
        let o2 = TestStructMut2::from_bytes(&mut buf);
        assert_eq!(o2, o)
    }

    #[test]
    pub fn test_2() {
        let a = &rand::random();
        let b = &(0..10).map(|_| rand::random::<char>()).collect::<String>();
        let c = &(0..15).map(|_| rand::random::<char>()).collect::<String>();
        let o = TestStruct2 { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());
        let o2 = TestStruct2::from_bytes(&buf);
        assert_eq!(o2, o)
    }
}
