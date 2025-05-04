mod common;
mod mk_impl;

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

pub trait Sealed
{
    type Buffer;
    fn new_buffer() -> Self::Buffer;
    const BUFFER_LEN: usize;
    fn buffer_as_ptr(buf: &mut Self::Buffer) -> *mut u8;

    fn classify(&self) -> FloatType;
    unsafe fn format_exp_finite(self, buf: *mut u8) -> usize;
}

impl Sealed for f64 {
    type Buffer = [core::mem::MaybeUninit<u8>; 24];

    fn new_buffer() -> Self::Buffer {
        [core::mem::MaybeUninit::uninit(); 24]
    }

    const BUFFER_LEN: usize = 24;

    fn buffer_as_ptr(buf: &mut Self::Buffer) -> *mut u8 {
        buf.as_mut_ptr() as *mut u8
    }

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

    unsafe fn format_exp_finite(self, buf: *mut u8) -> usize {
        let result = mk_impl::Result::new(self);
        unsafe { result.format_exp(buf) }
    }
}
