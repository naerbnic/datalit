use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use syn::{Error, LitByte, LitByteStr, LitCStr, LitInt};

use crate::{
    EntryState,
    to_bytes::{Endianness, IntType, base10_digits_to_bytes},
};

fn new_literal_bytes_stmt(state: &mut EntryState, bytes: &[u8]) -> TokenStream {
    let data_var = state.data_var();
    let byte_literals: Vec<_> = bytes.iter().map(|b| Literal::u8_suffixed(*b)).collect();
    quote! {
        #data_var.extend_from_slice(&[#(#byte_literals),*]);
    }
}

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
    assert_eq!(8 % digits_per_byte, 0);
    if digits.len() % digits_per_byte != 0 {
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
fn parse_int_literal(default_endianness: Endianness, lit: LitInt) -> syn::Result<Vec<u8>> {
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
                &lit,
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
            &lit,
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

    pub fn into_tokens(self, state: &mut EntryState) -> syn::Result<TokenStream> {
        let bytes: Vec<_> = parse_int_literal(state.endian_mode(), self.value)?;
        Ok(new_literal_bytes_stmt(state, &bytes))
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
    pub fn into_tokens(self, state: &mut EntryState) -> syn::Result<TokenStream> {
        let byte = self.value.value();
        Ok(new_literal_bytes_stmt(state, &[byte]))
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

    pub fn into_tokens(self, state: &mut EntryState) -> syn::Result<TokenStream> {
        let bytes = self.value.value();
        Ok(new_literal_bytes_stmt(state, &bytes))
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

    pub fn into_tokens(self, state: &mut EntryState) -> syn::Result<TokenStream> {
        let c_string = self.value.value();
        let bytes = c_string.as_bytes_with_nul();
        Ok(new_literal_bytes_stmt(state, bytes))
    }
}
