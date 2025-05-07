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
            teju::FloatType::PosInf => "inf",
            teju::FloatType::NegInf => "-inf",
            teju::FloatType::Nan => "NaN",
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
        let n = unsafe { num.format_general_finite(F::buffer_as_ptr(&mut self.bytes)) };
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
            teju::FloatType::PosInf => "inf",
            teju::FloatType::NegInf => "-inf",
            teju::FloatType::Nan => "NaN",
        }
    }

    /// Print a floating point `num` into this buffer in scientific notation, and return a
    /// reference to its string representation within the buffer, provied that `num.is_finite()`.
    /// 
    /// This function **does not** check that `num` is indeed finite, for performance reasons; in
    /// this case it will print an unspecified (but valid) string.
    pub fn format_exp_finite(&mut self, num: F) -> &str {
        let n = unsafe { num.format_exp_finite(F::buffer_as_ptr(&mut self.bytes)) };
        let slice = unsafe { core::slice::from_raw_parts(F::buffer_as_ptr(&mut self.bytes), n) };
        debug_assert!(n <= F::BUFFER_LEN);
        unsafe { core::str::from_utf8_unchecked(slice) }
    }
}

