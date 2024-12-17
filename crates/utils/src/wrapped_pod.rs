pub trait WrappedPod<'a> {
    fn export(&self, buffer: &mut Vec<u8>);
    fn size(&self) -> usize;
    fn from_bytes(buffer: &'a [u8]) -> Self;
    #[allow(unused_variables)]
    fn try_from_bytes(buffer: &'a [u8]) -> Result<Box<Self>, std::io::Error> {
        // Default implementation for backward comp
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }
}

pub trait WrappedPodMut<'a> {
    fn export(&self, buffer: &mut Vec<u8>);
    fn size(&self) -> usize;
    fn from_bytes(buffer: &'a mut [u8]) -> Self;
    #[allow(unused_variables)]
    fn try_from_bytes(buffer: &'a mut [u8]) -> Result<Box<Self>, std::io::Error> {
        // Default implementation for backward comp
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Not implemented yet",
        ))
    }
}

#[cfg(test)]
pub mod tests {

    use super::{WrappedPod, WrappedPodMut};
    use bonfida_macros::{WrappedPod, WrappedPodMut};
    use std::mem::{size_of, size_of_val};

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
        assert_eq!(o2, o);

        let o2_try = TestStructMut::try_from_bytes(&mut buf).unwrap();
        assert_eq!(*o2_try, o);
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
        assert_eq!(o2, o);

        let o2_try = TestStruct::try_from_bytes(&buf).unwrap();
        assert_eq!(*o2_try, o);
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
        assert_eq!(o2, o);

        let o2_try = TestStructMutStr::try_from_bytes(&mut buf).unwrap();
        assert_eq!(*o2_try, o);
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
        assert_eq!(o2, o);
        let o2_try = TestStructStr::try_from_bytes(&buf).unwrap();
        assert_eq!(*o2_try, o);
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

        let o_try = CompatTestStructNew::try_from_bytes(&buf_old).unwrap();
        assert_eq!(*o_try, o_new_reference);
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
        {
            let o_new = CompatTestStructMutNew::from_bytes(&mut buf_old);
            assert_eq!(o_new, o_new_reference);
        }

        let o_try = CompatTestStructMutNew::try_from_bytes(&mut buf_old).unwrap();
        assert_eq!(*o_try, o_new_reference)
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

        let o_try = CompatTestStructNewStr::try_from_bytes(&buf_old).unwrap();
        assert_eq!(*o_try, o_new_reference);
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
        {
            let o_new = CompatTestStructMutNewStr::from_bytes(&mut buf_old);
            assert_eq!(o_new, o_new_reference);
        }

