use crate::teju::{common, fmt};
use crate::teju::lut::f64::*;

/// The mantissa is represented by an unsigned integer the same size as the float (in this case,
/// u64 for f64).
pub type Mant = u64;
pub type Exp = common::Exp;

/// The **absolute value** of a finite `f64` decoded into exponent and mantissa.
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct Binary {
    exp: Exp,
    mant: Mant,
}

/// A decimal representation of the **absolute value** of a finite `f64`.
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct Decimal {
    exp: Exp,
    mant: Mant,
}

/// The result of running Tejú Jaguá on a finite `f64`.
#[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Result {
    sign: bool,
    decimal: Decimal,
}

// TODO strong typing to keep track of decimal/binary exponent/mantissa?

/// Calculates the result of `a * mult / 2^(2N)` without overflow, where `N` is the number of bits
/// of `a`, `mult.hi`, `mult.lo`.
#[inline]
const fn multiword_multiply_shift(a: Mant, mult: &common::Multiplier<Mant>) -> Mant {
    let result_hi = mult.hi as u128 * a as u128;
    let result_lo = mult.lo as u128 * a as u128;
    let result = (result_hi + (result_lo >> Mant::BITS)) >> Mant::BITS;
    result as Mant
}

/// Calculates the result of `multiword_multiply_shift(2^k, mult)` without overflow.
#[inline]
const fn multiword_multiply_shift_pow2(k: u32, mult: &common::Multiplier<Mant>) -> Mant {
    let s: Exp = k as Exp - Mant::BITS as Exp;
    if s <= 0 {
        mult.hi >> (-s as u32)
    } else {
        (mult.hi << s as u32) | mult.lo >> (-(k as Exp) as u32)
    }
}


/// Returns the lowest `n` bits of `x`.
pub const fn lsb(x: Mant, n: u32) -> Mant {
    x % (1 << n)
}

/// Checks if `n` is an even number, in which case a mantissa of `n` wins the tiebreak against its
/// neighbours (in a "round to nearest, ties to even" rounding rule).
#[inline]
pub const fn is_even(n: Mant) -> bool {
    n % 2 == 0
}

impl Binary {
    /// Number of bits in precision of the mantissa, including the implicit `1.`.
    const BITS_MANTISSA: u32 = 53;

    /// Number of bits of the mantissa that are actually stored.
    const BITS_MANTISSA_EXPLICIT: u32 = Self::BITS_MANTISSA - 1;

    /// The exponent bias, including the implicit factor of `2 ^ Self::BITS_MANTISSA` from treating
    /// the mantissa as a fixed-point decimal.
    const MIN_EXP: Exp = f64::MIN_EXP - Self::BITS_MANTISSA as i32;

    /// 1 + the maximum mantissa value storable in a float.
    const MAX_MANT: Mant = 1 << Self::BITS_MANTISSA_EXPLICIT;

    /// Decomposes a **finite** `f64` into the binary exponent and mantissa of its absolute
    /// value, i.e. such that `|num| = mant * 2^exp`.
    ///
    /// If `num` is infinite or NaN, returns an unspecified value; this is not checked except in
    /// debug assertions.
    #[inline]
    pub const fn new(num: f64) -> Self {
        debug_assert!(num.is_finite());

        let num = num.abs();
        let mut mant = lsb(num.to_bits(), Self::BITS_MANTISSA_EXPLICIT);
        let mut exp = (num.to_bits() >> Self::BITS_MANTISSA_EXPLICIT) as Exp;

        if exp != 0 {
            exp -= 1;
            mant |= 1 << Self::BITS_MANTISSA_EXPLICIT;
        }

        Binary{
            exp: exp + Self::MIN_EXP,
            mant,
        }
    }

    /// Returns the largest exponent `f` such that `10^f ≤ 2^self.exp`, i.e. the integer part of
    /// `log10(2^self.exp)`.
    #[inline]
    const fn exp_log10_pow2(&self) -> Exp {
        common::exp_log10_pow2(self.exp)
    }

