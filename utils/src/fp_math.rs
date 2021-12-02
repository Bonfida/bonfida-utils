pub const FP_32_ONE: u64 = 1 << 32;

/// a is fp0, b is fp32 and result is a/b fp0
pub fn fp32_div(a: u64, b_fp32: u64) -> u64 {
    (((a as u128) << 32) / (b_fp32 as u128)) as u64
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn fp32_mul(a: u64, b_fp32: u64) -> u64 {
    (((a as u128) * (b_fp32 as u128)) >> 32) as u64
}

/// a is fp0, b is fp32 and result is a/b fp0
pub fn ifp32_div(a: i64, b_fp32: u64) -> i64 {
    a.signum() * ((((a.abs() as u128) << 32) / (b_fp32 as u128)) as i64)
}

/// a is fp0, b is fp32 and result is a*b fp0
pub fn ifp32_mul(a: i64, b_fp32: u64) -> i64 {
    a.signum() * (((a.abs() as u128) * (b_fp32 as u128)) >> 32) as i64
}

/// a is fp0, b is fp64 and result is a/b fp0
pub fn fp64_div(a: u64, b_fp64: u64) -> u64 {
    (((a as u128) << 64) / (b_fp64 as u128)) as u64
}

/// a is fp0, b is fp64 and result is a*b fp0
pub fn fp64_mul(a: u64, b_fp64: u64) -> u64 {
    (((a as u128) * (b_fp64 as u128)) >> 64) as u64
}
