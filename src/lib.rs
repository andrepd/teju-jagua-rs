//! Rust implementation of [Tejú Jaguá](https://github.com/cassioneri/teju_jagua), a fast algorithm
//! for converting a floating point number to a (decimal) string.
//!
//! The interface mimics that of [Ryu](https://docs.rs/ryu/).

#![cfg_attr(not(test), no_std)]

mod teju;
pub use teju::Float;

/// Safe API for formatting floating point numbers to text.
///
/// ## Example
///
/// ```
/// let mut buffer = teju::Buffer::new();
/// let printed = buffer.format_exp_finite(1.234);
/// assert_eq!(printed, "1.234e0");
/// ```
#[derive(Clone, Copy)]
pub struct Buffer<F: Float> {
    bytes: F::Buffer,
}

/*#[derive(Debug)]
pub enum Format {
    // General,
    // Decimal,
    Scientific,
}*/

const POS_INF: &str = "inf";
const NEG_INF: &str = "-inf";
const NAN: &str = "NaN";
const POS_ZERO: &str = "0.0";
const NEG_ZERO: &str = "-0.0";
const POS_ZERO_EXP: &str = "0e0";
const NEG_ZERO_EXP: &str = "-0e0";

impl<F: Float> Buffer<F> {
    /// This is a cheap operation; you don't need to worry about reusing buffers for efficiency.
    pub fn new() -> Self {
        Buffer { bytes: F::new_buffer() }
    }

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
            teju::FloatType::Finite => self.format_finite(num),
            teju::FloatType::PosInf => POS_INF,
            teju::FloatType::NegInf => NEG_INF,
            teju::FloatType::Nan => NAN,
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
            teju::FiniteFloatType::PosZero => return POS_ZERO,
            teju::FiniteFloatType::NegZero => return NEG_ZERO,
            teju::FiniteFloatType::Nonzero => (),
        }
        let n = unsafe { num.format_general_finite_nonzero(F::buffer_as_ptr(&mut self.bytes)) };
        let slice = unsafe { core::slice::from_raw_parts(F::buffer_as_ptr(&mut self.bytes), n) };
        debug_assert!(n <= F::BUFFER_LEN);
        unsafe { core::str::from_utf8_unchecked(slice) }
    }

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
            teju::FloatType::Finite => self.format_exp_finite(num),
            teju::FloatType::PosInf => POS_INF,
            teju::FloatType::NegInf => NEG_INF,
            teju::FloatType::Nan => NAN,
        }
    }

    /// Print a floating point `num` into this buffer in scientific notation, and return a
    /// reference to its string representation within the buffer, provied that `num.is_finite()`.
    /// 
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print an unspecified (but valid) string.
    pub fn format_exp_finite(&mut self, num: F) -> &str {
        match num.classify_finite() {
            teju::FiniteFloatType::PosZero => return POS_ZERO_EXP,
            teju::FiniteFloatType::NegZero => return NEG_ZERO_EXP,
            teju::FiniteFloatType::Nonzero => (),
        }
        let n = unsafe { num.format_exp_finite_nonzero(F::buffer_as_ptr(&mut self.bytes)) };
        let slice = unsafe { core::slice::from_raw_parts(F::buffer_as_ptr(&mut self.bytes), n) };
        debug_assert!(n <= F::BUFFER_LEN);
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}