        let o_try = CompatTestStructMutNewStr::try_from_bytes(&mut buf_old).unwrap();
        assert_eq!(*o_try, o_new_reference);
    }

    #[test]
    pub fn test_try_from_bytes_success() {
        // TestStruct (immutable)
        let a = &rand::random();
        let b = &rand::random::<[u32; 4]>();
        let c = &rand::random::<[u128; 7]>();
        let o = TestStruct { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);

        // Should succeed
        let o2 = TestStruct::try_from_bytes(&buf).unwrap();
        assert_eq!(*o2, o);

        // TestStructMut (mutable)
        let a_mut = &mut rand::random();
        let b_mut = &mut rand::random::<[u32; 4]>();
        let c_mut = &mut rand::random::<[u128; 7]>();
        let o_mut = TestStructMut {
            a: a_mut,
            b: b_mut,
            c: c_mut,
        };
        let mut buf_mut = Vec::with_capacity(o_mut.size());
        o_mut.export(&mut buf_mut);

        // Should succeed
        let o2_mut = TestStructMut::try_from_bytes(&mut buf_mut).unwrap();
        assert_eq!(*o2_mut, o_mut);

        // TestStructStr
        let a_str = &rand::random();
        let b_str = &(0..10).map(|_| rand::random::<char>()).collect::<String>();
        let c_str = &(0..15).map(|_| rand::random::<char>()).collect::<String>();
        let o_str = TestStructStr {
            a: a_str,
            b: b_str,
            c: c_str,
        };
        let mut buf_str = Vec::with_capacity(o_str.size());
        o_str.export(&mut buf_str);

        // Should succeed
        let o2_str = TestStructStr::try_from_bytes(&buf_str).unwrap();
        assert_eq!(*o2_str, o_str);

        // TestStructMutStr
        let a_mut_str = &mut rand::random();
        let b_mut_str = &mut (0..10).map(|_| rand::random::<char>()).collect::<String>();
        let c_mut_str = &mut (0..15).map(|_| rand::random::<char>()).collect::<String>();
        let o_mut_str = TestStructMutStr {
            a: a_mut_str,
            b: b_mut_str,
            c: c_mut_str,
        };
        let mut buf_mut_str = Vec::with_capacity(o_mut_str.size());
        o_mut_str.export(&mut buf_mut_str);

        // Should succeed
        let o2_mut_str = TestStructMutStr::try_from_bytes(&mut buf_mut_str).unwrap();
        assert_eq!(*o2_mut_str, o_mut_str);
    }

    #[test]
    pub fn test_try_from_bytes_truncated_buffer() {
        let a = &rand::random();
        let b = &rand::random::<[u32; 4]>();
        let c = &rand::random::<[u128; 7]>();
        let o = TestStruct { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);

        // Truncate the buffer intentionally
        let truncated_len = buf.len() - 5;
        buf.truncate(truncated_len);

        // Now try_from_bytes should fail due to unexpected EOF
        let res = TestStruct::try_from_bytes(&buf);
        assert!(
            res.is_err(),
            "try_from_bytes should fail with truncated buffer"
        );
    }

    #[test]
    pub fn test_try_from_bytes_invalid_cast() {
        // We'll test an invalid scenario. For instance, let's take a structure with a slice
        // and corrupt the length field so it tries to read beyond the buffer length.

        let a = &rand::random::<u64>();
        let b = &rand::random::<[u32; 2]>();
        let c = &rand::random::<[u128; 1]>();
        let o = TestStruct { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);

        // The buffer for TestStruct is structured as:
        // a (8 bytes) + length for b (8 bytes) + contents of b + length for c (8 bytes) + contents of c
        // Let's corrupt the length of b in the buffer to something large, causing an error.

        // at offset 8 we have the length of b slice in bytes (which should be 4 * sizeof(u32) = 16 bytes)
        // Let's set it to something huge.
        if buf.len() > 16 {
            buf[8..16].copy_from_slice(&(10_000u64.to_le_bytes())); // Set a huge length for b
        }

        // Now try_from_bytes should fail due to buffer too short
        let res = TestStruct::try_from_bytes(&buf);
        assert!(
            res.is_err(),
            "try_from_bytes should fail with invalid length"
        );
        let err = res.err().unwrap();

        assert_eq!(
            err.kind(),
            std::io::ErrorKind::UnexpectedEof,
            "Expected UnexpectedEof error due to invalid length"
        );
    }

    #[test]
    pub fn test_try_from_bytes_invalid_utf8() {
        // Test invalid UTF-8 scenario.
        // We'll use a TestStructStr, export it, and then corrupt the string content to invalid UTF-8.

        let a = &rand::random::<u64>();
        let b = &"HelloWorld".to_string(); // 10 bytes
        let c = &"AnotherTest".to_string(); // 11 bytes
        let o = TestStructStr { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);

        // Corrupt one byte in the b string region to invalid UTF-8.
        // The structure is: a (8 bytes), length of b (8 bytes), b bytes (10 bytes), length of c (8 bytes), c bytes (11 bytes)
        // After the first 8 bytes (a), the next 8 are length of b, then 10 bytes of b. Let's corrupt one byte in the b region.

        let b_start = 8 + 8; // after a and length of b
                             // Replace one byte in 'HelloWorld' (ASCII) with 0xFF (invalid in UTF-8)
        if buf.len() > b_start {
            buf[b_start] = 0xFF;
        }

        // Now try_from_bytes should fail due to invalid UTF-8
        let res = TestStructStr::try_from_bytes(&buf);
        assert!(
            res.is_err(),
            "try_from_bytes should fail with invalid UTF-8"
        );
        let err = res.err().unwrap();
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::InvalidData,
            "Expected InvalidData error due to invalid UTF-8"
        );
    }

    #[test]
    pub fn test_try_from_bytes_empty_buffer() {
        // try_from_bytes should fail if the buffer is empty or too small for even the first field
        let buf: Vec<u8> = Vec::new();

        let res = TestStruct::try_from_bytes(&buf);
        assert!(res.is_err(), "try_from_bytes should fail with empty buffer");
        let err = res.err().unwrap();
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::UnexpectedEof,
            "Expected UnexpectedEof for empty buffer"
        );
    }

    #[test]
    pub fn test_try_from_bytes_same_as_from_bytes_on_valid_data() {
        // We want to ensure that if the data is valid, try_from_bytes and from_bytes behave identically.
        let a = &rand::random::<u64>();
        let b = &rand::random::<[u32; 4]>();
        let c = &rand::random::<[u128; 7]>();
        let o = TestStruct { a, b, c };
        let mut buf = Vec::with_capacity(o.size());
        o.export(&mut buf);

        let o_from = TestStruct::from_bytes(&buf);
        let o_try = TestStruct::try_from_bytes(&buf).unwrap();
        assert_eq!(
            o_from, *o_try,
            "from_bytes and try_from_bytes differ on valid data"
        );

        let a_mut = &mut rand::random::<u64>();
        let b_mut = &mut rand::random::<[u32; 4]>();
        let c_mut = &mut rand::random::<[u128; 7]>();
        let o_mut = TestStructMut {
            a: a_mut,
            b: b_mut,
            c: c_mut,
        };
        let mut buf_mut = Vec::with_capacity(o_mut.size());
        o_mut.export(&mut buf_mut);

        let mut buf_mut_clone = buf_mut.clone();
        let o_from_mut = TestStructMut::from_bytes(&mut buf_mut_clone);
        let o_try_mut = TestStructMut::try_from_bytes(&mut buf_mut).unwrap();
        assert_eq!(
            o_from_mut, *o_try_mut,
            "from_bytes and try_from_bytes differ on valid data (mutable)"
        );
    }

    #[test]
    pub fn test_try_from_bytes_back_compat() {
        // Ensure that try_from_bytes can work with old data as well, mirroring from_bytes tests.
        let a = &rand::random();
        let b = &rand::random::<[u32; 4]>();
        let o_old_reference = CompatTestStructOld { a, b };
        let o_new_reference = CompatTestStructNew { a, b };
        let mut buf_new = Vec::with_capacity(o_new_reference.size());
        o_new_reference.export(&mut buf_new);

        // Old struct doesn't implement try_from_bytes (uses default which returns NotImplemented)
        // So we only test the new struct on the old buffer
        let o_old = CompatTestStructOld::try_from_bytes(&buf_new);
        // The old struct is derived using old macros without try_from_bytes implementation
        // It should return Err(Unsupported) by default
        assert!(o_old.is_err());
        let err = o_old.err().unwrap();
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::Unsupported,
            "Old struct without try_from_bytes should return Unsupported"
        );

        // Now test the new struct on old-structured buffer
        // We'll export old reference using the old macro struct and parse with new macro struct.
        let mut buf_old = Vec::with_capacity(o_old_reference.size());
        o_old_reference.export(&mut buf_old);

        // The new struct should parse the old buffer correctly.
        let o_new = CompatTestStructNew::try_from_bytes(&buf_old)
            .expect("try_from_bytes should succeed for new struct on old buffer");
        assert_eq!(*o_new, o_new_reference);
    }

    #[test]
    pub fn test_try_from_bytes_back_compat_str() {
        // Same logic for str
        let a = &rand::random();
        let b = &(0..10).map(|_| rand::random::<char>()).collect::<String>();
        let o_old_reference = CompatTestStructOldStr { a, b };
        let o_new_reference = CompatTestStructNewStr { a, b };
        let mut buf_new = Vec::with_capacity(o_new_reference.size());
        o_new_reference.export(&mut buf_new);

        let o_old = CompatTestStructOldStr::try_from_bytes(&buf_new);
        // Old struct uses old macros without try_from_bytes implementation
        assert!(o_old.is_err());
        let err = o_old.err().unwrap();
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);

        let mut buf_old = Vec::with_capacity(o_old_reference.size());
        o_old_reference.export(&mut buf_old);

        // The new struct should parse the old buffer correctly using try_from_bytes
        let o_new = CompatTestStructNewStr::try_from_bytes(&buf_old).unwrap();
        assert_eq!(*o_new, o_new_reference);
    }
}
