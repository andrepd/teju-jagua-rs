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

pub trait Sealed
where
    Self: core::panic::RefUnwindSafe + Send + Sync + Unpin + core::panic::UnwindSafe 
{
    fn classify(&self) -> FloatType;
    fn classify_finite(&self) -> FiniteFloatType;

    unsafe fn format_general_finite_nonzero(self, buf: *mut u8) -> usize;
    unsafe fn format_exp_finite_nonzero(self, buf: *mut u8) -> usize;
    unsafe fn format_dec_finite_nonzero(self, buf: *mut u8) -> usize;
}
