//! Rust implementation of [Tejú Jaguá](https://github.com/cassioneri/teju_jagua), a fast algorithm
//! for converting a floating point number to a (decimal) string.
//!
//! The interface mimics that of [Ryu](https://docs.rs/ryu/).
//!
//! ## Usage
//!
//! ```
//! let mut buffer = teju::Buffer::new();
//! let printed = buffer.format_finite(1.234);
//! assert_eq!(printed, "1.234");
//! ```
//!
//! Numbers whose decimal representation is short are written as decimals; numbers with lots of
//! zeroes are written in scientific notation. To force either way, use `format_dec` or
//! `format_exp`, respectively.
//!
//! ```
//! assert_eq!(teju::Buffer::new().format(1e3), "1000.0");
//! assert_eq!(teju::Buffer::new().format_dec(1e3), "1000.0");
//! assert_eq!(teju::Buffer::new().format_exp(1e3), "1e3");
//!
//! assert_eq!(teju::Buffer::new().format(1e30), "1e30");
//! assert_eq!(teju::Buffer::new().format_exp(1e30), "1e30");
//! assert_eq!(teju::Buffer::new().format_dec(1e30), "1000000000000000000000000000000.0");
//! ```
//!
//! ## Performance
//! 
//! ![Microbenchmark chart comparing teju with ryu and std](https://raw.githubusercontent.com/andrepd/teju-jagua-rs/master/microbench.png)

#![cfg_attr(not(test), no_std)]

use core::marker::PhantomData;

mod teju;
pub use teju::float::Float;
use teju::format::{self, Format};

/// Safe API for formatting floating point numbers to text.
///
/// ## Example
///
/// ```
/// let mut buffer = teju::Buffer::new();
/// let printed = buffer.format(1.234);
/// assert_eq!(printed, "1.234");
/// ```
#[derive(Clone, Copy)]
pub struct Buffer<F: Float, Fmt: Format> {
    float: PhantomData<F>,
    bytes: Fmt::Buffer,
}

const POS_INF: &str = "inf";
const NEG_INF: &str = "-inf";
const NAN: &str = "NaN";
const POS_ZERO: &str = "0.0";
const NEG_ZERO: &str = "-0.0";
const POS_ZERO_EXP: &str = "0e0";
const NEG_ZERO_EXP: &str = "-0e0";

impl<F: Float, Fmt: Format> Buffer<F, Fmt> {
    /// This is a cheap operation; you don't need to worry about reusing buffers for efficiency.
    pub fn new() -> Self {
        Buffer { float: PhantomData, bytes: Fmt::new_buffer() }
    }
}

