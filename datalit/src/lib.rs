
#[doc = include_str!("../docs/datalit.md")]
pub use datalit_macros::datalit;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_literals() {
        let bytes = datalit!(1u8, 2u8, 3u8);
        assert_eq!(bytes, &*vec![1u8, 2u8, 3u8]);
    }

    #[test]
    fn test_endian_literals() {
        let bytes = datalit!('bar: 1u16_le, 'foo: { 2u16_be });
        assert_eq!(bytes, &*vec![1u8, 0u8, 0u8, 2u8]);
    }

    #[test]
    fn test_binary_literals() {
        let bytes = datalit!(0b0000_0001_0010_0011_0100_0101_0110_0111_1000_1001);
        assert_eq!(bytes, &*vec![0x01u8, 0x23, 0x45, 0x67, 0x89]);
    }

    #[test]
    fn supports_u24() {
        let bytes = datalit!(0x123456u24_le, 0x789ABCu24_be);
        assert_eq!(bytes, &*vec![0x56u8, 0x34, 0x12, 0x78, 0x9A, 0xBC]);
    }

    #[test]
    fn supports_mode_change() {
        let bytes = datalit!(
            @endian = le,
            // Unspecified endianness uses current mode (little-endian)
            1u16,
            @endian = be,
            // Unspecified endianness uses current mode (little-endian)
            1u16,
            // Specified endianness overrides current mode
            1u16_le,
        );
        assert_eq!(bytes, &*vec![1u8, 0u8, 0u8, 1u8, 1u8, 0u8]);
    }

    #[test]
    fn supports_forward_refs() {
        let bytes = datalit!(
            @endian = le,
            1u8,
            3u8,
            start('next): u16,
            'next: 0x00
        );
        assert_eq!(bytes, &*vec![0x01u8, 0x03, 0x04, 0x00, 0x00]);
    }

    #[test]
    fn supports_align() {
        let bytes = datalit!(
            1u8,
            2u8,
            // Should add 2 bytes of padding
            align(4),
            3u8,
            // Should add 3 bytes of padding
            align(4),
            1_000_000u24,
            // Should add 1 byte of padding
            align(4),
            // Realigning to 4 bytes when already aligned should do nothing
            align(4),
            5u8
        );
        assert_eq!(
            bytes,
            &*vec![
                1u8, 2u8, 0u8, 0u8, 3u8, 0x00, 0x00, 0x00, 64, 66, 15, 0x00, 5u8
            ]
        );
    }

    #[test]
    fn supports_len() {
        assert_eq!(
            datalit!(
                'data: {
                    1u8,
                    12u16,
                },
                len('data): u8,
            ),
            &*vec![1u8, 12u8, 0u8, 3u8]
        )
    }

    #[test]
    fn supports_end() {
        assert_eq!(
            datalit!(
                0xFFFF,
                'data: {
                    1u8,
                    12u16le,
                },
                start('data): u8,
                end('data): u8,
                len('data): u8,
            ),
            &*vec![0xFFu8, 0xFF, 1u8, 12u8, 0u8, 2u8, 5u8, 3u8]
        )
    }

    #[test]
    fn supports_repeat() {
        assert_eq!(
            datalit!(
                // Simple single literal repeat
                [1u8; 3]
            ),
            &*vec![1u8, 1u8, 1u8,]
        );
        assert_eq!(
            datalit!(
                // Compound request
                [{1u8, 2u8}; 2]
            ),
            &*vec![1u8, 2u8, 1u8, 2u8]
        );
        assert_eq!(
            datalit!(
                // Works combined with directives
                [{
                    1u8,
                    // First time through will add 1 byte of padding,
                    // Second time will add nothing since already aligned
                    align(2),
                    2u8
                }; 2]
            ),
            &*vec![1u8, 0u8, 2u8, 1u8, 2u8]
        );
    }

    #[test]
    #[ignore = "syn panics on invalid byte literal"]
    fn test_datalit_macro() {
        datalit!(
            @endian = le,
            // It appears that when syn tries to parse an invalid byte literal,
            // it will panic instead of returning a parse error.
            // b'CAFEBABE',
        );
    }

    // Compile test: Can be used in a constant context
    #[allow(dead_code, reason = "Compile test only")]
    const _DATA: &[u8] = datalit!(
        0xDE, 0xAD, 0xBE, 0xEF,
    );
}
