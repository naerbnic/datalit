#![doc(hidden)]
#![forbid(unsafe_code)]

use proc_macro2::{Span, TokenStream};

use crate::state::StateOperation as _;
use quote::quote;

mod entry;
mod parse;
mod state;
mod to_bytes;

pub fn generate_data(input: TokenStream) -> syn::Result<Vec<u8>> {
    let entries: entry::SequenceEntry = syn::parse2(input)?;

    let mut state = state::EntryState::new();
    entries.apply_to(&mut state)?;
    state.check()?;
    state.generate_data()
}

pub fn generate_expr(input: TokenStream) -> syn::Result<TokenStream> {
    let byte_array = generate_data(input)?
        .into_iter()
        .map(|b| syn::LitByte::new(b, Span::call_site()));
    Ok(quote! {{
        let __slice: &'static [u8] = &[
            #(#byte_array),*
        ];
        __slice
    }})
}

pub fn generate_expr_raw(input: TokenStream) -> TokenStream {
    generate_expr(input).unwrap_or_else(|e| {
        let errors = e.into_iter().map(syn::Error::into_compile_error);
        quote! {{#(#errors);*}}
    })
}