impl<F: Float, Fmt: Format> Default for Buffer<F, Fmt> {
    /// This is a cheap operation; you don't need to worry about reusing buffers for efficiency.
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Float> Buffer<F, format::General> {
    /// Print a floating point `num` into this buffer, and return a reference to its string
    /// representation.
    ///
    /// The number is formatted as a decimal if it fits in a "small" number of characters, or in
    /// scientific notation otherwise.
    ///
    /// This function formats NaN as the string `"NaN"`, positive infinity as `"inf"`, and negative
    /// infinity as `"-inf"`, to match [core::fmt].
    ///
    /// If `num` is known to be finite, you may get better performance by calling the
    /// [Self::format_exp_finite] method instead of format to avoid the checks for special cases.
    ///
    /// ```
    /// assert_eq!(teju::Buffer::new().format(3.14159), "3.14159");
    /// assert_eq!(teju::Buffer::new().format(-1. / 0.), "-inf");
    /// ```
    pub fn format(&mut self, num: F) -> &str {
        match num.classify() {
            teju::float::FloatType::Finite => self.format_finite(num),
            teju::float::FloatType::PosInf => POS_INF,
            teju::float::FloatType::NegInf => NEG_INF,
            teju::float::FloatType::Nan => NAN,
        }
    }

    /// Print a floating point `num` into this buffer, and return a reference to its string
    /// representation, **provided that `num.is_finite()`**.
    ///
    /// The number is formatted as a decimal if it fits in a "small" number of characters, or in
    /// scientific notation otherwise.
    ///
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print a string with unspecified contents.
    ///
    /// ```
    /// assert_eq!(teju::Buffer::new().format_finite(3.14159), "3.14159");
    /// ```
    pub fn format_finite(&mut self, num: F) -> &str {
        match num.classify_finite() {
            teju::float::FiniteFloatType::PosZero => return POS_ZERO,
            teju::float::FiniteFloatType::NegZero => return NEG_ZERO,
            teju::float::FiniteFloatType::Nonzero => (),
        }
        let ptr = <format::General as teju::format::Sealed>::buffer_as_ptr(&mut self.bytes);
        let n = unsafe { num.format_general_finite_nonzero(ptr) };
        let slice = unsafe { core::slice::from_raw_parts(ptr, n) };
        debug_assert!(n <= self.bytes.len());
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

impl<F: Float> Buffer<F, format::Scientific> {
    /// Print a floating point `num` into this buffer in scientific notation, and return a
    /// reference to its string representation.
    ///
    /// The number is always formatted in the form `[mantissa]e[exponent]`, where `mantissa` is a
    /// number between 1 (inclusive) and 10 (exclusive), even if `exponent` is `0`.
    /// 
    /// This function formats NaN as the string `"NaN"`, positive infinity as `"inf"`, and negative
    /// infinity as `"-inf"`, to match [core::fmt].
    ///
    /// If `num` is known to be finite, you may get better performance by calling the
    /// [Self::format_exp_finite] method instead of format to avoid the checks for special cases.
    ///
    /// ```
    /// assert_eq!(teju::Buffer::new().format(137.035999177), "137.035999177");
    /// assert_eq!(teju::Buffer::new().format_exp(137.035999177), "1.37035999177e2");
    /// ```
    pub fn format_exp(&mut self, num: F) -> &str {
        match num.classify() {
            teju::float::FloatType::Finite => self.format_exp_finite(num),
            teju::float::FloatType::PosInf => POS_INF,
            teju::float::FloatType::NegInf => NEG_INF,
            teju::float::FloatType::Nan => NAN,
        }
    }

    /// Print a floating point `num` into this buffer in scientific notation, and return a
    /// reference to its string representation, provied that `num.is_finite()`.
    /// 
    /// The number is always formatted in the form `[mantissa]e[exponent]`, where `mantissa` is a
    /// number between 1 (inclusive) and 10 (exclusive), even if `exponent` is `0`.
    /// 
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print a string with unspecified contents.
    ///
    /// ```
    /// assert_eq!(teju::Buffer::new().format_finite(137.035999177), "137.035999177");
    /// assert_eq!(teju::Buffer::new().format_exp_finite(137.035999177), "1.37035999177e2");
    /// ```
    pub fn format_exp_finite(&mut self, num: F) -> &str {
        match num.classify_finite() {
            teju::float::FiniteFloatType::PosZero => return POS_ZERO_EXP,
            teju::float::FiniteFloatType::NegZero => return NEG_ZERO_EXP,
            teju::float::FiniteFloatType::Nonzero => (),
        }
        let ptr = <format::Scientific as teju::format::Sealed>::buffer_as_ptr(&mut self.bytes);
        let n = unsafe { num.format_exp_finite_nonzero(ptr) };
        let slice = unsafe { core::slice::from_raw_parts(ptr, n) };
        debug_assert!(n <= self.bytes.len());
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

impl<F: Float> Buffer<F, format::Decimal> {
    /// Print a floating point `num` into this buffer in decimal notation, and return a reference
    /// to its string representation.
    /// 
    /// The number is always formatted as `[integral part].[fractional part]`.
    /// 
    /// This function formats NaN as the string `"NaN"`, positive infinity as `"inf"`, and negative
    /// infinity as `"-inf"`, to match [core::fmt].
    ///
    /// If `num` is known to be finite, you may get better performance by calling the
    /// [Self::format_dec_finite] method instead of format to avoid the checks for special cases.
    ///
    /// ```
    /// assert_eq!(teju::Buffer::new().format(1.602176634e-19), "1.602176634e-19");
    /// assert_eq!(teju::Buffer::new().format_dec(1.602176634e-19), "0.0000000000000000001602176634");
    /// ```
    pub fn format_dec(&mut self, num: F) -> &str {
        match num.classify() {
            teju::float::FloatType::Finite => self.format_dec_finite(num),
            teju::float::FloatType::PosInf => POS_INF,
            teju::float::FloatType::NegInf => NEG_INF,
            teju::float::FloatType::Nan => NAN,
        }
    }

    /// Print a floating point `num` into this buffer in decimal notation, and return a reference
    /// to its string representation, provied that `num.is_finite()`.
    /// 
    /// The number is always formatted as `[integral part].[fractional part]`.
    /// 
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print a string with unspecified contents.
    ///
    /// ```
    /// assert_eq!(teju::Buffer::new().format(1.602176634e-19), "1.602176634e-19");
    /// assert_eq!(teju::Buffer::new().format_dec(1.602176634e-19), "0.0000000000000000001602176634");
    /// ```
    pub fn format_dec_finite(&mut self, num: F) -> &str {
        match num.classify_finite() {
            teju::float::FiniteFloatType::PosZero => return POS_ZERO,
            teju::float::FiniteFloatType::NegZero => return NEG_ZERO,
            teju::float::FiniteFloatType::Nonzero => (),
        }
        let ptr = <format::Decimal as teju::format::Sealed>::buffer_as_ptr(&mut self.bytes);
        let n = unsafe { num.format_dec_finite_nonzero(ptr) };
        let slice = unsafe { core::slice::from_raw_parts(ptr, n) };
        debug_assert!(n <= self.bytes.len());
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}