    /// Returns `self.exp - e_0`, where `e_0` is the smallest exponent such that the integer part
    /// of `log10(2^e_0)` is equal to the integer part of `log10(2^self.exp)`.
    #[inline]
    const fn exp_log10_pow2_residual(&self) -> u32 {
        common::exp_log10_pow2_residual(self.exp)
    }

    /// Checks whether `self.mant` is a multiple of `2 ^ self.exp`.
    ///
    /// If not `0 ≤ self.exp < f64::BITS`, this returns an unspecified value.
    #[inline]
    const fn is_multiple_of_pow2(&self) -> bool {
        /*(self.mant >> self.exp) << self.exp == self.mant*/
        lsb(self.mant, self.exp as u32) == 0
    }

    /// Checks whether `self` is a "small integer", i.e. in the range of the contiguous integers
    /// representable by an `f64` without rounding.
    #[inline]
    const fn is_small_integer(&self) -> bool {
        // `self.exp` has to be in the interval [0; BITS_MANTISSA[, and `self` must be a clean
        // multiple of a power of 2 (with no information loss).
        let neg_exp = -self.exp;
        0 <= neg_exp && neg_exp < Self::BITS_MANTISSA as Exp
            && Binary{exp: neg_exp, .. *self}.is_multiple_of_pow2()
    }

    /// The core of Tejú Jaguá: finds the shortest decimal representation of `self` if it can, or
    /// the closest if it must.
    #[inline]
    /*const*/ fn teju_jagua_inner(self) -> Decimal {
        if self.mant == 0 { return Decimal { exp: 0, mant: 0 } }

        let exp_floor = self.exp_log10_pow2();
        let exp_residual = self.exp_log10_pow2_residual();
        // SAFETY: exp_floor is in bounds
        let mult = unsafe { MULTIPLIERS.get(exp_floor) };

        // Case 1: centered
        if self.mant != Self::MAX_MANT || self.exp == Self::MIN_EXP {
            let mant_a = (2 * self.mant - 1) << exp_residual;
            let mant_b = (2 * self.mant + 1) << exp_residual;
            let a = multiword_multiply_shift(mant_a, mult);
            let b = multiword_multiply_shift(mant_b, mult);
            let decimal_a = Decimal{ exp: exp_floor, mant: mant_a };
            let decimal_b = Decimal{ exp: exp_floor, mant: mant_b };

            let q = b / 10;
            let s = q * 10;
            if a < s {
                if s < b || is_even(self.mant) || !decimal_b.is_tie() {
                    return Decimal{exp: exp_floor + 1, mant: q }.remove_trailing_zeros()
                }
            } else if s == a && is_even(self.mant) && decimal_a.is_tie() {
                return Decimal{exp: exp_floor + 1, mant: q }.remove_trailing_zeros()
            } else if !is_even(a + b) {
                return Decimal{exp: exp_floor, mant: (a + b) / 2 + 1}
            }

            // Factor out these 5 lines
            let mant_c = (4 * self.mant) << exp_residual;
            let c2 = multiword_multiply_shift(mant_c, mult);
            let c = c2 / 2;

            let round_up = !(is_even(c2) || (is_even(c) && Decimal{exp: -exp_floor, mant: c2}.is_tie()));
            return Decimal{exp: exp_floor, mant: c + (round_up as Mant)}
        }

        // Case 2: uncentered
        else {
            // self.mant == Self::MAX_MANT
            let mant_a = (4 * Self::MAX_MANT - 1) << exp_residual;
            let mant_b = (2 * Self::MAX_MANT + 1) << exp_residual;
            let a = multiword_multiply_shift(mant_a, mult) / 2;
            let b = multiword_multiply_shift(mant_b, mult);
            let decimal_a = Decimal{ exp: exp_floor, mant: mant_a };
            let decimal_b = Decimal{ exp: exp_floor, mant: mant_b };

            if a < b {  // TODO calculation_sorted
                let q = b / 10;
                let s = q * 10;
                if a < s {
                    if s < b || is_even(Self::MAX_MANT) || !decimal_b.is_tie_uncentered() {
                        return Decimal{exp: exp_floor + 1, mant: q }.remove_trailing_zeros()
                    }
                } else if s == a && is_even(Self::MAX_MANT) && decimal_a.is_tie_uncentered() {
                    return Decimal{exp: exp_floor + 1, mant: q }.remove_trailing_zeros()
                } else if (a + b) % 2 == 1 {
                    return Decimal{exp: exp_floor, mant: (a + b) / 2 + 1}
                }

                let log2_mant_c = Self::BITS_MANTISSA + exp_residual + 1;
                let c2 = multiword_multiply_shift_pow2(log2_mant_c, mult);
                let c = c2 / 2;

                let round_up = 
                    (c == a && !decimal_a.is_tie_uncentered())
                    ||
                    !(is_even(c2) || (is_even(c) && Decimal{exp: -exp_floor, mant: c2}.is_tie()));
                return Decimal{exp: exp_floor, mant: c + (round_up as Mant)}
            } else if decimal_a.is_tie_uncentered() {
                return Decimal{exp: exp_floor, mant: a}.remove_trailing_zeros()
            } else {
                let mant_c = (40 * Self::MAX_MANT) << exp_residual;
                let c2 = multiword_multiply_shift(mant_c, mult);
                let c = c2 / 2;

                let round_up = !(is_even(c2) || (is_even(c) && Decimal{exp: -exp_floor, mant: c2}.is_tie()));
                return Decimal{exp: exp_floor - 1, mant: c + (round_up as Mant)}
            }
        }
    }

