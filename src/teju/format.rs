/// A format for serialising floats.
///
/// This trait is "sealed", meaning it cannot be implemented for any other types.
pub trait Format: Sealed {}
impl Format for General {}
impl Format for Scientific {}
impl Format for Decimal {}

pub struct General;
pub struct Scientific;
pub struct Decimal;

pub trait Sealed {
    type Buffer;
    fn new_buffer() -> Self::Buffer;
    fn buffer_as_ptr(buf: &mut Self::Buffer) -> *mut u8;
}

/// Size of buffer necessary for serialising any `f64` in scientific notation.
const LEN_EXP: usize = {
    12 + 20
};

/// Size of buffer necessary for serialising any `f64` in decimal notation.
const LEN_DEC: usize = {
    let max_exp = 324usize;
    let decimal_point = 2;
    let mantissa = 20;
    (max_exp + decimal_point + mantissa).next_multiple_of(8)
};

impl Sealed for General {
    type Buffer = [core::mem::MaybeUninit<u8>; LEN_EXP];

    fn new_buffer() -> Self::Buffer {
        [core::mem::MaybeUninit::uninit(); LEN_EXP]
    }

    fn buffer_as_ptr(buf: &mut Self::Buffer) -> *mut u8 {
        buf.as_mut_ptr() as *mut u8
    }
}

impl Sealed for Scientific {
    type Buffer = [core::mem::MaybeUninit<u8>; LEN_EXP];

    fn new_buffer() -> Self::Buffer {
        [core::mem::MaybeUninit::uninit(); LEN_EXP]
    }

    fn buffer_as_ptr(buf: &mut Self::Buffer) -> *mut u8 {
        buf.as_mut_ptr() as *mut u8
    }
}

impl Sealed for Decimal {
    type Buffer = [core::mem::MaybeUninit<u8>; LEN_DEC];

    fn new_buffer() -> Self::Buffer {
        [core::mem::MaybeUninit::uninit(); LEN_DEC]
    }

    fn buffer_as_ptr(buf: &mut Self::Buffer) -> *mut u8 {
        buf.as_mut_ptr() as *mut u8
    }
}
