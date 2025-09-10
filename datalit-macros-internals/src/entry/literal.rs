use quote::ToTokens;
use syn::{Error, LitByte, LitByteStr, LitCStr, LitInt};

use crate::{
    state::{EntryState, StateOperation},
    to_bytes::{Endianness, IntType, base10_digits_to_bytes},
};

fn parse_byte_literal<T>(
    err_context: &T,
    digit_type_name: &str,
    digits: &str,
    digits_per_byte: usize,
) -> syn::Result<Vec<u8>>
where
    T: ToTokens,
{
    // This should be a valid hex string, which should be in ascii.
    // We can use the byte length to determine how many chars were used.
    // We need an even number of hex digits to form bytes.
    assert!(8usize.is_multiple_of(digits_per_byte),
        "digits_per_byte must divide 8 evenly to unambiguously form bytes");
    if !digits.len().is_multiple_of(digits_per_byte) {
        return Err(Error::new_spanned(
            err_context,
            format!(
                "{} literal must have an even number of digits to form bytes. Has {} digits.",
                digit_type_name,
                digits.len()
            ),
        ));
    }
    (0..digits.len())
        .step_by(digits_per_byte)
        .map(|i| {
            u8::from_str_radix(
                &digits[i..i + digits_per_byte],
                2u32.pow((8 / digits_per_byte) as u32),
            )
        })
        .collect::<std::result::Result<Vec<u8>, _>>()
        .map_err(|e| {
            Error::new_spanned(
                err_context,
                format!("Invalid {} literal: {e}", digit_type_name.to_lowercase()),
            )
        })
}

fn parse_int_literal(default_endianness: Endianness, lit: &LitInt) -> syn::Result<Vec<u8>> {
    let mut suffix = lit.suffix();

    if suffix.is_empty() {
        // Check to see if the representation is a hexidecimal literal
        let literal_digits = lit.to_string().to_ascii_lowercase();
        if literal_digits.starts_with("0x") {
            // This should be a valid hex string, which should be in ascii.
            // We can use the byte length to determine how many chars were used.
            // We need an even number of hex digits to form bytes.
            let hex_digits = literal_digits.trim_start_matches("0x").replace('_', "");
            return parse_byte_literal(&lit, "Hex", &hex_digits, 2);
        } else if literal_digits.starts_with("0b") {
            let bin_digits = literal_digits.trim_start_matches("0b").replace('_', "");
            return parse_byte_literal(&lit, "Binary", &bin_digits, 8);
        } else {
            return Err(Error::new_spanned(
                lit,
                "Integer literal must have a type suffix (e.g. 'u8', 'i32', etc.) or be a hex (0x...) or binary (0b...) literal",
            ));
        }
    }

    let endianness = if suffix.ends_with("le") {
        suffix = suffix.trim_end_matches("le");
        suffix = suffix.trim_end_matches('_');
        Endianness::Little
    } else if suffix.ends_with("be") {
        suffix = suffix.trim_end_matches("be");
        suffix = suffix.trim_end_matches('_');
        Endianness::Big
    } else if suffix.ends_with("ne") {
        suffix = suffix.trim_end_matches("ne");
        suffix = suffix.trim_end_matches('_');
        Endianness::Native
    } else {
        default_endianness
    };

    let int_type = IntType::from_suffix(suffix).ok_or_else(|| {
        Error::new_spanned(
            lit,
            format!("Invalid or missing integer type suffix: '{}'", lit.suffix()),
        )
    })?;

    base10_digits_to_bytes(lit.base10_digits(), int_type, endianness)
}

#[derive(derive_syn_parse::Parse)]
pub struct IntLiteral {
    value: LitInt,
}

impl IntLiteral {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(LitInt)
    }
}

impl StateOperation for IntLiteral {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        let bytes: Vec<_> = parse_int_literal(state.endian_mode(), &self.value)?;
        state.append_bytes(&bytes);
        Ok(())
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct ByteLiteral {
    value: LitByte,
}

impl ByteLiteral {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(LitByte)
    }
}

impl StateOperation for ByteLiteral {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        let byte = self.value.value();
        state.append_bytes(&[byte]);
        Ok(())
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct ByteStringLiteral {
    value: LitByteStr,
}

impl ByteStringLiteral {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(LitByteStr)
    }
}

impl StateOperation for ByteStringLiteral {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        state.append_bytes(&self.value.value());
        Ok(())
    }
}

#[derive(derive_syn_parse::Parse)]
pub struct CStringLiteral {
    value: LitCStr,
}

impl CStringLiteral {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        input.peek(LitCStr)
    }
}

impl StateOperation for CStringLiteral {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        state.append_bytes(self.value.value().as_bytes_with_nul());
        Ok(())
    }
}

#[derive(derive_syn_parse::Parse)]
pub enum LiteralEntry {
    #[peek_with(IntLiteral::peek, name = "integer literal")]
    Int(IntLiteral),
    #[peek_with(ByteLiteral::peek, name = "byte literal")]
    Byte(ByteLiteral),
    #[peek_with(ByteStringLiteral::peek, name = "byte string literal")]
    ByteString(ByteStringLiteral),
    #[peek_with(CStringLiteral::peek, name = "C string literal")]
    CString(CStringLiteral),
}

impl LiteralEntry {
    pub fn peek(input: syn::parse::ParseStream) -> bool {
        IntLiteral::peek(input)
            || ByteLiteral::peek(input)
            || ByteStringLiteral::peek(input)
            || CStringLiteral::peek(input)
    }
}

impl StateOperation for LiteralEntry {
    fn apply_to(&self, state: &mut EntryState) -> syn::Result<()> {
        match self {
            LiteralEntry::Int(int_lit) => int_lit.apply_to(state),
            LiteralEntry::Byte(byte_lit) => byte_lit.apply_to(state),
            LiteralEntry::ByteString(byte_str_lit) => byte_str_lit.apply_to(state),
            LiteralEntry::CString(cstr_lit) => cstr_lit.apply_to(state),
        }
    }
}
