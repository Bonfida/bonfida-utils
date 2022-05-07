pub const FP_32_ONE: u64 = 1 << 32;

/// a is fp0, b is fp32 and result is a/b fp0
pub fn fp32_div(a: u64, b_fp32: u64) -> Option<u64> {
    ((a as u128) << 32)
        .checked_div(b_fp32 as u128)
        .and_then(safe_downcast)
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn fp32_mul(a: u64, b_fp32: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(|x| safe_downcast(x >> 32))
}

/// a is fp0, b is fp32 and result is a/b fp0
pub fn ifp32_div(a: i64, b_fp32: u64) -> Option<i64> {
    ((a.abs() as u128) << 32)
        .checked_div(b_fp32 as u128)
        .and_then(safe_downcast)
        .map(|x| a.signum() * (x as i64))
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn ifp32_mul(a: i64, b_fp32: u64) -> Option<i64> {
    (a.abs() as u128)
        .checked_mul(b_fp32 as u128)
        .and_then(|x| safe_downcast(x >> 32))
        .map(|x| a.signum() * x as i64)
}

/// a is fp0, b is fp64 and result is a/b fp0
pub fn fp64_div(a: u64, b_fp64: u64) -> Option<u64> {
    ((a as u128) << 64)
        .checked_div(b_fp64 as u128)
        .and_then(safe_downcast)
}

/// a is fp0, b is fp64 and result is a*b fp0
pub fn fp64_mul(a: u64, b_fp64: u64) -> Option<u64> {
    (a as u128)
        .checked_mul(b_fp64 as u128)
        .map(|x| (x >> 64))
        .and_then(safe_downcast)
}

pub fn safe_downcast(n: u128) -> Option<u64> {
    static BOUND: u128 = u64::MAX as u128;
    if n > BOUND {
        None
    } else {
        Some(n as u64)
    }
}

#[test]
fn test() {
    // fp32_div
    assert_eq!(fp32_div(124345678765454, 45654 << 32).unwrap(), 2723653541);
    assert_eq!(fp32_div(124345678765454, 6787654 << 32).unwrap(), 18319389);

    // fp32_mul
    assert_eq!(fp32_mul(5676543, 6787654 << 32).unwrap(), 38530409800122);
    assert_eq!(fp32_mul(12454, 45654 << 32).unwrap(), 568574916);

    // ifp32_div
    assert_eq!(ifp32_div(124345678765454, 6787654 << 32).unwrap(), 18319389);
    assert_eq!(ifp32_div(124345678765454, 45654 << 32).unwrap(), 2723653541);

    // ifp32_mul
    assert_eq!(ifp32_mul(5676543, 6787654 << 32).unwrap(), 38530409800122);
    assert_eq!(ifp32_mul(12454, 45654 << 32).unwrap(), 568574916);

    // fp64_div
    assert_eq!(fp64_div(5676543, 345678909876543456).unwrap(), 302921968);
    assert_eq!(fp64_div(12454, 345678909876543456).unwrap(), 664592);

    // fp64_mul
    assert_eq!(fp64_mul(5676543, 345678909876543456).unwrap(), 106374);
    assert_eq!(fp64_mul(12454, 345678909876543456).unwrap(), 233)
}
