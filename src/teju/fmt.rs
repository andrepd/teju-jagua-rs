//! Routines for actually formatting the numbers as strings.

use core::mem::MaybeUninit;

const DIGITS_LUT: &[u8; 200] = 
    b"00010203040506070809\
      10111213141516171819\
      20212223242526272829\
      30313233343536373839\
      40414243444546474849\
      50515253545556575859\
      60616263646566676869\
      70717273747576777879\
      80818283848586878889\
      90919293949596979899";

#[inline]
unsafe fn write_lut_u64(buf: *mut u8, offset: usize, lo: u64, hi: u64) {
    unsafe {
        let digits = DIGITS_LUT.as_ptr().add((lo * 2 - hi * 200) as usize);
        core::ptr::copy_nonoverlapping(digits, buf.add(offset), 2);
    }
}

/// Number of digits of `x`. Invariant: `x` has at most 17 digits.
pub fn len_u64(x: u64) -> usize {
    debug_assert!(x < 10u64.pow(17));
    // Hypothesis: the average output length among all `f64`s is 16.38 digits, so high-to-low is
    // likelier to get well predicted.
    if x >= 10000000000000000 {
        17
    } else if x >= 1000000000000000 {
        16
    } else if x >= 100000000000000 {
        15
    } else if x >= 10000000000000 {
        14
    } else if x >= 1000000000000 {
        13
    } else if x >= 100000000000 {
        12
    } else if x >= 10000000000 {
        11
    } else if x >= 1000000000 {
        10
    } else if x >= 100000000 {
        9
    } else if x >= 10000000 {
        8
    } else if x >= 1000000 {
        7
    } else if x >= 100000 {
        6
    } else if x >= 10000 {
        5
    } else if x >= 1000 {
        4
    } else if x >= 100 {
        3
    } else if x >= 10 {
        2
    } else {
        1
    }
}

/// Print an `u64`, returning the number of bytes written. Invariant: `x` has at most 17 digits.
/// May clobber / write junk to `buf` after the bytes written (up to 20 bytes in total).
///
/// Performance should be better than `len_u64` + `print_u64_mantissa_known_len` if `len_u64` is
/// not well predicted, and worse if not (but microbenchmarks don't show this!).
#[inline]
#[allow(unused)]
pub unsafe fn print_u64_mantissa(x: u64, buf: *mut u8) -> usize {
    debug_assert!(x < 10u64.pow(17));
    unsafe {
        if x == 0 {
            *buf = b'0';
            return 1
        }

        let mut digits = [MaybeUninit::<u8>::uninit(); 20];
        let digits_ptr = digits.as_mut_ptr() as *mut u8;
        let top12 = x / 100000000u64;
        let top4 = x / 10000000000000000u64;
        /*let top4 = 0u64;*/

        let uvalue_0 = x - top12 * 100000000;
        let uvalue_1 = (uvalue_0 * 1374389535u64) >> 37;
        let uvalue_2 = (uvalue_0 * 3518437209u64) >> 45;
        let uvalue_3 = (uvalue_0 * 1125899907u64) >> 50;

        write_lut_u64(digits_ptr, 18, uvalue_0, uvalue_1);
        write_lut_u64(digits_ptr, 16, uvalue_1, uvalue_2);
        write_lut_u64(digits_ptr, 14, uvalue_2, uvalue_3);
        write_lut_u64(digits_ptr, 12, uvalue_3, 0);

        let uvalue_4 = top12 - top4 * 100000000;
        let uvalue_5 = (uvalue_4 * 1374389535u64) >> 37;
        let uvalue_6 = (uvalue_4 * 3518437209u64) >> 45;
        let uvalue_7 = (uvalue_4 * 1125899907u64) >> 50;

        write_lut_u64(digits_ptr, 10, uvalue_4, uvalue_5);
        write_lut_u64(digits_ptr,  8, uvalue_5, uvalue_6);
        write_lut_u64(digits_ptr,  6, uvalue_6, uvalue_7);
        write_lut_u64(digits_ptr,  4, uvalue_7, 0);

        let uvalue_8 = top4;
        let uvalue_9 = (uvalue_8 * 1374389535u64) >> 37;

        write_lut_u64(digits_ptr,  2, uvalue_8, uvalue_9);
        write_lut_u64(digits_ptr,  0, uvalue_9, 0);
        /*digits_ptr.add(3) = b'0' + uvalue_8 as u8;*/

        // No need to calculate the `len_u64` beforehand, this is an arithmetic way to get that
        // value.
        let neg_log2 = x.leading_zeros() as usize;
        let offset = neg_log2 * 1233 >> 12; // 1233 / 2**12 ≈ log10(2)
        let offset = offset + (*digits_ptr.add(offset) == b'0') as usize;

        core::ptr::copy_nonoverlapping(digits_ptr.add(offset), buf, 20);
        return 20 - offset;
    }
}

