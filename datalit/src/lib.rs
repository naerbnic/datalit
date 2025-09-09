#[doc(hidden)]
pub mod support;

pub use datalit_macros::datalit;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_literals() {
        let bytes = datalit!(1u8, 2u8, 3u8);
        assert_eq!(bytes, vec![1u8, 2u8, 3u8]);
    }

    #[test]
    fn test_endian_literals() {
        let bytes = datalit!('bar: 1u16_le, 'foo: { 2u16_be });
        assert_eq!(bytes, vec![1u8, 0u8, 0u8, 2u8]);
    }

    #[test]
    fn test_binary_literals() {
        let bytes = datalit!(0b0000_0001_0010_0011_0100_0101_0110_0111_1000_1001);
        assert_eq!(bytes, vec![0x01u8, 0x23, 0x45, 0x67, 0x89]);
    }

    #[test]
    fn supports_u24() {
        let bytes = datalit!(0x123456u24_le, 0x789ABCu24_be);
        assert_eq!(bytes, vec![0x56u8, 0x34, 0x12, 0x78, 0x9A, 0xBC]);
    }

    #[test]
    fn supports_mode_change() {
        let bytes = datalit!(
            @endian_mode = le,
            // Unspecified endianness uses current mode (little-endian)
            1u16,
            @endian_mode = be,
            // Unspecified endianness uses current mode (little-endian)
            1u16,
            // Specified endianness overrides current mode
            1u16_le,
        );
        assert_eq!(bytes, vec![1u8, 0u8, 0u8, 1u8, 1u8, 0u8]);
    }
}