    /// The final Tejú Jaguá: short-circuits the "small integer" case.
    pub /*const*/ fn teju_jagua(self) -> Decimal {
        if self.is_small_integer() {
            debug_assert!(self.exp <= 0);
            return Decimal{exp: 0, mant: self.mant >> (-self.exp as u32)}.remove_trailing_zeros()
        }
        self.teju_jagua_inner()
    }
}

impl Decimal {
    #[inline]
    /*const*/ fn is_tie(&self) -> bool {
        0 <= self.exp && (self.exp as usize) < 27
            && self.is_multiple_of_pow5()
    }

    #[inline]
    /*const*/ fn is_tie_uncentered(&self) -> bool {
        self.mant % 5 == 0
            && 0 <= self.exp
            && self.is_multiple_of_pow5()
    }

    /// Checks whether `self.mant` is a "small" multiple of `5 ^ self.exp`.
    #[inline]
    /*const*/ fn is_multiple_of_pow5(&self) -> bool {
        // SAFETY: 
        let entry = unsafe { MULT_INVERSES.get(self.exp) };
        // self.mant * entry.multiplier <= entry.bound
        self.mant.wrapping_mul(entry.multiplier) <= entry.bound
    }

    /// Shortens `self` by removing trailing zeros from `self.mant` while possible, and
    /// incrementing `self.exp` by the same amount.
    const fn remove_trailing_zeros(mut self) -> Self {
        const M_INV5: Mant = -((Mant::MAX / 5) as i64) as Mant;
        const BOUND: Mant = Mant::MAX / 10 + 1;
        loop {
            // let q = (self.mant * M_INV5).rotate_right(1);
            let q = self.mant.wrapping_mul(M_INV5).rotate_right(1);
            if q >= BOUND {
                return self
            }
            self.exp += 1;
            self.mant = q;
        }
    }
}

impl Result {
    #[inline]
    pub fn new(num: f64) -> Self {
        debug_assert!(num.is_finite());
        Result{
            sign: num.is_sign_positive(),
            decimal: Binary::new(num).teju_jagua(),
        }
    }

