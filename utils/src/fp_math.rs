use std::convert::TryInto;

pub const FP_32_ONE: u64 = 1 << 32;

/// a is fp0, b is fp32 and result is a/b fp0
pub fn fp32_div(a: u64, b_fp32: u64) -> Option<u64> {
    ((a as u128) << 32)
        .checked_div(b_fp32 as u128)
        .and_then(|x| x.try_into().ok())
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn fp32_mul_floor(a: u64, b_fp32: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(|x| (x >> 32).try_into().ok())
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn fp32_mul_ceil(a: u64, b_fp32: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(fp32_ceil_util)
        .and_then(|x| (x >> 32).try_into().ok())
}

/// a is fp0, b is fp32 and result is a/b fp0
pub fn ifp32_div(a: i64, b_fp32: u64) -> Option<i64> {
    ((a.unsigned_abs() as u128) << 32)
        .checked_div(b_fp32 as u128)
        .and_then(|x| x.try_into().ok())
        .map(|x: i64| a.signum() * x)
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn ifp32_mul_floor(a: i64, b_fp32: u64) -> Option<i64> {
    (a.unsigned_abs() as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(|x| (x >> 32).try_into().ok())
        .map(|x: i64| a.signum() * x)
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn ifp32_mul_ceil(a: i64, b_fp32: u64) -> Option<i64> {
    (a.unsigned_abs() as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(fp32_ceil_util)
        .and_then(|x| (x >> 32).try_into().ok())
        .map(|x: i64| a.signum() * x)
}

/// a is fp0, b is fp64 and result is a/b fp0
pub fn fp64_div(a: u64, b_fp64: u64) -> Option<u64> {
    ((a as u128) << 64)
        .checked_div(b_fp64 as u128)
        .and_then(|x| x.try_into().ok())
}

/// a is fp0, b is fp64 and result is a*b fp0
pub fn fp64_mul_floor(a: u64, b_fp64: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp64 as u128)
        .map(|x| (x >> 64))
        .and_then(|x| x.try_into().ok())
}

/// a is fp0, b is fp64 and result is a*b fp0
pub fn fp64_mul_ceil(a: u64, b_fp64: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp64 as u128)
        .map(|x| (x >> 64))
        .and_then(fp64_ceil_util)
        .and_then(|x| x.try_into().ok())
}

#[inline(always)]
fn fp32_ceil_util(x_fp32: u128) -> Option<u128> {
    let add_one = (!(x_fp32 as u32)).wrapping_add(1) as u128;
    x_fp32.checked_add(add_one)
}

#[inline(always)]
fn fp64_ceil_util(x_fp64: u128) -> Option<u128> {
    let add_one = (!(x_fp64 as u64)).wrapping_add(1) as u128;
    x_fp64.checked_add(add_one)
}

#[test]
fn test() {
    // fp32_div
    assert_eq!(fp32_div(124345678765454, 45654 << 32).unwrap(), 2723653541);
    assert_eq!(fp32_div(124345678765454, 6787654 << 32).unwrap(), 18319389);

    // fp32_mul
    assert_eq!(
        fp32_mul_floor(5676543, 6787654 << 32).unwrap(),
        38530409800122
    );
    assert_eq!(fp32_mul_floor(12454, 45654 << 32).unwrap(), 568574916);
    assert_eq!(fp32_mul_floor(5, 1 << 31).unwrap(), 2);
    assert_eq!(
        fp32_mul_ceil(5676543, 6787654 << 32).unwrap(),
        38530409800122
    );
    assert_eq!(fp32_mul_ceil(12454, 45654 << 32).unwrap(), 568574916);
    assert_eq!(fp32_mul_ceil(5, 1 << 31).unwrap(), 3);

    // ifp32_div
    assert_eq!(ifp32_div(124345678765454, 6787654 << 32).unwrap(), 18319389);
    assert_eq!(ifp32_div(124345678765454, 45654 << 32).unwrap(), 2723653541);

    // ifp32_mul
    assert_eq!(
        ifp32_mul_floor(5676543, 6787654 << 32).unwrap(),
        38530409800122
    );
    assert_eq!(ifp32_mul_floor(12454, 45654 << 32).unwrap(), 568574916);

    // fp64_div
    assert_eq!(fp64_div(5676543, 345678909876543456).unwrap(), 302921968);
    assert_eq!(fp64_div(12454, 345678909876543456).unwrap(), 664592);

    // fp64_mul
    assert_eq!(fp64_mul_floor(5676543, 345678909876543456).unwrap(), 106374);
    assert_eq!(fp64_mul_floor(12454, 345678909876543456).unwrap(), 233)
}
