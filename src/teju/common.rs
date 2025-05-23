//! Routines and types that are *shared* between all implementations.

/// The exponent is represented by an i32 regardless of underlying type; this is sufficiently wide
/// to accomodate the exponent of any floating point format.
pub type Exp = i32;

//

pub const EXP_LOG10_POW2_BOUNDS: core::ops::RangeInclusive<i32> = -112815 ..= 112815;

/// Returns the largest exponent `f` such that `10^f ≤ 2^e`, i.e. the integer part of
/// `log_10(2^e)`.
///
/// Uses an euclidean approximation that is only valid in the range [EXP_LOG10_POW2_BOUNDS]. If
/// `exp` is not in that range, the result is unspecified.
#[inline]
pub const fn exp_log10_pow2(exp: i32) -> i32 {
    debug_assert!(*EXP_LOG10_POW2_BOUNDS.start() <= exp && exp <= *EXP_LOG10_POW2_BOUNDS.end());
    let x = 1292913987i64 * exp as i64;
    (x >> 32) as i32
}

/// Returns the largest exponent `f` such that `10^f ≤ 2^e`, i.e. the integer part of
/// `log_10(2^e)`.
///
/// Uses an euclidean approximation that is only valid in the range [EXP_LOG10_POW2_BOUNDS]. If
/// `exp` is not in that range, the result is unspecified.
#[inline]
pub const fn exp_log10_pow2_residual(exp: i32) -> u32 {
    debug_assert!(*EXP_LOG10_POW2_BOUNDS.start() <= exp && exp <= *EXP_LOG10_POW2_BOUNDS.end());
    let x = 1292913987i64 * exp as i64;
    x as u32 / 1292913987u32
}

//

pub struct Multipliers<T, const N: usize, const MIN_EXP: i32> (
    [Multiplier<T>; N]
);

pub struct Multiplier<T> {
    pub hi: T,
    pub lo: T,
}

impl<T, const N: usize, const MIN_EXP: i32> Multipliers<T, N, MIN_EXP> {
    const OFFSET: i32 = exp_log10_pow2(MIN_EXP);

    pub const fn new(table: [Multiplier<T>; N]) -> Self {
        Self(table)
    }

    pub const unsafe fn get(&self, exp_floor: i32) -> &Multiplier<T> {
        let idx = exp_floor - Self::OFFSET;
        debug_assert!(0 <= idx && idx < N as i32);
        unsafe { &*self.0.as_ptr().add(idx as usize) }
    }
}

pub struct MultInverses<T, const N: usize> (
    [MultInverse<T>; N]
);

pub struct MultInverse<T> {
    pub multiplier: T,
    pub bound: T,
}

impl<T, const N: usize> MultInverses<T, N> {
    pub const fn len(&self) -> usize { N }

    pub const fn new(table: [MultInverse<T>; N]) -> Self {
        Self(table)
    }

    pub const unsafe fn get(&self, exp: i32) -> &MultInverse<T> {
        debug_assert!(0 <= exp && exp < N as i32);
        unsafe { &*self.0.as_ptr().add(exp as usize) }
    }
}
