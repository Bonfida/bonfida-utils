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
    pub struct TestStructMutStr<'a> {
        pub a: &'a mut u64,
        pub b: &'a mut str,
        pub c: &'a mut str,
    }

    #[derive(WrappedPod, PartialEq, Debug)]
    pub struct TestStructStr<'a> {
        pub a: &'a u64,
        pub b: &'a str,
        pub c: &'a str,
    }

    #[derive(bonfida_macros_old::WrappedPodMut, PartialEq, Debug)]
    pub struct CompatTestStructMutOld<'a> {
        pub a: &'a mut u64,
        pub b: &'a mut [u32],
    }

    #[derive(bonfida_macros_old::WrappedPod, PartialEq, Debug)]
    pub struct CompatTestStructOld<'a> {
        pub a: &'a u64,
        pub b: &'a [u32],
    }

    #[derive(WrappedPodMut, PartialEq, Debug)]
    pub struct CompatTestStructMutNew<'a> {
        pub a: &'a mut u64,
        pub b: &'a mut [u32],
    }

    #[derive(WrappedPod, PartialEq, Debug)]
    pub struct CompatTestStructNew<'a> {
        pub a: &'a u64,
        pub b: &'a [u32],
    }

    #[derive(bonfida_macros_old::WrappedPodMut, PartialEq, Debug)]
    pub struct CompatTestStructMutOldStr<'a> {
        pub a: &'a mut u64,
        pub b: &'a mut str,
    }

    #[derive(bonfida_macros_old::WrappedPod, PartialEq, Debug)]
    pub struct CompatTestStructOldStr<'a> {
        pub a: &'a u64,
        pub b: &'a str,
    }

    #[derive(WrappedPodMut, PartialEq, Debug)]
    pub struct CompatTestStructMutNewStr<'a> {
        pub a: &'a mut u64,
        pub b: &'a mut str,
    }

    #[derive(WrappedPod, PartialEq, Debug)]
    pub struct CompatTestStructNewStr<'a> {
        pub a: &'a u64,
        pub b: &'a str,
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
        assert_eq!(
            buf.len(),
            size_of::<u64>() + 8 + size_of_val(o.b) + size_of_val(o.c)
        );
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
        assert_eq!(
            buf.len(),
            size_of::<u64>() + 8 + size_of_val(o.b) + size_of_val(o.c)
        );
        let o2 = TestStruct::from_bytes(&buf);
        assert_eq!(o2, o)
    }

    #[test]
    pub fn test_mut_2() {
        let a = &mut rand::random();
        let b = &mut (0..10).map(|_| rand::random::<char>()).collect::<String>();
        let c = &mut (0..15).map(|_| rand::random::<char>()).collect::<String>();
        let o = TestStructMutStr { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());

        assert_eq!(buf.len(), size_of::<u64>() + 8 + o.b.len() + o.c.len());
        let o2 = TestStructMutStr::from_bytes(&mut buf);
        assert_eq!(o2, o)
    }

    #[test]
    pub fn test_2() {
        let a = &rand::random();
        let b = &(0..10).map(|_| rand::random::<char>()).collect::<String>();
        let c = &(0..15).map(|_| rand::random::<char>()).collect::<String>();
        let o = TestStructStr { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);
        assert_eq!(buf.len(), o.size());
        assert_eq!(buf.len(), size_of::<u64>() + 8 + o.b.len() + o.c.len());
        let o2 = TestStructStr::from_bytes(&buf);
        assert_eq!(o2, o)
    }

    #[test]
    pub fn test_back_compat() {
        let a = &rand::random();
        let b = &rand::random::<[u32; 4]>();
        let o_old_reference = CompatTestStructOld { a, b };
        let o_new_reference = CompatTestStructNew { a, b };
        let mut buf_new = Vec::with_capacity(o_new_reference.size());
        o_new_reference.export(&mut buf_new);
        let o_old = CompatTestStructOld::from_bytes(&buf_new);
        assert_eq!(o_old, o_old_reference);

        let mut buf_old = Vec::with_capacity(o_old.size());
        o_old_reference.export(&mut buf_old);
        let o_new = CompatTestStructNew::from_bytes(&buf_old);
        assert_eq!(o_new, o_new_reference);
    }

    #[test]
    pub fn test_back_compat_mut() {
        let a: &mut u64 = &mut rand::random();
        let b = &mut rand::random::<[u32; 4]>();
        let a_clone = &mut a.clone();
        let b_clone = &mut b.clone();
        let o_old_reference = CompatTestStructMutOld {
            a: a_clone,
            b: b_clone,
        };
        let o_new_reference = CompatTestStructMutNew { a, b };
        let mut buf_new = Vec::with_capacity(o_new_reference.size());
        o_new_reference.export(&mut buf_new);
        let o_old = CompatTestStructMutOld::from_bytes(&mut buf_new);
        assert_eq!(o_old, o_old_reference);

        let mut buf_old = Vec::with_capacity(o_old.size());
        o_old_reference.export(&mut buf_old);
        let o_new = CompatTestStructMutNew::from_bytes(&mut buf_old);
        assert_eq!(o_new, o_new_reference);
    }

    #[test]
    pub fn test_back_compat_str() {
        let a = &rand::random();
        let b = &(0..10).map(|_| rand::random::<char>()).collect::<String>();
        let o_old_reference = CompatTestStructOldStr { a, b };
        let o_new_reference = CompatTestStructNewStr { a, b };
        let mut buf_new = Vec::with_capacity(o_new_reference.size());
        o_new_reference.export(&mut buf_new);
        let o_old = CompatTestStructOldStr::from_bytes(&buf_new);
        assert_eq!(o_old, o_old_reference);

        let mut buf_old = Vec::with_capacity(o_old.size());
        o_old_reference.export(&mut buf_old);
        let o_new = CompatTestStructNewStr::from_bytes(&buf_old);
        assert_eq!(o_new, o_new_reference);
    }

    #[test]
    pub fn test_back_compat_mut_str() {
        let a: &mut u64 = &mut rand::random();
        let b = &mut (0..10).map(|_| rand::random::<char>()).collect::<String>();
        let a_clone = &mut a.clone();
        let b_clone = &mut b.clone();
        let o_old_reference = CompatTestStructMutOldStr {
            a: a_clone,
            b: b_clone,
        };
        let o_new_reference = CompatTestStructMutNewStr { a, b };
        let mut buf_new = Vec::with_capacity(o_new_reference.size());
        o_new_reference.export(&mut buf_new);
        let o_old = CompatTestStructMutOldStr::from_bytes(&mut buf_new);
        assert_eq!(o_old, o_old_reference);

        let mut buf_old = Vec::with_capacity(o_old.size());
        o_old_reference.export(&mut buf_old);
        let o_new = CompatTestStructMutNewStr::from_bytes(&mut buf_old);
        assert_eq!(o_new, o_new_reference);
    }
}