    #[inline]
    pub unsafe fn format_exp(self, mut buf: *mut u8) -> usize {
        let buf_orig = buf;
        unsafe {
            buf.write(b'-');
            buf = buf.add(!self.sign as usize);

            *buf.add(2) = b'0';
            /*let mant_len = fmt::print_u64_mantissa(self.decimal.mant, buf.add(1));*/
            let mant_len = {
                let len = fmt::len_u64(self.decimal.mant);
                fmt::print_u64_mantissa_known_len(self.decimal.mant, buf.add(1), len)
            };

            *buf = *buf.add(1);
            *buf.add(1) = b'.';
            let mant_len_after_point = mant_len - 1;
            buf = buf.add(mant_len + ((mant_len_after_point > 0) as usize));

            *buf = b'e';
            let exp_len = fmt::print_i32_exp(self.decimal.exp + mant_len_after_point as i32, buf.add(1));

            buf.offset_from(buf_orig) as usize + 1 + exp_len
        }
    }

    /*#[inline]
    unsafe fn format_exp_fixed(sign: bool, decimal: Decimal, mut buf: *mut u8) -> usize {
        let buf_orig = buf;
        unsafe {
            if sign {
                fmt::write_char_to(b'+', &mut buf);
            } else {
                fmt::write_char_to(b'-', &mut buf);
            }

            let mut itoa_buf = itoa::Buffer::new();
            fmt::write_to(itoa_buf.format(decimal.mant).as_bytes(), &mut buf);
            fmt::write_char_to(b'e', &mut buf);
            fmt::write_to(itoa_buf.format(decimal.exp).as_bytes(), &mut buf);

            buf.offset_from(buf_orig) as usize
        }
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    mod binary {
        use super::*;

        /// Aux function, assert that `num` is decoded as `binary`; repeat for `-num`.
        fn assert_finite(num: f64, binary: Binary) {
            assert!(num.is_finite());
            assert_eq!(Binary::new(num.abs()), binary);
            assert_eq!(Binary::new(-num.abs()), binary);
        }

        #[test]
        fn extremes() {
            assert_finite(0.0, Binary{ exp: Binary::MIN_EXP, mant: 0 });
            assert_finite(4.94065645841246544177e-324, Binary{ exp: -1022-52, mant: 1 });
            assert_finite(f64::MIN_POSITIVE, Binary{ exp: -1022-52, mant: 1 << 52 });
            assert_finite(f64::MAX, Binary{ exp: 1023-52, mant: (1 << 53) - 1 });
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100_000))]
            
            #[test]
            fn float_roundtrip(
                float in f64::MIN .. f64::MAX,
            ) {
                let binary = Binary::new(float);
                let refloat = (2f64.powi(binary.exp) * binary.mant as f64).copysign(float);
                assert_eq!(refloat, float);
            }
        }
    }

    mod decimal {
        use super::*;

        /// Aux function, assert that `num` is decoded as a `Result` with the given `decimal`;
        /// repeat for `-num` (with the opposite sign).
        fn assert_finite(num: f64, decimal: Decimal) {
            assert!(num.is_finite());
            assert_eq!(Result::new(num.abs()), Result { sign: true, decimal });
            assert_eq!(Result::new(-num.abs()), Result { sign: false, decimal });
        }

        #[test]
        fn small() {
            assert_finite(123.456, Decimal{ exp: -3, mant: 123456 });
            assert_finite(0.1234, Decimal{ exp: -4, mant: 1234 });
            assert_finite(core::f64::consts::PI, Decimal{ exp: -15, mant: 3_141592653589793 });
            assert_finite(core::f64::consts::E, Decimal{ exp: -15, mant: 2_718281828459045 });
            assert_finite(core::f64::consts::LN_2, Decimal{ exp: -16, mant: 0_6931471805599453 });
        }

        #[test]
        fn small_integer() {
            assert_finite(123456., Decimal{ exp: 0, mant: 123456 });
            assert_finite(1., Decimal{ exp: 0, mant: 1 });
            assert_finite(123000123000., Decimal{ exp: 3, mant: 123000123 });
        }

        #[test]
        fn extremes() {
            assert_finite(0.0, Decimal{ exp: 0, mant: 0 });
            assert_finite(4.94065645841246544177e-324, Decimal{ exp: -324, mant: 5 });
            assert_finite(f64::MIN_POSITIVE, Decimal{ exp: -308-16, mant: 22250738585072014 });
            assert_finite(f64::MAX, Decimal{ exp: 308-16, mant: 17976931348623157 });
        }

        const INT_BOUND: i64 = (1u64 << Binary::BITS_MANTISSA) as i64;
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100_000))]
            
            #[test]
            fn integer_roundtrip(
                int in !INT_BOUND .. INT_BOUND,
            ) {
                let float = int as f64;
                assert_eq!(
                    Result::new(float),
                    Result{
                        sign: (int >= 0),
                        decimal: Decimal{ exp: 0, mant: int.unsigned_abs() }.remove_trailing_zeros(),
                    }
                )
            }
            
            #[test]
            fn float_roundtrip(
                float in f64::MIN .. f64::MAX,
            ) {
                let mut buf = crate::Buffer::new();
                let str = buf.format_exp(float);
                let refloat = str.parse().unwrap();
                assert_eq!(float, refloat)
            }
        }
    }

    mod string {
        use super::*;

        /// Aux function, assert that `num` is serialised as `str`, both via `format` and
        /// `format_finite`; repeat for `-num` being serialised as `-str`.
        fn assert_exp_finite(num: f64, str: &str) {
            assert!(num.is_finite());

            let str_neg = 
                if num.is_sign_positive() {
                    "-".to_string() + str
                } else {
                    str[1..].to_string()
                };

            assert_eq!(crate::Buffer::new().format_exp(num), str);
            assert_eq!(crate::Buffer::new().format_exp_finite(num), str);

            assert_eq!(crate::Buffer::new().format_exp(-num), str_neg.as_str());
            assert_eq!(crate::Buffer::new().format_exp_finite(-num), str_neg.as_str());
        }

        #[test]
        fn small() {
            assert_exp_finite(123.456, "1.23456e2");
            assert_exp_finite(0.1234, "1.234e-1");
            assert_exp_finite(core::f64::consts::PI, "3.141592653589793e0");
            assert_exp_finite(core::f64::consts::E, "2.718281828459045e0");
            assert_exp_finite(core::f64::consts::LN_2, "6.931471805599453e-1");
        }

        #[test]
        fn small_integer() {
            assert_exp_finite(123456., "1.23456e5");
            assert_exp_finite(1., "1.0");
            assert_exp_finite(123000123000., "1.23000123e11");
        }

        #[test]
        fn extremes() {
            assert_exp_finite(0.0, "0.0");
            assert_exp_finite(4.94065645841246544177e-324, "5e-324");
            assert_exp_finite(f64::MIN_POSITIVE, "2.2250738585072014e-308" );
            assert_exp_finite(f64::MAX, "1.7976931348623157e308" );
        }

        #[test]
        fn specials() {
            assert_eq!(crate::Buffer::new().format_exp(f64::NAN), "NaN");
            assert_eq!(crate::Buffer::new().format_exp(-f64::NAN), "NaN");
            assert_eq!(crate::Buffer::new().format_exp(f64::INFINITY), "inf");
            assert_eq!(crate::Buffer::new().format_exp(f64::NEG_INFINITY), "-inf");
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100_000))]
            
            /*#[test]
            fn ryu(
                float in f64::MIN .. f64::MAX,
            ) {
                assert_eq!(
                    crate::Buffer::new().format(float),
                    ryu::Buffer::new().format(float),
                )
            }*/
        }
    }
}