/// Print an `u64` with `len` digits, returning the number of bytes written. Invariant: `x` has at
/// most 17 digits. May clobber / write junk to `buf` after the bytes written (up to 20 bytes in
/// total).
#[inline]
#[allow(unused)]
pub unsafe fn print_u64_mantissa_known_len(x: u64, buf: *mut u8, len: usize) -> usize {
    debug_assert!(x < 10u64.pow(17));
    debug_assert!(len <= 17);
    unsafe {
        if x == 0 {
            *buf = b'0';
            return 1
        }

        // TODO build directly in buf? But then we'd have to branch
        let mut digits = [MaybeUninit::<u8>::uninit(); 20];
        let digits_ptr = digits.as_mut_ptr() as *mut u8;
        let top12 = x / 100000000u64;
        let top4 = x / 10000000000000000u64;

        let uvalue_0 = x - top12 * 100000000;
        let uvalue_1 = (uvalue_0 * 1374389535u64) >> 37;
        let uvalue_2 = (uvalue_0 * 3518437209u64) >> 45;
        let uvalue_3 = (uvalue_0 * 1125899907u64) >> 50;

        write_lut_u64(digits_ptr, 18, uvalue_0, uvalue_1);
        write_lut_u64(digits_ptr, 16, uvalue_1, uvalue_2);
        write_lut_u64(digits_ptr, 14, uvalue_2, uvalue_3);
        write_lut_u64(digits_ptr, 12, uvalue_3, 0);

        let uvalue_4 = top12 - top4 * 100000000;
        let uvalue_5 = (uvalue_4 * 1374389535u64) >> 37;
        let uvalue_6 = (uvalue_4 * 3518437209u64) >> 45;
        let uvalue_7 = (uvalue_4 * 1125899907u64) >> 50;

        write_lut_u64(digits_ptr, 10, uvalue_4, uvalue_5);
        write_lut_u64(digits_ptr,  8, uvalue_5, uvalue_6);
        write_lut_u64(digits_ptr,  6, uvalue_6, uvalue_7);
        write_lut_u64(digits_ptr,  4, uvalue_7, 0);

        let uvalue_8 = top4;
        /*debug_assert!(uvalue_8 <= 9);
        write_lut_u64(digits_ptr,  2, uvalue_8, 0);*/

        /*let uvalue_9 = (uvalue_8 * 1374389535u64) >> 37;

        write_lut_u64(digits_ptr,  2, uvalue_8, uvalue_9);
        write_lut_u64(digits_ptr,  0, uvalue_9, 0);*/
        *digits_ptr.add(3) = b'0' + uvalue_8 as u8;

        let offset = 20 - len;

        core::ptr::copy_nonoverlapping(digits_ptr.add(offset), buf, 20);
        return len;
    }
}

