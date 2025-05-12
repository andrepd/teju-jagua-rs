//! Rust implementation of [Tejú Jaguá](https://github.com/cassioneri/teju_jagua), a fast algorithm
//! for converting a floating point number to a (decimal) string.
//!
//! The interface mimics that of [Ryu](https://docs.rs/ryu/).

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
/// let printed = buffer.format_finite(1.234);
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

impl<F: Float> Buffer<F, format::General> {
    /// Print a floating point `num` into this buffer, and return a reference to its string
    /// representation within the buffer. The number is formatted as a decimal if it fits in
    /// a "small" number of characters, or in scientific notation otherwise.
    ///
    /// This function formats NaN as the string `"NaN"`, positive infinity as `"inf"`, and negative
    /// infinity as `"-inf"`, to match [std::fmt].
    ///
    /// If `num` is known to be finite, you may get better performance by calling the
    /// [Self::format_exp_finite] method instead of format to avoid the checks for special cases.
    pub fn format(&mut self, num: F) -> &str {
        match num.classify() {
            teju::float::FloatType::Finite => self.format_finite(num),
            teju::float::FloatType::PosInf => POS_INF,
            teju::float::FloatType::NegInf => NEG_INF,
            teju::float::FloatType::Nan => NAN,
        }
    }

    /// Print a floating point `num` into this buffer, and return a reference to its string
    /// representation within the buffer, **provided that `num.is_finite()`**. The number is
    /// formatted as a decimal if it fits in a "small" number of characters, or in scientific
    /// notation otherwise.
    ///
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print an unspecified (but valid) string.
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
    /// reference to its string representation within the buffer.
    /// 
    /// This function formats NaN as the string `"NaN"`, positive infinity as `"inf"`, and negative
    /// infinity as `"-inf"`, to match [std::fmt].
    ///
    /// If `num` is known to be finite, you may get better performance by calling the
    /// [Self::format_exp_finite] method instead of format to avoid the checks for special cases.
    pub fn format_exp(&mut self, num: F) -> &str {
        match num.classify() {
            teju::float::FloatType::Finite => self.format_exp_finite(num),
            teju::float::FloatType::PosInf => POS_INF,
            teju::float::FloatType::NegInf => NEG_INF,
            teju::float::FloatType::Nan => NAN,
        }
    }

    /// Print a floating point `num` into this buffer in scientific notation, and return a
    /// reference to its string representation within the buffer, provied that `num.is_finite()`.
    /// 
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print an unspecified (but valid) string.
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
    /// Print a floating point `num` into this buffer in scientific notation, and return a
    /// reference to its string representation within the buffer.
    /// 
    /// This function formats NaN as the string `"NaN"`, positive infinity as `"inf"`, and negative
    /// infinity as `"-inf"`, to match [std::fmt].
    ///
    /// If `num` is known to be finite, you may get better performance by calling the
    /// [Self::format_dec_finite] method instead of format to avoid the checks for special cases.
    pub fn format_dec(&mut self, num: F) -> &str {
        match num.classify() {
            teju::float::FloatType::Finite => self.format_dec_finite(num),
            teju::float::FloatType::PosInf => POS_INF,
            teju::float::FloatType::NegInf => NEG_INF,
            teju::float::FloatType::Nan => NAN,
        }
    }

    /// Print a floating point `num` into this buffer in scientific notation, and return a
    /// reference to its string representation within the buffer, provied that `num.is_finite()`.
    /// 
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print an unspecified (but valid) string.
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
