use std::str::FromStr;

use num::ToPrimitive as _;

fn consume_suffix(suffix: &mut &str, to_consume: &str) -> bool {
    if suffix.ends_with(to_consume) {
        let trimmed = suffix.trim_end_matches(to_consume);
        let trimmed = trimmed.trim_end_matches('_');
        *suffix = trimmed;
        true
    } else {
        false
    }
}

const _: () = {
    assert!(std::mem::size_of::<usize>() <= std::mem::size_of::<u64>());
};

#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    Little,
    Big,
    Native,
}

impl Endianness {
    pub fn parse_from_suffix(suffix: &mut &str) -> Option<Self> {
        if consume_suffix(suffix, "le") {
            Some(Endianness::Little)
        } else if consume_suffix(suffix, "be") {
            Some(Endianness::Big)
        } else if consume_suffix(suffix, "ne") {
            Some(Endianness::Native)
        } else {
            None
        }
    }

    pub fn to_bytes<T>(self, number: T) -> T::Bytes
    where
        T: num::traits::ToBytes,
    {
        match self {
            Endianness::Little => number.to_le_bytes(),
            Endianness::Big => number.to_be_bytes(),
            Endianness::Native => number.to_ne_bytes(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Value out of range for u24 (must be 0..=16777215)")]
pub struct InvalidU24Error;

struct U24(u32);

impl U24 {
    pub fn new(value: u32) -> Result<Self, InvalidU24Error> {
        if value > 0xFFFFFF {
            return Err(InvalidU24Error);
        }
        assert!(value <= 0xFFFFFF);
        Ok(U24(value))
    }
}

impl FromStr for U24 {
    type Err = InvalidU24Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u32 = s.parse().map_err(|_| InvalidU24Error)?;
        U24::new(value)
    }
}

impl TryFrom<&num::BigInt> for U24 {
    type Error = InvalidU24Error;

    fn try_from(value: &num::BigInt) -> Result<Self, Self::Error> {
        let u32_value: u32 = value.to_u32().ok_or(InvalidU24Error)?;
        U24::new(u32_value)
    }
}

impl num::traits::ToBytes for U24 {
    type Bytes = [u8; 3];

    fn to_be_bytes(&self) -> Self::Bytes {
        self.0.to_be_bytes()[1..4].try_into().unwrap()
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        self.0.to_le_bytes()[0..3].try_into().unwrap()
    }
}

struct I24(i32);

impl I24 {
    const MIN: I24 = I24(-(2i32.pow(23))); // -2^23
    const MAX: I24 = I24(2i32.pow(23) - 1); // 2^23 - 1

    pub fn new(value: i32) -> Result<Self, InvalidU24Error> {
        if !(I24::MIN.0..=I24::MAX.0).contains(&value) {
            return Err(InvalidU24Error);
        }
        Ok(I24(value))
    }
}

impl FromStr for I24 {
    type Err = InvalidU24Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: i32 = s.parse().map_err(|_| InvalidU24Error)?;
        I24::new(value)
    }
}

impl TryFrom<&num::BigInt> for I24 {
    type Error = InvalidU24Error;

    fn try_from(value: &num::BigInt) -> Result<Self, Self::Error> {
        let i32_value: i32 = value.to_i32().ok_or(InvalidU24Error)?;
        I24::new(i32_value)
    }
}

impl num::traits::ToBytes for I24 {
    type Bytes = [u8; 3];

    fn to_be_bytes(&self) -> Self::Bytes {
        self.0.to_be_bytes()[1..4].try_into().unwrap()
    }

    fn to_le_bytes(&self) -> Self::Bytes {
        self.0.to_le_bytes()[0..3].try_into().unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IntType {
    U8,
    U16,
    U24,
    U32,
    U64,
    USize,
    I8,
    I16,
    I24,
    I32,
    I64,
    ISize,
}

impl IntType {
    pub fn from_suffix(suffix: &str) -> Option<Self> {
        match suffix {
            "u8" => Some(IntType::U8),
            "u16" => Some(IntType::U16),
            "u24" => Some(IntType::U24),
            "u32" => Some(IntType::U32),
            "u64" => Some(IntType::U64),
            "usize" => Some(IntType::USize),
            "i8" => Some(IntType::I8),
            "i16" => Some(IntType::I16),
            "i24" => Some(IntType::I24),
            "i32" => Some(IntType::I32),
            "i64" => Some(IntType::I64),
            "isize" => Some(IntType::ISize),
            _ => None,
        }
    }

    pub fn num_bytes(self) -> usize {
        match self {
            IntType::U8 | IntType::I8 => 1,
            IntType::U16 | IntType::I16 => 2,
            IntType::I24 | IntType::U24 => 3,
            IntType::U32 | IntType::I32 => 4,
            IntType::U64 | IntType::I64 => 8,
            IntType::USize | IntType::ISize => std::mem::size_of::<usize>(),
        }
    }

    pub fn write_bytes_from_bigint(
        self,
        n: &num::BigInt,
        endianness: Endianness,
        dest: &mut [u8],
    ) -> syn::Result<()> {
        macro_rules! impl_for {
            ($t:ty) => {{
                let value: $t = <$t as TryFrom<&num::BigInt>>::try_from(n).map_err(|_| {
                    syn::Error::new(
                        proc_macro2::Span::call_site(),
                        format!("Value {} out of range for {}", n, stringify!($t)),
                    )
                })?;
                let bytes = endianness.to_bytes(value);
                dest[..bytes.len()].copy_from_slice(&bytes);
                Ok(())
            }};
        }

        match self {
            IntType::U8 => impl_for!(u8),
            IntType::U16 => impl_for!(u16),
            IntType::U24 => impl_for!(U24),
            IntType::U32 => impl_for!(u32),
            IntType::U64 => impl_for!(u64),
            IntType::USize => impl_for!(usize),
            IntType::I8 => impl_for!(i8),
            IntType::I16 => impl_for!(i16),
            IntType::I24 => impl_for!(I24),
            IntType::I32 => impl_for!(i32),
            IntType::I64 => impl_for!(i64),
            IntType::ISize => impl_for!(isize),
        }
    }
}

pub fn base10_digits_to_bytes(
    digits: &str,
    int_type: IntType,
    endianness: Endianness,
) -> syn::Result<Vec<u8>> {
    macro_rules! parse_int {
        ($t:ty, $digits:expr) => {{
            let value: $t = $digits.parse().map_err(|e| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Failed to parse {}: {e}", stringify!($t)),
                )
            })?;
            Ok(endianness.to_bytes(value).to_vec())
        }};
    }
    match int_type {
        IntType::U8 => parse_int!(u8, digits),
        IntType::U16 => parse_int!(u16, digits),
        IntType::U24 => parse_int!(U24, digits),
        IntType::U32 => parse_int!(u32, digits),
        IntType::U64 => parse_int!(u64, digits),
        IntType::USize => parse_int!(usize, digits),
        IntType::I8 => parse_int!(i8, digits),
        IntType::I16 => parse_int!(i16, digits),
        IntType::I24 => parse_int!(I24, digits),
        IntType::I32 => parse_int!(i32, digits),
        IntType::I64 => parse_int!(i64, digits),
        IntType::ISize => parse_int!(isize, digits),
    }
}