#[inline]
pub unsafe fn print_i32_exp(x: i32, buf: *mut u8) -> usize {
    // Invariant: never more than 4 digits
    debug_assert!(-999 <= x && x <= 999);

    unsafe {
        let sign = x >= 0;
        let x_abs = if sign {x} else {-x};

        *buf = b'-';
        let buf = buf.add(!sign as usize);

        if x_abs >= 100 {
            *buf = b'0' + (x_abs / 100) as u8;
            let d = DIGITS_LUT.as_ptr().add(x_abs as usize % 100 * 2);
            core::ptr::copy_nonoverlapping(d, buf.offset(1), 2);
            !sign as usize + 3
        } else if x_abs >= 10 {
            let d = DIGITS_LUT.as_ptr().add(x_abs as usize * 2);
            core::ptr::copy_nonoverlapping(d, buf, 2);
            !sign as usize + 2
        } else {
            *buf = b'0' + x_abs as u8;
            !sign as usize + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_mantissa() {
        let mut buf = [0u8; 80];

        let n = unsafe { print_u64_mantissa(1234, buf.as_mut_ptr()) };
        assert_eq!(&buf[..n], b"1234");
        
        let n = unsafe { print_u64_mantissa(0, buf.as_mut_ptr()) };
        assert_eq!(&buf[..n], b"0");
        
        let n = unsafe { print_u64_mantissa(1, buf.as_mut_ptr()) };
        assert_eq!(&buf[..n], b"1");
        
        let n = unsafe { print_u64_mantissa(9, buf.as_mut_ptr()) };
        assert_eq!(&buf[..n], b"9");
        
        let n = unsafe { print_u64_mantissa(10, buf.as_mut_ptr()) };
        assert_eq!(&buf[..n], b"10");
        
        let n = unsafe { print_u64_mantissa(061295, buf.as_mut_ptr()) };
        assert_eq!(&buf[..n], b"61295");
        
        let n = unsafe { print_u64_mantissa(99_999_999_999_999_999, buf.as_mut_ptr()) };
        assert_eq!(&buf[..n], b"99999999999999999");
    }

    #[test]
    fn test_u64_mantissa_known_len() {
        let mut buf = [0u8; 80];

        let n = unsafe { print_u64_mantissa_known_len(1234, buf.as_mut_ptr(), 4) };
        assert_eq!(&buf[..n], b"1234");
        
        let n = unsafe { print_u64_mantissa_known_len(0, buf.as_mut_ptr(), 1) };
        assert_eq!(&buf[..n], b"0");
        
        let n = unsafe { print_u64_mantissa_known_len(1, buf.as_mut_ptr(), 1) };
        assert_eq!(&buf[..n], b"1");
        
        let n = unsafe { print_u64_mantissa_known_len(9, buf.as_mut_ptr(), 1) };
        assert_eq!(&buf[..n], b"9");
        
        let n = unsafe { print_u64_mantissa_known_len(10, buf.as_mut_ptr(), 2) };
        assert_eq!(&buf[..n], b"10");
        
        let n = unsafe { print_u64_mantissa_known_len(061295, buf.as_mut_ptr(), 5) };
        assert_eq!(&buf[..n], b"61295");
        
        let n = unsafe { print_u64_mantissa_known_len(99_999_999_999_999_999, buf.as_mut_ptr(), 17) };
        assert_eq!(&buf[..n], b"99999999999999999");
    }

    #[test]
    fn test_i32_exp() {
        let mut buf = [0u8; 80];

        for x in -999 ..= 999 {
            let len = unsafe { print_i32_exp(x, buf.as_mut_ptr()) };
            let std = format!("{x}");
            assert_eq!(&buf[..len], std.as_bytes())
        }
    }

    use proptest::prelude::*;
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200_000))]
        
        #[test]
        fn proptest_u64_mantissa(x in 0u64 .. 1 << 54) {
            let mut buf = [0u8; 80];
            let len = unsafe { print_u64_mantissa(x, buf.as_mut_ptr()) };
            let std = format!("{x}");
            assert_eq!(&buf[..len], std.as_bytes())
        }
        
        #[test]
        fn proptest_u64_mantissa_known_len(x in 0u64 .. 1 << 54) {
            let mut buf = [0u8; 80];
            let std = format!("{x}");
            let len = unsafe { print_u64_mantissa_known_len(x, buf.as_mut_ptr(), std.len()) };
            assert_eq!(len, std.len());
            assert_eq!(&buf[..len], std.as_bytes())
        }
    }
}
