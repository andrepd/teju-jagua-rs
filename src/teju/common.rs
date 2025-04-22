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

pub struct Multipliers<T, const N: usize> (
    [Multiplier<T>; N]
);

pub struct Multiplier<T> {
    pub hi: T,
    pub lo: T,
}

impl<T, const N: usize> Multipliers<T, N> {
    // pub const LEN: usize = N;

    const OFFSET: i32 = -324; // TODO

    pub const fn new(table: [Multiplier<T>; N]) -> Self {
        Self(table)
    }

    pub unsafe fn get(&self, exp_floor: i32) -> &Multiplier<T> {
        let idx = exp_floor - Self::OFFSET;
        // debug_assert!(0 <= idx && idx < N as i32);
        unsafe { self.0.get_unchecked(idx as usize) }
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
    // pub const LEN: usize = N;

    pub const fn new(table: [MultInverse<T>; N]) -> Self {
        Self(table)
    }

    pub unsafe fn get(&self, exp_floor: i32) -> &MultInverse<T> {
        // debug_assert!(0 <= exp_floor && exp_floor < N as i32);
        unsafe { self.0.get_unchecked(exp_floor as usize) }
    }
}

//

#[inline]
pub unsafe fn write_to(str: &[u8], buf: &mut *mut u8) -> usize {
    unsafe { buf.copy_from(str.as_ptr(), str.len()) }
    *buf = unsafe { buf.add(str.len()) };
    str.len()
}

#[inline]
pub unsafe fn write_char_to(char: u8, buf: &mut *mut u8) -> usize {
    unsafe { buf.write(char) };
    *buf = unsafe { buf.add(1) };
    1
}

/// The result of running Tejú Jaguá on a float value.
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub enum Result<Decimal> {
    Finite {sign: bool, decimal: Decimal},
    Nan,
    Inf {sign: bool},
}
