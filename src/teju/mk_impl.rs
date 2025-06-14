macro_rules! mk_impl { (
    float = $f:ident,
    mant = $mant:ident,
    mant_signed = $mant_signed:ident,
    mant_double = $mant_double:ident,
    len_mantissa = $len_mantissa:path,
    print_mantissa = $print_mantissa:path,
    print_mantissa_known_len = $print_mantissa_known_len:path,
    tests = {
        pi = {
            dec = $pi_dec:literal,
            exp = $pi_exp:literal,
            decimal = $pi_decimal:expr,
        },
        e = {
            dec = $e_dec:literal,
            exp = $e_exp:literal,
            decimal = $e_decimal:expr,
        },
        ln2 = {
            dec = $ln2_dec:literal,
            exp = $ln2_exp:literal,
            decimal = $ln2_decimal:expr,
        },
        min_subnormal = {
            dec = $min_subnormal_dec:literal,
            exp = $min_subnormal_exp:literal,
            decimal = $min_subnormal_decimal:expr,
        },
        min_normal = {
            dec = $min_normal_dec:literal,
            exp = $min_normal_exp:literal,
            decimal = $min_normal_decimal:expr,
        },
        max = {
            dec = $max_dec:literal,
            exp = $max_exp:literal,
            decimal = $max_decimal:expr,
        },
    }
) => {

use crate::teju::{common, fmt};

/// The mantissa is represented by an unsigned integer the same size as the float (in this case,
/// $m for $f).
pub type Mant = $mant;
pub type Exp = common::Exp;

/// The **absolute value** of a finite `$f` decoded into exponent and mantissa.
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct Binary {
    exp: Exp,
    mant: Mant,
}

/// A decimal representation of the **absolute value** of a finite `$f`.
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct Decimal {
    exp: Exp,
    mant: Mant,
}

/// The result of running Tejú Jaguá on a **finite**, **nonzero** `$f`.
#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq, Eq)]
pub struct Result {
    sign: bool,
    decimal: Decimal,
}

/// Calculates the result of `a * mult / 2^(2N)` without overflow, where `N` is the number of bits
/// of `a`, `mult.hi`, `mult.lo`.
#[inline]
const fn multiword_multiply_shift(a: Mant, mult: &common::Multiplier<Mant>) -> Mant {
    let result_hi = mult.hi as $mant_double * a as $mant_double;
    let result_lo = mult.lo as $mant_double * a as $mant_double;
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
    const BITS_MANTISSA: u32 = $f::MANTISSA_DIGITS;

    /// Number of bits of the mantissa that are actually stored.
    const BITS_MANTISSA_EXPLICIT: u32 = Self::BITS_MANTISSA - 1;

    /// The exponent bias, including the implicit factor of `2 ^ Self::BITS_MANTISSA` from treating
    /// the mantissa as a fixed-point decimal.
    const MIN_EXP: Exp = $f::MIN_EXP - Self::BITS_MANTISSA as i32;

    /// 1 + the maximum mantissa value storable in a float.
    const MAX_MANT: Mant = 1 << Self::BITS_MANTISSA_EXPLICIT;

    /// Decomposes a **finite** `$f` into the binary exponent and mantissa of its absolute
    /// value, i.e. such that `|num| = mant * 2^exp`.
    ///
    /// If `num` is infinite or NaN, returns an unspecified value; this is not checked except in
    /// debug assertions.
    #[inline]
    pub const fn new(num: $f) -> Self {
        if !cfg!(test) {
            debug_assert!(num.is_finite());
            debug_assert!(num.abs() != 0.0);
        }

        // Extract fields
        let num = num.abs();
        let mut mant = lsb(num.to_bits(), Self::BITS_MANTISSA_EXPLICIT);
        let mut exp = (num.to_bits() >> Self::BITS_MANTISSA_EXPLICIT) as Exp;

        // Normals have implicit unit (`1.xxx`) and -1 bias; subnormals don't
        if exp != 0 {
            exp -= 1;
            mant |= 1 << Self::BITS_MANTISSA_EXPLICIT;
        }

        // Add bias
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
    /// If not `0 ≤ self.exp < $f::BITS`, this returns an unspecified value.
    #[inline]
    const fn is_multiple_of_pow2(&self) -> bool {
        /*(self.mant >> self.exp) << self.exp == self.mant*/
        lsb(self.mant, self.exp as u32) == 0
    }

    /// Checks whether `self` is a "small integer", i.e. in the range of the contiguous integers
    /// representable by an `$f` without rounding.
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
    const unsafe fn teju_jagua_inner(self) -> Decimal {
        debug_assert!(!(self.mant == 0 && self.exp == Self::MIN_EXP));

        let exp_floor = self.exp_log10_pow2();
        let exp_residual = self.exp_log10_pow2_residual();
        // SAFETY: exp_floor is in bounds
        let mult = unsafe { lut::MULTIPLIERS.get(exp_floor) };

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
    pub const unsafe fn teju_jagua(self) -> Decimal {
        if self.is_small_integer() {
            debug_assert!(self.exp <= 0);
            return Decimal{exp: 0, mant: self.mant >> (-self.exp as u32)}.remove_trailing_zeros()
        }
        unsafe { self.teju_jagua_inner() }
    }
}

impl Decimal {
    #[inline]
    const fn is_tie(&self) -> bool {
        0 <= self.exp && (self.exp as usize) < lut::MULT_INVERSES.len()
            && self.is_multiple_of_pow5()
    }

    #[inline]
    const fn is_tie_uncentered(&self) -> bool {
        self.mant % 5 == 0
            && 0 <= self.exp
            && self.is_multiple_of_pow5()
    }

    /// Checks whether `self.mant` is a "small" multiple of `5 ^ self.exp`.
    #[inline]
    const fn is_multiple_of_pow5(&self) -> bool {
        // SAFETY: 
        let entry = unsafe { lut::MULT_INVERSES.get(self.exp) };
        // self.mant * entry.multiplier <= entry.bound
        self.mant.wrapping_mul(entry.multiplier) <= entry.bound
    }

    /// Shortens `self` by removing trailing zeros from `self.mant` while possible, and
    /// incrementing `self.exp` by the same amount.
    const fn remove_trailing_zeros(mut self) -> Self {
        const M_INV5: Mant = -((Mant::MAX / 5) as $mant_signed) as Mant;
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
    /// Uses Tejú Jaguá to find a decimal representation for a **finite** and **nonzero** `num`.
    ///
    /// If `num` is infinite, NaN, or ±0, this is undefined behaviour.
    #[inline]
    pub unsafe fn new(num: $f) -> Self {
        if !cfg!(test) {
            debug_assert!(num.is_finite());
            debug_assert!(num.abs() != 0.0);
        }
        // dbg!(num);
        // dbg!(Binary::new(num));
        // dbg!(Binary::new(num).teju_jagua());
        Result{
            sign: num.is_sign_positive(),
            decimal: unsafe { Binary::new(num).teju_jagua() },
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
                let len = $len_mantissa(self.decimal.mant as u64);
                $print_mantissa_known_len(self.decimal.mant as u64, buf.add(1), len)
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

    #[inline]
    pub unsafe fn format_general(self, mut buf: *mut u8) -> usize {
        unsafe {
            buf.write(b'-');
            buf = buf.add(!self.sign as usize);

            let mant_len = $len_mantissa(self.decimal.mant as u64);
            let decimal_exp = mant_len as i32 + self.decimal.exp;

            if self.decimal.exp >= 0 && decimal_exp <= 16 {  // Implies mant_len <= 16
                // 1234e7 -> 12340000000.0
                // Write mantissa, pad with zeros (up to 17 of them), write decimal point at
                // `decimal_exp`. Careful not to overflow 32 byte `buf`.
                $print_mantissa_known_len(self.decimal.mant as u64, buf, mant_len);
                core::ptr::write_bytes(buf.add(mant_len), b'0', 8);
                if mant_len < 8 { core::ptr::write_bytes(buf.add(mant_len + 8), b'0', 10) };
                *buf.add(decimal_exp as usize) = b'.';
                !self.sign as usize + decimal_exp as usize + 2
            } else if 0 < decimal_exp && decimal_exp <= 16 {
                // 1234e-1 -> 123.4
                // Write mantissa, shift digits after `decimal_exp` digit 1 place to the right,
                // write decimal point in between.
                debug_assert!(self.decimal.exp < 0);
                $print_mantissa_known_len(self.decimal.mant as u64, buf, mant_len);
                core::ptr::copy(
                    buf.add(decimal_exp as usize),
                    buf.add(decimal_exp as usize + 1),
                    -self.decimal.exp as usize,
                );
                *buf.add(decimal_exp as usize) = b'.';
                !self.sign as usize + mant_len + 1
            } else if -5 < decimal_exp && decimal_exp <= 0 {
                // 1234e-6 -> 0.001234
                // Pad with zeros (up to 7 of them), write decimal point at second digit, write
                // mantissa after.
                core::ptr::write_bytes(buf, b'0', 8);
                *buf.add(1) = b'.';
                let n_zeros = (2 - decimal_exp) as usize;
                $print_mantissa_known_len(self.decimal.mant as u64, buf.add(n_zeros), mant_len);
                (!self.sign as i32 + 2 - self.decimal.exp) as usize
            } else if mant_len == 1 {
                // 1e30
                // Write mantissa with no decimal point, then `e`, then exponent.
                *buf = b'0' + self.decimal.mant as u8;
                *buf.add(1) = b'e';
                let exp_len = fmt::print_i32_exp(decimal_exp - 1, buf.add(2));
                !self.sign as usize + 2 + exp_len
            } else {
                // 1234e30 -> 1.234e33
                // Write mantissa, shift first digit to add decimal point, then `e`, then exponent.
                $print_mantissa_known_len(self.decimal.mant as u64, buf.add(1), mant_len);
                *buf = *buf.add(1);
                *buf.add(1) = b'.';
                *buf.add(mant_len + 1) = b'e';                
                let exp_len = fmt::print_i32_exp(decimal_exp - 1, buf.add(2 + mant_len));
                !self.sign as usize + 2 + mant_len + exp_len
            }
        }
    }

    #[inline]
    pub unsafe fn format_dec(self, mut buf: *mut u8) -> usize {
        unsafe {
            buf.write(b'-');
            buf = buf.add(!self.sign as usize);

            let mant_len = $len_mantissa(self.decimal.mant as u64);
            let decimal_exp = mant_len as i32 + self.decimal.exp;

            if self.decimal.exp >= 0 {
                // 1234e7 -> 12340000000.0
                // Write mantissa, pad with zeros (in 8 byte chunks), write decimal point at
                // `decimal_exp`.
                $print_mantissa_known_len(self.decimal.mant as u64, buf, mant_len);
                let n_zeros = self.decimal.exp as usize + 2;
                core::ptr::write_bytes(buf.add(mant_len), b'0', n_zeros.next_multiple_of(8));
                *buf.add(decimal_exp as usize) = b'.';
                !self.sign as usize + decimal_exp as usize + 2
            } else if decimal_exp > 0 {
                // 1234e-1 -> 123.4
                // Write mantissa, shift digits after `decimal_exp` digit 1 place to the right,
                // write decimal point in between.
                $print_mantissa_known_len(self.decimal.mant as u64, buf, mant_len);
                core::ptr::copy(
                    buf.add(decimal_exp as usize),
                    buf.add(decimal_exp as usize + 1),
                    -self.decimal.exp as usize,
                );
                *buf.add(decimal_exp as usize) = b'.';
                !self.sign as usize + mant_len + 1
            } else {
                // 1234e-6 -> 0.001234
                // Pad with zeros (in 8 byte chunks), write decimal point at second digit, write
                // mantissa after.
                let n_zeros = (2 - decimal_exp) as usize;
                core::ptr::write_bytes(buf, b'0', n_zeros.next_multiple_of(8));
                *buf.add(1) = b'.';
                $print_mantissa_known_len(self.decimal.mant as u64, buf.add(n_zeros), mant_len);
                (!self.sign as i32 + 2 - self.decimal.exp) as usize
            }
        }
    }
}

use $crate::teju::float;

impl float::Sealed for $f {
    #[inline]
    fn classify(&self) -> float::FloatType {
        if self.is_finite() {
            float::FloatType::Finite
        } else if self.is_infinite() {
            if self.is_sign_positive() {float::FloatType::PosInf} else {float::FloatType::NegInf}
        } else {
            float::FloatType::Nan
        }
    }

    #[inline]
    fn classify_finite(&self) -> float::FiniteFloatType {
        if self.abs().to_bits() != 0 {
            float::FiniteFloatType::Nonzero
        } else {
            if self.is_sign_positive() {float::FiniteFloatType::PosZero} else {float::FiniteFloatType::NegZero}
        }
    }

    #[inline]
    unsafe fn format_general_finite_nonzero(self, buf: *mut u8) -> usize {
        unsafe { Result::new(self).format_general(buf) }
    }

    #[inline]
    unsafe fn format_exp_finite_nonzero(self, buf: *mut u8) -> usize {
        unsafe { Result::new(self).format_exp(buf) }
    }

    #[inline]
    unsafe fn format_dec_finite_nonzero(self, buf: *mut u8) -> usize {
        unsafe { Result::new(self).format_dec(buf) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    mod binary {
        use super::*;

        /// Aux function, assert that `num` is decoded as `binary`; repeat for `-num`.
        fn assert_finite(num: $f, binary: Binary) {
            assert!(num.is_finite());
            assert_eq!(Binary::new(num.abs()), binary);
            assert_eq!(Binary::new(-num.abs()), binary);
        }

        #[test]
        fn extremes() {
            // Zero
            assert_finite(0.0, Binary { exp: Binary::MIN_EXP, mant: 0 });
            // Smallest subnormal
            assert_finite($f::from_bits(1), Binary {
                exp: $f::MIN_EXP - $f::MANTISSA_DIGITS as Exp,
                mant: 1,
            });
            // Smallest normal
            assert_finite($f::MIN_POSITIVE, Binary {
                exp: $f::MIN_EXP - $f::MANTISSA_DIGITS as Exp,
                mant: Binary::MAX_MANT,
            });
            // Largest
            assert_finite($f::MAX, Binary {
                exp: $f::MAX_EXP - $f::MANTISSA_DIGITS as Exp,
                mant: (Binary::MAX_MANT << 1) - 1,
            });
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(200_000))]
            
            #[test]
            fn float_roundtrip(
                float in $f::MIN .. $f::MAX,
            ) {
                prop_assume!(float.abs() != 0.0);
                let binary = Binary::new(float);
                let refloat = ((2 as $f).powi(binary.exp) * binary.mant as $f).copysign(float);
                assert_eq!(refloat, float);
            }
        }
    }

    mod decimal {
        use super::*;

        /// Aux function, assert that `num` is decoded as a `Result` with the given `decimal`;
        /// repeat for `-num` (with the opposite sign).
        fn assert_finite(num: $f, decimal: Decimal) {
            assert!(num.is_finite());
            assert!(num.abs() != 0.0);
            assert_eq!(unsafe { Result::new(num.abs()) }, Result { sign: true, decimal });
            assert_eq!(unsafe { Result::new(-num.abs()) }, Result { sign: false, decimal });
        }

        #[test]
        fn small() {
            assert_finite(123.456, Decimal{ exp: -3, mant: 123456 });
            assert_finite(0.1234, Decimal{ exp: -4, mant: 1234 });
            assert_finite(core::$f::consts::PI, $pi_decimal);
            assert_finite(core::$f::consts::E, $e_decimal);
            assert_finite(core::$f::consts::LN_2, $ln2_decimal);
        }

        #[test]
        fn small_integer() {
            assert_finite(123456., Decimal{ exp: 0, mant: 123456 });
            assert_finite(1., Decimal{ exp: 0, mant: 1 });
            assert_finite(12003400., Decimal{ exp: 2, mant: 120034 });
        }

        #[test]
        fn extremes() {
            assert_finite($f::from_bits(1), $min_subnormal_decimal);
            assert_finite($f::MIN_POSITIVE, $min_normal_decimal);
            assert_finite($f::MAX, $max_decimal);
        }

        const INT_BOUND: $mant_signed = ((1 as $mant) << Binary::BITS_MANTISSA) as $mant_signed;
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(200_000))]
            
            #[test]
            fn integer_roundtrip(
                int in !INT_BOUND .. INT_BOUND,
            ) {
                prop_assume!(int != 0);
                let float = int as $f;
                assert_eq!(
                    unsafe { Result::new(float) },
                    Result{
                        sign: (int >= 0),
                        decimal: Decimal{ exp: 0, mant: int.unsigned_abs() }.remove_trailing_zeros(),
                    }
                )
            }
            
            /*#[test]
            fn float_roundtrip(
                float in f64::MIN .. f64::MAX,
            ) {
                prop_assume!(float.abs() != 0.0);
                let result = unsafe { Result::new(float) };
                let refloat = 10f64.powi(result.decimal.exp) * result.decimal.mant as f64;
                let refloat = if result.sign {refloat} else {-refloat};
                assert_eq!(float, refloat)
            }*/
        }
    }

    mod string {
        use super::*;

        /// Aux function, assert that `num` is serialised as `str` via `format{,_finite}`. Repeat
        /// for `-num`.
        fn assert_finite(num: $f, str: &str) {
            assert!(num.is_finite());
            assert_eq!(str.parse::<$f>().unwrap(), num, "Incorrect test case!");
            let str_neg = 
                if num.is_sign_positive() {
                    "-".to_string() + str
                } else {
                    str[1..].to_string()
                };

            assert_eq!(crate::Buffer::new().format(num), str);
            assert_eq!(crate::Buffer::new().format_finite(num), str);

            assert_eq!(crate::Buffer::new().format(-num), str_neg.as_str());
            assert_eq!(crate::Buffer::new().format_finite(-num), str_neg.as_str());

            assert_eq!(crate::Buffer::new().format(num), ryu::Buffer::new().format(num));
            assert_eq!(crate::Buffer::new().format(-num), ryu::Buffer::new().format(-num));
        }

        /// Aux function, assert that `num` is serialised as `str` via `format_exp{,_finite}`.
        /// Repeat for `-num`.
        fn assert_exp_finite(num: $f, str: &str) {
            assert!(num.is_finite());
            assert_eq!(str.parse::<$f>().unwrap(), num, "Incorrect test case!");
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

        /// Aux function, assert that `num` is serialised as `str` via `format_dec{,_finite}`.
        /// Repeat for `-num`.
        fn assert_dec_finite(num: $f, str: &str) {
            assert!(num.is_finite());
            assert_eq!(str.parse::<$f>().unwrap(), num, "Incorrect test case!");
            let str_neg = 
                if num.is_sign_positive() {
                    "-".to_string() + str
                } else {
                    str[1..].to_string()
                };

            assert_eq!(crate::Buffer::new().format_dec(num), str);
            assert_eq!(crate::Buffer::new().format_dec_finite(num), str);

            assert_eq!(crate::Buffer::new().format_dec(-num), str_neg.as_str());
            assert_eq!(crate::Buffer::new().format_dec_finite(-num), str_neg.as_str());
        }

        /// Aux function, assert that `num` is serialised as `str_general` via `format
        /// {,_finite}`, as `str_exp` via `format_exp{,_finite}`, and as `str_dec` via `format_dec
        /// {,_finite}`. Repeat for `-num`.
        fn assert_all_finite(num: $f, str_general: &str, str_exp: &str, str_dec: &str) {
            assert!(num.is_finite());
            assert_finite(num, str_general);
            assert_exp_finite(num, str_exp);
            assert_dec_finite(num, str_dec);
        }

        #[test]
        fn general() {
            assert_finite(1234e-30, "1.234e-27");
            assert_finite(1234e-6, "0.001234");
            assert_finite(1234e-4, "0.1234");
            assert_finite(1234e-2, "12.34");
            assert_finite(1234e0, "1234.0");
            assert_finite(1234e+2, "123400.0");
            assert_finite(1234e+7, "12340000000.0");
            assert_finite(1234e+30, "1.234e33");
            assert_finite(1e30, "1e30");
            if core::mem::size_of::<$f>() >= 64 {
                assert_finite(1234e+12, "1234000000000000.0");
                assert_finite(1234567890123456.0, "1234567890123456.0");
                assert_finite(1000000000000000.0, "1000000000000000.0");
            }
        }

        #[test]
        fn small() {
            assert_all_finite(
                123.456,
                "123.456",
                "1.23456e2",
                "123.456",
            );
            assert_all_finite(
                0.1234,
                "0.1234",
                "1.234e-1",
                "0.1234",
            );
            assert_all_finite(
                0.001234,
                "0.001234",
                "1.234e-3",
                "0.001234",
            );
            assert_all_finite(
                core::$f::consts::PI,
                $pi_dec,
                $pi_exp,
                $pi_dec,
            );
            assert_all_finite(
                core::$f::consts::E,
                $e_dec,
                $e_exp,
                $e_dec,
            );
            assert_all_finite(
                core::$f::consts::LN_2,
                $ln2_dec,
                $ln2_exp,
                $ln2_dec,
            );
        }

        #[test]
        fn small_integer() {
            assert_all_finite(
                123456.,
                "123456.0",
                "1.23456e5",
                "123456.0",
            );
            assert_all_finite(
                1.,
                "1.0",
                "1e0",
                "1.0",
            );
            assert_all_finite(
                12003400.,
                "12003400.0",
                "1.20034e7",
                "12003400.0",
            );
        }

        #[test]
        fn extremes() {
            assert_all_finite(0.0,
                "0.0",
                "0e0",
                "0.0",
            );
            assert_all_finite($f::from_bits(1),
                $min_subnormal_exp,
                $min_subnormal_exp,
                $min_subnormal_dec,
            );
            assert_all_finite($f::MIN_POSITIVE,
                $min_normal_exp,
                $min_normal_exp,
                $min_normal_dec,
            );
            assert_all_finite($f::MAX,
                $max_exp,
                $max_exp,
                $max_dec,
            );
        }

        #[test]
        fn specials() {
            for (value, str) in [
                ($f::NAN, "NaN"),
                (-$f::NAN, "NaN"),
                ($f::INFINITY, "inf"),
                ($f::NEG_INFINITY, "-inf"),
            ] {
                assert_eq!(crate::Buffer::new().format(value), str);
                assert_eq!(crate::Buffer::new().format_exp(value), str);
                assert_eq!(crate::Buffer::new().format_dec(value), str);
                // These should produce unspecified strings
                core::hint::black_box(crate::Buffer::new().format_finite(value));
                core::hint::black_box(crate::Buffer::new().format_exp_finite(value));
                core::hint::black_box(crate::Buffer::new().format_dec_finite(value));
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(800_000))]
                        
            #[test]
            fn nan_payloads(
                sign in 0 as Mant ..= 1,
                payload in 1 as Mant .. Binary::MAX_MANT,
            ) {
                let bits_exp = Mant::BITS - Binary::BITS_MANTISSA_EXPLICIT - 1;
                let exp = ((1 << bits_exp) - 1) << Binary::BITS_MANTISSA_EXPLICIT;
                let float = $f::from_bits(sign | exp | payload);
                assert!(float.is_nan(), "Incorrect test case!");

                assert_eq!(crate::Buffer::new().format(float), "NaN");
                assert_eq!(crate::Buffer::new().format_exp(float), "NaN");
                assert_eq!(crate::Buffer::new().format_dec(float), "NaN");
                // These should produce unspecified strings
                core::hint::black_box(crate::Buffer::new().format_finite(float));
                core::hint::black_box(crate::Buffer::new().format_exp_finite(float));
                core::hint::black_box(crate::Buffer::new().format_dec_finite(float));
            }

            #[test]
            fn float_roundtrip_general(
                float in $f::MIN .. $f::MAX,
            ) {
                let mut buf = crate::Buffer::new();
                let str = buf.format(float);
                let refloat = str.parse::<$f>().unwrap();
                assert_eq!(float, refloat)
            }
            
            #[test]
            fn float_roundtrip_exp(
                float in $f::MIN .. $f::MAX,
            ) {
                let mut buf = crate::Buffer::new();
                let str = buf.format_exp(float);
                let refloat = str.parse::<$f>().unwrap();
                assert_eq!(float, refloat)
            }
            
            #[test]
            fn float_roundtrip_dec(
                float in $f::MIN .. $f::MAX,
            ) {
                let mut buf = crate::Buffer::new();
                let str = buf.format_dec(float);
                let refloat = str.parse::<$f>().unwrap();
                assert_eq!(float, refloat)
            }

            #[test]
            fn ryu(
                float in $f::MIN .. $f::MAX
            ) {
                assert_eq!(
                    crate::Buffer::new().format(float),
                    ryu::Buffer::new().format(float),
                )
            }
        }
    }
}

}} // mk_impl

pub(crate) use mk_impl;
