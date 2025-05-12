/// A floating point type which [teju](crate) can serialise into a string.
///
/// This trait is "sealed", meaning it cannot be implemented for any other types.
pub trait Float: Sealed {}
impl Float for f64 {}

#[derive(Debug)]
pub enum FloatType {
    Finite,
    PosInf,
    NegInf,
    Nan,
}

#[derive(Debug)]
pub enum FiniteFloatType {
    Nonzero,
    PosZero,
    NegZero,
}

pub trait Sealed {
    fn classify(&self) -> FloatType;
    fn classify_finite(&self) -> FiniteFloatType;

    unsafe fn format_general_finite_nonzero(self, buf: *mut u8) -> usize;
    unsafe fn format_exp_finite_nonzero(self, buf: *mut u8) -> usize;
    unsafe fn format_dec_finite_nonzero(self, buf: *mut u8) -> usize;
}

impl Sealed for f64 {
    #[inline]
    fn classify(&self) -> FloatType {
        if self.is_finite() {
            FloatType::Finite
        } else if self.is_infinite() {
            if self.is_sign_positive() {FloatType::PosInf} else {FloatType::NegInf}
        } else {
            FloatType::Nan
        }
    }

    #[inline]
    fn classify_finite(&self) -> FiniteFloatType {
        if self.abs().to_bits() != 0 {
            FiniteFloatType::Nonzero
        } else {
            if self.is_sign_positive() {FiniteFloatType::PosZero} else {FiniteFloatType::NegZero}
        }
    }

    #[inline]
    unsafe fn format_general_finite_nonzero(self, buf: *mut u8) -> usize {
        unsafe { crate::teju::f64::Result::new(self).format_general(buf) }
    }

    #[inline]
    unsafe fn format_exp_finite_nonzero(self, buf: *mut u8) -> usize {
        unsafe { crate::teju::f64::Result::new(self).format_exp(buf) }
    }

    #[inline]
    unsafe fn format_dec_finite_nonzero(self, buf: *mut u8) -> usize {
        unsafe { crate::teju::f64::Result::new(self).format_dec(buf) }
    }
}
