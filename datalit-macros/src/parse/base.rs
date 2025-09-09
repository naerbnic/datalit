//! Basic parsing structures.
//!
//! These can be used as members of other structures to build up more complex parsing.

use proc_macro2::TokenStream;
use syn::{Ident, parse::ParseStream};

use crate::to_bytes::{Endianness, IntType};

pub struct PrimitiveSpec {
    ident: Ident,
    int_type: IntType,
    endianness: Option<Endianness>,
}

impl PrimitiveSpec {
    pub fn int_type(&self) -> IntType {
        self.int_type
    }

    pub fn endianness(&self) -> Option<Endianness> {
        self.endianness
    }
}

impl syn::parse::Parse for PrimitiveSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let ident_string = ident.to_string();
        let mut suffix = ident_string.as_str();

        let endianness = Endianness::parse_from_suffix(&mut suffix);

        let int_type = IntType::from_suffix(suffix).ok_or_else(|| {
            syn::Error::new_spanned(
                &ident,
                format!("Invalid or missing integer type suffix: '{}'", ident),
            )
        })?;

        Ok(PrimitiveSpec {
            ident,
            int_type,
            endianness,
        })
    }
}

impl quote::ToTokens for PrimitiveSpec {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
    }
}
