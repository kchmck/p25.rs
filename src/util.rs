//! Just some utilities.

use num_traits::One;
use std;

/// Calculate ceil(a / b).
pub fn div_ceil<T>(a: T, b: T) -> T where
    T: std::ops::Add<T, Output = T> + std::ops::Sub<T, Output = T> +
       std::ops::Div<T, Output = T> + One + Copy
{
    (a + b - T::one()) / b
}

/// Slice 16 bits from the given bytes (in P25 big endian format.).
pub fn slice_u16(bytes: &[u8]) -> u16 {
    (bytes[0] as u16) << 8 | bytes[1] as u16
}

/// Slice 24 bits from the given bytes (in P25 big endian format.).
pub fn slice_u24(bytes: &[u8]) -> u32 {
    (slice_u16(bytes) as u32) << 8 | bytes[2] as u32
}

/// Slice 32 bits from the given bytes (in P25 big endian format.)
pub fn slice_u32(bytes: &[u8]) -> u32 {
    (slice_u16(bytes) as u32) << 16 | slice_u16(&bytes[2..]) as u32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_div_ceil() {
        assert_eq!(div_ceil(13, 12), 2);
        assert_eq!(div_ceil(1, 2), 1);
        assert_eq!(div_ceil(0, 3), 0);
    }

    #[test]
    fn test_slice_u16() {
        assert_eq!(slice_u16(&[0xDE, 0xAD]), 0xDEAD);
        assert_eq!(slice_u16(&[0xAB, 0xCD, 0xEF]), 0xABCD);
    }

    #[test]
    fn test_slice_u24() {
        assert_eq!(slice_u24(&[0xDE, 0xAD, 0xBE]), 0xDEADBE);
        assert_eq!(slice_u24(&[0xAB, 0xCD, 0xEF, 0x12]), 0xABCDEF);
    }

    #[test]
    fn test_slice_u32() {
        assert_eq!(slice_u32(&[0xDE, 0xAD, 0xBE, 0xEF]), 0xDEADBEEF);
        assert_eq!(slice_u32(&[0xDE, 0xAD, 0xBE, 0xEF, 0x12]), 0xDEADBEEF);
    }
}
